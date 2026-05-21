//! Cross-language topological-order determinism (Rust side).
//!
//! Paired with `python/fln/tests/test_topo_vectors.py` — both must produce
//! identical orderings on every case so cross-language consensus holds.

use fln_core::{CausalDAG, CausalEdge, CausalNode, EdgeKind, NodeKind};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Manifest {
    version: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    name: String,
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
    expected_order: Vec<String>,
}

fn vectors_path() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(dir)
        .join("..")
        .join("..")
        .join("tests")
        .join("vectors")
        .join("v1")
        .join("topo_order.json")
}

#[test]
fn topological_order_matches_fixture() {
    let bytes = fs::read(vectors_path()).unwrap();
    let manifest: Manifest = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(manifest.version, 1);
    for case in manifest.cases {
        let mut g = CausalDAG::new();
        for id in &case.nodes {
            g.add_node(CausalNode {
                id: id.clone(),
                label: id.clone(),
                kind: NodeKind::Cause,
            })
            .unwrap();
        }
        for (a, b) in &case.edges {
            g.add_edge(CausalEdge {
                from: a.clone(),
                to: b.clone(),
                kind: EdgeKind::Direct,
            })
            .unwrap();
        }
        let actual = g.topological_order().expect("DAG should have a topo order");
        assert_eq!(
            actual, case.expected_order,
            "topo drift on case `{}`",
            case.name
        );
    }
}
