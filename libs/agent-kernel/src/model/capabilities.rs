/// Wire protocol determines whether an output ceiling must be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProtocol {
    ChatCompletions,
    Responses,
    AnthropicMessages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySource {
    Unknown,
    /// A provisional value used only because the protocol requires a number.
    ProtocolFallback,
    ProviderMetadata,
}

/// Effective capabilities for one (endpoint, model, protocol) selection.
///
/// The caller must resolve the active endpoint before supplying its catalogue
/// metadata: the same named model may have different limits on different hosts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCapabilities {
    pub protocol: ModelProtocol,
    pub context_window_tokens: Option<usize>,
    pub max_output_tokens: Option<u32>,
    pub context_source: CapabilitySource,
    pub output_source: CapabilitySource,
    /// Optional, endpoint-declared modalities; unknown is not the same as false.
    pub supports_tools: Option<bool>,
    pub supports_vision: Option<bool>,
    /// Raw supported reasoning levels, if the service provides them.
    pub reasoning_efforts: Option<Vec<String>>,
}

impl ModelCapabilities {
    pub fn resolve(
        protocol: ModelProtocol,
        _model: &str,
        declared_context: Option<usize>,
        declared_output: Option<u32>,
    ) -> Self {
        let context_window_tokens = declared_context.filter(|n| *n > 0);
        // A model name is not an authoritative capability source. Do not carry
        // a stale built-in model catalogue inside the kernel.
        // Anthropic Messages requires max_tokens; an optimistic protocol fallback
        // is reduced by the provider only when the endpoint rejects it explicitly.
        let mut output = (protocol == ModelProtocol::AnthropicMessages).then_some(128_000);
        let mut output_source = if output.is_some() {
            CapabilitySource::ProtocolFallback
        } else {
            CapabilitySource::Unknown
        };
        if let Some(limit) = declared_output.filter(|n| *n > 0) {
            output = Some(limit);
            output_source = CapabilitySource::ProviderMetadata;
        }
        Self {
            protocol,
            context_window_tokens,
            max_output_tokens: output,
            context_source: if context_window_tokens.is_some() {
                CapabilitySource::ProviderMetadata
            } else {
                CapabilitySource::Unknown
            },
            output_source,
            supports_tools: None,
            supports_vision: None,
            reasoning_efforts: None,
        }
    }

    /// Incorporate only declared capability data; do not guess abilities from a
    /// model alias or assume that an absent flag means the model lacks a feature.
    pub fn with_declared_features(
        mut self,
        supports_tools: Option<bool>,
        supports_vision: Option<bool>,
        reasoning_efforts: Option<Vec<String>>,
    ) -> Self {
        self.supports_tools = supports_tools;
        self.supports_vision = supports_vision;
        self.reasoning_efforts = reasoning_efforts;
        self
    }

    /// Automatic output policy: do not cap the Responses or Chat Completions
    /// request. Anthropic Messages requires max_tokens and must receive one.
    pub fn request_output_limit(&self) -> Option<u32> {
        match self.protocol {
            ModelProtocol::AnthropicMessages => self.max_output_tokens,
            ModelProtocol::Responses | ModelProtocol::ChatCompletions => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_compatible_protocols_never_send_a_cap() {
        for protocol in [ModelProtocol::ChatCompletions, ModelProtocol::Responses] {
            let caps = ModelCapabilities::resolve(protocol, "unknown", None, None);
            assert_eq!(caps.max_output_tokens, None);
            assert_eq!(caps.request_output_limit(), None);
            let declared = ModelCapabilities::resolve(protocol, "unknown", Some(200_000), Some(65_536));
            assert_eq!(declared.max_output_tokens, Some(65_536));
            assert_eq!(declared.request_output_limit(), None);
            assert_eq!(declared.context_source, CapabilitySource::ProviderMetadata);
        }
    }

    #[test]
    fn declared_features_preserve_unknown_and_explicit_false() {
        let unknown = ModelCapabilities::resolve(ModelProtocol::Responses, "alias", None, None);
        assert_eq!((unknown.supports_tools, unknown.supports_vision, unknown.reasoning_efforts), (None, None, None));
        let declared = ModelCapabilities::resolve(ModelProtocol::Responses, "alias", None, None)
            .with_declared_features(Some(false), Some(true), Some(vec!["high".into()]));
        assert_eq!(declared.supports_tools, Some(false));
        assert_eq!(declared.supports_vision, Some(true));
        assert_eq!(declared.reasoning_efforts, Some(vec!["high".to_string()]));
    }

    #[test]
    fn anthropic_capability_sources_and_precedence_are_explicit() {
        let unknown = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "brand-new-model-2030", None, None);
        assert_eq!(unknown.request_output_limit(), Some(128_000));
        assert_eq!(unknown.output_source, CapabilitySource::ProtocolFallback);
        let proxied = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "brand-new-model-2030", Some(80_000), Some(16_384));
        assert_eq!(proxied.request_output_limit(), Some(16_384));
        assert_eq!(proxied.output_source, CapabilitySource::ProviderMetadata);
        let old_name = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "claude-3-haiku", None, None);
        assert_eq!(old_name.request_output_limit(), Some(128_000),
            "model IDs must never pick a hard-coded legacy limit");
        assert_eq!(old_name.output_source, CapabilitySource::ProtocolFallback);
    }
}
