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
    ModelFamily,
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
}

impl ModelCapabilities {
    pub fn resolve(
        protocol: ModelProtocol,
        model: &str,
        declared_context: Option<usize>,
        declared_output: Option<u32>,
    ) -> Self {
        let context_window_tokens = declared_context.filter(|n| *n > 0);
        let mut output = None;
        let mut output_source = CapabilitySource::Unknown;
        if protocol == ModelProtocol::AnthropicMessages {
            let (limit, source) = anthropic_family_limit(model);
            output = Some(limit);
            output_source = source;
        }
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
        }
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

fn anthropic_family_limit(model: &str) -> (u32, CapabilitySource) {
    const FAMILIES: &[(&str, u32)] = &[
        ("claude-fable", 128_000), ("claude-sonnet-5", 128_000),
        ("claude-opus-4-8", 128_000), ("claude-opus-4-7", 128_000),
        ("claude-opus-4-6", 128_000), ("claude-sonnet-4-6", 64_000),
        ("claude-opus-4-5", 64_000), ("claude-sonnet-4-5", 64_000),
        ("claude-haiku-4-5", 64_000), ("claude-opus-4", 32_000),
        ("claude-sonnet-4", 64_000), ("claude-3-7-sonnet", 128_000),
        ("claude-3-5-sonnet", 8_192), ("claude-3-5-haiku", 8_192),
        ("claude-3-opus", 4_096), ("claude-3-sonnet", 4_096),
        ("claude-3-haiku", 4_096), ("minimax", 131_072),
        ("qwen3", 65_536),
    ];
    let normalized = model.to_ascii_lowercase().replace('.', "-");
    match FAMILIES.iter()
        .filter(|(prefix, _)| normalized.contains(*prefix))
        .max_by_key(|(prefix, _)| prefix.len())
    {
        Some((_, limit)) => (*limit, CapabilitySource::ModelFamily),
        None => (128_000, CapabilitySource::ProtocolFallback),
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
    fn anthropic_capability_sources_and_precedence_are_explicit() {
        let known = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "claude-sonnet-4.5:fast", None, None);
        assert_eq!(known.request_output_limit(), Some(64_000));
        assert_eq!(known.output_source, CapabilitySource::ModelFamily);
        let proxied = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "claude-sonnet-4.5:fast", Some(80_000), Some(16_384));
        assert_eq!(proxied.request_output_limit(), Some(16_384));
        assert_eq!(proxied.output_source, CapabilitySource::ProviderMetadata);
        let unknown = ModelCapabilities::resolve(ModelProtocol::AnthropicMessages, "proxy-unknown-model", None, None);
        assert_eq!(unknown.request_output_limit(), Some(128_000));
        assert_eq!(unknown.output_source, CapabilitySource::ProtocolFallback);
    }
}
