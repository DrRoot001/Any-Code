//! Normalized types every provider adapter speaks (PRD §17, §79-80). Nothing here may
//! reference a specific vendor's request/response shape — that translation lives entirely
//! inside each adapter (docs/ARCHITECTURE.md invariant #3).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAuthMode {
    OAuth,
    DeviceCode,
    ApiKey,
    AccessToken,
    ServiceAccount,
    CloudCredentials,
    Local,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub auth_modes: &'static [ProviderAuthMode],
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
}

/// A model as the provider itself reports it — id and display name only. Anything a
/// provider doesn't actually expose (context window, pricing) stays absent rather than
/// guessed; see docs/SECURITY.md and the provider-adapter skill's "never invent numbers".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDefinition {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    /// A tool's result being fed back to the model. Requires `tool_call_id`.
    Tool,
}

/// One tool the assistant chose to call. Distinguishes an assistant turn that just talks
/// from one that acted — both are Role::Assistant messages, but only the latter has these.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRequest {
    /// Provider-issued id, echoed back in the Role::Tool message that answers it.
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: Role,
    /// Empty string is valid for an assistant message that is pure tool calls.
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    /// Set when `role` is `Tool`: which call this message answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    fn plain(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::plain(Role::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::plain(Role::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::plain(Role::Assistant, content)
    }
}

/// A tool the model may call, described the way every mainstream function-calling API
/// wants it: a name, a human description, and a JSON Schema for its arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Capability names are dotted (`filesystem.read.workspace`), but function-calling APIs
/// only accept `[a-zA-Z0-9_-]` — OpenAI and Anthropic reject a dotted name outright. This
/// is a wire-format detail, so adapters translate at the boundary and nothing above them
/// ever sees the encoded form. Registry names never contain `__` (anycode-tools asserts
/// it), which is what makes the mapping reversible.
pub fn tool_name_to_wire(name: &str) -> String {
    name.replace('.', "__")
}

/// Inverse of [`tool_name_to_wire`].
pub fn tool_name_from_wire(name: &str) -> String {
    name.replace("__", ".")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestMetadata {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Most tokens the model may generate in this turn. `None` leaves the provider's
    /// default. The agent loop always sets it: without a cap, one degenerate turn ran for
    /// half an hour in a live run (docs/ARCHITECTURE.md invariant #11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub metadata: RequestMetadata,
}

/// Token accounting for one completed request. `None` for a field the provider's API
/// genuinely doesn't report — never estimated in its place.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    TextDelta {
        text: String,
    },
    /// Emitted once per call, fully assembled — never as partial-argument fragments.
    /// A provider that streams arguments incrementally accumulates them internally and
    /// emits this only once a call is complete.
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
    },
    Done {
        usage: Usage,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("no API key configured for this provider")]
    MissingCredential,
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider returned an error: {0}")]
    Api(String),
    #[error("could not parse provider response: {0}")]
    Parse(String),
}
