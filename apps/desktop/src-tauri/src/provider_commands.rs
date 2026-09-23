//! Provider connections, model discovery, and streaming chat. This is the only place
//! that knows how to turn a provider id into a concrete `ModelProvider` — everything
//! above (the UI, the eventual agent runtime) talks to the trait, never OpenAI or
//! Anthropic by name (docs/ARCHITECTURE.md invariant #3).

use anycode_models::{
    AnthropicProvider, Message, ModelDefinition, ModelProvider, ModelRequest, OllamaProvider,
    OpenAiProvider, ProviderError, RequestMetadata, StreamEvent,
};
use anycode_store::UsageStatus;
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use crate::AppState;

/// Providers that require a user-supplied API key. Ollama is intentionally absent —
/// it's local and unauthenticated (PRD §21).
const KEYED_PROVIDERS: &[(&str, &str)] = &[("openai", "OpenAI"), ("anthropic", "Anthropic")];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub requires_key: bool,
    pub has_key: bool,
}

fn provider_error_message(err: &ProviderError) -> String {
    err.to_string()
}

pub(crate) fn build_provider(provider_id: &str) -> Result<Box<dyn ModelProvider>, String> {
    match provider_id {
        "ollama" => Ok(Box::new(OllamaProvider::default())),
        "openai" => {
            let key = anycode_secrets::get_api_key("openai")
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "no API key configured for OpenAI".to_string())?;
            Ok(Box::new(OpenAiProvider::new(key)))
        }
        "anthropic" => {
            let key = anycode_secrets::get_api_key("anthropic")
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "no API key configured for Anthropic".to_string())?;
            Ok(Box::new(AnthropicProvider::new(key)))
        }
        other => Err(format!("unknown provider: {other}")),
    }
}

#[tauri::command]
pub fn list_providers() -> Result<Vec<ProviderStatus>, String> {
    let mut statuses = vec![ProviderStatus {
        id: "ollama".into(),
        name: "Ollama".into(),
        requires_key: false,
        has_key: true,
    }];
    for (id, name) in KEYED_PROVIDERS {
        let has_key = anycode_secrets::get_api_key(id)
            .map_err(|e| e.to_string())?
            .is_some();
        statuses.push(ProviderStatus {
            id: id.to_string(),
            name: name.to_string(),
            requires_key: true,
            has_key,
        });
    }
    Ok(statuses)
}

#[tauri::command]
pub fn set_provider_key(provider: String, key: String) -> Result<(), String> {
    anycode_secrets::set_api_key(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_provider_key(provider: String) -> Result<(), String> {
    anycode_secrets::delete_api_key(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_models(provider: String) -> Result<Vec<ModelDefinition>, String> {
    let adapter = build_provider(&provider)?;
    adapter
        .models()
        .await
        .map_err(|e| provider_error_message(&e))
}

#[derive(Clone, Serialize)]
struct ChatDeltaEvent {
    text: String,
}

#[derive(Clone, Serialize)]
struct ChatDoneEvent {
    #[serde(rename = "inputTokens")]
    input_tokens: Option<u32>,
    #[serde(rename = "outputTokens")]
    output_tokens: Option<u32>,
}

#[derive(Clone, Serialize)]
struct ChatErrorEvent {
    message: String,
}

#[derive(Clone, Serialize)]
struct ChatToolCallEvent {
    id: String,
    name: String,
    arguments: Value,
}

/// Starts a streaming chat request and returns immediately with a request id; the
/// response arrives as `chat:delta:{id}` / `chat:done:{id}` / `chat:error:{id}` events
/// (same pattern as the terminal's PTY output — see terminal_commands.rs).
#[tauri::command]
pub fn send_chat(
    app: AppHandle,
    provider: String,
    model: String,
    session_id: String,
    messages: Vec<Message>,
) -> Result<String, String> {
    let adapter = build_provider(&provider)?;
    let request_id = Uuid::new_v4().to_string();
    let emit_id = request_id.clone();

    tauri::async_runtime::spawn(async move {
        let request = ModelRequest {
            model: model.clone(),
            messages,
            temperature: None,
            tools: None,
            metadata: RequestMetadata {
                session_id,
                task_id: None,
            },
        };

        let record = |input: Option<u32>, output: Option<u32>, status: UsageStatus| {
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(store) = state.store.lock() {
                    let _ = store.record_usage_event(&provider, &model, input, output, status);
                }
            }
        };

        let mut stream = match adapter.stream(request).await {
            Ok(stream) => stream,
            Err(err) => {
                record(None, None, UsageStatus::Error);
                let _ = app.emit(
                    &format!("chat:error:{emit_id}"),
                    ChatErrorEvent {
                        message: provider_error_message(&err),
                    },
                );
                return;
            }
        };

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::TextDelta { text }) => {
                    let _ = app.emit(&format!("chat:delta:{emit_id}"), ChatDeltaEvent { text });
                }
                // Surfaced but not executed: running a tool here would skip the
                // permission gate entirely (docs/ARCHITECTURE.md invariant #2/#4).
                // The orchestration loop that checks anycode-security, asks the user
                // when required, executes through anycode-tools, and feeds the result
                // back to the model is the next increment — not something to rush past
                // the one thing this whole phase exists to get right.
                Ok(StreamEvent::ToolCall {
                    id,
                    name,
                    arguments,
                }) => {
                    let _ = app.emit(
                        &format!("chat:tool_call:{emit_id}"),
                        ChatToolCallEvent {
                            id,
                            name,
                            arguments,
                        },
                    );
                }
                Ok(StreamEvent::Done { usage }) => {
                    record(
                        usage.input_tokens,
                        usage.output_tokens,
                        UsageStatus::Success,
                    );
                    let _ = app.emit(
                        &format!("chat:done:{emit_id}"),
                        ChatDoneEvent {
                            input_tokens: usage.input_tokens,
                            output_tokens: usage.output_tokens,
                        },
                    );
                }
                Err(err) => {
                    record(None, None, UsageStatus::Error);
                    let _ = app.emit(
                        &format!("chat:error:{emit_id}"),
                        ChatErrorEvent {
                            message: provider_error_message(&err),
                        },
                    );
                    break;
                }
            }
        }
    });

    Ok(request_id)
}
