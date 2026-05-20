# fln-core

**Falsifier Ledger Network — Rust core.**

*모든 고차 의사결정에 기계 검증 가능한 falsifier 와 인과 그래프를 자동 첨부·영속·검증하는 개인 인프라.*

## 4중 베이스

- **Popper** falsifiability — `Falsifier` 폐기 조건 영속화
- **Pearl** do-calculus — `CausalDAG` 인과 그래프 (cycle 검출 + 위상 정렬)
- **Merkle DAG** — `MerkleNode` + `Ledger` append-only 영속화
- **Bayesian update** — `causal_decay_weight` (Soros reflexivity: regime-shift → memory wipe)

## Quickstart

```rust
use fln_core::{Thesis, Domain, KeyPair, Ledger, causal_decay_weight};
use fln_core::{CausalNode, CausalEdge, NodeKind, EdgeKind};

let mut thesis = Thesis::new(
    "btc-2026-q2-entry",
    Domain::Invest,
    "BTC reaches 150k within 90d if VIX < 20",
);

thesis.causal_dag.add_node(CausalNode {
    id: "VIX".into(), label: "Volatility index".into(), kind: NodeKind::Confounder,
}).unwrap();
thesis.causal_dag.add_node(CausalNode {
    id: "BTC".into(), label: "BTC price".into(), kind: NodeKind::Effect,
}).unwrap();
thesis.causal_dag.add_edge(CausalEdge {
    from: "VIX".into(), to: "BTC".into(), kind: EdgeKind::Direct,
}).unwrap();

let kp = KeyPair::generate();
let claim = thesis.sign(&kp).unwrap();
assert!(claim.verify());

let mut ledger = Ledger::new();
ledger.append(thesis.to_merkle_node(vec![]).unwrap());
let root = ledger.root().unwrap();

// 30 일 후 falsifier 가 부분 확증 (+0.5) 됐고 VIX 정상
let new_weight = causal_decay_weight(
    thesis.weight, 30.0, 0.5, 15.0, &thesis.decay,
);
```

## Causal Decay 수학

```text
w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[regime_signal ≥ threshold])
       + α · falsifier_outcome_t · (1 - exp(-Δt/τ))
```

- `τ` (반감기, 일): Invest 180 / Health 730 / RealEstate 365 / Policy 365 / Science 1825 / Engineering 365
- `regime_signal ≥ threshold` (default VIX ≥ 30) → 이전 weight 즉시 망각
- `α` (학습률, default 0.1)

## License

MIT OR Apache-2.0
