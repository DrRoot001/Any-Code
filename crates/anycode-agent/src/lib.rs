//! The agent runtime's decisions (PRD §30): what state a task is in, what plan it is
//! following, and whether what it did actually worked.
//!
//! Kept free of Tauri and of every provider (docs/ARCHITECTURE.md: `anycode-agent`
//! orchestrates but does not know provider names), so each rule here is tested
//! directly. The loop that drives these decisions lives in the desktop app because it
//! needs the app's event channels and approval prompts; it asks this crate for every
//! judgement rather than making its own.

mod plan;
mod state;
mod verdict;

pub use plan::{parse_plan, PLANNER_INSTRUCTION};
pub use state::{InvalidTransition, TaskMachine, TaskState};
pub use verdict::{is_known_check, replan_prompt, verdict, CommandRecord, Observed, Verdict};
