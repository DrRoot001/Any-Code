//! Anthropic adapter — Messages API. BYOK only (PRD §19): subscription-based auth is
//! only implemented if Anthropic documents a flow this client may use, which it
//! currently doesn't for third-party apps — no reverse-engineered auth.

use crate::provider::{ModelProvider, ModelStream};
use crate::sse::SseDecoder;
use crate::types::{
    ModelDefinition, ModelRequest, ProviderAuthMode, ProviderError, ProviderManifest, Role,
    StreamEvent, Usage,
};
use async_stream::try_stream;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::{json, Value};

const BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 4096;

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

/// Anthropic takes `system` as a top-level field, not a message with role "system".
fn build_messages_request(request: &ModelRequest) -> Value {
    let mut system = None;
    let mut messages = Vec::new();
    for m in &request.messages {
        match m.role {
            Role::System => system = Some(m.content.clone()),
            Role::User | Role::Tool => {
                messages.push(json!({ "role": "user", "content": m.content }))
            }
            Role::Assistant => messages.push(json!({ "role": "assistant", "content": m.content })),
        }
    }
    let mut body = json!({
        "model": request.model,
        "messages": messages,
        // Anthropic requires a cap; the caller's, when it gave one.
        "max_tokens": request.max_output_tokens.unwrap_or(MAX_TOKENS),
        "stream": true,
    });
    if let Some(system) = system {
        body["system"] = json!(system);
    }
    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }
    body
}

/// One named SSE event -> zero or more normalized events. Anthropic spreads usage
/// across `message_start` (input_tokens) and `message_delta` (output_tokens).
fn parse_event(event_name: Option<&str>, data: &str) -> Result<Vec<StreamEvent>, ProviderError> {
    let value: Value =
        serde_json::from_str(data).map_err(|e| ProviderError::Parse(e.to_string()))?;
    let mut events = Vec::new();

    match event_name {
        Some("content_block_delta") => {
            if let Some(text) = value["delta"]["text"].as_str() {
                events.push(StreamEvent::TextDelta {
                    text: text.to_string(),
                });
            }
        }
        Some("message_delta") => {
            if let Some(output) = value["usage"]["output_tokens"].as_u64() {
                events.push(StreamEvent::Done {
                    usage: Usage {
                        input_tokens: None,
                        output_tokens: Some(output as u32),
                    },
                });
            }
        }
        _ => {}
    }
    Ok(events)
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn manifest(&self) -> ProviderManifest {
        ProviderManifest {
            id: "anthropic",
            name: "Anthropic",
            auth_modes: &[ProviderAuthMode::ApiKey],
            supports_streaming: true,
            // Anthropic represents tool calls/results as typed content blocks
            // (tool_use/tool_result), not OpenAI's flat tool_call_id — that's a real
            // format difference, not a detail, so it gets its own increment rather
            // than a half-correct mapping here. See PRD §26.1: capabilities are
            // declared, never assumed.
            supports_tools: false,
            supports_vision: true,
        }
    }

    async fn models(&self) -> Result<Vec<ModelDefinition>, ProviderError> {
        let response = self
            .client
            .get(format!("{BASE_URL}/models"))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(ProviderError::Api(
                response.text().await.unwrap_or_default(),
            ));
        }
        let body: Value = response.json().await?;
        let list = body["data"]
            .as_array()
            .ok_or_else(|| ProviderError::Parse("missing data array".into()))?;
        Ok(list
            .iter()
            .filter_map(|m| {
                Some((
                    m["id"].as_str()?,
                    m["display_name"].as_str().unwrap_or_default(),
                ))
            })
            .map(|(id, name)| ModelDefinition {
                id: id.to_string(),
                display_name: if name.is_empty() {
                    id.to_string()
                } else {
                    name.to_string()
                },
            })
            .collect())
    }

    async fn stream(&self, request: ModelRequest) -> Result<ModelStream, ProviderError> {
        let body = build_messages_request(&request);
        let response = self
            .client
            .post(format!("{BASE_URL}/messages"))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ProviderError::Api(
                response.text().await.unwrap_or_default(),
            ));
        }

        let stream = try_stream! {
            let mut bytes = response.bytes_stream();
            let mut decoder = SseDecoder::new();
            while let Some(chunk) = bytes.next().await {
                let chunk = chunk?;
                let text = String::from_utf8_lossy(&chunk);
                for event in decoder.push(&text) {
                    for parsed in parse_event(event.event.as_deref(), &event.data)? {
                        yield parsed;
                    }
                }
            }
        };
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, RequestMetadata};

    #[test]
    fn a_callers_output_cap_replaces_the_default() {
        let request = ModelRequest {
            model: "m".into(),
            max_output_tokens: Some(321),
            messages: vec![Message::user("hi")],
            temperature: None,
            tools: None,
            metadata: RequestMetadata::default(),
        };
        assert_eq!(build_messages_request(&request)["max_tokens"], 321);
    }

    #[test]
    fn splits_system_message_from_the_transcript() {
        let request = ModelRequest {
            model: "claude-opus-5".into(),
            max_output_tokens: None,
            messages: vec![Message::system("be terse"), Message::user("hi")],
            temperature: None,
            tools: None,
            metadata: RequestMetadata::default(),
        };
        let body = build_messages_request(&request);
        assert_eq!(body["system"], "be terse");
        assert_eq!(body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(body["messages"][0]["role"], "user");
    }

    #[test]
    fn parses_content_block_delta() {
        let events =
            parse_event(Some("content_block_delta"), r#"{"delta":{"text":"hi"}}"#).unwrap();
        assert_eq!(events, vec![StreamEvent::TextDelta { text: "hi".into() }]);
    }

    #[test]
    fn parses_output_usage_from_message_delta() {
        let events =
            parse_event(Some("message_delta"), r#"{"usage":{"output_tokens":7}}"#).unwrap();
        assert_eq!(
            events,
            vec![StreamEvent::Done {
                usage: Usage {
                    input_tokens: None,
                    output_tokens: Some(7)
                }
            }]
        );
    }

    #[test]
    fn ignores_unhandled_event_types() {
        assert_eq!(parse_event(Some("ping"), "{}").unwrap(), vec![]);
    }
}
