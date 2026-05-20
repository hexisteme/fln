"""FLN MCP server (stdio).

Exposes the FLN primitives — thesis lifecycle, ed25519 signing, merkle ledger,
causal DAG, causal-decay weight — as MCP tools. Persists state to JSON under
``FLN_STATE_DIR`` (default ``~/.fln``) so the server is restartable.
"""

from __future__ import annotations

import json
import os
from dataclasses import asdict
from pathlib import Path
from typing import Any

from fln import (
    CausalDecayParams,
    CausalEdge,
    CausalNode,
    Domain,
    EdgeKind,
    Falsifier,
    KeyPair,
    Ledger,
    MerkleNode,
    NodeKind,
    SignedClaim,
    Thesis,
    causal_decay_weight,
)
from mcp.server.fastmcp import FastMCP

STATE_DIR = Path(os.environ.get("FLN_STATE_DIR", str(Path.home() / ".fln")))
STATE_DIR.mkdir(parents=True, exist_ok=True)


def _thesis_path(id: str) -> Path:
    return STATE_DIR / "theses" / f"{id}.json"


def _ledger_path(name: str) -> Path:
    return STATE_DIR / "ledgers" / f"{name}.json"


def _key_path(name: str) -> Path:
    return STATE_DIR / "keys" / f"{name}"


def _dump_thesis(t: Thesis) -> None:
    p = _thesis_path(t.id)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(t.canonical_bytes())


def _load_thesis(id: str) -> Thesis:
    data = json.loads(_thesis_path(id).read_text())
    return Thesis(
        id=data["id"],
        domain=Domain(data["domain"]),
        claim=data["claim"],
        falsifiers=[Falsifier(**f) for f in data.get("falsifiers", [])],
        causal_dag=_load_causal(data.get("causal_dag", {})),
        decay=CausalDecayParams(**data.get("decay", {})),
        weight=data.get("weight", 0.0),
        created_at=data.get("created_at"),
    )


def _load_causal(d: dict) -> Any:
    from fln import CausalDAG

    g = CausalDAG()
    for n in d.get("nodes", []):
        g.nodes.append(CausalNode(id=n["id"], label=n["label"], kind=NodeKind(n["kind"])))
    for e in d.get("edges", []):
        g.edges.append(CausalEdge(from_=e["from"], to=e["to"], kind=EdgeKind(e["kind"])))
    return g


mcp = FastMCP("fln")


@mcp.tool()
def create_thesis(id: str, domain: str, claim: str) -> dict:
    """Create a new thesis and persist it. ``domain`` ∈ invest|health|real_estate|policy|science|engineering."""
    t = Thesis.new(id, Domain(domain), claim)
    _dump_thesis(t)
    return {"id": t.id, "domain": t.domain.value, "tau_days": t.decay.tau_days}


@mcp.tool()
def add_falsifier(id: str, condition: str, deadline: str | None = None) -> dict:
    t = _load_thesis(id)
    t.falsifiers.append(Falsifier(condition=condition, deadline=deadline))
    _dump_thesis(t)
    return {"falsifier_count": len(t.falsifiers)}


@mcp.tool()
def add_causal_node(id: str, node_id: str, label: str, kind: str) -> dict:
    """``kind`` ∈ cause|effect|confounder|mediator."""
    t = _load_thesis(id)
    t.causal_dag.add_node(CausalNode(id=node_id, label=label, kind=NodeKind(kind)))
    _dump_thesis(t)
    return {"nodes": len(t.causal_dag.nodes), "edges": len(t.causal_dag.edges)}


@mcp.tool()
def add_causal_edge(id: str, from_node: str, to_node: str, kind: str = "direct") -> dict:
    t = _load_thesis(id)
    t.causal_dag.add_edge(CausalEdge(from_=from_node, to=to_node, kind=EdgeKind(kind)))
    _dump_thesis(t)
    return {"nodes": len(t.causal_dag.nodes), "edges": len(t.causal_dag.edges)}


@mcp.tool()
def causal_topo(id: str) -> dict:
    t = _load_thesis(id)
    order = t.causal_dag.topological_order()
    return {"order": order or [], "ok": order is not None}


@mcp.tool()
def generate_key(name: str) -> dict:
    """Generate an Ed25519 keypair and store both halves under ``$FLN_STATE_DIR/keys/<name>``."""
    kp = KeyPair.generate()
    path = _key_path(name)
    path.parent.mkdir(parents=True, exist_ok=True)
    Path(f"{path}.sk").write_text(kp.secret_bytes().hex())
    Path(f"{path}.pk").write_text(kp.public_bytes().hex())
    return {"public_key_hex": kp.public_bytes().hex()}


@mcp.tool()
def sign_thesis(id: str, key_name: str) -> dict:
    """Sign the canonical thesis bytes; return public key + signature in hex."""
    t = _load_thesis(id)
    sk = bytes.fromhex(Path(f"{_key_path(key_name)}.sk").read_text().strip())
    claim = SignedClaim.new(KeyPair.from_bytes(sk), t.canonical_bytes())
    return {
        "signer_hex": claim.signer.hex(),
        "signature_hex": claim.signature.hex(),
        "verified": claim.verify(),
    }


@mcp.tool()
def append_ledger(ledger_name: str, id: str) -> dict:
    """Append a thesis to ``<ledger_name>``; persist + return root + entry hash."""
    t = _load_thesis(id)
    lp = _ledger_path(ledger_name)
    if lp.exists():
        raw = json.loads(lp.read_text())
        ledger = Ledger(
            entries=[
                MerkleNode(
                    payload=bytes(e["payload"]),
                    parents=[bytes(p) for p in e.get("parents", [])],
                )
                for e in raw.get("entries", [])
            ]
        )
    else:
        ledger = Ledger()
        lp.parent.mkdir(parents=True, exist_ok=True)
    node = t.to_merkle_node()
    h = ledger.append(node)
    root = ledger.root()
    serialized = {
        "entries": [
            {"payload": list(n.payload), "parents": [list(p) for p in n.parents]}
            for n in ledger.entries
        ]
    }
    lp.write_text(json.dumps(serialized))
    return {
        "entry_hash_hex": h.hex(),
        "root_hex": root.hex() if root else None,
        "count": len(ledger),
    }


@mcp.tool()
def decay_update(
    id: str,
    delta_days: float,
    outcome: float,
    regime_signal: float = 0.0,
) -> dict:
    """Update the thesis's posterior weight via Causal Decay; return new weight."""
    t = _load_thesis(id)
    new_w = causal_decay_weight(t.weight, delta_days, outcome, regime_signal, t.decay)
    t.weight = new_w
    _dump_thesis(t)
    return {"weight": new_w, "tau_days": t.decay.tau_days}


@mcp.tool()
def get_thesis(id: str) -> dict:
    """Return the full thesis JSON."""
    t = _load_thesis(id)
    return {
        "id": t.id,
        "domain": t.domain.value,
        "claim": t.claim,
        "falsifiers": [asdict(f) for f in t.falsifiers],
        "causal_dag": {
            "nodes": [{"id": n.id, "label": n.label, "kind": n.kind.value} for n in t.causal_dag.nodes],
            "edges": [{"from": e.from_, "to": e.to, "kind": e.kind.value} for e in t.causal_dag.edges],
        },
        "decay": asdict(t.decay),
        "weight": t.weight,
        "created_at": t.created_at,
    }


def main() -> None:
    mcp.run()


if __name__ == "__main__":
    main()
