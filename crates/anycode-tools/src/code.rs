//! Code-intelligence tools (Phase 4): targeted, read-only lookups so an agent can find
//! what it needs instead of reading whole files. All three are Low risk in
//! `anycode-security` — they read the workspace and change nothing.
//!
//! Results are workspace content, so the orchestration loop hands them to the model inside
//! the untrusted envelope like any other tool output.

use crate::{Tool, ToolContext, ToolError};
use anycode_security::{capability_risk, RiskLevel};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Enough to be useful, few enough that a result never floods the context window.
const MAX_RESULTS: usize = 50;

fn required<'a>(input: &'a Value, field: &str) -> Result<&'a str, ToolError> {
    input[field]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| ToolError::InvalidInput(format!("missing \"{field}\"")))
}

fn limit(input: &Value) -> usize {
    input["limit"]
        .as_u64()
        .map_or(MAX_RESULTS, |n| (n as usize).clamp(1, MAX_RESULTS))
}

fn search_error(e: impl std::fmt::Display) -> ToolError {
    ToolError::InvalidInput(e.to_string())
}

/// Text search across the workspace, honouring `.gitignore`. Works whether or not the
/// index has finished building: it reads the files directly.
pub struct CodeSearchTool;

#[async_trait]
impl Tool for CodeSearchTool {
    fn name(&self) -> &'static str {
        "code.search"
    }
    fn risk(&self, _input: &Value) -> RiskLevel {
        capability_risk(self.name())
    }
    fn description(&self) -> &'static str {
        "Search the workspace's files for text (gitignored files excluded). Returns matching \
         lines with their path and line number. Use this to find where something is before \
         reading a file."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Text to find." },
                "regex": { "type": "boolean", "description": "Treat query as a regular expression." },
                "limit": { "type": "integer", "description": "Most results to return (max 50)." },
            },
            "required": ["query"],
        })
    }
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let query = required(&input, "query")?.to_string();
        let regex = input["regex"].as_bool() == Some(true);
        let limit = limit(&input);
        let root = ctx.workspace_path.clone();
        let matches = tokio::task::spawn_blocking(move || {
            anycode_code_intelligence::search_live(&root, &query, regex, limit)
        })
        .await
        .map_err(search_error)?
        .map_err(search_error)?;
        Ok(json!({ "matches": matches }))
    }
}

/// Where a symbol is defined, from the index's tree-sitter definitions.
pub struct CodeDefinitionTool;

#[async_trait]
impl Tool for CodeDefinitionTool {
    fn name(&self) -> &'static str {
        "code.definition"
    }
    fn risk(&self, _input: &Value) -> RiskLevel {
        capability_risk(self.name())
    }
    fn description(&self) -> &'static str {
        "Find where a function, type, class or other symbol is defined, by exact name. \
         Returns path, kind and line range for each definition."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Exact symbol name, e.g. \"parse_plan\"." },
            },
            "required": ["name"],
        })
    }
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let name = required(&input, "name")?.to_string();
        // Said plainly rather than answered with an empty list that would read as "none".
        let index = ctx.index.clone().ok_or_else(|| {
            ToolError::InvalidInput(
                "the workspace index is not ready yet; use code.search instead".into(),
            )
        })?;
        // The watcher may hold the index while it re-indexes; wait off the async threads.
        let definitions = tokio::task::spawn_blocking(move || {
            let index = index.lock().map_err(|_| {
                ToolError::InvalidInput("the workspace index is unavailable".into())
            })?;
            index.definitions(&name, MAX_RESULTS).map_err(search_error)
        })
        .await
        .map_err(search_error)??;
        Ok(json!({ "definitions": definitions }))
    }
}

/// Every whole-word use of an identifier across the workspace.
pub struct CodeReferencesTool;

#[async_trait]
impl Tool for CodeReferencesTool {
    fn name(&self) -> &'static str {
        "code.references"
    }
    fn risk(&self, _input: &Value) -> RiskLevel {
        capability_risk(self.name())
    }
    fn description(&self) -> &'static str {
        "Find every place an identifier is used (whole-word matches, gitignored files \
         excluded). Returns path, line number and the line."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The identifier, e.g. \"TaskMachine\"." },
                "limit": { "type": "integer", "description": "Most results to return (max 50)." },
            },
            "required": ["name"],
        })
    }
    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let name = required(&input, "name")?.to_string();
        let limit = limit(&input);
        let root = ctx.workspace_path.clone();
        let matches = tokio::task::spawn_blocking(move || {
            anycode_code_intelligence::references(&root, &name, limit)
        })
        .await
        .map_err(search_error)?
        .map_err(search_error)?;
        Ok(json!({ "references": matches }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::sync::{Arc, Mutex};

    fn context(dir: &TempDir, with_index: bool) -> ToolContext {
        let index = with_index.then(|| {
            let mut index = anycode_code_intelligence::Index::open_in_memory(dir.path()).unwrap();
            index.refresh().unwrap();
            Arc::new(Mutex::new(index))
        });
        ToolContext {
            fs_root: anycode_fs::WorkspaceRoot::new(dir.path()).unwrap(),
            workspace_path: dir.path().to_path_buf(),
            path_env: None,
            index,
        }
    }

    fn fixture() -> TempDir {
        let dir = TempDir::new();
        std::fs::write(
            dir.path().join("calc.py"),
            "def add(a, b):\n    return a + b\n\n\ndef multiply(a, b):\n    return add(a, 0) * b\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("main.py"),
            "from calc import multiply\nprint(multiply(2, 3))\n",
        )
        .unwrap();
        dir
    }

    #[tokio::test]
    async fn search_finds_lines_and_respects_the_limit() {
        let dir = fixture();
        let ctx = context(&dir, false);
        let out = CodeSearchTool
            .execute(json!({ "query": "multiply" }), &ctx)
            .await
            .unwrap();
        let matches = out["matches"].as_array().unwrap();
        assert!(matches.iter().any(|m| m["path"] == "calc.py"));
        assert!(matches.iter().any(|m| m["path"] == "main.py"));
        let one = CodeSearchTool
            .execute(json!({ "query": "multiply", "limit": 1 }), &ctx)
            .await
            .unwrap();
        assert_eq!(one["matches"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn definition_uses_the_index_and_says_when_it_is_not_ready() {
        let dir = fixture();
        let found = CodeDefinitionTool
            .execute(json!({ "name": "multiply" }), &context(&dir, true))
            .await
            .unwrap();
        let defs = found["definitions"].as_array().unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0]["path"], "calc.py");
        // Without an index the tool refuses plainly rather than claiming "no definitions".
        let err = CodeDefinitionTool
            .execute(json!({ "name": "multiply" }), &context(&dir, false))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not ready"), "{err}");
    }

    #[tokio::test]
    async fn references_are_whole_words() {
        let dir = fixture();
        std::fs::write(dir.path().join("other.py"), "multiplyer = 1\n").unwrap();
        let out = CodeReferencesTool
            .execute(json!({ "name": "multiply" }), &context(&dir, false))
            .await
            .unwrap();
        let refs = out["references"].as_array().unwrap();
        assert!(!refs.is_empty());
        assert!(refs.iter().all(|r| r["path"] != "other.py"), "{refs:?}");
    }

    #[test]
    fn all_three_are_low_risk() {
        for tool in [
            &CodeSearchTool as &dyn Tool,
            &CodeDefinitionTool,
            &CodeReferencesTool,
        ] {
            assert_eq!(tool.risk(&json!({})), RiskLevel::Low, "{}", tool.name());
        }
    }
}
