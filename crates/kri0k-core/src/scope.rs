//! Scope validation (M-01..M-04, ADR-0011).
//!
//! Phase 4 = exact-match allowlist (D-48).
//! Phase 7 extends to CIDR + wildcards.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Full lookahead schema for scope.yaml (D-58).
///
/// Phase 4 consumes: `version`, `targets`, `safeguards.propose_only`.
/// Fields `targets_cidr`, `targets_wildcards`, `rate_limits`, `audit_path`
/// are parsed but unused until Phase 7/8.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeConfig {
    /// Schema version — MUST be 1; other values are rejected.
    pub version: u32,
    /// Human-readable objective for this engagement.
    pub objective: Option<String>,
    /// Operator identification (M-42).
    pub operator: Option<String>,
    /// Exact-match domain/host allowlist (D-48). Phase 4 uses this.
    #[serde(default)]
    pub targets: Vec<String>,
    /// CIDR-based target ranges — Phase 7 lookahead stub.
    #[serde(default)]
    pub targets_cidr: Vec<String>,
    /// Wildcard-based target patterns — Phase 7 lookahead stub.
    #[serde(default)]
    pub targets_wildcards: Vec<String>,
    /// Runtime safeguards configuration (D-49, ADR-0006).
    #[serde(default)]
    pub safeguards: SafeguardsSection,
    /// Per-TTP rate limiting configuration — Phase 4 parsed but unused.
    #[serde(default)]
    pub rate_limits: RateLimitsSection,
    /// Path for audit log output — Phase 8 lookahead stub.
    #[serde(default)]
    pub audit_path: Option<String>,
    /// Raw YAML bytes as loaded from disk or serialized from dict.
    /// Used for deterministic SHA-256 hashing (M-03). Skipped during serde.
    #[serde(skip)]
    pub raw_yaml: String,
}

/// Safeguards section within scope.yaml (D-49, ADR-0006).
///
/// # Pitfall 11 — manual `Default` required
///
/// `#[derive(Default)]` would set `propose_only = false` because `bool::default() == false`.
/// That would silently allow TTP execution when the YAML omits the `safeguards:` block.
/// The manual `impl Default` below forces `propose_only = true` as the safe default.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafeguardsSection {
    /// When `true`, TTPs are only proposed — never executed (D-49, ADR-0006 default).
    #[serde(default = "default_true")]
    pub propose_only: bool,
    /// Kill switch: when `true`, all operations are halted immediately (M-36).
    #[serde(default)]
    pub kill_switch: bool,
}

impl Default for SafeguardsSection {
    /// Returns the secure-default safeguards: propose-only enabled, kill switch off.
    fn default() -> Self {
        Self {
            propose_only: true,
            kill_switch: false,
        }
    }
}

/// Rate limits section — Phase 4 parsed but unused.
///
/// Phase 7 will wire these into TTP execution throttling.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RateLimitsSection {
    /// Global requests per second cap (None = unlimited).
    #[serde(default)]
    pub global_rps: Option<u32>,
}

/// Returns `true` — used as `#[serde(default = "default_true")]` for `propose_only`.
const fn default_true() -> bool {
    true
}

impl ScopeConfig {
    /// Load and parse a scope.yaml from disk.
    ///
    /// Validates schema `version == 1` (D-58).
    ///
    /// # Errors
    ///
    /// - [`crate::Error::Io`] on file read failure.
    /// - [`crate::Error::ParseError`] on invalid YAML or unsupported schema version.
    pub fn from_yaml(path: &Path) -> crate::Result<Self> {
        let raw = std::fs::read_to_string(path).map_err(crate::Error::Io)?;
        let mut config: Self =
            serde_yaml_ng::from_str(&raw).map_err(|e| crate::Error::ParseError {
                origin: "scope.yaml".to_string(),
                detail: e.to_string(),
            })?;
        if config.version != 1 {
            return Err(crate::Error::ParseError {
                origin: "scope.yaml".to_string(),
                detail: format!("unsupported scope version: {} (expected 1)", config.version),
            });
        }
        config.raw_yaml = raw;
        Ok(config)
    }

    /// Build `ScopeConfig` from a JSON value.
    ///
    /// Used by `Engagement::new` when Python passes a scope dict (Plano 04-05).
    /// The value is also serialized to YAML for a consistent `raw_yaml` used by
    /// `compute_hash`.
    ///
    /// # Errors
    ///
    /// - [`crate::Error::ParseError`] on YAML serialization failure or unsupported version.
    /// - [`crate::Error::Json`] if the JSON value does not match the `ScopeConfig` shape.
    pub fn from_dict_value(value: serde_json::Value) -> crate::Result<Self> {
        // Serialize to YAML so raw_yaml is consistent with file-based loading (M-03).
        let raw = serde_yaml_ng::to_string(&value).map_err(|e| crate::Error::ParseError {
            origin: "scope_dict".to_string(),
            detail: e.to_string(),
        })?;
        let mut config: Self = serde_json::from_value(value)?;
        if config.version != 1 {
            return Err(crate::Error::ParseError {
                origin: "scope_dict".to_string(),
                detail: format!("unsupported scope version: {} (expected 1)", config.version),
            });
        }
        config.raw_yaml = raw;
        Ok(config)
    }

    /// SHA-256 hex digest of the raw YAML (M-03).
    ///
    /// Embedded in every `Snapshot` for scope-tampering detection (ADR-0011).
    /// The hash is deterministic: same YAML bytes → same digest.
    #[must_use]
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.raw_yaml.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Exact-match allowlist check (D-48, M-02).
    ///
    /// Phase 4 accepts a target only if it appears verbatim in `self.targets`.
    /// Phase 7 will extend with CIDR and wildcard matching.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::ScopeViolation`] if `target` is not in the allowlist.
    pub fn validate_target(&self, target: &str) -> crate::Result<()> {
        // M-02 (D-48): Phase 4 = exact-match allowlist. Phase 7 will add CIDR + wildcards.
        if self.targets.iter().any(|t| t == target) {
            Ok(())
        } else {
            Err(crate::Error::ScopeViolation {
                target: target.to_string(),
                reason: format!("target {target:?} not in scope.targets allowlist"),
            })
        }
    }
}

/// Backwards-compatible free function alias for `ScopeConfig::validate_target`.
///
/// Kept so that any code referencing the old free-function API compiles without changes.
///
/// # Errors
///
/// Returns [`crate::Error::ScopeViolation`] if `target` is not in `scope.targets`.
pub fn validate_target(scope: &ScopeConfig, target: &str) -> crate::Result<()> {
    scope.validate_target(target)
}

/// Legacy scope struct — retained for API compatibility while consumers migrate.
///
/// New code should use [`ScopeConfig`] directly.
#[derive(Debug, Clone)]
#[deprecated(since = "0.2.0", note = "Use ScopeConfig instead")]
pub struct Scope {
    /// Authorized target networks/hosts.
    pub targets: Vec<String>,
    /// Operator identification.
    pub operator: String,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_yaml(content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("kri0k-scope-{}.yaml", ulid::Ulid::new()));
        let mut f = std::fs::File::create(&path).expect("create temp");
        f.write_all(content.as_bytes()).expect("write");
        path
    }

    const FULL_V1_YAML: &str = "
version: 1
operator: test@example.com
objective: \"test recon\"
targets:
  - example.com
  - foo.example.org
targets_cidr: []
targets_wildcards: []
safeguards:
  propose_only: false
  kill_switch: true
rate_limits:
  global_rps: 10
audit_path: null
";

    const MINIMAL_YAML: &str = "
version: 1
targets:
  - example.com
";

    #[test]
    fn parses_full_v1_schema() {
        let path = write_temp_yaml(FULL_V1_YAML);
        let config = ScopeConfig::from_yaml(&path).expect("parse");
        assert_eq!(config.version, 1);
        assert_eq!(config.operator.as_deref(), Some("test@example.com"));
        assert_eq!(
            config.targets,
            vec!["example.com".to_string(), "foo.example.org".to_string()]
        );
        assert!(!config.safeguards.propose_only);
        assert!(config.safeguards.kill_switch);
        assert_eq!(config.rate_limits.global_rps, Some(10));
    }

    #[test]
    fn unknown_version_rejected() {
        let path = write_temp_yaml("version: 2\ntargets: []\n");
        let err = ScopeConfig::from_yaml(&path).expect_err("should reject");
        match err {
            crate::Error::ParseError { detail, .. } => {
                assert!(detail.contains("unsupported"), "got: {detail}");
            },
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[test]
    fn missing_version_rejected() {
        let path = write_temp_yaml("targets:\n  - example.com\n");
        let err = ScopeConfig::from_yaml(&path).expect_err("missing version should fail");
        assert!(
            matches!(err, crate::Error::ParseError { .. }),
            "expected ParseError"
        );
    }

    #[test]
    fn validate_target_accepts_exact_match() {
        let path = write_temp_yaml(MINIMAL_YAML);
        let config = ScopeConfig::from_yaml(&path).expect("parse");
        assert!(config.validate_target("example.com").is_ok());
    }

    #[test]
    fn validate_target_rejects_non_match() {
        let path = write_temp_yaml(MINIMAL_YAML);
        let config = ScopeConfig::from_yaml(&path).expect("parse");
        let err = config
            .validate_target("evil.com")
            .expect_err("should reject");
        match err {
            crate::Error::ScopeViolation { target, reason } => {
                assert_eq!(target, "evil.com");
                assert!(reason.contains("evil.com"));
            },
            other => panic!("expected ScopeViolation, got {other:?}"),
        }
    }

    #[test]
    fn validate_target_substring_not_accepted() {
        let path = write_temp_yaml(MINIMAL_YAML);
        let config = ScopeConfig::from_yaml(&path).expect("parse");
        assert!(
            config.validate_target("xexample.com").is_err(),
            "substrings must not match"
        );
        assert!(
            config.validate_target("example.com.evil").is_err(),
            "suffixes must not match"
        );
        assert!(config.validate_target("").is_err(), "empty must not match");
    }

    #[test]
    fn compute_hash_deterministic() {
        let path = write_temp_yaml(MINIMAL_YAML);
        let c1 = ScopeConfig::from_yaml(&path).expect("parse 1");
        let c2 = ScopeConfig::from_yaml(&path).expect("parse 2");
        assert_eq!(c1.compute_hash(), c2.compute_hash());
        assert_eq!(c1.compute_hash().len(), 64, "SHA-256 hex is 64 chars");
        assert!(c1.compute_hash().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compute_hash_differs_for_different_yaml() {
        let p1 = write_temp_yaml("version: 1\ntargets:\n  - a.com\n");
        let p2 = write_temp_yaml("version: 1\ntargets:\n  - b.com\n");
        let c1 = ScopeConfig::from_yaml(&p1).expect("parse 1");
        let c2 = ScopeConfig::from_yaml(&p2).expect("parse 2");
        assert_ne!(c1.compute_hash(), c2.compute_hash());
    }

    #[test]
    fn default_safeguards_is_propose_only_true() {
        // Pitfall 11: YAML sem safeguards block MUST default propose_only=true
        let path = write_temp_yaml(MINIMAL_YAML);
        let config = ScopeConfig::from_yaml(&path).expect("parse");
        assert!(
            config.safeguards.propose_only,
            "default propose_only must be true (D-49 + Pitfall 11)"
        );
        assert!(
            !config.safeguards.kill_switch,
            "default kill_switch must be false"
        );
    }

    #[test]
    fn from_dict_value_parses_json() {
        let json = serde_json::json!({
            "version": 1,
            "targets": ["example.com"],
        });
        let config = ScopeConfig::from_dict_value(json).expect("parse");
        assert_eq!(config.version, 1);
        assert_eq!(config.targets, vec!["example.com".to_string()]);
        assert!(
            config.safeguards.propose_only,
            "from_dict default propose_only must be true"
        );
        assert!(!config.raw_yaml.is_empty(), "raw_yaml must be populated");
    }

    #[test]
    fn from_dict_value_rejects_version_2() {
        let json = serde_json::json!({ "version": 2, "targets": [] });
        assert!(matches!(
            ScopeConfig::from_dict_value(json),
            Err(crate::Error::ParseError { .. })
        ));
    }

    #[test]
    fn parses_committed_example_yaml() {
        let path = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/scope.example.yaml"
        ));
        let config = ScopeConfig::from_yaml(&path).expect("example yaml must parse");
        assert_eq!(config.version, 1);
        assert!(
            config.targets.contains(&"example.com".to_string()),
            "example.yaml must include example.com"
        );
        assert!(
            config.safeguards.propose_only,
            "example must show secure default"
        );
    }
}
