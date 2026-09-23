//! Permission engine (PRD §49-52, docs/SECURITY.md). "The model may request. The
//! runtime decides." — this crate is the runtime side of that sentence.
//!
//! Deliberately pure: no I/O, no SQLite, no Tauri. Whether a *standing* grant exists is
//! looked up elsewhere (anycode-store) and passed in; `decide` only encodes the policy
//! table itself, so the policy is one small thing to read and test, not scattered across
//! every call site that happens to touch a capability.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Ask,
    Deny,
}

/// Whether a capability has a standing "always allow" grant for the current workspace.
/// A one-time approval is never represented here — it isn't persisted, so it can't be
/// looked up; the caller that received a one-time "yes" just proceeds for that call only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandingGrant {
    None,
    WorkspaceAllowed,
}

/// The one function that turns a risk level into a decision. Critical never becomes
/// Allow, no matter what's stored — docs/SECURITY.md: "deny by default", and v1 has no
/// override path for it at all. High and Medium need either an existing workspace grant
/// or a live prompt; nothing above Low runs unattended on first encounter.
pub fn decide(risk: RiskLevel, grant: StandingGrant) -> Decision {
    if risk == RiskLevel::Critical {
        return Decision::Deny;
    }
    if grant == StandingGrant::WorkspaceAllowed {
        return Decision::Allow;
    }
    match risk {
        RiskLevel::Low => Decision::Allow,
        RiskLevel::Medium | RiskLevel::High => Decision::Ask,
        RiskLevel::Critical => unreachable!("handled above"),
    }
}

/// Static risk for capabilities whose risk doesn't depend on their arguments. Shell
/// commands are the exception — see [`classify_shell_command`]. An unrecognized
/// capability is Medium, never Low: an unknown thing must not run silently.
pub fn capability_risk(capability: &str) -> RiskLevel {
    match capability {
        "filesystem.read.workspace"
        | "code.search"
        | "code.definition"
        | "code.references"
        | "git.status"
        | "git.diff"
        | "git.branch" => RiskLevel::Low,

        "filesystem.write.workspace"
        | "filesystem.edit.workspace"
        | "git.commit"
        | "build.run"
        | "lint.run"
        | "test.run" => RiskLevel::Medium,

        "git.push" | "deployment.staging" => RiskLevel::High,

        "filesystem.write.outside_workspace"
        | "shell.admin"
        | "deployment.production"
        | "database.destructive" => RiskLevel::Critical,

        _ => RiskLevel::Medium,
    }
}

/// Commands that never require a prompt because they can't change anything.
const LOW_RISK_COMMANDS: &[&str] = &[
    "ls",
    "pwd",
    "cat",
    "echo",
    "git status",
    "git diff",
    "git log",
    "git branch",
    "npm test",
    "cargo check",
    "cargo test",
    "cargo build",
];

/// Patterns severe enough to deny outright rather than ask — the kind of command where
/// "are you sure?" isn't a meaningful safeguard.
const CRITICAL_SHELL_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf ~",
    "rm -rf *",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",
];

/// Classifies a shell command by matching against small allow/deny lists (docs/SECURITY.md
/// §51's examples), defaulting unmatched commands to Medium.
///
/// This is a **labeling heuristic**, not the security boundary — it exists to prioritize
/// and word the approval prompt sensibly. The actual boundary is that `decide()` still
/// asks for every Medium/High command without a standing grant, and denies Critical
/// outright: a command this function misjudges as Medium instead of Critical still can't
/// run without the user seeing its exact text and approving it first.
pub fn classify_shell_command(command: &str) -> RiskLevel {
    let trimmed = command.trim();
    if CRITICAL_SHELL_PATTERNS.iter().any(|p| trimmed.contains(p)) {
        return RiskLevel::Critical;
    }
    if LOW_RISK_COMMANDS.contains(&trimmed) {
        return RiskLevel::Low;
    }
    if trimmed.starts_with("git push") || trimmed.starts_with("terraform apply") {
        return RiskLevel::High;
    }
    RiskLevel::Medium
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_risk_is_always_allowed() {
        assert_eq!(decide(RiskLevel::Low, StandingGrant::None), Decision::Allow);
    }

    #[test]
    fn medium_and_high_ask_without_a_grant() {
        assert_eq!(
            decide(RiskLevel::Medium, StandingGrant::None),
            Decision::Ask
        );
        assert_eq!(decide(RiskLevel::High, StandingGrant::None), Decision::Ask);
    }

    #[test]
    fn medium_and_high_are_allowed_with_a_workspace_grant() {
        assert_eq!(
            decide(RiskLevel::Medium, StandingGrant::WorkspaceAllowed),
            Decision::Allow
        );
        assert_eq!(
            decide(RiskLevel::High, StandingGrant::WorkspaceAllowed),
            Decision::Allow
        );
    }

    #[test]
    fn critical_is_denied_even_with_a_grant() {
        // There is no grant that unlocks Critical in v1 — a caller can't accidentally
        // persist one, because StandingGrant is only ever looked up for Medium/High.
        assert_eq!(
            decide(RiskLevel::Critical, StandingGrant::WorkspaceAllowed),
            Decision::Deny
        );
        assert_eq!(
            decide(RiskLevel::Critical, StandingGrant::None),
            Decision::Deny
        );
    }

    #[test]
    fn known_capabilities_map_to_their_documented_risk() {
        assert_eq!(capability_risk("filesystem.read.workspace"), RiskLevel::Low);
        assert_eq!(
            capability_risk("filesystem.write.workspace"),
            RiskLevel::Medium
        );
        assert_eq!(capability_risk("git.push"), RiskLevel::High);
        assert_eq!(
            capability_risk("filesystem.write.outside_workspace"),
            RiskLevel::Critical
        );
    }

    #[test]
    fn unknown_capability_defaults_to_medium_not_low() {
        assert_eq!(capability_risk("some.future.capability"), RiskLevel::Medium);
    }

    #[test]
    fn classifies_known_shell_commands() {
        assert_eq!(classify_shell_command("git status"), RiskLevel::Low);
        assert_eq!(classify_shell_command("npm install"), RiskLevel::Medium);
        assert_eq!(
            classify_shell_command("git push origin main"),
            RiskLevel::High
        );
        assert_eq!(classify_shell_command("rm -rf /"), RiskLevel::Critical);
    }

    #[test]
    fn unmatched_shell_command_defaults_to_medium() {
        assert_eq!(
            classify_shell_command("some-custom-script.sh --deploy"),
            RiskLevel::Medium
        );
    }
}
