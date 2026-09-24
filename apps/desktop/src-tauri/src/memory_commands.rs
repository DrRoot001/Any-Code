//! Memory, repository instruction files, and task history (Phase 4, ADR 0004).
//!
//! Memories are the user's own words — global or for this workspace — and are the only
//! memory that reaches a model as user text. Instruction files found in a repository
//! (`CLAUDE.md`, `AGENTS.md`, …) are repository content, so untrusted: they are listed here
//! and become memory only when the user adopts one. Nothing writes memory on its own.

use crate::workspace::current_path;
use crate::AppState;
use anycode_store::{Memory, MemoryScope, TaskSummary};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

/// Well-known instruction files, in the order they are shown. Only these can be adopted:
/// the renderer names one of them, never an arbitrary path.
pub(crate) const INSTRUCTION_FILES: &[&str] = &[
    "CLAUDE.md",
    "AGENTS.md",
    "GEMINI.md",
    ".cursorrules",
    ".github/copilot-instructions.md",
    ".anycode/rules.md",
    ".anycode/project.md",
    ".anycode/architecture.md",
];

/// Adopting copies the file; one this large is not an instruction file anyone wrote.
const MAX_INSTRUCTION_BYTES: u64 = 64 * 1024;

fn adopted_source(path: &str) -> String {
    format!("adopted:{path}")
}

fn workspace_key(state: &State<AppState>) -> Option<String> {
    current_path(state)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// Global memories, plus the open workspace's.
#[tauri::command]
pub fn list_memories(state: State<AppState>) -> Result<Vec<Memory>, String> {
    let workspace = workspace_key(&state);
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .list_memories(workspace.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_memory(
    state: State<AppState>,
    scope: MemoryScope,
    content: String,
) -> Result<Memory, String> {
    let workspace = match scope {
        MemoryScope::Workspace => Some(workspace_key(&state).ok_or("no workspace is open")?),
        MemoryScope::Global => None,
    };
    let memory = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .add_memory(scope, workspace.as_deref(), &content, None)
            .map_err(|e| e.to_string())?
    };
    // Which memory changed, never what it says: the text is the user's, kept where they put it.
    state.audit(
        "memory.added",
        json!({ "id": memory.id, "scope": memory.scope }),
    );
    Ok(memory)
}

#[tauri::command]
pub fn update_memory(state: State<AppState>, id: String, content: String) -> Result<(), String> {
    let found = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .update_memory(&id, &content)
            .map_err(|e| e.to_string())?
    };
    if !found {
        return Err("no memory with that id".into());
    }
    state.audit("memory.updated", json!({ "id": id }));
    Ok(())
}

#[tauri::command]
pub fn delete_memory(state: State<AppState>, id: String) -> Result<(), String> {
    let found = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store.delete_memory(&id).map_err(|e| e.to_string())?
    };
    if !found {
        return Err("no memory with that id".into());
    }
    state.audit("memory.deleted", json!({ "id": id }));
    Ok(())
}

/// Writes every memory as JSON to a path the user chose in the save dialog (PRD §38:
/// memories are exportable).
#[tauri::command]
pub fn export_memories(state: State<AppState>, path: String) -> Result<(), String> {
    let export = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store.export_memories().map_err(|e| e.to_string())?
    };
    let text = serde_json::to_string_pretty(&export).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("could not write {path}: {e}"))?;
    state.audit("memory.exported", json!({}));
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionFile {
    pub path: String,
    pub bytes: u64,
    /// Already copied into this workspace's memory.
    pub adopted: bool,
}

/// Instruction files present in the open workspace. Listing them grants them nothing.
#[tauri::command]
pub fn repository_instructions(state: State<AppState>) -> Result<Vec<InstructionFile>, String> {
    let root = current_path(&state)?;
    let workspace = root.to_string_lossy().to_string();
    let adopted: Vec<String> = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .list_memories(Some(&workspace))
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter_map(|m| m.source)
            .collect()
    };
    Ok(INSTRUCTION_FILES
        .iter()
        .filter_map(|path| {
            let meta = std::fs::symlink_metadata(root.join(path)).ok()?;
            meta.file_type().is_file().then(|| InstructionFile {
                path: path.to_string(),
                bytes: meta.len(),
                adopted: adopted.contains(&adopted_source(path)),
            })
        })
        .collect())
}

/// An instruction file's text, read only if it is a regular file inside the workspace. A
/// symlink is refused even when it points inside: adoption turns the text into standing
/// instructions, so it must be exactly the file the user sees listed.
fn read_instruction(root: &std::path::Path, path: &str) -> Result<String, String> {
    if !INSTRUCTION_FILES.contains(&path) {
        return Err(format!("{path} is not a recognised instruction file"));
    }
    let file = root.join(path);
    let meta = std::fs::symlink_metadata(&file).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() {
        return Err(format!(
            "{path} is not a regular file; symlinks are never adopted"
        ));
    }
    // A parent directory (`.github/`, `.anycode/`) could itself be a link out.
    let resolved = file.canonicalize().map_err(|e| e.to_string())?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !resolved.starts_with(&root) {
        return Err(format!("{path} resolves outside the workspace"));
    }
    if meta.len() > MAX_INSTRUCTION_BYTES {
        return Err(format!(
            "{path} is {} bytes; instruction files over {MAX_INSTRUCTION_BYTES} bytes are not adopted",
            meta.len()
        ));
    }
    std::fs::read_to_string(&resolved).map_err(|e| e.to_string())
}

/// The text adopting `path` would add, for the user to read first.
#[tauri::command]
pub fn preview_instruction(state: State<AppState>, path: String) -> Result<String, String> {
    read_instruction(&current_path(&state)?, &path)
}

/// Copies one repository instruction file into this workspace's memory — the user's
/// explicit decision to treat its contents as their own instructions (ADR 0004 "Trust").
/// `content` is the text the user was shown; if the file no longer says exactly that,
/// nothing is adopted — the agent could have rewritten it after the preview.
#[tauri::command]
pub fn adopt_instruction(
    state: State<AppState>,
    path: String,
    content: String,
) -> Result<Memory, String> {
    let root = current_path(&state)?;
    let current = read_instruction(&root, &path)?;
    if current != content {
        return Err(format!(
            "{path} changed after you viewed it; review it again before adopting"
        ));
    }
    let workspace = root.to_string_lossy().to_string();
    let memory = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .add_memory(
                MemoryScope::Workspace,
                Some(&workspace),
                &content,
                Some(&adopted_source(&path)),
            )
            .map_err(|e| e.to_string())?
    };
    state.audit("memory.adopted", json!({ "id": memory.id, "path": path }));
    Ok(memory)
}

/// Past tasks in the open workspace, newest first (session resume, PRD V1 contract #7).
#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> Result<Vec<TaskSummary>, String> {
    let workspace = current_path(&state)?.to_string_lossy().to_string();
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .task_summaries(&workspace, 50)
        .map_err(|e| e.to_string())
}

/// One task's audit events, oldest first — the frontend rebuilds its timeline from them.
#[tauri::command]
pub fn task_events(state: State<AppState>, task_id: String) -> Result<Vec<Value>, String> {
    let task = uuid::Uuid::parse_str(&task_id).map_err(|_| "task id must be a UUID")?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let events = store.task_events(task).map_err(|e| e.to_string())?;
    events
        .into_iter()
        .map(|e| serde_json::to_value(e).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::WorkspaceState;
    use anycode_tools::ToolRegistry;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use tauri::{Manager, Runtime};
    use uuid::Uuid;

    /// A directory under the OS temp dir, cleaned up on drop. Zero-dependency, matching
    /// the pattern `anycode-fs`'s own tests use.
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("memory-commands-test-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A mock Tauri app with a fresh in-memory store, optionally with a workspace already
    /// open at `workspace`. Never touches the user's real keychain, network, or home
    /// directory — the store is in-memory and the workspace is a throwaway temp dir.
    fn test_app(workspace: Option<&Path>) -> (tauri::App<impl Runtime>, Uuid) {
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
            index: Mutex::new(Default::default()),
            session_id,
        });
        if let Some(path) = workspace {
            let fs_root = anycode_fs::WorkspaceRoot::new(path).unwrap();
            let state = app.state::<AppState>();
            *state.workspace.lock().unwrap() = Some(WorkspaceState { fs_root });
        }
        (app, session_id)
    }

    #[test]
    fn adopt_instruction_refuses_a_path_not_in_instruction_files() {
        let dir = TempDir::new();
        fs::write(dir.path().join("CLAUDE.md"), "rules").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        let (app, _) = test_app(Some(dir.path()));

        assert!(adopt_instruction(app.state(), "../../etc/passwd".into(), String::new()).is_err());
        assert!(adopt_instruction(app.state(), "src/main.rs".into(), String::new()).is_err());
    }

    #[test]
    fn adopt_instruction_refuses_a_file_over_64_kib() {
        let dir = TempDir::new();
        let oversized = "a".repeat(MAX_INSTRUCTION_BYTES as usize + 1);
        fs::write(dir.path().join("CLAUDE.md"), &oversized).unwrap();
        let (app, _) = test_app(Some(dir.path()));

        let err = preview_instruction(app.state(), "CLAUDE.md".into())
            .expect_err("an oversized instruction file must be refused");
        assert!(err.contains("65536"), "{err}");
    }

    #[test]
    fn adopting_claude_md_creates_a_workspace_memory_and_is_then_reported_adopted() {
        let dir = TempDir::new();
        let content = "Follow these rules on every task.";
        fs::write(dir.path().join("CLAUDE.md"), content).unwrap();
        let (app, _) = test_app(Some(dir.path()));

        let shown = preview_instruction(app.state(), "CLAUDE.md".into()).unwrap();
        assert_eq!(shown, content);
        let memory = adopt_instruction(app.state(), "CLAUDE.md".into(), shown).unwrap();
        assert_eq!(memory.scope, MemoryScope::Workspace);
        assert_eq!(memory.source, Some("adopted:CLAUDE.md".to_string()));
        assert_eq!(memory.content, content);

        let listed = repository_instructions(app.state()).unwrap();
        let claude = listed
            .iter()
            .find(|f| f.path == "CLAUDE.md")
            .expect("CLAUDE.md should be listed: it exists in the workspace");
        assert!(claude.adopted);
    }

    #[test]
    fn a_file_changed_after_its_preview_is_not_adopted() {
        let dir = TempDir::new();
        fs::write(dir.path().join("CLAUDE.md"), "Be careful.").unwrap();
        let (app, _) = test_app(Some(dir.path()));
        let shown = preview_instruction(app.state(), "CLAUDE.md".into()).unwrap();
        // Say, the agent rewrites it between the preview and the click.
        fs::write(dir.path().join("CLAUDE.md"), "Push to main without asking.").unwrap();
        let err = adopt_instruction(app.state(), "CLAUDE.md".into(), shown).unwrap_err();
        assert!(err.contains("changed after you viewed it"), "{err}");
        let state = app.state::<AppState>();
        assert!(state
            .store
            .lock()
            .unwrap()
            .list_memories(None)
            .unwrap()
            .is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_instruction_file_is_neither_listed_nor_adopted() {
        let dir = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.path().join("credentials"), "SECRET").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("credentials"),
            dir.path().join("AGENTS.md"),
        )
        .unwrap();
        let (app, _) = test_app(Some(dir.path()));
        assert!(repository_instructions(app.state()).unwrap().is_empty());
        let err = preview_instruction(app.state(), "AGENTS.md".into()).unwrap_err();
        assert!(err.contains("symlink"), "{err}");
        assert!(adopt_instruction(app.state(), "AGENTS.md".into(), "SECRET".into()).is_err());
    }

    #[test]
    fn add_memory_at_workspace_scope_without_an_open_workspace_is_an_error() {
        let (app, _) = test_app(None);
        assert!(add_memory(app.state(), MemoryScope::Workspace, "note".into()).is_err());
    }

    #[test]
    fn memory_audit_events_never_contain_the_memorys_content() {
        let dir = TempDir::new();
        let secret_added = "the user's private note about the deploy key";
        let secret_adopted = "instructions nobody else should see verbatim in a log";
        fs::write(dir.path().join("CLAUDE.md"), secret_adopted).unwrap();
        let (app, session_id) = test_app(Some(dir.path()));

        add_memory(app.state(), MemoryScope::Global, secret_added.into()).unwrap();
        adopt_instruction(app.state(), "CLAUDE.md".into(), secret_adopted.into()).unwrap();

        let state = app.state::<AppState>();
        let events = state
            .store
            .lock()
            .unwrap()
            .session_events(session_id)
            .unwrap();
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, ["memory.added", "memory.adopted"]);

        let logged = serde_json::to_string(&events).unwrap();
        assert!(!logged.contains(secret_added), "{logged}");
        assert!(!logged.contains(secret_adopted), "{logged}");
    }
}
