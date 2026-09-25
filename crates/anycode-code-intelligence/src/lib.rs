//! Any Code's repository index (ADR 0004, Phase 4): a gitignore-aware walk,
//! SQLite storage with an FTS5 full-text index, tree-sitter symbol
//! extraction, and ripgrep-backed live search that needs no index at all.
//!
//! The database is a derived, rebuildable cache (docs/ARCHITECTURE.md
//! invariant #8) — never written into the repository it indexes, safe to
//! delete, never synced. This crate does no networking and knows nothing
//! about providers, agents or permissions; it only answers "what's in this
//! repository".

mod fts;
mod lsp;
mod path_util;
mod search;
mod symbols;
mod walk;

pub use lsp::{server_for, Location as LspLocation, LspClient, LspError, ServerCommand};
pub use search::{references, search_live};

use rusqlite::{params, Connection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("search error: {0}")]
    Search(String),
}

/// A source language, detected from a file's extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Other,
}

impl Language {
    fn for_path(rel: &str) -> Language {
        let ext = Path::new(rel)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match ext.as_str() {
            "rs" => Language::Rust,
            "ts" | "mts" | "cts" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "py" | "pyi" => Language::Python,
            _ => Language::Other,
        }
    }

    fn as_db_str(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Tsx => "tsx",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Other => "other",
        }
    }

    fn from_db_str(s: &str) -> Language {
        match s {
            "rust" => Language::Rust,
            "typescript" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "javascript" => Language::JavaScript,
            "python" => Language::Python,
            _ => Language::Other,
        }
    }
}

/// The kind of a definition tree-sitter's tags query found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    Class,
    Type,
    Const,
    Module,
}

impl SymbolKind {
    fn as_db_str(self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
            SymbolKind::Class => "class",
            SymbolKind::Type => "type",
            SymbolKind::Const => "const",
            SymbolKind::Module => "module",
        }
    }

    fn from_db_str(s: &str) -> Option<SymbolKind> {
        Some(match s {
            "function" => SymbolKind::Function,
            "method" => SymbolKind::Method,
            "struct" => SymbolKind::Struct,
            "enum" => SymbolKind::Enum,
            "trait" => SymbolKind::Trait,
            "interface" => SymbolKind::Interface,
            "class" => SymbolKind::Class,
            "type" => SymbolKind::Type,
            "const" => SymbolKind::Const,
            "module" => SymbolKind::Module,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub language: Language,
    pub lines: u32,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub files: usize,
    pub chunks: usize,
    pub symbols: usize,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshReport {
    pub scanned: usize,
    pub reindexed: usize,
    pub removed: usize,
    pub skipped: usize,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub container: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub path: String,
    pub target: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineMatch {
    pub path: String,
    pub line: u32,
    pub text: String,
}

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS files (
        path        TEXT PRIMARY KEY,
        language    TEXT NOT NULL,
        size        INTEGER NOT NULL,
        mtime_nanos INTEGER NOT NULL,
        hash        TEXT NOT NULL,
        lines       INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS chunks (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        path       TEXT NOT NULL,
        start_line INTEGER NOT NULL,
        end_line   INTEGER NOT NULL,
        text       TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS chunks_by_path ON chunks (path);

    -- External-content FTS5 table: the indexed text lives once, in `chunks`,
    -- instead of being duplicated inside FTS5's own shadow tables. We never
    -- UPDATE a chunk row (a re-index always deletes and re-inserts a file's
    -- chunks), so only insert/delete triggers are needed to keep the index
    -- in sync — the pattern SQLite's own documentation recommends for
    -- external-content tables.
    CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(text, content='chunks', content_rowid='id');
    CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
        INSERT INTO chunks_fts(rowid, text) VALUES (new.id, new.text);
    END;
    CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
        INSERT INTO chunks_fts(chunks_fts, rowid, text) VALUES ('delete', old.id, old.text);
    END;

    CREATE TABLE IF NOT EXISTS symbols (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        path       TEXT NOT NULL,
        name       TEXT NOT NULL,
        kind       TEXT NOT NULL,
        start_line INTEGER NOT NULL,
        end_line   INTEGER NOT NULL,
        container  TEXT
    );
    CREATE INDEX IF NOT EXISTS symbols_by_name ON symbols (name);
    CREATE INDEX IF NOT EXISTS symbols_by_path ON symbols (path);

    CREATE TABLE IF NOT EXISTS imports (
        id     INTEGER PRIMARY KEY AUTOINCREMENT,
        path   TEXT NOT NULL,
        target TEXT NOT NULL,
        line   INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS imports_by_path ON imports (path);
";

pub struct Index {
    conn: Connection,
    root: PathBuf,
}

enum Outcome {
    Reindexed,
    Skipped,
    Unchanged,
}

fn canonical_root(workspace_root: &Path) -> PathBuf {
    workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf())
}

fn delete_file_rows(conn: &Connection, rel: &str) -> Result<(), IndexError> {
    conn.execute("DELETE FROM chunks WHERE path = ?1", params![rel])?;
    conn.execute("DELETE FROM symbols WHERE path = ?1", params![rel])?;
    conn.execute("DELETE FROM imports WHERE path = ?1", params![rel])?;
    conn.execute("DELETE FROM files WHERE path = ?1", params![rel])?;
    Ok(())
}

/// Deletes `rel` and, if it was a directory, everything indexed under it. Returns how
/// many files were removed.
fn delete_tree_rows(conn: &Connection, rel: &str) -> Result<usize, IndexError> {
    let prefix = format!("{rel}/");
    let under = "path = ?1 OR substr(path, 1, length(?2)) = ?2";
    for table in ["chunks", "symbols", "imports"] {
        conn.execute(
            &format!("DELETE FROM {table} WHERE {under}"),
            params![rel, prefix],
        )?;
    }
    Ok(conn.execute(
        &format!("DELETE FROM files WHERE {under}"),
        params![rel, prefix],
    )?)
}

fn fetch_file_meta(conn: &Connection, rel: &str) -> Result<Option<(i64, i64, String)>, IndexError> {
    let mut stmt = conn.prepare("SELECT size, mtime_nanos, hash FROM files WHERE path = ?1")?;
    let mut rows = stmt.query(params![rel])?;
    match rows.next()? {
        Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?))),
        None => Ok(None),
    }
}

fn touch_file_meta(
    conn: &Connection,
    rel: &str,
    size: u64,
    mtime_nanos: i64,
) -> Result<(), IndexError> {
    conn.execute(
        "UPDATE files SET size = ?2, mtime_nanos = ?3 WHERE path = ?1",
        params![rel, size as i64, mtime_nanos],
    )?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Splits `text` into 50-line chunks: `(start_line, end_line, text)`, both
/// 1-indexed and inclusive.
fn chunk_lines(text: &str) -> Vec<(u32, u32, String)> {
    const CHUNK_LINES: usize = 50;
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let end = (i + CHUNK_LINES).min(lines.len());
        out.push(((i + 1) as u32, end as u32, lines[i..end].join("\n")));
        i = end;
    }
    out
}

/// Decides whether `rel` needs (re-)reading and, if its content changed,
/// (re-)indexing — the freshness rule from ADR 0004: size and mtime decide
/// whether to re-read, a content hash decides whether to re-index.
fn index_one(
    conn: &Connection,
    rel: &str,
    fs_path: &Path,
    size: u64,
    mtime_nanos: i64,
) -> Result<Outcome, IndexError> {
    if size > walk::MAX_FILE_BYTES {
        delete_file_rows(conn, rel)?;
        return Ok(Outcome::Skipped);
    }
    let existing = fetch_file_meta(conn, rel)?;
    if let Some((esize, emtime, _)) = existing {
        if esize == size as i64 && emtime == mtime_nanos {
            return Ok(Outcome::Unchanged);
        }
    }

    // One unreadable file (permissions, vanished mid-scan) is skipped, never allowed to
    // fail the batch: a failed batch rolls back every other change in it, including the
    // removal of a file that just became a link to a secret.
    let Ok(bytes) = std::fs::read(fs_path) else {
        delete_file_rows(conn, rel)?;
        return Ok(Outcome::Skipped);
    };
    if walk::is_probably_binary(&bytes) {
        delete_file_rows(conn, rel)?;
        return Ok(Outcome::Skipped);
    }
    let Ok(text) = std::str::from_utf8(&bytes) else {
        delete_file_rows(conn, rel)?;
        return Ok(Outcome::Skipped);
    };

    let hash = sha256_hex(&bytes);
    if let Some((_, _, ehash)) = &existing {
        if *ehash == hash {
            touch_file_meta(conn, rel, size, mtime_nanos)?;
            return Ok(Outcome::Unchanged);
        }
    }

    delete_file_rows(conn, rel)?;
    let language = Language::for_path(rel);
    let lines = text.lines().count() as u32;
    conn.execute(
        "INSERT INTO files (path, language, size, mtime_nanos, hash, lines) VALUES (?1,?2,?3,?4,?5,?6)",
        params![rel, language.as_db_str(), size as i64, mtime_nanos, hash, lines],
    )?;
    for (start_line, end_line, chunk_text) in chunk_lines(text) {
        conn.execute(
            "INSERT INTO chunks (path, start_line, end_line, text) VALUES (?1,?2,?3,?4)",
            params![rel, start_line, end_line, chunk_text],
        )?;
    }
    for symbol in symbols::extract_symbols(language, text, rel) {
        conn.execute(
            "INSERT INTO symbols (path, name, kind, start_line, end_line, container) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                symbol.path,
                symbol.name,
                symbol.kind.as_db_str(),
                symbol.start_line,
                symbol.end_line,
                symbol.container
            ],
        )?;
    }
    for import in symbols::extract_imports(language, text, rel) {
        conn.execute(
            "INSERT INTO imports (path, target, line) VALUES (?1,?2,?3)",
            params![import.path, import.target, import.line],
        )?;
    }
    Ok(Outcome::Reindexed)
}

fn row_to_symbol(row: &rusqlite::Row) -> rusqlite::Result<Symbol> {
    let kind_str: String = row.get(1)?;
    Ok(Symbol {
        name: row.get(0)?,
        kind: SymbolKind::from_db_str(&kind_str).unwrap_or(SymbolKind::Function),
        path: row.get(2)?,
        start_line: row.get::<_, i64>(3)? as u32,
        end_line: row.get::<_, i64>(4)? as u32,
        container: row.get(5)?,
    })
}

/// Bumped whenever extraction changes what a file's rows are. 2: one symbol per Rust
/// method (the tags query tags each twice).
const INDEX_FORMAT: u32 = 2;

impl Index {
    /// Opens (creating if needed) the index database at `db_path` and
    /// applies its schema. `workspace_root` is canonicalised once here and
    /// used for every relative-path computation.
    pub fn open(db_path: &Path, workspace_root: &Path) -> Result<Self, IndexError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        // Two builds of one workspace can overlap briefly (re-opening a folder while its
        // first build runs); the second waits for the writer instead of failing at once.
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(SCHEMA)?;
        // An index written by an older extractor is emptied, so the next refresh rebuilds
        // it rather than keeping rows the current code would not produce.
        let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < INDEX_FORMAT {
            conn.execute_batch(
                "DELETE FROM chunks; DELETE FROM symbols; DELETE FROM imports; DELETE FROM files;",
            )?;
            conn.pragma_update(None, "user_version", INDEX_FORMAT)?;
        }
        Ok(Self {
            conn,
            root: canonical_root(workspace_root),
        })
    }

    /// An in-memory index, for tests and for "search without persisting".
    pub fn open_in_memory(workspace_root: &Path) -> Result<Self, IndexError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            root: canonical_root(workspace_root),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Full scan of the workspace: every eligible file is checked, missing
    /// files are removed from the index.
    pub fn refresh(&mut self) -> Result<RefreshReport, IndexError> {
        let start = Instant::now();
        let root = self.root.clone();
        let entries = walk::walk(&root, &root);

        let tx = self.conn.transaction()?;
        let existing: HashSet<String> = {
            let mut stmt = tx.prepare("SELECT path FROM files")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<_, _>>()?
        };

        let mut scanned = 0usize;
        let mut reindexed = 0usize;
        let mut skipped = 0usize;
        let mut seen: HashSet<String> = HashSet::new();
        for entry in entries {
            scanned += 1;
            seen.insert(entry.rel.clone());
            match index_one(
                &tx,
                &entry.rel,
                &entry.fs_path,
                entry.size,
                entry.mtime_nanos,
            )? {
                Outcome::Reindexed => reindexed += 1,
                Outcome::Skipped => skipped += 1,
                Outcome::Unchanged => {}
            }
        }

        let mut removed = 0usize;
        for path in existing.difference(&seen) {
            delete_file_rows(&tx, path)?;
            removed += 1;
        }

        tx.commit()?;
        Ok(RefreshReport {
            scanned,
            reindexed,
            removed,
            skipped,
            elapsed_ms: start.elapsed().as_millis(),
        })
    }

    /// Re-indexes exactly the given paths (created, changed or deleted),
    /// absolute or workspace-relative. A missing path is a deletion; a path
    /// outside the workspace root or covered by `.gitignore`/`.ignore` is
    /// skipped; a directory is walked (for a newly created tree).
    pub fn update_paths(&mut self, paths: &[PathBuf]) -> Result<RefreshReport, IndexError> {
        let start = Instant::now();
        let root = self.root.clone();
        let tx = self.conn.transaction()?;

        let mut scanned = 0usize;
        let mut reindexed = 0usize;
        let mut removed = 0usize;
        let mut skipped = 0usize;

        for input in paths {
            scanned += 1;
            let Some(rel) = path_util::lexical_relative(&root, input) else {
                skipped += 1;
                continue;
            };
            let fs_path = path_util::to_fs_path(&root, &rel);
            let is_link = fs_path
                .symlink_metadata()
                .is_ok_and(|m| m.file_type().is_symlink());

            // Gone, a symlink (the walk never follows one — it may point out of the
            // workspace), or excluded by the same rules as the full scan: either way, not
            // indexed under this name any more.
            if !fs_path.exists() || is_link || walk::is_excluded(&root, &fs_path) {
                // A directory that became a link takes everything indexed under it along.
                match delete_tree_rows(&tx, &rel)? {
                    0 => skipped += 1,
                    n => removed += n,
                }
                continue;
            }
            if fs_path.is_dir() {
                for entry in walk::walk(&root, &fs_path) {
                    scanned += 1;
                    match index_one(
                        &tx,
                        &entry.rel,
                        &entry.fs_path,
                        entry.size,
                        entry.mtime_nanos,
                    )? {
                        Outcome::Reindexed => reindexed += 1,
                        Outcome::Skipped => skipped += 1,
                        Outcome::Unchanged => {}
                    }
                }
                continue;
            }
            let Ok(metadata) = std::fs::metadata(&fs_path) else {
                delete_file_rows(&tx, &rel)?;
                skipped += 1;
                continue;
            };
            match index_one(
                &tx,
                &rel,
                &fs_path,
                metadata.len(),
                walk::mtime_nanos(&metadata),
            )? {
                Outcome::Reindexed => reindexed += 1,
                Outcome::Skipped => skipped += 1,
                Outcome::Unchanged => {}
            }
        }

        tx.commit()?;
        Ok(RefreshReport {
            scanned,
            reindexed,
            removed,
            skipped,
            elapsed_ms: start.elapsed().as_millis(),
        })
    }

    pub fn stats(&self) -> Result<IndexStats, IndexError> {
        let files: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))?;
        let chunks: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))?;
        let symbols: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))?;
        let bytes: i64 =
            self.conn
                .query_row("SELECT COALESCE(SUM(size), 0) FROM files", [], |r| r.get(0))?;
        Ok(IndexStats {
            files: files as usize,
            chunks: chunks as usize,
            symbols: symbols as usize,
            bytes: bytes as u64,
        })
    }

    pub fn files(&self) -> Result<Vec<FileInfo>, IndexError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, language, lines, size FROM files ORDER BY path")?;
        let rows = stmt.query_map([], |r| {
            let language_str: String = r.get(1)?;
            Ok(FileInfo {
                path: r.get(0)?,
                language: Language::from_db_str(&language_str),
                lines: r.get::<_, i64>(2)? as u32,
                bytes: r.get::<_, i64>(3)? as u64,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// FTS5 `bm25`-ranked search, OR-ing every term. Terms are quoted as FTS5
    /// string literals (see `fts::build_or_query`), so arbitrary user text
    /// can never be read as query syntax.
    pub fn search_chunks(&self, terms: &[&str], limit: usize) -> Result<Vec<Chunk>, IndexError> {
        if terms.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let query = fts::build_or_query(terms);
        let mut stmt = self.conn.prepare(
            "SELECT c.path, c.start_line, c.end_line, c.text, bm25(chunks_fts)
             FROM chunks_fts JOIN chunks c ON c.id = chunks_fts.rowid
             WHERE chunks_fts MATCH ?1
             ORDER BY bm25(chunks_fts)
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |r| {
            let bm25: f64 = r.get(4)?;
            Ok(Chunk {
                path: r.get(0)?,
                start_line: r.get::<_, i64>(1)? as u32,
                end_line: r.get::<_, i64>(2)? as u32,
                text: r.get(3)?,
                score: -bm25,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn definitions(&self, name: &str, limit: usize) -> Result<Vec<Symbol>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, kind, path, start_line, end_line, container
             FROM symbols WHERE name = ?1 LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![name, limit as i64], row_to_symbol)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn symbols_in(&self, path: &str) -> Result<Vec<Symbol>, IndexError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, kind, path, start_line, end_line, container
             FROM symbols WHERE path = ?1 ORDER BY start_line",
        )?;
        let rows = stmt.query_map(params![path], row_to_symbol)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn imports_of(&self, path: &str) -> Result<Vec<Import>, IndexError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, target, line FROM imports WHERE path = ?1 ORDER BY line")?;
        let rows = stmt.query_map(params![path], |r| {
            Ok(Import {
                path: r.get(0)?,
                target: r.get(1)?,
                line: r.get::<_, i64>(2)? as u32,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

/// `<app_data>/index/<first 16 hex of sha256(canonical workspace path)>.sqlite`
pub fn index_db_path(app_data_dir: &Path, workspace_root: &Path) -> PathBuf {
    let root = canonical_root(workspace_root);
    let mut hasher = Sha256::new();
    hasher.update(root.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    app_data_dir.join("index").join(format!("{hex}.sqlite"))
}

#[cfg(test)]
mod tests;
