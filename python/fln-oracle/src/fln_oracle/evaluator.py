"""Predicate evaluator — turn a structured predicate into a triggered/not decision."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pandas as pd

from .predicate import Op, Predicate, PredicateSet
from .sources import MarketSource

_FIELD_TO_COL = {"close": "Close", "high": "High", "low": "Low", "open": "Open"}


def _cmp(lhs: float, op: Op, rhs: float) -> bool:
    if op == "lt":
        return lhs < rhs
    if op == "lte":
        return lhs <= rhs
    if op == "gt":
        return lhs > rhs
    if op == "gte":
        return lhs >= rhs
    raise ValueError(f"unknown op `{op}`")


@dataclass
class EvaluationResult:
    falsifier_idx: int
    ticker: str
    triggered: bool
    observed_value: float | None
    observed_at: str | None  # ISO 8601 UTC
    window_days: int
    reason: str

    def to_json_dict(self) -> dict:
        return asdict(self)


def evaluate_predicate(predicate: Predicate, source: MarketSource) -> EvaluationResult:
    df: pd.DataFrame = source.history(predicate.ticker, predicate.window.lookback_days)
    if df.empty:
        return EvaluationResult(
            falsifier_idx=predicate.falsifier_idx,
            ticker=predicate.ticker,
            triggered=False,
            observed_value=None,
            observed_at=None,
            window_days=predicate.window.lookback_days,
            reason="no data in window",
        )

    col = _FIELD_TO_COL[predicate.field]
    series = df[col].dropna()
    if series.empty:
        return EvaluationResult(
            falsifier_idx=predicate.falsifier_idx,
            ticker=predicate.ticker,
            triggered=False,
            observed_value=None,
            observed_at=None,
            window_days=predicate.window.lookback_days,
            reason=f"no {predicate.field} samples",
        )

    kind = predicate.window.kind

    if kind == "any_close":
        mask = series.apply(lambda v: _cmp(float(v), predicate.op, predicate.rhs))
        hits = series[mask]
        if len(hits) > 0:
            ts = hits.index[0]
            return EvaluationResult(
                falsifier_idx=predicate.falsifier_idx,
                ticker=predicate.ticker,
                triggered=True,
                observed_value=float(hits.iloc[0]),
                observed_at=ts.isoformat(),
                window_days=predicate.window.lookback_days,
                reason=f"first crossing on {ts.date().isoformat()}",
            )
        return EvaluationResult(
            falsifier_idx=predicate.falsifier_idx,
            ticker=predicate.ticker,
            triggered=False,
            observed_value=float(series.iloc[-1]),
            observed_at=series.index[-1].isoformat(),
            window_days=predicate.window.lookback_days,
            reason="no crossing",
        )

    if kind in ("min_close", "max_close"):
        extreme_idx = series.idxmin() if kind == "min_close" else series.idxmax()
        extreme = float(series.loc[extreme_idx])
        triggered = _cmp(extreme, predicate.op, predicate.rhs)
        return EvaluationResult(
            falsifier_idx=predicate.falsifier_idx,
            ticker=predicate.ticker,
            triggered=triggered,
            observed_value=extreme,
            observed_at=extreme_idx.isoformat(),
            window_days=predicate.window.lookback_days,
            reason=f"window {kind} = {extreme:g}",
        )

    if kind == "drawdown_from_high":
        if predicate.op not in ("gt", "gte"):
            raise ValueError("drawdown_from_high requires op gt or gte")
        running_max = series.cummax()
        drawdown = (running_max - series) / running_max
        max_dd_idx = drawdown.idxmax()
        max_dd = float(drawdown.loc[max_dd_idx])
        triggered = _cmp(max_dd, predicate.op, predicate.rhs)
        return EvaluationResult(
            falsifier_idx=predicate.falsifier_idx,
            ticker=predicate.ticker,
            triggered=triggered,
            observed_value=max_dd,
            observed_at=max_dd_idx.isoformat(),
            window_days=predicate.window.lookback_days,
            reason=f"max drawdown {max_dd:.4f}",
        )

    raise ValueError(f"unknown window kind `{kind}`")


def evaluate_predicates(predicates: PredicateSet, source: MarketSource) -> dict:
    results = [evaluate_predicate(p, source) for p in predicates.predicates]
    return {
        "thesis_id": predicates.thesis_id,
        "evaluated_at": datetime.now(UTC).isoformat(),
        "results": [r.to_json_dict() for r in results],
        "any_triggered": any(r.triggered for r in results),
    }
