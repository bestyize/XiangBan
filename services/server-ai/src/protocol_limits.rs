/// Preserve a machine-readable output-cap failure without exposing provider names,
/// credentials, request text or arbitrary upstream diagnostics.
pub(super) fn output_limit_error(text: &str) -> Option<Option<u32>> {
    let field = ["max_tokens", "max_output_tokens", "max_completion_tokens", "maximum output tokens"].iter().any(|s| text.contains(s));
    let upper = ["too large", "exceeds", "cannot exceed", "maximum allowed", "at most", "limited to", "output limit", "less than or equal", "range of max_tokens"].iter().any(|s| text.contains(s));
    let lower = ["greater than zero", "greater than 0", "at least 1", "must be positive"].iter().any(|s| text.contains(s));
    if !field || !upper || lower { return None; }
    let reported = ["output limit of ", "output limit is ", "supports at most ", "limited to ", "less than or equal to ", "maximum output tokens (", "max_tokens should be [1,", " > "].iter().find_map(|marker| {
        let tail = text.split_once(marker)?.1;
        let first = tail.find(|c: char| c.is_ascii_digit())?;
        let digits: String = tail[first..].chars().take_while(|c| c.is_ascii_digit() || *c == '_').filter(|c| *c != '_').collect();
        digits.parse::<u32>().ok().filter(|n| *n > 0 && *n <= 2_000_000)
    });
    Some(reported)
}


#[cfg(test)] mod tests {
    use super::*;
    #[test] fn output_caps_preserve_only_a_safe_numeric_limit() {
        assert_eq!(output_limit_error("max_tokens: 128000 > 64000, which is the maximum allowed number of output tokens (private model)"), Some(Some(64000)));
        assert_eq!(output_limit_error("max_tokens is too large"), Some(None));
        assert_eq!(output_limit_error("max_tokens must be greater than zero"), None);
        assert_eq!(output_limit_error("bad request"), None);
    }
}
