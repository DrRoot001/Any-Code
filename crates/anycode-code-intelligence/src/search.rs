//! Live, index-free search backed by ripgrep's own libraries
//! (`ignore` for the gitignore-aware walk, `grep-searcher`/`grep-regex` for
//! matching) — no SQLite involved, so this works even before a workspace has
//! been indexed at all.

use crate::path_util::{best_effort_canonical, has_git_component};
use crate::{IndexError, LineMatch};
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use std::path::Path;

fn walk_and_search(
    root: &Path,
    matcher: &grep_regex::RegexMatcher,
    limit: usize,
) -> Result<Vec<LineMatch>, IndexError> {
    // `to_workspace_relative` expects a canonical root (the same contract
    // `Index` upholds by canonicalising once at `open` time) — resolve it
    // here too so an entry path and `root` compare on the same footing even
    // when the workspace sits under a symlink.
    let root = &best_effort_canonical(root);
    let mut out = Vec::new();
    let mut builder = WalkBuilder::new(root);
    // See walk::walk's comment: honour .gitignore/.ignore even outside a real
    // git repository.
    builder.require_git(false);
    'walk: for result in builder.build() {
        let Ok(entry) = result else { continue };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if has_git_component(entry.path()) {
            continue;
        }
        let path = entry.path();
        let Some(rel) = crate::path_util::to_workspace_relative(root, path) else {
            continue;
        };
        // A single unreadable file (binary, permissions, vanished mid-walk)
        // should not fail the whole search.
        let _: Result<(), std::io::Error> = Searcher::new().search_path(
            matcher,
            path,
            UTF8(|line_number, line| -> Result<bool, std::io::Error> {
                out.push(LineMatch {
                    path: rel.clone(),
                    line: line_number as u32,
                    text: line.trim_end_matches(['\n', '\r']).to_string(),
                });
                Ok(out.len() < limit)
            }),
        );
        if out.len() >= limit {
            break 'walk;
        }
    }
    Ok(out)
}

/// Index-free search over `root`. `pattern` is a regex when `regex` is true,
/// otherwise it is matched literally (escaped before being handed to the
/// regex engine, so it can never be read as regex syntax).
pub fn search_live(
    root: &Path,
    pattern: &str,
    regex: bool,
    limit: usize,
) -> Result<Vec<LineMatch>, IndexError> {
    if limit == 0 || pattern.is_empty() {
        return Ok(Vec::new());
    }
    let matcher = RegexMatcherBuilder::new()
        .fixed_strings(!regex)
        .build(pattern)
        .map_err(|e| IndexError::Search(e.to_string()))?;
    walk_and_search(root, &matcher, limit)
}

/// Whole-word occurrences of `identifier` — a literal, word-bounded search,
/// used for "find references" without needing an index.
pub fn references(
    root: &Path,
    identifier: &str,
    limit: usize,
) -> Result<Vec<LineMatch>, IndexError> {
    if limit == 0 || identifier.is_empty() {
        return Ok(Vec::new());
    }
    let matcher = RegexMatcherBuilder::new()
        .fixed_strings(true)
        .word(true)
        .build(identifier)
        .map_err(|e| IndexError::Search(e.to_string()))?;
    walk_and_search(root, &matcher, limit)
}
