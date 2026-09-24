//! Path normalisation shared by the walker and `update_paths`.
//!
//! Stored paths are always workspace-relative with `/` separators, on every OS
//! (ADR 0004). Filesystem paths are built with `Path::join`, never string
//! concatenation.

use std::path::{Component, Path, PathBuf};

/// Joins a workspace-relative path (stored form, `/`-separated) back onto the
/// filesystem root using `Path::join`.
pub fn to_fs_path(root: &Path, rel: &str) -> PathBuf {
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        out.push(part);
    }
    out
}

/// Turns any path (absolute, or relative to `root`) into the stored
/// workspace-relative form (`/`-separated components). `root` is expected to
/// already be canonical (`Index` canonicalises it once at open time); an
/// absolute `input` is resolved as far as the filesystem allows so the two
/// sides compare on the same footing even when the workspace sits under a
/// symlink (e.g. macOS's `/tmp` -> `/private/tmp`, or `/var` -> `/private/var`,
/// which is exactly what `std::env::temp_dir()` returns) — this still works
/// for a path that no longer exists (a deletion), since only the deepest
/// existing ancestor needs resolving.
///
/// Returns `None` when the path resolves outside `root` (a `..` that escapes
/// it, or an absolute path under a different tree).
pub fn to_workspace_relative(root: &Path, input: &Path) -> Option<String> {
    let absolute = if input.is_absolute() {
        best_effort_canonical(input)
    } else {
        root.join(input)
    };
    relative_to(root, &absolute)
}

/// The name `input` has inside `root` without resolving symlinks — what a file watcher
/// calls a path. Used to drop the rows of a file that became a symlink: resolving it
/// would name the link's target instead, or nothing when it points outside the root.
pub fn lexical_relative(root: &Path, input: &Path) -> Option<String> {
    let absolute = root.join(input);
    // Resolve the directory (so `/var/…` and `/private/var/…` agree), never the file.
    let absolute = match (absolute.parent(), absolute.file_name()) {
        (Some(parent), Some(name)) => best_effort_canonical(parent).join(name),
        _ => absolute,
    };
    relative_to(root, &absolute)
}

fn relative_to(root: &Path, absolute: &Path) -> Option<String> {
    let normalized = lexically_normalize(absolute);
    let root_normalized = lexically_normalize(root);
    let rel = normalized.strip_prefix(&root_normalized).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            // A workspace-relative path never contains `..`, `.`, prefixes or a
            // bare root once stripped of the workspace root itself.
            _ => return None,
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("/"))
}

/// Canonicalises `path` if it exists; otherwise canonicalises its deepest
/// existing ancestor and rejoins the remaining components (verbatim — including
/// a literal `..`, which is why this walks `components()` rather than using
/// `Path::parent()`/`file_name()`, which both return `None` once the path
/// ends in `..`). A path with no existing ancestor at all is returned as-is.
pub fn best_effort_canonical(path: &Path) -> PathBuf {
    if let Ok(resolved) = path.canonicalize() {
        return resolved;
    }
    let components: Vec<Component> = path.components().collect();
    for split in (1..components.len()).rev() {
        let ancestor: PathBuf = components[..split].iter().collect();
        if let Ok(resolved) = ancestor.canonicalize() {
            let mut out = resolved;
            for component in &components[split..] {
                out.push(component.as_os_str());
            }
            return out;
        }
    }
    path.to_path_buf()
}

/// Resolves `..`/`.` components lexically (no filesystem access), so it works
/// for paths that may not exist. This is deliberately simpler than
/// `fs::canonicalize`: it does not resolve symlinks, which is fine here since
/// both sides of the comparison (root and candidate) go through the same
/// normalisation.
fn lexically_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// True if any component of the path is `.git` — always skipped regardless of
/// gitignore rules.
pub fn has_git_component(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == std::ffi::OsStr::new(".git"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_uses_forward_slashes() {
        let root = Path::new("/work/repo");
        let abs = Path::new("/work/repo/src/main.rs");
        assert_eq!(
            to_workspace_relative(root, abs).as_deref(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn path_outside_root_is_none() {
        let root = Path::new("/work/repo");
        let abs = Path::new("/work/other/main.rs");
        assert_eq!(to_workspace_relative(root, abs), None);
    }

    #[test]
    fn dot_dot_escaping_root_is_none() {
        let root = Path::new("/work/repo");
        let abs = Path::new("/work/repo/../other/main.rs");
        assert_eq!(to_workspace_relative(root, abs), None);
    }

    #[test]
    fn already_relative_input_is_joined_to_root() {
        let root = Path::new("/work/repo");
        let rel = Path::new("src/lib.rs");
        assert_eq!(
            to_workspace_relative(root, rel).as_deref(),
            Some("src/lib.rs")
        );
    }
}
