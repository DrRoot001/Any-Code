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
use tauri::{AppHandle, Emitter, Manager, State};

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
pub fn terminal_spawn(
    app: AppHandle,
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
pub fn terminal_resize(state: State<AppState>, id: String, cols: u16, rows: u16) -> Result<(), String> {
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
