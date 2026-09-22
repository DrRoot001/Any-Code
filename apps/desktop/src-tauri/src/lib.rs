//! Tauri command boundary (docs/ARCHITECTURE.md invariant #1: privileged operations —
//! the filesystem, git, the shell, the local store — live here, never in the renderer).
//! Modules below are thin: each delegates to the crate that owns the actual logic and
//! translates its result into something `invoke()` can carry across the IPC boundary.

mod agent_commands;
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
    store.set_setting(THEME_KEY, &theme).map_err(|e| e.to_string())
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
            provider_commands::list_models,
            provider_commands::send_chat,
            agent_commands::run_task,
            agent_commands::respond_to_approval,
            agent_commands::cancel_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Any Code");
}
