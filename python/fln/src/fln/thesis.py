"""Thesis — wire-compatible with ``fln-core::thesis``.

Canonical JSON serialization matches the Rust ``serde_json::to_vec`` output
byte-for-byte: compact separators, declaration-order fields,
``Vec<u8>``/``[u8; N]`` serialized as arrays of integers.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from enum import Enum

from .causal import CausalDAG, CausalEdge, CausalError, CausalNode, EdgeKind, NodeKind
from .decay import CausalDecayParams
from .merkle import Hash, MerkleNode
from .sign import KeyPair, SignedClaim


class Domain(str, Enum):
    INVEST = "invest"
    HEALTH = "health"
    REAL_ESTATE = "real_estate"
    POLICY = "policy"
    SCIENCE = "science"
    ENGINEERING = "engineering"

    @property
    def default_tau_days(self) -> float:
        return _DOMAIN_TAU[self]


_DOMAIN_TAU: dict[Domain, float] = {
    Domain.INVEST: 180.0,
    Domain.HEALTH: 730.0,
    Domain.REAL_ESTATE: 365.0,
    Domain.POLICY: 365.0,
    Domain.SCIENCE: 1825.0,
    Domain.ENGINEERING: 365.0,
}


@dataclass
class Falsifier:
    condition: str
    deadline: str | None = None
    triggered: bool = False


WIRE_VERSION = 1


@dataclass
class Thesis:
    id: str
    domain: Domain
    claim: str
    version: int = WIRE_VERSION
    falsifiers: list[Falsifier] = field(default_factory=list)
    causal_dag: CausalDAG = field(default_factory=CausalDAG)
    decay: CausalDecayParams = field(default_factory=CausalDecayParams)
    weight: float = 0.0
    created_at: str | None = None
    nonce: str | None = None

    @classmethod
    def new(cls, id: str, domain: Domain, claim: str) -> "Thesis":
        return cls(
            id=id,
            domain=domain,
            claim=claim,
            decay=CausalDecayParams(tau_days=domain.default_tau_days),
        )

    @classmethod
    def with_random_nonce(cls, thesis: "Thesis") -> "Thesis":
        """Return a copy of ``thesis`` with a fresh 16-byte hex nonce attached."""
        import os
        thesis.nonce = os.urandom(16).hex()
        return thesis

    def to_canonical_dict(self) -> dict:
        """Match the serde_json struct serialization (declaration order).

        v0.2 places ``version`` as the first canonical field. v0.2.1 emits
        the optional ``nonce`` field only when set, preserving byte-equal
        canonical bytes for nonce-less theses.
        """
        out: dict = {
            "version": self.version,
            "id": self.id,
            "domain": self.domain.value,
            "claim": self.claim,
            "falsifiers": [
                {"condition": f.condition, "deadline": f.deadline, "triggered": f.triggered}
                for f in self.falsifiers
            ],
            "causal_dag": {
                "nodes": [
                    {"id": n.id, "label": n.label, "kind": n.kind.value}
                    for n in self.causal_dag.nodes
                ],
                "edges": [
                    {"from": e.from_, "to": e.to, "kind": e.kind.value}
                    for e in self.causal_dag.edges
                ],
            },
            "decay": {
                "tau_days": self.decay.tau_days,
                "alpha": self.decay.alpha,
                "regime_shift_threshold": self.decay.regime_shift_threshold,
            },
            "weight": self.weight,
            "created_at": self.created_at,
        }
        if self.nonce is not None:
            out["nonce"] = self.nonce
        return out

    def canonical_bytes(self) -> bytes:
        # allow_nan=False — reject NaN/Inf in canonical bytes (v0.2 hardening).
        return json.dumps(
            self.to_canonical_dict(),
            separators=(",", ":"),
            allow_nan=False,
        ).encode("utf-8")

    def to_merkle_node(self, parents: list[Hash] | None = None) -> MerkleNode:
        return MerkleNode(payload=self.canonical_bytes(), parents=list(parents or []))

    def sign(self, keypair: KeyPair) -> SignedClaim:
        return SignedClaim.new(keypair, self.canonical_bytes())


__all__ = [
    "Domain",
    "Falsifier",
    "Thesis",
    "WIRE_VERSION",
    "CausalDAG",
    "CausalEdge",
    "CausalError",
    "CausalNode",
    "EdgeKind",
    "NodeKind",
]
