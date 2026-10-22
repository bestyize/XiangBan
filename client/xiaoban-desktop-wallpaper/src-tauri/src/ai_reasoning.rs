//! Reasoning capabilities belong to models/protocols, not to endpoint domain names.
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyPreference {
    #[default] Default,
    #[serde(alias = "low")] Faster,
    Medium,
    #[serde(alias = "high")] Deeper,
    Xhigh,
    Max,
}
impl ReplyPreference {
    pub fn effort(self) -> Option<&'static str> {
        match self { Self::Default => None, Self::Faster => Some("low"), Self::Medium => Some("medium"), Self::Deeper => Some("high"), Self::Xhigh => Some("xhigh"), Self::Max => Some("max") }
    }
}
pub const EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max"];
pub fn validate(levels: &[String]) -> Result<(), String> {
    if levels.len() > EFFORTS.len() || levels.iter().enumerate().any(|(i, s)| !EFFORTS.contains(&s.as_str()) || levels[..i].contains(s)) {
        return Err("思考等级只能包含不重复的 low、medium、high、xhigh、max".into());
    }
    Ok(())
}
/// Explicit declarations override inference, including an empty list to disable the capability.
/// Inference accepts qualified model IDs (provider/model) on compatible gateways.
pub fn levels(model: &str, protocol: &str, declared: Option<&[String]>) -> Vec<String> {
    if !matches!(protocol, "responses" | "chat") { return Vec::new(); }
    if let Some(levels) = declared {
        return if validate(levels).is_ok() { levels.to_vec() } else { Vec::new() };
    }
    let name = model.trim().rsplit('/').next().unwrap_or_default().to_ascii_lowercase();
    let list: &[&str] = match name.as_str() {
        "gpt-6-astra" | "gpt-6.1-sol" | "gpt-6-sol" | "gpt-6-luna" => &["low", "medium", "high", "xhigh", "max"],
        "deepseek-flash" | "deepseek-v4-pro" | "kimi-k3" | "glm-5.2" | "glm-5.3" | "glm-5.3-flash" => &["low", "high", "max"],
        _ => &[],
    };
    list.iter().map(|s| (*s).into()).collect()
}
pub fn supported(_base_url: &str, model: &str, protocol: &str) -> bool { !levels(model, protocol, None).is_empty() }
pub fn body(protocol: &str, preference: ReplyPreference, levels: &[String]) -> Map<String, Value> {
    let effort = preference.effort().filter(|e| levels.iter().any(|v| v == e));
    match protocol {
        // Request a public reasoning summary for models whose capability is known,
        // independently of whether the user adjusted the inference effort.
        // Unknown-compatible endpoints are not sent an unsupported summary field.
        "responses" if !levels.is_empty() => {
            let mut reasoning = json!({"summary": "auto"});
            if let Some(effort) = effort { reasoning["effort"] = json!(effort); }
            Map::from_iter([("reasoning".into(), reasoning)])
        }
        "chat" => effort.map_or_else(Map::new, |effort|
            Map::from_iter([("reasoning_effort".into(), json!(effort))])),
        _ => Map::new(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gateways_and_third_party_models_are_not_rejected_by_domain() {
        for model in ["gpt-6.1-sol", "deepseek-flash", "moonshot/kimi-k3", "glm-5.3"] {
            for protocol in ["responses", "chat"] {
                assert!(supported("https://gateway.example/v1", model, protocol));
                let levels = levels(model, protocol, None);
                let wire = body(protocol, ReplyPreference::Max, &levels);
                assert_eq!(if protocol == "chat" { &wire["reasoning_effort"] } else { &wire["reasoning"]["effort"] }, "max");
                if protocol == "responses" { assert_eq!(wire["reasoning"]["summary"], "auto"); }
                let automatic = body(protocol, ReplyPreference::Default, &levels);
                if protocol == "responses" {
                    assert_eq!(automatic["reasoning"]["summary"], "auto");
                    assert!(automatic["reasoning"].get("effort").is_none());
                } else {
                    assert!(automatic.is_empty());
                }
            }
        }
    }
    #[test]
    fn explicit_alias_capabilities_can_enable_disable_or_restrict_levels() {
        let declared = vec!["low".into(), "high".into()];
        assert_eq!(levels("private-alias", "responses", Some(&declared)), declared);
        assert!(levels("gpt-6.1-sol", "responses", Some(&[])).is_empty());
        assert!(body("responses", ReplyPreference::Max, &declared).is_empty());
        assert!(levels("unknown", "chat", None).is_empty());
        assert!(validate(&["bogus".into()]).is_err());
        assert!(validate(&["high".into(), "high".into()]).is_err());
        assert!(levels("gpt-6.1-sol", "anthropic", None).is_empty());
    }
    #[test]
    fn legacy_preferences_and_canonical_level_names_are_compatible() {
        assert_eq!(serde_json::from_str::<ReplyPreference>("\"faster\"").unwrap(), ReplyPreference::Faster);
        assert_eq!(serde_json::from_str::<ReplyPreference>("\"high\"").unwrap(), ReplyPreference::Deeper);
        assert_eq!(serde_json::from_str::<ReplyPreference>("\"max\"").unwrap().effort(), Some("max"));
    }
}
