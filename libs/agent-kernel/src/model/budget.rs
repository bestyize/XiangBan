/// Request context management is separate from the model's maximum output
/// capability: a 128K-capable model does not need 128K output reserved on
/// every short response. Never treat the generation ceiling as a window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenBudget {
    pub context_limit: usize,
    pub trim_headroom: usize,
}

impl TokenBudget {
    /// `configured_limit` is the user's desired retained conversation size;
    /// `model_window` is optional authoritative endpoint metadata.
    pub fn resolve(configured_limit: usize, model_window: Option<usize>) -> Self {
        let cap = model_window.filter(|limit| *limit > 0)
            .map_or(configured_limit, |limit| configured_limit.min(limit));
        let context_limit = cap.max(1);
        // A modest estimation/output allowance, independent of max_output_tokens.
        // Retain at least one token so impossible input-only prompts fail clearly.
        let trim_headroom = ((context_limit / 4).min(16_384) + context_limit / 20)
            .min(context_limit.saturating_sub(1));
        Self { context_limit, trim_headroom }
    }

    pub fn input_budget(self) -> usize {
        self.context_limit.saturating_sub(self.trim_headroom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_window_caps_the_user_setting_without_becoming_an_output_limit() {
        let budget = TokenBudget::resolve(256_000, Some(128_000));
        assert_eq!(budget.context_limit, 128_000);
        assert_eq!(budget.trim_headroom, 16_384 + 6_400);
        assert_eq!(budget.input_budget(), 128_000 - 22_784);
        let unknown = TokenBudget::resolve(256_000, None);
        assert_eq!(unknown.context_limit, 256_000);
        assert_eq!(unknown.trim_headroom, 16_384 + 12_800);
    }

    #[test]
    fn a_small_window_never_underflows() {
        assert_eq!(TokenBudget::resolve(1, None).input_budget(), 1);
        assert_eq!(TokenBudget::resolve(16_000, None).trim_headroom, 4_800);
        assert_eq!(TokenBudget::resolve(10_000, Some(0)).context_limit, 10_000);
    }
}
