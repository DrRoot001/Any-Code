//! Gitignore-aware repository walk (the `ignore` crate) and the cheap
//! eligibility checks that decide whether a file is a re-read/re-index
//! candidate at all.

use crate::path_util::{has_git_component, to_workspace_relative};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// One MiB — files larger than this are never indexed (PRD: keep the index
/// small; a 1 MiB text file is already an outlier for source code).
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// How many leading bytes are sniffed for a NUL byte to call a file binary.
const BINARY_SNIFF_BYTES: usize = 8192;

/// A file discovered by the walk, before content is read.
pub struct WalkEntry {
    pub rel: String,
    pub fs_path: PathBuf,
    pub size: u64,
    pub mtime_nanos: i64,
}

/// Walks every file under `start` (which must be `root` or a directory inside
/// it), respecting `.gitignore`/`.ignore` and always skipping `.git`.
pub fn walk(root: &Path, start: &Path) -> Vec<WalkEntry> {
    let mut out = Vec::new();
    let mut builder = WalkBuilder::new(start);
    // Honour .gitignore/.ignore even when the workspace isn't (yet) a real
    // git repository — `ignore`'s default only applies git-sourced ignore
    // rules inside an actual repo.
    builder.require_git(false);
    for result in builder.build() {
        let Ok(entry) = result else { continue };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let fs_path = entry.path().to_path_buf();
        if has_git_component(&fs_path) {
            continue;
        }
        let Some(rel) = to_workspace_relative(root, &fs_path) else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        out.push(WalkEntry {
            rel,
            fs_path,
            size: metadata.len(),
            mtime_nanos: mtime_nanos(&metadata),
        });
    }
    out
}

/// Nanoseconds since the Unix epoch, for the freshness check. Falls back to 0
/// on platforms/files that can't report an mtime, which just forces a re-read
/// (never a correctness problem, only a wasted one).
pub fn mtime_nanos(metadata: &std::fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

/// NUL in the first 8 KiB is treated as "binary" (ripgrep's heuristic).
pub fn is_probably_binary(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(BINARY_SNIFF_BYTES)].contains(&0)
}

/// Whether `fs_path` is covered by `root`'s `.gitignore`/`.ignore`, or lives
/// outside `root` entirely.
///
/// ponytail: only the root-level `.gitignore`/`.ignore` are consulted here —
/// `update_paths` gets one explicit path at a time from a file watcher, not a
/// tree to walk, so there is no cheap way to discover nested `.gitignore`
/// files without walking. `refresh()` uses the full recursive `ignore` walker
/// above, which already honours nested `.gitignore` files correctly. Upgrade
/// this to consult ancestor directories between `root` and the file if a
/// project's `.gitignore` files stop being root-only in practice.
pub fn is_ignored_or_outside_root(root: &Path, fs_path: &Path) -> bool {
    if to_workspace_relative(root, fs_path).is_none() {
        return true;
    }
    if has_git_component(fs_path) {
        return true;
    }
    let mut builder = ignore::gitignore::GitignoreBuilder::new(root);
    let _ = builder.add(root.join(".gitignore"));
    let _ = builder.add(root.join(".ignore"));
    let Ok(matcher) = builder.build() else {
        return false;
    };
    let is_dir = fs_path.is_dir();
    matcher.matched(fs_path, is_dir).is_ignore()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nul_byte_marks_binary() {
        assert!(is_probably_binary(b"hello\0world"));
        assert!(!is_probably_binary(b"hello world"));
    }
}
