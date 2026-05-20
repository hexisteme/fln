"""Generated theses MUST validate against the canonical JSON Schemas."""

from __future__ import annotations

import json
from pathlib import Path

import jsonschema
import pytest

from fln import (
    CausalEdge,
    CausalNode,
    Domain,
    EdgeKind,
    Falsifier,
    NodeKind,
    Thesis,
)

SCHEMA_DIR = Path(__file__).resolve().parents[3] / "schema"


def _load(name: str) -> dict:
    return json.loads((SCHEMA_DIR / name).read_text())


@pytest.fixture(scope="module")
def thesis_validator():
    thesis = _load("thesis.schema.json")
    falsifier = _load("falsifier.schema.json")
    causal = _load("causal_dag.schema.json")
    # Bundle so relative $refs from `thesis.schema.json` resolve.
    bundled = dict(thesis)
    bundled["$defs"] = {
        "falsifier": falsifier,
        "causal_dag": causal,
    }
    # Rewrite refs to use the bundled $defs.
    bundled["properties"]["falsifiers"]["items"] = {"$ref": "#/$defs/falsifier"}
    bundled["properties"]["causal_dag"] = {"$ref": "#/$defs/causal_dag"}
    return jsonschema.Draft202012Validator(bundled)


def test_real_thesis_validates(thesis_validator):
    t = Thesis.new("btc-q2-test", Domain.INVEST, "BTC ≥ 150k")
    t.falsifiers.append(Falsifier(condition="BTC < 80k", deadline="2026-09-01"))
    t.causal_dag.add_node(CausalNode(id="VIX", label="VIX", kind=NodeKind.CONFOUNDER))
    t.causal_dag.add_node(CausalNode(id="BTC", label="BTC", kind=NodeKind.EFFECT))
    t.causal_dag.add_edge(CausalEdge(from_="VIX", to="BTC", kind=EdgeKind.DIRECT))
    payload = json.loads(t.canonical_bytes())
    errors = list(thesis_validator.iter_errors(payload))
    assert errors == [], f"validation errors: {errors}"


def test_invalid_thesis_id_rejected(thesis_validator):
    t = Thesis.new("Has Capitals", Domain.INVEST, "x")
    payload = json.loads(t.canonical_bytes())
    errors = list(thesis_validator.iter_errors(payload))
    assert any("id" in str(e.path) or "id" in e.message for e in errors)
