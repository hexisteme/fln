"""Unit tests for the predicate evaluator using deterministic InMemorySource."""

from __future__ import annotations

import pandas as pd
import pytest

from fln_oracle import (
    EvaluationResult,
    InMemorySource,
    Predicate,
    PredicateSet,
    Window,
    evaluate_predicate,
    evaluate_predicates,
)


def _series(prices: list[float], end: str = "2026-05-20") -> pd.DataFrame:
    end_ts = pd.Timestamp(end, tz="UTC")
    idx = pd.date_range(end=end_ts, periods=len(prices), freq="D", tz="UTC")
    return pd.DataFrame(
        {
            "Open": prices,
            "High": [p * 1.01 for p in prices],
            "Low": [p * 0.99 for p in prices],
            "Close": prices,
        },
        index=idx,
    )


@pytest.fixture()
def source() -> InMemorySource:
    return InMemorySource(
        {
            "BTC-USD": _series([100000, 95000, 90000, 85000, 79000, 82000, 88000]),
            "SPY": _series([500, 480, 470, 460, 450, 470, 500]),
            "STABLE": _series([100, 100, 100, 100, 100, 100, 100]),
        }
    )


def test_any_close_triggers_on_crossing(source: InMemorySource) -> None:
    p = Predicate(
        falsifier_idx=0,
        ticker="BTC-USD",
        field="close",
        op="lt",
        rhs=80000.0,
        window=Window(kind="any_close", lookback_days=10),
    )
    r = evaluate_predicate(p, source)
    assert r.triggered is True
    assert r.observed_value == pytest.approx(79000.0)


def test_any_close_does_not_trigger_when_above(source: InMemorySource) -> None:
    p = Predicate(
        falsifier_idx=0,
        ticker="BTC-USD",
        field="close",
        op="lt",
        rhs=50000.0,
        window=Window(kind="any_close", lookback_days=10),
    )
    r = evaluate_predicate(p, source)
    assert r.triggered is False
    assert r.observed_value is not None


def test_min_close_triggers_when_min_below(source: InMemorySource) -> None:
    p = Predicate(
        falsifier_idx=1,
        ticker="SPY",
        field="close",
        op="lte",
        rhs=460.0,
        window=Window(kind="min_close", lookback_days=10),
    )
    r = evaluate_predicate(p, source)
    assert r.triggered is True
    assert r.observed_value == pytest.approx(450.0)


def test_drawdown_from_high_triggers_above_threshold(source: InMemorySource) -> None:
    p = Predicate(
        falsifier_idx=2,
        ticker="SPY",
        field="close",
        op="gt",
        rhs=0.08,
        window=Window(kind="drawdown_from_high", lookback_days=10),
    )
    r = evaluate_predicate(p, source)
    assert r.triggered is True
    assert r.observed_value is not None and r.observed_value > 0.08


def test_drawdown_from_high_skips_stable(source: InMemorySource) -> None:
    p = Predicate(
        falsifier_idx=3,
        ticker="STABLE",
        field="close",
        op="gt",
        rhs=0.01,
        window=Window(kind="drawdown_from_high", lookback_days=10),
    )
    r = evaluate_predicate(p, source)
    assert r.triggered is False


def test_drawdown_requires_gt_or_gte(source: InMemorySource) -> None:
    with pytest.raises(ValueError, match="drawdown"):
        evaluate_predicate(
            Predicate(
                falsifier_idx=4,
                ticker="SPY",
                field="close",
                op="lt",
                rhs=0.1,
                window=Window(kind="drawdown_from_high", lookback_days=10),
            ),
            source,
        )


def test_evaluate_predicates_aggregates(source: InMemorySource) -> None:
    ps = PredicateSet(
        thesis_id="multi",
        predicates=[
            Predicate(0, "BTC-USD", "close", "lt", 80000, Window("any_close", 10)),
            Predicate(1, "SPY", "close", "lt", 1.0, Window("any_close", 10)),
        ],
    )
    report = evaluate_predicates(ps, source)
    assert report["any_triggered"] is True
    assert len(report["results"]) == 2
    assert report["results"][0]["triggered"] is True
    assert report["results"][1]["triggered"] is False
