//! FLN — Falsifier Ledger Network core.
//!
//! *모든 고차 의사결정에 기계 검증 가능한 falsifier 와 인과 그래프를
//! 자동 첨부·영속·검증하는 개인 인프라.*
//!
//! ## 4중 베이스
//!
//! - **Popper** falsifiability — [`Falsifier`] 폐기 조건 영속화
//! - **Pearl** do-calculus — [`CausalDAG`] 인과 그래프
//! - **Merkle** DAG — [`MerkleNode`] + [`Ledger`] append-only 영속
//! - **Bayesian update** — [`causal_decay_weight`] reflexivity weight
//!
//! ## Quickstart
//!
//! ```
//! use fln_core::{Thesis, Domain, KeyPair, Ledger};
//!
//! let mut t = Thesis::new("btc-q2", Domain::Invest, "BTC ≥ 150k within 90d");
//! let kp = KeyPair::generate();
//! let claim = t.sign(&kp).unwrap();
//! assert!(claim.verify());
//!
//! let mut ledger = Ledger::new();
//! let node = t.to_merkle_node(vec![]).unwrap();
//! ledger.append(node);
//! assert!(ledger.root().is_some());
//! ```

pub mod anchor;
pub mod canonical;
pub mod causal;
pub mod decay;
pub mod ledger;
pub mod merkle;
pub mod sign;
pub mod thesis;

pub use anchor::{Anchor, AnchorPayload};
pub use canonical::{CanonicalError, is_strict_iso8601_utc, validate_canonical_bytes};
pub use causal::{CausalDAG, CausalEdge, CausalError, CausalNode, EdgeKind, NodeKind};
pub use decay::{
    CausalDecayParams, DecayError, causal_decay_weight, try_causal_decay_weight,
};
pub use ledger::Ledger;
pub use merkle::{Hash, MerkleNode, merkle_root};
pub use sign::{KeyPair, SignedClaim};
pub use thesis::{Domain, Falsifier, Thesis};
