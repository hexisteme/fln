//! Property-based tests — proptest invariants over the core primitives.

use fln_core::{
    Anchor, CausalDAG, CausalDecayParams, CausalEdge, CausalNode, CanonicalError, EdgeKind,
    KeyPair, Ledger, MerkleNode, NodeKind, SignedClaim, causal_decay_weight, merkle_root,
    try_causal_decay_weight, validate_canonical_bytes,
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

    // === v0.2 hardening invariants ===

    #[test]
    fn merkle_root_not_aliased_by_last_leaf_duplication(
        leaves in vec(any::<[u8; 32]>(), 1..16),
    ) {
        // CVE-2012-2459 regression: root([..., L]) ≠ root([..., L, L]).
        let mut dup = leaves.clone();
        dup.push(*leaves.last().unwrap());
        prop_assert_ne!(merkle_root(&leaves), merkle_root(&dup));
    }

    #[test]
    fn merkle_root_count_is_bound(h in any::<[u8; 32]>()) {
        // Same single-leaf hash, different counts → different roots.
        prop_assert_ne!(merkle_root(&[h]), merkle_root(&[h, h]));
    }

    #[test]
    fn try_decay_rejects_negative_delta(
        prev in -1.0f64..=1.0,
        delta in -1000.0f64..0.0,
        outcome in -1.0f64..=1.0,
    ) {
        let params = CausalDecayParams::default();
        prop_assert!(try_causal_decay_weight(prev, delta, outcome, 0.0, &params).is_err());
    }

    #[test]
    fn try_decay_rejects_outcome_out_of_range(
        prev in -1.0f64..=1.0,
        delta in 0.0f64..=1000.0,
        outcome in 1.0001f64..1000.0,
    ) {
        let params = CausalDecayParams::default();
        prop_assert!(try_causal_decay_weight(prev, delta, outcome, 0.0, &params).is_err());
    }

    #[test]
    fn canonical_validator_accepts_well_formed_thesis(
        id in "[a-z0-9]{1,16}",
        claim in "[A-Za-z0-9 ]{1,32}",
    ) {
        use fln_core::{Domain, Thesis};
        let mut t = Thesis::new(id, Domain::Invest, claim);
        t.created_at = Some("2026-05-21T00:00:00Z".into());
        let bytes = t.canonical_bytes().unwrap();
        prop_assert!(validate_canonical_bytes(&bytes).is_ok());
    }

    #[test]
    fn canonical_validator_rejects_duplicate_keys(
        a in "[a-z]{3,10}",
        b in "[a-z]{3,10}",
    ) {
        // Inject a duplicate key into otherwise-valid JSON.
        let hostile = format!(r#"{{"version":1,"id":"{a}","id":"{b}"}}"#);
        prop_assert!(matches!(
            validate_canonical_bytes(hostile.as_bytes()),
            Err(CanonicalError::DuplicateKey(_))
        ));
    }

    #[test]
    fn anchor_chain_payload_hash_is_deterministic(
        root in any::<[u8; 32]>(),
        count in any::<u64>(),
    ) {
        let kp = KeyPair::generate();
        let a = Anchor::new(&kp, root, count, "2026-05-21T00:00:00Z", None).unwrap();
        let h1 = a.payload_hash().unwrap();
        let h2 = a.payload_hash().unwrap();
        prop_assert_eq!(h1, h2);
        // Chained anchor verifies and carries the prev hash.
        let b = Anchor::new(&kp, root, count.wrapping_add(1), "2026-05-22T00:00:00Z", Some(h1)).unwrap();
        prop_assert!(b.verify());
        prop_assert_eq!(b.prev_anchor_hash, Some(h1));
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
