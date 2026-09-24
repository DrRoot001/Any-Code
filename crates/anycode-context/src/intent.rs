//! Intent extraction (PRD §37, first stage): which files, symbols and words an instruction
//! is about. Deliberately plain text processing — no model call — so it is fast, free,
//! deterministic and testable.

use serde::Serialize;
use std::collections::BTreeSet;

const MAX_TERMS: usize = 12;
const MAX_PATH_MATCHES: usize = 5;

/// Words that say what to do rather than what to do it to.
const STOPWORDS: &[&str] = &[
    "the",
    "and",
    "for",
    "with",
    "that",
    "this",
    "from",
    "into",
    "make",
    "sure",
    "should",
    "would",
    "could",
    "please",
    "implement",
    "add",
    "fix",
    "update",
    "change",
    "create",
    "file",
    "files",
    "code",
    "function",
    "method",
    "run",
    "runs",
    "passes",
    "pass",
    "using",
    "use",
    "when",
    "what",
    "which",
    "will",
    "have",
    "has",
    "not",
    "all",
    "any",
    "can",
    "need",
    "needs",
    "does",
    "doesn",
    "don",
    "its",
    "then",
    "than",
    "also",
    "there",
    "their",
    "them",
    "you",
    "your",
    "our",
    "way",
    "work",
    "works",
    "working",
    "new",
    "via",
    "like",
    "just",
];

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Intent {
    /// Repository files the instruction names, as indexed paths.
    pub paths: Vec<String>,
    /// Words that look like code identifiers: `backticked`, CamelCase, snake_case, `call()`.
    pub identifiers: Vec<String>,
    /// Other meaningful words, lowercased, for full-text search.
    pub terms: Vec<String>,
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn looks_like_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    let starts_well = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_');
    starts_well && word.chars().all(is_identifier_char) && word.len() >= 2
}

fn is_code_shaped(word: &str) -> bool {
    let has_lower = word.chars().any(|c| c.is_lowercase());
    let has_upper = word.chars().skip(1).any(|c| c.is_uppercase());
    word.contains('_') || (has_lower && has_upper)
}

/// `known` is every indexed path; a path in the instruction is matched exactly, or by file
/// name when that name is unique enough to mean something (at most a few matches).
pub fn extract_intent(instruction: &str, known: &BTreeSet<&str>) -> Intent {
    let mut paths: Vec<String> = Vec::new();
    let mut identifiers: Vec<String> = Vec::new();
    let mut terms: Vec<String> = Vec::new();
    let push = |list: &mut Vec<String>, value: String| {
        if !list.contains(&value) {
            list.push(value);
        }
    };

    // `backticked` spans are the strongest signal of a code name.
    for (i, part) in instruction.split('`').enumerate() {
        let part = part.trim().trim_end_matches("()");
        if i % 2 == 1 && looks_like_identifier(part) {
            push(&mut identifiers, part.to_string());
        }
    }

    for raw in instruction.split(|c: char| c.is_whitespace() || "`,;\"'()[]{}<>".contains(c)) {
        let token = raw.trim_matches(|c: char| ".:!?".contains(c));
        // `src/lib.rs:42` names the file.
        let token = token.split(':').next().unwrap_or(token);
        if token.is_empty() {
            continue;
        }

        if token.contains('.') || token.contains('/') {
            let normalised = token.trim_start_matches("./");
            if known.contains(normalised) {
                push(&mut paths, normalised.to_string());
                continue;
            }
            let by_name: Vec<&&str> = known
                .iter()
                .filter(|p| p.rsplit('/').next() == Some(normalised))
                .collect();
            if !by_name.is_empty() && by_name.len() <= MAX_PATH_MATCHES {
                for p in by_name {
                    push(&mut paths, p.to_string());
                }
                continue;
            }
        }

        if looks_like_identifier(token) {
            if is_code_shaped(token) {
                push(&mut identifiers, token.to_string());
            } else {
                let lower = token.to_lowercase();
                if lower.len() >= 3
                    && !STOPWORDS.contains(&lower.as_str())
                    && !identifiers.iter().any(|i| i.eq_ignore_ascii_case(&lower))
                {
                    push(&mut terms, lower);
                }
            }
        }
    }
    terms.truncate(MAX_TERMS);
    Intent {
        paths,
        identifiers,
        terms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known() -> BTreeSet<&'static str> {
        [
            "calc.py",
            "test_calc.py",
            "src/lib.rs",
            "src/agent/state.rs",
            "web/src/App.tsx",
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn finds_named_files_backticked_names_and_words() {
        let intent = extract_intent(
            "Implement the `multiply` function in calc.py so that the test suite passes.",
            &known(),
        );
        assert_eq!(intent.paths, ["calc.py"]);
        assert_eq!(intent.identifiers, ["multiply"]);
        assert!(intent.terms.contains(&"test".to_string()));
        assert!(intent.terms.contains(&"suite".to_string()));
        // Words that say what to do are not search terms.
        for stop in ["implement", "function", "passes", "the"] {
            assert!(!intent.terms.contains(&stop.to_string()), "{stop}");
        }
        // A backticked identifier is not also a plain term.
        assert!(!intent.terms.contains(&"multiply".to_string()));
    }

    #[test]
    fn recognises_code_shaped_words_without_backticks() {
        let intent = extract_intent("Why does TaskMachine reject run_task calls?", &known());
        assert!(intent.identifiers.contains(&"TaskMachine".to_string()));
        assert!(intent.identifiers.contains(&"run_task".to_string()));
    }

    #[test]
    fn matches_a_path_by_exact_path_by_name_and_with_a_line_suffix() {
        let intent = extract_intent(
            "Look at src/lib.rs:42 and App.tsx, then state.rs.",
            &known(),
        );
        assert_eq!(
            intent.paths,
            ["src/lib.rs", "web/src/App.tsx", "src/agent/state.rs"]
        );
    }

    #[test]
    fn a_path_that_is_not_in_the_repository_is_not_invented() {
        let intent = extract_intent("Create docs/NEW.md describing it.", &known());
        assert!(intent.paths.is_empty());
    }

    #[test]
    fn terms_are_deduplicated_and_capped() {
        let words = (0..40)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let intent = extract_intent(&format!("{words} word1 word1"), &known());
        assert_eq!(intent.terms.len(), MAX_TERMS);
        let unique: BTreeSet<_> = intent.terms.iter().collect();
        assert_eq!(unique.len(), intent.terms.len());
    }
}
