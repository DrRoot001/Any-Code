use crate::{Tool, ToolContext, ToolError};
use anycode_security::{capability_risk, path_risk, RiskLevel};

/// A filesystem call is at least as risky as the path it names: reading `.env` is not the
/// same act as reading `README.md`, whatever the tool (audit S1).
fn risk_for_path(capability: &str, input: &Value) -> RiskLevel {
    let base = capability_risk(capability);
    match input["path"].as_str().and_then(path_risk) {
        Some((risk, _)) => base.max(risk),
        None => base,
    }
}

fn reason_for_path(input: &Value) -> Option<String> {
    let path = input["path"].as_str()?;
    path_risk(path).map(|(_, reason)| format!("{path}: {reason}"))
}
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct FilesystemReadTool;

#[async_trait]
impl Tool for FilesystemReadTool {
    fn name(&self) -> &'static str {
        "filesystem.read.workspace"
    }

    fn risk(&self, input: &Value) -> RiskLevel {
        risk_for_path(self.name(), input)
    }

    fn risk_reason(&self, input: &Value) -> Option<String> {
        reason_for_path(input)
    }

    fn description(&self) -> &'static str {
        "Read a text file's full contents from the open workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path relative to the workspace root." },
            },
            "required": ["path"],
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("missing \"path\"".into()))?;
        let content = anycode_fs::read_file(&ctx.fs_root, path)?;
        Ok(json!({ "content": content }))
    }
}

pub struct FilesystemWriteTool;

#[async_trait]
impl Tool for FilesystemWriteTool {
    fn name(&self) -> &'static str {
        "filesystem.write.workspace"
    }

    fn risk(&self, input: &Value) -> RiskLevel {
        risk_for_path(self.name(), input)
    }

    fn risk_reason(&self, input: &Value) -> Option<String> {
        reason_for_path(input)
    }

    fn description(&self) -> &'static str {
        "Create a new text file, or replace an existing file's entire contents. To change \
         part of an existing file, use filesystem.edit.workspace instead — this discards \
         everything not in `content`."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path relative to the workspace root." },
                "content": { "type": "string", "description": "Full file contents to write." },
            },
            "required": ["path", "content"],
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("missing \"path\"".into()))?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("missing \"content\"".into()))?;
        anycode_fs::write_file(&ctx.fs_root, path, content)?;
        Ok(json!({}))
    }
}

/// Replaces one exact span of a file. Exists because whole-file writes are the wrong
/// primitive for changing code: a live run showed a model "implementing" one function by
/// writing a file that contained only that function, silently deleting the rest. An
/// exact-match edit either changes precisely what was named or refuses — it cannot
/// truncate a file by accident.
pub struct FilesystemEditTool;

#[async_trait]
impl Tool for FilesystemEditTool {
    fn name(&self) -> &'static str {
        "filesystem.edit.workspace"
    }

    fn risk(&self, input: &Value) -> RiskLevel {
        risk_for_path(self.name(), input)
    }

    fn risk_reason(&self, input: &Value) -> Option<String> {
        reason_for_path(input)
    }

    fn description(&self) -> &'static str {
        "Change part of an existing text file: replace old_text, which must appear exactly \
         once in the file, with new_text. The rest of the file is kept as it is. Copy \
         old_text exactly from the file, including enough surrounding lines to be unique."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path relative to the workspace root." },
                "old_text": { "type": "string", "description": "Exact text to replace; must occur once." },
                "new_text": { "type": "string", "description": "Text to put in its place." },
            },
            "required": ["path", "old_text", "new_text"],
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let field = |name: &str| {
            input[name]
                .as_str()
                .ok_or_else(|| ToolError::InvalidInput(format!("missing \"{name}\"")))
        };
        let (path, old_text, new_text) = (field("path")?, field("old_text")?, field("new_text")?);
        if old_text.is_empty() {
            return Err(ToolError::InvalidInput(
                "old_text is empty; to create a file use filesystem.write.workspace".into(),
            ));
        }

        let content = anycode_fs::read_file(&ctx.fs_root, path)?;
        // The error text is what the model reads, so it says how to recover.
        match content.matches(old_text).count() {
            1 => {}
            0 => {
                return Err(ToolError::InvalidInput(format!(
                    "old_text was not found in {path}; read the file and copy the text exactly"
                )))
            }
            n => {
                return Err(ToolError::InvalidInput(format!(
                    "old_text occurs {n} times in {path}; include more surrounding text so it \
                     matches exactly once"
                )))
            }
        }
        anycode_fs::write_file(&ctx.fs_root, path, &content.replacen(old_text, new_text, 1))?;
        Ok(json!({ "replaced": 1 }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::path::Path;

    fn context() -> (TempDir, ToolContext) {
        let dir = TempDir::new();
        let fs_root = anycode_fs::WorkspaceRoot::new(dir.path()).unwrap();
        let workspace_path = dir.path().to_path_buf();
        (
            dir,
            ToolContext {
                fs_root,
                workspace_path,
                path_env: None,
            },
        )
    }

    #[test]
    fn secret_paths_raise_every_filesystem_tool() {
        let env = json!({ "path": ".env.production" });
        let key = json!({ "path": "deploy/id_ed25519" });
        let plain = json!({ "path": "src/lib.rs" });
        // Audit S1: reading .env used to be Low — auto-allowed, sent to the model.
        assert_eq!(FilesystemReadTool.risk(&env), RiskLevel::High);
        assert_eq!(FilesystemReadTool.risk(&key), RiskLevel::Critical);
        assert_eq!(FilesystemReadTool.risk(&plain), RiskLevel::Low);
        assert_eq!(FilesystemWriteTool.risk(&env), RiskLevel::High);
        assert_eq!(FilesystemEditTool.risk(&plain), RiskLevel::Medium);
        assert!(FilesystemReadTool
            .risk_reason(&env)
            .unwrap()
            .contains("environment file"));
        assert_eq!(FilesystemReadTool.risk_reason(&plain), None);
    }

    #[tokio::test]
    async fn write_then_read_round_trips() {
        let (_dir, ctx) = context();
        FilesystemWriteTool
            .execute(json!({ "path": "a.txt", "content": "hello" }), &ctx)
            .await
            .unwrap();
        let result = FilesystemReadTool
            .execute(json!({ "path": "a.txt" }), &ctx)
            .await
            .unwrap();
        assert_eq!(result["content"], "hello");
    }

    #[tokio::test]
    async fn read_rejects_missing_path_argument() {
        let (_dir, ctx) = context();
        let err = FilesystemReadTool
            .execute(json!({}), &ctx)
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn write_cannot_escape_the_workspace_root() {
        let (_dir, ctx) = context();
        let err = FilesystemWriteTool
            .execute(json!({ "path": "../escape.txt", "content": "x" }), &ctx)
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Fs(_)));
        assert!(!Path::new("/tmp/escape.txt").exists());
    }
    async fn write(ctx: &ToolContext, content: &str) {
        FilesystemWriteTool
            .execute(json!({ "path": "calc.py", "content": content }), ctx)
            .await
            .unwrap();
    }

    async fn read(ctx: &ToolContext) -> String {
        FilesystemReadTool
            .execute(json!({ "path": "calc.py" }), ctx)
            .await
            .unwrap()["content"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn edit_changes_only_the_named_span() {
        let (_dir, ctx) = context();
        write(
            &ctx,
            "def add(a, b):\n    return a + b\n\ndef mul(a, b):\n    raise NotImplementedError\n",
        )
        .await;
        FilesystemEditTool
            .execute(
                json!({ "path": "calc.py", "old_text": "    raise NotImplementedError", "new_text": "    return a * b" }),
                &ctx,
            )
            .await
            .unwrap();
        // The function the edit didn't name is still there — the failure a whole-file
        // write produced in a live run.
        assert_eq!(
            read(&ctx).await,
            "def add(a, b):\n    return a + b\n\ndef mul(a, b):\n    return a * b\n"
        );
    }

    #[tokio::test]
    async fn edit_refuses_text_that_is_absent_or_ambiguous() {
        let (_dir, ctx) = context();
        write(&ctx, "x = 1\nx = 1\n").await;
        for (old_text, expected) in [("y = 2", "not found"), ("x = 1", "occurs 2 times")] {
            let err = FilesystemEditTool
                .execute(
                    json!({ "path": "calc.py", "old_text": old_text, "new_text": "z" }),
                    &ctx,
                )
                .await
                .unwrap_err();
            assert!(err.to_string().contains(expected), "{err}");
        }
        assert_eq!(
            read(&ctx).await,
            "x = 1\nx = 1\n",
            "a refused edit must change nothing"
        );
    }

    #[tokio::test]
    async fn edit_cannot_escape_the_workspace_root() {
        let (_dir, ctx) = context();
        let err = FilesystemEditTool
            .execute(
                json!({ "path": "../calc.py", "old_text": "a", "new_text": "b" }),
                &ctx,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::Fs(_)));
    }
}
