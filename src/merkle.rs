use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

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

pub fn merkle_root(leaves: &[Hash]) -> Option<Hash> {
    if leaves.is_empty() {
        return None;
    }
    let mut layer: Vec<Hash> = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        for pair in layer.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(pair[0]);
            hasher.update(pair.get(1).unwrap_or(&pair[0]));
            next.push(hasher.finalize().into());
        }
        layer = next;
    }
    Some(layer[0])
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
}
