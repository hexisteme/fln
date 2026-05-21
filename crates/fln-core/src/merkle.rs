//! Merkle DAG primitives.
//!
//! ## v0.2 hardening (CVE-2012-2459 — Bitcoin merkle malleability)
//!
//! v0.1 used Bitcoin-style odd-leaf duplication, which means the trees
//! for `[A, B, C]` and `[A, B, C, C]` share a root — a malleability vector.
//! v0.2 changes:
//!
//! 1. **Domain separation**: leaves are hashed under tag `0x00`, internal
//!    nodes under `0x01`, the final root under `0xFF`.
//! 2. **No leaf duplication**: when a layer has an odd count, the lone last
//!    item is *promoted* to the next layer unchanged (RFC 6962 §2.1 style).
//! 3. **Leaf count bound into the root**: the final root binds
//!    `H(0xFF || be64(leaf_count) || tree_root)`, so two layouts that
//!    would otherwise alias yield distinct roots.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

pub const LEAF_TAG: u8 = 0x00;
pub const NODE_TAG: u8 = 0x01;
pub const ROOT_TAG: u8 = 0xFF;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleNode {
    pub payload: Vec<u8>,
    pub parents: Vec<Hash>,
}

impl MerkleNode {
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update((self.payload.len() as u64).to_be_bytes());
        hasher.update(&self.payload);
        hasher.update((self.parents.len() as u64).to_be_bytes());
        for parent in &self.parents {
            hasher.update(parent);
        }
        hasher.finalize().into()
    }
}

fn hash_leaf(h: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update([LEAF_TAG]);
    hasher.update(h);
    hasher.finalize().into()
}

fn hash_node(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update([NODE_TAG]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub fn merkle_root(leaves: &[Hash]) -> Option<Hash> {
    if leaves.is_empty() {
        return None;
    }
    let count = leaves.len() as u64;
    let mut layer: Vec<Hash> = leaves.iter().map(hash_leaf).collect();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        let mut iter = layer.chunks_exact(2);
        for pair in iter.by_ref() {
            next.push(hash_node(&pair[0], &pair[1]));
        }
        // Promote the lone tail (RFC 6962 §2.1) instead of duplicating it.
        if let [tail] = iter.remainder() {
            next.push(*tail);
        }
        layer = next;
    }
    let tree_root = layer[0];
    let mut final_hasher = Sha256::new();
    final_hasher.update([ROOT_TAG]);
    final_hasher.update(count.to_be_bytes());
    final_hasher.update(tree_root);
    Some(final_hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_hash_is_deterministic() {
        let node = MerkleNode { payload: b"thesis-A".to_vec(), parents: vec![] };
        assert_eq!(node.hash(), node.hash());
    }

    #[test]
    fn merkle_root_handles_odd_leaves() {
        let leaves = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
        let root = merkle_root(&leaves).expect("root for non-empty");
        assert_ne!(root, [0u8; 32]);
    }

    #[test]
    fn merkle_root_empty_returns_none() {
        assert!(merkle_root(&[]).is_none());
    }

    #[test]
    fn odd_leaf_no_longer_aliases_with_duplicated_form() {
        // v0.1 bug: root([A,B,C]) == root([A,B,C,C]). v0.2 fix: must differ.
        let a = [1u8; 32];
        let b = [2u8; 32];
        let c = [3u8; 32];
        let three = merkle_root(&[a, b, c]).unwrap();
        let four_with_dup = merkle_root(&[a, b, c, c]).unwrap();
        assert_ne!(three, four_with_dup, "CVE-2012-2459 must not affect this implementation");
    }

    #[test]
    fn distinct_count_binds_into_root() {
        // Even if a synthetic tree produces the same internal-layer state,
        // root binds `count` so different leaf-counts yield different roots.
        let h = [7u8; 32];
        let one = merkle_root(&[h]).unwrap();
        let two = merkle_root(&[h, h]).unwrap();
        assert_ne!(one, two);
    }
}
