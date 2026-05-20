import pytest

from fln import CausalDAG, CausalEdge, CausalError, CausalNode, EdgeKind, NodeKind


def node(id: str, kind: NodeKind = NodeKind.CAUSE) -> CausalNode:
    return CausalNode(id=id, label=id, kind=kind)


def edge(a: str, b: str) -> CausalEdge:
    return CausalEdge(from_=a, to=b, kind=EdgeKind.DIRECT)


def test_add_node_rejects_duplicate():
    g = CausalDAG()
    g.add_node(node("X"))
    with pytest.raises(CausalError):
        g.add_node(node("X"))


def test_add_edge_rejects_unknown_endpoint():
    g = CausalDAG()
    g.add_node(node("X"))
    with pytest.raises(CausalError):
        g.add_edge(edge("X", "Y"))


def test_add_edge_rejects_cycle():
    g = CausalDAG()
    g.add_node(node("X"))
    g.add_node(node("Y", NodeKind.EFFECT))
    g.add_edge(edge("X", "Y"))
    with pytest.raises(CausalError):
        g.add_edge(edge("Y", "X"))


def test_topological_order_respects_dependencies():
    g = CausalDAG()
    g.add_node(node("A"))
    g.add_node(node("B", NodeKind.MEDIATOR))
    g.add_node(node("C", NodeKind.EFFECT))
    g.add_edge(edge("A", "B"))
    g.add_edge(edge("B", "C"))
    order = g.topological_order()
    assert order is not None
    assert order.index("A") < order.index("B") < order.index("C")
