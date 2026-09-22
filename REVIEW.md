# QA and review ledger

This is the shared verification record for Any Code. Every agent that runs tests, performs QA, or
reviews a change appends a dated entry before reporting completion. Results are never rewritten
from fail to pass; a later rerun gets its own row so the history remains inspectable.

## Current quality status

| Area | Last result | Evidence |
|------|-------------|----------|
| Rust workspace | Pass | 53 tests passed on 2026-09-23 |
| Rust lint | Pass | Clippy completed with warnings denied on 2026-08-24 |
| TypeScript | Pass | Workspace typecheck passed on 2026-08-24 |
| Web production build | Pass with warning | Vite production build completed on 2026-08-24; large Monaco chunks remain |
| Rust formatting | Pass | `cargo fmt --all -- --check` on 2026-08-24 |
| Native desktop compile | Pass | Release build completed and `Any Code.app` launched on 2026-09-23 |
| macOS bundle | Pass | `.app` and unsigned `.dmg` produced locally on 2026-08-23 |
| Accessibility/static UI | Pass with limitations | Second-pass keyboard-source review completed on 2026-08-24; automated accessibility, screen-reader, and captured native-app walkthrough remain |
| Windows installer | Not run locally | GitHub Actions release job is the canonical Windows environment |
| GitHub CI | Pass | Run `32608603364` passed on Linux, macOS, and Windows |
| Agent runtime (live model) | **Not run** | No provider key available in this environment; see 2026-09-23 entry |

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
