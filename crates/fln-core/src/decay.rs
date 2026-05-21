use serde::{Deserialize, Serialize};

/// Causal Decay 파라미터 — Soros reflexivity 인코딩.
///
/// w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[regime_signal ≥ θ])
///        + α · falsifier_outcome · (1 - exp(-Δt/τ))
///
/// Defaults: τ = 180 days, α = 0.1, θ = 30.0.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CausalDecayParams {
    /// 반감기 상수 (일 단위). invest=180, health=730, real-estate=365.
    pub tau_days: f64,
    /// 학습률 — falsifier_outcome 을 weight 에 반영하는 비율.
    pub alpha: f64,
    /// regime-shift threshold (e.g. VIX>30 → 30.0).
    pub regime_shift_threshold: f64,
}

impl Default for CausalDecayParams {
    fn default() -> Self {
        Self { tau_days: 180.0, alpha: 0.1, regime_shift_threshold: 30.0 }
    }
}

/// Errors that prevent a safe weight update.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecayError {
    #[error("delta_days must be finite and non-negative; got {0}")]
    InvalidDeltaDays(String),
    #[error("falsifier_outcome must be finite and within [-1, 1]; got {0}")]
    InvalidOutcome(String),
    #[error("regime_signal must be finite; got {0}")]
    InvalidRegimeSignal(String),
    #[error("prev_weight must be finite; got {0}")]
    InvalidPrevWeight(String),
    #[error("tau_days must be finite and > 0; got {0}")]
    InvalidTau(String),
}

/// Strict variant: validates every numeric input before applying the update.
///
/// Catches the v0.1 attack surface flagged by the v0.2 audit:
/// - negative `Δt` → exponential growth of weight
/// - `NaN` / `±Inf` in any input → silent NaN propagation
/// - `outcome` outside `[-1, 1]` → unbounded weight inflation
pub fn try_causal_decay_weight(
    prev_weight: f64,
    delta_days: f64,
    falsifier_outcome: f64,
    regime_signal: f64,
    params: &CausalDecayParams,
) -> Result<f64, DecayError> {
    if !prev_weight.is_finite() {
        return Err(DecayError::InvalidPrevWeight(format!("{prev_weight}")));
    }
    if !delta_days.is_finite() || delta_days < 0.0 {
        return Err(DecayError::InvalidDeltaDays(format!("{delta_days}")));
    }
    if !falsifier_outcome.is_finite() || !(-1.0..=1.0).contains(&falsifier_outcome) {
        return Err(DecayError::InvalidOutcome(format!("{falsifier_outcome}")));
    }
    if !regime_signal.is_finite() {
        return Err(DecayError::InvalidRegimeSignal(format!("{regime_signal}")));
    }
    if !params.tau_days.is_finite() || params.tau_days <= 0.0 {
        return Err(DecayError::InvalidTau(format!("{}", params.tau_days)));
    }
    Ok(causal_decay_weight_unchecked(
        prev_weight,
        delta_days,
        falsifier_outcome,
        regime_signal,
        params,
    ))
}

/// Convenience wrapper that clamps invalid inputs to the nearest sane value
/// instead of returning an error. Use when the upstream cannot recover.
pub fn causal_decay_weight(
    prev_weight: f64,
    delta_days: f64,
    falsifier_outcome: f64,
    regime_signal: f64,
    params: &CausalDecayParams,
) -> f64 {
    let prev_weight = if prev_weight.is_finite() { prev_weight } else { 0.0 };
    let delta_days = if delta_days.is_finite() { delta_days.max(0.0) } else { 0.0 };
    let outcome = if falsifier_outcome.is_finite() {
        falsifier_outcome.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let regime = if regime_signal.is_finite() { regime_signal } else { 0.0 };
    let safe_params = if params.tau_days.is_finite() && params.tau_days > 0.0 {
        *params
    } else {
        CausalDecayParams::default()
    };
    causal_decay_weight_unchecked(prev_weight, delta_days, outcome, regime, &safe_params)
}

fn causal_decay_weight_unchecked(
    prev_weight: f64,
    delta_days: f64,
    falsifier_outcome: f64,
    regime_signal: f64,
    params: &CausalDecayParams,
) -> f64 {
    let regime_indicator =
        if regime_signal >= params.regime_shift_threshold { 1.0 } else { 0.0 };
    let decay = (-delta_days / params.tau_days).exp();
    let memory_term = prev_weight * decay * (1.0 - regime_indicator);
    let update_term = params.alpha * falsifier_outcome * (1.0 - decay);
    memory_term + update_term
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regime_shift_resets_weight() {
        let params = CausalDecayParams::default();
        let w = causal_decay_weight(0.9, 10.0, 0.0, 35.0, &params);
        assert!(w.abs() < 1e-9, "regime shift must wipe memory: {w}");
    }

    #[test]
    fn decay_alone_reduces_weight() {
        let params = CausalDecayParams::default();
        let w = causal_decay_weight(1.0, 180.0, 0.0, 10.0, &params);
        assert!((w - (-1.0f64).exp()).abs() < 1e-9);
    }

    #[test]
    fn falsifier_outcome_pushes_weight() {
        let params = CausalDecayParams::default();
        let positive = causal_decay_weight(0.0, 30.0, 1.0, 10.0, &params);
        let negative = causal_decay_weight(0.0, 30.0, -1.0, 10.0, &params);
        assert!(positive > 0.0);
        assert!(negative < 0.0);
    }

    #[test]
    fn try_variant_rejects_negative_delta() {
        let params = CausalDecayParams::default();
        let err = try_causal_decay_weight(0.5, -1.0, 0.0, 0.0, &params).unwrap_err();
        assert_eq!(err, DecayError::InvalidDeltaDays("-1".into()));
    }

    #[test]
    fn try_variant_rejects_outcome_out_of_range() {
        let params = CausalDecayParams::default();
        assert!(matches!(
            try_causal_decay_weight(0.0, 30.0, 1.5, 0.0, &params),
            Err(DecayError::InvalidOutcome(_))
        ));
    }

    #[test]
    fn try_variant_rejects_nan_anywhere() {
        let params = CausalDecayParams::default();
        let nan = f64::NAN;
        assert!(try_causal_decay_weight(nan, 1.0, 0.0, 0.0, &params).is_err());
        assert!(try_causal_decay_weight(0.0, nan, 0.0, 0.0, &params).is_err());
        assert!(try_causal_decay_weight(0.0, 1.0, nan, 0.0, &params).is_err());
        assert!(try_causal_decay_weight(0.0, 1.0, 0.0, nan, &params).is_err());
    }

    #[test]
    fn lenient_variant_clamps_negative_delta_to_zero() {
        let params = CausalDecayParams::default();
        // Δt = 0 → decay = 1 → memory_term = prev_weight, update_term = 0
        let w = causal_decay_weight(0.5, -10.0, 1.0, 0.0, &params);
        assert!((w - 0.5).abs() < 1e-9);
    }

    #[test]
    fn lenient_variant_clamps_nan_inputs() {
        let params = CausalDecayParams::default();
        let w = causal_decay_weight(f64::NAN, f64::INFINITY, f64::NAN, f64::NAN, &params);
        assert!(w.is_finite());
    }
}
