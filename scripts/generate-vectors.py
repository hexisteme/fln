"""Generate canonical FLN test vectors.

Writes ``tests/vectors/v1/`` containing one ``.thesis.json`` per case plus a
``manifest.json`` listing the expected ``canonical_bytes_hex`` and
``merkle_hash_hex``. Both the Rust crate and the Python package include
tests that consume this manifest, so any wire drift causes CI to fail.
"""

from __future__ import annotations

import json
from pathlib import Path

from fln import (
    CausalDAG,
    CausalEdge,
    CausalNode,
    Domain,
    EdgeKind,
    Falsifier,
    NodeKind,
    Thesis,
)

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "tests" / "vectors" / "v1"
OUT.mkdir(parents=True, exist_ok=True)


def case_empty() -> Thesis:
    return Thesis.new("empty-thesis", Domain.INVEST, "no causal nodes, no falsifiers")


def case_single_falsifier() -> Thesis:
    t = Thesis.new("single-falsifier", Domain.HEALTH, "VO2max stays above 50")
    t.falsifiers.append(Falsifier(condition="VO2max < 50", deadline=None))
    return t


def case_falsifier_with_deadline() -> Thesis:
    t = Thesis.new("falsifier-deadline", Domain.INVEST, "BTC closes ≥ 150k by 2026-09-01")
    t.falsifiers.append(Falsifier(condition="BTC<80k", deadline="2026-09-01", triggered=False))
    return t


def case_multi_falsifier() -> Thesis:
    t = Thesis.new("multi-falsifier", Domain.POLICY, "Bill X passes 2026 session")
    t.falsifiers.extend([
        Falsifier(condition="Senate vote NO", deadline="2026-12-31"),
        Falsifier(condition="committee tables", deadline=None),
        Falsifier(condition="amendment rewrites Section 3", deadline=None, triggered=True),
    ])
    return t


def case_rich_causal() -> Thesis:
    t = Thesis.new("rich-causal", Domain.INVEST, "ETH outperforms BTC if ETF approves")
    t.causal_dag.add_node(CausalNode(id="ETF_APPROVAL", label="Spot ETH ETF", kind=NodeKind.CAUSE))
    t.causal_dag.add_node(CausalNode(id="VIX", label="VIX", kind=NodeKind.CONFOUNDER))
    t.causal_dag.add_node(CausalNode(id="STAKING_YIELD", label="Yield", kind=NodeKind.MEDIATOR))
    t.causal_dag.add_node(CausalNode(id="ETH", label="ETH price", kind=NodeKind.EFFECT))
    t.causal_dag.add_node(CausalNode(id="BTC", label="BTC price", kind=NodeKind.EFFECT))
    t.causal_dag.add_edge(CausalEdge(from_="ETF_APPROVAL", to="STAKING_YIELD", kind=EdgeKind.DIRECT))
    t.causal_dag.add_edge(CausalEdge(from_="STAKING_YIELD", to="ETH", kind=EdgeKind.DIRECT))
    t.causal_dag.add_edge(CausalEdge(from_="VIX", to="ETH", kind=EdgeKind.CONFOUNDED))
    t.causal_dag.add_edge(CausalEdge(from_="VIX", to="BTC", kind=EdgeKind.CONFOUNDED))
    return t


def case_utf8() -> Thesis:
    return Thesis.new("utf8-claim", Domain.SCIENCE, "정총무 가설: 한국어 + 日本語 + 中文 → BTC ≥ 150k")


def case_all_kinds() -> Thesis:
    t = Thesis.new("all-causal-kinds", Domain.SCIENCE, "Coverage of every NodeKind and EdgeKind")
    for k in NodeKind:
        t.causal_dag.add_node(CausalNode(id=k.value.upper(), label=k.value, kind=k))
    t.causal_dag.add_edge(CausalEdge(from_="CAUSE",      to="EFFECT",   kind=EdgeKind.DIRECT))
    t.causal_dag.add_edge(CausalEdge(from_="CONFOUNDER", to="EFFECT",   kind=EdgeKind.CONFOUNDED))
    t.causal_dag.add_edge(CausalEdge(from_="MEDIATOR",   to="EFFECT",   kind=EdgeKind.BACKDOOR))
    return t


CASES = [
    ("empty",                  case_empty),
    ("single-falsifier",       case_single_falsifier),
    ("falsifier-deadline",     case_falsifier_with_deadline),
    ("multi-falsifier",        case_multi_falsifier),
    ("rich-causal",            case_rich_causal),
    ("utf8-claim",             case_utf8),
    ("all-causal-kinds",       case_all_kinds),
]


def main() -> None:
    manifest = {"version": 1, "cases": []}
    for name, ctor in CASES:
        t = ctor()
        thesis_path = OUT / f"{name}.thesis.json"
        thesis_path.write_text(json.dumps(t.to_canonical_dict(), separators=(",", ":")))
        canon = t.canonical_bytes()
        node = t.to_merkle_node()
        manifest["cases"].append({
            "name": name,
            "thesis": f"{name}.thesis.json",
            "canonical_bytes_hex": canon.hex(),
            "merkle_hash_hex": node.hash().hex(),
        })
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2))
    print(f"wrote {len(CASES)} vectors to {OUT}")


if __name__ == "__main__":
    main()
