//! Stale read detection.

/// Result of a stale read check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadResult {
    Fresh { version: u64 },
    Stale { read_version: u64, latest_version: u64 },
    Unavailable,
}

/// Detects stale reads by tracking version information across replicas.
#[derive(Debug, Clone)]
pub struct StaleReadDetector {
    /// node_id -> last known version
    replica_versions: std::collections::HashMap<u64, u64>,
    /// Committed (global) version
    committed_version: u64,
}

impl StaleReadDetector {
    pub fn new() -> Self {
        Self {
            replica_versions: std::collections::HashMap::new(),
            committed_version: 0,
        }
    }

    /// Update the version on a replica.
    pub fn update_replica(&mut self, node_id: u64, version: u64) {
        let entry = self.replica_versions.entry(node_id).or_insert(0);
        *entry = std::cmp::max(*entry, version);
    }

    /// Advance the committed version.
    pub fn commit(&mut self, version: u64) {
        self.committed_version = std::cmp::max(self.committed_version, version);
    }

    /// Check if a read from a replica is stale.
    pub fn check_read(&self, node_id: u64) -> ReadResult {
        let replica_ver = self.replica_versions.get(&node_id).copied().unwrap_or(0);
        if replica_ver == 0 {
            return ReadResult::Unavailable;
        }
        if replica_ver >= self.committed_version {
            ReadResult::Fresh { version: replica_ver }
        } else {
            ReadResult::Stale {
                read_version: replica_ver,
                latest_version: self.committed_version,
            }
        }
    }

    /// Get the number of replicas at the committed version.
    pub fn up_to_date_replicas(&self) -> usize {
        self.replica_versions.values()
            .filter(|&&v| v >= self.committed_version)
            .count()
    }

    /// Get the number of stale replicas.
    pub fn stale_replicas(&self) -> usize {
        self.replica_versions.values()
            .filter(|&&v| v < self.committed_version && v > 0)
            .count()
    }

    /// Get the committed version.
    pub fn committed_version(&self) -> u64 {
        self.committed_version
    }
}

impl Default for StaleReadDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_read() {
        let mut det = StaleReadDetector::new();
        det.update_replica(1, 5);
        det.commit(5);
        assert_eq!(det.check_read(1), ReadResult::Fresh { version: 5 });
    }

    #[test]
    fn test_stale_read() {
        let mut det = StaleReadDetector::new();
        det.update_replica(1, 3);
        det.commit(5);
        assert_eq!(det.check_read(1), ReadResult::Stale { read_version: 3, latest_version: 5 });
    }

    #[test]
    fn test_unavailable_read() {
        let det = StaleReadDetector::new();
        assert_eq!(det.check_read(1), ReadResult::Unavailable);
    }

    #[test]
    fn test_up_to_date_count() {
        let mut det = StaleReadDetector::new();
        det.update_replica(1, 5);
        det.update_replica(2, 5);
        det.update_replica(3, 3);
        det.commit(5);
        assert_eq!(det.up_to_date_replicas(), 2);
    }

    #[test]
    fn test_stale_count() {
        let mut det = StaleReadDetector::new();
        det.update_replica(1, 5);
        det.update_replica(2, 3);
        det.update_replica(3, 1);
        det.commit(5);
        assert_eq!(det.stale_replicas(), 2);
    }

    #[test]
    fn test_commit_advances() {
        let mut det = StaleReadDetector::new();
        det.commit(3);
        det.commit(7);
        assert_eq!(det.committed_version(), 7);
        det.commit(5); // doesn't go backwards
        assert_eq!(det.committed_version(), 7);
    }
}
