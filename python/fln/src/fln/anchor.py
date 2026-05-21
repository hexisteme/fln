"""Anchor — wire-compatible with ``fln-core::anchor``.

v0.2 introduces ``prev_anchor_hash`` so anchors form a hash chain and
a forking signer is detectable by observers.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PublicKey,
)

from .merkle import Hash
from .sign import KeyPair, SignedClaim

WIRE_VERSION = 1


@dataclass
class AnchorPayload:
    version: int
    ledger_root: Hash
    entry_count: int
    anchored_at: str
    prev_anchor_hash: Hash | None

    def canonical_bytes(self) -> bytes:
        return json.dumps(
            {
                "version": self.version,
                "ledger_root": list(self.ledger_root),
                "entry_count": self.entry_count,
                "anchored_at": self.anchored_at,
                "prev_anchor_hash": (
                    list(self.prev_anchor_hash) if self.prev_anchor_hash is not None else None
                ),
            },
            separators=(",", ":"),
            allow_nan=False,
        ).encode("utf-8")


@dataclass
class Anchor:
    version: int
    ledger_root: Hash
    entry_count: int
    anchored_at: str
    prev_anchor_hash: Hash | None
    signer: Hash
    signature: bytes

    @classmethod
    def new(
        cls,
        keypair: KeyPair,
        ledger_root: Hash,
        entry_count: int,
        anchored_at: str,
        prev_anchor_hash: Hash | None = None,
    ) -> "Anchor":
        payload = AnchorPayload(
            version=WIRE_VERSION,
            ledger_root=ledger_root,
            entry_count=entry_count,
            anchored_at=anchored_at,
            prev_anchor_hash=prev_anchor_hash,
        )
        claim = SignedClaim.new(keypair, payload.canonical_bytes())
        return cls(
            version=WIRE_VERSION,
            ledger_root=ledger_root,
            entry_count=entry_count,
            anchored_at=anchored_at,
            prev_anchor_hash=prev_anchor_hash,
            signer=claim.signer,
            signature=claim.signature,
        )

    def payload(self) -> AnchorPayload:
        return AnchorPayload(
            version=self.version,
            ledger_root=self.ledger_root,
            entry_count=self.entry_count,
            anchored_at=self.anchored_at,
            prev_anchor_hash=self.prev_anchor_hash,
        )

    def payload_hash(self) -> Hash:
        return hashlib.sha256(self.payload().canonical_bytes()).digest()

    def verify(self) -> bool:
        if self.version != WIRE_VERSION:
            return False
        if len(self.signer) != 32 or len(self.signature) != 64:
            return False
        try:
            Ed25519PublicKey.from_public_bytes(self.signer).verify(
                self.signature, self.payload().canonical_bytes()
            )
            return True
        except (InvalidSignature, ValueError):
            return False
