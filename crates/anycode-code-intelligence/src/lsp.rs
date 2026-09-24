//! A minimal JSON-RPC-over-stdio LSP client (ADR 0004 "Language servers").
//!
//! Language servers are found **only** on the user's `PATH` — never
//! downloaded or bundled by this crate. [`server_for`] reports plainly (by
//! returning `None`) when nothing is there; callers must not fake a result.
//!
//! Server stdout/stderr is untrusted, and it can echo back file contents
//! (e.g. inside a diagnostic message), so nothing read from a server is ever
//! written to a log here. Waiting for a response times out (writes to the
//! server's stdin do not yet — a server that stops reading can block a
//! request; wire this into the agent only behind a thread it can abandon).
//! The child process is always killed on drop, even after a panic mid-request.

use crate::path_util;
use crate::Language;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("language server did not respond within the timeout")]
    Timeout,
    #[error("language server reported an error: {0}")]
    Server(String),
    #[error("no language server is configured for this language")]
    UnsupportedLanguage,
    #[error("the language server process exited")]
    ProcessExited,
}

/// A language server command, resolved from the user's `PATH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
}

/// Searches `PATH` for `program`. Only absolute entries count, and a candidate that
/// resolves inside the workspace is refused: `.`, `node_modules/.bin` or a direnv-added
/// project directory would otherwise run a binary the repository supplies. The
/// resolved, absolute path is what gets spawned. std-only: no dependency pulls its
/// weight over a directory scan and an executable-bit check.
fn find_on_path(program: &str, workspace_root: &Path) -> Option<PathBuf> {
    find_in(program, &std::env::var_os("PATH")?, workspace_root)
}

fn find_in(program: &str, path_var: &std::ffi::OsStr, workspace_root: &Path) -> Option<PathBuf> {
    let workspace = path_util::best_effort_canonical(workspace_root);
    let accept = |candidate: PathBuf| -> Option<PathBuf> {
        let resolved = candidate.canonicalize().ok()?;
        (!resolved.starts_with(&workspace)).then_some(resolved)
    };
    for dir in std::env::split_paths(path_var).filter(|d| d.is_absolute()) {
        let candidate = dir.join(program);
        if is_executable_file(&candidate) {
            if let Some(found) = accept(candidate) {
                return Some(found);
            }
        }
        #[cfg(windows)]
        for ext in ["exe", "cmd", "bat"] {
            let with_ext = dir.join(format!("{program}.{ext}"));
            if with_ext.is_file() {
                if let Some(found) = accept(with_ext) {
                    return Some(found);
                }
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// Looks up the language server for `language`, if one is on `PATH` and outside
/// `workspace_root`. `None` means plainly "not found" — never a faked server.
///
/// Starting a language server runs repository code (rust-analyzer runs `build.rs` and
/// proc macros; typescript-language-server loads the workspace's own TypeScript), so
/// wiring this into the app needs its own capability in the permission engine first.
pub fn server_for(language: Language, workspace_root: &Path) -> Option<ServerCommand> {
    match language {
        Language::Rust => {
            find_on_path("rust-analyzer", workspace_root).map(|program| ServerCommand {
                program,
                args: vec![],
            })
        }
        Language::TypeScript | Language::Tsx | Language::JavaScript => {
            find_on_path("typescript-language-server", workspace_root).map(|program| {
                ServerCommand {
                    program,
                    args: vec!["--stdio".to_string()],
                }
            })
        }
        Language::Python => {
            find_on_path("pyright-langserver", workspace_root).map(|program| ServerCommand {
                program,
                args: vec!["--stdio".to_string()],
            })
        }
        Language::Other => None,
    }
}

fn language_id_for(language: Language) -> Option<&'static str> {
    match language {
        Language::Rust => Some("rust"),
        Language::TypeScript => Some("typescript"),
        Language::Tsx => Some("typescriptreact"),
        Language::JavaScript => Some("javascript"),
        Language::Python => Some("python"),
        Language::Other => None,
    }
}

/// A location returned by `textDocument/definition` or
/// `textDocument/references`: a workspace-relative path plus a range. Lines
/// are 1-indexed inclusive, matching [`crate::Symbol`]; characters are
/// 0-indexed UTF-16 code units, LSP's own unit, left as-is since nothing
/// else in this crate has an established convention for them.
///
/// ponytail: a definition/reference outside the workspace root (e.g. into a
/// vendored dependency or the standard library) is dropped rather than
/// returned with an absolute path — upgrade if cross-workspace results turn
/// out to matter.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub path: String,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

/// Content-Length-framed JSON-RPC message, LSP's own wire format.
fn write_message<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Largest message accepted from a server. Its `Content-Length` is untrusted; without a cap
/// a corrupt header would allocate whatever it claims.
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

/// Reads one framed message, or `Ok(None)` on a clean EOF.
fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let len = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    if len > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message of {len} bytes exceeds the {MAX_MESSAGE_BYTES}-byte limit"),
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let value =
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}

/// A running language server, speaking JSON-RPC over its stdio.
pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    _reader: JoinHandle<()>,
    next_id: i64,
    root: PathBuf,
}

impl LspClient {
    /// Spawns `cmd` with `workspace_root` as its working directory and as
    /// the root of every URI this client builds. Does not call `initialize`
    /// — that is a separate step so a caller can pick its own timeout.
    pub fn spawn(cmd: &ServerCommand, workspace_root: &Path) -> Result<Self, LspError> {
        let mut child = Command::new(&cmd.program)
            .args(&cmd.args)
            .current_dir(workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Server stderr is untrusted and may echo file contents in a
            // diagnostic; never surface it.
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("child has no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("child has no stdout"))?;

        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut buf = BufReader::new(stdout);
            while let Ok(Some(value)) = read_message(&mut buf) {
                if tx.send(value).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            rx,
            _reader: reader,
            next_id: 1,
            root: path_util::best_effort_canonical(workspace_root),
        })
    }

    fn take_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send(&mut self, value: &Value) -> Result<(), LspError> {
        write_message(&mut self.stdin, value).map_err(LspError::Io)
    }

    /// Sends a request and blocks for its response, up to `timeout`. While
    /// waiting, any server-to-client request seen on the wire is
    /// acknowledged with a null result (so a server that expects e.g.
    /// `client/registerCapability` to be answered isn't left hanging), and
    /// any notification is discarded.
    fn request(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, LspError> {
        let id = self.take_id();
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))?;
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(LspError::Timeout);
            }
            let msg = match self.rx.recv_timeout(remaining) {
                Ok(msg) => msg,
                Err(RecvTimeoutError::Timeout) => return Err(LspError::Timeout),
                Err(RecvTimeoutError::Disconnected) => return Err(LspError::ProcessExited),
            };
            let msg_id = msg.get("id").cloned();
            let is_request = msg.get("method").is_some() && msg_id.is_some();
            if is_request {
                let _ = self.send(&json!({"jsonrpc": "2.0", "id": msg_id, "result": Value::Null}));
                continue;
            }
            if msg.get("method").is_some() {
                continue; // a notification; nothing to correlate it to.
            }
            if msg_id == Some(Value::from(id)) {
                if let Some(error) = msg.get("error") {
                    return Err(LspError::Server(error.to_string()));
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
            // A response to an id we're no longer waiting on; drop it.
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
    }

    /// `initialize` + `initialized`, the LSP handshake.
    pub fn initialize(&mut self, timeout: Duration) -> Result<(), LspError> {
        let root_uri = path_to_uri(&self.root);
        let name = self
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {},
            "workspaceFolders": [{"uri": root_uri, "name": name}],
        });
        self.request("initialize", params, timeout)?;
        self.notify("initialized", json!({}))
    }

    /// `textDocument/didOpen` for a workspace-relative path.
    pub fn did_open(
        &mut self,
        rel_path: &str,
        language: Language,
        text: &str,
    ) -> Result<(), LspError> {
        let language_id = language_id_for(language).ok_or(LspError::UnsupportedLanguage)?;
        let uri = path_to_uri(&path_util::to_fs_path(&self.root, rel_path));
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                }
            }),
        )
    }

    /// `textDocument/definition` at a 1-indexed `line`.
    pub fn definition(
        &mut self,
        rel_path: &str,
        line: u32,
        character: u32,
        timeout: Duration,
    ) -> Result<Vec<Location>, LspError> {
        let result = self.request(
            "textDocument/definition",
            self.position_params(rel_path, line, character),
            timeout,
        )?;
        Ok(parse_locations(&result, &self.root))
    }

    /// `textDocument/references` at a 1-indexed `line`, including the
    /// declaration itself.
    pub fn references(
        &mut self,
        rel_path: &str,
        line: u32,
        character: u32,
        timeout: Duration,
    ) -> Result<Vec<Location>, LspError> {
        let mut params = self.position_params(rel_path, line, character);
        params["context"] = json!({"includeDeclaration": true});
        let result = self.request("textDocument/references", params, timeout)?;
        Ok(parse_locations(&result, &self.root))
    }

    /// `textDocument/hover` at a 1-indexed `line`, flattened to plain text.
    pub fn hover(
        &mut self,
        rel_path: &str,
        line: u32,
        character: u32,
        timeout: Duration,
    ) -> Result<Option<String>, LspError> {
        let result = self.request(
            "textDocument/hover",
            self.position_params(rel_path, line, character),
            timeout,
        )?;
        Ok(extract_hover_text(&result))
    }

    fn position_params(&self, rel_path: &str, line: u32, character: u32) -> Value {
        let uri = path_to_uri(&path_util::to_fs_path(&self.root, rel_path));
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line.saturating_sub(1), "character": character},
        })
    }

    /// `shutdown` + `exit`, waiting up to `timeout` for the process to exit
    /// on its own before this client is dropped (which kills it
    /// unconditionally).
    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), LspError> {
        let _ = self.request("shutdown", Value::Null, timeout);
        let _ = self.notify("exit", Value::Null);
        let deadline = Instant::now() + timeout;
        loop {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Best-effort: the process may already have exited via `shutdown`.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Converts a filesystem path to a `file://` URI. std-only percent-encoding:
/// the only bytes that need it in practice are spaces and other punctuation
/// in a path component, and there is no dependency worth adding for that.
fn path_to_uri(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let path_str = path_str.replace('\\', "/");
    let mut out = String::from("file://");
    for segment in path_str.split('/') {
        if segment.is_empty() {
            continue;
        }
        out.push('/');
        for byte in segment.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':' => {
                    out.push(byte as char)
                }
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
    }
    out
}

/// The inverse of [`path_to_uri`].
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    decoded.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    let decoded = String::from_utf8(decoded).ok()?;
    Some(PathBuf::from(decoded))
}

fn uri_to_relative_path(uri: &str, root: &Path) -> Option<String> {
    let path = uri_to_path(uri)?;
    path_util::to_workspace_relative(root, &path)
}

fn parse_locations(value: &Value, root: &Path) -> Vec<Location> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| parse_one_location(item, root))
            .collect(),
        Value::Object(_) => parse_one_location(value, root).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// Parses one `Location` or `LocationLink` (the two shapes `definition` and
/// `references` responses use), dropping it if it falls outside the
/// workspace root.
fn parse_one_location(value: &Value, root: &Path) -> Option<Location> {
    let (uri, range) = match value.get("uri") {
        Some(uri) => (uri.as_str()?, value.get("range")?),
        None => (
            value.get("targetUri")?.as_str()?,
            value
                .get("targetSelectionRange")
                .or_else(|| value.get("targetRange"))?,
        ),
    };
    let start = range.get("start")?;
    let end = range.get("end")?;
    Some(Location {
        path: uri_to_relative_path(uri, root)?,
        start_line: start.get("line")?.as_u64()? as u32 + 1,
        start_character: start.get("character")?.as_u64()? as u32,
        end_line: end.get("line")?.as_u64()? as u32 + 1,
        end_character: end.get("character")?.as_u64()? as u32,
    })
}

/// Flattens a `Hover` result's `contents` (a `MarkedString`, a list of
/// them, or `MarkupContent`) into plain text.
fn extract_hover_text(value: &Value) -> Option<String> {
    let contents = value.get("contents")?;
    fn one(v: &Value) -> Option<String> {
        match v {
            Value::String(s) => Some(s.clone()),
            Value::Object(_) => v.get("value").and_then(|v| v.as_str()).map(str::to_string),
            _ => None,
        }
    }
    let text = match contents {
        Value::Array(items) => items
            .iter()
            .filter_map(one)
            .collect::<Vec<_>>()
            .join("\n\n"),
        other => one(other)?,
    };
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn framing_round_trips() {
        let original = json!({"jsonrpc": "2.0", "id": 7, "method": "textDocument/definition", "params": {"a": 1}});
        let mut buf = Vec::new();
        write_message(&mut buf, &original).expect("write");
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).expect("read").expect("some");
        assert_eq!(decoded, original);
    }

    #[test]
    fn an_oversized_content_length_is_refused_without_allocating() {
        let wire = format!("Content-Length: {}\r\n\r\n", MAX_MESSAGE_BYTES + 1);
        let err = read_message(&mut std::io::Cursor::new(wire.into_bytes())).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn framing_reports_clean_eof() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        assert!(read_message(&mut cursor).expect("read").is_none());
    }

    #[test]
    fn path_to_uri_round_trips() {
        let path = std::env::temp_dir().join("any code test dir").join("f.rs");
        let uri = path_to_uri(&path);
        assert!(uri.starts_with("file:///"));
        let back = uri_to_path(&uri).expect("decodes");
        assert_eq!(back, path);
    }

    #[cfg(unix)]
    #[test]
    fn a_server_inside_the_workspace_is_never_found() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = dir.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let fake = bin.join("anycode-test-lsp");
        std::fs::write(&fake, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        // An absolute PATH entry that points into the workspace: refused.
        assert_eq!(
            find_in("anycode-test-lsp", bin.as_os_str(), dir.path()),
            None
        );
        // A relative entry is never searched, wherever it resolves.
        assert_eq!(
            find_in("anycode-test-lsp", "bin".as_ref(), dir.path()),
            None
        );
        // The same binary is found when the workspace is elsewhere.
        let elsewhere = tempfile::tempdir().expect("tempdir");
        assert!(find_in("anycode-test-lsp", bin.as_os_str(), elsewhere.path()).is_some());
    }

    #[test]
    fn server_for_reports_missing_plainly() {
        // Rust variant may or may not resolve depending on the machine;
        // `Other` never has a server configured at all.
        assert!(server_for(Language::Other, Path::new("/")).is_none());
    }

    /// Exercises a real `rust-analyzer` end to end, but only when one is
    /// actually on PATH — never fakes a server.
    #[test]
    fn rust_analyzer_definition_lookup() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let Some(server) = server_for(Language::Rust, root) else {
            eprintln!("skipping rust_analyzer_definition_lookup: no rust-analyzer on PATH");
            return;
        };
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"lsp-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        let source = "fn greet() -> u32 {\n    41\n}\n\nfn main() {\n    let _ = greet();\n}\n";
        std::fs::write(root.join("src/main.rs"), source).unwrap();

        let timeout = Duration::from_secs(60);
        let mut client = LspClient::spawn(&server, root).expect("spawn rust-analyzer");

        // A `rust-analyzer` resolved on PATH can still be a non-functional
        // stub (e.g. a `rustup` proxy for a toolchain component that was
        // never installed, which exits immediately with an error). That is
        // a real, reportable condition ("found on PATH" and "actually
        // works" are different things) — skip the rest of this exercise
        // rather than fail the suite over a broken local toolchain, but
        // only for that specific case; any other failure is a real bug.
        let give_up_at = Instant::now() + Duration::from_secs(3);
        loop {
            if let Ok(Some(status)) = client.child.try_wait() {
                eprintln!(
                    "skipping rust_analyzer_definition_lookup: the rust-analyzer on PATH exited \
                     ({status}) instead of speaking LSP — likely a rustup proxy for an \
                     uninstalled component, not a real server"
                );
                return;
            }
            if Instant::now() >= give_up_at {
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }

        client.initialize(timeout).expect("initialize");
        client
            .did_open("src/main.rs", Language::Rust, source)
            .expect("did_open");

        // `greet()` is called on line 6 (1-indexed); "greet" starts at
        // character 12 in `    let _ = greet();`.
        let locations = client
            .definition("src/main.rs", 6, 13, timeout)
            .expect("definition");

        client.shutdown(Duration::from_secs(5)).ok();

        assert!(
            !locations.is_empty(),
            "expected rust-analyzer to resolve greet()'s definition"
        );
        assert_eq!(locations[0].path, "src/main.rs");
        assert_eq!(locations[0].start_line, 1);
    }
}
