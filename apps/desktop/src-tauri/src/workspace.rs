//! Workspace selection and persistence. Opening a workspace is the one action that
//! establishes the filesystem/git security boundary for the rest of the session — every
//! other command in `fs_commands`/`git_commands` operates relative to whatever is set
//! here, never an absolute path supplied by the renderer.

use crate::{AppState, LAST_WORKSPACE_KEY};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime, State};

pub(crate) struct WorkspaceState {
    pub fs_root: anycode_fs::WorkspaceRoot,
}

/// The open workspace's root path, or an error every command surfaces the same way:
/// there is nothing to operate on until a workspace is opened.
pub(crate) fn current_path(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.workspace.lock().map_err(|e| e.to_string())?;
    guard
        .as_ref()
        .map(|w| w.fs_root.path().to_path_buf())
        .ok_or_else(|| "no workspace is open".to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub path: String,
    pub name: String,
}

fn to_info(root: &anycode_fs::WorkspaceRoot) -> WorkspaceInfo {
    let path = root.path().to_string_lossy().into_owned();
    let name = root
        .path()
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    WorkspaceInfo { path, name }
}

/// The workspace open at the end of the previous session, if its folder still exists.
/// Phase 1 has no multi-workspace switcher yet — one workspace open at a time.
#[tauri::command]
pub fn get_last_workspace<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
) -> Result<Option<WorkspaceInfo>, String> {
    let path = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .get_setting(LAST_WORKSPACE_KEY)
            .map_err(|e| e.to_string())?
    };
    let Some(path) = path else { return Ok(None) };
    open_workspace_at(&app, &state, &path).map(Some)
}

#[tauri::command]
pub fn open_workspace<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    path: String,
) -> Result<WorkspaceInfo, String> {
    let info = open_workspace_at(&app, &state, &path)?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .set_setting(LAST_WORKSPACE_KEY, &info.path)
        .map_err(|e| e.to_string())?;
    Ok(info)
}

fn open_workspace_at<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<AppState>,
    path: &str,
) -> Result<WorkspaceInfo, String> {
    let fs_root =
        anycode_fs::WorkspaceRoot::new(path).map_err(|e| format!("cannot open '{path}': {e}"))?;
    let info = to_info(&fs_root);
    let mut workspace = state.workspace.lock().map_err(|e| e.to_string())?;
    let root = fs_root.path().to_path_buf();
    *workspace = Some(WorkspaceState { fs_root });
    drop(workspace);
    crate::index_commands::start(app, root);
    Ok(info)
}
