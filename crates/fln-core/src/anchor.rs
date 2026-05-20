//! Public anchor — a signed record of (ledger_root, entry_count, anchored_at).
//!
//! Anchors are the wire format for the FLN L2 → L5 bridge: publish them to a
//! Pages site, an OTS service, or a public Git repo to create a tamper-evident
//! timeline of ledger states.

use crate::merkle::Hash;
use crate::sign::{KeyPair, SignedClaim};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayload {
    pub ledger_root: Hash,
    pub entry_count: u64,
    pub anchored_at: String,
}

impl AnchorPayload {
    pub fn canonical_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub ledger_root: Hash,
    pub entry_count: u64,
    pub anchored_at: String,
    pub signer: [u8; 32],
    pub signature: Vec<u8>,
}

impl Anchor {
    pub fn new(
        keypair: &KeyPair,
        ledger_root: Hash,
        entry_count: u64,
        anchored_at: impl Into<String>,
    ) -> serde_json::Result<Self> {
        let anchored_at: String = anchored_at.into();
        let payload = AnchorPayload {
            ledger_root,
            entry_count,
            anchored_at: anchored_at.clone(),
        };
        let claim = SignedClaim::new(keypair, payload.canonical_bytes()?);
        Ok(Self {
            ledger_root,
            entry_count,
            anchored_at,
            signer: claim.signer,
            signature: claim.signature,
        })
    }

    pub fn verify(&self) -> bool {
        let payload = AnchorPayload {
            ledger_root: self.ledger_root,
            entry_count: self.entry_count,
            anchored_at: self.anchored_at.clone(),
        };
        let Ok(bytes) = payload.canonical_bytes() else { return false };
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
    fn anchor_roundtrip_verifies() {
        let kp = KeyPair::generate();
        let root = [7u8; 32];
        let a = Anchor::new(&kp, root, 42, "2026-05-21T12:00:00Z").unwrap();
        assert!(a.verify());
    }

    #[test]
    fn anchor_tamper_breaks_verify() {
        let kp = KeyPair::generate();
        let mut a = Anchor::new(&kp, [7u8; 32], 42, "2026-05-21T12:00:00Z").unwrap();
        a.entry_count = 43;
        assert!(!a.verify());
    }

    #[test]
    fn anchor_json_roundtrip() {
        let kp = KeyPair::generate();
        let a = Anchor::new(&kp, [9u8; 32], 1, "2026-05-21T00:00:00Z").unwrap();
        let s = serde_json::to_string(&a).unwrap();
        let parsed: Anchor = serde_json::from_str(&s).unwrap();
        assert!(parsed.verify());
    }
}
