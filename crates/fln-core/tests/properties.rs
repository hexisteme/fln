//! Property-based tests — proptest invariants over the core primitives.

use fln_core::{
    CausalDAG, CausalDecayParams, CausalEdge, CausalNode, EdgeKind, KeyPair, Ledger, MerkleNode,
    NodeKind, SignedClaim, causal_decay_weight, merkle_root,
};
use proptest::collection::vec;
use proptest::prelude::*;

proptest! {
    #[test]
    fn merkle_hash_is_deterministic(payload in vec(any::<u8>(), 0..256)) {
        let node = MerkleNode { payload: payload.clone(), parents: vec![] };
        prop_assert_eq!(node.hash(), node.hash());
    }

    #[test]
    fn merkle_root_is_deterministic(leaves in vec(any::<[u8; 32]>(), 1..32)) {
        prop_assert_eq!(merkle_root(&leaves), merkle_root(&leaves));
    }

    #[test]
    fn distinct_payloads_yield_distinct_hashes(
        a in vec(any::<u8>(), 1..64),
        b in vec(any::<u8>(), 1..64),
    ) {
        prop_assume!(a != b);
        let h_a = MerkleNode { payload: a, parents: vec![] }.hash();
        let h_b = MerkleNode { payload: b, parents: vec![] }.hash();
        prop_assert_ne!(h_a, h_b);
    }

    #[test]
    fn ledger_root_changes_after_append(
        existing in vec(vec(any::<u8>(), 1..32), 1..16),
        new_payload in vec(any::<u8>(), 1..32),
    ) {
        let mut ledger = Ledger::new();
        for p in &existing {
            ledger.append(MerkleNode { payload: p.clone(), parents: vec![] });
        }
        let before = ledger.root();
        ledger.append(MerkleNode { payload: new_payload, parents: vec![] });
        let after = ledger.root();
        prop_assert_ne!(before, after);
    }

    #[test]
    fn sign_verify_roundtrip_for_any_payload(payload in vec(any::<u8>(), 0..512)) {
        let kp = KeyPair::generate();
        let claim = SignedClaim::new(&kp, payload);
        prop_assert!(claim.verify());
    }

    #[test]
    fn flipping_one_payload_byte_breaks_verify(
        payload in vec(any::<u8>(), 1..256),
        idx in 0usize..256,
    ) {
        let idx = idx % payload.len();
        let kp = KeyPair::generate();
        let mut claim = SignedClaim::new(&kp, payload.clone());
        claim.payload[idx] ^= 0xFF;
        prop_assert!(!claim.verify());
    }

    #[test]
    fn add_edge_never_introduces_cycle(
        n in 2usize..16,
        seed in any::<u64>(),
    ) {
        // Build a random DAG with n nodes and up to 3n edge attempts.
        let mut g = CausalDAG::new();
        for i in 0..n {
            g.add_node(CausalNode {
                id: format!("N{i}"),
                label: format!("N{i}"),
                kind: NodeKind::Cause,
            }).unwrap();
        }
        let mut rng = LcgRng::new(seed);
        for _ in 0..(3 * n) {
            let from = rng.next() as usize % n;
            let to = rng.next() as usize % n;
            if from == to { continue; }
            let _ = g.add_edge(CausalEdge {
                from: format!("N{from}"),
                to:   format!("N{to}"),
                kind: EdgeKind::Direct,
            });
        }
        // topological_order must succeed because add_edge rejects cycles.
        prop_assert!(g.topological_order().is_some());
    }

    #[test]
    fn decay_weight_stays_in_bounds(
        prev_weight in -1.0f64..=1.0,
        delta_days in 0.1f64..=2000.0,
        outcome in -1.0f64..=1.0,
        regime in 0.0f64..=50.0,
    ) {
        let params = CausalDecayParams::default();
        let w = causal_decay_weight(prev_weight, delta_days, outcome, regime, &params);
        // weight is a sum of two bounded terms; verify it stays finite.
        prop_assert!(w.is_finite());
        // for any reasonable input, |w| ≤ |prev| + |alpha · outcome|
        prop_assert!(w.abs() <= prev_weight.abs() + params.alpha * outcome.abs() + 1e-12);
    }

    #[test]
    fn regime_shift_zeroes_memory_term(
        prev_weight in -1.0f64..=1.0,
        delta_days in 0.1f64..=2000.0,
    ) {
        let params = CausalDecayParams::default();
        let w = causal_decay_weight(prev_weight, delta_days, 0.0, 100.0, &params);
        // outcome=0, regime above threshold → entire formula collapses to 0.
        prop_assert!(w.abs() < 1e-12);
    }
}

/// Tiny LCG so the property test stays deterministic without an extra dep.
struct LcgRng(u64);
impl LcgRng {
    fn new(seed: u64) -> Self { Self(seed.wrapping_add(0x9E3779B97F4A7C15)) }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
}
