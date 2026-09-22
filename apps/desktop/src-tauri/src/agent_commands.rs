//! The agent loop (Phase 3). Given an instruction, lets the model call tools from the
//! standard `ToolRegistry` — but every call crosses `anycode-security`'s gate first.
//! This is the only place in the application that ever calls `Tool::execute`: no other
//! code path lets a model-originated request touch the filesystem, git, or a shell
//! (docs/ARCHITECTURE.md invariants #2 and #4).
//!
//! Completion carries evidence, not a claim. The runtime records which files actually
//! changed (a real `git status` delta) and which commands actually ran with which exit
//! codes, independently of whatever the model says it did — so "it works" can't be
//! asserted into being true (PRD §8.6, §33).

use crate::provider_commands::build_provider;
use crate::workspace::current_path;
use crate::AppState;
use anycode_models::{
    Message, ModelRequest, RequestMetadata, Role, StreamEvent, ToolCallRequest, ToolDefinition,
};
use anycode_security::{decide, Decision, StandingGrant};
use anycode_store::UsageStatus;
use anycode_tools::ToolContext;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;

/// The user's answer to a `task:approval_requested` prompt. `AllowWorkspace` persists
/// a standing grant (anycode-store); `AllowOnce` only affects this one call.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalResponse {
    AllowOnce,
    AllowWorkspace,
    Deny,
}

/// A pending approval is abandoned rather than left open forever — an agent task
/// shouldn't hang indefinitely because a prompt was never answered.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(300);
/// Bounds a runaway tool-call loop (a model that never stops asking for tools).
const MAX_TOOL_ROUNDS: u32 = 8;
/// The tool whose results count as executed commands in a task's evidence.
const SHELL_TOOL: &str = "shell.execute";

/// A pending approval remembers which task raised it, so cancelling that task can
/// release it instead of leaving the agent blocked on a prompt nobody will answer.
pub(crate) struct PendingApproval {
    pub task_id: String,
    pub responder: oneshot::Sender<ApprovalResponse>,
}

#[derive(Clone, Serialize)]
struct TaskDeltaEvent {
    text: String,
}
#[derive(Clone, Serialize)]
struct TaskToolCallEvent {
    id: String,
    name: String,
    arguments: Value,
    risk: &'static str,
}
#[derive(Clone, Serialize)]
struct TaskToolResultEvent {
    id: String,
    name: String,
    result: Value,
}
#[derive(Clone, Serialize)]
struct TaskApprovalRequestedEvent {
    id: String,
    name: String,
    arguments: Value,
    risk: &'static str,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskDoneEvent {
    text: String,
    evidence: TaskEvidence,
    usage: TaskUsage,
}

/// Tokens the task actually consumed, summed across rounds. Reported in tokens rather
/// than currency: a dollar figure would require a price table this app has no
/// authoritative source for, and inventing one is exactly what PRD §100 forbids.
#[derive(Clone, Copy, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct TaskUsage {
    input_tokens: u32,
    output_tokens: u32,
}
#[derive(Clone, Serialize)]
struct TaskErrorEvent {
    message: String,
}

/// One command the agent actually ran. `exit_code` is whatever the process returned —
/// `None` only when the platform reported no code (killed by a signal).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandRecord {
    command: String,
    exit_code: Option<i64>,
}

/// What the runtime observed during a task, as opposed to what the model claimed.
/// Both fields are measured: `files_changed` is the difference between `git status`
/// before and after, `commands` comes from real `shell.execute` exit codes.
#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct TaskEvidence {
    files_changed: Vec<String>,
    commands: Vec<CommandRecord>,
}

fn risk_label(risk: anycode_security::RiskLevel) -> &'static str {
    use anycode_security::RiskLevel::*;
    match risk {
        Low => "low",
        Medium => "medium",
        High => "high",
        Critical => "critical",
    }
}

/// Paths git currently reports as dirty. An error (not a repository, git unavailable)
/// yields an empty set rather than failing the task — evidence is best-effort, but it
/// is never invented: if we can't observe it, we report nothing changed observed.
fn dirty_paths(workspace_path: &Path) -> BTreeSet<String> {
    anycode_git::status(workspace_path)
        .map(|entries| entries.into_iter().map(|e| e.path).collect())
        .unwrap_or_default()
}

/// What the model is told about its situation. It is instructed to verify its own work,
/// but nothing here decides whether verification happened — `TaskEvidence` does, from
/// recorded exit codes.
fn system_prompt(workspace_path: &Path) -> String {
    format!(
        "You are the Any Code agent, working in the repository at {}.\n\
         \n\
         Use the tools to inspect the workspace before changing it — never guess a file's \
         contents. Paths are relative to the repository root.\n\
         \n\
         Before you report a task as complete, verify it: run the project's own build, \
         lint, or test command with {SHELL_TOOL} and read the real output. If a command \
         fails, fix the cause and run it again. Never state that something works when you \
         have not run it; if you could not verify something, say so plainly.\n\
         \n\
         A tool call may be refused by the user or by workspace policy. If that happens, \
         do not retry it in a loop — explain what you needed it for and stop.",
        workspace_path.display()
    )
}

/// Starts an agent task. Progress arrives as `task:delta:{id}` / `task:tool_call:{id}` /
/// `task:approval_requested:{id}` / `task:tool_result:{id}` / `task:done:{id}` /
/// `task:error:{id}` / `task:cancelled:{id}`.
///
/// The caller supplies `task_id` rather than receiving one back, so it can subscribe to
/// those channels *before* the task starts. Generating the id here would leave a window
/// between spawning the task and the caller learning its id, in which an early failure
/// would be emitted to nobody and the task would appear to run forever.
#[tauri::command]
pub fn run_task(
    app: AppHandle,
    task_id: String,
    provider: String,
    model: String,
    session_id: String,
    instruction: String,
) -> Result<String, String> {
    if task_is_running(&app, &task_id) {
        return Err("a task with that id is already running".to_string());
    }
    let adapter = build_provider(&provider)?;
    let workspace_path = {
        let state = app.state::<AppState>();
        current_path(&state)?
    };
    let fs_root = anycode_fs::WorkspaceRoot::new(&workspace_path).map_err(|e| e.to_string())?;

    let emit_id = task_id.clone();

    let cancelled = Arc::new(AtomicBool::new(false));
    register_task(&app, &task_id, cancelled.clone());

    tauri::async_runtime::spawn(async move {
        let tool_defs: Vec<ToolDefinition> = {
            let state = app.state::<AppState>();
            state
                .tools
                .specs()
                .into_iter()
                .map(|s| ToolDefinition {
                    name: s.name.to_string(),
                    description: s.description.to_string(),
                    input_schema: s.input_schema,
                })
                .collect()
        };

        let record_usage = |input: Option<u32>, output: Option<u32>, status: UsageStatus| {
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(store) = state.store.lock() {
                    let _ = store.record_usage_event(&provider, &model, input, output, status);
                }
            }
        };

        let dirty_before = dirty_paths(&workspace_path);
        let mut evidence = TaskEvidence::default();
        let mut usage_total = TaskUsage::default();

        let mut messages = vec![
            Message::system(system_prompt(&workspace_path)),
            Message::user(instruction),
        ];
        let mut final_text = String::new();

        for _round in 0..MAX_TOOL_ROUNDS {
            if cancelled.load(Ordering::Relaxed) {
                finish_cancelled(&app, &emit_id);
                return;
            }

            let request = ModelRequest {
                model: model.clone(),
                messages: messages.clone(),
                temperature: None,
                tools: Some(tool_defs.clone()),
                metadata: RequestMetadata {
                    session_id: session_id.clone(),
                    task_id: Some(emit_id.clone()),
                },
            };

            let mut stream = match adapter.stream(request).await {
                Ok(s) => s,
                Err(err) => {
                    record_usage(None, None, UsageStatus::Error);
                    finish_error(&app, &emit_id, err.to_string());
                    return;
                }
            };

            let mut round_text = String::new();
            let mut tool_calls: Vec<ToolCallRequest> = Vec::new();

            while let Some(event) = stream.next().await {
                if cancelled.load(Ordering::Relaxed) {
                    drop(stream);
                    finish_cancelled(&app, &emit_id);
                    return;
                }
                match event {
                    Ok(StreamEvent::TextDelta { text }) => {
                        round_text.push_str(&text);
                        let _ =
                            app.emit(&format!("task:delta:{emit_id}"), TaskDeltaEvent { text });
                    }
                    Ok(StreamEvent::ToolCall {
                        id,
                        name,
                        arguments,
                    }) => {
                        tool_calls.push(ToolCallRequest {
                            id,
                            name,
                            arguments,
                        });
                    }
                    Ok(StreamEvent::Done { usage }) => {
                        usage_total.input_tokens += usage.input_tokens.unwrap_or(0);
                        usage_total.output_tokens += usage.output_tokens.unwrap_or(0);
                        record_usage(usage.input_tokens, usage.output_tokens, UsageStatus::Success);
                    }
                    Err(err) => {
                        record_usage(None, None, UsageStatus::Error);
                        finish_error(&app, &emit_id, err.to_string());
                        return;
                    }
                }
            }

            final_text.push_str(&round_text);

            if tool_calls.is_empty() {
                evidence.files_changed = changed_since(&workspace_path, &dirty_before);
                finish_done(&app, &emit_id, final_text, evidence, usage_total);
                return;
            }

            messages.push(Message {
                role: Role::Assistant,
                content: round_text,
                tool_calls: Some(tool_calls.clone()),
                tool_call_id: None,
            });

            for call in tool_calls {
                if cancelled.load(Ordering::Relaxed) {
                    finish_cancelled(&app, &emit_id);
                    return;
                }

                let result = run_gated_tool(&app, &emit_id, &workspace_path, &fs_root, &call).await;

                if call.name == SHELL_TOOL {
                    if let Some(command) = call.arguments["command"].as_str() {
                        evidence.commands.push(CommandRecord {
                            command: command.to_string(),
                            exit_code: result["exitCode"].as_i64(),
                        });
                    }
                }

                let _ = app.emit(
                    &format!("task:tool_result:{emit_id}"),
                    TaskToolResultEvent {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        result: result.clone(),
                    },
                );
                messages.push(Message {
                    role: Role::Tool,
                    content: result.to_string(),
                    tool_calls: None,
                    tool_call_id: Some(call.id),
                });
            }
        }

        finish_error(
            &app,
            &emit_id,
            format!("stopped after {MAX_TOOL_ROUNDS} tool-call rounds without finishing"),
        );
    });

    Ok(task_id)
}

/// Paths dirty now that weren't dirty when the task started. A file the agent edited
/// and then reverted correctly doesn't appear — it genuinely didn't change.
fn changed_since(workspace_path: &Path, before: &BTreeSet<String>) -> Vec<String> {
    dirty_paths(workspace_path)
        .into_iter()
        .filter(|path| !before.contains(path))
        .collect()
}

fn finish_done(
    app: &AppHandle,
    task_id: &str,
    text: String,
    evidence: TaskEvidence,
    usage: TaskUsage,
) {
    unregister_task(app, task_id);
    let _ = app.emit(
        &format!("task:done:{task_id}"),
        TaskDoneEvent {
            text,
            evidence,
            usage,
        },
    );
}

fn finish_error(app: &AppHandle, task_id: &str, message: String) {
    unregister_task(app, task_id);
    let _ = app.emit(&format!("task:error:{task_id}"), TaskErrorEvent { message });
}

fn finish_cancelled(app: &AppHandle, task_id: &str) {
    unregister_task(app, task_id);
    let _ = app.emit(&format!("task:cancelled:{task_id}"), ());
}

/// Runs one tool call through the permission gate: compute risk, check for a standing
/// workspace grant, decide, execute or ask or deny. Always returns a JSON value — a
/// denial or an execution failure is data the model needs to see, not a Rust error.
async fn run_gated_tool(
    app: &AppHandle,
    task_id: &str,
    workspace_path: &Path,
    fs_root: &anycode_fs::WorkspaceRoot,
    call: &ToolCallRequest,
) -> Value {
    let Some(risk) = tool_risk(app, call) else {
        return json!({ "error": format!("unknown tool: {}", call.name) });
    };

    let workspace_key = workspace_path.to_string_lossy().to_string();
    let grant = if has_grant(app, &call.name, &workspace_key) {
        StandingGrant::WorkspaceAllowed
    } else {
        StandingGrant::None
    };

    let _ = app.emit(
        &format!("task:tool_call:{task_id}"),
        TaskToolCallEvent {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments: call.arguments.clone(),
            risk: risk_label(risk),
        },
    );

    let approved = match decide(risk, grant) {
        Decision::Deny => {
            return json!({ "error": "denied by workspace policy", "risk": risk_label(risk) });
        }
        Decision::Allow => true,
        Decision::Ask => match request_approval(app, task_id, call, risk).await {
            ApprovalResponse::AllowOnce => true,
            ApprovalResponse::AllowWorkspace => {
                grant_permission(app, &call.name, &workspace_key);
                true
            }
            ApprovalResponse::Deny => false,
        },
    };

    if !approved {
        return json!({ "error": "denied by user" });
    }

    execute_tool(app, workspace_path, fs_root, call).await
}

/// Each of these touches `AppState` in its own function so the `State` guard and any
/// `MutexGuard` it produces are released at return, rather than living across an await.
fn tool_risk(app: &AppHandle, call: &ToolCallRequest) -> Option<anycode_security::RiskLevel> {
    let state = app.state::<AppState>();
    let tool = state.tools.get(&call.name)?;
    Some(tool.risk(&call.arguments))
}

fn has_grant(app: &AppHandle, capability: &str, workspace_key: &str) -> bool {
    let state = app.state::<AppState>();
    let Ok(store) = state.store.lock() else {
        return false;
    };
    store.has_permission_grant(capability, workspace_key).unwrap_or(false)
}

fn grant_permission(app: &AppHandle, capability: &str, workspace_key: &str) {
    let state = app.state::<AppState>();
    if let Ok(store) = state.store.lock() {
        let _ = store.grant_permission(capability, workspace_key);
    };
}

async fn execute_tool(
    app: &AppHandle,
    workspace_path: &Path,
    fs_root: &anycode_fs::WorkspaceRoot,
    call: &ToolCallRequest,
) -> Value {
    let ctx = ToolContext {
        fs_root: fs_root.clone(),
        workspace_path: workspace_path.to_path_buf(),
    };
    // Re-looked-up rather than held across the await: `State` must not outlive a yield.
    let state = app.state::<AppState>();
    let Some(tool) = state.tools.get(&call.name) else {
        return json!({ "error": format!("unknown tool: {}", call.name) });
    };
    match tool.execute(call.arguments.clone(), &ctx).await {
        Ok(value) => value,
        Err(err) => json!({ "error": err.to_string() }),
    }
}

/// Asks the frontend to approve `call` and waits for `respond_to_approval` to answer —
/// or the timeout, treated as a denial rather than leaving the task hanging forever.
async fn request_approval(
    app: &AppHandle,
    task_id: &str,
    call: &ToolCallRequest,
    risk: anycode_security::RiskLevel,
) -> ApprovalResponse {
    let (tx, rx) = oneshot::channel();
    insert_pending_approval(app, call.id.clone(), task_id.to_string(), tx);

    let _ = app.emit(
        &format!("task:approval_requested:{task_id}"),
        TaskApprovalRequestedEvent {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments: call.arguments.clone(),
            risk: risk_label(risk),
        },
    );

    match tokio::time::timeout(APPROVAL_TIMEOUT, rx).await {
        Ok(Ok(response)) => response,
        // Timed out, or the sender was dropped because the task was cancelled.
        _ => {
            remove_pending_approval(app, &call.id);
            ApprovalResponse::Deny
        }
    }
}

fn insert_pending_approval(
    app: &AppHandle,
    id: String,
    task_id: String,
    responder: oneshot::Sender<ApprovalResponse>,
) {
    let state = app.state::<AppState>();
    if let Ok(mut pending) = state.pending_approvals.lock() {
        pending.insert(
            id,
            PendingApproval {
                task_id,
                responder,
            },
        );
    };
}

fn remove_pending_approval(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    if let Ok(mut pending) = state.pending_approvals.lock() {
        pending.remove(id);
    };
}

fn task_is_running(app: &AppHandle, task_id: &str) -> bool {
    let state = app.state::<AppState>();
    let Ok(running) = state.running_tasks.lock() else {
        return false;
    };
    running.contains_key(task_id)
}

fn register_task(app: &AppHandle, task_id: &str, flag: Arc<AtomicBool>) {
    let state = app.state::<AppState>();
    if let Ok(mut running) = state.running_tasks.lock() {
        running.insert(task_id.to_string(), flag);
    };
}

fn unregister_task(app: &AppHandle, task_id: &str) {
    let state = app.state::<AppState>();
    if let Ok(mut running) = state.running_tasks.lock() {
        running.remove(task_id);
    };
}

/// Answers a pending `task:approval_requested` prompt raised by [`run_task`].
#[tauri::command]
pub fn respond_to_approval(
    app: AppHandle,
    id: String,
    response: ApprovalResponse,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let pending = state
        .pending_approvals
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&id)
        .ok_or_else(|| "no pending approval with that id".to_string())?;
    pending
        .responder
        .send(response)
        .map_err(|_| "approval request was already abandoned".to_string())
}

/// Stops a running task (docs/ARCHITECTURE.md invariant #11: every agent task is
/// cancellable). The loop notices between rounds, between stream events, and between
/// tool calls; any approval it was waiting on is released so it can't block the exit.
#[tauri::command]
pub fn cancel_task(app: AppHandle, task_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();

    {
        let running = state.running_tasks.lock().map_err(|e| e.to_string())?;
        match running.get(&task_id) {
            Some(flag) => flag.store(true, Ordering::Relaxed),
            None => return Err("no running task with that id".to_string()),
        }
    }

    // Dropping the responder resolves the waiting `rx` as an error, which
    // `request_approval` already treats as a denial.
    let mut pending = state.pending_approvals.lock().map_err(|e| e.to_string())?;
    pending.retain(|_, approval| approval.task_id != task_id);

    Ok(())
}
