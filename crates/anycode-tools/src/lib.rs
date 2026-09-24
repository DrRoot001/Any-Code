//! The capability registry (PRD §41-42): every native tool an agent can call lives here,
//! behind one trait. docs/ARCHITECTURE.md invariant #4 — "agent never bypasses the
//! capability runtime" — means exactly this: an agent that wants to touch the
//! filesystem, git, or a shell has no path to do so except through a `Tool` looked up in
//! a `ToolRegistry`. There is no other function anywhere that lets a model-originated
//! request reach the filesystem directly.
//!
//! This crate computes risk (via anycode-security) but does not decide allow/ask/deny
//! and does not check standing grants — that needs the workspace path and the local
//! store, which belong to the orchestration layer (src-tauri), not here. A tool answers
//! two questions: "what's the risk of running with this input?" and "run it."

mod code;
mod filesystem;
mod git;
mod shell;

use anycode_security::RiskLevel;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

pub use code::{CodeDefinitionTool, CodeReferencesTool, CodeSearchTool};
pub use filesystem::{FilesystemEditTool, FilesystemReadTool, FilesystemWriteTool};
pub use git::GitStatusTool;
pub use shell::ShellExecuteTool;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error(transparent)]
    Fs(#[from] anycode_fs::FsError),
    #[error(transparent)]
    Git(#[from] anycode_git::GitError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("command timed out after {0} seconds")]
    Timeout(u64),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

/// What a tool needs to run — nothing more. No secrets, no store handle: a tool cannot
/// reach further than the workspace it was given.
pub struct ToolContext {
    pub fs_root: anycode_fs::WorkspaceRoot,
    pub workspace_path: PathBuf,
    /// `PATH` for commands a tool spawns, when the caller resolved the user's own (see
    /// `anycode_terminal::login_shell_path`). `None` inherits this process's, which for
    /// an app opened from Finder is only the system directories.
    pub path_env: Option<String>,
    /// The workspace's code index, once built. `None` while it is building; tools that
    /// need it say so rather than pretending there is nothing to find.
    pub index: Option<std::sync::Arc<std::sync::Mutex<anycode_code_intelligence::Index>>>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    /// Stable identifier, e.g. `filesystem.write.workspace`. Doubles as the capability
    /// id anycode-security's static risk table and the permission-grant store key on.
    fn name(&self) -> &'static str;

    /// Risk for *this* invocation. Most tools return a static risk via
    /// `anycode_security::capability_risk(self.name())`; shell.execute is the one
    /// exception — its risk depends on the command text, not the tool identity.
    fn risk(&self, input: &Value) -> RiskLevel;

    /// Why this call is riskier than the tool usually is, for the approval prompt — e.g.
    /// "environment file — usually holds secrets". `None` when the tool's ordinary risk
    /// explains itself.
    fn risk_reason(&self, _input: &Value) -> Option<String> {
        None
    }

    /// What an "always allow" granted on this call covers — the key the standing grant is
    /// stored under. By default the whole capability; a tool whose calls differ in kind
    /// (a shell runs anything) narrows it to what the user actually saw.
    fn grant_scope(&self, _input: &Value) -> String {
        self.name().to_string()
    }

    /// One-line description for a model deciding whether to call this tool.
    fn description(&self) -> &'static str;

    /// JSON Schema for `input`. Owned by the tool itself so the schema can never drift
    /// from what `execute` actually reads out of the input.
    fn input_schema(&self) -> Value;

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError>;
}

/// One tool's model-facing description — name, description, input schema. Deliberately
/// not `anycode_models::ToolDefinition`: this crate has no reason to depend on the
/// provider-abstraction crate just to name a type. The orchestration layer, which
/// depends on both, does that conversion.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Every tool an agent may call, looked up by name. Construction is the one place that
/// decides what's on the menu at all — a tool not registered here cannot be invoked, no
/// matter what a model asks for.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// The standard set available to every workspace in Phase 3.
    pub fn standard() -> Self {
        Self {
            tools: vec![
                Box::new(FilesystemReadTool),
                Box::new(FilesystemWriteTool),
                Box::new(FilesystemEditTool),
                Box::new(GitStatusTool),
                Box::new(CodeSearchTool),
                Box::new(CodeDefinitionTool),
                Box::new(CodeReferencesTool),
                Box::new(ShellExecuteTool),
            ],
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.tools.iter().map(|t| t.name())
    }

    /// Model-facing descriptions for every registered tool, for building a
    /// provider-agnostic tool-call request.
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools
            .iter()
            .map(|t| ToolSpec {
                name: t.name(),
                description: t.description(),
                input_schema: t.input_schema(),
            })
            .collect()
    }
}

/// Required arguments `input` lacks, according to `schema`'s `required` list. Checked
/// before a call reaches the permission gate: a malformed call can never run, so asking
/// a person to approve it wastes their attention on a decision that doesn't exist.
pub fn missing_required(schema: &Value, input: &Value) -> Vec<String> {
    schema["required"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|name| input.get(*name).is_none_or(Value::is_null))
        .map(String::from)
        .collect()
}

/// Zero-dependency temp dir shared by every tool's tests, matching the pattern
/// anycode-fs and anycode-git already use.
#[cfg(test)]
pub(crate) mod test_support {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    pub struct TempDir(PathBuf);
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    impl TempDir {
        pub fn new() -> Self {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("anycode-tools-test-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_registry_exposes_the_expected_tools() {
        let registry = ToolRegistry::standard();
        let names: Vec<_> = registry.names().collect();
        assert!(names.contains(&"filesystem.read.workspace"));
        assert!(names.contains(&"filesystem.write.workspace"));
        assert!(names.contains(&"filesystem.edit.workspace"));
        assert!(names.contains(&"git.status"));
        assert!(names.contains(&"shell.execute"));
        for code_tool in ["code.search", "code.definition", "code.references"] {
            assert!(names.contains(&code_tool), "{code_tool}");
        }
    }

    #[test]
    fn missing_required_arguments_are_named() {
        let registry = ToolRegistry::standard();
        let edit = registry
            .get("filesystem.edit.workspace")
            .unwrap()
            .input_schema();
        // The shape a live model actually sent: write-style arguments to the edit tool.
        let sent = serde_json::json!({ "path": "calc.py", "content": "def f(): pass" });
        assert_eq!(missing_required(&edit, &sent), ["old_text", "new_text"]);
        let null_counts_as_missing =
            serde_json::json!({ "path": null, "old_text": "a", "new_text": "b" });
        assert_eq!(missing_required(&edit, &null_counts_as_missing), ["path"]);
        let complete = serde_json::json!({ "path": "a", "old_text": "b", "new_text": "" });
        assert!(missing_required(&edit, &complete).is_empty());
    }

    #[test]
    fn unknown_tool_name_is_not_found() {
        assert!(ToolRegistry::standard().get("database.drop").is_none());
    }

    /// Model adapters encode `.` as `__` for function-calling APIs that reject dots, and
    /// decode it back. That only round-trips if no name already contains `__`, and only
    /// passes validation if every other character is one those APIs accept.
    #[test]
    fn every_tool_name_survives_wire_encoding() {
        for name in ToolRegistry::standard().names() {
            assert!(!name.contains("__"), "{name} would not round-trip");
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'),
                "{name} contains a character function-calling APIs reject"
            );
        }
    }

    #[test]
    fn every_tool_produces_a_non_empty_spec() {
        for spec in ToolRegistry::standard().specs() {
            assert!(!spec.name.is_empty());
            assert!(!spec.description.is_empty());
            assert!(
                spec.input_schema["type"] == "object",
                "{} has no object schema",
                spec.name
            );
        }
    }
}
