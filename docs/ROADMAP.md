# Roadmap

Phases from [PRD.md](../PRD.md) §98. A phase is done when its **exit condition** is demonstrated on
a real repository — not when its files exist.

Release boundaries and rules for admitting work are defined in
[PRODUCT-SCOPE.md](PRODUCT-SCOPE.md). V1 covers Phases 0–8; V1.5 begins with Phase 9; V2 begins
with Phase 10. Only the current phase is active implementation scope.

Phases are sequential for a reason: each one is the foundation the next assumes. Building Phase 5
parallelism before Phase 3's approval system means shipping unbounded agents with no brakes.

| Phase | Ships | Exit condition |
|-------|-------|----------------|
| **0 · Foundation** | Monorepo, Tauri + React shell, Rust core, CI, lint/format/test, SQLite, event system, design tokens, themes | App launches on Windows and macOS |
| **1 · Workbench** | Workspace selection, explorer, Monaco, tabs, terminal, git status, diff, command palette, settings | Usable as a lightweight coding environment without AI |
| **2 · Provider layer** | Provider abstraction, OpenAI, Anthropic, Gemini, OpenRouter, Ollama, streaming, model selector, credential vault, usage events | Same task switches providers with no change outside the adapter |
| **3 · Agent runtime** | Planner, task state machine, tool calls, filesystem/terminal/git tools, approvals, timeline, verification | Agent implements *and verifies* a simple repository task |
| **4 · Code intelligence** | tree-sitter, ripgrep, LSP, symbol index, SQLite FTS, context builder, context inspector, memory | Agents retrieve targeted context instead of dumping files |
| **5 · Multi-agent** | Task DAG, subagents, git worktrees, parallel execution, merge coordinator, agent dashboard | Two agents work concurrently without corrupting the workspace |
| **6 · Browser verification** | Playwright, browser panel, screenshots, console, network, DOM, responsive checks | A frontend agent proves its work |
| **7 · Capability platform** | MCP, connectors, skills, plugin host, capability permissions, install UX | All external capabilities flow through one registry |
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

## Phase 1 · Workbench

- [x] Workspace selection (native folder picker, persisted and restored across restarts)
- [x] File explorer (lazy-expanding tree, scoped to the workspace root — `anycode-fs`)
- [x] Monaco editor (lazy-loaded, self-hosted workers, no CDN)
- [x] Tabs (multi-file, dirty tracking, Cmd/Ctrl+S to save)
- [x] Terminal (native PTY via `anycode-terminal`, streamed over Tauri events)
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

Not built in Phase 2 (deferred, per PRD §9.1/§26 these belong to later phases): Gemini and
OpenRouter adapters (the abstraction is proven with three; add the rest when a task needs
them), automatic/cost-aware routing (Model Router, Phase 8), fallback chains (PRD §27),
budget controls (Phase 8), the full Agent Dock (Phase 3) — the Chat panel is a thin proof
of the exit condition, not the agent runtime's conversation surface.

**Exit condition met:** the Chat panel's code has no branch on provider identity — only
which two dropdown values are selected. Verified locally with the OpenAI/Anthropic
request-building and response-parsing logic (27 unit tests); live network calls weren't
exercised in the build environment (no API keys present there), but the code path from UI
to Rust command to adapter is real, not mocked.

## Current phase: 3 · Agent runtime

- [x] Permission engine (`anycode-security`) — risk levels, allow/ask/deny, Critical
      never overridable, unknown capabilities default to Medium
- [x] Capability registry (`anycode-tools`) — the only path from a model-originated
      request to the filesystem, git, or a shell
- [x] Tool-calling in the OpenAI adapter (streaming fragments reassembled by index)
- [x] Orchestration loop — every tool call gated before execution; results, including
      denials, fed back to the model
- [x] Approval UI — exact command/path, risk level, workspace scope; no global allow
- [x] Task timeline — streamed text, tool calls, results, failures
- [x] Cancellation (`cancel_task`) — architecture invariant #11
- [x] Evidence on completion — measured `git status` delta and real command exit codes,
      plus token totals
- [ ] Exit condition demonstrated against a live provider — the loop is built and
      unit-tested, but an end-to-end run needs an API key (see REVIEW.md)

Deferred past Phase 3 (not needed for the exit condition): a task DAG and parallel
subagents (Phase 5), tool-calling for Anthropic and Ollama (the abstraction is proven
with one adapter; the others still honestly declare no tool support), and durable task
history across restarts.

**On verification:** the runtime records what it observed — which files `git status`
reports as newly dirty, and what exit code each `shell.execute` returned — separately
from anything the model says. A task where no command ran is reported as *unverified*,
never as success. The model is instructed to verify its own work, but that instruction
cannot set the flag; only real exit codes can.

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

Desktop packaging belongs to Phase 0. Android and iOS jobs must not be presented as supported
release jobs before Phase 10 supplies real mobile application targets. CI may use unsigned
artifacts on pull requests, but anything labeled a release must be signed and traceable to its
source commit.

## V1 success criterion

A new user installs Any Code, opens a repository, connects one provider, asks for a feature,
watches execution, reviews commands and diffs, runs tests, sees the cost, restarts the app and
resumes the session — with no configuration outside Any Code beyond provider authorisation.
