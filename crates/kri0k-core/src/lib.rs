//! Core types and error handling for kri0k.

use serde::{Deserialize, Serialize};
use std::fmt;

// T7 security safeguards stubs
/// Audit logging (M-12..M-22, ADR-0007).
pub mod audit;
/// Runtime safeguards configuration (M-05, M-36).
pub mod safeguards;
/// Scope validation (M-01..M-04, ADR-0011).
pub mod scope;
/// TTP trait and execution flow (M-05, M-15, M-34..M-36, ADR-0012).
pub mod ttp;

/// Common error type for kri0k operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error with context.
    #[error("{0}")]
    Generic(String),

    /// I/O error from the operating system.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Scope violation: target not in scope allowlist (M-02).
    #[error("scope violation for target {target:?}: {reason}")]
    ScopeViolation {
        /// Target that was rejected.
        target: String,
        /// Reason for rejection.
        reason: String,
    },

    /// Rate limit exceeded for a TTP (M-34).
    #[error("rate limit exceeded for TTP {ttp_id}: retry in {retry_in_ms}ms")]
    RateLimitExceeded {
        /// TTP identifier that hit the limit.
        ttp_id: String,
        /// Milliseconds to wait before retrying.
        retry_in_ms: u64,
    },

    /// Subprocess timeout for a TTP (M-34, D-51).
    #[error("TTP {ttp_id} subprocess timeout after {timeout_ms}ms")]
    SubprocessTimeout {
        /// TTP identifier that timed out.
        ttp_id: String,
        /// Timeout threshold in milliseconds.
        timeout_ms: u64,
    },

    /// Parse error in a structured input (scope.yaml, whois output).
    #[error("parse error in {origin}: {detail}")]
    ParseError {
        /// Source identifier (e.g. "scope.yaml", "whois stdout").
        origin: String,
        /// Human-readable detail about the parse failure.
        detail: String,
    },

    /// Missing external binary dependency (D-50).
    #[error(
        "missing dependency: binary {binary:?} not found in PATH. \
        Install with: winget install Microsoft.Sysinternals.Whois (Windows) \
        or apt install whois (Linux)"
    )]
    MissingDependency {
        /// Name of the binary that was not found.
        binary: String,
    },

    /// Unknown TTP identifier requested (D-52).
    #[error("unknown TTP id: {ttp_id:?}")]
    UnknownTtp {
        /// The unrecognized TTP identifier.
        ttp_id: String,
    },

    /// Operation was cancelled via kill switch (D-62, M-36).
    #[error("operation cancelled")]
    Cancelled,
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Node identifier using ULID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(ulid::Ulid);

impl NodeId {
    /// Create a new unique node ID.
    #[must_use]
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }

    /// Get the inner ULID value.
    #[must_use]
    pub const fn inner(&self) -> ulid::Ulid {
        self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Edge identifier using ULID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(ulid::Ulid);

impl EdgeId {
    /// Create a new unique edge ID.
    #[must_use]
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }

    /// Get the inner ULID value.
    #[must_use]
    pub const fn inner(&self) -> ulid::Ulid {
        self.0
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)] // expect is ok in tests
mod tests {
    use super::*;

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_edge_id_uniqueness() {
        let id1 = EdgeId::new();
        let id2 = EdgeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_id_serialization() {
        let id = NodeId::new();
        let json = serde_json::to_string(&id).expect("serialize");
        let deserialized: NodeId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_error_io_from_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test file not found");
        let e: Error = io_err.into();
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn test_error_scope_violation_display() {
        let e = Error::ScopeViolation {
            target: "evil.com".into(),
            reason: "not in allowlist".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("evil.com"), "expected 'evil.com' in: {msg}");
    }

    #[test]
    fn test_error_missing_dependency_display() {
        let e = Error::MissingDependency {
            binary: "whois".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("whois"), "expected 'whois' in: {msg}");
        assert!(msg.contains("PATH"), "expected 'PATH' in: {msg}");
    }

    #[test]
    fn test_error_cancelled_display() {
        assert_eq!(Error::Cancelled.to_string(), "operation cancelled");
    }

    #[test]
    fn test_error_json_still_works() {
        let parse_err = serde_json::from_str::<i32>("not json")
            .expect_err("parsing 'not json' as i32 should fail");
        let e: Error = Error::from(parse_err);
        assert!(matches!(e, Error::Json(_)));
    }
}
