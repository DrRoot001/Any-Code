//! The task state machine (PRD §30).
//!
//! PRD §30 also lists `waiting` and `blocked`. Both describe one task depending on
//! another — the task DAG of Phase 5 — and nothing in a single-agent runtime can enter
//! them. A state no code path reaches would be simulated state (PRD §100), so they are
//! added when the scheduler that produces them is.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Created,
    Planning,
    Running,
    AwaitingApproval,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// The only legal moves. Failing and cancelling are possible from anywhere live;
    /// finishing is possible only through `Verifying`, so no path marks a task complete
    /// without the runtime having looked at the evidence first.
    pub fn can_become(self, next: TaskState) -> bool {
        use TaskState::*;
        if self.is_terminal() {
            return false;
        }
        if matches!(next, Failed | Cancelled) {
            return true;
        }
        matches!(
            (self, next),
            (Created, Planning)
                | (Planning, Running)
                | (Running, AwaitingApproval)
                | (AwaitingApproval, Running)
                | (Running, Verifying)
                // Replan: verification found a problem and the model gets another go.
                | (Verifying, Running)
                | (Verifying, Completed)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a task cannot move from {from:?} to {to:?}")]
pub struct InvalidTransition {
    pub from: TaskState,
    pub to: TaskState,
}

/// One task's current state. The only way to change it is [`TaskMachine::advance`], so
/// an illegal transition is an error at the call site rather than a quietly wrong UI.
#[derive(Debug)]
pub struct TaskMachine {
    state: TaskState,
}

impl Default for TaskMachine {
    fn default() -> Self {
        Self {
            state: TaskState::Created,
        }
    }
}

impl TaskMachine {
    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn advance(&mut self, next: TaskState) -> Result<TaskState, InvalidTransition> {
        if !self.state.can_become(next) {
            return Err(InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::TaskState::*;
    use super::*;

    fn run(steps: &[TaskState]) -> Result<TaskState, InvalidTransition> {
        let mut machine = TaskMachine::default();
        for &step in steps {
            machine.advance(step)?;
        }
        Ok(machine.state())
    }

    #[test]
    fn a_task_with_an_approval_and_a_replan_completes() {
        let path = [
            Planning,
            Running,
            AwaitingApproval,
            Running,
            Verifying,
            Running,
            Verifying,
            Completed,
        ];
        assert_eq!(run(&path), Ok(Completed));
    }

    #[test]
    fn nothing_completes_without_passing_through_verification() {
        for from in [Created, Planning, Running, AwaitingApproval] {
            assert!(!from.can_become(Completed), "{from:?} -> Completed");
        }
    }

    #[test]
    fn a_task_cannot_skip_planning() {
        assert_eq!(
            run(&[Running]),
            Err(InvalidTransition {
                from: Created,
                to: Running
            })
        );
    }

    #[test]
    fn any_live_state_can_fail_or_be_cancelled() {
        for from in [Created, Planning, Running, AwaitingApproval, Verifying] {
            assert!(from.can_become(Failed), "{from:?} -> Failed");
            assert!(from.can_become(Cancelled), "{from:?} -> Cancelled");
        }
    }

    #[test]
    fn terminal_states_are_final() {
        for from in [Completed, Failed, Cancelled] {
            for to in [Planning, Running, Verifying, Completed, Failed, Cancelled] {
                assert!(!from.can_become(to), "{from:?} -> {to:?}");
            }
        }
    }

    #[test]
    fn serializes_as_the_snake_case_the_frontend_matches_on() {
        assert_eq!(
            serde_json::to_string(&AwaitingApproval).unwrap(),
            "\"awaiting_approval\""
        );
    }
}
