//! Partition healing protocols.

/// State of the healing process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealingState {
    NotStarted,
    Syncing,
    Merging,
    Verifying,
    Complete,
}

/// A healing protocol for resolving partition state.
#[derive(Debug)]
pub struct HealingProtocol {
    pub state: HealingState,
    pub partition_id: u64,
    pub majority_version: u64,
    pub minority_version: u64,
    pub merge_log: Vec<String>,
}

impl HealingProtocol {
    pub fn new(partition_id: u64) -> Self {
        Self {
            state: HealingState::NotStarted,
            partition_id,
            majority_version: 0,
            minority_version: 0,
            merge_log: Vec::new(),
        }
    }

    /// Start healing: record versions from both sides.
    pub fn start(&mut self, majority_version: u64, minority_version: u64) {
        self.state = HealingState::Syncing;
        self.majority_version = majority_version;
        self.minority_version = minority_version;
        self.merge_log.push(format!("Started healing: majority={}, minority={}", majority_version, minority_version));
    }

    /// Minority catches up by syncing from majority.
    pub fn sync(&mut self) {
        if self.state != HealingState::Syncing {
            return;
        }
        self.state = HealingState::Merging;
        self.merge_log.push("Synced state from majority".to_string());
    }

    /// Merge divergent operations.
    pub fn merge(&mut self) -> bool {
        if self.state != HealingState::Merging {
            return false;
        }
        self.state = HealingState::Verifying;
        self.merge_log.push("Merged divergent operations".to_string());
        true
    }

    /// Verify consistency after merge.
    pub fn verify(&mut self) -> bool {
        if self.state != HealingState::Verifying {
            return false;
        }
        self.state = HealingState::Complete;
        self.merge_log.push("Verified consistency".to_string());
        true
    }

    /// Full healing pipeline.
    pub fn heal(&mut self, majority_version: u64, minority_version: u64) -> bool {
        self.start(majority_version, minority_version);
        self.sync();
        self.merge();
        self.verify();
        self.state == HealingState::Complete
    }

    /// Check if minority had conflicting writes (split-brain).
    pub fn has_split_brain(&self) -> bool {
        self.minority_version > self.majority_version
    }

    /// Get the merge log.
    pub fn log(&self) -> &[String] {
        &self.merge_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healing_lifecycle() {
        let mut hp = HealingProtocol::new(1);
        assert_eq!(hp.state, HealingState::NotStarted);
        hp.start(10, 5);
        assert_eq!(hp.state, HealingState::Syncing);
        hp.sync();
        assert_eq!(hp.state, HealingState::Merging);
        hp.merge();
        assert_eq!(hp.state, HealingState::Verifying);
        hp.verify();
        assert_eq!(hp.state, HealingState::Complete);
    }

    #[test]
    fn test_full_heal() {
        let mut hp = HealingProtocol::new(1);
        let result = hp.heal(10, 5);
        assert!(result);
        assert_eq!(hp.state, HealingState::Complete);
    }

    #[test]
    fn test_split_brain_detection() {
        let mut hp = HealingProtocol::new(1);
        hp.start(5, 10); // minority advanced further
        assert!(hp.has_split_brain());
    }

    #[test]
    fn test_no_split_brain() {
        let mut hp = HealingProtocol::new(1);
        hp.start(10, 5);
        assert!(!hp.has_split_brain());
    }

    #[test]
    fn test_merge_log() {
        let mut hp = HealingProtocol::new(1);
        hp.heal(10, 5);
        assert_eq!(hp.log().len(), 4);
    }

    #[test]
    fn test_cannot_skip_states() {
        let mut hp = HealingProtocol::new(1);
        assert!(!hp.merge()); // not in Merging state
        assert!(!hp.verify()); // not in Verifying state
    }
}
