//! Network partition detection and simulation.

/// State of a partition between groups of nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionState {
    Active,
    Healing,
    Healed,
}

/// A network partition dividing nodes into disconnected groups.
#[derive(Debug, Clone)]
pub struct Partition {
    pub id: u64,
    pub groups: Vec<Vec<u64>>,
    pub state: PartitionState,
}

impl Partition {
    pub fn new(id: u64, groups: Vec<Vec<u64>>) -> Self {
        Self { id, groups, state: PartitionState::Active }
    }

    /// Check if two nodes can communicate (in the same group).
    pub fn can_communicate(&self, a: u64, b: u64) -> bool {
        for group in &self.groups {
            if group.contains(&a) && group.contains(&b) {
                return true;
            }
        }
        false
    }

    /// Get the group a node belongs to.
    pub fn group_of(&self, node: u64) -> Option<usize> {
        self.groups.iter().position(|g| g.contains(&node))
    }

    /// Begin healing the partition.
    pub fn begin_healing(&mut self) {
        self.state = PartitionState::Healing;
    }

    /// Complete healing.
    pub fn complete_healing(&mut self) {
        self.state = PartitionState::Healed;
    }
}

/// Detects and tracks network partitions.
#[derive(Debug)]
pub struct PartitionDetector {
    partitions: Vec<Partition>,
    next_id: u64,
    heartbeat_timeouts: std::collections::HashMap<u64, u64>,
    timeout_threshold: u64,
}

impl PartitionDetector {
    pub fn new(timeout_threshold: u64) -> Self {
        Self {
            partitions: Vec::new(),
            next_id: 0,
            heartbeat_timeouts: std::collections::HashMap::new(),
            timeout_threshold,
        }
    }

    /// Record a heartbeat from a node.
    pub fn heartbeat(&mut self, node_id: u64, round: u64) {
        self.heartbeat_timeouts.insert(node_id, round);
    }

    /// Detect partitions based on heartbeat timeouts.
    pub fn detect(&mut self, all_nodes: &[u64], current_round: u64) -> Vec<&Partition> {
        let mut alive = Vec::new();
        let mut dead = Vec::new();

        for &node in all_nodes {
            let last = self.heartbeat_timeouts.get(&node).copied().unwrap_or(0);
            if current_round > last + self.timeout_threshold {
                dead.push(node);
            } else {
                alive.push(node);
            }
        }

        // If some nodes are unreachable, that's a partition
        self.partitions.clear();
        if !dead.is_empty() {
            let id = self.next_id;
            self.next_id += 1;
            self.partitions.push(Partition::new(id, vec![alive, dead]));
        }

        self.partitions.iter().collect()
    }

    /// Get active partitions.
    pub fn active_partitions(&self) -> Vec<&Partition> {
        self.partitions.iter()
            .filter(|p| p.state == PartitionState::Active)
            .collect()
    }

    /// Check if two nodes are in the same partition group.
    pub fn can_communicate(&self, a: u64, b: u64) -> bool {
        for p in &self.partitions {
            if p.state != PartitionState::Active {
                continue;
            }
            if !p.can_communicate(a, b) {
                return false;
            }
        }
        true
    }

    /// Number of partitions.
    pub fn len(&self) -> usize {
        self.partitions.len()
    }

    /// Is there any active partition?
    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_communication() {
        let p = Partition::new(0, vec![vec![1, 2, 3], vec![4, 5]]);
        assert!(p.can_communicate(1, 2));
        assert!(!p.can_communicate(1, 4));
    }

    #[test]
    fn test_group_of() {
        let p = Partition::new(0, vec![vec![1, 2], vec![3, 4]]);
        assert_eq!(p.group_of(1), Some(0));
        assert_eq!(p.group_of(3), Some(1));
        assert_eq!(p.group_of(99), None);
    }

    #[test]
    fn test_healing_states() {
        let mut p = Partition::new(0, vec![vec![1, 2], vec![3]]);
        assert_eq!(p.state, PartitionState::Active);
        p.begin_healing();
        assert_eq!(p.state, PartitionState::Healing);
        p.complete_healing();
        assert_eq!(p.state, PartitionState::Healed);
    }

    #[test]
    fn test_detect_partition() {
        let mut det = PartitionDetector::new(3);
        let nodes = vec![1, 2, 3, 4, 5];
        // Nodes 1,2,3 heartbeat; 4,5 don't
        det.heartbeat(1, 1);
        det.heartbeat(2, 1);
        det.heartbeat(3, 1);
        let partitions = det.detect(&nodes, 10);
        assert_eq!(partitions.len(), 1);
    }

    #[test]
    fn test_no_partition_all_alive() {
        let mut det = PartitionDetector::new(3);
        let nodes = vec![1, 2, 3];
        for &n in &nodes {
            det.heartbeat(n, 5);
        }
        let partitions = det.detect(&nodes, 7);
        assert!(partitions.is_empty());
    }

    #[test]
    fn test_can_communicate_no_partition() {
        let det = PartitionDetector::new(3);
        assert!(det.can_communicate(1, 2));
    }

    #[test]
    fn test_can_communicate_with_partition() {
        let mut det = PartitionDetector::new(3);
        let nodes = vec![1, 2, 3, 4];
        det.heartbeat(1, 8); // alive at round 8
        det.heartbeat(2, 8); // alive at round 8
        // nodes 3,4 never heartbeated
        det.detect(&nodes, 10);
        assert!(!det.can_communicate(1, 3));
    }
}
