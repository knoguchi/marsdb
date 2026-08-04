//! Large-scale correctness checks, not performance (see benches/graph_ops.rs
//! for that). `#[ignore]`d so `cargo test` stays fast by default; run with
//! `cargo test -p marsdb-graph --test stress -- --ignored --nocapture`.

use std::collections::{BTreeMap, HashSet};

use marsdb_graph::{Direction, GraphStore, NodeId, PropertyValue};

/// Deterministic, dependency-free PRNG (xorshift32) — reproducible without
/// pulling in `rand`/`proptest` for a handful of stress tests.
struct Xorshift32(u32);
impl Xorshift32 {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n
    }
}

#[test]
#[ignore]
fn stress_50k_node_chain_correctness() {
    const N: i64 = 50_000;
    let store = GraphStore::open_memory().unwrap();

    let mut ids = Vec::with_capacity(N as usize);
    let mut prev: Option<NodeId> = None;
    for i in 0..N {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), PropertyValue::Int(i));
        let id = store.create_node(&["Item"], props).unwrap();
        if let Some(p) = prev {
            store.create_edge("NEXT", p, id, BTreeMap::new()).unwrap();
        }
        ids.push(id);
        prev = Some(id);
    }

    let all = store.all_nodes(Some("Item")).unwrap();
    assert_eq!(
        all.len(),
        N as usize,
        "every created node must be scannable"
    );

    // Spot-check traversal correctness at start, middle, end — not just count.
    for &check_idx in &[0usize, N as usize / 2, N as usize - 2] {
        let out = store
            .neighbors(ids[check_idx], Direction::Out, Some("NEXT"))
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].other, ids[check_idx + 1]);

        let inn = store
            .neighbors(ids[check_idx + 1], Direction::In, Some("NEXT"))
            .unwrap();
        assert_eq!(inn.len(), 1);
        assert_eq!(inn[0].other, ids[check_idx]);
    }

    // Last node has no outgoing NEXT edge, first has no incoming.
    assert_eq!(
        store
            .neighbors(*ids.last().unwrap(), Direction::Out, Some("NEXT"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        store
            .neighbors(ids[0], Direction::In, Some("NEXT"))
            .unwrap()
            .len(),
        0
    );
}

#[test]
#[ignore]
fn stress_supernode_fanout_10k() {
    const FANOUT: usize = 10_000;
    let store = GraphStore::open_memory().unwrap();
    let center = store.create_node(&["Hub"], BTreeMap::new()).unwrap();

    let mut leaves = HashSet::with_capacity(FANOUT);
    for i in 0..FANOUT {
        let mut props = BTreeMap::new();
        props.insert("idx".to_string(), PropertyValue::Int(i as i64));
        let leaf = store.create_node(&["Leaf"], props).unwrap();
        // Alternate edge label to exercise label filtering at scale too.
        let label = if i % 2 == 0 { "EVEN" } else { "ODD" };
        store
            .create_edge(label, center, leaf, BTreeMap::new())
            .unwrap();
        leaves.insert(leaf);
    }

    let all_out = store.neighbors(center, Direction::Out, None).unwrap();
    assert_eq!(all_out.len(), FANOUT);
    let returned: HashSet<_> = all_out.iter().map(|e| e.other).collect();
    assert_eq!(returned, leaves);

    let even_out = store
        .neighbors(center, Direction::Out, Some("EVEN"))
        .unwrap();
    assert_eq!(even_out.len(), FANOUT / 2);

    // Detach-delete the hub: every leaf's incoming edge must be gone too.
    assert!(store.delete_node(center, true).unwrap());
    for &leaf in leaves.iter().take(50) {
        assert_eq!(store.neighbors(leaf, Direction::In, None).unwrap().len(), 0);
    }
}

#[test]
#[ignore]
fn stress_random_create_delete_matches_oracle() {
    const OPS: usize = 20_000;
    let store = GraphStore::open_memory().unwrap();
    let mut rng = Xorshift32(0xC0FFEE);

    // Oracle: the set of node ids we believe are currently live.
    let mut live: Vec<NodeId> = Vec::new();

    for _ in 0..OPS {
        let create = live.is_empty() || rng.below(3) != 0; // ~66% create, else delete
        if create {
            let id = store.create_node(&["N"], BTreeMap::new()).unwrap();
            live.push(id);
        } else {
            let idx = rng.below(live.len());
            let id = live.swap_remove(idx);
            assert!(
                store.delete_node(id, true).unwrap(),
                "oracle says {id:?} is live"
            );
        }
    }

    let scanned = store.all_nodes(Some("N")).unwrap();
    assert_eq!(
        scanned.len(),
        live.len(),
        "storage's live node count must match the oracle after {OPS} random ops"
    );
    let scanned_ids: HashSet<_> = scanned.iter().map(|n| n.id).collect();
    let live_ids: HashSet<_> = live.into_iter().collect();
    assert_eq!(
        scanned_ids, live_ids,
        "storage's live node set must match the oracle exactly"
    );
}
