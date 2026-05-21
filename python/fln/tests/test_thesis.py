from fln import (
    CausalEdge,
    CausalNode,
    Domain,
    EdgeKind,
    KeyPair,
    Ledger,
    NodeKind,
    Thesis,
)


def sample_thesis() -> Thesis:
    t = Thesis.new("btc-q2", Domain.INVEST, "BTC ≥ 150k within 90d")
    t.causal_dag.add_node(CausalNode(id="VIX", label="VIX", kind=NodeKind.CONFOUNDER))
    t.causal_dag.add_node(CausalNode(id="BTC", label="BTC price", kind=NodeKind.EFFECT))
    t.causal_dag.add_edge(CausalEdge(from_="VIX", to="BTC", kind=EdgeKind.DIRECT))
    return t


def test_canonical_bytes_deterministic():
    t = sample_thesis()
    assert t.canonical_bytes() == t.canonical_bytes()


def test_sign_verify_roundtrip():
    t = sample_thesis()
    kp = KeyPair.generate()
    claim = t.sign(kp)
    assert claim.verify()


def test_domain_tau_matches_spec():
    assert Domain.INVEST.default_tau_days == 180.0
    assert Domain.HEALTH.default_tau_days == 730.0
    assert Domain.REAL_ESTATE.default_tau_days == 365.0


def test_ledger_append_and_root():
    ledger = Ledger()
    t = sample_thesis()
    h = ledger.append(t.to_merkle_node())
    assert h is not None
    root = ledger.root()
    assert root is not None
    # v0.2: root binds entry_count + tags, so root ≠ leaf hash even for n=1.
    assert root != h
    assert ledger.verify_integrity()
