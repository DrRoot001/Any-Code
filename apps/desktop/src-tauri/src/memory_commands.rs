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
            let meta = std::fs::metadata(root.join(path)).ok()?;
            meta.is_file().then(|| InstructionFile {
                path: path.to_string(),
                bytes: meta.len(),
                adopted: adopted.contains(&adopted_source(path)),
            })
        })
        .collect())
}

/// Copies one repository instruction file into this workspace's memory — the user's
/// explicit decision to treat its contents as their own instructions (ADR 0004 "Trust").
#[tauri::command]
pub fn adopt_instruction(state: State<AppState>, path: String) -> Result<Memory, String> {
    if !INSTRUCTION_FILES.contains(&path.as_str()) {
        return Err(format!("{path} is not a recognised instruction file"));
    }
    let root = current_path(&state)?;
    let file = root.join(&path);
    let size = std::fs::metadata(&file).map_err(|e| e.to_string())?.len();
    if size > MAX_INSTRUCTION_BYTES {
        return Err(format!(
            "{path} is {size} bytes; instruction files over {MAX_INSTRUCTION_BYTES} bytes are not adopted"
        ));
    }
    let content = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
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
