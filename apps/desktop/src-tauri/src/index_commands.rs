//! The open workspace's code index (ADR 0004 "Freshness"): built off the UI thread when a
//! workspace opens, then kept current by a debounced file watcher that re-indexes only the
//! paths that changed. Status is pushed to the frontend as `index:status` and can be read
//! with `index_status`; it is only ever what actually happened, never an estimate.

use crate::AppState;
use anycode_code_intelligence::Index;
use notify_debouncer_mini::{
    new_debouncer_opt, notify, notify::RecursiveMode, DebounceEventResult,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

/// Long enough to coalesce a save or a `git checkout`, short enough for PRD §70's ≤ 1 s.
const DEBOUNCE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum IndexStatus {
    None,
    Building,
    Ready {
        files: usize,
        chunks: usize,
        symbols: usize,
        #[serde(rename = "lastRefreshMs")]
        last_refresh_ms: u128,
    },
    Failed {
        error: String,
    },
}

type Watcher = notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>;

pub(crate) struct IndexSlot {
    /// Bumped by every `start`. A build or watcher event from an older generation is
    /// discarded, never installed — even when the user has since re-opened the same folder.
    generation: u64,
    root: Option<PathBuf>,
    status: IndexStatus,
    index: Option<Arc<Mutex<Index>>>,
    _watcher: Option<Watcher>,
}

impl Default for IndexSlot {
    fn default() -> Self {
        Self {
            generation: 0,
            root: None,
            status: IndexStatus::None,
            index: None,
            _watcher: None,
        }
    }
}

/// The index for `root` if it is built. `None` while building or after a failure — the
/// agent's tools and context builder say so rather than guessing.
pub(crate) fn handle(state: &AppState, root: &Path) -> Option<Arc<Mutex<Index>>> {
    let slot = state.index.lock().ok()?;
    (slot.root.as_deref() == Some(root))
        .then(|| slot.index.clone())
        .flatten()
}

fn ready_status(index: &Index, last_refresh_ms: u128) -> IndexStatus {
    match index.stats() {
        Ok(stats) => IndexStatus::Ready {
            files: stats.files,
            chunks: stats.chunks,
            symbols: stats.symbols,
            last_refresh_ms,
        },
        Err(e) => IndexStatus::Failed {
            error: e.to_string(),
        },
    }
}

/// Records `status` if `generation` is still current, and tells the frontend.
fn publish<R: Runtime>(app: &AppHandle<R>, generation: u64, status: IndexStatus) {
    let state = app.state::<AppState>();
    let Ok(mut slot) = state.index.lock() else {
        return;
    };
    if slot.generation != generation {
        return;
    }
    slot.status = status.clone();
    drop(slot);
    let _ = app.emit("index:status", status);
}

/// Starts indexing `root`, replacing whatever index was open. Returns at once.
pub(crate) fn start<R: Runtime>(app: &AppHandle<R>, root: PathBuf) {
    let state = app.state::<AppState>();
    let generation;
    if let Ok(mut slot) = state.index.lock() {
        let current = matches!(
            slot.status,
            IndexStatus::Building | IndexStatus::Ready { .. }
        );
        if slot.root.as_deref() == Some(root.as_path()) && current {
            return; // Re-opening the same workspace: the watcher already keeps it current.
        }
        generation = slot.generation + 1;
        *slot = IndexSlot {
            generation,
            root: Some(root.clone()),
            status: IndexStatus::Building,
            ..IndexSlot::default()
        };
    } else {
        return;
    }
    let _ = app.emit("index:status", IndexStatus::Building);

    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(error) = build(&app, &root, generation) {
            publish(&app, generation, IndexStatus::Failed { error });
        }
    });
}

fn build<R: Runtime>(app: &AppHandle<R>, root: &Path, generation: u64) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = anycode_code_intelligence::index_db_path(&data_dir, root);
    let mut index = Index::open(&db_path, root).map_err(|e| e.to_string())?;
    let report = index.refresh().map_err(|e| e.to_string())?;
    let status = ready_status(&index, report.elapsed_ms);
    let index = Arc::new(Mutex::new(index));

    let watcher = watch(app, root, generation, index.clone())?;
    let state = app.state::<AppState>();
    let mut slot = state.index.lock().map_err(|e| e.to_string())?;
    if slot.generation != generation {
        return Ok(()); // The user opened another workspace meanwhile.
    }
    slot.index = Some(index);
    slot._watcher = Some(watcher);
    slot.status = status.clone();
    drop(slot);
    let _ = app.emit("index:status", status);
    Ok(())
}

fn watch<R: Runtime>(
    app: &AppHandle<R>,
    root: &Path,
    generation: u64,
    index: Arc<Mutex<Index>>,
) -> Result<Watcher, String> {
    let app = app.clone();
    // Symlinks are never followed: the index never reads through one, and following a
    // link such as `x -> /` would spend the user's inotify watches on the whole disk.
    let config = notify_debouncer_mini::Config::default()
        .with_timeout(DEBOUNCE)
        .with_notify_config(notify::Config::default().with_follow_symlinks(false));
    let mut debouncer = new_debouncer_opt::<_, notify::RecommendedWatcher>(
        config,
        move |result: DebounceEventResult| {
            let Ok(events) = result else { return };
            let paths: Vec<PathBuf> = events.into_iter().map(|e| e.path).collect();
            // ponytail: ignored paths (target/, .git/) are filtered inside update_paths, so a
            // build's burst of events still takes the lock once per batch; pre-filter if it shows.
            if paths.is_empty() {
                return;
            }
            let status = {
                let Ok(mut index) = index.lock() else { return };
                match index.update_paths(&paths) {
                    Ok(report) if report.reindexed == 0 && report.removed == 0 => return,
                    Ok(report) => ready_status(&index, report.elapsed_ms),
                    Err(e) => IndexStatus::Failed {
                        error: e.to_string(),
                    },
                }
            };
            publish(&app, generation, status);
        },
    )
    .map_err(|e| format!("cannot watch the workspace: {e}"))?;
    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .map_err(|e| format!("cannot watch the workspace: {e}"))?;
    Ok(debouncer)
}

#[tauri::command]
pub fn index_status(state: State<AppState>) -> Result<IndexStatus, String> {
    let slot = state.index.lock().map_err(|e| e.to_string())?;
    Ok(slot.status.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real watcher, end to end: a file written after indexing is picked up within the
    /// debounce. Uses an in-memory index so nothing is written to the app's data directory.
    #[test]
    fn the_watcher_indexes_a_new_file() {
        let dir = std::env::temp_dir().join(format!("anycode-watch-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let root = dir.canonicalize().unwrap();
        std::fs::write(root.join("a.py"), "def a():\n    pass\n").unwrap();
        let mut index = Index::open_in_memory(&root).unwrap();
        index.refresh().unwrap();
        let index = Arc::new(Mutex::new(index));

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(crate::AppState::for_tests());
        let _watcher = watch(app.handle(), &root, 1, index.clone()).unwrap();
        // FSEvents may deliver the watch's own start-up late; give it a moment first.
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(root.join("b.py"), "def b():\n    pass\n").unwrap();

        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let found = loop {
            let files = index.lock().unwrap().files().unwrap();
            if files.iter().any(|f| f.path == "b.py") {
                break true;
            }
            if std::time::Instant::now() > deadline {
                break false;
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        let _ = std::fs::remove_dir_all(&dir);
        assert!(found, "b.py was never indexed by the watcher");
    }

    #[test]
    fn status_serialises_to_the_frontend_contract() {
        // apps/desktop/src/lib/tauri.ts `IndexStatus`.
        let ready = IndexStatus::Ready {
            files: 3,
            chunks: 7,
            symbols: 11,
            last_refresh_ms: 42,
        };
        assert_eq!(
            serde_json::to_value(ready).unwrap(),
            serde_json::json!({ "state": "ready", "files": 3, "chunks": 7, "symbols": 11, "lastRefreshMs": 42 })
        );
        assert_eq!(
            serde_json::to_value(IndexStatus::Building).unwrap(),
            serde_json::json!({ "state": "building" })
        );
        assert_eq!(
            serde_json::to_value(IndexStatus::Failed { error: "x".into() }).unwrap(),
            serde_json::json!({ "state": "failed", "error": "x" })
        );
    }
}
