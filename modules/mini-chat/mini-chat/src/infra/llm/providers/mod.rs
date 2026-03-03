//! Provider-specific LLM adapters.
//!
//! Each adapter implements [`LlmProvider`](super::LlmProvider) by converting
//! [`LlmRequest`](super::LlmRequest) to the provider's wire format, proxying
//! through OAGW, and translating SSE events back to `TranslatedEvent`.

pub mod openai_chat;
pub mod openai_responses;

use std::sync::Arc;

use oagw_sdk::ServiceGatewayClientV1;

pub use openai_chat::OpenAiChatProvider;
pub use openai_responses::OpenAiResponsesProvider;

// ════════════════════════════════════════════════════════════════════════════
// Provider selection
// ════════════════════════════════════════════════════════════════════════════

/// Which provider adapter to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// `OpenAI` Responses API (`/v1/responses`).
    OpenAiResponses,
    /// `OpenAI` Chat Completions API (`/v1/chat/completions`).
    OpenAiChatCompletions,
}

/// Configuration for a provider adapter.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Which provider adapter to use.
    pub kind: ProviderKind,
    /// OAGW upstream alias (e.g., "openai", "azure-openai", "anthropic").
    pub upstream_alias: String,
}

/// Create a provider adapter from configuration.
#[must_use]
pub fn create_provider(
    gateway: Arc<dyn ServiceGatewayClientV1>,
    config: ProviderConfig,
) -> Arc<dyn super::LlmProvider> {
    match config.kind {
        ProviderKind::OpenAiResponses => {
            Arc::new(OpenAiResponsesProvider::new(gateway, config.upstream_alias))
        }
        ProviderKind::OpenAiChatCompletions => {
            Arc::new(OpenAiChatProvider::new(gateway, config.upstream_alias))
        }
    }
}
