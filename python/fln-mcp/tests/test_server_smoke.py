"""In-process smoke test of fln-mcp by invoking the tool functions directly."""

from __future__ import annotations

import os
import tempfile
from pathlib import Path


def test_full_pipeline(monkeypatch):
    tmp = tempfile.mkdtemp(prefix="fln-mcp-test-")
    monkeypatch.setenv("FLN_STATE_DIR", tmp)
    # Re-import so the module picks up the new STATE_DIR.
    import importlib
    import fln_mcp.server as server

    importlib.reload(server)

    out = server.create_thesis(id="btc-q2", domain="invest", claim="BTC ≥ 150k within 90d")
    assert out["tau_days"] == 180.0

    server.add_falsifier(id="btc-q2", condition="BTC < 80k", deadline="2026-09-01")
    server.add_causal_node(id="btc-q2", node_id="VIX", label="Volatility", kind="confounder")
    server.add_causal_node(id="btc-q2", node_id="BTC", label="BTC price", kind="effect")
    server.add_causal_edge(id="btc-q2", from_node="VIX", to_node="BTC", kind="direct")

    topo = server.causal_topo(id="btc-q2")
    assert topo["ok"]
    assert topo["order"].index("VIX") < topo["order"].index("BTC")

    server.generate_key(name="alice")
    sig = server.sign_thesis(id="btc-q2", key_name="alice")
    assert sig["verified"]
    assert len(sig["signer_hex"]) == 64
    assert len(sig["signature_hex"]) == 128

    appended = server.append_ledger(ledger_name="main", id="btc-q2")
    assert appended["count"] == 1
    assert appended["root_hex"] == appended["entry_hash_hex"]

    weights = []
    weights.append(server.decay_update(id="btc-q2", delta_days=30, outcome=0.5, regime_signal=15)["weight"])
    assert weights[-1] > 0
    weights.append(server.decay_update(id="btc-q2", delta_days=1, outcome=0, regime_signal=35)["weight"])
    assert abs(weights[-1]) < 1e-9
