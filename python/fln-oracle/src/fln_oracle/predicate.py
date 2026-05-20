"""Structured predicates — the machine-readable side of a Falsifier.

A Falsifier's free-form ``condition`` field stays for humans; a paired
``*.predicates.json`` file carries the predicates the oracle actually
evaluates. This keeps signed thesis bytes immutable while letting evaluation
results accumulate alongside.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

Op = Literal["lt", "lte", "gt", "gte"]
Field = Literal["close", "high", "low", "open"]
WindowKind = Literal["any_close", "min_close", "max_close", "drawdown_from_high"]


@dataclass(frozen=True)
class Window:
    kind: WindowKind
    lookback_days: int

    @classmethod
    def from_dict(cls, d: dict[str, Any]) -> "Window":
        return cls(kind=d["kind"], lookback_days=int(d["lookback_days"]))


@dataclass(frozen=True)
class Predicate:
    """Single structured predicate.

    Semantics:
      * `any_close`: predicate holds if any 1-D close in the last
        ``window.lookback_days`` satisfies ``field op rhs``.
      * `min_close` / `max_close`: predicate holds if the period's min/max
        close satisfies the comparison.
      * `drawdown_from_high`: predicate holds if (peak - trough) / peak
        crosses ``rhs`` within the window. ``op`` MUST be ``gt`` or ``gte``.
    """

    falsifier_idx: int
    ticker: str
    field: Field
    op: Op
    rhs: float
    window: Window

    @classmethod
    def from_dict(cls, d: dict[str, Any]) -> "Predicate":
        return cls(
            falsifier_idx=int(d["falsifier_idx"]),
            ticker=str(d["ticker"]),
            field=d["field"],
            op=d["op"],
            rhs=float(d["rhs"]),
            window=Window.from_dict(d["window"]),
        )


@dataclass(frozen=True)
class PredicateSet:
    thesis_id: str
    predicates: list[Predicate]

    @classmethod
    def load(cls, path: str | Path) -> "PredicateSet":
        data = json.loads(Path(path).read_text())
        return cls(
            thesis_id=str(data["thesis_id"]),
            predicates=[Predicate.from_dict(p) for p in data["predicates"]],
        )
