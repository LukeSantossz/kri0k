// TODO(T7/M-01..M-04): Scope validation implementation pending
//
// Mitigations from THREAT_MODEL.md:
// - M-01: scope.yaml mandatory at boot
// - M-02: Deterministic Rust validator checks target ∈ scope.yaml before execution
// - M-03: scope_hash (sha256) embedded in every Snapshot
// - M-04: scope.yaml requires non-bypassable fields
//
// See: docs/security/THREAT_MODEL.md §2.1
// ADR: docs/adr/ADR-0011 (scope validation)

use std::collections::HashMap;

/// Scope definition for an engagement.
///
/// The scope defines which targets are authorized for testing
/// and is mandatory before any TTP execution (M-01).
#[derive(Debug, Clone)]
pub struct Scope {
    /// Authorized target networks/hosts (CIDR notation).
    pub targets: Vec<String>,

    /// Operator identification (M-42).
    pub operator: String,

    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl Scope {
    /// Load scope from YAML file.
    ///
    /// # Errors
    /// Returns error if scope.yaml is missing or invalid (M-01).
    #[allow(clippy::todo)]
    pub fn from_yaml(_path: &str) -> Result<Self, crate::Error> {
        // TODO(T7/M-01): Implement YAML parsing
        todo!("Scope::from_yaml not implemented (T7/M-01)")
    }

    /// Compute SHA256 hash of the scope definition (M-03).
    #[must_use]
    #[allow(clippy::todo)]
    pub fn compute_hash(&self) -> String {
        // TODO(T7/M-03): Implement scope hashing
        todo!("Scope::compute_hash not implemented (T7/M-03)")
    }
}

/// Validate if a target is within the authorized scope.
///
/// This is the core pre-execution validator (M-02, ADR-0005).
///
/// # Errors
/// Returns error if target is out of scope or validation fails.
pub fn validate_target(_scope: &Scope, _target: &str) -> Result<(), crate::Error> {
    // TODO(T7/M-02): Implement deterministic target validation
    // - Expand DNS to IPs
    // - Check if target matches any scope CIDR
    // - Mark out_of_scope_attempt in audit log (M-21)
    Err(crate::Error::Generic(
        "scope validation not implemented (T7/M-02)".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_validation_todo() {
        let scope = Scope {
            targets: vec!["192.168.1.0/24".to_string()],
            operator: "test@example.com".to_string(),
            metadata: HashMap::new(),
        };

        // Should return error until implemented
        let result = validate_target(&scope, "192.168.1.10");
        assert!(
            matches!(result, Err(crate::Error::Generic(ref s)) if s.contains("not implemented"))
        );
    }
}
