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
        if is_sensitive(&rel) {
            continue;
        }
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

/// Files never indexed or searched, whatever the ignore files say: everything
/// `anycode_security::path_risk` classifies — keys, credentials, env files. The code tools
/// are Low risk and run without asking, so they must never hand the model what
/// `filesystem.read` would have had to ask for.
pub fn is_sensitive(rel: &str) -> bool {
    anycode_security::path_risk(rel).is_some()
}

/// Whether `fs_path` must stay out of the index — the watcher's single-path twin of
/// [`walk`], applying the same rules so a file the full scan skips is never indexed
/// just because it changed: outside `root`, under `.git`, hidden (as the walker's
/// default skips), sensitive, or ignored by any `.gitignore`/`.ignore` between the
/// repository's top level and the file, or by `.git/info/exclude`.
///
/// The user's global git excludes (`core.excludesFile`), which the full walk honours
/// too. Read once: it lives in git config, not in the workspace.
fn global_excludes() -> &'static ignore::gitignore::Gitignore {
    static GLOBAL: std::sync::OnceLock<ignore::gitignore::Gitignore> = std::sync::OnceLock::new();
    GLOBAL.get_or_init(|| ignore::gitignore::Gitignore::global().0)
}

/// Whether `rel` or any directory above it matches `matcher`, whose patterns are relative.
fn matches_relative(matcher: &ignore::gitignore::Gitignore, rel: &str, is_dir: bool) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    (1..=parts.len()).any(|n| {
        let prefix = parts[..n].join("/");
        let this_is_dir = n < parts.len() || is_dir;
        matcher.matched(Path::new(&prefix), this_is_dir).is_ignore()
    })
}

pub fn is_excluded(root: &Path, fs_path: &Path) -> bool {
    let Some(rel) = to_workspace_relative(root, fs_path) else {
        return true;
    };
    if has_git_component(fs_path) || rel.split('/').any(|part| part.starts_with('.')) {
        return true;
    }
    if is_sensitive(&rel) {
        return true;
    }
    let is_dir = fs_path.is_dir();
    if matches_relative(global_excludes(), &rel, is_dir) {
        return true;
    }
    // Deepest directory first: as in git, its rules override its parents'. The climb goes
    // past `root` to the enclosing repository's top level, as the full walk does — a
    // workspace opened at `monorepo/apps` still honours `monorepo/.gitignore`.
    let mut dir = fs_path.parent();
    while let Some(d) = dir {
        let mut builder = ignore::gitignore::GitignoreBuilder::new(d);
        // Later files take precedence in one matcher; `.ignore` outranks `.gitignore`.
        let _ = builder.add(d.join(".gitignore"));
        let _ = builder.add(d.join(".ignore"));
        let top_level = d.join(".git").exists();
        if top_level {
            let _ = builder.add(d.join(".git").join("info").join("exclude"));
        }
        if let Ok(matcher) = builder.build() {
            let hit = matcher.matched_path_or_any_parents(fs_path, is_dir);
            if hit.is_ignore() {
                return true;
            }
            if hit.is_whitelist() {
                return false;
            }
        }
        if top_level {
            break;
        }
        dir = d.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_patterns_match_a_path_or_any_directory_above_it() {
        let mut builder = ignore::gitignore::GitignoreBuilder::new("");
        builder.add_line(None, "*.secret").unwrap();
        builder.add_line(None, "scratch/").unwrap();
        let matcher = builder.build().unwrap();
        assert!(matches_relative(&matcher, "a/b/key.secret", false));
        assert!(matches_relative(&matcher, "scratch/notes.md", false));
        assert!(matches_relative(&matcher, "deep/scratch/notes.md", false));
        assert!(!matches_relative(&matcher, "src/main.rs", false));
    }

    #[test]
    fn nul_byte_marks_binary() {
        assert!(is_probably_binary(b"hello\0world"));
        assert!(!is_probably_binary(b"hello world"));
    }
}
