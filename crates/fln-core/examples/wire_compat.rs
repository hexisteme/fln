//! Print canonical bytes + merkle hash for a fixed thesis.
//! Mirror script: `scripts/wire_compat.py`. The hex outputs must match.

use fln_core::{
    CausalEdge, CausalNode, Domain, EdgeKind, Falsifier, NodeKind, Thesis,
};

fn main() {
    let mut t = Thesis::new("fixed-test", Domain::Invest, "deterministic claim");
    t.created_at = Some("2026-05-20T00:00:00Z".into());
    t.falsifiers.push(Falsifier {
        condition: "x<y".into(),
        deadline: Some("2026-06-01".into()),
        triggered: false,
    });
    t.causal_dag
        .add_node(CausalNode { id: "A".into(), label: "node-A".into(), kind: NodeKind::Cause })
        .unwrap();
    t.causal_dag
        .add_node(CausalNode { id: "B".into(), label: "node-B".into(), kind: NodeKind::Effect })
        .unwrap();
    t.causal_dag
        .add_edge(CausalEdge { from: "A".into(), to: "B".into(), kind: EdgeKind::Direct })
        .unwrap();

    let bytes = t.canonical_bytes().unwrap();
    let node = t.to_merkle_node(vec![]).unwrap();
    println!("canonical_bytes_hex {}", hex::encode(&bytes));
    println!("merkle_hash_hex     {}", hex::encode(node.hash()));
}
