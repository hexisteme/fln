"""Merkle DAG primitives — wire-compatible with the Rust ``fln-core::merkle``.

Hash layout matches the Rust implementation exactly:

    sha256(
        be64(len(payload)) || payload ||
        be64(len(parents)) || parent_1 || parent_2 || ...
    )
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field

Hash = bytes  # 32 bytes


def _be64(n: int) -> bytes:
    return n.to_bytes(8, "big", signed=False)


@dataclass
class MerkleNode:
    payload: bytes
    parents: list[Hash] = field(default_factory=list)

    def hash(self) -> Hash:
        h = hashlib.sha256()
        h.update(_be64(len(self.payload)))
        h.update(self.payload)
        h.update(_be64(len(self.parents)))
        for p in self.parents:
            h.update(p)
        return h.digest()


def merkle_root(leaves: list[Hash]) -> Hash | None:
    if not leaves:
        return None
    layer = list(leaves)
    while len(layer) > 1:
        nxt: list[Hash] = []
        for i in range(0, len(layer), 2):
            left = layer[i]
            right = layer[i + 1] if i + 1 < len(layer) else left
            nxt.append(hashlib.sha256(left + right).digest())
        layer = nxt
    return layer[0]
