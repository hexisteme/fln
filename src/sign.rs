use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct KeyPair {
    signing: SigningKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        Self { signing: SigningKey::generate(&mut OsRng) }
    }

    pub fn from_bytes(secret: &[u8; 32]) -> Self {
        Self { signing: SigningKey::from_bytes(secret) }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing.verifying_key()
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing.sign(message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedClaim {
    pub payload: Vec<u8>,
    pub signer: [u8; 32],
    /// Ed25519 signature (64 bytes). `Vec<u8>` for serde compatibility.
    pub signature: Vec<u8>,
}

impl SignedClaim {
    pub fn new(keypair: &KeyPair, payload: Vec<u8>) -> Self {
        let signature = keypair.sign(&payload);
        Self {
            payload,
            signer: keypair.verifying_key().to_bytes(),
            signature: signature.to_bytes().to_vec(),
        }
    }

    pub fn verify(&self) -> bool {
        let Ok(vk) = VerifyingKey::from_bytes(&self.signer) else { return false };
        let Ok(sig_bytes) = <[u8; 64]>::try_from(self.signature.as_slice()) else { return false };
        let signature = Signature::from_bytes(&sig_bytes);
        vk.verify(&self.payload, &signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = KeyPair::generate();
        let claim = SignedClaim::new(&kp, b"BTC entry thesis v1".to_vec());
        assert!(claim.verify());
    }

    #[test]
    fn tampered_payload_fails_verify() {
        let kp = KeyPair::generate();
        let mut claim = SignedClaim::new(&kp, b"BTC entry thesis v1".to_vec());
        claim.payload[0] ^= 0xFF;
        assert!(!claim.verify());
    }
}
