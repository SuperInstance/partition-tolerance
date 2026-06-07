//! Quorum checking for reads and writes.

/// Result of a quorum check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumResult {
    /// Quorum achieved with N nodes.
    Achieved { count: usize },
    /// Quorum not achieved.
    Failed { have: usize, need: usize },
}

/// Checks quorum requirements for distributed operations.
#[derive(Debug)]
pub struct QuorumChecker {
    pub total_nodes: usize,
    pub write_quorum: usize,
    pub read_quorum: usize,
}

impl QuorumChecker {
    pub fn new(total_nodes: usize) -> Self {
        let majority = total_nodes / 2 + 1;
        Self {
            total_nodes,
            write_quorum: majority,
            read_quorum: majority,
        }
    }

    /// Create with custom quorum sizes.
    pub fn with_quorums(total: usize, write: usize, read: usize) -> Self {
        Self {
            total_nodes: total,
            write_quorum: write,
            read_quorum: read,
        }
    }

    /// Check if we have write quorum.
    pub fn check_write(&self, available: usize) -> QuorumResult {
        if available >= self.write_quorum {
            QuorumResult::Achieved { count: available }
        } else {
            QuorumResult::Failed { have: available, need: self.write_quorum }
        }
    }

    /// Check if we have read quorum.
    pub fn check_read(&self, available: usize) -> QuorumResult {
        if available >= self.read_quorum {
            QuorumResult::Achieved { count: available }
        } else {
            QuorumResult::Failed { have: available, need: self.read_quorum }
        }
    }

    /// Check strict quorum (R + W > N) — ensures read and write quorums overlap.
    pub fn is_strict(&self) -> bool {
        self.read_quorum + self.write_quorum > self.total_nodes
    }

    /// Get the majority size for the cluster.
    pub fn majority(&self) -> usize {
        self.total_nodes / 2 + 1
    }

    /// Check if a group of nodes forms a majority partition.
    pub fn is_majority_partition(&self, group_size: usize) -> bool {
        group_size >= self.majority()
    }

    /// Check if a group is a minority partition.
    pub fn is_minority_partition(&self, group_size: usize) -> bool {
        group_size < self.majority()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_quorum() {
        let qc = QuorumChecker::new(5);
        assert_eq!(qc.majority(), 3);
        assert_eq!(qc.write_quorum, 3);
    }

    #[test]
    fn test_write_quorum_achieved() {
        let qc = QuorumChecker::new(5);
        let result = qc.check_write(3);
        assert_eq!(result, QuorumResult::Achieved { count: 3 });
    }

    #[test]
    fn test_write_quorum_failed() {
        let qc = QuorumChecker::new(5);
        let result = qc.check_write(2);
        assert_eq!(result, QuorumResult::Failed { have: 2, need: 3 });
    }

    #[test]
    fn test_strict_quorum() {
        // R=3, W=3, N=5 => 3+3=6 > 5 => strict
        let qc = QuorumChecker::with_quorums(5, 3, 3);
        assert!(qc.is_strict());
    }

    #[test]
    fn test_non_strict_quorum() {
        // R=2, W=2, N=5 => 2+2=4 <= 5 => not strict
        let qc = QuorumChecker::with_quorums(5, 2, 2);
        assert!(!qc.is_strict());
    }

    #[test]
    fn test_majority_partition() {
        let qc = QuorumChecker::new(5);
        assert!(qc.is_majority_partition(3));
        assert!(!qc.is_majority_partition(2));
    }

    #[test]
    fn test_minority_partition() {
        let qc = QuorumChecker::new(5);
        assert!(qc.is_minority_partition(2));
        assert!(!qc.is_minority_partition(3));
    }

    #[test]
    fn test_read_quorum() {
        let qc = QuorumChecker::new(5);
        assert_eq!(qc.check_read(4), QuorumResult::Achieved { count: 4 });
        assert_eq!(qc.check_read(1), QuorumResult::Failed { have: 1, need: 3 });
    }
}
