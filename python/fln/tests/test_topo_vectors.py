"""Cross-language topological-order determinism (Python side)."""

from __future__ import annotations

import json
from pathlib import Path

from fln import CausalDAG, CausalEdge, CausalNode, EdgeKind, NodeKind

VECTORS = Path(__file__).resolve().parents[3] / "tests" / "vectors" / "v1" / "topo_order.json"


def test_topological_order_matches_fixture():
    data = json.loads(VECTORS.read_text())
    for case in data["cases"]:
        g = CausalDAG()
        for n in case["nodes"]:
            g.add_node(CausalNode(id=n, label=n, kind=NodeKind.CAUSE))
        for a, b in case["edges"]:
            g.add_edge(CausalEdge(from_=a, to=b, kind=EdgeKind.DIRECT))
        actual = g.topological_order()
        assert actual == case["expected_order"], (
            f"topo drift on case `{case['name']}`: expected {case['expected_order']}, got {actual}"
        )
