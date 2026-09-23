//! Tauri command boundary (docs/ARCHITECTURE.md invariant #1: privileged operations —
//! the filesystem, git, the shell, the local store — live here, never in the renderer).
//! Modules below are thin: each delegates to the crate that owns the actual logic and
//! translates its result into something `invoke()` can carry across the IPC boundary.

mod agent_commands;
#[cfg(test)]
mod agent_live_test;
mod fs_commands;
mod git_commands;
mod provider_commands;
mod terminal_commands;
mod workspace;

use agent_commands::PendingApproval;
use anycode_store::Store;
use anycode_tools::ToolRegistry;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use workspace::WorkspaceState;

pub(crate) struct AppState {
    store: Mutex<Store>,
    workspace: Mutex<Option<WorkspaceState>>,
    terminals: Mutex<HashMap<String, anycode_terminal::PtySession>>,
    tools: ToolRegistry,
    pending_approvals: Mutex<HashMap<String, PendingApproval>>,
    /// Cancellation flag per running agent task, keyed by task id.
    running_tasks: Mutex<HashMap<String, Arc<AtomicBool>>>,
    /// Scopes audit events that belong to the app rather than to one agent task, such as
    /// a provider being connected. One per launch.
    session_id: uuid::Uuid,
}

impl AppState {
    /// Appends an app-level event to the audit log (docs/ARCHITECTURE.md invariant #10).
    /// A write failure is reported, never allowed to fail the action being audited.
    fn audit(&self, kind: &str, payload: serde_json::Value) {
        let event = anycode_core::Event::new(
            kind,
            anycode_core::EventScope {
                session_id: self.session_id,
                ..Default::default()
            },
            payload,
        );
        let written = match self.store.lock() {
            Ok(store) => store.append_event(&event).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        if let Err(err) = written {
            eprintln!("audit log write failed for {kind}: {err}");
        }
    }
}

const THEME_KEY: &str = "theme";
const LAST_WORKSPACE_KEY: &str = "last_workspace";
const DEFAULT_THEME: &str = "system";

#[tauri::command]
fn get_theme(state: State<AppState>) -> Result<String, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .get_setting(THEME_KEY)
        .map(|value| value.unwrap_or_else(|| DEFAULT_THEME.to_string()))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_theme(state: State<AppState>, theme: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .set_setting(THEME_KEY, &theme)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let store = Store::open(data_dir.join("anycode.db"))
                .map_err(|e| format!("failed to open local store: {e}"))?;
            app.manage(AppState {
                store: Mutex::new(store),
                workspace: Mutex::new(None),
                terminals: Mutex::new(HashMap::new()),
                tools: ToolRegistry::standard(),
                pending_approvals: Mutex::new(HashMap::new()),
                running_tasks: Mutex::new(HashMap::new()),
                session_id: uuid::Uuid::new_v4(),
            });
            // Sourcing the user's shell profile can take seconds; do it now, off the UI,
            // so an agent's first command doesn't wait for it.
            std::thread::spawn(|| {
                anycode_terminal::login_shell_path();
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                if let Some(state) = window.try_state::<AppState>() {
                    if let Ok(mut terminals) = state.terminals.lock() {
                        for (_, mut session) in terminals.drain() {
                            let _ = session.kill();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_theme,
            set_theme,
            workspace::get_last_workspace,
            workspace::open_workspace,
            fs_commands::list_dir,
            fs_commands::read_file,
            fs_commands::write_file,
            git_commands::git_status,
            git_commands::git_diff,
            git_commands::git_branch,
            terminal_commands::terminal_spawn,
            terminal_commands::terminal_write,
            terminal_commands::terminal_resize,
            terminal_commands::terminal_kill,
            provider_commands::list_providers,
            provider_commands::set_provider_key,
            provider_commands::remove_provider_key,
            provider_commands::set_provider_endpoint,
            provider_commands::list_models,
            provider_commands::send_chat,
            agent_commands::run_task,
            agent_commands::respond_to_approval,
            agent_commands::cancel_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Any Code");
}
