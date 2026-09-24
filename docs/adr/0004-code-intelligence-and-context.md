# 4. Code intelligence, context and memory

- **Status:** Accepted
- **Date:** 2026-09-24

## Context

Phase 4's exit condition is that agents retrieve targeted repository context instead of
dumping files (PRD §35–40, docs/ROADMAP.md). Today the planner sees a top-level directory
listing and the agent discovers everything else by reading whole files. Several choices here
are costly to reverse — where the index lives, what counts as trusted, how memory reaches a
prompt — so they are fixed up front.

## Decision

**Crates (PRD §94).** `anycode-code-intelligence` owns the index: the gitignore-aware walk,
SQLite storage, tree-sitter symbol extraction, full-text and live search, and the LSP client.
`anycode-context` owns the context builder, which turns a task into a context package, and
the assembly of memory into it. Neither knows provider names; neither executes tools.

**Index storage.** One SQLite database per workspace, in the app data directory
(`index/<first 16 hex of sha256(canonical workspace path)>.sqlite`). It is a derived,
rebuildable cache: never written into the repository, never synced, safe to delete
(invariant #8). SQLite's FTS5 — compiled into the bundled SQLite — does full-text search;
there is no graph database (PRD §35).

**Schema.**
- `files`: path, size, mtime, content hash, language, line count.
- `chunks` plus an FTS5 index over them: files split into line ranges, so retrieval returns a
  passage rather than a whole file.
- `symbols`: definitions — name, kind, line range, the enclosing symbol.
- `imports`: what each file imports, as written.

Call-graph edges are left out until something consumes them — an empty table is scaffolding.

**Freshness.** Opening a workspace refreshes the index. A file's size and mtime decide
whether it is re-read, and its content hash decides whether it is re-indexed. After that, a
debounced file watcher re-indexes only changed paths. Target (PRD §70): a changed file is
re-indexed in ≤ 1 s. Indexing runs off the UI thread.

**Retrieval** follows PRD §37's stages: intent (identifiers, paths and terms taken from the
instruction) → symbol lookup → lexical (FTS5, bm25) → structural (imports, and the files
defining referenced symbols) → git (files changed in the working tree) → rerank → token
budget. The output is a context package. Every item records **why** it was included, and
everything left out that scored records why it was excluded. Token counts are estimates —
bytes ÷ 4 — and are labelled as such everywhere they are shown.

**Trust.** All repository content is untrusted data. That includes `CLAUDE.md`, `AGENTS.md`
and `.anycode/rules.md`: a repository cloned from anywhere must not be able to instruct the
agent. Context reaches the model inside the `<untrusted>` envelope with its origin.
Instruction files found in a repository are listed in the Memory view; the user can
**adopt** one, which copies it into workspace memory, where it becomes user-authored.
`.anycode/permissions.yaml` is honoured **only in the tightening direction**: protected paths
and approval-required actions can raise risk, and nothing in a repository can lower it.

**Memory (PRD §38).** Global and workspace memories live in `anycode-store`. Every memory is
visible, editable, deletable and exportable. They are the user's own words, so they reach the
model as user text. Nothing is written to memory without the user doing it — no inferred
profiling. Session and task scope is the task's own history.

**Session resume.** The audit log is the source of truth. Each completed model turn's text is
recorded as a bounded `task.turn` event, so a past task's timeline can be rebuilt after a
restart without a second history store.

**Language servers.** Only servers already on the user's `PATH` are started, and nothing is
downloaded silently. They speak JSON-RPC over stdio. A missing server is reported plainly.
`.vsix` language servers (ADR 0003) are third-party executables and go through the same
approval as any other.

## Consequences

- The agent gains read-only, Low-risk tools — `code.search`, `code.definition` and
  `code.references`, already classified in `anycode-security` — and a context package in
  its prompt instead of a directory listing.
- Index size scales with the repository; it lives outside the repository and can be rebuilt
  at any time.
- Adopting instruction files is one deliberate click, which is the price of not letting any
  cloned repository instruct the agent.
- The exit test is measured on this repository and recorded in REVIEW.md: the fraction of
  repository tokens sent, and whether the files the change needs were included.
