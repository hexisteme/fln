"""Mirror of crates/fln-core/examples/wire_compat.rs — must produce identical hex."""

from __future__ import annotations

from fln import (
    CausalEdge,
    CausalNode,
    Domain,
    EdgeKind,
    Falsifier,
    NodeKind,
    Thesis,
)


def main() -> None:
    t = Thesis.new("fixed-test", Domain.INVEST, "deterministic claim")
    t.created_at = "2026-05-20T00:00:00Z"
    t.falsifiers.append(Falsifier(condition="x<y", deadline="2026-06-01", triggered=False))
    t.causal_dag.add_node(CausalNode(id="A", label="node-A", kind=NodeKind.CAUSE))
    t.causal_dag.add_node(CausalNode(id="B", label="node-B", kind=NodeKind.EFFECT))
    t.causal_dag.add_edge(CausalEdge(from_="A", to="B", kind=EdgeKind.DIRECT))

    canon = t.canonical_bytes()
    node = t.to_merkle_node()
    print(f"canonical_bytes_hex {canon.hex()}")
    print(f"merkle_hash_hex     {node.hash().hex()}")


if __name__ == "__main__":
    main()
