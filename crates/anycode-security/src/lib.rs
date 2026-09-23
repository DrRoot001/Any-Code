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
/// override path for it at all. High is asked **every time**: a standing grant never
/// covers it, so one "always allow" can't quietly authorise a push or a secret read later.
/// Medium runs unattended only under a standing grant for that exact scope.
pub fn decide(risk: RiskLevel, grant: StandingGrant) -> Decision {
    match risk {
        RiskLevel::Critical => Decision::Deny,
        RiskLevel::High => Decision::Ask,
        RiskLevel::Medium if grant == StandingGrant::WorkspaceAllowed => Decision::Allow,
        RiskLevel::Medium => Decision::Ask,
        RiskLevel::Low => Decision::Allow,
    }
}

/// Whether "always allow" may be offered — and persisted — for this risk. Only Medium:
/// Low never asks, High must be asked each time, Critical never runs.
pub fn standing_grant_permitted(risk: RiskLevel) -> bool {
    risk == RiskLevel::Medium
}

/// Why a path is sensitive. Returned alongside the risk so the approval prompt can say
/// what is actually at stake, instead of a generic line per risk level.
pub type RiskReason = &'static str;

/// Template files are meant to be read and committed; they hold placeholders, not values.
const ENV_TEMPLATE_SUFFIXES: &[&str] = &[".example", ".sample", ".template", ".dist"];

/// Risk that a path carries by itself, regardless of which tool touches it (PRD §52: an
/// agent must not automatically receive a complete .env, SSH private keys, or cloud
/// credentials). `None` for an ordinary file. Matching is on the file name and the
/// directories above it, case-insensitively; it is a denylist of well-known secret
/// locations, not a content scanner — a secret pasted into `notes.txt` is not caught.
pub fn path_risk(path: &str) -> Option<(RiskLevel, RiskReason)> {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let parts: Vec<&str> = lower
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let name = *parts.last()?;
    let in_dir = |dir: &str| parts[..parts.len() - 1].contains(&dir);

    // Private key material never goes to a model, whoever asks.
    let ssh_key = ["id_rsa", "id_dsa", "id_ecdsa", "id_ed25519"]
        .iter()
        .any(|k| {
            name == *k
                || (name.starts_with(&format!("{k}_")) || name.starts_with(&format!("{k}-")))
                    && !name.ends_with(".pub")
        });
    let key_file = [".key", ".p12", ".pfx", ".jks", ".keystore"]
        .iter()
        .any(|ext| name.ends_with(ext));
    if ssh_key || key_file {
        return Some((RiskLevel::Critical, "private key material"));
    }

    let env_file = (name == ".env" || name.starts_with(".env.") || name.ends_with(".env"))
        && !ENV_TEMPLATE_SUFFIXES
            .iter()
            .any(|suffix| name.ends_with(suffix));
    if env_file {
        return Some((RiskLevel::High, "environment file — usually holds secrets"));
    }
    let credential_file = matches!(
        name,
        ".npmrc"
            | ".pypirc"
            | ".netrc"
            | ".git-credentials"
            | ".dockercfg"
            | "credentials"
            | "credentials.json"
            | "secrets.json"
            | "secrets.yaml"
            | "secrets.yml"
    ) || name.ends_with(".pem")
        || name.ends_with(".tfstate");
    if credential_file {
        return Some((RiskLevel::High, "credential file"));
    }
    if in_dir(".ssh") || in_dir(".aws") || in_dir(".gnupg") || in_dir(".kube") {
        return Some((RiskLevel::High, "credential directory"));
    }
    // A remote URL in .git/config can carry an access token.
    if in_dir(".git") && name == "config" {
        return Some((
            RiskLevel::High,
            "git configuration — remote URLs can embed tokens",
        ));
    }
    None
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

/// Why a shell command is riskier than its verb suggests, if it is: it names a
/// secret-bearing path, or prints the environment. `None` otherwise.
pub fn shell_risk_reason(command: &str) -> Option<(RiskLevel, RiskReason)> {
    let tokens: Vec<&str> = command
        .split(|c: char| c.is_whitespace() || "|;&<>()'\"=`$".contains(c))
        .filter(|t| !t.is_empty())
        .collect();
    let worst_path = tokens
        .iter()
        .filter_map(|t| path_risk(t))
        .max_by_key(|(risk, _)| *risk);
    if worst_path.is_some() {
        return worst_path;
    }
    if tokens.iter().any(|t| matches!(*t, "env" | "printenv")) {
        return Some((
            RiskLevel::High,
            "prints environment variables, which can hold secrets",
        ));
    }
    None
}

/// Classifies a shell command by matching against small allow/deny lists (docs/SECURITY.md
/// §51's examples), defaulting unmatched commands to Medium. A command that touches a
/// secret-bearing path or the environment is raised to that risk (see
/// [`shell_risk_reason`]).
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
    if let Some((risk, _)) = shell_risk_reason(trimmed) {
        return risk;
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
    fn a_workspace_grant_covers_medium_only() {
        assert_eq!(
            decide(RiskLevel::Medium, StandingGrant::WorkspaceAllowed),
            Decision::Allow
        );
        // Audit S2: a grant used to unlock High too, so "always allow" on one command
        // silently authorised pushes. High is asked every time.
        assert_eq!(
            decide(RiskLevel::High, StandingGrant::WorkspaceAllowed),
            Decision::Ask
        );
        assert!(standing_grant_permitted(RiskLevel::Medium));
        for risk in [RiskLevel::Low, RiskLevel::High, RiskLevel::Critical] {
            assert!(!standing_grant_permitted(risk), "{risk:?}");
        }
    }

    #[test]
    fn secret_bearing_paths_are_not_ordinary_files() {
        use RiskLevel::*;
        let risk = |p| path_risk(p).map(|(r, _)| r);
        // Audit S1: these were Low and read without a prompt.
        for p in [
            ".env",
            ".env.production",
            "apps/web/.env.local",
            ".env.vps",
            "prod.env",
        ] {
            assert_eq!(risk(p), Some(High), "{p}");
        }
        for p in [
            "id_rsa",
            "home/.ssh/id_ed25519",
            "certs/server.key",
            "keys/app.p12",
        ] {
            assert_eq!(risk(p), Some(Critical), "{p}");
        }
        for p in [
            ".npmrc",
            ".git/config",
            "infra/terraform.tfstate",
            ".aws/config",
            "TLS/cert.PEM",
        ] {
            assert_eq!(risk(p), Some(High), "{p}");
        }
        // Templates, public keys and ordinary files stay ordinary.
        for p in [
            ".env.example",
            ".env.sample",
            "id_rsa.pub",
            "src/main.rs",
            "environment.md",
            "config",
        ] {
            assert_eq!(risk(p), None, "{p}");
        }
    }

    #[test]
    fn shell_commands_touching_secrets_are_raised() {
        assert_eq!(classify_shell_command("cat .env"), RiskLevel::High);
        assert_eq!(
            classify_shell_command("grep KEY .env.production | head"),
            RiskLevel::High
        );
        assert_eq!(
            classify_shell_command("cat ~/.ssh/id_rsa"),
            RiskLevel::Critical
        );
        assert_eq!(classify_shell_command("printenv"), RiskLevel::High);
        assert_eq!(classify_shell_command("FOO=1 env | sort"), RiskLevel::High);
        // Words that merely contain the names are not matches.
        assert_eq!(
            classify_shell_command("cat environment.md"),
            RiskLevel::Medium
        );
        assert_eq!(classify_shell_command("npm test"), RiskLevel::Low);
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
