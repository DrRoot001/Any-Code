# Roadmap

Phases from [PRD.md](../PRD.md) §98. A phase is done when its **exit condition** is demonstrated on
a real repository — not when its files exist.

Release boundaries and rules for admitting work are defined in
[PRODUCT-SCOPE.md](PRODUCT-SCOPE.md). V1 covers Phases 0–8; V1.5 begins with Phase 9; V2 begins
with Phase 10. Only the current phase is active implementation scope.

**Where we are (audited 2026-09-24, [AUDIT.md](AUDIT.md)):** Phases 0–3 are built. Before
Phase 4 starts, the [Phase 3 close-out](#current-phase-3-close-out) fixes three security
findings in shipped code and closes the earlier phases' unmet exit conditions.

Phases are sequential for a reason: each one is the foundation the next assumes. Building Phase 5
parallelism before Phase 3's approval system means shipping unbounded agents with no brakes.

| Phase | Ships | Exit condition |
|-------|-------|----------------|
| **0 · Foundation** | Monorepo, Tauri + React shell, Rust core, CI, lint/format/test, SQLite, event system, design tokens, themes | App launches on Windows and macOS |
| **1 · Workbench** | Workspace selection, explorer, Monaco, tabs, terminal, git status, diff, command palette, settings | Usable as a lightweight coding environment without AI |
| **2 · Provider layer** | Provider abstraction, OpenAI, Anthropic, Gemini, OpenRouter, Ollama, streaming, model selector, credential vault, usage events | Same task switches providers with no change outside the adapter |
| **3 · Agent runtime** | Planner, task state machine, tool calls, filesystem/terminal/git tools, approvals, timeline, verification | Agent implements *and verifies* a simple repository task |
| **4 · Code intelligence** | tree-sitter, ripgrep, LSP, symbol index, SQLite FTS, context builder, context inspector, memory, `.vsix` themes/grammars and extension-supplied language servers ([ADR 0003](adr/0003-extension-compatibility-and-registry.md)) | Agents retrieve targeted context instead of dumping files |
| **5 · Multi-agent** | Task DAG, subagents, git worktrees, parallel execution, merge coordinator, agent dashboard | Two agents work concurrently without corrupting the workspace |
| **6 · Browser verification** | Playwright, browser panel, screenshots, console, network, DOM, responsive checks | A frontend agent proves its work |
| **7 · Capability platform** | MCP, connectors, skills, plugin host, capability permissions, install UX, Open VSX extension browsing and install as one more package category | All external capabilities flow through one registry |
| **8 · Subscription intelligence** | Usage dashboard, budgets, provider/model analytics, subscription ledger, routing policies | User can see where AI money goes and change it |
| **9 · Cloud** | Identity, cloud API, PostgreSQL, sync, devices, remote sessions, web dashboard | A session continues on another authorised device |
| **10 · Mobile** | iOS, Android, task management, diff review, approvals, usage, notifications | Real work supervised without a laptop |

## Phase 0 · Foundation

- [x] Repository, git, CI, formatting, linting, testing
- [x] Event system foundation (`anycode-core`)
- [x] Trust tagging foundation (`anycode-core`)
- [x] Architecture, standards and security documents
- [x] Design tokens + dark/light/high-contrast themes
- [x] Tauri 2 shell + React shell
- [x] SQLite local store and migrations
- [ ] Signed builds for Windows and macOS — unsigned pipeline built (`.github/workflows/desktop-release.yml`);
      blocked on the user supplying Apple/Windows signing certificates, see [RELEASING.md](RELEASING.md)
- [ ] **Exit condition on Windows** — the installer now builds in CI; it has not been
      launched on Windows yet. macOS: the CI universal build launched (close-out C4)

## Phase 1 · Workbench

- [x] Workspace selection (native folder picker, persisted and restored across restarts)
- [x] File explorer (lazy-expanding tree, scoped to the workspace root — `anycode-fs`)
- [x] Monaco editor (lazy-loaded, self-hosted workers, no CDN)
- [x] Tabs (multi-file, dirty tracking, Cmd/Ctrl+S to save)
- [x] Terminal (native PTY via `anycode-terminal`, streamed over Tauri events) — four
      defects fixed 2026-09-23; not yet confirmed by hand (close-out C5)
- [x] Git status (`anycode-git`, polled)
- [x] Diff view (Monaco diff editor, HEAD vs working tree)
- [x] Command palette (Cmd/Ctrl+Shift+P)
- [x] Settings panel (theme, workspace info)

Not built in Phase 1 (explicitly out of scope — see docs/PRODUCT-SCOPE.md): LSP/code intelligence
(Phase 4), git write operations — stage/commit/push (gated by approval, later phase), split editor
and minimap, multi-workspace switching.

## Phase 2 · Provider layer

- [x] Provider abstraction (`anycode-models`: `ModelProvider` trait, normalized request/
      stream/usage types — orchestration never references a vendor by name)
- [x] OpenAI adapter (Chat Completions streaming, live model discovery)
- [x] Anthropic adapter (Messages API streaming, live model discovery)
- [x] Ollama adapter (local, no credential, newline-delimited JSON streaming)
- [x] Credential vault (`anycode-secrets`, OS keychain via the `keyring` crate)
- [x] Usage events (`anycode-store`'s `usage_events` table — every request, success or
      failure, real token counts only, never estimated)
- [x] Model selector + Connections UI (Settings → Providers; Chat panel's provider/model
      dropdowns)

- [ ] Gemini adapter — a PRD Phase 2 deliverable, previously deferred without a home (close-out C7)
- [ ] OpenRouter — OpenAI-compatible, so a configurable base URL on the OpenAI adapter
      (also covers LM Studio and other compatible endpoints; close-out C7)
- [ ] **Exit condition demonstrated live** — previously marked met on unit tests alone. The
      Chat panel has no provider branch, but switching has never been observed across two
      live providers (close-out C7)

Deferred to later phases: automatic/cost-aware routing (Phase 8), fallback chains (PRD §27),
budget controls (Phase 8).

## Phase 3 · Agent runtime

PRD §3587 deliverables: planner, task state machine, tool calls, filesystem/terminal/git
tools, approval system, event timeline, verification. Exit condition: *an agent can
implement and verify a simple repository task.*

- [x] Permission engine (`anycode-security`) — risk levels, allow/ask/deny, Critical
      never overridable, unknown capabilities default to Medium
- [x] Capability registry (`anycode-tools`) — the only path from a model-originated
      request to the filesystem, git, or a shell
- [x] Tool-calling in the OpenAI adapter (streaming fragments reassembled by index)
- [x] Tool-calling in the Ollama adapter — local, no credential
- [x] Tool names encoded for the wire (`.` → `__`) — function-calling APIs reject dotted
      names, so before this every OpenAI agent request would have been refused
- [x] Planner — a tool-less planning turn before any action; the plan is shown and then
      followed, and authorises nothing
- [x] Task state machine (`anycode-agent`) — `created → planning → running ⇄
      awaiting_approval → verifying → completed | failed | cancelled`; nothing reaches
      `completed` without passing through `verifying`. PRD §30's `waiting`/`blocked`
      describe inter-task dependencies and arrive with Phase 5's scheduler
- [x] Orchestration loop — every tool call gated before execution; results, including
      denials, fed back to the model inside an `<untrusted>` envelope
- [x] Approval UI — exact command/path, risk level, workspace scope; no global allow
- [x] Task timeline — state, plan, streamed text, tool calls, results, replans, failures
- [x] Audit log — every state change, tool call, approval decision and result appended to
      the store's `events` table (architecture invariant #10)
- [x] Cancellation (`cancel_task`) — architecture invariant #11
- [x] Verification — the model marks which commands are checks; the runtime judges each
      check by its *latest* exit code; a task that stops with checks failing, or with
      edits never checked, is sent back a bounded number of times, then fails
- [x] Agent commands run with the user's login-shell `PATH`, so an app opened from
      Finder still finds `npm`, `cargo`, `python3`
- [x] Exit condition demonstrated end to end — a local model implemented and verified a
      task through the production loop (1 of 6 live runs; the runtime judged all 6
      correctly). Not yet re-confirmed on the final code, and reliability with a 3B model
      is low — see REVIEW.md

Deferred past Phase 3 (not needed for the exit condition): a task DAG and parallel
subagents (Phase 5), tool-calling for Anthropic (it still honestly declares no tool
support, and `run_task` refuses it rather than running a tool-less "agent"), and
restoring task history across restarts — the audit log is durable, but nothing reads it
back into the dock yet.

**On verification:** the runtime records what it observed — which files `git status`
reports as newly dirty, and what exit code each `shell.execute` returned — separately
from anything the model says. A task where no check ran is reported as *unverified*,
never as success. The model decides which commands are checks; it cannot make a failing
check pass, and `anycode_agent::verdict` alone decides the outcome.

## Current phase: 3 close-out

Work that belongs to Phases 0–3 and is not finished. It comes before Phase 4 because Phase 4
builds on all of it — and S1/S2 are security defects in code users can run today. Detail and
evidence for every item: [AUDIT.md](AUDIT.md).

**Security (first):**

- [x] **C1 · Secret-bearing paths are not low risk** (S1). Reading `.env*`, `*.pem`, `*.key`,
      `id_rsa*`, `.git/config` and similar asks every time and is never covered by a standing
      grant; private key material is denied. Test: reading `.env` in a workspace prompts.
- [x] **C2 · Standing grants cover what the user saw** (S2). Shell grants are keyed on the exact
      command; High risk is never persisted. Test: granting `npm test` does not allow
      `git push` or `npm install`.
- [x] **C3 · Content Security Policy** (S3). Replace `"csp": null` with a strict policy;
      verify Monaco, workers and the app still load in the release build. *Verified in a
      browser against the production build under the identical policy: no violations,
      Monaco renders and tokenises. Not yet observed inside the native WebView.*

**Earlier phases' exit conditions:**

- [ ] **C4 · Phase 0 on Windows** — run the Desktop Release workflow; **owner** launches the
      Windows installer. *Workflow fixed (four defects) and passing on all three platforms;
      the `.exe`/`.msi` are in draft release `app-v0.1.0`. The universal macOS build was
      launched on an Intel Mac. Remaining: a Windows launch.*
- [ ] **C5 · Terminal** — **owner** opens the Terminal panel once and confirms a prompt.
- [ ] **C6 · Phase 3 on the final code** — one passing live run of `agent_live_test`.
- [ ] **C7 · Phase 2** — configurable base URL (OpenRouter, LM Studio), Gemini adapter, then
      the same chat task switched between two live providers (**owner**: keys).
      *Built: Gemini (API key), OpenRouter, OpenAI-compatible endpoint. Open: the live
      switch — `the_same_chat_task_switches_providers_live` is written and needs a local
      Ollama; Gemini and OpenRouter need keys to verify.*

**Invariants:**

- [x] **C8 · Usage at the adapter boundary** (invariant #9) — one wrapper every provider goes
      through, instead of a record call at each call site.
- [x] **C9 · Audit provider-key changes** (invariant #10, PRD §92) — `provider.connected` /
      `provider.disconnected` events. Never the key itself.

**Found during the close-out, fixed:**

- [x] **Credential vault never persisted a key** — `keyring` 3 fell back to its in-memory
      mock, so every key saved in Settings was lost at once. Now the OS store per platform,
      verified against the real macOS Keychain.
- [x] **Chat subscribe race** — `send_chat` could emit an instant failure before the panel
      listened, leaving the chat waiting forever.

**Quality:**

- [x] **C10 · Tests** — `anycode-secrets` (currently 0); a first frontend test for the Agent
      Dock's event handling, runnable in CI.

## Phase 4 · Code intelligence (next)

Exit condition: **agents retrieve targeted repository context instead of dumping files.**
Specified by PRD §35–40 and [ADR 0003](adr/0003-extension-compatibility-and-registry.md).
Crates, per PRD §94: `anycode-code-intelligence` (index, search, symbols, LSP) and
`anycode-context` (context builder, memory). The planner's top-level directory listing is the
stopgap this phase replaces.

In build order — each step is usable, and tested, before the next starts:

- [ ] **4.1 · Index foundation** — gitignore-aware walk; SQLite `files`, `symbols`,
      `symbol_edges`, `imports`, `search_index` (FTS5); incremental re-index on filesystem
      events, changed files only. Target (PRD §70): a changed file re-indexed in ≤ 1 s.
- [ ] **4.2 · Lexical search** — ripgrep's library as the `code.search` tool (already Low risk
      in the permission table).
- [ ] **4.3 · Symbols** — tree-sitter for TypeScript/JavaScript, Rust and Python first;
      `code.definition` and `code.references` tools (already in the permission table).
- [ ] **4.4 · Context builder** — PRD §37 stages: intent → symbols → lexical → structural →
      git → rerank → token budget, producing a context package where every item records why it
      was included. Token counts are labelled estimates, never presented as exact.
- [ ] **4.5 · Context Inspector** — for each task: which files the agent saw, why, what was
      excluded, estimated tokens (PRD §37). Built on `Tagged.origin`, which exists for this.
- [ ] **4.6 · Memory and workspace rules** — scopes: global, workspace, repository, session,
      task; every memory visible, editable, deletable, exportable (PRD §38). `.anycode/`
      rules (§39); protected paths and approval-required actions feed the permission engine.
      Import CLAUDE.md, AGENTS.md and similar as repository memory, originals untouched (§40).
- [ ] **4.7 · Session resume** — V1 contract #7: after a restart the dock shows past tasks,
      read back from the `events` audit log.
- [ ] **4.8 · LSP client** — spawn language servers found on the user's `PATH`; hover,
      definition and diagnostics for Monaco and the agent. Then ADR 0003 rung B: `.vsix`
      themes, TextMate grammars and bundled language servers from Open VSX.

**Exit test:** on a real repository larger than the model's context window (this one), an
agent task's context package contains the files the change needs and a small, measured
fraction of the repository's tokens. The Context Inspector shows why each file was
included. Measured and recorded in REVIEW.md, not asserted.

## Assigned by the 2026-09-24 audit

PRD sections that had no phase, now placed ([AUDIT.md §5](AUDIT.md#5-prd-scope-that-belongs-to-no-phase)):
Gemini/OpenRouter → close-out C7 · session resume, §39 workspace rules, §40 instruction
imports → Phase 4 · §58 changes review → Phase 5 · §91 privacy modes / Local Only, MCP config
import → Phase 7 · §92 audit log viewer → Phase 8 · §93 auto-update, crash reporting,
telemetry controls, §59/61–63 onboarding and home → Distribution gate · §69 accessibility and
§70 performance targets → every phase's definition of done, measured at the Distribution gate.

## Distribution gate

GitHub Actions is the canonical packaging path. A platform is not considered shipped until CI can
build its native installer from a clean checkout, run the applicable tests, sign it with protected
release credentials, generate checksums, and publish it as a versioned release artifact.

Expected formats as platform clients become available:

| Platform | Required release artifacts |
|----------|----------------------------|
| macOS | Signed and notarized `.dmg`, plus the updater artifact required by Tauri |
| Windows | Signed `.exe`/NSIS installer and `.msi`, plus the updater artifact |
| Linux | `.AppImage` and `.deb`; add `.rpm` when Linux support enters active scope |
| Android | Signed `.apk` for testing and `.aab` for store distribution |
| iOS | Signed archive/`.ipa` delivered through the approved Apple distribution workflow |

Also required before V1 ships (assigned by the audit): auto-update (PRD §93), opt-in crash
reporting and telemetry controls (§9.1), first-run onboarding and home screen (§61–63), and a
measured pass against the performance targets in §70.

Desktop packaging belongs to Phase 0. Android and iOS jobs must not be presented as supported
release jobs before Phase 10 supplies real mobile application targets. CI may use unsigned
artifacts on pull requests, but anything labeled a release must be signed and traceable to its
source commit.

## V1 success criterion

A new user installs Any Code, opens a repository, connects one provider, asks for a feature,
watches execution, reviews commands and diffs, runs tests, sees the cost, restarts the app and
resumes the session — with no configuration outside Any Code beyond provider authorisation.
