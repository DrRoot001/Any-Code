//! Provider abstraction (PRD §17, §79-80). Orchestration and UI depend only on
//! [`ModelProvider`] and the normalized types in [`types`] — never on a vendor SDK.
//! docs/ARCHITECTURE.md invariant #3: model-specific code stays inside its adapter.

pub mod anthropic;
mod metered;
pub mod ollama;
pub mod openai;
mod provider;
mod sse;
pub mod types;

pub use anthropic::AnthropicProvider;
pub use metered::{Metered, UsageOutcome, UsageReport, UsageSink};
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use provider::{ModelProvider, ModelStream};
pub use types::*;
