//! Symbol and import extraction.
//!
//! Symbols come from each tree-sitter grammar's own bundled `TAGS_QUERY` —
//! the same ctags-style query the grammar ships for editor "go to symbol"
//! support — rather than a hand-rolled query per language. That query tags
//! definitions as `@definition.function`, `@definition.class`, etc. (a
//! ctags-style label, not our [`SymbolKind`] directly), paired with a
//! `@name` capture and sometimes an enclosing-scope capture we also reuse for
//! [`Symbol::container`].
//!
//! Imports are extracted with a per-line scan rather than a query: the ADR
//! asks for imports "as written", and a grammar's own import syntax varies
//! too much (bare `import "x"`, `import {a} from "x"`, `require("x")`,
//! Python's `from . import x`) to be worth a shared query. ponytail: this
//! only looks at single physical lines, so a `use`/`import` statement that
//! wraps across lines is missed — upgrade to a tree-sitter node walk if that
//! turns out to matter in practice.

use crate::{Import, Language, Symbol, SymbolKind};
use tree_sitter::{Parser, Query, QueryCursor, QueryMatch, QueryPredicateArg, StreamingIterator};

fn ts_language(language: Language) -> Option<tree_sitter::Language> {
    match language {
        Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        Language::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
        Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
        Language::Other => None,
    }
}

fn tags_query_source(language: Language) -> Option<&'static str> {
    match language {
        Language::Rust => Some(tree_sitter_rust::TAGS_QUERY),
        Language::TypeScript | Language::Tsx => Some(tree_sitter_typescript::TAGS_QUERY),
        Language::JavaScript => Some(tree_sitter_javascript::TAGS_QUERY),
        Language::Python => Some(tree_sitter_python::TAGS_QUERY),
        Language::Other => None,
    }
}

/// Maps a tags-query capture (its ctags-style label, plus the tagged node's
/// own grammar kind for the cases the label alone doesn't disambiguate) onto
/// our [`SymbolKind`]. Rust's `@definition.class` covers structs, enums,
/// unions *and* type aliases (ctags has no separate label for them), so that
/// one arm also looks at `node_kind`.
fn classify(language: Language, tag: &str, node_kind: &str) -> Option<SymbolKind> {
    use SymbolKind::{
        Class, Const, Enum, Function, Interface, Method, Module, Struct, Trait, Type,
    };
    match language {
        Language::Rust => match tag {
            "definition.function" => Some(Function),
            "definition.method" => Some(Method),
            "definition.interface" => Some(Trait), // trait_item
            "definition.module" => Some(Module),
            "definition.class" => match node_kind {
                "struct_item" | "union_item" => Some(Struct),
                "enum_item" => Some(Enum),
                "type_item" => Some(Type),
                _ => None,
            },
            _ => None,
        },
        Language::TypeScript | Language::Tsx => match tag {
            "definition.function" => Some(Function),
            "definition.method" => Some(Method),
            "definition.class" => Some(Class),
            "definition.interface" => Some(Interface),
            "definition.module" => Some(Module),
            _ => None,
        },
        Language::JavaScript => match tag {
            "definition.function" => Some(Function),
            "definition.method" => Some(Method),
            "definition.class" => Some(Class),
            "definition.constant" => Some(Const),
            _ => None,
        },
        Language::Python => match tag {
            "definition.function" => Some(Function),
            "definition.class" => Some(Class),
            "definition.constant" => Some(Const),
            _ => None,
        },
        Language::Other => None,
    }
}

/// Anything that names a scope another symbol can sit inside — every
/// definition, plus Rust's `impl Type { .. }` block (tagged
/// `@reference.implementation` by the grammar's own query because it is a
/// use of the type name, not a new definition of it, but it is exactly the
/// span a method's `container` should resolve to).
fn is_container_candidate(tag: &str) -> bool {
    tag.starts_with("definition.") || tag == "reference.implementation"
}

/// tree-sitter's `Query` parses `#eq?`/`#not-eq?`/etc predicates but does not
/// apply them — that's on every caller. The only one our four grammars' tags
/// queries rely on for correctness is JavaScript's
/// `(#not-eq? @name "constructor")` (everything else is `#strip!`/
/// `#select-adjacent!`, which only shape the `@doc` capture we never read).
/// Handling `eq?`/`not-eq?` generically covers that without hard-coding
/// "constructor" as special knowledge here.
fn predicates_hold(query: &Query, source: &[u8], m: &QueryMatch) -> bool {
    for predicate in query.general_predicates(m.pattern_index) {
        let is_eq = match predicate.operator.as_ref() {
            "eq?" => true,
            "not-eq?" => false,
            _ => continue,
        };
        let [a, b] = predicate.args.as_ref() else {
            continue;
        };
        let resolve = |arg: &QueryPredicateArg| -> Option<String> {
            match arg {
                QueryPredicateArg::String(s) => Some(s.to_string()),
                QueryPredicateArg::Capture(idx) => m
                    .captures
                    .iter()
                    .find(|c| c.index == *idx)
                    .and_then(|c| c.node.utf8_text(source).ok())
                    .map(str::to_owned),
            }
        };
        if (resolve(a) == resolve(b)) != is_eq {
            return false;
        }
    }
    true
}

struct Candidate {
    name: String,
    node_kind: &'static str,
    tag: String,
    start_line: u32,
    end_line: u32,
}

/// Runs `language`'s bundled tags query over `source` and returns the
/// definitions it finds as [`Symbol`]s (already stamped with `path`), each
/// with `container` resolved to its tightest enclosing candidate.
pub fn extract_symbols(language: Language, source: &str, path: &str) -> Vec<Symbol> {
    let Some(ts_lang) = ts_language(language) else {
        return Vec::new();
    };
    let Some(query_src) = tags_query_source(language) else {
        return Vec::new();
    };
    let mut parser = Parser::new();
    if parser.set_language(&ts_lang).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let Ok(query) = Query::new(&ts_lang, query_src) else {
        return Vec::new();
    };
    let names = query.capture_names();

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while let Some(m) = matches.next() {
        if !predicates_hold(&query, source.as_bytes(), m) {
            continue;
        }
        let mut name_text: Option<String> = None;
        let mut tagged: Option<(&'static str, &str, u32, u32)> = None;
        for capture in m.captures {
            let capture_name = names[capture.index as usize];
            if capture_name == "name" {
                name_text = capture
                    .node
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(str::to_owned);
            } else if is_container_candidate(capture_name) {
                tagged = Some((
                    capture.node.kind(),
                    capture_name,
                    capture.node.start_position().row as u32 + 1,
                    capture.node.end_position().row as u32 + 1,
                ));
            }
        }
        if let (Some(name), Some((node_kind, tag, start_line, end_line))) = (name_text, tagged) {
            candidates.push(Candidate {
                name,
                node_kind,
                tag: tag.to_string(),
                start_line,
                end_line,
            });
        }
    }

    let mut symbols: Vec<Symbol> = candidates
        .iter()
        .filter_map(|c| {
            classify(language, &c.tag, c.node_kind).map(|kind| Symbol {
                name: c.name.clone(),
                kind,
                path: path.to_string(),
                start_line: c.start_line,
                end_line: c.end_line,
                container: None,
            })
        })
        .collect();
    // Rust's tags query tags a method both `definition.method` and `definition.function`:
    // one symbol per definition, the more specific kind kept.
    symbols.sort_by_key(|s| {
        (
            s.start_line,
            s.end_line,
            s.name.clone(),
            s.kind != SymbolKind::Method,
        )
    });
    symbols.dedup_by(|a, b| {
        a.start_line == b.start_line && a.end_line == b.end_line && a.name == b.name
    });

    for symbol in &mut symbols {
        let self_span = symbol.end_line.saturating_sub(symbol.start_line);
        let mut best: Option<(&str, u32)> = None;
        for c in &candidates {
            if !is_container_candidate(&c.tag) {
                continue;
            }
            if c.start_line > symbol.start_line || c.end_line < symbol.end_line {
                continue;
            }
            let span = c.end_line.saturating_sub(c.start_line);
            if span == self_span && c.name == symbol.name {
                continue; // the symbol's own definition, not an enclosing one
            }
            if best.is_none_or(|(_, best_span)| span < best_span) {
                best = Some((c.name.as_str(), span));
            }
        }
        symbol.container = best.map(|(name, _)| name.to_string());
    }

    symbols
}

fn quoted_after(line: &str, marker: &str) -> Option<String> {
    let idx = line.find(marker)?;
    let rest = line[idx + marker.len()..].trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let after_quote = &rest[quote.len_utf8()..];
    let end = after_quote.find(quote)?;
    Some(after_quote[..end].to_string())
}

/// Per-line, best-effort import extraction (see module docs for the scope
/// this deliberately doesn't cover).
pub fn extract_imports(language: Language, source: &str, path: &str) -> Vec<Import> {
    let mut out = Vec::new();
    for (idx, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim_start();
        let line_no = idx as u32 + 1;
        let target = match language {
            Language::Rust => {
                if line.starts_with("//") {
                    None
                } else {
                    line.strip_prefix("use ")
                        .map(|rest| rest.trim_end_matches(';').trim().to_string())
                }
            }
            Language::TypeScript | Language::Tsx | Language::JavaScript => {
                if line.starts_with("//") || line.starts_with('*') {
                    None
                } else if line.starts_with("import") {
                    quoted_after(line, "from").or_else(|| quoted_after(line, "import"))
                } else {
                    quoted_after(line, "from").or_else(|| quoted_after(line, "require("))
                }
            }
            Language::Python => {
                if line.starts_with('#') {
                    None
                } else if let Some(rest) = line.strip_prefix("import ") {
                    rest.split([',', ' ']).next().map(|s| s.trim().to_string())
                } else if let Some(rest) = line.strip_prefix("from ") {
                    rest.split(" import").next().map(|s| s.trim().to_string())
                } else {
                    None
                }
            }
            Language::Other => None,
        };
        if let Some(target) = target.filter(|t| !t.is_empty()) {
            out.push(Import {
                path: path.to_string(),
                target,
                line: line_no,
            });
        }
    }
    out
}
