//! Local SQLite store (PRD §75). Tables are added by the phase that produces the data
//! they hold — an empty table nobody writes to is not "done", it's a schema nobody
//! asked for yet. Tables so far: `app_settings` (Phase 0), `usage_events` (Phase 2 —
//! every model request emits telemetry, docs/ARCHITECTURE.md invariant #9), and
//! `permission_grants` (Phase 3 — standing "always allow" decisions, scoped to one
//! workspace; anycode-security decides policy, this only remembers past answers), and
//! `events` (Phase 3 — the append-only audit log of every agent task's state changes,
//! tool calls, approval decisions and results; docs/ARCHITECTURE.md invariant #10).

use anycode_core::Event;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
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
}
