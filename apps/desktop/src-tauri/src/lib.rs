//! Tauri command boundary (docs/ARCHITECTURE.md invariant #1: privileged operations —
//! the filesystem, git, the shell, the local store — live here, never in the renderer).
//! Modules below are thin: each delegates to the crate that owns the actual logic and
//! translates its result into something `invoke()` can carry across the IPC boundary.

mod agent_commands;
#[cfg(test)]
mod agent_live_test;
mod fs_commands;
mod git_commands;
mod index_commands;
mod memory_commands;
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
    /// The open workspace's code index and its watcher (index_commands.rs).
    index: Mutex<index_commands::IndexSlot>,
    /// Scopes audit events that belong to the app rather than to one agent task, such as
    /// a provider being connected. One per launch.
    session_id: uuid::Uuid,
}

impl AppState {
    /// An empty state over an in-memory store, for tests.
    #[cfg(test)]
    pub(crate) fn for_tests() -> Self {
        Self {
            store: Mutex::new(Store::open_in_memory().unwrap()),
            workspace: Mutex::new(None),
            terminals: Mutex::new(HashMap::new()),
            tools: ToolRegistry::standard(),
            pending_approvals: Mutex::new(HashMap::new()),
            running_tasks: Mutex::new(HashMap::new()),
            index: Mutex::new(Default::default()),
            session_id: uuid::Uuid::new_v4(),
        }
    }

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

/// The app's compiled-in configuration and assets. One function because the macro can
/// only be expanded once per crate (it embeds Info.plist), and the tests need to inspect
/// exactly what `run()` ships.
fn app_context() -> tauri::Context<tauri::Wry> {
    tauri::generate_context!()
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
                index: Mutex::new(Default::default()),
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
            memory_commands::list_memories,
            memory_commands::add_memory,
            memory_commands::update_memory,
            memory_commands::delete_memory,
            memory_commands::export_memories,
            memory_commands::repository_instructions,
            memory_commands::preview_instruction,
            memory_commands::adopt_instruction,
            memory_commands::list_tasks,
            memory_commands::task_events,
            index_commands::index_status,
        ])
        .run(app_context())
        .expect("error while running Any Code");
}

#[cfg(test)]
mod tests {
    /// Audit S3. Tauri serves every page with a `Content-Security-Policy` header built at
    /// runtime from the embedded config (tauri `protocol/tauri.rs`), so what matters is
    /// that the context compiled into the app — the same `generate_context!` `run()`
    /// uses — carries the policy. `"csp": null` would fail this.
    #[test]
    fn the_app_ships_a_strict_content_security_policy() {
        let context = super::app_context();
        let csp = context
            .config()
            .app
            .security
            .csp
            .as_ref()
            .expect("a Content-Security-Policy must be configured")
            .to_string();
        for directive in [
            "default-src 'self'",
            "script-src 'self'",
            "object-src 'none'",
            "frame-ancestors 'none'",
            "connect-src 'self' ipc: http://ipc.localhost",
        ] {
            assert!(csp.contains(directive), "missing `{directive}` in: {csp}");
        }
        // Script execution is the thing a CSP exists to restrict here; neither escape
        // hatch may creep back in.
        let scripts = csp
            .split(';')
            .find(|d| d.trim_start().starts_with("script-src"))
            .unwrap();
        assert!(
            !scripts.contains("unsafe-inline") && !scripts.contains("unsafe-eval"),
            "{scripts}"
        );
    }
}
