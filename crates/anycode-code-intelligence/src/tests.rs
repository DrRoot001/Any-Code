use crate::{index_db_path, references, search_live, Index, Language, SymbolKind};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

fn find<'a>(symbols: &'a [crate::Symbol], name: &str) -> &'a crate::Symbol {
    symbols
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("symbol `{name}` not found among {symbols:?}"))
}

// --- fixture repo: one file per supported language, with known definitions
// and imports, plus a gitignored file, a binary file and an oversized file.

fn build_fixture(root: &Path) {
    write(
        root,
        "src/lib.rs",
        r#"use std::collections::HashMap;

pub struct Widget {
    pub name: String,
}

impl Widget {
    pub fn new(name: &str) -> Widget {
        Widget { name: name.to_string() }
    }
}

pub fn free_function() -> i32 {
    42
}
"#,
    );
    write(
        root,
        "src/greeter.ts",
        r#"import { Widget } from "./widget";

export interface Greeter {
  greet(name: string): string;
}

export abstract class Base {
  abstract run(): void;
}
"#,
    );
    write(
        root,
        "src/props.tsx",
        r#"export interface Props {
  label(): string;
}
"#,
    );
    write(
        root,
        "src/widget.js",
        r#"// a comment
const dep = require("./dep");

class Widget {
  constructor() {}
  render() {
    return 1;
  }
}

function helper() {
  return 2;
}

const arrow = () => 3;
"#,
    );
    write(
        root,
        "src/greeter.py",
        r#"import os
from collections import OrderedDict

class Greeter:
    def greet(self, name):
        return name

def helper():
    return 1

CONST = 1
"#,
    );

    // gitignored file: must never be indexed.
    write(root, ".gitignore", "ignored.rs\n");
    write(root, "ignored.rs", "pub fn should_not_be_indexed() {}\n");

    // binary file (NUL in the first bytes): must be skipped.
    let binary_path = root.join("assets/logo.bin");
    fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
    fs::write(&binary_path, [0u8, 1, 2, 3, b'b', b'i', b'n']).unwrap();

    // oversized file (> 1 MiB): must be skipped.
    let huge_path = root.join("assets/huge.rs");
    fs::write(&huge_path, vec![b'a'; 2 * 1024 * 1024]).unwrap();
}

#[test]
fn refresh_indexes_known_definitions_and_imports_and_skips_the_rest() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    let report = index.refresh().unwrap();

    // The gitignored file, the binary file and the oversized file were all
    // scanned but none of them ended up in the index.
    assert_eq!(report.removed, 0);
    assert!(
        report.skipped >= 2,
        "expected the binary and oversized files to be skipped, got {report:?}"
    );
    assert!(
        report.reindexed >= 5,
        "expected at least the 5 fixture source files, got {report:?}"
    );

    let files = index.files().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(
        !paths.contains(&"ignored.rs"),
        "gitignored file leaked into the index: {paths:?}"
    );
    assert!(
        !paths.contains(&"assets/logo.bin"),
        "binary file leaked into the index: {paths:?}"
    );
    assert!(
        !paths.contains(&"assets/huge.rs"),
        "oversized file leaked into the index: {paths:?}"
    );

    // Rust: struct, its impl method (container resolved via the impl block),
    // and a free function.
    let rust_symbols = index.symbols_in("src/lib.rs").unwrap();
    assert_eq!(find(&rust_symbols, "Widget").kind, SymbolKind::Struct);
    let new_fn = find(&rust_symbols, "new");
    assert_eq!(new_fn.kind, SymbolKind::Method);
    assert_eq!(new_fn.container.as_deref(), Some("Widget"));
    assert_eq!(
        find(&rust_symbols, "free_function").kind,
        SymbolKind::Function
    );
    let rust_imports = index.imports_of("src/lib.rs").unwrap();
    assert!(rust_imports
        .iter()
        .any(|i| i.target == "std::collections::HashMap"));

    // TypeScript: interface + its method, abstract class + its method.
    let ts_symbols = index.symbols_in("src/greeter.ts").unwrap();
    assert_eq!(find(&ts_symbols, "Greeter").kind, SymbolKind::Interface);
    let greet = find(&ts_symbols, "greet");
    assert_eq!(greet.kind, SymbolKind::Method);
    assert_eq!(greet.container.as_deref(), Some("Greeter"));
    assert_eq!(find(&ts_symbols, "Base").kind, SymbolKind::Class);
    let run = find(&ts_symbols, "run");
    assert_eq!(run.kind, SymbolKind::Method);
    assert_eq!(run.container.as_deref(), Some("Base"));
    let ts_imports = index.imports_of("src/greeter.ts").unwrap();
    assert!(ts_imports.iter().any(|i| i.target == "./widget"));

    // TSX: same grammar, different extension.
    let tsx_symbols = index.symbols_in("src/props.tsx").unwrap();
    assert_eq!(find(&tsx_symbols, "Props").kind, SymbolKind::Interface);
    assert_eq!(
        find(&tsx_symbols, "label").container.as_deref(),
        Some("Props")
    );

    // JavaScript: class + method (constructor filtered out by the grammar's
    // own `#not-eq?` predicate), a function, an arrow assigned to `const`,
    // and both `require(...)` and no false `import` positives.
    let js_symbols = index.symbols_in("src/widget.js").unwrap();
    assert_eq!(find(&js_symbols, "Widget").kind, SymbolKind::Class);
    assert!(
        !js_symbols.iter().any(|s| s.name == "constructor"),
        "constructor should be filtered by the grammar's own #not-eq? predicate: {js_symbols:?}"
    );
    let render = find(&js_symbols, "render");
    assert_eq!(render.kind, SymbolKind::Method);
    assert_eq!(render.container.as_deref(), Some("Widget"));
    assert_eq!(find(&js_symbols, "helper").kind, SymbolKind::Function);
    assert_eq!(find(&js_symbols, "arrow").kind, SymbolKind::Function);
    let js_imports = index.imports_of("src/widget.js").unwrap();
    assert!(js_imports.iter().any(|i| i.target == "./dep"));

    // Python: class + method, function, module-level constant.
    let py_symbols = index.symbols_in("src/greeter.py").unwrap();
    assert_eq!(find(&py_symbols, "Greeter").kind, SymbolKind::Class);
    let greet_py = find(&py_symbols, "greet");
    assert_eq!(greet_py.kind, SymbolKind::Function);
    assert_eq!(greet_py.container.as_deref(), Some("Greeter"));
    assert_eq!(find(&py_symbols, "helper").kind, SymbolKind::Function);
    assert_eq!(find(&py_symbols, "CONST").kind, SymbolKind::Const);
    let py_imports = index.imports_of("src/greeter.py").unwrap();
    assert!(py_imports.iter().any(|i| i.target == "os"));
    assert!(py_imports.iter().any(|i| i.target == "collections"));

    // Language detection landed correctly for each fixture file too.
    let lang_of = |path: &str| files.iter().find(|f| f.path == path).unwrap().language;
    assert_eq!(lang_of("src/lib.rs"), Language::Rust);
    assert_eq!(lang_of("src/greeter.ts"), Language::TypeScript);
    assert_eq!(lang_of("src/props.tsx"), Language::Tsx);
    assert_eq!(lang_of("src/widget.js"), Language::JavaScript);
    assert_eq!(lang_of("src/greeter.py"), Language::Python);

    // definitions() does exact-name lookup across the whole index.
    let defs = index.definitions("Widget", 10).unwrap();
    assert!(defs
        .iter()
        .any(|s| s.path == "src/lib.rs" && s.kind == SymbolKind::Struct));
    assert!(defs
        .iter()
        .any(|s| s.path == "src/widget.js" && s.kind == SymbolKind::Class));

    let stats = index.stats().unwrap();
    assert_eq!(stats.files, files.len());
    assert!(stats.chunks > 0);
    assert!(stats.symbols > 0);
    assert!(stats.bytes > 0);
}

#[test]
fn update_paths_reindexes_only_the_changed_file() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    index.refresh().unwrap();
    let before = index.stats().unwrap();

    let lib_rs = dir.path().join("src/lib.rs");
    fs::write(
        &lib_rs,
        r#"pub fn only_this_function_should_exist() {}
"#,
    )
    .unwrap();

    let report = index.update_paths(std::slice::from_ref(&lib_rs)).unwrap();
    assert_eq!(report.reindexed, 1);
    assert_eq!(report.removed, 0);
    assert_eq!(report.skipped, 0);

    let symbols = index.symbols_in("src/lib.rs").unwrap();
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "only_this_function_should_exist");

    // Every other file's symbols/chunks were untouched.
    let after = index.stats().unwrap();
    assert_eq!(after.files, before.files);

    // Calling update_paths again with the same (now unchanged) path re-reads
    // (size/mtime still decide that) but does not re-index (hash matches).
    let report2 = index.update_paths(&[lib_rs]).unwrap();
    assert_eq!(report2.reindexed, 0);
}

#[test]
fn update_paths_removes_a_deleted_file() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    index.refresh().unwrap();

    let lib_rs = dir.path().join("src/lib.rs");
    fs::remove_file(&lib_rs).unwrap();

    let report = index.update_paths(&[lib_rs]).unwrap();
    assert_eq!(report.removed, 1);
    assert_eq!(report.reindexed, 0);

    assert!(index.symbols_in("src/lib.rs").unwrap().is_empty());
    assert!(!index
        .files()
        .unwrap()
        .iter()
        .any(|f| f.path == "src/lib.rs"));
}

#[test]
fn update_paths_ignores_gitignored_and_outside_root_paths() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    index.refresh().unwrap();

    let ignored = dir.path().join("ignored.rs");
    let outside = tempdir().unwrap();
    let outside_file = outside.path().join("outside.rs");
    fs::write(&outside_file, "pub fn outside() {}\n").unwrap();

    let report = index.update_paths(&[ignored, outside_file]).unwrap();
    assert_eq!(report.skipped, 2);
    assert_eq!(report.reindexed, 0);
    assert!(!index
        .files()
        .unwrap()
        .iter()
        .any(|f| f.path == "ignored.rs"));
}

#[test]
fn search_chunks_finds_indexed_text_and_ranks_by_relevance() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    index.refresh().unwrap();

    let results = index.search_chunks(&["Widget"], 20).unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|c| c.path == "src/lib.rs"));
    // bm25 is negated so that higher is better, per the public contract.
    for pair in results.windows(2) {
        assert!(pair[0].score >= pair[1].score);
    }
}

#[test]
fn search_chunks_treats_fts5_operators_as_literal_text() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let mut index = Index::open_in_memory(dir.path()).unwrap();
    index.refresh().unwrap();

    // None of these should raise an FTS5 syntax error; each is just text
    // that (mostly) won't be found, which is fine — the point is `Ok(_)`.
    for term in ["\"", "*", "OR", "-", "NEAR(", "col:value"] {
        let result = index.search_chunks(&[term], 5);
        assert!(result.is_ok(), "term {term:?} caused an error: {result:?}");
    }
}

#[test]
fn index_db_path_is_stable_and_lives_inside_the_app_data_dir() {
    let app_data = tempdir().unwrap();
    let workspace = tempdir().unwrap();
    let a = index_db_path(app_data.path(), workspace.path());
    let b = index_db_path(app_data.path(), workspace.path());
    assert_eq!(a, b, "index_db_path must be stable for the same inputs");
    assert!(a.starts_with(app_data.path().join("index")));
    assert_eq!(a.extension().unwrap(), "sqlite");
    let stem = a.file_stem().unwrap().to_str().unwrap();
    assert_eq!(stem.len(), 16, "expected 16 hex chars, got {stem:?}");
    assert!(stem.chars().all(|c| c.is_ascii_hexdigit()));

    // A different workspace root hashes to a different path.
    let other = tempdir().unwrap();
    let c = index_db_path(app_data.path(), other.path());
    assert_ne!(a, c);
}

#[test]
fn search_live_matches_substrings_references_are_whole_word() {
    let dir = tempdir().unwrap();
    write(dir.path(), "a.txt", "foo\nfoobar\nbaz foo qux\n");

    let substrings = search_live(dir.path(), "foo", false, 100).unwrap();
    assert_eq!(
        substrings.len(),
        3,
        "literal search should match all 3 lines containing `foo`"
    );

    let whole_word = references(dir.path(), "foo", 100).unwrap();
    assert_eq!(
        whole_word.len(),
        2,
        "references should skip `foobar`: {whole_word:?}"
    );
    assert!(whole_word.iter().all(|m| m.text != "foobar"));
}

#[test]
fn search_live_regex_mode_and_literal_mode() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "a.txt",
        "fn one() {}\nfn two() {}\nlet x = 1;\n",
    );

    let regex_matches = search_live(dir.path(), r"^fn \w+\(\)", true, 100).unwrap();
    assert_eq!(regex_matches.len(), 2);

    // In literal mode, regex metacharacters are inert.
    let literal_matches = search_live(dir.path(), "fn \\w+", false, 100).unwrap();
    assert!(literal_matches.is_empty());
}

#[test]
fn search_live_respects_gitignore_and_skips_dot_git() {
    let dir = tempdir().unwrap();
    build_fixture(dir.path());
    let matches = search_live(dir.path(), "should_not_be_indexed", false, 100).unwrap();
    assert!(
        matches.is_empty(),
        "search_live must not see gitignored content: {matches:?}"
    );
}

fn assert_send<T: Send>() {}

#[test]
fn index_is_send() {
    assert_send::<Index>();
}

// --- security review, 2026-09-24: what the Low-risk code tools and the automatic
// context can reach. Each test reproduces a finding against the real index.

fn indexed(index: &Index) -> Vec<String> {
    index.files().unwrap().into_iter().map(|f| f.path).collect()
}

#[test]
fn secret_files_are_neither_indexed_nor_searched() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write(root, "app.py", "TOKEN_NAME = 'SECRETWORD'\n");
    write(root, "secrets.yaml", "password: SECRETWORD\n");
    write(root, "prod.env", "API_KEY=SECRETWORD\n");
    write(root, "server.key", "SECRETWORD\n");
    let mut index = Index::open_in_memory(root).unwrap();
    index.refresh().unwrap();
    assert_eq!(indexed(&index), ["app.py"]);
    let hits: Vec<String> = search_live(root, "SECRETWORD", false, 50)
        .unwrap()
        .into_iter()
        .map(|m| m.path)
        .collect();
    assert_eq!(hits, ["app.py"]);
    // Nor through the watcher.
    index
        .update_paths(&[root.join("secrets.yaml"), root.join("server.key")])
        .unwrap();
    assert_eq!(indexed(&index), ["app.py"]);
}

#[test]
fn the_watcher_skips_what_the_full_scan_skips() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write(root, "main.py", "print(1)\n");
    write(root, "apps/api/.gitignore", "local.json\n");
    let mut index = Index::open_in_memory(root).unwrap();
    index.refresh().unwrap();
    let dotenv = write(root, ".env.local", "X=1\n");
    let nested = write(root, "apps/api/local.json", "{\"key\": 1}\n");
    let hidden_dir = write(root, ".config/tool.toml", "a = 1\n");
    let kept = write(root, "apps/api/server.py", "print(2)\n");
    index
        .update_paths(&[dotenv, nested, hidden_dir, kept])
        .unwrap();
    let mut paths = indexed(&index);
    paths.sort();
    assert_eq!(paths, ["apps/api/server.py", "main.py"]);
}

#[cfg(unix)]
#[test]
fn a_file_swapped_for_a_symlink_leaves_the_index() {
    let dir = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let root = dir.path();
    let secret = write(outside.path(), "credentials", "OUTSIDE_SECRET\n");
    let config = write(root, "config.md", "harmless\n");
    let mut index = Index::open_in_memory(root).unwrap();
    index.refresh().unwrap();
    assert_eq!(indexed(&index), ["config.md"]);
    fs::remove_file(&config).unwrap();
    std::os::unix::fs::symlink(&secret, &config).unwrap();
    let report = index.update_paths(&[config]).unwrap();
    assert_eq!(report.removed, 1);
    assert!(indexed(&index).is_empty());
}

#[test]
fn search_truncates_long_lines_and_skips_oversized_files() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write(root, "min.js", &format!("NEEDLE{}\n", "x".repeat(10_000)));
    write(
        root,
        "huge.txt",
        &format!(
            "NEEDLE\n{}",
            "y".repeat(crate::walk::MAX_FILE_BYTES as usize)
        ),
    );
    let hits = search_live(root, "NEEDLE", false, 50).unwrap();
    assert_eq!(
        hits.len(),
        1,
        "{:?}",
        hits.iter().map(|h| &h.path).collect::<Vec<_>>()
    );
    assert_eq!(hits[0].path, "min.js");
    assert!(hits[0].text.chars().count() <= 401);
    assert!(search_live(root, &"a".repeat(1001), false, 50).is_err());
}

// --- security re-verification, 2026-09-25

#[test]
fn the_watcher_honours_ignore_files_above_the_root_and_ignore_over_gitignore() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    write(repo.path(), ".gitignore", "apps/api/config.local.json\n");
    let root = repo.path().join("apps");
    write(&root, "api/server.py", "print(1)\n");
    write(&root, ".ignore", "prec.txt\n");
    write(&root, ".gitignore", "!prec.txt\n");
    let mut index = Index::open_in_memory(&root).unwrap();
    index.refresh().unwrap();
    let local = write(&root, "api/config.local.json", "{\"token\": 1}\n");
    let prec = write(&root, "prec.txt", "x\n");
    index.update_paths(&[local, prec]).unwrap();
    assert_eq!(indexed(&index), ["api/server.py"]);
}

#[cfg(unix)]
#[test]
fn a_directory_swapped_for_a_symlink_takes_its_rows_along() {
    let dir = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let root = dir.path();
    write(root, "sub/a.txt", "a\n");
    write(root, "sub/b.txt", "b\n");
    let mut index = Index::open_in_memory(root).unwrap();
    index.refresh().unwrap();
    fs::remove_dir_all(root.join("sub")).unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join("sub")).unwrap();
    let report = index.update_paths(&[root.join("sub")]).unwrap();
    assert_eq!(report.removed, 2);
    assert!(indexed(&index).is_empty());
}

#[cfg(unix)]
#[test]
fn one_unreadable_file_does_not_fail_the_batch() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir().unwrap();
    let root = dir.path();
    write(root, "gone.md", "old\n");
    let mut index = Index::open_in_memory(root).unwrap();
    index.refresh().unwrap();
    let locked = write(root, "locked.txt", "x\n");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    fs::remove_file(root.join("gone.md")).unwrap();
    let result = index.update_paths(&[locked.clone(), root.join("gone.md")]);
    let refreshed = index.refresh();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(result.is_ok(), "{result:?}");
    assert!(refreshed.is_ok(), "{refreshed:?}");
    assert!(indexed(&index).is_empty());
}
