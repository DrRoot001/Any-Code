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

/// Longest line returned. A minified bundle is one enormous line; fifty of them would fill
/// the model's context with nothing it can use.
const MAX_LINE_CHARS: usize = 400;
/// Longest pattern accepted, and the compiled-regex memory it may use.
const MAX_PATTERN_CHARS: usize = 1000;
const REGEX_SIZE_LIMIT: usize = 10 * 1024 * 1024;

fn truncate_line(line: &str) -> String {
    match line.char_indices().nth(MAX_LINE_CHARS) {
        Some((cut, _)) => format!("{}…", &line[..cut]),
        None => line.to_string(),
    }
}

fn matcher(
    pattern: &str,
    literal: bool,
    word: bool,
) -> Result<grep_regex::RegexMatcher, IndexError> {
    if pattern.chars().count() > MAX_PATTERN_CHARS {
        return Err(IndexError::Search(format!(
            "pattern is longer than {MAX_PATTERN_CHARS} characters"
        )));
    }
    RegexMatcherBuilder::new()
        .fixed_strings(literal)
        .word(word)
        .size_limit(REGEX_SIZE_LIMIT)
        .dfa_size_limit(REGEX_SIZE_LIMIT)
        .build(pattern)
        .map_err(|e| IndexError::Search(e.to_string()))
}

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
        // The same files the index leaves out (walk.rs): secrets, and anything too large
        // to be source.
        if crate::walk::is_sensitive(&rel)
            || entry
                .metadata()
                .map_or(true, |m| m.len() > crate::walk::MAX_FILE_BYTES)
        {
            continue;
        }
        // A single unreadable file (binary, permissions, vanished mid-walk)
        // should not fail the whole search.
        let _: Result<(), std::io::Error> = Searcher::new().search_path(
            matcher,
            path,
            UTF8(|line_number, line| -> Result<bool, std::io::Error> {
                out.push(LineMatch {
                    path: rel.clone(),
                    line: line_number as u32,
                    text: truncate_line(line.trim_end_matches(['\n', '\r'])),
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
    walk_and_search(root, &matcher(pattern, !regex, false)?, limit)
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
    walk_and_search(root, &matcher(identifier, true, true)?, limit)
}
