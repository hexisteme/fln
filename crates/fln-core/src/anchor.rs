//! Public anchor — a signed record binding (ledger_root, entry_count,
//! anchored_at) optionally chained to the previous anchor.
//!
//! ## v0.2 hardening — anchor chain integrity
//!
//! v0.1 anchors didn't link to predecessors. A compromised signer could
//! publish two valid anchors for the same root with different counts and
//! observers had no way to detect the fork. v0.2 introduces
//! `prev_anchor_hash: Option<[u8; 32]>` (None for the first/genesis anchor),
//! turning the anchor sequence into a hash chain.

use crate::merkle::Hash;
use crate::sign::{KeyPair, SignedClaim};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorPayload {
    pub version: u32,
    pub ledger_root: Hash,
    pub entry_count: u64,
    pub anchored_at: String,
    pub prev_anchor_hash: Option<Hash>,
}

impl AnchorPayload {
    pub fn canonical_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub version: u32,
    pub ledger_root: Hash,
    pub entry_count: u64,
    pub anchored_at: String,
    pub prev_anchor_hash: Option<Hash>,
    pub signer: [u8; 32],
    pub signature: Vec<u8>,
}

impl Anchor {
    pub const WIRE_VERSION: u32 = 1;

    pub fn new(
        keypair: &KeyPair,
        ledger_root: Hash,
        entry_count: u64,
        anchored_at: impl Into<String>,
        prev_anchor_hash: Option<Hash>,
    ) -> serde_json::Result<Self> {
        let anchored_at: String = anchored_at.into();
        let payload = AnchorPayload {
            version: Self::WIRE_VERSION,
            ledger_root,
            entry_count,
            anchored_at: anchored_at.clone(),
            prev_anchor_hash,
        };
        let claim = SignedClaim::new(keypair, payload.canonical_bytes()?);
        Ok(Self {
            version: Self::WIRE_VERSION,
            ledger_root,
            entry_count,
            anchored_at,
            prev_anchor_hash,
            signer: claim.signer,
            signature: claim.signature,
        })
    }

    pub fn payload(&self) -> AnchorPayload {
        AnchorPayload {
            version: self.version,
            ledger_root: self.ledger_root,
            entry_count: self.entry_count,
            anchored_at: self.anchored_at.clone(),
            prev_anchor_hash: self.prev_anchor_hash,
        }
    }

    /// Stable digest of the anchor payload — used as `prev_anchor_hash` by
    /// the next anchor in the chain.
    pub fn payload_hash(&self) -> serde_json::Result<Hash> {
        let bytes = self.payload().canonical_bytes()?;
        Ok(Sha256::digest(&bytes).into())
    }

    pub fn verify(&self) -> bool {
        if self.version != Self::WIRE_VERSION {
            return false;
        }
        let Ok(bytes) = self.payload().canonical_bytes() else { return false };
        let Ok(vk) = VerifyingKey::from_bytes(&self.signer) else { return false };
        let Ok(sig_bytes) = <[u8; 64]>::try_from(self.signature.as_slice()) else { return false };
        let signature = Signature::from_bytes(&sig_bytes);
        vk.verify(&bytes, &signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_anchor_verifies_with_no_prev() {
        let kp = KeyPair::generate();
        let a = Anchor::new(&kp, [7u8; 32], 42, "2026-05-21T12:00:00Z", None).unwrap();
        assert!(a.verify());
        assert_eq!(a.prev_anchor_hash, None);
        assert_eq!(a.version, Anchor::WIRE_VERSION);
    }

    #[test]
    fn chained_anchor_verifies() {
        let kp = KeyPair::generate();
        let a1 = Anchor::new(&kp, [1u8; 32], 1, "2026-05-21T00:00:00Z", None).unwrap();
        let prev = a1.payload_hash().unwrap();
        let a2 = Anchor::new(&kp, [2u8; 32], 2, "2026-05-22T00:00:00Z", Some(prev)).unwrap();
        assert!(a2.verify());
        assert_eq!(a2.prev_anchor_hash, Some(prev));
    }

    #[test]
    fn tampering_with_chain_field_breaks_verify() {
        let kp = KeyPair::generate();
        let mut a = Anchor::new(&kp, [7u8; 32], 42, "2026-05-21T12:00:00Z", None).unwrap();
        a.prev_anchor_hash = Some([0xAB; 32]);
        assert!(!a.verify());
    }

    #[test]
    fn wrong_version_rejected() {
        let kp = KeyPair::generate();
        let mut a = Anchor::new(&kp, [7u8; 32], 1, "2026-05-21T00:00:00Z", None).unwrap();
        a.version = 99;
        assert!(!a.verify());
    }
}
