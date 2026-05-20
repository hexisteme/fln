"""Append-only Merkle ledger — wire-compatible with ``fln-core::ledger``."""

from __future__ import annotations

from dataclasses import dataclass, field

from .merkle import Hash, MerkleNode, merkle_root


@dataclass
class Ledger:
    entries: list[MerkleNode] = field(default_factory=list)
    _cached_root: Hash | None = field(default=None, repr=False)

    def append(self, node: MerkleNode) -> Hash:
        h = node.hash()
        self.entries.append(node)
        self._cached_root = None
        return h

    def __len__(self) -> int:
        return len(self.entries)

    def root(self) -> Hash | None:
        if self._cached_root is None:
            self._cached_root = merkle_root([n.hash() for n in self.entries])
        return self._cached_root

    def verify_integrity(self) -> bool:
        recomputed = merkle_root([n.hash() for n in self.entries])
        return recomputed == self._cached_root
