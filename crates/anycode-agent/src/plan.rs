//! The planner (PRD §30: Intent → Planner → …). Before a task touches anything, the
//! model is asked for a short plan with no tools available, so the user sees what the
//! agent intends before it starts acting, and the plan anchors the steps that follow.
//!
//! A plan is model output: it is shown to the user and followed by the model, but it
//! authorises nothing. Every step still crosses the permission gate as it runs.

/// Appended to the task instruction for the planning request.
pub const PLANNER_INSTRUCTION: &str = "Before doing anything, write a short plan for the \
    task above: a numbered list of at most 7 concrete steps, ending with how you will \
    verify the result. Do not call tools, do not write code, and write nothing except \
    the numbered list.";

/// Most steps kept. A model that ignores "at most 7" shouldn't flood the timeline.
const MAX_STEPS: usize = 12;

/// Pulls the steps out of a planner reply. Accepts `1.`, `1)`, `-`, `*` and `•` list
/// markers, since models vary; drops everything else (preambles, sign-offs). Returns an
/// empty list when the reply contained no list — the caller shows the raw text instead
/// of pretending there was a structured plan.
pub fn parse_plan(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            // Bold markers mean nothing in a plain-text timeline, and a model that wraps
            // a whole step (`**1. Read it**`) would otherwise hide the list marker.
            let line = line.replace("**", "");
            let line = line.trim();
            let rest = if let Some(rest) = line.strip_prefix(['-', '*', '•']) {
                rest
            } else {
                let digits =
                    line.len() - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
                if digits == 0 {
                    return None;
                }
                line[digits..].strip_prefix(['.', ')'])?
            };
            let step = rest.trim();
            (!step.is_empty()).then(|| step.to_string())
        })
        .take(MAX_STEPS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_numbered_plan_and_drops_the_preamble() {
        let reply = "Here is my plan:\n1. Read src/lib.rs\n2) Add the function\n3. Run `cargo test`\nLet me know!";
        assert_eq!(
            parse_plan(reply),
            ["Read src/lib.rs", "Add the function", "Run `cargo test`"]
        );
    }

    #[test]
    fn parses_bullets_and_strips_markdown_bold() {
        assert_eq!(
            parse_plan("- **Inspect** the file\n* Edit it\n• Verify\n**4. Report**"),
            ["Inspect the file", "Edit it", "Verify", "Report"]
        );
    }

    #[test]
    fn a_reply_with_no_list_yields_no_steps() {
        assert!(parse_plan("I will just do it.").is_empty());
        // A number that isn't a list marker isn't a step.
        assert!(parse_plan("2024 was a good year").is_empty());
    }

    #[test]
    fn an_overlong_plan_is_capped() {
        let reply: String = (1..=30).map(|n| format!("{n}. step\n")).collect();
        assert_eq!(parse_plan(&reply).len(), MAX_STEPS);
    }
}
