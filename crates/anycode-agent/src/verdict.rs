//! Whether a task's work actually passed (docs/ARCHITECTURE.md invariant #12).
//!
//! Not every command an agent runs is a check. `grep` exits 1 when it finds nothing;
//! `cat` of a file the agent is about to create fails by design. Counting those as
//! failed verification would punish exploration, so the model marks which commands are
//! meant to verify the work — and the runtime alone decides whether they passed, from
//! the exit codes it recorded. The model can choose not to verify; it cannot make a
//! failing check pass. A task that verified nothing is reported as exactly that.

use serde::Serialize;

/// One command the agent ran, with the exit code the runtime observed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRecord {
    pub command: String,
    /// `None` only when the platform reported no code — the process was killed.
    pub exit_code: Option<i64>,
    /// The agent ran this to check its work (build, lint, test), not to look around.
    pub verification: bool,
}

impl CommandRecord {
    fn passed(&self) -> bool {
        self.exit_code == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Verdict {
    /// Every verification command's most recent run exited 0.
    Passed { checks: Vec<String> },
    /// At least one verification command's most recent run did not exit 0.
    Failed { failing: Vec<CommandRecord> },
    /// No verification command ran. Not a failure, and never presented as a success.
    Unverified,
}

/// Judges verification by each check's *latest* run: a test that failed, was fixed, and
/// then passed has passed. Judging by any failure would mark the ordinary
/// fix-and-rerun loop as failed.
pub fn verdict(commands: &[CommandRecord]) -> Verdict {
    let mut latest: Vec<&CommandRecord> = Vec::new();
    for record in commands.iter().filter(|c| c.verification) {
        match latest.iter_mut().find(|c| c.command == record.command) {
            Some(slot) => *slot = record,
            None => latest.push(record),
        }
    }
    if latest.is_empty() {
        return Verdict::Unverified;
    }
    let failing: Vec<CommandRecord> = latest
        .iter()
        .filter(|c| !c.passed())
        .map(|c| (*c).clone())
        .collect();
    if failing.is_empty() {
        Verdict::Passed {
            checks: latest.iter().map(|c| c.command.clone()).collect(),
        }
    } else {
        Verdict::Failed { failing }
    }
}

/// What the runtime measured about a task besides its commands. A struct rather than
/// two positional bools, which are too easy to pass the wrong way round.
#[derive(Debug, Clone, Copy, Default)]
pub struct Observed {
    /// `git status` shows files the task changed.
    pub files_changed: bool,
    /// The model called at least one tool during the task.
    pub tools_used: bool,
}

/// What the runtime tells the model when it stops with its work unproven, or `None`
/// when there is nothing to push back on. The loop sends this at most a bounded number
/// of times, so a model that cannot make a check pass ends as failed, not in a loop.
pub fn replan_prompt(verdict: &Verdict, observed: Observed) -> Option<String> {
    let Observed {
        files_changed,
        tools_used,
    } = observed;
    match verdict {
        Verdict::Failed { failing } => {
            let list = failing
                .iter()
                .map(|c| match c.exit_code {
                    Some(code) => format!("- `{}` exited {code}", c.command),
                    None => format!("- `{}` was killed before it exited", c.command),
                })
                .collect::<Vec<_>>()
                .join("\n");
            // Observed in a live run: a model that describes a fix in prose believes it
            // has made the change. The git delta says otherwise, so say that — it is a
            // measurement, not a hint about what the fix should be.
            let unchanged = if files_changed {
                ""
            } else {
                "\n\nNo file in the repository has changed. Describing a fix does not \
                 apply it: make the change with filesystem.edit.workspace, then run the \
                 check again."
            };
            Some(format!(
                "Any Code runtime: verification has not passed. The latest run of each \
                 check was:\n{list}{unchanged}\n\nFix the cause and run the check again, \
                 or explain plainly why it cannot pass."
            ))
        }
        Verdict::Unverified if files_changed => Some(
            "Any Code runtime: you changed files but ran no verification. Run the \
             project's own build, lint or test command with shell.execute and \
             verify: true, or state plainly that the change cannot be verified."
                .to_string(),
        ),
        // Observed in a live run: a model answered an implementation task in prose
        // without touching a tool. The runtime can't tell that from a question being
        // answered, but an agent that never looked at the workspace gets one push.
        Verdict::Unverified if !tools_used => Some(
            "Any Code runtime: you have not used any tool, so nothing in the repository \
             has been inspected or changed. If the task asks for a change, make it with \
             the tools and verify it; if it only asks a question, read the relevant files \
             and then answer."
                .to_string(),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(command: &str, exit_code: Option<i64>) -> CommandRecord {
        CommandRecord {
            command: command.into(),
            exit_code,
            verification: true,
        }
    }

    fn explore(command: &str, exit_code: Option<i64>) -> CommandRecord {
        CommandRecord {
            verification: false,
            ..check(command, exit_code)
        }
    }

    #[test]
    fn a_check_that_failed_then_passed_has_passed() {
        let runs = [check("cargo test", Some(101)), check("cargo test", Some(0))];
        assert_eq!(
            verdict(&runs),
            Verdict::Passed {
                checks: vec!["cargo test".into()]
            }
        );
    }

    #[test]
    fn a_check_that_passed_then_failed_has_failed() {
        let runs = [check("npm test", Some(0)), check("npm test", Some(1))];
        assert_eq!(
            verdict(&runs),
            Verdict::Failed {
                failing: vec![check("npm test", Some(1))]
            }
        );
    }

    #[test]
    fn exploration_never_counts_either_way() {
        // grep exiting 1 means "no match", not "the work is broken".
        let runs = [
            explore("grep -r TODO .", Some(1)),
            check("make test", Some(0)),
        ];
        assert!(matches!(verdict(&runs), Verdict::Passed { .. }));
        assert_eq!(verdict(&[explore("ls", Some(0))]), Verdict::Unverified);
    }

    #[test]
    fn one_failing_check_fails_the_task_even_if_others_pass() {
        let runs = [
            check("cargo build", Some(0)),
            check("cargo test", Some(101)),
        ];
        assert!(matches!(verdict(&runs), Verdict::Failed { failing } if failing.len() == 1));
    }

    #[test]
    fn a_killed_check_is_a_failure_not_a_pass() {
        assert!(matches!(
            verdict(&[check("pytest", None)]),
            Verdict::Failed { .. }
        ));
    }

    #[test]
    fn nothing_run_is_unverified() {
        assert_eq!(verdict(&[]), Verdict::Unverified);
    }

    const EDITED: Observed = Observed {
        files_changed: true,
        tools_used: true,
    };
    const LOOKED_ONLY: Observed = Observed {
        files_changed: false,
        tools_used: true,
    };
    const NOTHING: Observed = Observed {
        files_changed: false,
        tools_used: false,
    };

    #[test]
    fn failing_checks_are_pushed_back_and_an_empty_delta_is_named() {
        let failed = verdict(&[check("cargo test", Some(101))]);
        let prompt = replan_prompt(&failed, EDITED).unwrap();
        assert!(prompt.contains("`cargo test` exited 101"));
        // Only when the git delta is empty does the runtime say nothing was changed.
        assert!(!prompt.contains("No file in the repository has changed"));
        let untouched = replan_prompt(&failed, LOOKED_ONLY).unwrap();
        assert!(untouched.contains("No file in the repository has changed"));
    }

    #[test]
    fn unverified_work_is_pushed_back_only_when_something_suggests_work_was_owed() {
        // Edits nobody checked.
        assert!(replan_prompt(&Verdict::Unverified, EDITED).is_some());
        // Never touched a tool at all.
        let idle = replan_prompt(&Verdict::Unverified, NOTHING).unwrap();
        assert!(idle.contains("not used any tool"));
        // Read files and answered without changing anything: a question, left alone.
        assert!(replan_prompt(&Verdict::Unverified, LOOKED_ONLY).is_none());
    }

    #[test]
    fn passed_work_is_never_pushed_back() {
        let passed = verdict(&[check("cargo test", Some(0))]);
        for observed in [EDITED, LOOKED_ONLY, NOTHING] {
            assert!(replan_prompt(&passed, observed).is_none());
        }
    }

    #[test]
    fn serializes_with_a_kind_tag_for_the_frontend() {
        let json = serde_json::to_value(verdict(&[check("x", Some(2))])).unwrap();
        assert_eq!(json["kind"], "failed");
        assert_eq!(json["failing"][0]["exitCode"], 2);
    }
}
