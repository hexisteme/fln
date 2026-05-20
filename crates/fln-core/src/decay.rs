use serde::{Deserialize, Serialize};

/// Causal Decay 파라미터 — Soros reflexivity 인코딩.
///
/// w_{t+1} = w_t · exp(-Δt/τ) · (1 - I[VIX_t > 30])
///        + α · falsifier_outcome_t · (1 - exp(-Δt/τ))
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

/// Causal Decay weight 갱신.
///
/// regime-shift signal (`regime_signal >= threshold`) 발생 시 이전 weight 를 0 으로 망각.
/// falsifier_outcome ∈ [-1, 1] — thesis 검증 결과 (양수=확증, 음수=폐기).
pub fn causal_decay_weight(
    prev_weight: f64,
    delta_days: f64,
    falsifier_outcome: f64,
    regime_signal: f64,
    params: &CausalDecayParams,
) -> f64 {
    let regime_indicator = if regime_signal >= params.regime_shift_threshold { 1.0 } else { 0.0 };
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
}
