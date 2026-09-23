//! Provider connections, model discovery, and streaming chat. This is the only place
//! that knows how to turn a provider id into a concrete `ModelProvider` — everything
//! above (the UI, the eventual agent runtime) talks to the trait, never OpenAI or
//! Anthropic by name (docs/ARCHITECTURE.md invariant #3).

use anycode_models::{
    AnthropicProvider, Message, Metered, ModelDefinition, ModelProvider, ModelRequest,
    OllamaProvider, OpenAiProvider, ProviderError, RequestMetadata, StreamEvent, UsageOutcome,
    UsageReport, UsageSink,
};
use anycode_store::UsageStatus;
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use uuid::Uuid;

use crate::AppState;

/// How a provider is connected (PRD §17, §21).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Connection {
    /// Runs on this machine; nothing to configure.
    Local,
    /// Needs an API key in the OS keychain.
    Key,
    /// Needs a base URL; a key is optional (LM Studio takes none, a gateway may).
    Endpoint,
}

/// Every provider the app can build. The single list the settings screen, the model
/// pickers and `build_adapter` agree on.
const PROVIDERS: &[(&str, &str, Connection)] = &[
    ("ollama", "Ollama", Connection::Local),
    ("openai", "OpenAI", Connection::Key),
    ("anthropic", "Anthropic", Connection::Key),
    ("gemini", "Google Gemini", Connection::Key),
    ("openrouter", "OpenRouter", Connection::Key),
    (
        "openai_compatible",
        "OpenAI-compatible endpoint",
        Connection::Endpoint,
    ),
];

/// Google's official OpenAI-compatible surface for the Gemini API (API-key mode, PRD §20).
/// Vertex AI, ADC and Google sign-in are separate modes, not built yet.
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

fn endpoint_setting(provider_id: &str) -> String {
    format!("provider.{provider_id}.base_url")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub requires_key: bool,
    pub has_key: bool,
    /// Configured by base URL rather than by key alone.
    pub needs_endpoint: bool,
    pub endpoint: Option<String>,
    /// Everything it needs is configured. Not a claim that it is reachable — Ollama is
    /// ready here even when it isn't running; the model list says whether it answers.
    pub ready: bool,
}

fn provider_error_message(err: &ProviderError) -> String {
    err.to_string()
}

/// The one place a provider is constructed. Every adapter comes back wrapped in
/// [`Metered`], so every model request is recorded to `usage_events` — including failed
/// and cancelled ones — without any call site doing it (docs/ARCHITECTURE.md
/// invariant #9).
pub(crate) fn build_provider<R: Runtime>(
    app: &AppHandle<R>,
    provider_id: &str,
) -> Result<Box<dyn ModelProvider>, String> {
    let endpoint = app
        .state::<AppState>()
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .get_setting(&endpoint_setting(provider_id))
        .map_err(|e| e.to_string())?;
    let adapter = build_adapter(provider_id, endpoint)?;
    let app = app.clone();
    let sink: UsageSink = Arc::new(move |report: UsageReport| {
        let status = match report.outcome {
            UsageOutcome::Success => UsageStatus::Success,
            UsageOutcome::Error => UsageStatus::Error,
            UsageOutcome::Cancelled => UsageStatus::Cancelled,
        };
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(store) = state.store.lock() {
                let _ = store.record_usage_event(
                    &report.provider,
                    &report.model,
                    report.input_tokens,
                    report.output_tokens,
                    status,
                );
            }
        }
    });
    Ok(Box::new(Metered::new(provider_id, adapter, sink)))
}

fn build_adapter(
    provider_id: &str,
    endpoint: Option<String>,
) -> Result<Box<dyn ModelProvider>, String> {
    let key = || anycode_secrets::get_api_key(provider_id).map_err(|e| e.to_string());
    let required_key = || key()?.ok_or_else(|| format!("no API key configured for {provider_id}"));
    match provider_id {
        "ollama" => Ok(Box::new(OllamaProvider::default())),
        "openai" => Ok(Box::new(OpenAiProvider::new(required_key()?))),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(required_key()?))),
        "gemini" => Ok(Box::new(OpenAiProvider::compatible(
            "gemini",
            "Google Gemini",
            GEMINI_BASE_URL,
            Some(required_key()?),
        ))),
        "openrouter" => Ok(Box::new(OpenAiProvider::compatible(
            "openrouter",
            "OpenRouter",
            OPENROUTER_BASE_URL,
            Some(required_key()?),
        ))),
        "openai_compatible" => {
            let base_url = endpoint.ok_or_else(|| {
                "no base URL configured for the OpenAI-compatible endpoint".to_string()
            })?;
            // Re-checked at use, not only when saved: the setting could predate the rule.
            let base_url = anycode_models::openai::validate_compatible_base_url(&base_url)?;
            Ok(Box::new(OpenAiProvider::compatible(
                "openai_compatible",
                "OpenAI-compatible endpoint",
                base_url,
                key()?,
            )))
        }
        other => Err(format!("unknown provider: {other}")),
    }
}

#[tauri::command]
pub fn list_providers(state: tauri::State<AppState>) -> Result<Vec<ProviderStatus>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    PROVIDERS
        .iter()
        .map(|&(id, name, connection)| {
            let has_key = connection != Connection::Local
                && anycode_secrets::get_api_key(id)
                    .map_err(|e| e.to_string())?
                    .is_some();
            let endpoint = if connection == Connection::Endpoint {
                store
                    .get_setting(&endpoint_setting(id))
                    .map_err(|e| e.to_string())?
            } else {
                None
            };
            let ready = match connection {
                Connection::Local => true,
                Connection::Key => has_key,
                Connection::Endpoint => endpoint.is_some(),
            };
            Ok(ProviderStatus {
                id: id.to_string(),
                name: name.to_string(),
                requires_key: connection == Connection::Key,
                has_key,
                needs_endpoint: connection == Connection::Endpoint,
                endpoint,
                ready,
            })
        })
        .collect()
}

/// Sets (or, with `None`, clears) the base URL of an endpoint-configured provider. The
/// URL is validated here, in Rust: the renderer cannot point the app at a plain-HTTP host
/// across the network (see `validate_compatible_base_url`).
#[tauri::command]
pub fn set_provider_endpoint(
    state: tauri::State<AppState>,
    provider: String,
    base_url: Option<String>,
) -> Result<(), String> {
    let is_endpoint_provider = PROVIDERS
        .iter()
        .any(|&(id, _, c)| id == provider && c == Connection::Endpoint);
    if !is_endpoint_provider {
        return Err(format!("{provider} is not configured by endpoint"));
    }
    let setting = endpoint_setting(&provider);
    match base_url {
        Some(url) => {
            let url = anycode_models::openai::validate_compatible_base_url(&url)?;
            state
                .store
                .lock()
                .map_err(|e| e.to_string())?
                .set_setting(&setting, &url)
                .map_err(|e| e.to_string())?;
            state.audit(
                "provider.connected",
                serde_json::json!({ "provider": provider, "endpoint": url }),
            );
        }
        None => {
            state
                .store
                .lock()
                .map_err(|e| e.to_string())?
                .delete_setting(&setting)
                .map_err(|e| e.to_string())?;
            state.audit(
                "provider.disconnected",
                serde_json::json!({ "provider": provider }),
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_provider_key(
    state: tauri::State<AppState>,
    provider: String,
    key: String,
) -> Result<(), String> {
    anycode_secrets::set_api_key(&provider, &key).map_err(|e| e.to_string())?;
    // PRD §92 "provider connected" / "secret changed". Which provider — never the key,
    // not even a fragment of it (docs/SECURITY.md).
    state.audit(
        "provider.connected",
        serde_json::json!({ "provider": provider }),
    );
    Ok(())
}

#[tauri::command]
pub fn remove_provider_key(state: tauri::State<AppState>, provider: String) -> Result<(), String> {
    anycode_secrets::delete_api_key(&provider).map_err(|e| e.to_string())?;
    state.audit(
        "provider.disconnected",
        serde_json::json!({ "provider": provider }),
    );
    Ok(())
}

#[tauri::command]
pub async fn list_models<R: Runtime>(
    app: AppHandle<R>,
    provider: String,
) -> Result<Vec<ModelDefinition>, String> {
    let adapter = build_provider(&app, &provider)?;
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

/// Starts a streaming chat request; the response arrives as `chat:delta:{id}` /
/// `chat:done:{id}` / `chat:error:{id}` events.
///
/// The caller supplies `request_id` and subscribes to those channels *before* calling.
/// Minting the id here meant the caller could only listen afterwards, so an instant
/// failure — a local server that isn't running — was emitted to nobody and the chat
/// waited forever. Same fix as `run_task` and `terminal_spawn`.
#[tauri::command]
pub fn send_chat<R: Runtime>(
    app: AppHandle<R>,
    request_id: String,
    provider: String,
    model: String,
    session_id: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    Uuid::parse_str(&request_id).map_err(|_| "request id must be a UUID")?;
    let adapter = build_provider(&app, &provider)?;
    let emit_id = request_id;

    tauri::async_runtime::spawn(async move {
        let request = ModelRequest {
            model,
            messages,
            temperature: None,
            tools: None,
            metadata: RequestMetadata {
                session_id,
                task_id: None,
            },
        };

        let mut stream = match adapter.stream(request).await {
            Ok(stream) => stream,
            Err(err) => {
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
                // Surfaced but never executed: chat sends no tools, and running one here
                // would skip the permission gate (docs/ARCHITECTURE.md invariants #2, #4).
                // Tools run only through the agent loop in agent_commands.rs.
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
                    let _ = app.emit(
                        &format!("chat:done:{emit_id}"),
                        ChatDoneEvent {
                            input_tokens: usage.input_tokens,
                            output_tokens: usage.output_tokens,
                        },
                    );
                }
                Err(err) => {
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anycode_tools::ToolRegistry;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[test]
    fn provider_key_changes_are_audited_without_the_key() {
        // Never the user's real keychain in a test.
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let session_id = Uuid::new_v4();
        app.manage(AppState {
            store: Mutex::new(anycode_store::Store::open_in_memory().unwrap()),
            workspace: Mutex::new(None),
            terminals: Mutex::new(HashMap::new()),
            tools: ToolRegistry::standard(),
            pending_approvals: Mutex::new(HashMap::new()),
            running_tasks: Mutex::new(HashMap::new()),
            session_id,
        });

        let secret = "sk-live-0123456789abcdef";
        set_provider_key(app.state(), "openai".into(), secret.into()).unwrap();
        remove_provider_key(app.state(), "openai".into()).unwrap();

        let state = app.state::<AppState>();
        let events = state
            .store
            .lock()
            .unwrap()
            .session_events(session_id)
            .unwrap();
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, ["provider.connected", "provider.disconnected"]);
        assert_eq!(events[0].payload["provider"], "openai");
        // The property that matters: no trace of the key, whole or in part.
        let logged = serde_json::to_string(&events).unwrap();
        assert!(
            !logged.contains(secret) && !logged.contains("0123456789"),
            "{logged}"
        );
    }

    /// Phase 2's exit condition: *the same chat task switches providers with no change
    /// anywhere else.* One request, sent twice through `send_chat`, changing nothing but
    /// the provider id — once through the Ollama adapter (NDJSON) and once through the
    /// OpenAI adapter (SSE) pointed at Ollama's OpenAI-compatible API. Two adapters and two
    /// wire formats, one server: it proves the switch and exercises the OpenAI adapter
    /// against a real endpoint. It does not prove two *vendors*; that needs a key.
    ///
    /// `ANYCODE_LIVE_MODEL=qwen2.5:3b cargo test --lib chat_switches -- --ignored --nocapture`
    #[test]
    #[ignore = "needs a local Ollama; exercises two adapters live"]
    fn the_same_chat_task_switches_providers_live() {
        use std::sync::mpsc;
        use std::time::Duration;
        use tauri::Listener;

        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let model = std::env::var("ANYCODE_LIVE_MODEL").unwrap_or_else(|_| "qwen2.5:3b".into());
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(AppState {
            store: Mutex::new(anycode_store::Store::open_in_memory().unwrap()),
            workspace: Mutex::new(None),
            terminals: Mutex::new(HashMap::new()),
            tools: ToolRegistry::standard(),
            pending_approvals: Mutex::new(HashMap::new()),
            running_tasks: Mutex::new(HashMap::new()),
            session_id: Uuid::new_v4(),
        });
        set_provider_endpoint(
            app.state(),
            "openai_compatible".into(),
            Some("http://localhost:11434/v1".into()),
        )
        .unwrap();

        let task = vec![Message::user("Reply with exactly the word: ready")];
        for provider in ["ollama", "openai_compatible"] {
            let request_id = Uuid::new_v4().to_string();
            let (tx, rx) = mpsc::channel::<(&'static str, String)>();
            for channel in ["delta", "done", "error"] {
                let tx = tx.clone();
                app.listen(format!("chat:{channel}:{request_id}"), move |e| {
                    let _ = tx.send((channel, e.payload().to_string()));
                });
            }
            send_chat(
                app.handle().clone(),
                request_id,
                provider.into(),
                model.clone(),
                Uuid::new_v4().to_string(),
                task.clone(),
            )
            .unwrap();

            let mut reply = String::new();
            loop {
                let (channel, payload) = rx
                    .recv_timeout(Duration::from_secs(600))
                    .unwrap_or_else(|_| panic!("{provider}: no reply within 10 minutes"));
                let value: serde_json::Value = serde_json::from_str(&payload).unwrap();
                match channel {
                    "delta" => reply.push_str(value["text"].as_str().unwrap_or_default()),
                    "error" => panic!("{provider}: {}", value["message"]),
                    _ => break,
                }
            }
            println!("{provider:>18}: {:?}", reply.trim());
            assert!(!reply.trim().is_empty(), "{provider} streamed no text");
        }

        // Metered at the adapter boundary: one usage row per request, per provider.
        let state = app.state::<AppState>();
        let usage = state.store.lock().unwrap().list_usage_events(10).unwrap();
        let mut providers: Vec<&str> = usage.iter().map(|u| u.provider.as_str()).collect();
        providers.sort();
        assert_eq!(providers, ["ollama", "openai_compatible"], "{usage:?}");
        for row in &usage {
            println!(
                "{:>18}: {:?} in / {:?} out, {}",
                row.provider, row.input_tokens, row.output_tokens, row.status
            );
        }
    }
}
