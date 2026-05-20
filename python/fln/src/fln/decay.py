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


def causal_decay_weight(
    prev_weight: float,
    delta_days: float,
    falsifier_outcome: float,
    regime_signal: float,
    params: CausalDecayParams,
) -> float:
    regime = 1.0 if regime_signal >= params.regime_shift_threshold else 0.0
    decay = math.exp(-delta_days / params.tau_days)
    return prev_weight * decay * (1.0 - regime) + params.alpha * falsifier_outcome * (1.0 - decay)
