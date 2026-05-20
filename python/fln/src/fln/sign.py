"""Ed25519 signing — wire-compatible with the Rust ``fln-core::sign``."""

from __future__ import annotations

import os
from dataclasses import dataclass, field

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)
from cryptography.hazmat.primitives.serialization import (
    Encoding,
    NoEncryption,
    PrivateFormat,
    PublicFormat,
)


class KeyPair:
    def __init__(self, signing: Ed25519PrivateKey):
        self._signing = signing

    @classmethod
    def generate(cls) -> "KeyPair":
        return cls(Ed25519PrivateKey.generate())

    @classmethod
    def from_bytes(cls, secret: bytes) -> "KeyPair":
        if len(secret) != 32:
            raise ValueError("secret must be 32 bytes")
        return cls(Ed25519PrivateKey.from_private_bytes(secret))

    def secret_bytes(self) -> bytes:
        return self._signing.private_bytes(
            encoding=Encoding.Raw,
            format=PrivateFormat.Raw,
            encryption_algorithm=NoEncryption(),
        )

    def public_bytes(self) -> bytes:
        return self._signing.public_key().public_bytes(
            encoding=Encoding.Raw, format=PublicFormat.Raw
        )

    def sign(self, message: bytes) -> bytes:
        return self._signing.sign(message)


@dataclass
class SignedClaim:
    payload: bytes
    signer: bytes
    signature: bytes

    @classmethod
    def new(cls, keypair: KeyPair, payload: bytes) -> "SignedClaim":
        return cls(payload=payload, signer=keypair.public_bytes(), signature=keypair.sign(payload))

    def verify(self) -> bool:
        if len(self.signer) != 32 or len(self.signature) != 64:
            return False
        try:
            Ed25519PublicKey.from_public_bytes(self.signer).verify(self.signature, self.payload)
            return True
        except (InvalidSignature, ValueError):
            return False
