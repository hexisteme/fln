# Changelog

## 0.1.0 — 2026-05-20

Initial release. L0 Rust crate per FLN v2.1 spec.

- `merkle` — `MerkleNode` (payload + parents hashing) + `merkle_root` (odd-leaf duplication)
- `sign` — `KeyPair` (Ed25519) + `SignedClaim` (serializable, tamper-detecting)
- `ledger` — `Ledger` append-only with cached root and `verify_integrity`
- `causal` — `CausalDAG` with cycle-rejecting `add_edge` and Kahn topological order
- `decay` — `causal_decay_weight` Soros reflexivity formula with regime-shift wipe
- `thesis` — `Thesis` integrating Popper falsifier + Pearl DAG + Bayesian weight,
  domain-specific tau defaults (Invest 180 / Health 730 / RealEstate 365 / etc.)
- `examples/btc_thesis.rs` — end-to-end demo (sign → append → topo → decay → regime shift)
