"""Merkle DAG primitives — wire-compatible with the Rust ``fln-core::merkle``.

Node hash:

    sha256(
        be64(len(payload)) || payload ||
        be64(len(parents)) || parent_1 || parent_2 || ...
    )

## v0.2 hardening (CVE-2012-2459 fix)

The ``merkle_root`` function:

1. Domain-separates leaves (tag ``0x00``), internal nodes (tag ``0x01``),
   and the final root (tag ``0xFF``).
2. Promotes the lone tail item to the next layer instead of duplicating it
   (RFC 6962 §2.1 style).
3. Binds ``leaf_count`` into the final root hash, so ``[A, B, C]`` and
   ``[A, B, C, C]`` produce distinct roots.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field

Hash = bytes  # 32 bytes

LEAF_TAG = 0x00
NODE_TAG = 0x01
ROOT_TAG = 0xFF


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


def _hash_leaf(h: Hash) -> Hash:
    s = hashlib.sha256()
    s.update(bytes([LEAF_TAG]))
    s.update(h)
    return s.digest()


def _hash_node(left: Hash, right: Hash) -> Hash:
    s = hashlib.sha256()
    s.update(bytes([NODE_TAG]))
    s.update(left)
    s.update(right)
    return s.digest()


def merkle_root(leaves: list[Hash]) -> Hash | None:
    if not leaves:
        return None
    count = len(leaves)
    layer: list[Hash] = [_hash_leaf(leaf) for leaf in leaves]
    while len(layer) > 1:
        nxt: list[Hash] = []
        i = 0
        while i + 1 < len(layer):
            nxt.append(_hash_node(layer[i], layer[i + 1]))
            i += 2
        if i < len(layer):
            nxt.append(layer[i])  # promote tail (no duplication)
        layer = nxt
    tree_root = layer[0]
    final = hashlib.sha256()
    final.update(bytes([ROOT_TAG]))
    final.update(_be64(count))
    final.update(tree_root)
    return final.digest()
