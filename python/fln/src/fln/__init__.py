"""FLN — Falsifier Ledger Network (Python reference).

Wire-compatible with the Rust ``fln-core`` crate: identical SHA-256 Merkle
layout, identical canonical JSON serialization for theses, identical
Ed25519 signature bytes.
"""

from .anchor import Anchor, AnchorPayload
from .canonical import CanonicalError, is_strict_iso8601_utc, validate_canonical_bytes
from .causal import CausalDAG, CausalEdge, CausalError, CausalNode, EdgeKind, NodeKind
from .decay import (
    CausalDecayParams,
    DecayError,
    causal_decay_weight,
    try_causal_decay_weight,
)
from .ledger import Ledger
from .merkle import MerkleNode, merkle_root
from .sign import KeyPair, SignedClaim
from .thesis import Domain, Falsifier, Thesis

__version__ = "0.2.1"

__all__ = [
    "Anchor",
    "AnchorPayload",
    "CanonicalError",
    "is_strict_iso8601_utc",
    "validate_canonical_bytes",
    "CausalDAG",
    "CausalEdge",
    "CausalError",
    "CausalNode",
    "EdgeKind",
    "NodeKind",
    "CausalDecayParams",
    "DecayError",
    "causal_decay_weight",
    "try_causal_decay_weight",
    "Ledger",
    "MerkleNode",
    "merkle_root",
    "KeyPair",
    "SignedClaim",
    "Domain",
    "Falsifier",
    "Thesis",
]
