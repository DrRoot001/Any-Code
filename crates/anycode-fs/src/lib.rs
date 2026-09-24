//! Filesystem access scoped to a workspace root (docs/ARCHITECTURE.md invariant #8: the
//! UI never touches the filesystem directly; every read/write crosses this boundary).
//!
//! The one property this crate exists to guarantee: no relative path, however it's
//! spelled, resolves outside the workspace root. `..` components are rejected during
//! path resolution rather than filtered by canonicalizing afterward, so it holds even
//! for paths that don't exist yet (a file being created).

use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("path escapes the workspace root: {0}")]
    EscapesRoot(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("file is not valid UTF-8: {0}")]
    NotUtf8(String),
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoot {
    root: PathBuf,
}

impl WorkspaceRoot {
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            root: fs::canonicalize(root)?,
        })
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Resolves a `/`-separated relative path against the root, rejecting anything
    /// that would climb above it. The empty string resolves to the root itself.
    ///
    /// An absolute path is accepted only when it names something inside the root: the
    /// root prefix is stripped (component-wise, so `/ws2` is not inside `/ws`) and the
    /// remainder goes through the same rules, so `/ws/../etc` is still refused. Models
    /// often write absolute paths; refusing an in-root one as an "escape" was both wrong
    /// and a dead end for them.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, FsError> {
        let given = Path::new(relative);
        let relative_part = if given.is_absolute() {
            given
                .strip_prefix(&self.root)
                .map_err(|_| FsError::EscapesRoot(relative.to_string()))?
        } else {
            given
        };
        let mut resolved = self.root.clone();
        for component in relative_part.components() {
            match component {
                Component::Normal(part) => resolved.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(FsError::EscapesRoot(relative.to_string()));
                }
            }
        }
        Ok(resolved)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    /// `/`-separated, relative to the workspace root.
    pub path: String,
    pub is_dir: bool,
}

/// Lists the immediate children of `relative` (a directory). Directories sort first,
/// then alphabetically — the ordering an explorer tree is expected to render in.
pub fn list_dir(root: &WorkspaceRoot, relative: &str) -> Result<Vec<Entry>, FsError> {
    let dir = root.resolve(relative)?;
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        let is_dir = item.file_type()?.is_dir();
        let path = if relative.is_empty() {
            name.clone()
        } else {
            format!("{relative}/{name}")
        };
        entries.push(Entry { name, path, is_dir });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

pub fn read_file(root: &WorkspaceRoot, relative: &str) -> Result<String, FsError> {
    let bytes = fs::read(root.resolve(relative)?)?;
    String::from_utf8(bytes).map_err(|_| FsError::NotUtf8(relative.to_string()))
}

pub fn write_file(root: &WorkspaceRoot, relative: &str, contents: &str) -> Result<(), FsError> {
    let path = root.resolve(relative)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> (tempfile_free::TempDir, WorkspaceRoot) {
        let dir = tempfile_free::TempDir::new();
        let root = WorkspaceRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    #[test]
    fn rejects_parent_dir_traversal() {
        let (_dir, root) = workspace();
        assert!(matches!(
            root.resolve("../etc/passwd"),
            Err(FsError::EscapesRoot(_))
        ));
        assert!(matches!(
            root.resolve("a/../../b"),
            Err(FsError::EscapesRoot(_))
        ));
    }

    #[test]
    fn rejects_an_absolute_path_outside_the_root() {
        let (_dir, root) = workspace();
        assert!(matches!(
            root.resolve("/etc/passwd"),
            Err(FsError::EscapesRoot(_))
        ));
        // A sibling whose name merely starts with the root's is not inside it.
        let sibling = format!("{}2/x", root.path().display());
        assert!(matches!(
            root.resolve(&sibling),
            Err(FsError::EscapesRoot(_))
        ));
        // Traversal is refused after the prefix is stripped, too.
        let climbing = format!("{}/../etc", root.path().display());
        assert!(matches!(
            root.resolve(&climbing),
            Err(FsError::EscapesRoot(_))
        ));
    }

    #[test]
    fn accepts_an_absolute_path_inside_the_root() {
        let (_dir, root) = workspace();
        let absolute = format!("{}/src/calc.py", root.path().display());
        assert_eq!(
            root.resolve(&absolute).unwrap(),
            root.resolve("src/calc.py").unwrap()
        );
    }

    #[test]
    fn round_trips_a_file_inside_the_root() {
        let (_dir, root) = workspace();
        write_file(&root, "notes/todo.md", "hello").unwrap();
        assert_eq!(read_file(&root, "notes/todo.md").unwrap(), "hello");
        let listing = list_dir(&root, "notes").unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].path, "notes/todo.md");
    }

    /// Zero-dependency temp dir so this crate doesn't need a dev-dependency just for tests.
    mod tempfile_free {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU64, Ordering};

        pub struct TempDir(PathBuf);
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        impl TempDir {
            pub fn new() -> Self {
                let n = COUNTER.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("anycode-fs-test-{}-{n}", std::process::id()));
                std::fs::create_dir_all(&path).unwrap();
                Self(path)
            }

            pub fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}
