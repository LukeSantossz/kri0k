//! `WhoisTtp` — MITRE T1590.001 (Gather Victim Network Information: Domain Properties).
//!
//! Stub module to allow `ttp/mod.rs` to compile. Will be fully implemented in Task 2.

/// Placeholder for `WhoisOutput` — populated in Task 2.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct WhoisOutput {
    /// Registrant Organization (D-42: only this contact is captured).
    pub registrant: Option<String>,
    /// Registrar name (e.g., "`MarkMonitor` Inc.").
    pub registrar: Option<String>,
    /// Name servers (lowercased, deduplicated). Order preserved.
    pub nameservers: Vec<String>,
    /// Creation date in ISO 8601 (raw from whois output).
    pub created_at: Option<String>,
    /// Last updated date.
    pub updated_at: Option<String>,
    /// Expiration date.
    pub expires_at: Option<String>,
    /// Lines not matching the key:value heuristic (D-41 graceful degradation).
    pub raw_unparsed: Vec<String>,
}
