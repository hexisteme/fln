use crate::causal::CausalDAG;
use crate::decay::CausalDecayParams;
use crate::merkle::MerkleNode;
use crate::sign::{KeyPair, SignedClaim};
use serde::{Deserialize, Serialize};

/// Domain category — 6 도메인 spec (FLN v2.1 §L1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Invest,
    Health,
    RealEstate,
    Policy,
    Science,
    Engineering,
}

impl Domain {
    /// Default tau (decay 반감기, 일) per domain — FLN v2.1 §Causal Decay.
    pub fn default_tau_days(self) -> f64 {
        match self {
            Domain::Invest => 180.0,
            Domain::Health => 730.0,
            Domain::RealEstate => 365.0,
            Domain::Policy => 365.0,
            Domain::Science => 1825.0,
            Domain::Engineering => 365.0,
        }
    }
}

/// Falsifier condition — thesis 폐기 트리거. 기계 검증 가능 형식.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Falsifier {
    /// e.g. "BTC/USD < 80000 at any 1D close"
    pub condition: String,
    /// optional ISO8601 — 시간 기반 falsifier ("by 2026-09-01").
    pub deadline: Option<String>,
    /// fired/not.
    pub triggered: bool,
}

/// Thesis — FLN 의 영속 ledger entry. Pearl/Popper/Bayesian 4중 베이스.
///
/// v0.2 adds `version` as the first canonical field — a forward-compatibility
/// marker so future on-wire changes can fail cleanly instead of silently
/// producing colliding canonical bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thesis {
    pub version: u32,
    pub id: String,
    pub domain: Domain,
    pub claim: String,
    pub falsifiers: Vec<Falsifier>,
    pub causal_dag: CausalDAG,
    pub decay: CausalDecayParams,
    /// posterior weight ∈ [-1, 1] — Causal Decay 누적값.
    pub weight: f64,
    /// optional ISO8601 of creation.
    pub created_at: Option<String>,
    /// Optional anti-replay nonce (hex of ≥ 16 random bytes). Wire-additive:
    /// theses without a nonce omit the field entirely so existing v0.2 test
    /// vectors are byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl Thesis {
    pub const WIRE_VERSION: u32 = 1;

    pub fn new(id: impl Into<String>, domain: Domain, claim: impl Into<String>) -> Self {
        let decay = CausalDecayParams {
            tau_days: domain.default_tau_days(),
            ..CausalDecayParams::default()
        };
        Self {
            version: Self::WIRE_VERSION,
            id: id.into(),
            domain,
            claim: claim.into(),
            falsifiers: Vec::new(),
            causal_dag: CausalDAG::new(),
            decay,
            weight: 0.0,
            created_at: None,
            nonce: None,
        }
    }

    /// Attach a fresh 16-byte random nonce. Idempotent on subsequent calls.
    pub fn with_random_nonce(mut self) -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let mut hex = String::with_capacity(32);
        for b in bytes {
            hex.push_str(&format!("{b:02x}"));
        }
        self.nonce = Some(hex);
        self
    }

    /// Canonical bytes — deterministic serialization for hashing/signing.
    pub fn canonical_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }

    /// Build a [`MerkleNode`] for ledger append.
    pub fn to_merkle_node(
        &self,
        parents: Vec<crate::merkle::Hash>,
    ) -> serde_json::Result<MerkleNode> {
        Ok(MerkleNode { payload: self.canonical_bytes()?, parents })
    }

    /// Sign canonical bytes with sovereign Ed25519 key.
    pub fn sign(&self, keypair: &KeyPair) -> serde_json::Result<SignedClaim> {
        Ok(SignedClaim::new(keypair, self.canonical_bytes()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal::{CausalEdge, CausalNode, EdgeKind, NodeKind};
    use crate::decay::causal_decay_weight;

    fn sample_thesis() -> Thesis {
        let mut t = Thesis::new(
            "btc-2026-q2-entry",
            Domain::Invest,
            "BTC reaches 150k within 90d if VIX < 20",
        );
        t.falsifiers.push(Falsifier {
            condition: "BTC/USD < 80000 at any 1D close".into(),
            deadline: Some("2026-09-01".into()),
            triggered: false,
        });
        t.causal_dag
            .add_node(CausalNode {
                id: "VIX".into(),
                label: "VIX".into(),
                kind: NodeKind::Confounder,
            })
            .unwrap();
        t.causal_dag
            .add_node(CausalNode {
                id: "BTC".into(),
                label: "BTC price".into(),
                kind: NodeKind::Effect,
            })
            .unwrap();
        t.causal_dag
            .add_edge(CausalEdge {
                from: "VIX".into(),
                to: "BTC".into(),
                kind: EdgeKind::Direct,
            })
            .unwrap();
        t
    }

    #[test]
    fn thesis_canonical_bytes_are_deterministic() {
        let t = sample_thesis();
        let a = t.canonical_bytes().unwrap();
        let b = t.canonical_bytes().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn thesis_sign_verify_roundtrip() {
        let t = sample_thesis();
        let kp = KeyPair::generate();
        let claim = t.sign(&kp).unwrap();
        assert!(claim.verify());
    }

    #[test]
    fn domain_tau_matches_spec() {
        assert_eq!(Domain::Invest.default_tau_days(), 180.0);
        assert_eq!(Domain::Health.default_tau_days(), 730.0);
        assert_eq!(Domain::RealEstate.default_tau_days(), 365.0);
    }

    #[test]
    fn weight_update_uses_thesis_decay_params() {
        let mut t = sample_thesis();
        let new_w = causal_decay_weight(t.weight, 30.0, 0.5, 10.0, &t.decay);
        t.weight = new_w;
        assert!(t.weight > 0.0);
    }

    #[test]
    fn version_field_present_in_canonical_bytes() {
        let t = sample_thesis();
        let bytes = t.canonical_bytes().unwrap();
        // version is the first field in the canonical JSON
        assert!(bytes.starts_with(br#"{"version":1,"#), "{:?}", &bytes[..20]);
    }

    #[test]
    fn nonce_is_omitted_when_none() {
        // Wire-additivity: theses without a nonce produce v0.2-identical bytes.
        let t = sample_thesis();
        let bytes = t.canonical_bytes().unwrap();
        assert!(!bytes.windows(7).any(|w| w == b"\"nonce\""));
    }

    #[test]
    fn nonce_is_included_when_set() {
        let t = sample_thesis().with_random_nonce();
        let bytes = t.canonical_bytes().unwrap();
        assert!(bytes.windows(7).any(|w| w == b"\"nonce\""));
        // Different random nonces yield different bytes (replay protection).
        let other = sample_thesis().with_random_nonce();
        assert_ne!(bytes, other.canonical_bytes().unwrap());
    }
}
