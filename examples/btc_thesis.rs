//! End-to-end FLN example — BTC entry thesis with falsifier, causal DAG,
//! sovereign signature, ledger append, and Causal Decay weight update.
//!
//! Run: `cargo run --example btc_thesis`

use fln_core::{
    CausalEdge, CausalNode, Domain, EdgeKind, Falsifier, KeyPair, Ledger, NodeKind, Thesis,
    causal_decay_weight,
};

fn main() {
    // 1. Thesis
    let mut thesis = Thesis::new(
        "btc-2026-q2-entry",
        Domain::Invest,
        "BTC reaches 150k within 90d if VIX < 20 holds",
    );
    thesis.created_at = Some("2026-05-20T23:30:00Z".into());

    // 2. Popper — falsifier
    thesis.falsifiers.push(Falsifier {
        condition: "BTC/USD < 80000 at any 1D close".into(),
        deadline: Some("2026-09-01".into()),
        triggered: false,
    });

    // 3. Pearl — causal DAG
    thesis
        .causal_dag
        .add_node(CausalNode {
            id: "VIX".into(),
            label: "Volatility index".into(),
            kind: NodeKind::Confounder,
        })
        .unwrap();
    thesis
        .causal_dag
        .add_node(CausalNode {
            id: "ETF_FLOW".into(),
            label: "Spot ETF net inflow".into(),
            kind: NodeKind::Cause,
        })
        .unwrap();
    thesis
        .causal_dag
        .add_node(CausalNode {
            id: "BTC".into(),
            label: "BTC price".into(),
            kind: NodeKind::Effect,
        })
        .unwrap();
    thesis
        .causal_dag
        .add_edge(CausalEdge {
            from: "VIX".into(),
            to: "BTC".into(),
            kind: EdgeKind::Direct,
        })
        .unwrap();
    thesis
        .causal_dag
        .add_edge(CausalEdge {
            from: "ETF_FLOW".into(),
            to: "BTC".into(),
            kind: EdgeKind::Direct,
        })
        .unwrap();

    // 4. Ed25519 sovereign signature
    let kp = KeyPair::generate();
    let claim = thesis.sign(&kp).unwrap();
    assert!(claim.verify(), "signature must verify");

    // 5. Merkle ledger append
    let mut ledger = Ledger::new();
    let h = ledger.append(thesis.to_merkle_node(vec![]).unwrap());
    let root = ledger.root().expect("root for non-empty ledger");

    println!("thesis id      = {}", thesis.id);
    println!("entry hash     = {}", hex::encode(h));
    println!("ledger root    = {}", hex::encode(root));
    println!("signer pubkey  = {}", hex::encode(claim.signer));
    println!(
        "topo(causal)   = {:?}",
        thesis.causal_dag.topological_order().unwrap()
    );

    // 6. Bayesian/Causal Decay — 30d 후 부분 확증
    let w_30d = causal_decay_weight(thesis.weight, 30.0, 0.5, 15.0, &thesis.decay);
    println!("weight @ t+30d = {w_30d:.4}");

    // 7. Regime shift simulation — VIX 35 → weight 망각
    let w_after_shock = causal_decay_weight(w_30d, 5.0, 0.0, 35.0, &thesis.decay);
    println!("weight @ shock = {w_after_shock:.4}");
    assert!(w_after_shock.abs() < 1e-9, "regime shift must wipe memory");
}
