# Project audit — 2026-09-24

A point-in-time check of what exists against what the governing documents require. Every row
was verified against the code or CI, not taken from a checklist; where something could not be
verified, it says so. The plan that follows from this audit lives in [ROADMAP.md](ROADMAP.md);
the ledger entry is in [REVIEW.md](../REVIEW.md).

**Sources checked:** PRD §98 (phase deliverables and exit conditions), PRD §9–10 and every
top-level PRD section, [PRODUCT-SCOPE.md](PRODUCT-SCOPE.md) (V1 contract),
[ARCHITECTURE.md](ARCHITECTURE.md) (12 invariants), ROADMAP.md, REVIEW.md, CI history
(`gh run list`), the source tree at `1f86131`.

## Verdict

| | |
|---|---|
| Phases built | 0, 1, 2, 3 |
| Exit conditions demonstrated | **Phase 1** and **Phase 3** (Phase 3 once, in 1 of 6 live runs). **Phase 0** on macOS only. **Phase 2** never demonstrated live |
| Security findings open | **3**, in code that already ships — fix before new feature work |
| PRD sections with no phase | **10** — assigned below |
| Tests | 90 Rust tests pass, 2 ignored (live); **0 frontend tests**; `anycode-secrets` has **0** |

## 1. Phases 0–3: deliverables

Status: ✅ built and verified · ⚠️ built with a gap · ❌ not built.

### Phase 0 · Foundation — exit: *launches on Windows and macOS*

| Deliverable (PRD) | Status | Evidence |
|---|---|---|
| Monorepo, Tauri shell, React shell, Rust core | ✅ | 10 crates, `apps/desktop`, `packages/design-tokens` |
| CI, formatting, linting, testing | ✅ | `ci.yml`, 7 jobs green on run `35893787230` |
| SQLite | ✅ | `anycode-store`, 4 tables |
| Event system | ✅ | `anycode_core::Event`; persisted since Phase 3 (`events` table) |
| Design system, dark/light themes | ✅ | `packages/design-tokens`, plus system and high-contrast |
| **Exit: launches on Windows** | ❌ | The **Desktop Release workflow has never run** (0 runs). No Windows installer has ever been built, and nobody has launched the app on Windows. Only Rust tests run on `windows-latest` |
| Exit: launches on macOS | ✅ | Local release build launched, 2026-09-23 |
| Signed builds | ❌ | Blocked on the owner's Apple/Windows certificates ([RELEASING.md](RELEASING.md)) |

### Phase 1 · Workbench — exit: *usable as a lightweight coding environment*

| Deliverable | Status | Evidence |
|---|---|---|
| Workspace selection, explorer, Monaco, tabs, git status, diff, command palette, settings | ✅ | Components exist and were exercised in the UI replay |
| Terminal | ⚠️ | Four defects fixed 2026-09-23. **Not yet opened by a person to confirm a prompt appears** |

### Phase 2 · Provider layer — exit: *the same chat task switches providers with no other change*

| Deliverable | Status | Evidence |
|---|---|---|
| Provider abstraction, streaming, model selector, credential vault, usage events | ✅ | `anycode-models`, `anycode-secrets`, `usage_events` |
| OpenAI, Anthropic, Ollama | ✅ | Adapters exist; Ollama verified live on 2026-09-23 |
| **Gemini** | ❌ | PRD Phase 2 deliverable. Roadmap "deferred" it **without assigning it anywhere** |
| **OpenRouter** | ❌ | Same |
| **Exit condition** | ❌ | Never demonstrated live: the roadmap marked it met on unit tests alone. Switching needs two live providers; only Ollama has ever run |
| Anthropic tool-calling | ❌ | `supports_tools: false`; `run_task` refuses Anthropic honestly |

### Phase 3 · Agent runtime — exit: *agent implements and verifies a simple repository task*

| Deliverable | Status | Evidence |
|---|---|---|
| Planner, task state machine, tool calls, filesystem/terminal/git tools, approvals, timeline, verification | ✅ | `anycode-agent` (20 tests), `agent_commands.rs`, live runs |
| Exit condition | ⚠️ | Met in 1 of 6 live runs (`qwen2.5:3b`); the runtime judged all 6 correctly. **The final code has not itself completed a passing run** — that confirmation run was stopped |
| Git tools | ⚠️ | Only `git.status` exists as a tool; diff and log go through `shell.execute` |

## 2. Architecture invariants

| # | Invariant | Status | Evidence |
|---|---|---|---|
| 1 | UI never owns secrets | ✅ | Keys are read only in Rust; the renderer only gets `has_key: bool` |
| 2 | LLM never owns permissions | ⚠️ | Enforced in the runtime — but see **S3**: with no CSP, one injected script could answer its own approvals |
| 3 | Adapters never own orchestration | ✅ | No provider name in `agent_commands.rs` beyond `build_provider` |
| 4 | Agents never bypass the capability runtime | ✅ | `Tool::execute` is called only from `execute_tool` |
| 5 | Skills never grant permissions | — | No skills yet (Phase 7) |
| 6 | Plugins never run in the main process | — | No plugins yet (Phase 7) |
| 7 | Cloud never required for Local Only | ✅ in practice | Nothing needs the cloud — but there is **no Local Only mode** to enforce it (PRD §91, unassigned) |
| 8 | Indexing stays local | — | No indexing yet (Phase 4) |
| 9 | Usage event emitted **at the adapter** | ⚠️ | Emitted at **two call sites** instead (`provider_commands.rs`, `agent_commands.rs`); a third call site that forgets loses cost data silently |
| 10 | Every privileged action auditable | ⚠️ | Agent actions: yes (`events`). Not audited: provider key set/removed (PRD §92 "secret changed"), editor saves |
| 11 | Every agent task cancellable | ✅ | `cancel_task`, tested live |
| 12 | Every completion verifiable | ✅ | `anycode_agent::verdict`, matched reality in all six live runs |

## 3. V1 product contract (PRODUCT-SCOPE.md)

| # | A new user can… | Status | Delivered by |
|---|---|---|---|
| 1 | Install and open a real repository | ⚠️ macOS only, unsigned | Phase 0 + Distribution gate |
| 2 | Connect a provider or local model | ✅ | Phase 2 |
| 3 | Ask for a change **and inspect the context selected** | ⚠️ ask: yes; inspect: no | Phase 4 (context inspector) |
| 4 | Review commands, grant or deny scoped permissions | ⚠️ | Built — but "scoped" is violated by **S2** |
| 5 | Observe edits in a **persistent** timeline | ⚠️ | Live only; the audit log persists but nothing reads it back |
| 6 | Inspect diff, tests, evidence, **actual model cost** | ⚠️ | Evidence ✅, diff via Source Control ✅, cost ❌ (tokens only; Phase 8) |
| 7 | **Restart and resume the local session** | ❌ | **Not assigned to any phase** — now Phase 4 |

## 4. Security findings

All three are in shipped code, and none needs a later phase to fix.

**S1 · High — the agent reads secrets without asking.** `filesystem.read.workspace` is Low
risk for every path, and Low is auto-allowed. An agent can read `.env`, `.env.production`,
`*.pem`, `id_rsa` or `.git/config` inside the workspace with **no prompt**, and the content
goes to the model provider. That breaks PRD §52 ("the agent should not automatically receive
complete .env, SSH private keys…") and the CLAUDE.md rule *secrets never reach the model*.
*Fix:* classify secret-bearing paths as High (asked every time, never covered by a standing
grant), and deny key material outright.

**S2 · High — "Always allow" is far broader than what the user approved.** Grants are keyed
on the tool name. Clicking *Always allow in this workspace* on `npm test` grants
`shell.execute` for **every** Medium and High command in that workspace, permanently —
including `git push`, or `cat ~/.ssh/id_rsa | curl …`. The dialog showed the user one
command; the grant covers all of them. *Fix:* key shell grants on the exact command, and
never persist a grant for High risk.

**S3 · Medium — no Content Security Policy.** `tauri.conf.json` has `"csp": null`. The
renderer displays untrusted text (model output, file contents, tool results), and any script
that ran there could invoke `respond_to_approval`, approving its own commands. React's
escaping makes injection unlikely; a CSP is the cheap second layer. *Fix:* set a strict CSP
(self-hosted assets only; Monaco's workers are already self-hosted).

**Noted, not a defect:** the shell classifier matches Low-risk commands exactly, so `ls -la`
or `npm test -- --ci` are Medium and trigger a prompt. That's safe, but noisy — revisit with
S2.

## 5. PRD scope that belongs to no phase

| PRD section | Proposed home | Why |
|---|---|---|
| §20 Google/Gemini, §21 OpenRouter / OpenAI-compatible / LM Studio | **Phase 3 close-out** | Phase 2 deliverables; OpenRouter and LM Studio are OpenAI-compatible, so they need a configurable base URL, not a new adapter |
| V1 contract #7 — resume the session | **Phase 4** (session memory) | The audit log already holds the history |
| §39 Workspace rules (`.anycode/`, protected files, approval-required) | **Phase 4** (memory) | Repository-scope memory; protected paths feed the permission engine |
| §40 Import CLAUDE.md, AGENTS.md, Codex/Gemini instructions | **Phase 4** (instruction files) / **Phase 7** (MCP configs) | Instruction files are repository memory; MCP configs are capabilities |
| §58 Changes review (accept/reject agent changes) | **Phase 5** | Merge coordinator needs the same surface |
| §91 Privacy modes, Local Only | **Phase 7** | The first phase that adds network capabilities (MCP, connectors, registry) |
| §92 Audit log viewer + enterprise events | **Phase 8** | Next to the usage dashboard; recording starts in the close-out (C8) |
| §93 Auto updates, crash reporting, telemetry controls (§9.1) | **Distribution gate** | Release infrastructure |
| §59 Command bar, §61 Home, §62–63 Onboarding | **Distribution gate** | First-run experience for V1 |
| §69 Accessibility, §70 Performance targets | **Every phase's definition of done**, measured at the Distribution gate | Nothing has been measured yet |

## 6. Quality gaps

- **No frontend tests at all.** The UI has been checked by replaying recorded events through
  a stubbed IPC in a browser, which is not repeatable in CI.
- **`anycode-secrets` has 0 tests.**
- **No performance target in PRD §70 has been measured** (launch time, input latency, memory).
- **No screen-reader walkthrough** of any surface.
- **GUI never driven end to end in this environment:** macOS denies it Screen Recording and
  Accessibility. The agent loop is proven headless; the owner has to confirm the GUI flows.
- **Documentation drift, found and corrected in this audit:** REVIEW.md implied a Windows
  installer existed; ROADMAP.md marked Phase 2's exit met without a live run.

## 7. What's next

In order. Items marked **owner** need something only the owner can provide.

1. **Security close-out (S1–S3).** Small, contained, and each needs a test that fails without
   the fix.
2. **Phase 0 / 1 / 3 confirmations:**
   - Run the Desktop Release workflow once.
   - **Owner:** launch the Windows build.
   - **Owner:** open the terminal once.
   - Complete one passing live Phase 3 run on the final code.
3. **Phase 2 carry-over:**
   - Configurable base URL for OpenAI-compatible providers (OpenRouter, LM Studio).
   - Gemini adapter.
   - Usage recording moved to the adapter boundary (invariant #9).
   - Audit provider-key changes (invariant #10).
   - **Owner:** keys to verify live.
4. **Phase 4 · Code intelligence** — planned in [ROADMAP.md](ROADMAP.md#phase-4--code-intelligence-next).
