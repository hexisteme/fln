"""CausalDAG — Pearl do-calculus DAG primitives.

Wire-compatible with ``fln-core::causal``: same node/edge kinds, same
cycle-rejecting ``add_edge``, same Kahn topological order tie-break.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class NodeKind(str, Enum):
    CAUSE = "cause"
    EFFECT = "effect"
    CONFOUNDER = "confounder"
    MEDIATOR = "mediator"


class EdgeKind(str, Enum):
    DIRECT = "direct"
    CONFOUNDED = "confounded"
    BACKDOOR = "backdoor"


class CausalError(Exception):
    pass


@dataclass
class CausalNode:
    id: str
    label: str
    kind: NodeKind


@dataclass
class CausalEdge:
    from_: str
    to: str
    kind: EdgeKind = EdgeKind.DIRECT


@dataclass
class CausalDAG:
    nodes: list[CausalNode] = field(default_factory=list)
    edges: list[CausalEdge] = field(default_factory=list)

    def has_node(self, id: str) -> bool:
        return any(n.id == id for n in self.nodes)

    def add_node(self, node: CausalNode) -> None:
        if self.has_node(node.id):
            raise CausalError(f"node id `{node.id}` already exists")
        self.nodes.append(node)

    def add_edge(self, edge: CausalEdge) -> None:
        if not self.has_node(edge.from_):
            raise CausalError(f"edge endpoint `{edge.from_}` is unknown")
        if not self.has_node(edge.to):
            raise CausalError(f"edge endpoint `{edge.to}` is unknown")
        if self.path_exists(edge.to, edge.from_):
            raise CausalError(f"edge would introduce a cycle: {edge.from_} -> {edge.to}")
        self.edges.append(edge)

    def path_exists(self, src: str, dst: str) -> bool:
        adj: dict[str, list[str]] = {}
        for e in self.edges:
            adj.setdefault(e.from_, []).append(e.to)
        stack = [src]
        seen: set[str] = set()
        while stack:
            cur = stack.pop()
            if cur == dst:
                return True
            if cur in seen:
                continue
            seen.add(cur)
            stack.extend(adj.get(cur, []))
        return False

    def topological_order(self) -> list[str] | None:
        indegree: dict[str, int] = {n.id: 0 for n in self.nodes}
        adj: dict[str, list[str]] = {n.id: [] for n in self.nodes}
        for e in self.edges:
            indegree[e.to] += 1
            adj[e.from_].append(e.to)
        ready = sorted([k for k, v in indegree.items() if v == 0])
        order: list[str] = []
        while ready:
            cur = ready.pop()
            order.append(cur)
            released: list[str] = []
            for nxt in adj.get(cur, []):
                indegree[nxt] -= 1
                if indegree[nxt] == 0:
                    released.append(nxt)
            released.sort()
            ready.extend(released)
        return order if len(order) == len(self.nodes) else None
