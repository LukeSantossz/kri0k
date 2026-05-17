// TODO(T7/M-05, M-36): Safeguards coordination module pending
//
// Mitigations from THREAT_MODEL.md:
// - M-05: Propose-only mode (default; ADR-0006)
// - M-36: Kill switch (Engagement.kill) accessible by hotkey
//
// See: docs/security/THREAT_MODEL.md §2.5
// ADR: docs/adr/ADR-0006 (propose-only mode)

/// Runtime safeguards configuration.
#[derive(Debug, Clone)]
pub struct SafeguardsConfig {
    /// Propose-only mode (M-05). When `true`, no TTPs are executed,
    /// only proposed for human review.
    pub propose_only: bool,

    /// Kill switch activated (M-36). When `true`, all operations
    /// are halted immediately.
    pub kill_switch_active: bool,
}

impl Default for SafeguardsConfig {
    fn default() -> Self {
        Self {
            // M-05: Propose-only is the default for MVP-1
            propose_only: true,
            kill_switch_active: false,
        }
    }
}

impl SafeguardsConfig {
    /// Check if execution is allowed under current safeguards.
    #[must_use]
    pub const fn allows_execution(&self) -> bool {
        !self.propose_only && !self.kill_switch_active
    }

    /// Activate the kill switch (M-36).
    ///
    /// This immediately halts all operations.
    pub const fn activate_kill_switch(&mut self) {
        self.kill_switch_active = true;
    }

    /// Deactivate the kill switch (requires explicit action).
    pub const fn deactivate_kill_switch(&mut self) {
        self.kill_switch_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_propose_only() {
        let config = SafeguardsConfig::default();
        assert!(config.propose_only);
        assert!(!config.allows_execution());
    }

    #[test]
    fn test_kill_switch() {
        let mut config = SafeguardsConfig {
            propose_only: false,
            ..Default::default()
        };

        assert!(config.allows_execution());

        config.activate_kill_switch();
        assert!(!config.allows_execution());

        config.deactivate_kill_switch();
        assert!(config.allows_execution());
    }
}
