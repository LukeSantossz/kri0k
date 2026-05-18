// TODO(T7/M-12..M-22): Audit log implementation pending
//
// Mitigations from THREAT_MODEL.md:
// - M-12: sanitized-export mode (replaces secrets with placeholders)
// - M-13: Audit log separate from graph, unencrypted by default
// - M-14: regex-based redactor for PII/credentials
// - M-21: Events with dst_ip ∉ scope marked as out_of_scope_attempt
// - M-22: Hash-chained audit log (ADR-0007) prevents retroactive editing
//
// See: docs/security/THREAT_MODEL.md §2.2
// ADR: docs/adr/ADR-0007 (append-only audit log)

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Event sink for audit logging.
///
/// All security-relevant actions (TTP execution, scope violations,
/// engagement lifecycle) must be logged through this trait (M-13).
pub trait AuditSink: Send + Sync {
    /// Log a TTP execution event.
    ///
    /// # Errors
    /// Returns error if logging fails.
    fn log_ttp_execution(&mut self, event: TtpExecutionEvent) -> Result<(), crate::Error>;

    /// Log a scope violation attempt (M-21).
    ///
    /// # Errors
    /// Returns error if logging fails.
    fn log_scope_violation(&mut self, event: ScopeViolationEvent) -> Result<(), crate::Error>;

    /// Log an engagement lifecycle event (boot, kill, archive).
    ///
    /// # Errors
    /// Returns error if logging fails.
    fn log_engagement(&mut self, event: EngagementEvent) -> Result<(), crate::Error>;

    /// Finalize and flush audit log.
    ///
    /// # Errors
    /// Returns error if flush fails.
    fn flush(&mut self) -> Result<(), crate::Error>;
}

/// TTP execution event for audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtpExecutionEvent {
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// TTP identifier.
    pub ttp_id: String,
    /// Target address.
    pub target: String,
    /// Execution outcome.
    pub outcome: String,
    /// LLM provider used for this proposal (M-30).
    pub llm_provider: Option<String>,
}

/// Scope violation event (M-21).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeViolationEvent {
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Attempted target.
    pub target: String,
    /// Reason for rejection.
    pub reason: String,
}

/// Engagement lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEvent {
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Event type (boot, kill, archive, wipe).
    pub event_type: String,
    /// Operator identification (M-42, M-43).
    pub operator: String,
}

/// No-op audit sink for testing and default engagement bootstrap (D-38).
///
/// TODO(T7/M-13): Replace with real append-only implementation.
#[derive(Debug, Default)]
pub struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn log_ttp_execution(&mut self, _event: TtpExecutionEvent) -> Result<(), crate::Error> {
        // TODO(T7/M-13): Implement real audit logging
        Ok(())
    }

    fn log_scope_violation(&mut self, _event: ScopeViolationEvent) -> Result<(), crate::Error> {
        // TODO(T7/M-21): Log scope violations
        Ok(())
    }

    fn log_engagement(&mut self, _event: EngagementEvent) -> Result<(), crate::Error> {
        // TODO(T7/M-22): Log engagement events with hash chain
        Ok(())
    }

    fn flush(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }
}

/// Create an audit sink from a file path.
///
/// # Errors
/// Returns error if file cannot be created or opened.
pub fn create_audit_sink(_path: &Path) -> Result<Box<dyn AuditSink>, crate::Error> {
    // TODO(T7/M-13): Implement append-only JSONL audit sink
    // TODO(T7/M-14): Implement regex-based credential redactor
    // TODO(T7/M-22): Implement hash-chained log verification
    Ok(Box::new(NoopAuditSink))
}

#[cfg(test)]
#[allow(clippy::expect_used)] // expect is ok in tests
mod tests {
    use super::*;

    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_noop_audit_sink_is_boxable() {
        let mut sink: Box<dyn AuditSink> = Box::new(NoopAuditSink);
        let result = sink.log_ttp_execution(TtpExecutionEvent {
            timestamp: "2026-01-01T00:00:00Z".into(),
            ttp_id: "test".into(),
            target: "x".into(),
            outcome: "ok".into(),
            llm_provider: None,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_audit_sink_is_mutex_boxable_sync() {
        // Confirms Pitfall 5 + Pitfall 12 invariant: Box<dyn AuditSink + Send> inside Mutex is Sync.
        // Plano 04-05 Engagement::audit uses exactly this shape — failure here = breaks the pyclass.
        let _: std::sync::Mutex<Box<dyn AuditSink + Send>> =
            std::sync::Mutex::new(Box::new(NoopAuditSink));
        assert_sync::<std::sync::Mutex<Box<dyn AuditSink + Send>>>();
    }
}
