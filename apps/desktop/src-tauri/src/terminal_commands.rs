//! PTY session commands. Sessions live in `AppState.terminals`, keyed by a UUID the
//! frontend treats as opaque. Output is pushed to the renderer as events rather than
//! polled — a blocking `Read` on the PTY's own thread is the only sane way to consume
//! it, so a background thread per session forwards bytes as `terminal:data` events.
//!
//! Bytes are base64-encoded rather than sent as a JS string: shell output is arbitrary
//! bytes (ANSI escapes, non-UTF-8 from some programs), and chunk boundaries can split a
//! multi-byte UTF-8 sequence in half. xterm.js accepts a `Uint8Array` directly, so the
//! frontend decodes base64 back to bytes instead of losing fidelity through a lossy
//! UTF-8 conversion here.

use crate::workspace::current_path;
use crate::AppState;
use base64::Engine;
use serde::Serialize;
use std::io::Read;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

#[derive(Clone, Serialize)]
struct PtyDataEvent<'a> {
    id: &'a str,
    /// base64-encoded raw bytes.
    data: String,
}

#[derive(Clone, Serialize)]
struct PtyExitEvent<'a> {
    id: &'a str,
}

/// The caller supplies `id` so it can subscribe to `terminal:data:{id}` *before* the
/// shell starts. Minting the id here would emit the shell's greeting and first prompt
/// into a void, leaving the user staring at a blank pane.
#[tauri::command]
pub fn terminal_spawn<R: Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let cwd = current_path(&state)?;
    {
        let terminals = state.terminals.lock().map_err(|e| e.to_string())?;
        if terminals.contains_key(&id) {
            return Err("a terminal session with that id already exists".to_string());
        }
    }

    let (session, mut reader) =
        anycode_terminal::PtySession::spawn(&cwd, cols, rows).map_err(|e| e.to_string())?;

    {
        let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
        terminals.insert(id.clone(), session);
    }

    let reader_id = id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                    if app
                        .emit(
                            &format!("terminal:data:{reader_id}"),
                            PtyDataEvent {
                                id: &reader_id,
                                data,
                            },
                        )
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        // The shell is gone; drop the handle so a dead session can't accumulate in the
        // table for the lifetime of the app.
        if let Ok(mut terminals) = app.state::<AppState>().terminals.lock() {
            terminals.remove(&reader_id);
        }
        let _ = app.emit(
            &format!("terminal:exit:{reader_id}"),
            PtyExitEvent { id: &reader_id },
        );
    });

    Ok(())
}

#[tauri::command]
pub fn terminal_write(state: State<AppState>, id: String, data: String) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let session = terminals.get_mut(&id).ok_or("no such terminal session")?;
    session.write(data.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_resize(
    state: State<AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let session = terminals.get(&id).ok_or("no such terminal session")?;
    session.resize(cols, rows).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_kill(state: State<AppState>, id: String) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    if let Some(mut session) = terminals.remove(&id) {
        session.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::workspace::WorkspaceState;
    use anycode_tools::ToolRegistry;
    use std::collections::HashMap;
    use std::sync::{mpsc, Mutex};
    use std::time::{Duration, Instant};
    use tauri::Listener;

    /// The Terminal panel's whole backend path, driven the way the panel drives it: the
    /// caller picks the id and subscribes first, then spawns, types, and kills. Covers the
    /// four defects fixed on 2026-09-23 except the one that lives in React (unmounting).
    #[test]
    fn a_terminal_session_streams_from_the_first_byte_runs_input_and_cleans_up() {
        let workspace = std::env::temp_dir().join(format!("anycode-term-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&workspace).unwrap();
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(AppState {
            store: Mutex::new(anycode_store::Store::open_in_memory().unwrap()),
            workspace: Mutex::new(Some(WorkspaceState {
                fs_root: anycode_fs::WorkspaceRoot::new(&workspace).unwrap(),
            })),
            terminals: Mutex::new(HashMap::new()),
            tools: ToolRegistry::standard(),
            pending_approvals: Mutex::new(HashMap::new()),
            running_tasks: Mutex::new(HashMap::new()),
            session_id: uuid::Uuid::new_v4(),
        });

        let id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::channel::<Option<String>>();
        let data_tx = tx.clone();
        app.listen(format!("terminal:data:{id}"), move |event| {
            let payload: serde_json::Value = serde_json::from_str(event.payload()).unwrap();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload["data"].as_str().unwrap())
                .unwrap();
            let _ = data_tx.send(Some(String::from_utf8_lossy(&bytes).into_owned()));
        });
        app.listen(format!("terminal:exit:{id}"), move |_| {
            let _ = tx.send(None);
        });

        terminal_spawn(app.handle().clone(), app.state(), id.clone(), 80, 24).unwrap();

        // The shell speaks first (a prompt, a banner). Receiving it at all proves the
        // listener, registered before the spawn, saw the first bytes — the blank-pane
        // race. Generous: a login shell sources the user's whole profile.
        let mut seen = String::new();
        let read_until = |seen: &mut String, needle: Option<&str>, secs: u64| -> bool {
            let deadline = Instant::now() + Duration::from_secs(secs);
            while let Ok(Some(chunk)) =
                rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                seen.push_str(&chunk);
                match needle {
                    Some(n) if seen.contains(n) => return true,
                    None => return true,
                    _ => {}
                }
            }
            false
        };
        assert!(
            read_until(&mut seen, None, 30),
            "the shell's first output never arrived"
        );

        terminal_write(
            app.state(),
            id.clone(),
            "printf 'ANY<%s>\\n' \"$TERM\"\n".into(),
        )
        .unwrap();
        assert!(
            read_until(&mut seen, Some("ANY<xterm-256color>"), 30),
            "typed input did not run with TERM set; saw: {seen:?}"
        );

        terminal_kill(app.state(), id.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let exited = loop {
            match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(None) => break true,
                Ok(Some(_)) => continue,
                Err(_) => break false,
            }
        };
        assert!(exited, "no terminal:exit after kill");
        let state = app.state::<AppState>();
        assert!(
            !state.terminals.lock().unwrap().contains_key(&id),
            "dead session left in the table"
        );
        let _ = std::fs::remove_dir_all(&workspace);
    }
}
