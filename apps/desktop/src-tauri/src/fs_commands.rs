//! Filesystem commands. Every one resolves against the open workspace's root via
//! `anycode_fs`, which is what actually enforces "never outside the workspace" — this
//! file only adapts that crate's errors into strings `invoke()` can carry.

use crate::AppState;
use anycode_fs::Entry;
use tauri::State;

fn require_root<'a>(
    state: &'a State<AppState>,
) -> Result<std::sync::MutexGuard<'a, Option<crate::workspace::WorkspaceState>>, String> {
    let guard = state.workspace.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        return Err("no workspace is open".to_string());
    }
    Ok(guard)
}

#[tauri::command]
pub fn list_dir(state: State<AppState>, relative: String) -> Result<Vec<Entry>, String> {
    let guard = require_root(&state)?;
    anycode_fs::list_dir(&guard.as_ref().unwrap().fs_root, &relative).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file(state: State<AppState>, relative: String) -> Result<String, String> {
    let guard = require_root(&state)?;
    anycode_fs::read_file(&guard.as_ref().unwrap().fs_root, &relative).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file(
    state: State<AppState>,
    relative: String,
    contents: String,
) -> Result<(), String> {
    let guard = require_root(&state)?;
    anycode_fs::write_file(&guard.as_ref().unwrap().fs_root, &relative, &contents)
        .map_err(|e| e.to_string())
}
