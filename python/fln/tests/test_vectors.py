"""Cross-language test vectors — Python side of the wire-compat assertion.

Reads ``tests/vectors/v1/manifest.json`` at the workspace root and checks each
case against the Python ``MerkleNode`` hash. Paired with
``crates/fln-core/tests/vectors.rs``.
"""

from __future__ import annotations

import json
from pathlib import Path

from fln import MerkleNode

VECTORS = Path(__file__).resolve().parents[3] / "tests" / "vectors" / "v1"


def test_manifest_v1_round_trips() -> None:
    manifest = json.loads((VECTORS / "manifest.json").read_text())
    assert manifest["version"] == 1
    assert manifest["cases"], "manifest has no cases"

    for case in manifest["cases"]:
        payload = (VECTORS / case["thesis"]).read_bytes()
        assert payload.hex() == case["canonical_bytes_hex"], (
            f"canonical bytes drift for case `{case['name']}`"
        )
        node = MerkleNode(payload=payload, parents=[])
        assert node.hash().hex() == case["merkle_hash_hex"], (
            f"merkle hash drift for case `{case['name']}`"
        )
