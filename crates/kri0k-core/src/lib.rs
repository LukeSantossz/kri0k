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
}
