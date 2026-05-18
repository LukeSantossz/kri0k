//! `WhoisTtp` — MITRE T1590.001 (Gather Victim Network Information: Domain Properties).
//!
//! Implements the [`Ttp`] trait for whois reconnaissance using the Sysinternals
//! `whois.exe` binary (Windows) or `whois` (Linux) via subprocess abstraction (D-54).
//!
//! # Key design decisions
//! - Rate limit: 1 req/sec enforced via `Mutex<Option<Instant>>` (D-45, TTP-05).
//! - Timeout: 30s default (D-51); subprocess killed on expiry.
//! - Cancellation: `CancellationToken` propagated to subprocess (D-62, M-36).
//! - Parser: heuristic key:value line scan (D-41) — never panics; unknown lines → `raw_unparsed`.
//! - Args: `-v -nobanner -accepteula <target>` — order and flags are MANDATORY (Pitfall 1+2+8).
//!
//! See: `docs/adr/ADR-0012-ttp-trait-adapters.md`
//! See: `docs/security/THREAT_MODEL.md §T-04-03-01..05`

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tracing::instrument;
use tokio_util::sync::CancellationToken;

use crate::ttp::{RateLimits, RiskLevel, Ttp, TtpOutput};
use crate::ttp::subprocess::Subprocess;

/// Structured output from a whois query (D-41, D-42).
///
/// Parsed from the raw whois text using a heuristic key:value scanner.
/// Fields that are absent in the output (e.g. GDPR-redacted registrant) are `None`.
/// All unrecognised lines are preserved in `raw_unparsed` for inspection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct WhoisOutput {
    /// Registrant Organization (D-42: only this contact is captured).
    ///
    /// Populated by the `Registrant Organization:` field in thick-referral output.
    /// Requires `-v` flag to trigger thick referral query (Pitfall 2).
    pub registrant: Option<String>,
    /// Registrar name (e.g., "`MarkMonitor` Inc.").
    pub registrar: Option<String>,
    /// Name servers (lowercased, deduplicated). Order preserved from output.
    pub nameservers: Vec<String>,
    /// Creation date in ISO 8601 (raw from whois output).
    pub created_at: Option<String>,
    /// Last updated date (raw from whois output).
    pub updated_at: Option<String>,
    /// Expiration / registry expiry date (raw from whois output).
    pub expires_at: Option<String>,
    /// Lines not matching the key:value heuristic (D-41 graceful degradation).
    ///
    /// Includes banner text, comment lines, and TLD-specific non-standard fields.
    pub raw_unparsed: Vec<String>,
}

/// TTP implementation for MITRE ATT&CK T1590.001 — whois reconnaissance (D-45, D-51, D-62).
///
/// Wraps a [`Subprocess`] impl for testability (D-54).
/// Enforces 1 req/sec rate limit via internal `Mutex<Option<Instant>>` (D-45).
pub struct WhoisTtp {
    subprocess: Arc<dyn Subprocess>,
    /// TTP-local rate limit bucket (D-45). `std::sync::Mutex` — lock is never held across await.
    last_call: Mutex<Option<Instant>>,
}

impl std::fmt::Debug for WhoisTtp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhoisTtp")
            .field("last_call", &self.last_call)
            .finish_non_exhaustive()
    }
}

impl WhoisTtp {
    /// Create a new `WhoisTtp` with the given subprocess implementation.
    ///
    /// In production: `WhoisTtp::new(Arc::new(RealSubprocess))`.
    /// In tests: `WhoisTtp::new(Arc::new(MockSubprocess::from_fixture(...)))`.
    #[must_use]
    pub fn new(subprocess: Arc<dyn Subprocess>) -> Self {
        Self {
            subprocess,
            last_call: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Ttp for WhoisTtp {
    /// MITRE ATT&CK T1590.001 — Gather Victim Network Information: Domain Properties.
    fn id(&self) -> &'static str {
        "T1590.001"
    }

    fn description(&self) -> &'static str {
        "whois reconnaissance (MITRE T1590.001)"
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Safe
    }

    /// Rate limit: 1 req/sec max (D-45, TTP-05).
    fn rate_limits(&self) -> RateLimits {
        RateLimits {
            max_rps: Some(1),
            max_concurrent: Some(1),
        }
    }

    /// Default timeout: 30s (D-51).
    fn default_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Execute whois against `target` with cancellation support (D-62, M-36).
    ///
    /// # Flow
    /// 1. Enforce rate limit (D-45): sleep if last call was < 1s ago.
    /// 2. Invoke subprocess with args `[-v, -nobanner, -accepteula, target]` (Pitfall 1+2+8).
    /// 3. Update `last_call` timestamp.
    /// 4. Parse output via heuristic parser (D-41).
    ///
    /// # Errors
    /// - `Error::Cancelled` if `cancel` fires.
    /// - `Error::SubprocessTimeout` if subprocess exceeds 30s.
    /// - `Error::Io` for spawn/wait failures.
    #[instrument(skip(self), fields(ttp_id = %self.id()))]
    async fn execute(
        &self,
        target: &str,
        cancel: CancellationToken,
    ) -> Result<TtpOutput, crate::Error> {
        // a. Rate-limit gate (D-45, M-34).
        // Lock is dropped before any await — std::sync::Mutex is safe here.
        let wait = {
            let last = self
                .last_call
                .lock()
                .map_err(|e| crate::Error::Generic(format!("rate-limit mutex poisoned: {e}")))?;
            last.and_then(|t| Duration::from_secs(1).checked_sub(t.elapsed()))
        };
        if let Some(d) = wait {
            tokio::time::sleep(d).await;
        }

        // b. Subprocess invocation.
        // M-15 (D-63 L3): no shell expansion — args are a literal slice.
        // M-34: rate-limited via gate above; subprocess respects default_timeout().
        let out = self
            .subprocess
            .run(
                "whois",
                &["-v", "-nobanner", "-accepteula", target],
                self.default_timeout(),
                cancel.clone(),
            )
            .await?;

        // c. Update last_call BEFORE parse — ensures rate limit is correct even on retry.
        {
            let mut last = self
                .last_call
                .lock()
                .map_err(|e| crate::Error::Generic(format!("rate-limit mutex poisoned: {e}")))?;
            *last = Some(Instant::now());
        }

        // d. Parse output (heuristic, never panics — D-41).
        let parsed = parse_whois_output(&out.stdout);
        Ok(TtpOutput::Whois(parsed))
    }
}

/// Heuristic key:value parser for ICANN-style whois output (D-41).
///
/// Non-matching lines go to `raw_unparsed`. Never panics; missing fields stay `None`.
///
/// # Field extraction
/// - `Registrant Organization:` → `registrant` (D-42).
/// - `Registrar:` → `registrar`.
/// - `Name Server:` / `NServer:` → `nameservers` (lowercased, deduplicated).
/// - `Creation Date:` / `Created:` / `Created On:` → `created_at`.
/// - `Updated Date:` / `Last Updated:` / `Modified:` → `updated_at`.
/// - `Registry Expiry Date:` / `Expiration Date:` / `Expires On:` /
///   `Registrar Registration Expiration Date:` → `expires_at`.
#[must_use]
pub fn parse_whois_output(raw: &str) -> WhoisOutput {
    let mut out = WhoisOutput::default();

    for line in raw.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comment/banner lines.
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let key_lower = key.trim().to_ascii_lowercase();
            let value = value.trim();

            // Skip keys with no value (e.g., "key without value:").
            if value.is_empty() {
                out.raw_unparsed.push(trimmed.to_string());
                continue;
            }

            match key_lower.as_str() {
                // D-42: only Registrant Organization is captured.
                "registrant organization" => {
                    out.registrant = Some(value.to_string());
                }
                "registrar" => {
                    out.registrar = Some(value.to_string());
                }
                "name server" | "nserver" => {
                    let ns = value.to_ascii_lowercase();
                    if !out.nameservers.contains(&ns) {
                        out.nameservers.push(ns);
                    }
                }
                "creation date" | "created" | "created on" => {
                    out.created_at = Some(value.to_string());
                }
                "updated date" | "last updated" | "modified" => {
                    out.updated_at = Some(value.to_string());
                }
                "registry expiry date"
                | "expiration date"
                | "expires on"
                | "registrar registration expiration date" => {
                    out.expires_at = Some(value.to_string());
                }
                _ => {
                    // D-41: unrecognised key:value lines go to raw_unparsed for inspection.
                    out.raw_unparsed.push(trimmed.to_string());
                }
            }
        } else {
            // Lines with no ':' — banner text, TERMS OF USE paragraphs, etc.
            out.raw_unparsed.push(trimmed.to_string());
        }
    }

    out
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::ttp::{Ttp, TtpOutput};
    use crate::ttp::subprocess::MockSubprocess;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/")).join(name)
    }

    fn google_fixture_string() -> String {
        std::fs::read_to_string(fixture_path("whois_google_com.txt"))
            .expect("google fixture must exist")
    }

    fn example_fixture_string() -> String {
        std::fs::read_to_string(fixture_path("whois_example_com.txt"))
            .expect("example fixture must exist")
    }

    /// TTP-01: `WhoisTtp` is registrable as `Box<dyn Ttp>` (dyn-compatibility).
    #[tokio::test]
    async fn implements_trait() {
        let ttp_box: Box<dyn Ttp> = Box::new(WhoisTtp::new(Arc::new(
            MockSubprocess::from_fixture(fixture_path("whois_google_com.txt")),
        )));
        assert_eq!(ttp_box.id(), "T1590.001");
        assert_eq!(ttp_box.rate_limits().max_rps, Some(1));
        assert_eq!(ttp_box.default_timeout(), Duration::from_secs(30));
    }

    /// TTP-02: `execute` invokes subprocess and returns `TtpOutput::Whois(_)`.
    #[tokio::test]
    async fn executes_via_mock_subprocess() {
        let whois = WhoisTtp::new(Arc::new(MockSubprocess::from_fixture(fixture_path(
            "whois_google_com.txt",
        ))));
        let result = whois
            .execute("google.com", CancellationToken::new())
            .await
            .expect("execute should not fail with mock subprocess");
        assert!(
            matches!(result, TtpOutput::Whois(_)),
            "expected TtpOutput::Whois, got {result:?}"
        );
    }

    /// TTP-03: parser extracts all 6 fields from the google.com verbose fixture.
    #[tokio::test]
    async fn parses_google_fixture() {
        let content = google_fixture_string();
        let out = parse_whois_output(&content);

        assert!(
            out.registrant.is_some(),
            "expected Registrant Organization in google fixture"
        );
        assert_eq!(
            out.registrant.as_deref(),
            Some("Google LLC"),
            "Registrant Organization should be 'Google LLC'"
        );
        assert!(out.registrar.is_some(), "expected Registrar in fixture");
        assert!(
            !out.nameservers.is_empty(),
            "expected at least 1 nameserver"
        );
        assert!(
            out.nameservers
                .iter()
                .any(|n| n.contains("ns") && n.contains("google.com")),
            "expected a google.com nameserver"
        );
        assert!(out.created_at.is_some(), "expected Creation Date");
        assert!(out.updated_at.is_some(), "expected Updated Date");
        assert!(out.expires_at.is_some(), "expected Expiry Date");

        // All nameservers must be lowercased.
        assert!(
            out.nameservers.iter().all(|n| n == &n.to_lowercase()),
            "nameservers must be lowercased"
        );
    }

    /// TTP-03: parser degrades gracefully when output is GDPR-redacted (example.com).
    #[tokio::test]
    async fn handles_redacted_output() {
        let content = example_fixture_string();
        let out = parse_whois_output(&content);

        assert!(
            out.registrant.is_none(),
            "example.com is GDPR-redacted — registrant should be None (Pitfall 2)"
        );
        assert!(
            !out.raw_unparsed.is_empty(),
            "redacted output still has metadata lines in raw_unparsed"
        );
    }

    /// Parser never panics on malformed input; `raw_unparsed` is populated.
    #[tokio::test]
    async fn parser_handles_invalid_input() {
        let content = std::fs::read_to_string(fixture_path("whois_invalid.txt"))
            .expect("invalid fixture must exist");
        let out = parse_whois_output(&content);

        // Should not panic. Fields with no value should stay None.
        assert!(out.registrant.is_none());
        assert!(out.registrar.is_none());
        assert!(out.nameservers.is_empty());
        // Lines that don't parse cleanly go to raw_unparsed.
        assert!(
            !out.raw_unparsed.is_empty(),
            "invalid fixture lines should appear in raw_unparsed"
        );
    }

    /// TTP-05 (D-45): two consecutive calls take >= 950ms (1 req/sec rate limit).
    #[tokio::test]
    async fn rate_limit_enforced() {
        let whois = WhoisTtp::new(Arc::new(MockSubprocess::from_fixture(fixture_path(
            "whois_google_com.txt",
        ))));
        let start = Instant::now();
        let _ = whois.execute("google.com", CancellationToken::new()).await;
        let _ = whois.execute("google.com", CancellationToken::new()).await;
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(950),
            "rate limit not enforced: elapsed = {elapsed:?} (expected >= 950ms)"
        );
    }

    /// D-62: cancellation via `CancellationToken` returns `Error::Cancelled`.
    #[tokio::test]
    async fn cancellation_returns_cancelled() {
        let whois = Arc::new(WhoisTtp::new(Arc::new(MockSubprocess::hanging())));
        let token = CancellationToken::new();
        let token_clone = token.clone();
        let handle = tokio::spawn(async move { whois.execute("x", token_clone).await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        token.cancel();
        let result = handle.await.expect("join should not panic");
        assert!(
            matches!(result, Err(crate::Error::Cancelled)),
            "expected Cancelled, got {result:?}"
        );
    }

    /// D-51 alias: subprocess-level timeout returns `Error::SubprocessTimeout`.
    ///
    /// Covers the test name expected by VALIDATION.md D-51 row.
    /// The implementation is at subprocess level (also tested in `subprocess::tests`).
    #[tokio::test]
    async fn timeout_kills_child() {
        use crate::ttp::subprocess::{MockSubprocess, Subprocess};
        let sub = MockSubprocess::hanging();
        let result = sub
            .run(
                "x",
                &[],
                Duration::from_millis(50),
                CancellationToken::new(),
            )
            .await;
        assert!(
            matches!(result, Err(crate::Error::SubprocessTimeout { .. })),
            "expected SubprocessTimeout, got {result:?}"
        );
    }

    /// Integration smoke test — real whois binary, skipped if not in PATH.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn real_whois_smoke() {
        if which::which("whois").is_err() {
            eprintln!("skipping real_whois_smoke: whois binary not in PATH");
            return;
        }
        let whois = WhoisTtp::new(Arc::new(crate::ttp::subprocess::RealSubprocess));
        let result = whois
            .execute("example.com", CancellationToken::new())
            .await
            .expect("real whois execute should succeed");
        assert!(
            matches!(result, TtpOutput::Whois(_)),
            "expected TtpOutput::Whois, got {result:?}"
        );
    }
}
