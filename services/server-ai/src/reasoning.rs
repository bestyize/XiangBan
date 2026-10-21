//! Public model capabilities never contain upstream model names.
pub const LEVELS: &[&str] = &["low", "medium", "high", "xhigh", "max"];
pub fn valid(levels: &[String]) -> bool {
    levels.len() <= LEVELS.len() && levels.iter().enumerate().all(|(i, s)| LEVELS.contains(&s.as_str()) && !levels[..i].contains(s))
}
pub fn levels(model: &str, declared: Option<&[String]>) -> Vec<String> {
    if let Some(v) = declared { return if valid(v) { v.to_vec() } else { Vec::new() }; }
    let name = model.trim().rsplit('/').next().unwrap_or_default().to_ascii_lowercase();
    let levels: &[&str] = match name.as_str() {
        "gpt-6-astra" | "gpt-6.1-sol" | "gpt-6-sol" | "gpt-6-luna" => &["low", "medium", "high", "xhigh", "max"],
        "deepseek-flash" | "deepseek-v4-pro" | "kimi-k3" | "glm-5.2" | "glm-5.3" | "glm-5.3-flash" => &["low", "high", "max"],
        _ => &[],
    };
    levels.iter().map(|s| (*s).into()).collect()
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn aliases_and_compatibility_caps_are_explicit() {
        assert!(levels("unknown-alias", None).is_empty());
        assert_eq!(levels("unknown-alias", Some(&["high".into()])), vec!["high"]);
        assert!(levels("gpt-6.1-sol", Some(&[])).is_empty());
        assert_eq!(levels("openai/gpt-6.1-sol", None).len(), 5);
        assert!(!valid(&["high".into(), "high".into()]));
        assert!(!valid(&["unsupported".into()]));
    }
}
