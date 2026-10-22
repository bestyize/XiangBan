/// Differentiate a context overflow (which may need history trimming) from
/// an explicit generation-cap rejection (which must NEVER trigger compaction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitFailure {
    InputContextOverflow,
    OutputLimit,
    Other,
}

pub fn classify_limit_failure(message: &str) -> LimitFailure {
    let text = message.to_ascii_lowercase();
    let input_signals = [
        "too many input tokens", "prompt is too long", "input token count exceeds",
        "context_length_exceeded", "context_window_exceeded",
    ];
    if input_signals.iter().any(|signal| text.contains(signal)) {
        return LimitFailure::InputContextOverflow;
    }
    if is_output_cap_error(&text) {
        return LimitFailure::OutputLimit;
    }
    if ["maximum context length", "input messages exceed the context"]
        .iter().any(|signal| text.contains(signal))
    {
        return LimitFailure::InputContextOverflow;
    }
    LimitFailure::Other
}

fn is_output_cap_error(text: &str) -> bool {
    let output_field = [
        "max_tokens", "max_output_tokens", "max_completion_tokens", "maximum output tokens",
    ].iter().any(|s| text.contains(s));
    let cap_signal = [
        "too large", "exceeds", "cannot exceed", "greater than", "at most",
        "maximum allowed", "maximum output tokens", "output limit", "limited to",
        "less than or equal", "must be <", "range of max_tokens",
        "available_tokens", "available tokens",
    ].iter().any(|s| text.contains(s));
    output_field && cap_signal
}

/// Reduce only an explicitly rejected max_tokens request. An ambiguous error
/// cannot justify changing the model capability or dropping conversation turns.
pub fn next_output_limit(message: &str, requested: u32) -> Option<u32> {
    if requested <= 1 || classify_limit_failure(message) != LimitFailure::OutputLimit {
        return None;
    }
    let text = message.to_ascii_lowercase();
    fn after(text: &str, marker: &str) -> Option<u32> {
        let tail = text.split_once(marker)?.1;
        let first = tail.find(|c: char| c.is_ascii_digit())?;
        let digits: String = tail[first..].chars()
            .take_while(|c| c.is_ascii_digit() || *c == '_')
            .filter(|c| *c != '_').collect();
        digits.parse::<u32>().ok().filter(|n| *n > 0)
    }
    let reported = [
        "available_tokens:", "available tokens:", "output limit of ",
        "output limit is ", "supports at most ", "limited to ",
        "less than or equal to ", "maximum output tokens (",
        "max_tokens should be [1,", "max_tokens should be [1, ",
    ].iter().find_map(|marker| after(&text, marker))
        .or_else(|| {
            if text.contains("max_tokens:") && text.contains(" > ") && text.contains("maximum allowed") {
                after(&text, " > ")
            } else { None }
        });
    let target = reported.map_or(requested / 2, |value| value.saturating_sub(1));
    (target > 0 && target < requested).then_some(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_compress_when_the_output_ceiling_is_the_actual_problem() {
        assert_eq!(classify_limit_failure("max_tokens exceeds model's maximum output tokens (65536)"), LimitFailure::OutputLimit);
        assert_eq!(classify_limit_failure("context_length_exceeded"), LimitFailure::InputContextOverflow);
        assert_eq!(classify_limit_failure("HTTP 400: bad request"), LimitFailure::Other);
        assert_eq!(next_output_limit("max_tokens: 128000 > 64000, which is the maximum allowed number of output tokens", 128_000), Some(63_999));
        assert_eq!(next_output_limit("range of max_tokens should be [1, 65536]", 128_000), Some(65_535));
        assert_eq!(next_output_limit("max_tokens is too large", 128_000), Some(64_000));
        assert_eq!(next_output_limit("prompt is too long; max_tokens is too large", 128_000), None);
    }
}
