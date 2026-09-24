//! OpenAI adapter — Chat Completions API. BYOK only (PRD §18): official ChatGPT sign-in
//! isn't something a third-party app can embed, so this speaks the plain API-key path.
//!
//! The same wire protocol is spoken by OpenRouter, Google's Gemini API (through its
//! official OpenAI-compatible endpoint), LM Studio, vLLM, llama.cpp and Ollama's `/v1`, so
//! one tested implementation serves them all (PRD §21 "OpenAI compatible: base URL + API
//! key"). Each keeps its own identity, so usage and errors are attributed correctly.

use crate::provider::{ModelProvider, ModelStream};
use crate::sse::SseDecoder;
use crate::types::{
    tool_name_from_wire, tool_name_to_wire, ModelDefinition, ModelRequest, ProviderAuthMode,
    ProviderError, ProviderManifest, Role, StreamEvent, Usage,
};
use async_stream::try_stream;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAiProvider {
    id: &'static str,
    name: &'static str,
    base_url: String,
    /// `None` for a local endpoint that takes no credential.
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self::compatible("openai", "OpenAI", OPENAI_BASE_URL, Some(api_key))
    }

    /// Any endpoint that speaks the Chat Completions protocol. `base_url` is the part
    /// before `/chat/completions`, e.g. `https://openrouter.ai/api/v1`.
    pub fn compatible(
        id: &'static str,
        name: &'static str,
        base_url: impl Into<String>,
        api_key: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.authorised(self.client.get(format!("{}{path}", self.base_url)))
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.authorised(self.client.post(format!("{}{path}", self.base_url)))
    }

    fn authorised(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => request.bearer_auth(key),
            None => request,
        }
    }
}

/// Where a user-supplied endpoint may point. HTTPS anywhere; plain HTTP only to this
/// machine, since anything else would send the prompt — repository code — and any key in
/// cleartext across a network.
pub fn validate_compatible_base_url(url: &str) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("not a valid URL: {e}"))?;
    let loopback = matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
    );
    match parsed.scheme() {
        "https" => Ok(url.to_string()),
        "http" if loopback => Ok(url.to_string()),
        "http" => Err("plain http is only allowed to this machine (localhost); use https".into()),
        other => Err(format!(
            "unsupported scheme {other}: use https, or http to localhost"
        )),
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

fn message_to_json(m: &crate::types::Message) -> Value {
    let mut obj = json!({ "role": role_str(m.role), "content": m.content });
    if let Some(tool_calls) = &m.tool_calls {
        obj["tool_calls"] = json!(tool_calls
            .iter()
            .map(|tc| json!({
                "id": tc.id,
                "type": "function",
                "function": { "name": tool_name_to_wire(&tc.name), "arguments": tc.arguments.to_string() },
            }))
            .collect::<Vec<_>>());
    }
    if let Some(id) = &m.tool_call_id {
        obj["tool_call_id"] = json!(id);
    }
    obj
}

/// `limit_field` is where the output cap goes: OpenAI's current models reject the legacy
/// `max_tokens` in favour of `max_completion_tokens`; compatible servers (Ollama, LM
/// Studio, OpenRouter, Gemini) take `max_tokens`.
fn build_chat_request(request: &ModelRequest, limit_field: &str) -> Value {
    let messages: Vec<Value> = request.messages.iter().map(message_to_json).collect();
    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }
    if let Some(limit) = request.max_output_tokens {
        body[limit_field] = json!(limit);
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

/// A tool call as it arrives split across many chunks: an id and name on the first
/// fragment, an `arguments` JSON string built up one piece at a time after that.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ToolCallFragment {
    id: Option<String>,
    name: Option<String>,
    arguments_fragment: Option<String>,
}

/// Extracts this chunk's `delta.tool_calls[]` fragments, keyed by their stream index
/// (OpenAI interleaves fragments from multiple parallel calls by index, not by id).
fn extract_tool_call_fragments(chunk: &Value) -> Vec<(usize, ToolCallFragment)> {
    let Some(calls) = chunk["choices"][0]["delta"]["tool_calls"].as_array() else {
        return vec![];
    };
    calls
        .iter()
        .map(|call| {
            let index = call["index"].as_u64().unwrap_or(0) as usize;
            (
                index,
                ToolCallFragment {
                    id: call["id"].as_str().map(String::from),
                    name: call["function"]["name"].as_str().map(String::from),
                    arguments_fragment: call["function"]["arguments"].as_str().map(String::from),
                },
            )
        })
        .collect()
}

fn finish_reason(chunk: &Value) -> Option<&str> {
    chunk["choices"][0]["finish_reason"].as_str()
}

/// One decoded SSE `data:` payload -> zero or more normalized events, except tool
/// calls: those accumulate across many chunks (see [`extract_tool_call_fragments`]) so
/// they can't be emitted from a single chunk in isolation. The caller in `stream()`
/// owns that accumulation; this function only handles what's complete within one chunk.
/// OpenAI sends a terminal literal `[DONE]` rather than a JSON object.
fn parse_chunk(data: &str) -> Result<Vec<StreamEvent>, ProviderError> {
    if data == "[DONE]" {
        return Ok(vec![]);
    }
    let value: Value =
        serde_json::from_str(data).map_err(|e| ProviderError::Parse(e.to_string()))?;
    let mut events = Vec::new();

    if let Some(text) = value["choices"][0]["delta"]["content"].as_str() {
        events.push(StreamEvent::TextDelta {
            text: text.to_string(),
        });
    }
    if let Some(usage) = value.get("usage").filter(|u| !u.is_null()) {
        events.push(StreamEvent::Done {
            usage: Usage {
                input_tokens: usage["prompt_tokens"].as_u64().map(|n| n as u32),
                output_tokens: usage["completion_tokens"].as_u64().map(|n| n as u32),
            },
        });
    }
    Ok(events)
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    fn manifest(&self) -> ProviderManifest {
        ProviderManifest {
            id: self.id,
            name: self.name,
            auth_modes: if self.api_key.is_some() {
                &[ProviderAuthMode::ApiKey]
            } else {
                &[ProviderAuthMode::Local]
            },
            supports_streaming: true,
            // Whether a given model honours tools is the endpoint's call; one that doesn't
            // returns an error, which surfaces as a task error rather than silence.
            supports_tools: true,
            // Only claimed where it has been built against; unknown for other endpoints.
            supports_vision: self.id == "openai",
        }
    }

    async fn models(&self) -> Result<Vec<ModelDefinition>, ProviderError> {
        let response = self.get("/models").send().await?;
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
            .filter_map(|m| m["id"].as_str())
            .map(|id| ModelDefinition {
                id: id.to_string(),
                display_name: id.to_string(),
            })
            .collect())
    }

    async fn stream(&self, request: ModelRequest) -> Result<ModelStream, ProviderError> {
        let limit_field = if self.id == "openai" {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        let body = build_chat_request(&request, limit_field);
        let response = self.post("/chat/completions").json(&body).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::Api(
                response.text().await.unwrap_or_default(),
            ));
        }

        let stream = try_stream! {
            let mut bytes = response.bytes_stream();
            let mut decoder = SseDecoder::new();
            let mut calls: BTreeMap<usize, ToolCallFragment> = BTreeMap::new();

            while let Some(chunk) = bytes.next().await {
                let chunk = chunk?;
                let text = String::from_utf8_lossy(&chunk);
                for event in decoder.push(&text) {
                    if event.data == "[DONE]" {
                        continue;
                    }
                    let value: Value = serde_json::from_str(&event.data)
                        .map_err(|e| ProviderError::Parse(e.to_string()))?;

                    for (index, fragment) in extract_tool_call_fragments(&value) {
                        let entry = calls.entry(index).or_default();
                        if let Some(id) = fragment.id { entry.id = Some(id); }
                        if let Some(name) = fragment.name { entry.name = Some(name); }
                        if let Some(piece) = fragment.arguments_fragment {
                            entry.arguments_fragment.get_or_insert_with(String::new).push_str(&piece);
                        }
                    }

                    if finish_reason(&value) == Some("tool_calls") {
                        for (_, call) in std::mem::take(&mut calls) {
                            let raw = call.arguments_fragment.unwrap_or_default();
                            let arguments = serde_json::from_str(&raw).unwrap_or(Value::Null);
                            yield StreamEvent::ToolCall {
                                id: call.id.unwrap_or_default(),
                                name: tool_name_from_wire(&call.name.unwrap_or_default()),
                                arguments,
                            };
                        }
                    }

                    for parsed in parse_chunk(&event.data)? {
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
    use crate::types::{Message, RequestMetadata, ToolDefinition};

    #[test]
    fn compatible_endpoints_must_not_send_code_in_cleartext_across_a_network() {
        assert!(validate_compatible_base_url("https://openrouter.ai/api/v1").is_ok());
        assert!(validate_compatible_base_url("http://localhost:1234/v1").is_ok());
        assert!(validate_compatible_base_url("http://127.0.0.1:11434/v1/").is_ok());
        assert_eq!(
            validate_compatible_base_url("http://localhost:1234/v1/").unwrap(),
            "http://localhost:1234/v1"
        );
        assert!(validate_compatible_base_url("http://10.0.0.5:8000/v1").is_err());
        assert!(validate_compatible_base_url("http://gateway.example.com/v1").is_err());
        assert!(validate_compatible_base_url("ftp://localhost/v1").is_err());
        assert!(validate_compatible_base_url("not a url").is_err());
    }

    #[test]
    fn a_compatible_provider_keeps_its_own_identity() {
        let local =
            OpenAiProvider::compatible("lmstudio", "LM Studio", "http://localhost:1234/v1/", None);
        assert_eq!(local.manifest().id, "lmstudio");
        assert_eq!(local.manifest().auth_modes, &[ProviderAuthMode::Local]);
        assert_eq!(local.base_url, "http://localhost:1234/v1");
        assert!(!local.manifest().supports_vision);
        assert_eq!(OpenAiProvider::new("sk".into()).manifest().id, "openai");
    }

    #[test]
    fn the_output_cap_uses_the_field_each_endpoint_accepts() {
        let request = ModelRequest {
            model: "m".into(),
            max_output_tokens: Some(321),
            messages: vec![Message::user("hi")],
            temperature: None,
            tools: None,
            metadata: RequestMetadata::default(),
        };
        // OpenAI's current models reject the legacy field; compatible servers expect it.
        let openai = build_chat_request(&request, "max_completion_tokens");
        assert_eq!(openai["max_completion_tokens"], 321);
        assert!(openai.get("max_tokens").is_none());
        assert_eq!(
            build_chat_request(&request, "max_tokens")["max_tokens"],
            321
        );
    }

    #[test]
    fn builds_a_streaming_chat_request() {
        let request = ModelRequest {
            model: "gpt-5".into(),
            max_output_tokens: None,
            messages: vec![Message::user("hi")],
            temperature: Some(0.5),
            tools: None,
            metadata: RequestMetadata::default(),
        };
        let body = build_chat_request(&request, "max_tokens");
        assert_eq!(body["model"], "gpt-5");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["stream"], true);
        assert_eq!(body["temperature"], 0.5);
    }

    #[test]
    fn includes_tool_definitions_when_present() {
        let request = ModelRequest {
            model: "gpt-5".into(),
            max_output_tokens: None,
            messages: vec![Message::user("hi")],
            temperature: None,
            tools: Some(vec![ToolDefinition {
                name: "git.status".into(),
                description: "Show working tree status".into(),
                input_schema: json!({ "type": "object", "properties": {} }),
            }]),
            metadata: RequestMetadata::default(),
        };
        let body = build_chat_request(&request, "max_tokens");
        assert_eq!(body["tools"][0]["type"], "function");
        // OpenAI rejects any function name outside [a-zA-Z0-9_-] with a 400, so the dotted
        // capability name must never reach the wire as-is.
        let wire = body["tools"][0]["function"]["name"].as_str().unwrap();
        assert!(
            wire.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "{wire} is not a valid OpenAI function name"
        );
        assert_eq!(tool_name_from_wire(wire), "git.status");
    }

    #[test]
    fn history_tool_calls_use_the_wire_name() {
        let message = crate::types::Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: Some(vec![crate::types::ToolCallRequest {
                id: "call_1".into(),
                name: "filesystem.read.workspace".into(),
                arguments: json!({ "path": "a.txt" }),
            }]),
            tool_call_id: None,
        };
        let json = message_to_json(&message);
        assert_eq!(
            json["tool_calls"][0]["function"]["name"],
            "filesystem__read__workspace"
        );
    }

    #[test]
    fn serializes_a_tool_result_message_with_its_call_id() {
        let message = crate::types::Message {
            role: Role::Tool,
            content: "{\"entries\":[]}".into(),
            tool_calls: None,
            tool_call_id: Some("call_1".into()),
        };
        let json = message_to_json(&message);
        assert_eq!(json["role"], "tool");
        assert_eq!(json["tool_call_id"], "call_1");
    }

    #[test]
    fn parses_a_text_delta() {
        let events = parse_chunk(r#"{"choices":[{"delta":{"content":"hi"}}]}"#).unwrap();
        assert_eq!(events, vec![StreamEvent::TextDelta { text: "hi".into() }]);
    }

    #[test]
    fn parses_final_usage_chunk() {
        let events =
            parse_chunk(r#"{"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":3}}"#)
                .unwrap();
        assert_eq!(
            events,
            vec![StreamEvent::Done {
                usage: Usage {
                    input_tokens: Some(10),
                    output_tokens: Some(3)
                }
            }]
        );
    }

    #[test]
    fn done_marker_yields_no_events() {
        assert_eq!(parse_chunk("[DONE]").unwrap(), vec![]);
    }

    #[test]
    fn extracts_tool_call_fragments_by_index() {
        let chunk = json!({
            "choices": [{
                "delta": {
                    "tool_calls": [
                        { "index": 0, "id": "call_1", "function": { "name": "git.status", "arguments": "" } }
                    ]
                }
            }]
        });
        let fragments = extract_tool_call_fragments(&chunk);
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].0, 0);
        assert_eq!(fragments[0].1.id.as_deref(), Some("call_1"));
        assert_eq!(fragments[0].1.name.as_deref(), Some("git.status"));
    }

    #[test]
    fn accumulates_argument_fragments_across_chunks() {
        let first = json!({
            "choices": [{ "delta": { "tool_calls": [
                { "index": 0, "id": "call_1", "function": { "name": "filesystem.write.workspace", "arguments": "{\"path\":" } }
            ] } }]
        });
        let second = json!({
            "choices": [{ "delta": { "tool_calls": [
                { "index": 0, "function": { "arguments": "\"a.txt\"}" } }
            ] } }]
        });

        let mut buffer = String::new();
        for chunk in [&first, &second] {
            for (_, fragment) in extract_tool_call_fragments(chunk) {
                if let Some(piece) = fragment.arguments_fragment {
                    buffer.push_str(&piece);
                }
            }
        }
        let parsed: Value = serde_json::from_str(&buffer).unwrap();
        assert_eq!(parsed["path"], "a.txt");
    }

    #[test]
    fn recognizes_the_tool_calls_finish_reason() {
        let chunk = json!({ "choices": [{ "finish_reason": "tool_calls" }] });
        assert_eq!(finish_reason(&chunk), Some("tool_calls"));
        assert_eq!(finish_reason(&json!({ "choices": [{}] })), None);
    }
}
