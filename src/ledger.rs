use crate::merkle::{Hash, MerkleNode, merkle_root};
use serde::{Deserialize, Serialize};

/// Append-only ledger of [`MerkleNode`]s with cached batch root.
///
/// L2 storage 의 in-memory primitive — 6h batch anchoring 의 단위가 된다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ledger {
    entries: Vec<MerkleNode>,
    cached_root: Option<Hash>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a node, invalidate cached root, return new entry's hash.
    pub fn append(&mut self, node: MerkleNode) -> Hash {
        let h = node.hash();
        self.entries.push(node);
        self.cached_root = None;
        h
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[MerkleNode] {
        &self.entries
    }

    /// Compute (or return cached) Merkle root over all entry hashes.
    pub fn root(&mut self) -> Option<Hash> {
        if self.cached_root.is_none() {
            let leaves: Vec<Hash> = self.entries.iter().map(|n| n.hash()).collect();
            self.cached_root = merkle_root(&leaves);
        }
        self.cached_root
    }

    /// Recompute root from scratch and compare with the cached value.
    /// Returns `true` when the ledger is intact (no tampering).
    pub fn verify_integrity(&self) -> bool {
        let leaves: Vec<Hash> = self.entries.iter().map(|n| n.hash()).collect();
        let recomputed = merkle_root(&leaves);
        match (recomputed, self.cached_root) {
            (Some(r), Some(c)) => r == c,
            (None, None) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(payload: &[u8]) -> MerkleNode {
        MerkleNode { payload: payload.to_vec(), parents: vec![] }
    }

    #[test]
    fn append_increments_len() {
        let mut l = Ledger::new();
        assert!(l.is_empty());
        l.append(node(b"a"));
        l.append(node(b"b"));
        assert_eq!(l.len(), 2);
    }

    #[test]
    fn root_changes_after_append() {
        let mut l = Ledger::new();
        l.append(node(b"a"));
        let r1 = l.root().unwrap();
        l.append(node(b"b"));
        let r2 = l.root().unwrap();
        assert_ne!(r1, r2);
    }

    #[test]
    fn verify_integrity_after_root() {
        let mut l = Ledger::new();
        l.append(node(b"thesis-1"));
        l.append(node(b"thesis-2"));
        let _ = l.root();
        assert!(l.verify_integrity());
    }

    #[test]
    fn tampering_breaks_integrity() {
        let mut l = Ledger::new();
        l.append(node(b"thesis-1"));
        let _ = l.root();
        l.entries[0].payload[0] ^= 0xFF;
        assert!(!l.verify_integrity());
    }
}
