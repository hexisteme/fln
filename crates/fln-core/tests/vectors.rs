//! Cross-language test vectors — Rust side of the wire-compat assertion.
//!
//! Reads ``tests/vectors/v1/manifest.json`` (at the workspace root) and, for
//! each case, verifies that ``MerkleNode { payload: <thesis JSON>, parents: [] }``
//! hashes to ``merkle_hash_hex``. Python's ``python/fln/tests/test_vectors.py``
//! does the same against the same manifest, so any drift between the two
//! implementations breaks CI.

use fln_core::MerkleNode;
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
    thesis: String,
    canonical_bytes_hex: String,
    merkle_hash_hex: String,
}

fn vectors_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = /workspace/crates/fln-core
    let dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(dir).join("..").join("..").join("tests").join("vectors").join("v1")
}

#[test]
fn manifest_v1_round_trips() {
    let root = vectors_root();
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.version, 1, "expected manifest v1");
    assert!(!manifest.cases.is_empty(), "manifest has no cases");

    for c in manifest.cases {
        let payload = fs::read(root.join(&c.thesis))
            .unwrap_or_else(|e| panic!("read {}: {e}", c.thesis));
        assert_eq!(
            hex::encode(&payload),
            c.canonical_bytes_hex,
            "canonical bytes drift for case `{}`",
            c.name
        );

        let node = MerkleNode { payload, parents: vec![] };
        assert_eq!(
            hex::encode(node.hash()),
            c.merkle_hash_hex,
            "merkle hash drift for case `{}`",
            c.name
        );
    }
}
