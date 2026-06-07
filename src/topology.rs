//! Network topology management.

/// A node in the network topology.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    pub region: String,
    pub alive: bool,
    pub latency_ms: u32,
}

impl Node {
    pub fn new(id: u64, region: &str) -> Self {
        Self { id, region: region.to_string(), alive: true, latency_ms: 0 }
    }

    pub fn with_latency(mut self, latency_ms: u32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub fn kill(&mut self) {
        self.alive = false;
    }

    pub fn revive(&mut self) {
        self.alive = true;
    }
}

/// Network topology: manages nodes and their connectivity.
#[derive(Debug, Clone)]
pub struct Topology {
    pub nodes: Vec<Node>,
    pub blocked_links: Vec<(u64, u64)>,
}

impl Topology {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), blocked_links: Vec::new() }
    }

    /// Add a node to the topology.
    pub fn add_node(&mut self, node: Node) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    /// Block communication between two nodes.
    pub fn block_link(&mut self, a: u64, b: u64) {
        if !self.blocked_links.contains(&(a, b)) && !self.blocked_links.contains(&(b, a)) {
            self.blocked_links.push((a, b));
        }
    }

    /// Unblock communication between two nodes.
    pub fn unblock_link(&mut self, a: u64, b: u64) {
        self.blocked_links.retain(|&(x, y)| !(x == a && y == b || x == b && y == a));
    }

    /// Check if two nodes can communicate.
    pub fn can_communicate(&self, a: u64, b: u64) -> bool {
        let node_a = self.nodes.iter().find(|n| n.id == a);
        let node_b = self.nodes.iter().find(|n| n.id == b);
        match (node_a, node_b) {
            (Some(na), Some(nb)) => {
                na.alive && nb.alive && !self.is_blocked(a, b)
            }
            _ => false,
        }
    }

    fn is_blocked(&self, a: u64, b: u64) -> bool {
        self.blocked_links.iter().any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
    }

    /// Get all alive nodes.
    pub fn alive_nodes(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.alive).collect()
    }

    /// Get reachable nodes from a given node.
    pub fn reachable_from(&self, node_id: u64) -> Vec<u64> {
        self.nodes.iter()
            .filter(|n| n.id != node_id && self.can_communicate(node_id, n.id))
            .map(|n| n.id)
            .collect()
    }

    /// Simulate a partition: block all links between two groups.
    pub fn create_partition(&mut self, group_a: &[u64], group_b: &[u64]) {
        for &a in group_a {
            for &b in group_b {
                self.block_link(a, b);
            }
        }
    }

    /// Heal all partitions.
    pub fn heal_all(&mut self) {
        self.blocked_links.clear();
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Is the topology empty?
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get nodes in a specific region.
    pub fn nodes_in_region(&self, region: &str) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.region == region).collect()
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(2, "us-west"));
        assert_eq!(topo.len(), 2);
    }

    #[test]
    fn test_duplicate_node() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(1, "us-east"));
        assert_eq!(topo.len(), 1);
    }

    #[test]
    fn test_can_communicate() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(2, "us-west"));
        assert!(topo.can_communicate(1, 2));
    }

    #[test]
    fn test_blocked_link() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(2, "us-west"));
        topo.block_link(1, 2);
        assert!(!topo.can_communicate(1, 2));
    }

    #[test]
    fn test_unblock_link() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(2, "us-west"));
        topo.block_link(1, 2);
        topo.unblock_link(1, 2);
        assert!(topo.can_communicate(1, 2));
    }

    #[test]
    fn test_create_partition() {
        let mut topo = Topology::new();
        for i in 1..=6 {
            topo.add_node(Node::new(i, "region"));
        }
        topo.create_partition(&[1, 2, 3], &[4, 5, 6]);
        assert!(!topo.can_communicate(1, 4));
        assert!(topo.can_communicate(1, 2));
    }

    #[test]
    fn test_heal_all() {
        let mut topo = Topology::new();
        for i in 1..=4 {
            topo.add_node(Node::new(i, "region"));
        }
        topo.create_partition(&[1, 2], &[3, 4]);
        topo.heal_all();
        assert!(topo.can_communicate(1, 3));
    }

    #[test]
    fn test_dead_node() {
        let mut topo = Topology::new();
        let mut node = Node::new(1, "us-east");
        node.kill();
        topo.add_node(node);
        topo.add_node(Node::new(2, "us-west"));
        assert!(!topo.can_communicate(1, 2));
    }

    #[test]
    fn test_reachable_from() {
        let mut topo = Topology::new();
        for i in 1..=5 {
            topo.add_node(Node::new(i, "region"));
        }
        topo.block_link(1, 3);
        let reachable = topo.reachable_from(1);
        assert_eq!(reachable.len(), 3); // 2, 4, 5
        assert!(!reachable.contains(&3));
    }

    #[test]
    fn test_nodes_in_region() {
        let mut topo = Topology::new();
        topo.add_node(Node::new(1, "us-east"));
        topo.add_node(Node::new(2, "us-west"));
        topo.add_node(Node::new(3, "us-east"));
        assert_eq!(topo.nodes_in_region("us-east").len(), 2);
    }
}
