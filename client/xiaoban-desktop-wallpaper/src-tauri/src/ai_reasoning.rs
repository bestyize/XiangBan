//! User-facing reply preferences for explicitly supported endpoint/model pairs.
//! Unknown gateways keep their own defaults; model names alone do not prove capability.
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyPreference {
    #[default]
    Default,
    Faster,
    Deeper,
}

/// Keep aligned with supportsReplyPreference in src/ui/ai-config.ts.
pub fn supported(base_url: &str, model: &str, protocol: &str) -> bool {
    if protocol != "responses" {
        return false;
    }
    match base_url.trim().trim_end_matches('/') {
        "https://api.openai.com/v1" => matches!(
            model.trim(),
            "gpt-6-astra" | "gpt-6.1-sol" | "gpt-6-sol" | "gpt-6-luna"
        ),
        "https://api.deepseek.com" | "https://api.deepseek.com/v1" => {
            matches!(model.trim(), "deepseek-flash" | "deepseek-v4-pro")
        }
        _ => false,
    }
}

pub fn extra_body(
    base_url: &str,
    model: &str,
    protocol: &str,
    preference: ReplyPreference,
) -> Map<String, Value> {
    let effort = match preference {
        ReplyPreference::Default => return Map::new(),
        ReplyPreference::Faster => "low",
        ReplyPreference::Deeper => "high",
    };
    if !supported(base_url, model, protocol) {
        return Map::new();
    }
    Map::from_iter([("reasoning".into(), json!({ "effort": effort }))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_kernel::provider::{responses, wire::ProviderProfile, ChatRequest};

    #[test]
    fn preferences_reach_the_responses_request_without_changing_the_model() {
        for (url, model) in [
            ("https://api.openai.com/v1", "gpt-6.1-sol"),
            ("https://api.deepseek.com", "deepseek-flash"),
        ] {
            for (preference, expected) in [
                (ReplyPreference::Default, None),
                (ReplyPreference::Faster, Some("low")),
                (ReplyPreference::Deeper, Some("high")),
            ] {
                let mut profile = ProviderProfile::new(url, "test-key", model);
                profile.extra_body = extra_body(url, model, "responses", preference);
                let body = responses::build_request_body(
                    &profile,
                    &ChatRequest {
                        messages: vec![],
                        tools: vec![],
                    },
                );
                assert_eq!(body["model"], model);
                assert_eq!(body["reasoning"]["effort"].as_str(), expected);
                assert_eq!(body.get("reasoning").is_some(), expected.is_some());
            }
        }
    }

    #[test]
    fn unsupported_models_protocols_and_lookalike_endpoints_do_not_receive_parameters() {
        for (url, model, protocol) in [
            ("https://gateway.example/v1", "gpt-6.1-sol", "responses"),
            (
                "https://api.openai.com.evil.example/v1",
                "gpt-6.1-sol",
                "responses",
            ),
            (
                "https://api.openai.com/v1?proxy=1",
                "gpt-6.1-sol",
                "responses",
            ),
            ("http://api.openai.com/v1", "gpt-6.1-sol", "responses"),
            ("https://api.openai.com/v1", "gpt-4o", "responses"),
            ("https://api.openai.com/v1", "gpt-6.1-sol", "chat"),
            (
                "https://api.deepseek.com/anthropic",
                "deepseek-flash",
                "anthropic",
            ),
        ] {
            assert!(!supported(url, model, protocol), "{url} {model} {protocol}");
            assert!(extra_body(url, model, protocol, ReplyPreference::Deeper).is_empty());
        }
    }
}
