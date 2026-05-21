"""Causal Decay — Soros reflexivity weight update.

Wire-compatible with the Rust ``fln-core::decay`` formula:

    w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[regime_signal ≥ threshold])
           + α · falsifier_outcome_t · (1 - exp(-Δt/τ))
"""

from __future__ import annotations

import math
from dataclasses import dataclass


@dataclass
class CausalDecayParams:
    tau_days: float = 180.0
    alpha: float = 0.1
    regime_shift_threshold: float = 30.0


class DecayError(ValueError):
    """Raised when the strict variant detects an invalid numeric input."""


def _isfinite(x: float) -> bool:
    return math.isfinite(x)


def try_causal_decay_weight(
    prev_weight: float,
    delta_days: float,
    falsifier_outcome: float,
    regime_signal: float,
    params: CausalDecayParams,
) -> float:
    """Strict variant: validate every numeric input.

    v0.2 hardening — catches the audit-flagged inputs that previously
    propagated as silent NaN or exponential blow-up:
      * negative ``Δt``
      * ``NaN`` / ``±Inf`` in any field
      * ``outcome`` outside ``[-1, 1]``
      * non-positive ``tau_days``
    """
    if not _isfinite(prev_weight):
        raise DecayError(f"prev_weight must be finite; got {prev_weight}")
    if not _isfinite(delta_days) or delta_days < 0.0:
        raise DecayError(f"delta_days must be finite and ≥ 0; got {delta_days}")
    if not _isfinite(falsifier_outcome) or not -1.0 <= falsifier_outcome <= 1.0:
        raise DecayError(f"falsifier_outcome must be in [-1, 1]; got {falsifier_outcome}")
    if not _isfinite(regime_signal):
        raise DecayError(f"regime_signal must be finite; got {regime_signal}")
    if not _isfinite(params.tau_days) or params.tau_days <= 0.0:
        raise DecayError(f"tau_days must be > 0; got {params.tau_days}")
    return _decay_unchecked(prev_weight, delta_days, falsifier_outcome, regime_signal, params)


def causal_decay_weight(
    prev_weight: float,
    delta_days: float,
    falsifier_outcome: float,
    regime_signal: float,
    params: CausalDecayParams,
) -> float:
    """Lenient variant — clamps invalid inputs instead of raising."""
    prev_weight = prev_weight if _isfinite(prev_weight) else 0.0
    delta_days = max(0.0, delta_days) if _isfinite(delta_days) else 0.0
    falsifier_outcome = (
        max(-1.0, min(1.0, falsifier_outcome)) if _isfinite(falsifier_outcome) else 0.0
    )
    regime_signal = regime_signal if _isfinite(regime_signal) else 0.0
    safe_params = (
        params
        if _isfinite(params.tau_days) and params.tau_days > 0
        else CausalDecayParams()
    )
    return _decay_unchecked(
        prev_weight, delta_days, falsifier_outcome, regime_signal, safe_params
    )


def _decay_unchecked(
    prev_weight: float,
    delta_days: float,
    falsifier_outcome: float,
    regime_signal: float,
    params: CausalDecayParams,
) -> float:
    regime = 1.0 if regime_signal >= params.regime_shift_threshold else 0.0
    decay = math.exp(-delta_days / params.tau_days)
    return (
        prev_weight * decay * (1.0 - regime)
        + params.alpha * falsifier_outcome * (1.0 - decay)
    )
