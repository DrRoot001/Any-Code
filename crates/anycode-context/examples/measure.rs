//! Phase 4 exit test (docs/ROADMAP.md): build a context package for a real instruction over
//! a real repository and print what it chose, why, and what fraction of the repository's
//! tokens that is. Uses an in-memory index so it leaves nothing behind.
//!
//! cargo run -p anycode-context --example measure -- <repo> "<instruction>"

use anycode_code_intelligence::Index;
use anycode_context::{build, describe, ContextRequest, DEFAULT_BUDGET_TOKENS};
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(repo), Some(instruction)) = (args.next(), args.next()) else {
        eprintln!("usage: measure <repo> \"<instruction>\"");
        std::process::exit(2);
    };

    let started = Instant::now();
    let mut index = Index::open_in_memory(repo.as_ref()).expect("open index");
    index.refresh().expect("index repository");
    let indexed_ms = started.elapsed().as_millis();

    let started = Instant::now();
    let package = build(
        &index,
        &ContextRequest {
            instruction: &instruction,
            budget_tokens: DEFAULT_BUDGET_TOKENS,
            changed_paths: &[],
        },
    )
    .expect("build context");
    let built_ms = started.elapsed().as_millis();

    println!("instruction: {instruction}");
    println!("intent: {:?}", package.intent);
    println!(
        "repository: {} files, ~{} tokens (estimated) · indexed in {indexed_ms} ms",
        package.repo_files, package.repo_est_tokens
    );
    println!(
        "package: {} items, ~{} of {} budget tokens ({:.2}% of the repository) · built in {built_ms} ms",
        package.items.len(),
        package.est_tokens,
        package.budget_tokens,
        100.0 * package.est_tokens as f64 / package.repo_est_tokens.max(1) as f64
    );
    for item in &package.items {
        let reasons: Vec<String> = item.reasons.iter().map(describe).collect();
        let range = if item.outline {
            "outline".to_string()
        } else {
            format!("{}-{}", item.start_line, item.end_line)
        };
        println!(
            "  + {}:{range} ~{} — {}",
            item.path,
            item.est_tokens,
            reasons.join("; ")
        );
    }
    for excluded in &package.excluded {
        println!(
            "  - {}:{}-{} ~{} — {}",
            excluded.path,
            excluded.start_line,
            excluded.end_line,
            excluded.est_tokens,
            excluded.why
        );
    }
}
