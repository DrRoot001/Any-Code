//! The context builder (PRD §37, ADR 0004): turns a task's instruction into a *targeted*
//! context package — the passages an agent needs, each with the reason it was chosen —
//! instead of dumping files.
//!
//! Stages: intent → symbols → lexical → structural → working tree → rerank → token budget.
//! Everything that scored but did not fit is kept as `excluded`, with its reason, so the
//! Context Inspector can show what the agent did *not* see as well as what it did.
//!
//! Token counts are **estimates** (bytes ÷ 4) and are labelled so wherever they are shown;
//! only a provider can report real counts.

use anycode_code_intelligence::{Index, IndexError};
use anycode_core::trust::Tagged;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

mod intent;
pub use intent::{extract_intent, Intent};

/// Default token budget for a context package. Conservative on purpose: it must leave room
/// for the system prompt, tools and the conversation inside a small local model's window.
pub const DEFAULT_BUDGET_TOKENS: u32 = 3000;

/// A named file this long is outlined, not included whole.
const WHOLE_FILE_MAX_LINES: u32 = 200;
/// A definition longer than this is included from its start only.
const DEFINITION_MAX_LINES: u32 = 120;
const LEXICAL_HITS: usize = 20;
const DEFINITIONS_PER_NAME: usize = 5;

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error(transparent)]
    Index(#[from] IndexError),
}

/// Why a passage is in the package. More than one can apply.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reason {
    /// The instruction names this file.
    NamedInInstruction,
    /// It defines a symbol the instruction mentions.
    DefinesSymbol { symbol: String },
    /// Full-text match on terms from the instruction.
    MatchesTerms { terms: Vec<String> },
    /// A file already chosen imports it — shown as an outline, not in full.
    ImportedBy { path: String },
    /// It has uncommitted changes in the working tree.
    ChangedInWorkingTree,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextItem {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    /// For an outline item, the file's symbol list rather than its text.
    pub text: String,
    pub outline: bool,
    pub reasons: Vec<Reason>,
    pub score: f64,
    /// Estimated, not measured.
    pub est_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Excluded {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub reasons: Vec<Reason>,
    pub est_tokens: u32,
    /// Why it was left out, in words.
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackage {
    pub intent: Intent,
    pub items: Vec<ContextItem>,
    pub excluded: Vec<Excluded>,
    pub est_tokens: u32,
    pub budget_tokens: u32,
    /// The whole indexed repository, for scale: how targeted the package is.
    pub repo_est_tokens: u64,
    pub repo_files: usize,
}

pub struct ContextRequest<'a> {
    pub instruction: &'a str,
    pub budget_tokens: u32,
    /// Workspace-relative paths with uncommitted changes (from `git status`).
    pub changed_paths: &'a [String],
}

/// Bytes ÷ 4, rounded up. An estimate, deliberately simple and labelled as one.
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() as u32).div_ceil(4)
}

/// A candidate before budgeting, keyed by (path, start line) so stages that find the same
/// passage add their reasons and scores together rather than duplicating it.
struct Candidate {
    end_line: u32,
    text: String,
    outline: bool,
    reasons: Vec<Reason>,
    score: f64,
}

type Candidates = BTreeMap<(String, u32), Candidate>;

// One passage and why it matters; grouping them into a struct would only rename the list.
#[allow(clippy::too_many_arguments)]
fn add(
    candidates: &mut Candidates,
    path: &str,
    start: u32,
    end: u32,
    text: String,
    outline: bool,
    reason: Reason,
    score: f64,
) {
    let entry = candidates
        .entry((path.to_string(), start))
        .or_insert_with(|| Candidate {
            end_line: end,
            text,
            outline,
            reasons: Vec::new(),
            score: 0.0,
        });
    if !entry.reasons.contains(&reason) {
        entry.reasons.push(reason);
    }
    entry.score += score;
}

/// Lines `start..=end` (1-based) of a file on disk, or None if it cannot be read.
fn read_lines(index: &Index, path: &str, start: u32, end: u32) -> Option<String> {
    let text = std::fs::read_to_string(index.root().join(path)).ok()?;
    let lines: Vec<&str> = text
        .lines()
        .skip(start.saturating_sub(1) as usize)
        .take((end.saturating_sub(start) + 1) as usize)
        .collect();
    Some(lines.join("\n"))
}

fn outline(index: &Index, path: &str) -> Result<Option<String>, IndexError> {
    let symbols = index.symbols_in(path)?;
    if symbols.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        symbols
            .iter()
            .map(|s| format!("{:?} {} (line {})", s.kind, s.name, s.start_line).to_lowercase())
            .collect::<Vec<_>>()
            .join("\n"),
    ))
}

pub fn build(index: &Index, request: &ContextRequest) -> Result<ContextPackage, ContextError> {
    let files = index.files()?;
    let repo_est_tokens: u64 = files.iter().map(|f| f.bytes.div_ceil(4)).sum();
    let known: BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    let intent = extract_intent(request.instruction, &known);
    let mut candidates = Candidates::new();

    // 1. Files the instruction names: whole when short, outlined when long.
    for path in &intent.paths {
        let Some(info) = files.iter().find(|f| &f.path == path) else {
            continue;
        };
        if info.lines <= WHOLE_FILE_MAX_LINES {
            if let Some(text) = read_lines(index, path, 1, info.lines.max(1)) {
                add(
                    &mut candidates,
                    path,
                    1,
                    info.lines.max(1),
                    text,
                    false,
                    Reason::NamedInInstruction,
                    10.0,
                );
            }
        } else if let Some(text) = outline(index, path)? {
            add(
                &mut candidates,
                path,
                1,
                info.lines,
                text,
                true,
                Reason::NamedInInstruction,
                6.0,
            );
        }
    }

    // 2. Definitions of the symbols it mentions. Code-shaped names weigh more than plain
    // words — "implement multiply in calc.py" may mean a symbol called `multiply`, and the
    // lookup is an exact-name match, so trying is cheap and wrong guesses find nothing.
    let names = intent
        .identifiers
        .iter()
        .map(|n| (n, 8.0))
        .chain(intent.terms.iter().map(|n| (n, 5.0)));
    for (name, weight) in names {
        for symbol in index.definitions(name, DEFINITIONS_PER_NAME)? {
            let end = symbol
                .end_line
                .min(symbol.start_line + DEFINITION_MAX_LINES - 1)
                .max(symbol.start_line);
            if let Some(text) = read_lines(index, &symbol.path, symbol.start_line, end) {
                add(
                    &mut candidates,
                    &symbol.path,
                    symbol.start_line,
                    end,
                    text,
                    false,
                    Reason::DefinesSymbol {
                        symbol: name.clone(),
                    },
                    weight,
                );
            }
        }
    }

    // 3. Full-text: the passages that best match the instruction's words.
    let words: Vec<&str> = intent
        .identifiers
        .iter()
        .chain(intent.terms.iter())
        .map(String::as_str)
        .collect();
    if !words.is_empty() {
        let hits = index.search_chunks(&words, LEXICAL_HITS)?;
        let best = hits.iter().map(|h| h.score).fold(f64::MIN, f64::max);
        for hit in hits {
            let lowered = hit.text.to_lowercase();
            let matched: Vec<String> = words
                .iter()
                .filter(|w| lowered.contains(&w.to_lowercase()))
                .map(|w| w.to_string())
                .collect();
            // Normalised so the strongest lexical hit is worth about one symbol match.
            let relative = if best > 0.0 { hit.score / best } else { 0.5 };
            add(
                &mut candidates,
                &hit.path,
                hit.start_line,
                hit.end_line,
                hit.text,
                false,
                Reason::MatchesTerms { terms: matched },
                2.0 + 4.0 * relative.clamp(0.0, 1.0),
            );
        }
    }

    // 4. Structural: what the strongest files import, as outlines.
    let chosen: BTreeSet<String> = candidates
        .iter()
        .filter(|(_, c)| c.score >= 6.0)
        .map(|((path, _), _)| path.clone())
        .collect();
    for path in &chosen {
        for import in index.imports_of(path)? {
            if let Some(target) = resolve_import(&import.target, path, &known) {
                if chosen.contains(&target) {
                    continue;
                }
                if let Some(text) = outline(index, &target)? {
                    let lines = files
                        .iter()
                        .find(|f| f.path == target)
                        .map_or(1, |f| f.lines);
                    add(
                        &mut candidates,
                        &target,
                        1,
                        lines,
                        text,
                        true,
                        Reason::ImportedBy { path: path.clone() },
                        2.0,
                    );
                }
            }
        }
    }

    // 5. Working tree: uncommitted changes are usually what the task is about.
    for path in request.changed_paths {
        let mut touched = false;
        for ((p, _), candidate) in candidates.iter_mut() {
            if p == path {
                touched = true;
                if !candidate.reasons.contains(&Reason::ChangedInWorkingTree) {
                    candidate.reasons.push(Reason::ChangedInWorkingTree);
                }
                candidate.score += 2.0;
            }
        }
        if !touched && known.contains(path.as_str()) {
            if let Some(text) = outline(index, path)? {
                add(
                    &mut candidates,
                    path,
                    1,
                    1,
                    text,
                    true,
                    Reason::ChangedInWorkingTree,
                    1.5,
                );
            }
        }
    }

    Ok(budget(
        intent,
        candidates,
        request.budget_tokens,
        repo_est_tokens,
        files.len(),
    ))
}

/// Rerank by score and keep what fits. Overlapping passages of one file keep only the
/// better-scored one; everything that scored but did not fit is reported as excluded.
fn budget(
    intent: Intent,
    candidates: Candidates,
    budget_tokens: u32,
    repo_est_tokens: u64,
    repo_files: usize,
) -> ContextPackage {
    let mut ranked: Vec<((String, u32), Candidate)> = candidates.into_iter().collect();
    ranked.sort_by(|a, b| b.1.score.total_cmp(&a.1.score).then_with(|| a.0.cmp(&b.0)));

    let mut items: Vec<ContextItem> = Vec::new();
    let mut excluded = Vec::new();
    let mut used = 0u32;
    for ((path, start), c) in ranked {
        let est = estimate_tokens(&c.text);
        let overlaps = items.iter().any(|i| {
            !i.outline
                && !c.outline
                && i.path == path
                && start <= i.end_line
                && c.end_line >= i.start_line
        });
        let why = if overlaps {
            Some("overlaps a passage already included".to_string())
        } else if used + est > budget_tokens {
            Some(format!(
                "over budget — needs ~{est} tokens, {} left",
                budget_tokens - used
            ))
        } else {
            None
        };
        match why {
            Some(why) => excluded.push(Excluded {
                path,
                start_line: start,
                end_line: c.end_line,
                reasons: c.reasons,
                est_tokens: est,
                why,
            }),
            None => {
                used += est;
                items.push(ContextItem {
                    path,
                    start_line: start,
                    end_line: c.end_line,
                    text: c.text,
                    outline: c.outline,
                    reasons: c.reasons,
                    score: c.score,
                    est_tokens: est,
                });
            }
        }
    }
    ContextPackage {
        intent,
        items,
        excluded,
        est_tokens: used,
        budget_tokens,
        repo_est_tokens,
        repo_files,
    }
}

/// Maps an import as written to a file in the repository, when it plainly names one:
/// a relative path (`./calc`), a Python module (`calc`, `pkg.calc`), a Rust `crate::a::b`.
/// Anything else — an external package — is not in the repository and resolves to None.
fn resolve_import(target: &str, from: &str, known: &BTreeSet<&str>) -> Option<String> {
    let dir = from.rsplit_once('/').map_or("", |(d, _)| d);
    let join = |base: &str, rel: &str| -> String {
        let mut parts: Vec<&str> = if base.is_empty() {
            vec![]
        } else {
            base.split('/').collect()
        };
        for seg in rel.split('/') {
            match seg {
                "." | "" => {}
                ".." => {
                    parts.pop();
                }
                s => parts.push(s),
            }
        }
        parts.join("/")
    };
    let stem = if target.starts_with('.') {
        join(dir, target)
    } else if let Some(rest) = target.strip_prefix("crate::") {
        rest.split("::")
            .take_while(|s| !s.contains('{'))
            .collect::<Vec<_>>()
            .join("/")
    } else {
        target.replace('.', "/")
    };
    let extensions = [
        "",
        ".ts",
        ".tsx",
        ".js",
        ".jsx",
        ".py",
        ".rs",
        "/index.ts",
        "/index.js",
        "/mod.rs",
        "/__init__.py",
    ];
    let candidates = |stem: &str| -> Option<String> {
        extensions
            .iter()
            .map(|ext| format!("{stem}{ext}"))
            .find(|p| known.contains(p.as_str()))
    };
    candidates(&stem)
        .or_else(|| {
            (!dir.is_empty())
                .then(|| candidates(&join(dir, &stem)))
                .flatten()
        })
        .or_else(|| candidates(&format!("src/{stem}")))
}

/// The package as prompt text. Every passage is repository content — untrusted — so each
/// is fenced in the `<untrusted>` envelope with its origin; the reasons are the runtime's
/// own words and are stated outside it.
pub fn render(package: &ContextPackage) -> String {
    if package.items.is_empty() {
        return String::new();
    }
    let mut out = format!(
        "Repository context selected for this task (about {} tokens of the repository's ~{}, estimated):\n",
        package.est_tokens, package.repo_est_tokens
    );
    for item in &package.items {
        let why = item
            .reasons
            .iter()
            .map(describe)
            .collect::<Vec<_>>()
            .join("; ");
        let origin = if item.outline {
            format!("file:{} (outline)", item.path)
        } else {
            format!("file:{}:{}-{}", item.path, item.start_line, item.end_line)
        };
        out.push_str(&format!("\n{origin} — {why}\n"));
        out.push_str(&Tagged::untrusted(origin, item.text.clone()).to_prompt_text());
        out.push('\n');
    }
    out
}

pub fn describe(reason: &Reason) -> String {
    match reason {
        Reason::NamedInInstruction => "named in the instruction".into(),
        Reason::DefinesSymbol { symbol } => format!("defines `{symbol}`"),
        Reason::MatchesTerms { terms } if terms.is_empty() => "full-text match".into(),
        Reason::MatchesTerms { terms } => format!("matches {}", terms.join(", ")),
        Reason::ImportedBy { path } => format!("imported by {path}"),
        Reason::ChangedInWorkingTree => "changed in the working tree".into(),
    }
}
