//! Ollama adapter — local models, no credential (PRD §21). Unlike OpenAI/Anthropic,
//! Ollama streams newline-delimited JSON objects rather than SSE, so it doesn't use
//! `sse.rs`; each line is already a complete message.

use crate::provider::{ModelProvider, ModelStream};
use crate::types::{
    tool_name_from_wire, tool_name_to_wire, ModelDefinition, ModelRequest, ProviderAuthMode,
    ProviderError, ProviderManifest, Role, StreamEvent, Usage,
};
use async_stream::try_stream;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL.to_string())
    }
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

fn build_chat_request(request: &ModelRequest) -> Value {
    // Ollama correlates a tool result with its call by the tool's *name*, not by an id,
    // so remember which name each call id belonged to as the history is walked.
    let mut call_names: HashMap<&str, &str> = HashMap::new();
    let messages: Vec<Value> = request
        .messages
        .iter()
        .map(|m| {
            let mut obj = json!({ "role": role_str(m.role), "content": m.content });
            if let Some(calls) = &m.tool_calls {
                for call in calls {
                    call_names.insert(&call.id, &call.name);
                }
                obj["tool_calls"] = json!(calls
                    .iter()
                    .map(|c| json!({
                        "function": { "name": tool_name_to_wire(&c.name), "arguments": c.arguments },
                    }))
                    .collect::<Vec<_>>());
            }
            if let Some(name) = m.tool_call_id.as_deref().and_then(|id| call_names.get(id)) {
                obj["tool_name"] = json!(tool_name_to_wire(name));
            }
            obj
        })
        .collect();
    let mut body = json!({ "model": request.model, "messages": messages, "stream": true });
    if let Some(temperature) = request.temperature {
        body["options"] = json!({ "temperature": temperature });
    }
    if let Some(tools) = &request.tools {
        body["tools"] = json!(tools
            .iter()
            .map(|t| json!({
                "type": "function",
                "function": {
                    "name": tool_name_to_wire(&t.name),
                    "description": t.description,
                    "parameters": t.input_schema,
                },
            }))
            .collect::<Vec<_>>());
    }
    body
}

/// One newline-delimited JSON object -> the events it carries. Unlike OpenAI, Ollama
/// delivers each tool call whole in a single line, so there is nothing to accumulate —
/// but one line can hold several calls plus text, hence a `Vec`.
fn parse_line(line: &str) -> Result<Vec<StreamEvent>, ProviderError> {
    if line.trim().is_empty() {
        return Ok(vec![]);
    }
    let value: Value =
        serde_json::from_str(line).map_err(|e| ProviderError::Parse(e.to_string()))?;
    let mut events = Vec::new();

    if let Some(text) = value["message"]["content"]
        .as_str()
        .filter(|t| !t.is_empty())
    {
        events.push(StreamEvent::TextDelta {
            text: text.to_string(),
        });
    }
    for call in value["message"]["tool_calls"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let function = &call["function"];
        // Arguments are normally an object; tolerate a model that emits a JSON string.
        let arguments = match &function["arguments"] {
            Value::String(raw) => serde_json::from_str(raw).unwrap_or(Value::Null),
            other => other.clone(),
        };
        events.push(StreamEvent::ToolCall {
            // Ids key pending approvals app-wide, so a missing one must be unique rather
            // than a per-line counter that would repeat across rounds and tasks.
            id: call["id"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4())),
            name: tool_name_from_wire(function["name"].as_str().unwrap_or_default()),
            arguments,
        });
    }
    if value["done"].as_bool() == Some(true) {
        events.push(StreamEvent::Done {
            usage: Usage {
                input_tokens: value["prompt_eval_count"].as_u64().map(|n| n as u32),
                output_tokens: value["eval_count"].as_u64().map(|n| n as u32),
            },
        });
    }
    Ok(events)
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    fn manifest(&self) -> ProviderManifest {
        ProviderManifest {
            id: "ollama",
            name: "Ollama",
            auth_modes: &[ProviderAuthMode::Local],
            supports_streaming: true,
            // Whether a *particular* model honours tools is up to the model; Ollama returns
            // an error for one that doesn't, which surfaces as a task error, not silence.
            supports_tools: true,
            supports_vision: false,
        }
    }

    async fn models(&self) -> Result<Vec<ModelDefinition>, ProviderError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(ProviderError::Api(
                response.text().await.unwrap_or_default(),
            ));
        }
        let body: Value = response.json().await?;
        let list = body["models"]
            .as_array()
            .ok_or_else(|| ProviderError::Parse("missing models array".into()))?;
        Ok(list
            .iter()
            .filter_map(|m| m["name"].as_str())
            .map(|name| ModelDefinition {
                id: name.to_string(),
                display_name: name.to_string(),
            })
            .collect())
    }

    async fn stream(&self, request: ModelRequest) -> Result<ModelStream, ProviderError> {
        let body = build_chat_request(&request);
        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
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
            let mut buffer = String::new();
            while let Some(chunk) = bytes.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(pos) = buffer.find('\n') {
                    let line: String = buffer.drain(..=pos).collect();
                    for event in parse_line(line.trim_end())? {
                        yield event;
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

    use crate::types::{Message, RequestMetadata, ToolCallRequest, ToolDefinition};

    #[test]
    fn parses_a_content_line() {
        let events = parse_line(r#"{"message":{"content":"hi"},"done":false}"#).unwrap();
        assert_eq!(events, vec![StreamEvent::TextDelta { text: "hi".into() }]);
    }

    #[test]
    fn parses_the_final_usage_line() {
        let events = parse_line(r#"{"done":true,"prompt_eval_count":12,"eval_count":4}"#).unwrap();
        assert_eq!(
            events,
            vec![StreamEvent::Done {
                usage: Usage {
                    input_tokens: Some(12),
                    output_tokens: Some(4)
                }
            }]
        );
    }

    #[test]
    fn ignores_blank_lines() {
        assert!(parse_line("").unwrap().is_empty());
        assert!(parse_line("   ").unwrap().is_empty());
    }

    #[test]
    fn parses_a_whole_tool_call_and_decodes_its_name() {
        let line = r#"{"message":{"content":"","tool_calls":[{"function":{"name":"filesystem__read__workspace","arguments":{"path":"a.txt"}}}]},"done":false}"#;
        let events = parse_line(line).unwrap();
        let [StreamEvent::ToolCall {
            id,
            name,
            arguments,
        }] = events.as_slice()
        else {
            panic!("expected exactly one tool call, got {events:?}");
        };
        assert_eq!(name, "filesystem.read.workspace");
        assert_eq!(arguments["path"], "a.txt");
        assert!(!id.is_empty());
    }

    #[test]
    fn generated_call_ids_do_not_repeat() {
        let line =
            r#"{"message":{"tool_calls":[{"function":{"name":"git__status","arguments":{}}}]}}"#;
        let id = |events: Vec<StreamEvent>| match &events[0] {
            StreamEvent::ToolCall { id, .. } => id.clone(),
            other => panic!("{other:?}"),
        };
        assert_ne!(id(parse_line(line).unwrap()), id(parse_line(line).unwrap()));
    }

    #[test]
    fn tolerates_string_encoded_arguments() {
        let line = r#"{"message":{"tool_calls":[{"function":{"name":"shell__execute","arguments":"{\"command\":\"ls\"}"}}]}}"#;
        match &parse_line(line).unwrap()[0] {
            StreamEvent::ToolCall { arguments, .. } => assert_eq!(arguments["command"], "ls"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn request_carries_tools_and_links_results_to_their_call_by_name() {
        let request = ModelRequest {
            model: "qwen2.5:3b".into(),
            messages: vec![
                Message::user("read it"),
                Message {
                    role: Role::Assistant,
                    content: String::new(),
                    tool_calls: Some(vec![ToolCallRequest {
                        id: "call_1".into(),
                        name: "filesystem.read.workspace".into(),
                        arguments: json!({ "path": "a.txt" }),
                    }]),
                    tool_call_id: None,
                },
                Message {
                    role: Role::Tool,
                    content: "hello".into(),
                    tool_calls: None,
                    tool_call_id: Some("call_1".into()),
                },
            ],
            temperature: None,
            tools: Some(vec![ToolDefinition {
                name: "filesystem.read.workspace".into(),
                description: "read".into(),
                input_schema: json!({ "type": "object" }),
            }]),
            metadata: RequestMetadata::default(),
        };
        let body = build_chat_request(&request);
        assert_eq!(
            body["tools"][0]["function"]["name"],
            "filesystem__read__workspace"
        );
        assert_eq!(
            body["messages"][1]["tool_calls"][0]["function"]["arguments"]["path"],
            "a.txt"
        );
        assert_eq!(body["messages"][2]["role"], "tool");
        assert_eq!(
            body["messages"][2]["tool_name"],
            "filesystem__read__workspace"
        );
    }
}
