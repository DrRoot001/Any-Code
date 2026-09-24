//! FTS5 query construction. Every term is quoted as an FTS5 string literal so
//! user-supplied text can never inject query syntax (column filters, `NEAR()`,
//! boolean operators, prefix `*`, unary `-`, or an unbalanced quote that would
//! otherwise raise a syntax error).

/// Builds `"t1" OR "t2" OR ...` from `terms`, quoting (and escaping embedded
/// quotes in) each one. Empty `terms` yields an empty string — callers should
/// treat that as "no results" rather than querying FTS5 with it.
pub fn build_or_query(terms: &[&str]) -> String {
    terms
        .iter()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_and_joins_terms() {
        assert_eq!(build_or_query(&["foo", "bar"]), "\"foo\" OR \"bar\"");
    }

    #[test]
    fn escapes_embedded_quote() {
        assert_eq!(build_or_query(&["a\"b"]), "\"a\"\"b\"");
    }

    #[test]
    fn neutralises_fts5_operators() {
        // Each of these would be significant to the FTS5 query parser if left
        // unquoted; quoting turns every one into inert literal text.
        for term in ["*", "OR", "-", "NEAR(", "\"", "col:value"] {
            let query = build_or_query(&[term]);
            assert!(query.starts_with('"') && query.ends_with('"'));
        }
    }
}
