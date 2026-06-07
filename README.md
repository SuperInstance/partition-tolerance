# partition-tolerance

Network partition simulation: partition detector, quorum checking, stale read detection, healing protocols.

## Features

- **Partition Detection** — Heartbeat-based partition detection with configurable timeouts
- **Quorum Checking** — Majority quorum validation for reads and writes
- **Stale Read Detection** — Version-based stale read identification
- **Healing Protocols** — Multi-phase partition healing (sync → merge → verify)
- **Topology** — Network topology with link blocking and partition simulation

## Modules

| Module | Description |
|--------|-------------|
| `partition` | Network partition detection and simulation |
| `quorum` | Quorum checking for reads and writes |
| `stale` | Stale read detection |
| `heal` | Partition healing protocols |
| `topology` | Network topology management |

## Usage

```rust
use partition_tolerance::topology::Topology;
use partition_tolerance::topology::Node;

let mut topo = Topology::new();
topo.add_node(Node::new(1, "us-east"));
topo.add_node(Node::new(2, "us-west"));
topo.create_partition(&[1], &[2]);
assert!(!topo.can_communicate(1, 2));
topo.heal_all();
assert!(topo.can_communicate(1, 2));
```

## Testing

```bash
cargo test    # 37 tests
cargo clippy  # zero warnings
```

## License

MIT
