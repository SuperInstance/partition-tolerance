//! # partition-tolerance
//!
//! Network partition simulation: partition detector, quorum checking,
//! stale read detection, healing protocols.
//!
//! ## Modules
//! - `partition` — Network partition detection and simulation
//! - `quorum` — Quorum checking for reads and writes
//! - `stale` — Stale read detection
//! - `heal` — Partition healing protocols
//! - `topology` — Network topology management

pub mod partition;
pub mod quorum;
pub mod stale;
pub mod heal;
pub mod topology;

pub use partition::{PartitionDetector, Partition, PartitionState};
pub use quorum::{QuorumChecker, QuorumResult};
pub use stale::{StaleReadDetector, ReadResult};
pub use heal::{HealingProtocol, HealingState};
pub use topology::{Topology, Node};
