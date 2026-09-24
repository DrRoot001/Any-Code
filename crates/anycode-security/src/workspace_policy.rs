//! Restrictions a repository declares for itself in `.anycode/permissions.yaml` (PRD §39).
//!
//! The file is repository content — untrusted (ADR 0004) — so it is honoured **only in the
//! tightening direction**. It can protect paths and require approval for capabilities; it
//! can never make anything less restricted than the runtime already decided. A cloned
//! repository therefore cannot use it to grant itself anything.
//!
//! ```yaml
//! protected:
//!   files:
//!     - .env.production
//!     - migrations/production/**
//! approval_required:
//!   - git.push
//!   - shell.execute
//! ```

use crate::RiskLevel;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

/// The file's location inside a workspace.
pub const POLICY_FILE: &str = ".anycode/permissions.yaml";

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("{POLICY_FILE} is not valid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("{POLICY_FILE}: `{pattern}` is not a valid path pattern: {message}")]
    Pattern { pattern: String, message: String },
}

#[derive(Deserialize, Default)]
struct PolicyFile {
    #[serde(default)]
    protected: Protected,
    #[serde(default)]
    approval_required: Vec<String>,
}

#[derive(Deserialize, Default)]
struct Protected {
    #[serde(default)]
    files: Vec<String>,
}

/// What a call does to the path it names — protection means different things for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathAccess {
    Read,
    Write,
}

#[derive(Debug, Default)]
pub struct WorkspacePolicy {
    protected: Option<GlobSet>,
    approval_required: Vec<String>,
}

impl WorkspacePolicy {
    /// An empty file, or one without these sections, restricts nothing.
    pub fn parse(yaml: &str) -> Result<Self, PolicyError> {
        let file: PolicyFile = if yaml.trim().is_empty() {
            PolicyFile::default()
        } else {
            serde_yaml::from_str(yaml)?
        };
        let protected = if file.protected.files.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pattern in &file.protected.files {
                let glob = Glob::new(pattern.trim_start_matches("./")).map_err(|e| {
                    PolicyError::Pattern {
                        pattern: pattern.clone(),
                        message: e.to_string(),
                    }
                })?;
                builder.add(glob);
            }
            Some(builder.build().map_err(|e| PolicyError::Pattern {
                pattern: file.protected.files.join(", "),
                message: e.to_string(),
            })?)
        };
        Ok(Self {
            protected,
            approval_required: file.approval_required,
        })
    }

    /// Whether `path` (workspace-relative, `/`-separated) is protected.
    pub fn is_protected(&self, path: &str) -> bool {
        self.protected
            .as_ref()
            .is_some_and(|set| set.is_match(path.trim_start_matches("./")))
    }

    /// The risk after this workspace's restrictions, and why it was raised. **Never lower
    /// than `risk`.** Writing a protected path is Critical — refused outright; reading one,
    /// or using a capability that requires approval, is at least High — asked every time,
    /// never covered by "always allow".
    pub fn apply(
        &self,
        capability: &str,
        path: Option<(&str, PathAccess)>,
        risk: RiskLevel,
    ) -> (RiskLevel, Option<String>) {
        let mut raised = risk;
        let mut reason = None;
        if let Some((path, access)) = path {
            // The agent must not be able to rewrite the rules that restrict it — that would
            // let one task loosen the next. Protected whether or not the file exists yet.
            if access == PathAccess::Write && path.trim_start_matches("./") == POLICY_FILE {
                return (
                    RiskLevel::Critical,
                    Some(format!("{POLICY_FILE} holds this workspace's permission rules")),
                );
            }
            if self.is_protected(path) {
                let floor = match access {
                    PathAccess::Write => RiskLevel::Critical,
                    PathAccess::Read => RiskLevel::High,
                };
                if floor > raised {
                    raised = floor;
                    reason = Some(format!("{path} is protected by {POLICY_FILE}"));
                }
            }
        }
        if self.approval_required.iter().any(|c| c == capability) && RiskLevel::High > raised {
            raised = RiskLevel::High;
            reason = Some(format!("{capability} requires approval in {POLICY_FILE}"));
        }
        (raised, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RiskLevel::*;

    const POLICY: &str = "
protected:
  files:
    - .env.production
    - migrations/production/**
approval_required:
  - git.push
  - shell.execute
";

    #[test]
    fn writing_a_protected_path_is_refused_and_reading_one_is_asked() {
        let policy = WorkspacePolicy::parse(POLICY).unwrap();
        let write = policy.apply(
            "filesystem.write.workspace",
            Some(("migrations/production/001.sql", PathAccess::Write)),
            Medium,
        );
        assert_eq!(write.0, Critical);
        assert!(write.1.unwrap().contains("protected"));
        let read = policy.apply(
            "filesystem.read.workspace",
            Some(("migrations/production/001.sql", PathAccess::Read)),
            Low,
        );
        assert_eq!(read.0, High);
        // Not protected: the runtime's own decision stands.
        let other = policy.apply(
            "filesystem.write.workspace",
            Some(("migrations/staging/001.sql", PathAccess::Write)),
            Medium,
        );
        assert_eq!(other, (Medium, None));
    }

    #[test]
    fn the_agent_cannot_rewrite_its_own_rules() {
        for policy in [WorkspacePolicy::default(), WorkspacePolicy::parse(POLICY).unwrap()] {
            let (risk, reason) = policy.apply(
                "filesystem.write.workspace",
                Some((POLICY_FILE, PathAccess::Write)),
                Medium,
            );
            assert_eq!(risk, Critical);
            assert!(reason.is_some());
        }
    }

    #[test]
    fn a_listed_capability_is_asked_every_time() {
        let policy = WorkspacePolicy::parse(POLICY).unwrap();
        assert_eq!(policy.apply("shell.execute", None, Low).0, High);
        assert_eq!(policy.apply("git.status", None, Low), (Low, None));
    }

    #[test]
    fn a_policy_can_never_lower_risk() {
        // The property that makes an untrusted file safe to honour at all.
        let policy = WorkspacePolicy::parse(POLICY).unwrap();
        for risk in [Low, Medium, High, Critical] {
            for (cap, path) in [
                ("shell.execute", None),
                (
                    "filesystem.read.workspace",
                    Some(("src/lib.rs", PathAccess::Read)),
                ),
                (
                    "filesystem.write.workspace",
                    Some((".env.production", PathAccess::Write)),
                ),
            ] {
                assert!(policy.apply(cap, path, risk).0 >= risk, "{cap} {risk:?}");
            }
        }
    }

    #[test]
    fn an_empty_or_unrelated_file_restricts_nothing() {
        for yaml in ["", "   \n", "project:\n  package_manager: pnpm\n"] {
            let policy = WorkspacePolicy::parse(yaml).unwrap();
            assert_eq!(policy.apply("shell.execute", None, Low), (Low, None));
            assert!(!policy.is_protected(".env.production"));
        }
    }

    #[test]
    fn malformed_input_is_an_error_not_an_empty_policy() {
        // The caller fails closed on this: a restriction the user wrote must not be
        // silently dropped because it didn't parse.
        assert!(WorkspacePolicy::parse("protected: [unclosed").is_err());
        assert!(WorkspacePolicy::parse("protected:\n  files:\n    - 'a[b'\n").is_err());
    }
}
