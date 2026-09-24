use crate::{Tool, ToolContext, ToolError};
use anycode_security::{classify_shell_command, shell_risk_reason, RiskLevel};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// One-shot command execution with captured output — distinct from the interactive PTY
/// sessions the Terminal panel uses (Phase 1's `anycode-terminal`). An agent doesn't want
/// a shell to type into; it wants a command's stdout, stderr, and exit code.
pub struct ShellExecuteTool;

#[async_trait]
impl Tool for ShellExecuteTool {
    fn name(&self) -> &'static str {
        "shell.execute"
    }

    fn risk(&self, input: &Value) -> RiskLevel {
        match input["command"].as_str() {
            Some(command) => classify_shell_command(command),
            // Malformed input has no command text to classify — refuse to guess low.
            None => RiskLevel::Critical,
        }
    }

    fn risk_reason(&self, input: &Value) -> Option<String> {
        shell_risk_reason(input["command"].as_str()?).map(|(_, reason)| reason.to_string())
    }

    /// Audit S2: a grant used to cover the whole tool, so "always allow" on `npm test`
    /// allowed every shell command. It now covers exactly the command the user approved.
    fn grant_scope(&self, input: &Value) -> String {
        format!(
            "{}:{}",
            self.name(),
            input["command"].as_str().unwrap_or_default().trim()
        )
    }

    fn description(&self) -> &'static str {
        "Run a shell command in the workspace root and capture its stdout, stderr, and exit \
         code. Set verify to true when the command checks that your work is correct (a \
         build, lint, or test); leave it false for commands that only look around."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The command to run, e.g. \"npm test\"." },
                // Read by the orchestration loop, not by this tool: it decides which exit
                // codes count toward the task's verdict (anycode_agent::verdict).
                "verify": {
                    "type": "boolean",
                    "description": "True if this command verifies the work (build, lint, test).",
                },
            },
            "required": ["command"],
        })
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<Value, ToolError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("missing \"command\"".into()))?;

        let (shell, flag) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };
        let mut cmd = Command::new(shell);
        cmd.arg(flag).arg(command).current_dir(&ctx.workspace_path);
        if let Some(path) = &ctx.path_env {
            cmd.env("PATH", path);
        }
        // Dropping the future on timeout must not orphan the process.
        cmd.kill_on_drop(true);
        let run = cmd.output();

        let output = timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), run)
            .await
            .map_err(|_| ToolError::Timeout(DEFAULT_TIMEOUT_SECS))??;

        Ok(json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "exitCode": output.status.code(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn context(dir: &TempDir) -> ToolContext {
        let fs_root = anycode_fs::WorkspaceRoot::new(dir.path()).unwrap();
        ToolContext {
            fs_root,
            workspace_path: dir.path().to_path_buf(),
            path_env: None,
            index: None,
        }
    }

    #[tokio::test]
    async fn captures_stdout_and_exit_code() {
        let dir = TempDir::new();
        let result = ShellExecuteTool
            .execute(json!({ "command": "echo hi" }), &context(&dir))
            .await
            .unwrap();
        assert_eq!(result["stdout"].as_str().unwrap().trim(), "hi");
        assert_eq!(result["exitCode"], 0);
    }

    #[tokio::test]
    async fn a_failing_command_is_a_result_not_an_error() {
        let dir = TempDir::new();
        let result = ShellExecuteTool
            .execute(json!({ "command": "exit 7" }), &context(&dir))
            .await
            .unwrap();
        assert_eq!(result["exitCode"], 7);
    }

    #[tokio::test]
    async fn the_callers_path_reaches_the_command() {
        let dir = TempDir::new();
        let ctx = ToolContext {
            path_env: Some("/anycode-test-path:/usr/bin:/bin".into()),
            ..context(&dir)
        };
        let command = if cfg!(windows) {
            "echo %PATH%"
        } else {
            "echo $PATH"
        };
        let result = ShellExecuteTool
            .execute(json!({ "command": command }), &ctx)
            .await
            .unwrap();
        assert!(result["stdout"]
            .as_str()
            .unwrap()
            .contains("/anycode-test-path"));
    }

    #[test]
    fn a_grant_covers_only_the_command_it_was_given_for() {
        let scope = |c: &str| ShellExecuteTool.grant_scope(&json!({ "command": c }));
        assert_eq!(scope("npm test"), "shell.execute:npm test");
        assert_eq!(scope("  npm test "), scope("npm test"));
        assert_ne!(scope("npm test"), scope("npm install"));
        assert_ne!(scope("npm test"), scope("git push"));
    }

    #[test]
    fn missing_command_text_classifies_as_critical_not_low() {
        // Refuses to default a malformed request to a permissive risk level.
        assert_eq!(ShellExecuteTool.risk(&json!({})), RiskLevel::Critical);
    }

    #[test]
    fn risk_follows_the_command_text() {
        assert_eq!(
            ShellExecuteTool.risk(&json!({"command": "git status"})),
            RiskLevel::Low
        );
        assert_eq!(
            ShellExecuteTool.risk(&json!({"command": "rm -rf /"})),
            RiskLevel::Critical
        );
    }
}
