# QA and review ledger

This is the shared verification record for Any Code. Every agent that runs tests, performs QA, or
reviews a change appends a dated entry before reporting completion. Results are never rewritten
from fail to pass; a later rerun gets its own row so the history remains inspectable.

## Current quality status

| Area | Last result | Evidence |
|------|-------------|----------|
| Rust workspace | Pass | 90 passed, 0 failed, 2 ignored (live) on 2026-09-23 |
| Desktop runtime crate | Pass | fmt + clippy clean; 1 ignored live test; now gated in CI (new job) |
| Integrated terminal | Fixed, not hand-verified | 4 defects fixed on 2026-09-23; no interactive run observed — see entry |
| Rust lint | Pass | Clippy with warnings denied, both cargo workspaces, 2026-09-23 |
| TypeScript | Pass | `tsc -b --noEmit` on 2026-09-23 |
| Web production build | Pass with warning | `pnpm build` on 2026-09-23; large Monaco chunks remain |
| Rust formatting | Pass | `cargo fmt --check`, both cargo workspaces, 2026-09-23 |
| Native desktop compile | Pass | Release build completed and `Any Code.app` launched on 2026-09-23 |
| macOS bundle | Pass | `.app` and unsigned `.dmg` produced locally on 2026-08-23 |
| Accessibility/static UI | Pass with limitations | Second-pass keyboard-source review completed on 2026-08-24; automated accessibility, screen-reader, and captured native-app walkthrough remain |
| Windows installer | **Never built** | The Desktop Release workflow has 0 runs; no Windows build has ever been produced or launched (audit 2026-09-24) |
| GitHub CI | Pass | Run `35893787230`: all 7 jobs, including the new desktop-runtime job |
| Security review | **3 open findings** | S1 secret paths read without a prompt, S2 over-broad standing grants, S3 no CSP — see docs/AUDIT.md |
| Frontend tests | **None** | No component or end-to-end suite exists |
| Agent runtime (live model) | **Pass, 1 of 6 runs** | Local qwen2.5:3b implemented and verified a task once; the runtime judged all 6 runs correctly — see 2026-09-23 Phase 3 entry |

## Review protocol

Each entry records:

- UTC timestamp and source commit
- Scope and environment
- Exact command or manual procedure
- Pass, fail, blocked, or not-run result
- Relevant counts and artifact paths
- Failures, warnings, and unverified limitations

A failed test remains visible. Fixing it requires a new passing entry with a link or reference to
the earlier failure.

## Verification history

### 2026-09-24 — Full project audit

- **Scope:** everything built so far against PRD §98 (all phases), PRD §9–10 and every PRD
  section, PRODUCT-SCOPE.md's V1 contract, and ARCHITECTURE.md's 12 invariants. Source at
  `1f86131`; CI history via `gh run list`.
- **Method:** every claim checked against code or CI — grep and graphify over the source,
  reading the classifier and permission paths, the Tauri config, and the run history of both
  workflows. No tests were added or changed; this entry records findings, not fixes.
- **Result:** full report in [docs/AUDIT.md](docs/AUDIT.md); the plan that follows is in
  [docs/ROADMAP.md](docs/ROADMAP.md) ("Phase 3 close-out", then Phase 4).
- **Fail (security, open):** S1 — `filesystem.read.workspace` is Low risk for every path, so
  `.env` and key files inside the workspace are read without a prompt and sent to the model.
  S2 — standing grants are keyed on the tool name: "always allow" on one shell command allows
  every Medium and High shell command in that workspace, `git push` included. S3 —
  `"csp": null` in `tauri.conf.json`.
- **Fail (exit conditions):** Phase 0 was never verified on Windows — the release workflow
  has never run. Phase 2 was never demonstrated live; its "exit condition met" rested on unit
  tests, and its Gemini and OpenRouter deliverables were never built.
- **Deviation:** invariant #9 — usage is recorded at two call sites, not at the adapter.
  Invariant #10 is partial — provider-key changes are not audited.
- **Found:** 10 PRD sections with no phase, including V1 contract #7 (resume after restart),
  §39 workspace rules, §91 Local Only mode and §93 auto-update; each now assigned.
- **Corrected here:** this ledger's status table implied a Windows installer existed and
  carried four rows dated 2026-08-24; ROADMAP.md marked Phase 2's exit met without a live run.

### 2026-09-23 — Phase 3 completion: planner, state machine, audit log, live exit condition

- **Scope:** everything PRD §3587 lists for Phase 3 that did not exist — planner, task state
  machine — plus defects found along the way, and a live run of the exit condition.
- **Environment:** macOS 12.7.6, Intel i5-5250U (2 cores), 8 GB RAM; Ollama 0.34.3 run from
  `~/.local/ollama` (not installed system-wide) with `qwen2.5:3b` and `qwen2.5-coder:3b`.
  No OpenAI/Anthropic key exists here.

**Built:** `anycode-agent` crate (state machine, verdict, plan parsing — 20 tests); planner
turn; `events` audit table in the store; `<untrusted>` envelope for tool output
(`anycode_core::Tagged::to_prompt_text`, including a test that hostile content cannot close
its own envelope); Ollama tool-calling; `filesystem.edit.workspace`; bounded replan; login-
shell `PATH` for agent commands; approval dialog shows the content being written.

**Defects found and fixed:**

1. **Every OpenAI agent request would have been rejected.** Tool names are dotted
   (`filesystem.read.workspace`); OpenAI and Anthropic accept only `[a-zA-Z0-9_-]`. The
   existing unit test asserted the dotted name *on the wire*, enshrining the bug. Adapters
   now encode `.` ↔ `__`; the registry test asserts every name round-trips.
2. **The verdict punished normal work.** "Any non-zero exit = failed" counted `grep` finding
   nothing, and a test that failed-then-passed, as failure. The model now marks checks
   (`verify: true`); the runtime judges each check by its latest run.
3. **Whole-file write deleted code in a live run** (run 2: the model wrote only `multiply`,
   removing `add`). Added an exact-match edit tool that refuses rather than truncates.
4. **Approval dialog showed only the path for a write** — a user approved file contents they
   never saw. It now shows the full content or the old/new text.
5. **Audit misattribution:** an approval that timed out was recorded as `denied_by_user`.
   Now `denied_unanswered`. Unknown-tool calls were not audited at all; now
   `task.tool.rejected`.
6. **Malformed calls reached the user for approval** (run 5: edit tool called without
   `old_text`). Required arguments are now checked against the tool's schema first.
7. **A model that never touched a tool ended `completed`** (run 4). The runtime now pushes
   back once when no tool was used.
8. **The final message rendered twice** in the dock (streamed, then repeated in `done`).
9. **The desktop crate — which holds the whole agent loop — was never linted or
   format-checked in CI.** Added a `Desktop runtime` job; rustfmt had to reformat four
   files that had never been checked.

**Live runs of the exit condition** (the production `run_task` under Tauri's mock runtime,
real Ollama model, real git repo; the test approves every prompt as "Allow once" and prints
each; after the verdict it re-runs the *originally committed* tests itself):

| Run | Model | Code | Outcome | Runtime verdict | Correct? |
|-----|-------|------|---------|-----------------|----------|
| 1 | qwen2.5:3b | initial | Described the fix in prose, changed no file | failed | yes |
| 2 | qwen2.5:3b | + guidance | Wrote only `multiply`, deleting `add` | failed | yes |
| 3 | qwen2.5:3b | + edit tool | **Implemented `multiply`; suite exit 0; original tests pass** | **passed** | yes |
| 4 | qwen2.5-coder:3b | + audit fix | Called no tool at all | unverified (completed) | yes — led to fix 7 |
| 5 | qwen2.5:3b | + nudge | Put file content into `shell.execute`, then malformed edit | failed | yes |
| 6 | qwen2.5-coder:3b | final | Called no tool despite two nudges | unverified (completed) | yes |

- **Pass:** exit condition demonstrated (run 3): plan → read → write → check with
  `verify: true` → `verifying` → `completed`, verdict `passed`, 23 audit events, and the
  independent check against the original tests passed.
- **Pass:** the runtime's verdict matched independent reality in **all six** runs. No run
  that failed was reported as passing.
- **Observed live:** the model sent `shell.execute` with a *fabricated successful result*
  as its arguments (`{"exitCode":0,...}`); having no command, it was classified Critical and
  denied. Run 3's own summary misdescribed the work; the measured evidence was right.
- **Pass:** UI replay of runs 3 and 1 in the real frontend build (IPC stubbed, test-only):
  plan, state line, approval dialog showing file content, verdict and `check` labels; 0
  console errors during the checks (26 later errors were Vite's reload client after the dev
  server was stopped).
- **Pass:** `cargo test --workspace` 90 passed / 0 failed / 2 ignored; clippy with `-D
  warnings` and `cargo fmt --check` clean in both workspaces; `tsc` and `pnpm build` clean.

**Not run / limits:**

- **Reliability is low with a 3B model: 1 of 4 runs of `qwen2.5:3b` passed**, and
  `qwen2.5-coder:3b` never calls tools through Ollama. Run 3 passed on code from before
  fixes 5–7; a confirmation run on the final code was started and **stopped by the owner
  before it finished**, so the final code has not itself completed a passing live run. To
  rerun: `~/.local/ollama/ollama serve`, then
  `ANYCODE_LIVE_MODEL=qwen2.5:3b cargo test --lib agent_live -- --ignored --nocapture` in
  `apps/desktop/src-tauri`.
- Not driven through the GUI: macOS denies this environment Screen Recording and
  Accessibility. The loop ran headless under the mock runtime; only the webview was absent.
- OpenAI still not run live (no key); the wire-name fix means it can now work, unverified.
- Ollama ran this model with a 4096-token context; a long task can overflow it.
- Models see encoded tool names (`filesystem__edit__workspace`) while prompts use dotted
  ones; a small model can be confused by the mismatch.
- The audit log is written but nothing reads it back into the dock after a restart.

### 2026-09-23 — Scope decision: extensions and the store (documents only)

- **Scope:** ADR 0003, PRD §10 and §48, ROADMAP phases 4 and 7. No executable code changed, so
  no build, lint or test result is claimed for this change.
- **Decision:** rung B — Any Code applies a `.vsix`'s themes, TextMate grammars and bundled
  language servers, and does not implement the `vscode` API or run extension JavaScript.
  Packages come from Open VSX; Microsoft's Marketplace terms restrict it to Microsoft's own
  products, and PROJECT-RULES.md ranks provider terms above the PRD.
- **Placement:** rung B in Phase 4, the Extensions category in Phase 7. Nothing enters Phase 3.
- **Unverified:** the Open VSX coverage assumption. The claim that themes, grammars and LSP
  servers cover most real extension use is reasoning, not measurement — nobody has surveyed
  what this user's own extension list would actually need.

### 2026-09-23 — Integrated terminal defect hunt

- **Scope:** the report "terminal not working". `anycode-terminal`, `terminal_commands.rs`,
  `TerminalPanel.tsx`, and the bottom panel's mounting in `App.tsx`.
- **Source:** `ae80398` plus the uncommitted fixes under review.
- **Environment:** macOS 12.6, Node 20.19.5, local Rust toolchain.

**Reproduction evidence.** The PTY layer itself was exercised directly before any code
changed, by spawning `/bin/zsh` and `/bin/sh` under a real pty and dumping the raw byte
stream. The shell starts and reaches a prompt; the plumbing was never broken. The
environment of the running app (`ps eww 43529`) showed **no `TERM` variable at all**,
which is the condition under which a shell comes up with no line editor and no terminal
capabilities.

**Defects found and fixed (4):**

1. **Subscribe race — the likely cause of the blank pane.** `terminal_spawn` minted the
   session id and started the reader thread *before* returning it, so the frontend could
   only call `listen()` afterwards. Tauri does not buffer events, so the shell's greeting
   and first prompt were emitted to nobody. Fixed the same way as the agent loop: the
   caller supplies the id and subscribes before spawning.
2. **No `TERM`.** A GUI process inherits none, leaving the shell with no capabilities and
   `clear`/`less`/`vim` failing. Now set explicitly to `xterm-256color`, which is what
   xterm.js actually emulates.
3. **Login shell.** An app opened from Finder inherits only
   `/usr/bin:/bin:/usr/sbin:/sbin`, so nothing the user installed is on `PATH`. The PTY
   now spawns a login shell (non-Windows), matching what the user's own terminal does.
4. **Panel unmount killed the shell.** `TerminalPanel` was conditionally rendered, so
   switching to Chat or closing the bottom panel unmounted it and its cleanup killed the
   PTY — losing scrollback and any running process. Both panels are now kept mounted and
   toggled with `hidden`. The terminal is mounted on first open rather than at startup, so
   the app still does not spawn a shell before a workspace exists (which would fail).

Also fixed in passing: `.terminal-body` subtracted a 36px header that is not inside the
component (it lives in `App.tsx`), clipping the last row; dead sessions were never removed
from `AppState.terminals`; and `fit()` could run against a 0×0 container once the panel
became hideable.

- **Pass:** new `anycode-terminal` test `spawned_shell_reports_an_xterm_term` — spawns a
  real PTY and asserts the shell itself prints `TERM<xterm-256color>`. It fails against the
  pre-fix code in this environment (the parent shell here has `TERM` empty).
- **Pass:** `cargo test --workspace` — 54 passed, 0 failed.
- **Pass:** `cargo clippy --all-targets --all-features -- -D warnings` on both workspaces.
- **Pass:** `cargo fmt --all --check`.
- **Pass:** `tsc -b --noEmit`.

**Not run / unverified:**

- **The fixed terminal has not been driven by hand in the running app.** macOS denies this
  environment both permissions needed to do so: `screencapture -l <window>` returns "could
  not create image from window" and a full-screen capture renders the desktop with every
  application window omitted (Screen Recording), and System Events reports "osascript is
  not allowed to send keystrokes. (1002)" (Accessibility). The rebuilt `.app` launches and
  stays running, and spawns no shell before the panel is opened — which is the new lazy-mount
  behaviour — but nobody has seen a prompt appear. Everything above is compile-, unit- and
  byte-stream-level evidence, not a user-level observation. **The owner should open the
  Terminal panel once and confirm a prompt appears.**
- **`shell.execute` has the same `PATH` gap as defect 3 and is not fixed.** It runs
  `sh -c`, not a login shell, so an agent's `npm test` will fail with "command not found"
  when the app is launched from Finder. A login shell per command would cost seconds of
  profile sourcing each time; the correct fix is to resolve the user's shell environment
  once at startup. Not built.
- Windows and Linux behaviour of the login-shell change (`-l` is skipped on Windows, but
  nobody has run it there).

### 2026-08-23T00:41:22Z — Brand integration baseline

- **Scope:** Any Code logo, Heptagram AI attribution, React shell, Tauri metadata, and repository
  documentation.
- **Source:** `e3801e7` plus the then-uncommitted branding/documentation change under review.
- **Environment:** macOS, Node 20.19.5, pnpm 9.15.9, local Rust toolchain.
- **Pass:** `pnpm lint` — TypeScript project references completed without errors.
- **Pass:** `pnpm build` — 31 modules transformed; production assets emitted to
  `apps/desktop/dist/`.
- **Pass:** `cargo test --workspace` — 5 passed, 0 failed.
- **Pass:** `cargo fmt --all --check` — no formatting changes required.
- **Pass:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` — completed
  without warnings.
- **Pass:** `pnpm --filter @anycode/desktop exec tauri build --no-bundle` — optimized native
  executable produced at `apps/desktop/src-tauri/target/release/anycode-desktop`.
- **Pass:** native launch smoke test — the optimized executable started and exposed an on-screen
  `anycode-desktop` window.
- **Pass:** parallel `pnpm tauri build` packaging run — produced `Any Code.app` and
  `Any Code_0.1.0_x64.dmg`; process exited 0. The artifacts are unsigned and remain local build
  output.
- **Blocked:** real UI screenshot capture — macOS denied screen-capture/assistive access to the
  automation process. No mock screenshot was substituted.
- **Not run:** Windows installer execution, macOS notarization, screen-reader review, and full
  keyboard/accessibility walkthrough.

### 2026-08-23T00:45:09Z — GitHub CI after branding and governance push

- **Source:** `ab100dc2a8f49cd1755f0549d736ba5e6349209e` on `main`.
- **Pass:** [GitHub Actions run 32608603364](https://github.com/sabih-haider1/Any-Code/actions/runs/32608603364).
- **Pass:** committed-secret rejection.
- **Pass:** Rust formatting and Clippy with warnings denied.
- **Pass:** Rust workspace tests on Ubuntu, macOS, and Windows.
- **Pass:** frozen pnpm install and desktop frontend production build.
- **Warning:** GitHub reported that Node.js 20 action runtimes are deprecated and forced
  `actions/setup-node@v4` and `pnpm/action-setup@v4` onto Node 24. The run remained successful;
  action-version migration should be handled in a dedicated CI maintenance change.

### 2026-08-24 — Phase 1 workbench review and UX remediation

- **Scope:** Claude's Phase 1 explorer, editor, Git diff, terminal, command palette, settings,
  persistence, visual language, keyboard access, failure states, and workspace transitions.
- **Source:** `a831c23` plus the then-uncommitted workbench and model-provider changes under
  review. Unrelated concurrent changes were preserved.
- **Review — fail:** dirty editor tabs could be closed without confirmation; file-open,
  last-workspace, theme-save, terminal-write, and terminal-resize errors were silently swallowed;
  explorer and source-control rows were not keyboard controls; nested tab interactions used
  invalid button semantics; command palette and settings lacked complete dialog semantics; diff
  models were not disposed; opening another workspace retained stale diff state.
- **Review — fail:** the visual implementation was dominated by one-off inline styles, ambiguous
  text symbols, a purple AI-dashboard accent, no clear information hierarchy, and an app-wide
  fatal screen for recoverable runtime errors. The workbench did not consistently carry the Any
  Code identity or Heptagram AI attribution.
- **Remediated:** introduced a neutral, professional workbench hierarchy; restrained blue is used
  only for focus/selection/action; added a branded title bar and welcome flow; added explicit
  Explorer, Source Control, editor, terminal, status, settings, and command-palette regions; and
  retained the required “Powered by heptagram-ai.com” attribution on the welcome surface.
- **Remediated:** converted interactive rows to buttons, added landmarks and accessible names,
  dialog/listbox/tab semantics, visible focus, non-color status labels, Escape handling, empty and
  failure states, unsaved-change confirmation, save errors, safe workspace reset, and Monaco diff
  model cleanup.
- **Pass:** `pnpm lint` — TypeScript project references completed without errors.
- **Interrupted:** first `pnpm build` verification was manually stopped after Vite remained in its
  transform phase for more than two minutes. This result is retained rather than rewritten.
- **Pass:** repeated `pnpm build` — 1,313 modules transformed and production assets emitted in
  1m 54s.
- **Warning:** Vite reports oversized Monaco output, including a 7.03 MB TypeScript worker and a
  3.82 MB editor chunk. Lazy language loading/code splitting should be a focused performance task.
- **Pass:** `cargo fmt --all -- --check`.
- **Pass:** `cargo clippy --workspace --all-targets -- -D warnings`.
- **Pass:** `cargo test --workspace` — 26 passed, 0 failed, including core trust/event, filesystem
  boundary, Git, provider-stream parsing, persistence, and documentation targets.
- **Tooling gap:** `pnpm exec prettier --write ...` could not run because Prettier is not installed;
  no formatting claim is made for TypeScript/CSS beyond the successful compiler check.
- **Not run:** native visual capture, screen-reader walkthrough, signed package installation, and
  cross-platform installer execution. These require the relevant interactive/OS environments.

### 2026-08-24 — Second-pass UI/UX and ledger audit

- **Scope:** repeat source audit of runtime safety, focus management, keyboard operation,
  cross-platform labels, terminal lifecycle, product attribution, and ledger accuracy.
- **Review — fail:** `Explorer.tsx` contained a `useState` call at module scope. TypeScript accepted
  it, but React would throw an invalid-hook-call error while loading the module. The first-pass
  green TypeScript result therefore did not prove that the UI was runtime-safe.
- **Review — incomplete:** dialog roles had been added without focus containment or restoration.
  Terminal write/resize errors remained silent, xterm subscriptions were not explicitly disposed,
  and shortcut help showed only the macOS notation.
- **Review wording corrected:** the accessibility status now says “keyboard-source review,” not a
  completed interactive keyboard walkthrough. No automated accessibility or screen-reader result
  is claimed.
- **Remediated:** moved Explorer state into `FileRow`; added modal focus containment, Escape
  handling, and focus restoration; completed command-palette combobox semantics; surfaced terminal
  write/resize failures; disposed terminal subscriptions; made shortcut copy cross-platform; and
  added persistent Heptagram AI attribution to the workbench title bar.
- **Pass:** `pnpm lint` after remediation — TypeScript project references completed without errors.
- **Interrupted:** the second-pass `pnpm build` remained in Vite transformation for more than four
  minutes while another agent was concurrently running the same desktop build. It was stopped to
  avoid compounding resource contention. The earlier completed production build remains the latest
  build result; this run is not represented as a new pass or product failure.
- **Limitation:** the repository has no ESLint React Hooks rule, component test suite, automated
  accessibility runner, or native UI harness. The module-scope-hook defect demonstrates why the
  TypeScript-only `lint` script is insufficient and should be strengthened.

### 2026-09-23 — Phase 3 MVP: agent dock, approvals, cancellation, evidence

- **Scope:** completing the first usable MVP — the agent runtime's user-facing surface
  (task timeline, approval dialog, cancellation, completion evidence), plus the defects
  found while wiring it.
- **Source:** `afd3b91` plus the working tree under review.
- **Environment:** macOS, Node 20.19.5, pnpm 9.15.9, local Rust toolchain, no provider
  API key present.

**Defects found and fixed during this work:**

- **Fail → fixed:** `task:tool_call` and `task:approval_requested` were emitted on
  channels keyed by the *tool-call* id instead of the task id. No frontend could have
  subscribed to them — every approval would have hung for the full 5-minute timeout and
  then auto-denied. Both now use the task id.
- **Fail → fixed:** the frontend subscribed to a task's channels *after* `run_task` had
  already spawned it, so a fast-failing task could emit its terminal event before anyone
  was listening, leaving the UI stuck on "Working…". The task id is now supplied by the
  caller, which subscribes first and starts second.
- **Fail → fixed:** a shell result with no exit code (killed by a signal) rendered a ✓ in
  the timeline. It is now reported as a failure, matching what the evidence block already
  counted it as.
- **Fail → fixed:** saving an editor tab overwrote the file unconditionally, so an agent
  edit to an open file was silently discarded (PRD §57). Save now compares against disk
  and asks before overwriting.
- **Fail → fixed:** the approval dialog rendered top-left because the shared `.overlay`
  class carries no positioning by design; it was missing its own modifier.
- **Fail → fixed:** `detach()` aborted mid-loop if any listener failed to unregister,
  stranding the rest.

**Verification performed:**

- **Pass:** `cargo test --workspace` — 53 passed, 0 failed.
- **Pass:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` — no
  output, both the workspace and the standalone `src-tauri` crate.
- **Pass:** `cargo fmt --all --check`.
- **Pass:** `pnpm exec tsc -b --noEmit`.
- **Pass:** `pnpm tauri build` — `Any Code.app` and an unsigned `Any Code_0.1.0_x64.dmg`.
- **Pass:** native launch — the bundled app started and stayed running; no crash reports
  in `~/Library/Logs/DiagnosticReports`.
- **Pass:** UI review against the dev server with a stubbed IPC bridge (test scaffolding
  only; nothing stubbed ships). Confirmed visually: the agent dock renders with live
  provider/model pickers; the timeline shows streamed text, tool calls with non-colour
  risk labels, and pass/fail results; the approval dialog shows the exact subject,
  capability, risk and workspace with Deny / Always-allow-in-workspace / Allow-once and
  **no** global-allow; the evidence block reports "1 command ran, all exited 0" for a
  clean run and "Verification failed — 1 of 1 command exited non-zero" for a failing one.
- **Pass:** all 7 task event channels were registered *before* `run_task` was invoked,
  confirming the subscribe-then-start fix.
- **Pass:** browser console clean — 0 errors, 0 warnings — once the harness supplied the
  event-plugin global the real runtime injects. The 7 errors seen before that were
  entirely harness artifacts.

**Not run / unverified:**

- **The live agent loop.** No OpenAI/Anthropic key exists in this environment, so no real
  model has driven a tool call through the permission gate end to end. The riskiest
  untested seam is real streaming tool-call reassembly. To close it, a single command
  now exists:
  `OPENAI_API_KEY=sk-... cargo test -p anycode-models --test live_openai -- --ignored`.
  Until that runs, Phase 3's exit condition is **not** demonstrated.
- Screen-reader walkthrough of the new dock and dialog.
- Windows and Linux execution of the new UI (CI builds them; nobody has run them).
- Anthropic and Ollama tool-calling — still honestly unsupported, not merely untested.

## Open QA risks

- Release artifacts are unsigned until protected Apple and Windows signing credentials are
  configured in GitHub.
- The Phase 1 workbench has no component or end-to-end UI suite yet.
- The `lint` script is TypeScript-only and does not enforce React Hooks or accessibility rules.
- Monaco is not yet split by language and makes the first production build slow and the package
  larger than necessary.
- VPS service health is owner-reported and has not been independently checked in this QA run.
- The agent runtime has never been exercised against a live model. Everything below the
  provider boundary is unit-tested; the boundary itself is not.
- Agent task history is in-memory only: closing the app loses the timeline, and a task
  orphaned by a crash cannot be resumed.
- The app inherits its launcher's environment. The PTY terminal now works around this with
  a login shell, but `shell.execute` — the path every agent command takes — does not, so
  agent commands can fail with "command not found" when the app is opened from Finder
  rather than from a shell.
