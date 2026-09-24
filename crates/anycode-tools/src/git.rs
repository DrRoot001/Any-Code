use crate::{Tool, ToolContext, ToolError};
use anycode_security::{capability_risk, RiskLevel};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &'static str {
        "git.status"
    }

    fn risk(&self, _input: &Value) -> RiskLevel {
        capability_risk(self.name())
    }

    fn description(&self) -> &'static str {
        "Show the working tree's changed, added, deleted, and untracked files."
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    async fn execute(&self, _input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let entries = anycode_git::status(&ctx.workspace_path)?;
        Ok(json!({ "entries": entries }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::process::Command;

    fn run(dir: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn reports_an_untracked_file() {
        let dir = TempDir::new();
        run(dir.path(), &["init", "-q", "-b", "main"]);
        std::fs::write(dir.path().join("new.txt"), "hi").unwrap();

        let fs_root = anycode_fs::WorkspaceRoot::new(dir.path()).unwrap();
        let ctx = ToolContext {
            fs_root,
            workspace_path: dir.path().to_path_buf(),
            path_env: None,
            index: None,
        };

        let result = GitStatusTool.execute(json!({}), &ctx).await.unwrap();
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["path"], "new.txt");
        assert_eq!(entries[0]["status"], "untracked");
    }
}
