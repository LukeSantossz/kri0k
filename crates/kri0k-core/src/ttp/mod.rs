//! TTP (Tactics, Techniques, and Procedures) trait and execution infrastructure.
//!
//! All offensive actions must implement the [`Ttp`] trait to ensure:
//! - Async execution with cancellation support (M-36, D-62)
//! - Rate limit declaration (M-34, D-45)
//! - Risk level classification (M-21)
//!
//! # Architecture
//!
//! `LLM proposes → Engagement dispatches → WhoisTtp::execute (subprocess) → TtpOutput`
//!
//! See: `docs/adr/ADR-0012-ttp-trait-adapters.md`
//! See: `docs/security/THREAT_MODEL.md §2.3, §2.5`

pub mod subprocess;
pub mod whois;

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

/// Outcome of any TTP execution. Phase 4 only populates Whois.
///
/// Future phases will add Nmap, Dig, etc. as new variants.
#[derive(Debug, Clone)]
pub enum TtpOutput {
    /// Output from MITRE T1590.001 (whois reconnaissance).
    Whois(crate::ttp::whois::WhoisOutput),
    // Phase 5+: Nmap(...), Dig(...), etc.
}

/// Rate limit configuration for a TTP (M-34).
///
/// Each TTP declares its own limits. Enforcement is TTP-local via
/// `Mutex<Option<Instant>>` field (D-45).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Maximum requests per second.
    pub max_rps: Option<u32>,
    /// Maximum concurrent requests.
    pub max_concurrent: Option<u32>,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            max_rps: Some(10),
            max_concurrent: Some(5),
        }
    }
}

/// Risk level classification (M-21).
///
/// High-risk TTPs require human gate (Phase 11 TUI). Phase 4 whois is `Safe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only, no state changes.
    Safe,
    /// May trigger IDS/IPS but no damage.
    Low,
    /// May cause service degradation.
    Medium,
    /// May cause service disruption (requires human gate).
    High,
}

/// Async TTP trait for all offensive techniques (D-44, ADR-0012).
///
/// `#[async_trait]` is MANDATORY: native `async fn in trait` is not
/// dyn-compatible in Rust 1.85. `Box<dyn Ttp>` in the registry (D-52)
/// requires this macro to rewrite the async method signatures.
///
/// # Example
/// ```ignore
/// let ttp: Box<dyn Ttp> = Box::new(WhoisTtp::new(Arc::new(RealSubprocess)));
/// let output = ttp.execute("example.com", CancellationToken::new()).await?;
/// ```
#[async_trait]
pub trait Ttp: Send + Sync {
    /// MITRE ATT&CK TTP identifier (e.g., "T1590.001").
    fn id(&self) -> &str;

    /// Short description of what this TTP does.
    fn description(&self) -> &str;

    /// Risk level classification for gate decisions (M-21).
    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Safe
    }

    /// Rate limit configuration (M-34, D-45).
    fn rate_limits(&self) -> RateLimits;

    /// Default subprocess timeout (D-51).
    ///
    /// Override in concrete impls if the TTP needs more/less time.
    fn default_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Execute this TTP against `target`.
    ///
    /// Implementations MUST:
    /// - Respect `cancel` via `tokio::select!` biased (D-62, M-36).
    /// - Apply TTP-local rate limit before subprocess (D-45, M-34).
    /// - Use subprocess abstraction — never raw `std::process::Command` (D-44).
    ///
    /// # Errors
    /// - `Error::Cancelled` if `cancel` fires before completion.
    /// - `Error::SubprocessTimeout` if execution exceeds `default_timeout()`.
    /// - `Error::Io` for subprocess I/O failures.
    /// - `Error::RateLimitExceeded` (reserved for future explicit rejection).
    async fn execute(
        &self,
        target: &str,
        cancel: CancellationToken,
    ) -> Result<TtpOutput, crate::Error>;
}
