//! Local SQLite store (PRD §75). Tables are added by the phase that produces the data
//! they hold — an empty table nobody writes to is not "done", it's a schema nobody
//! asked for yet. Tables so far: `app_settings` (Phase 0), `usage_events` (Phase 2 —
//! every model request emits telemetry, docs/ARCHITECTURE.md invariant #9), and
//! `permission_grants` (Phase 3 — standing "always allow" decisions, scoped to one
//! workspace; anycode-security decides policy, this only remembers past answers), and
//! `events` (Phase 3 — the append-only audit log of every agent task's state changes,
//! tool calls, approval decisions and results; docs/ARCHITECTURE.md invariant #10), and
//! `memories` (Phase 4 — global and workspace notes the user wrote themselves or
//! explicitly adopted, e.g. from a repository's CLAUDE.md; PRD §38. Nothing else ever
//! writes here — no inferred memory).
//!
//! Session-resume task summaries (ADR 0004, "Session resume") are not a new table: the
//! `events` log is already the source of truth for a task's history, so they are a query
//! over it (see [`Store::task_summaries`]).

use anycode_core::Event;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A memory scope/workspace combination or empty content was rejected before it
    /// ever reached the database — better than storing an inconsistent row.
    #[error("{0}")]
    InvalidMemory(String),
}

pub struct Store {
    conn: Connection,
}

const MIGRATIONS: &str = "
    CREATE TABLE IF NOT EXISTS app_settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS usage_events (
        id             TEXT PRIMARY KEY,
        timestamp      TEXT NOT NULL,
        provider       TEXT NOT NULL,
        model          TEXT NOT NULL,
        input_tokens   INTEGER,
        output_tokens  INTEGER,
        status         TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS permission_grants (
        capability     TEXT NOT NULL,
        workspace_path TEXT NOT NULL,
        granted_at     TEXT NOT NULL,
        PRIMARY KEY (capability, workspace_path)
    );
    -- Append-only: nothing in this crate updates or deletes a row. The whole event is
    -- kept as JSON so a newer build's kinds and fields survive (ADR 0002); the scope
    -- columns exist only so a task's history can be found without scanning payloads.
    CREATE TABLE IF NOT EXISTS events (
        id          TEXT PRIMARY KEY,
        timestamp   TEXT NOT NULL,
        kind        TEXT NOT NULL,
        session_id  TEXT NOT NULL,
        task_id     TEXT,
        body        TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS events_by_task ON events (task_id, timestamp);
    -- Audit S2: shell grants used to be stored for the whole tool, so one 'always allow'
    -- covered every shell command. Grants are now per command (`shell.execute:<command>`);
    -- a surviving tool-wide row would still be honoured by nothing, but it records
    -- consent the user never gave, so it is removed.
    DELETE FROM permission_grants WHERE capability = 'shell.execute';
    -- Global and workspace memories (PRD §38). `workspace` is NULL for a global memory
    -- and required for a workspace one; `Store::add_memory` enforces that, not a CHECK
    -- constraint, so the error message can explain why.
    CREATE TABLE IF NOT EXISTS memories (
        id         TEXT PRIMARY KEY,
        scope      TEXT NOT NULL,
        workspace  TEXT,
        content    TEXT NOT NULL,
        source     TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    -- task_summaries filters the append-only events log by kind = 'task.created' before
    -- checking the JSON workspace field. Without an index on kind, that scan grows with
    -- the whole lifetime of the log rather than with one task's events.
    CREATE INDEX IF NOT EXISTS events_by_kind ON events (kind, timestamp);
";

impl Store {
    /// Opens (creating if needed) the SQLite database at `path` and applies migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(MIGRATIONS)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(MIGRATIONS)?;
        Ok(Self { conn })
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other.into()),
            })
    }

    pub fn delete_setting(&self, key: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Records one completed (or failed) model request. `input_tokens`/`output_tokens`
    /// are `None` when the provider's API didn't report them — never estimated.
    #[allow(clippy::too_many_arguments)]
    pub fn record_usage_event(
        &self,
        provider: &str,
        model: &str,
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        status: UsageStatus,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO usage_events (id, timestamp, provider, model, input_tokens, output_tokens, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                Uuid::new_v4().to_string(),
                OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap(),
                provider,
                model,
                input_tokens,
                output_tokens,
                status.as_str(),
            ],
        )?;
        Ok(())
    }

    /// Most recent usage events, newest first. Feeds the Phase 8 usage dashboard;
    /// exercised today only by this crate's own round-trip test.
    pub fn list_usage_events(&self, limit: i64) -> Result<Vec<UsageRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, provider, model, input_tokens, output_tokens, status
             FROM usage_events ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(UsageRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                input_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                status: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Records a standing "always allow" decision for one capability in one workspace.
    /// Never called for a one-time approval — those aren't persisted at all.
    pub fn grant_permission(
        &self,
        capability: &str,
        workspace_path: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO permission_grants (capability, workspace_path, granted_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(capability, workspace_path) DO NOTHING",
            params![
                capability,
                workspace_path,
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ],
        )?;
        Ok(())
    }

    pub fn revoke_permission(
        &self,
        capability: &str,
        workspace_path: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM permission_grants WHERE capability = ?1 AND workspace_path = ?2",
            params![capability, workspace_path],
        )?;
        Ok(())
    }

    /// Appends one event to the audit log. There is deliberately no update or delete.
    pub fn append_event(&self, event: &Event) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO events (id, timestamp, kind, session_id, task_id, body)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.id.to_string(),
                event
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
                event.kind,
                event.scope.session_id.to_string(),
                event.scope.task_id.map(|id| id.to_string()),
                serde_json::to_string(event)?,
            ],
        )?;
        Ok(())
    }

    /// One task's events, oldest first. A row this build can't parse is skipped, not
    /// fatal — the same forward-compatibility rule the log's format exists for.
    pub fn task_events(&self, task_id: Uuid) -> Result<Vec<Event>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT body FROM events WHERE task_id = ?1 ORDER BY timestamp, rowid")?;
        let rows = stmt.query_map(params![task_id.to_string()], |row| row.get::<_, String>(0))?;
        let mut events = Vec::new();
        for body in rows {
            if let Ok(event) = serde_json::from_str::<Event>(&body?) {
                events.push(event);
            }
        }
        Ok(events)
    }

    /// A session's events, oldest first — app-level ones included, which carry no task.
    /// Unparseable rows are skipped, as in [`Store::task_events`].
    pub fn session_events(&self, session_id: Uuid) -> Result<Vec<Event>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT body FROM events WHERE session_id = ?1 ORDER BY timestamp, rowid")?;
        let rows = stmt.query_map(params![session_id.to_string()], |row| {
            row.get::<_, String>(0)
        })?;
        let mut events = Vec::new();
        for body in rows {
            if let Ok(event) = serde_json::from_str::<Event>(&body?) {
                events.push(event);
            }
        }
        Ok(events)
    }

    pub fn has_permission_grant(
        &self,
        capability: &str,
        workspace_path: &str,
    ) -> Result<bool, StoreError> {
        self.conn
            .query_row(
                "SELECT 1 FROM permission_grants WHERE capability = ?1 AND workspace_path = ?2",
                params![capability, workspace_path],
                |_| Ok(()),
            )
            .map(|_| true)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(false),
                other => Err(other.into()),
            })
    }

    /// Adds one memory. Workspace scope requires `workspace`; global scope forbids one.
    /// Empty or whitespace-only content is rejected. This and the other `*_memory`
    /// methods are the only writers of the `memories` table — nothing is ever inferred.
    pub fn add_memory(
        &self,
        scope: MemoryScope,
        workspace: Option<&str>,
        content: &str,
        source: Option<&str>,
    ) -> Result<Memory, StoreError> {
        Self::check_memory_scope(scope, workspace)?;
        let content = Self::require_content(content)?;

        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        self.conn.execute(
            "INSERT INTO memories (id, scope, workspace, content, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, scope.as_str(), workspace, content, source, now],
        )?;
        Ok(Memory {
            id,
            scope,
            workspace: workspace.map(str::to_string),
            content: content.to_string(),
            source: source.map(str::to_string),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Replaces a memory's content. Returns `false` when `id` doesn't exist.
    pub fn update_memory(&self, id: &str, content: &str) -> Result<bool, StoreError> {
        let content = Self::require_content(content)?;
        let now = now_rfc3339();
        let changed = self.conn.execute(
            "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![content, now, id],
        )?;
        Ok(changed > 0)
    }

    /// Deletes a memory. Returns `false` when `id` doesn't exist.
    pub fn delete_memory(&self, id: &str) -> Result<bool, StoreError> {
        let changed = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    /// Global memories plus, when given, that workspace's — oldest first. A `workspace`
    /// of `None` never matches a workspace-scoped row, since SQLite's `= NULL` is never
    /// true, so only globals come back.
    pub fn list_memories(&self, workspace: Option<&str>) -> Result<Vec<Memory>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scope, workspace, content, source, created_at, updated_at
             FROM memories
             WHERE scope = 'global' OR (scope = 'workspace' AND workspace = ?1)
             ORDER BY created_at, rowid",
        )?;
        let rows = stmt.query_map(params![workspace], map_memory_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Every memory (all scopes, all workspaces) in a stable, documented shape:
    /// `{ "memories": [ { id, scope, workspace, content, source, createdAt, updatedAt }, ... ] }`,
    /// oldest first. This is what PRD §38 means by memory being "exportable".
    pub fn export_memories(&self) -> Result<serde_json::Value, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scope, workspace, content, source, created_at, updated_at
             FROM memories
             ORDER BY created_at, rowid",
        )?;
        let memories = stmt
            .query_map([], map_memory_row)?
            .collect::<Result<Vec<Memory>, _>>()?;
        Ok(serde_json::json!({ "memories": memories }))
    }

    fn check_memory_scope(scope: MemoryScope, workspace: Option<&str>) -> Result<(), StoreError> {
        match (scope, workspace) {
            (MemoryScope::Workspace, None) => Err(StoreError::InvalidMemory(
                "workspace-scoped memory requires a workspace path".into(),
            )),
            (MemoryScope::Workspace, Some(w)) if w.trim().is_empty() => Err(
                StoreError::InvalidMemory("workspace path must not be empty".into()),
            ),
            (MemoryScope::Global, Some(_)) => Err(StoreError::InvalidMemory(
                "global memory must not have a workspace path".into(),
            )),
            _ => Ok(()),
        }
    }

    fn require_content(content: &str) -> Result<&str, StoreError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(StoreError::InvalidMemory(
                "memory content must not be empty".into(),
            ));
        }
        Ok(trimmed)
    }

    /// Tasks started in `workspace`, newest first. Reads the append-only `events` log
    /// rather than a table of its own (ADR 0004, "Session resume"): `task.created`
    /// carries the instruction/provider/model/workspace, and `outcome` is the state of
    /// the last `task.state` event that reached a terminal state for that task — `None`
    /// when it never did (the app closed or crashed mid-task; never guessed).
    pub fn task_summaries(
        &self,
        workspace: &str,
        limit: usize,
    ) -> Result<Vec<TaskSummary>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                 tc.task_id,
                 tc.session_id,
                 json_extract(tc.body, '$.payload.instruction') AS instruction,
                 json_extract(tc.body, '$.payload.provider')    AS provider,
                 json_extract(tc.body, '$.payload.model')       AS model,
                 json_extract(tc.body, '$.payload.workspace')   AS workspace,
                 tc.timestamp AS started_at,
                 (
                     SELECT json_extract(s.body, '$.payload.state')
                     FROM events s
                     WHERE s.task_id = tc.task_id
                       AND s.kind = 'task.state'
                       AND json_extract(s.body, '$.payload.state')
                           IN ('completed', 'failed', 'cancelled')
                     ORDER BY s.timestamp DESC, s.rowid DESC
                     LIMIT 1
                 ) AS outcome
             FROM events tc
             WHERE tc.kind = 'task.created'
               AND json_extract(tc.body, '$.payload.workspace') = ?1
             ORDER BY tc.timestamp DESC, tc.rowid DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![workspace, limit as i64], |row| {
            Ok(TaskSummary {
                task_id: row.get(0)?,
                session_id: row.get(1)?,
                instruction: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                workspace: row.get(5)?,
                started_at: row.get(6)?,
                outcome: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

fn map_memory_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    let scope: String = row.get(1)?;
    Ok(Memory {
        id: row.get(0)?,
        scope: if scope == "workspace" {
            MemoryScope::Workspace
        } else {
            MemoryScope::Global
        },
        workspace: row.get(2)?,
        content: row.get(3)?,
        source: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageStatus {
    Success,
    Error,
    /// Abandoned before the provider finished; tokens were likely consumed but unreported.
    Cancelled,
}

impl UsageStatus {
    fn as_str(self) -> &'static str {
        match self {
            UsageStatus::Success => "success",
            UsageStatus::Error => "error",
            UsageStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    pub id: String,
    pub timestamp: String,
    pub provider: String,
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Global,
    Workspace,
}

impl MemoryScope {
    fn as_str(self) -> &'static str {
        match self {
            MemoryScope::Global => "global",
            MemoryScope::Workspace => "workspace",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub scope: MemoryScope,
    /// Some for `Workspace` scope (the workspace path), None for `Global`.
    pub workspace: Option<String>,
    pub content: String,
    /// Where it came from when not typed by the user directly, e.g. "adopted:CLAUDE.md".
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummary {
    pub task_id: String,
    pub session_id: String,
    pub instruction: String,
    pub provider: String,
    pub model: String,
    pub workspace: String,
    pub started_at: String,
    /// The last terminal state recorded ("completed" | "failed" | "cancelled"), or None
    /// when the task never reached one — the app closed or crashed mid-task. Never
    /// guessed.
    pub outcome: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anycode_core::EventScope;

    #[test]
    fn a_tasks_events_come_back_in_order_and_only_for_that_task() {
        let store = Store::open_in_memory().unwrap();
        let session_id = Uuid::new_v4();
        let task = Uuid::new_v4();
        let other = Uuid::new_v4();
        let scope = |task_id| EventScope {
            session_id,
            task_id: Some(task_id),
            ..Default::default()
        };

        for kind in ["task.state", "task.tool.call", "task.tool.result"] {
            store
                .append_event(&Event::new(
                    kind,
                    scope(task),
                    serde_json::json!({ "k": kind }),
                ))
                .unwrap();
        }
        store
            .append_event(&Event::new(
                "task.state",
                scope(other),
                serde_json::Value::Null,
            ))
            .unwrap();

        let events = store.task_events(task).unwrap();
        let kinds: Vec<_> = events.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, ["task.state", "task.tool.call", "task.tool.result"]);
        assert_eq!(events[1].payload["k"], "task.tool.call");
    }

    #[test]
    fn tool_wide_shell_grants_do_not_survive_reopening() {
        let path = std::env::temp_dir().join(format!("anycode-store-{}.db", Uuid::new_v4()));
        {
            let store = Store::open(&path).unwrap();
            store.grant_permission("shell.execute", "/ws").unwrap();
            store
                .grant_permission("shell.execute:npm test", "/ws")
                .unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert!(!store.has_permission_grant("shell.execute", "/ws").unwrap());
        assert!(store
            .has_permission_grant("shell.execute:npm test", "/ws")
            .unwrap());
        drop(store);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }

    #[test]
    fn setting_survives_get_after_set() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.get_setting("theme").unwrap(), None);
        store.set_setting("theme", "dark").unwrap();
        assert_eq!(store.get_setting("theme").unwrap(), Some("dark".into()));
        store.set_setting("theme", "light").unwrap();
        assert_eq!(store.get_setting("theme").unwrap(), Some("light".into()));
    }

    #[test]
    fn usage_events_round_trip_newest_first() {
        let store = Store::open_in_memory().unwrap();
        store
            .record_usage_event("openai", "gpt-5", Some(10), Some(4), UsageStatus::Success)
            .unwrap();
        store
            .record_usage_event("anthropic", "claude-opus-5", None, None, UsageStatus::Error)
            .unwrap();

        let events = store.list_usage_events(10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].provider, "anthropic");
        assert_eq!(events[0].status, "error");
        assert_eq!(events[0].input_tokens, None);
        assert_eq!(events[1].provider, "openai");
        assert_eq!(events[1].input_tokens, Some(10));
    }

    #[test]
    fn permission_grants_are_scoped_per_workspace() {
        let store = Store::open_in_memory().unwrap();
        assert!(!store.has_permission_grant("git.push", "/repo-a").unwrap());

        store.grant_permission("git.push", "/repo-a").unwrap();
        assert!(store.has_permission_grant("git.push", "/repo-a").unwrap());
        assert!(!store.has_permission_grant("git.push", "/repo-b").unwrap());

        store.revoke_permission("git.push", "/repo-a").unwrap();
        assert!(!store.has_permission_grant("git.push", "/repo-a").unwrap());
    }

    #[test]
    fn granting_twice_does_not_error() {
        let store = Store::open_in_memory().unwrap();
        store.grant_permission("git.push", "/repo-a").unwrap();
        store.grant_permission("git.push", "/repo-a").unwrap();
        assert!(store.has_permission_grant("git.push", "/repo-a").unwrap());
    }

    #[test]
    fn memory_add_list_update_delete_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let memory = store
            .add_memory(MemoryScope::Global, None, "prefers tabs", None)
            .unwrap();
        assert_eq!(memory.scope, MemoryScope::Global);
        assert_eq!(memory.workspace, None);
        assert_eq!(memory.content, "prefers tabs");

        assert_eq!(store.list_memories(None).unwrap(), vec![memory.clone()]);

        let updated = store.update_memory(&memory.id, "prefers spaces").unwrap();
        assert!(updated);
        let listed = store.list_memories(None).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].content, "prefers spaces");
        assert!(listed[0].updated_at >= memory.updated_at);

        assert!(!store.update_memory("no-such-id", "x").unwrap());
        assert!(!store.delete_memory("no-such-id").unwrap());

        assert!(store.delete_memory(&memory.id).unwrap());
        assert_eq!(store.list_memories(None).unwrap(), vec![]);
    }

    #[test]
    fn memory_scope_rules_are_enforced() {
        let store = Store::open_in_memory().unwrap();

        let err = store
            .add_memory(MemoryScope::Workspace, None, "x", None)
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidMemory(_)));

        let err = store
            .add_memory(MemoryScope::Global, Some("/ws"), "x", None)
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidMemory(_)));

        // Valid combinations still work.
        store
            .add_memory(MemoryScope::Workspace, Some("/ws"), "x", None)
            .unwrap();
        store
            .add_memory(MemoryScope::Global, None, "y", None)
            .unwrap();
    }

    #[test]
    fn memory_rejects_empty_or_whitespace_content() {
        let store = Store::open_in_memory().unwrap();
        assert!(store
            .add_memory(MemoryScope::Global, None, "   ", None)
            .is_err());
        assert!(store
            .add_memory(MemoryScope::Global, None, "", None)
            .is_err());

        let memory = store
            .add_memory(MemoryScope::Global, None, "real content", None)
            .unwrap();
        assert!(store.update_memory(&memory.id, "  \n\t").is_err());
    }

    #[test]
    fn memory_list_returns_global_plus_only_the_given_workspaces() {
        let store = Store::open_in_memory().unwrap();
        store
            .add_memory(MemoryScope::Global, None, "global note", None)
            .unwrap();
        store
            .add_memory(MemoryScope::Workspace, Some("/ws-a"), "ws-a note", None)
            .unwrap();
        store
            .add_memory(MemoryScope::Workspace, Some("/ws-b"), "ws-b note", None)
            .unwrap();

        let none = store.list_memories(None).unwrap();
        assert_eq!(none.len(), 1);
        assert_eq!(none[0].content, "global note");

        let ws_a = store.list_memories(Some("/ws-a")).unwrap();
        let contents: Vec<_> = ws_a.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(contents, ["global note", "ws-a note"]);

        let ws_b = store.list_memories(Some("/ws-b")).unwrap();
        let contents: Vec<_> = ws_b.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(contents, ["global note", "ws-b note"]);
    }

    #[test]
    fn memory_export_includes_everything() {
        let store = Store::open_in_memory().unwrap();
        store
            .add_memory(MemoryScope::Global, None, "global note", Some("typed"))
            .unwrap();
        store
            .add_memory(
                MemoryScope::Workspace,
                Some("/ws-a"),
                "adopted rule",
                Some("adopted:CLAUDE.md"),
            )
            .unwrap();

        let exported = store.export_memories().unwrap();
        let memories = exported["memories"].as_array().unwrap();
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0]["content"], "global note");
        assert_eq!(memories[0]["scope"], "global");
        assert_eq!(memories[1]["workspace"], "/ws-a");
        assert_eq!(memories[1]["source"], "adopted:CLAUDE.md");
    }

    #[test]
    fn opening_an_old_database_file_adds_the_memories_table() {
        let path = std::env::temp_dir().join(format!("anycode-store-old-{}.db", Uuid::new_v4()));
        {
            // Simulate a database file created before this change: no `memories` table
            // and none of its supporting index.
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS events (
                     id TEXT PRIMARY KEY, timestamp TEXT NOT NULL, kind TEXT NOT NULL,
                     session_id TEXT NOT NULL, task_id TEXT, body TEXT NOT NULL
                 );",
            )
            .unwrap();
        }

        // Store::open runs migrations, which must add `memories` to this pre-existing file.
        let store = Store::open(&path).unwrap();
        let memory = store
            .add_memory(MemoryScope::Global, None, "survives migration", None)
            .unwrap();
        assert_eq!(store.list_memories(None).unwrap(), vec![memory]);

        drop(store);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }

    fn task_created(session_id: Uuid, task_id: Uuid, workspace: &str, instruction: &str) -> Event {
        Event::new(
            "task.created",
            EventScope {
                session_id,
                task_id: Some(task_id),
                ..Default::default()
            },
            serde_json::json!({
                "instruction": instruction,
                "provider": "anthropic",
                "model": "claude-sonnet-5",
                "workspace": workspace,
            }),
        )
    }

    fn task_state(session_id: Uuid, task_id: Uuid, state: &str) -> Event {
        Event::new(
            "task.state",
            EventScope {
                session_id,
                task_id: Some(task_id),
                ..Default::default()
            },
            serde_json::json!({ "state": state }),
        )
    }

    #[test]
    fn task_summaries_separate_workspaces_pick_the_last_terminal_state_and_sort_newest_first() {
        let store = Store::open_in_memory().unwrap();
        let session = Uuid::new_v4();

        // ws-a: reaches a terminal state (through a non-terminal one first).
        let task_a = Uuid::new_v4();
        store
            .append_event(&task_created(session, task_a, "/ws-a", "do a"))
            .unwrap();
        store
            .append_event(&task_state(session, task_a, "planning"))
            .unwrap();
        store
            .append_event(&task_state(session, task_a, "completed"))
            .unwrap();

        // ws-a: never reaches a terminal state (app closed mid-task).
        let task_b = Uuid::new_v4();
        store
            .append_event(&task_created(session, task_b, "/ws-a", "do b"))
            .unwrap();
        store
            .append_event(&task_state(session, task_b, "running"))
            .unwrap();

        // ws-b: a different workspace, must not leak into ws-a's list.
        let task_c = Uuid::new_v4();
        store
            .append_event(&task_created(session, task_c, "/ws-b", "do c"))
            .unwrap();
        store
            .append_event(&task_state(session, task_c, "failed"))
            .unwrap();

        let ws_a = store.task_summaries("/ws-a", 10).unwrap();
        assert_eq!(ws_a.len(), 2);
        // task_b was created after task_a, so it sorts first (newest first).
        assert_eq!(ws_a[0].task_id, task_b.to_string());
        assert_eq!(ws_a[0].instruction, "do b");
        assert_eq!(ws_a[0].outcome, None);
        assert_eq!(ws_a[1].task_id, task_a.to_string());
        assert_eq!(ws_a[1].outcome, Some("completed".to_string()));
        assert_eq!(ws_a[1].provider, "anthropic");
        assert_eq!(ws_a[1].model, "claude-sonnet-5");
        assert_eq!(ws_a[1].session_id, session.to_string());

        let ws_b = store.task_summaries("/ws-b", 10).unwrap();
        assert_eq!(ws_b.len(), 1);
        assert_eq!(ws_b[0].task_id, task_c.to_string());
        assert_eq!(ws_b[0].outcome, Some("failed".to_string()));

        assert_eq!(
            store.task_summaries("/ws-does-not-exist", 10).unwrap(),
            vec![]
        );
    }

    #[test]
    fn task_summaries_respects_the_limit() {
        let store = Store::open_in_memory().unwrap();
        let session = Uuid::new_v4();
        for i in 0..3 {
            store
                .append_event(&task_created(
                    session,
                    Uuid::new_v4(),
                    "/ws",
                    &format!("task {i}"),
                ))
                .unwrap();
        }
        assert_eq!(store.task_summaries("/ws", 2).unwrap().len(), 2);
        assert_eq!(store.task_summaries("/ws", 10).unwrap().len(), 3);
    }
}
