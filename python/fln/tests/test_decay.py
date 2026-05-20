import math

from fln import CausalDecayParams, causal_decay_weight


def test_regime_shift_resets_weight():
    params = CausalDecayParams()
    w = causal_decay_weight(0.9, 10.0, 0.0, 35.0, params)
    assert abs(w) < 1e-9


def test_decay_alone_reduces_weight():
    params = CausalDecayParams()
    w = causal_decay_weight(1.0, 180.0, 0.0, 10.0, params)
    assert abs(w - math.exp(-1.0)) < 1e-9


def test_falsifier_outcome_pushes_weight():
    params = CausalDecayParams()
    assert causal_decay_weight(0.0, 30.0, 1.0, 10.0, params) > 0.0
    assert causal_decay_weight(0.0, 30.0, -1.0, 10.0, params) < 0.0
