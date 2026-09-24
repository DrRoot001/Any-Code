# QA and review ledger

This is the shared verification record for Any Code. Every agent that runs tests, performs QA, or
reviews a change appends a dated entry before reporting completion. Results are never rewritten
from fail to pass; a later rerun gets its own row so the history remains inspectable.

## Current quality status

| Area | Last result | Evidence |
|------|-------------|----------|
| Rust workspace | Pass | 184 passed, 0 failed, 3 ignored (live/keychain) on 2026-09-25 |
| Desktop runtime crate | Pass | fmt + clippy clean; 1 ignored live test; now gated in CI (new job) |
| Integrated terminal | Pass | Real PTY path tested end to end headless, and the panel checked in the browser, 2026-09-24; native window not seen |
| Rust lint | Pass | Clippy with warnings denied, both cargo workspaces, 2026-09-23 |
| TypeScript | Pass | `tsc -b --noEmit` on 2026-09-23 |
| Web production build | Pass with warning | `pnpm build` on 2026-09-23; large Monaco chunks remain |
| Rust formatting | Pass | `cargo fmt --check`, both cargo workspaces, 2026-09-23 |
| Native desktop compile | Pass | Release build completed and `Any Code.app` launched on 2026-09-23 |
| macOS bundle | Pass | CI universal `.dmg` (x86_64 + arm64) launched on an Intel Mac, 2026-09-24; unsigned |
| Accessibility/static UI | Pass with limitations | Second-pass keyboard-source review completed on 2026-08-24; automated accessibility, screen-reader, and captured native-app walkthrough remain |
| Windows installer | Built, **not launched** | `.exe` + `.msi` in draft release `app-v0.1.0` (run `35937745006`); nobody has run it on Windows |
| GitHub CI | Pass | `8196b39`: all 7 jobs, Windows included, 2026-09-25 (after Windows and macOS failures on `c5b8ff4` and `8b6b1bf`) |
| Credential vault | Pass | Real macOS Keychain round trip, 2026-09-24. **Before that date it never persisted a key** (in-memory mock) |
| Security review | Pass — S1–S3 fixed | Fixed and tested 2026-09-24; CSP checked in a browser, not yet in the native WebView |
| Frontend tests | Pass | 45 Vitest tests, 2026-09-25; no component or end-to-end suite yet |
| Agent runtime (live model) | Pass on final code, low reliability | Passed on `9c96cea`; 2 passes in 13 live runs with a 3B local model; verdict correct in every run |
| Code intelligence (Phase 4) | Pass with limitations | Index, code tools, context builder, memory and adoption tested; code-review blocker and 8 security findings fixed with regression tests, 2026-09-25; Context Inspector not seen in the native window |
| Phase 4 exit test | Partly met | ~1% of repo tokens per package on this repo; live agent run timed out (see 2026-09-25 entry) |
| Provider switch (Phase 2 exit) | Pass | Same chat task through two adapters live, 2026-09-24 |

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

### 2026-09-25 — Phase 4: index in the app, context in the agent loop, reviews

- **Scope:** uncommitted work on `591f376`.
  - The index is built in the background when a workspace opens and kept fresh by a debounced
    watcher (`index_commands.rs`, `index:status`).
  - Three code tools: `code.search`, `code.definition`, `code.references`.
  - A context package goes into both prompts (`task:context`, `task.context` audit).
  - Memories go into the system prompt (`task.memories`, ids only).
  - LSP client (not wired).
  - Context Inspector and the StatusBar index badge.
  - The context-builder tests.
- **Environment:** macOS 12 (Intel), Ollama with `qwen2.5:3b`, shared `target-agents/`.
- **Pass — gates:**
  - `cargo test --workspace`: 184 passed, 0 failed (after the re-verification fixes).
  - Desktop crate `cargo test --lib`: 19 passed, 2 ignored (live).
  - Clippy `-D warnings` and fmt clean in both cargo workspaces.
  - `pnpm test`: 45 passed; `tsc -b --noEmit` clean.
- **Exit-test measurement (`cargo run -p anycode-context --example measure --release -- . "<instruction>"`):**
  - **Repository:** 148 files, ~299,052 tokens (estimated as bytes ÷ 4). Indexing in memory
    took 2.0–2.6 s; building a package took 1–7 ms.
  - **"Add a bytes field to the index status that `index_status` reports to the frontend"**:
    9 items, ~2,934 of 3,000 tokens (0.98%). It included the definition in `index_commands.rs`,
    the `IndexStatus` type in `tauri.ts`, and `indexStatus.ts`. Noise: `openai.rs` and
    `anycode-git`. `StatusBar.tsx` was excluded as over budget.
  - **"Make `parse_plan` accept steps numbered like 1) …"**: 9 items, ~2,987 tokens (1.00%).
    It included `crates/anycode-agent/src/plan.rs` (definition and the parse loop) and the
    caller. The rest was term noise on "steps".
  - **"The `code.search` tool should let the agent restrict results to one directory"**:
    - **First run (fail):** the backticked dotted name was dropped from the intent entirely, so
      no identifier was extracted.
    - **The fix:** `intent.rs` now keeps backticked qualified names as a phrase, and gains
      stopwords `let`, `one`, `well`. Test: `a_backticked_dotted_name_is_kept_as_a_phrase`.
    - **Second run:** 8 items, 0.93%. It includes `crates/anycode-tools/src/code.rs` lines 1–50
      and 101–150, **but not 51–100**, where `CodeSearchTool::execute` lives. That is a
      partial miss.
- **Fail — live agent run** (`ANYCODE_LIVE_MODEL=qwen2.5:3b cargo test --lib agent_live -- --ignored`):
  - **What the test now requires:** the real index is built first (`index:status` building →
    ready: 3 files, 5 symbols, 71 ms), and a global memory exists. It asserts the
    `task.context` and `task.memories` audit events, and that the package contains `calc.py`.
  - **Observed:** the context was emitted before planning. It contained `calc.py`
    (named in the instruction), `README.md` and `test_calc.py`, ~134 of ~134 tokens (the
    fixture is tiny).
  - **Timeline:** planning took 252 s. The agent read `calc.py` at 778 s, and the suite failed
    at 779 s. It wrote `multiply` at 1027 s, and the suite passed.
  - **Result:** the task **did not reach `done` within the 45-minute deadline**. The model was
    still producing its final turn, so the test failed at 2,700 s.
  - **Not rerun.** A 3B model on this CPU is too slow for this loop to be a dependable
    signal; a faster model is needed.
- **Code review (code-reviewer)**, fixed:
  - **Blocker:** re-opening workspace A during its first build started a second concurrent
    build on the same SQLite file. The result was `SQLITE_BUSY` and a stale index winning.
    Fixed with a generation counter in `IndexSlot` and a 30 s `busy_timeout` in `Index::open`.
  - `code.definition` and `select_context` held the index mutex on async threads. They now run
    under `spawn_blocking`.
  - The LSP client allocated whatever `Content-Length` a server sent. It is now capped at 16 MiB
    (test: `an_oversized_content_length_is_refused_without_allocating`), and its doc comment no
    longer claims that writes time out.
  - The audit payload for `task.context` is now a tested function. Test:
    `the_context_audit_records_the_choice_but_never_the_passages`.
  - A poisoned store lock is now logged.
- **Security review (security-reviewer, reproduced with a probe against the real crates)**,
  fixed with regression tests:
  1. **High:** the Low-risk code tools and the automatic context returned `secrets.yaml`,
     `prod.env` and `server.key`. Anything `path_risk` classifies is now neither indexed nor
     searched (`walk::is_sensitive`). Hits in `.anycode/permissions.yaml`-protected paths are
     withheld from code-tool results and the context package, and the model is told how many.
     Tests: `secret_files_are_neither_indexed_nor_searched` and
     `code_tool_hits_in_protected_paths_are_withheld_and_counted`.
  2. **High:** the watcher indexed `.env.local` and nested-gitignored files that the full scan
     skips. `walk::is_excluded` now applies hidden-component, sensitive, ancestor
     `.gitignore`/`.ignore` and `.git/info/exclude` rules. Test:
     `the_watcher_skips_what_the_full_scan_skips`.
  3. **Medium:** a file swapped for a symlink out of the workspace stayed indexed, and its
     target was read into the prompt. `update_paths` now names paths lexically and drops links.
     The context builder refuses to read through a link out. Tests:
     `a_file_swapped_for_a_symlink_leaves_the_index` and
     `a_file_swapped_for_a_symlink_out_of_the_workspace_is_not_read`.
  4. **Medium:** a file name containing a newline put text on its own prompt line outside the
     envelope. The new `trust::prompt_label` escapes paths and origins. Tests:
     `a_path_with_a_newline_cannot_put_text_on_a_line_of_its_own` and
     `an_origin_cannot_break_out_of_its_attribute_or_line`.
  5. **Medium:** adoption followed symlinks and adopted text the user never saw. Now:
     - symlinks are refused and not listed;
     - "Review to adopt" shows the exact text;
     - `adopt_instruction(path, content)` refuses if the file changed since the preview.

     Tests: `a_file_changed_after_its_preview_is_not_adopted` and
     `a_symlinked_instruction_file_is_neither_listed_nor_adopted`.
  6. **Low-medium:** `</UNTRUSTED>` and `< /untrusted>` were not defused. Defusal is now
     case- and space-insensitive. Test:
     `closing_tags_in_any_case_or_spacing_cannot_end_the_envelope`.
  7. **High once wired (LSP):**
     - Relative PATH entries are skipped.
     - Binaries resolving inside the workspace are refused.
     - The canonical path is spawned.

     Test: `a_server_inside_the_workspace_is_never_found`. Starting a server runs repository
     code, so it needs its own capability before it is wired (ROADMAP 4.8).
  8. **Low:** search limits:
     - lines truncated to 400 characters;
     - files over 1 MiB skipped;
     - regex size limit of 10 MiB;
     - patterns of at most 1,000 characters.

     Test: `search_truncates_long_lines_and_skips_oversized_files`.
  - **Not fixed:** the reviewer reported an off-by-one in `uri_to_path`, but `i + 2 < len`
    already guarantees both hex digits exist, so the finding was wrong and nothing changed.
- **Security re-verification (security-reviewer, same probe, updated):**
  - **All eight fixes hold.** Among the evidence:
    - `search_live` finds no secret fixtures.
    - A symlink swap removes the file's row and reads nothing from outside.
    - Every closing-tag variant is defused, including tabs, newlines and upper case.
    - A 200 KB line comes back as 401 characters.
  - **New issues found:**
    - **A (medium):** the watcher ignored `.gitignore` files above the workspace root (a
      workspace opened at `monorepo/apps`), and let `.gitignore` override `.ignore`. The probe
      showed two such files reaching the prompt.
    - **B (low-medium):** the context builder followed a symlink that points inside the
      workspace, such as a link to `.env`. One unreadable file failed a whole `update_paths`
      batch or `refresh`, which rolled back the deletion of that link's row, so the window
      could become permanent.
    - **D (low):** when a directory was swapped for a symlink, the rows under it stayed.
    - **E (low):** `prompt_label` let U+2028/U+2029 and the bidi controls through.
  - **Fixed:**
    - A: `is_excluded` now climbs to the repository's top level and reads its
      `.git/info/exclude`, and `.ignore` now outranks `.gitignore`.
    - B: `read_lines` requires the resolved path to equal the lexical path. An unreadable file
      is now skipped (and its rows dropped) instead of failing the batch.
    - D: new `delete_tree_rows`.
    - E: those characters are now escaped.
  - **Tests added:**
    - `the_watcher_honours_ignore_files_above_the_root_and_ignore_over_gitignore`
    - `a_file_swapped_for_a_symlink_inside_the_workspace_is_not_read_either`
    - `one_unreadable_file_does_not_fail_the_batch`
    - `a_directory_swapped_for_a_symlink_takes_its_rows_along`
    - a U+2028/U+202E assertion in `an_origin_cannot_break_out_of_its_attribute_or_line`
  - **Not fixed (C, low):** the regex engine takes about 2.5 s to refuse `(\w{100}){100}` in
    a debug build. Not measured in release.
  - **Not re-probed after these last fixes:** the regression tests above cover each case
    instead.
- **Fail, then fixed: Windows CI on `c5b8ff4`.** `lsp::tests::path_to_uri_round_trips` got
  `/C:/Users/.../f.rs` back where it expected `C:\Users\...\f.rs`. On Windows, `uri_to_path`
  now drops the slash before the drive letter and uses backslashes, and `path_to_uri` strips
  the verbatim `\\?\` prefix. That branch can only be checked in Windows CI; see the next
  run. Every other job passed, including macOS, Ubuntu and the desktop runtime.
- **Fail, then fixed: macOS CI on `8b6b1bf`.** `rust_analyzer_definition_lookup` failed with
  `initialize: ProcessExited`, and Windows passed.
  - **Root cause, a bug from the PATH hardening:** `find_in` returned the canonicalised binary.
    `~/.cargo/bin/rust-analyzer` is a symlink to `rustup`, which picks what to run from the name
    it was invoked as, so the client spawned bare `rustup`. The same would break any multi-call
    binary.
  - **Fix:** a candidate is still judged by where it resolves, but it is spawned by the PATH
    entry's own absolute path.
  - **The test was also timing-dependent.** It waited 3 s to see whether the proxy exited, and
    the previous run only passed because the proxy exited inside that window. It now asks
    `--version` and skips when that fails. Locally the skip now fires, because the proxy's
    component is not installed.
- **Flaky test, fixed:** `a_file_named_in_the_instruction_is_included_whole_with_the_right_reason`
  failed once in 6 local runs. The context tests' `TempDir` was named by pid and nanoseconds,
  but macOS time has microsecond resolution, so two parallel tests could share a directory and
  delete each other's files. An atomic counter was added to the name. After the fix: 10 of 10
  runs passed.
- **Known limitations:**
  - notify 7's inotify backend follows symlinks when adding recursive watches (Linux). It has
    no option to stop until notify 8; content is still refused by canonicalisation.
  - The watcher path does not read the user's global git `excludesFile`.
  - When the index is not ready, no `task:context` event is sent. The model is told, but the
    timeline shows nothing.
  - A past task's replay (`history.ts`) does not show its context package, because the audit
    log keeps no passage text.
  - The Context Inspector, StatusBar badge and adoption preview were never seen in the native
    window.
  - Memory scopes are global and workspace only.

### 2026-09-24 — Close-out, continued: C3, C5, C6, C7 verified live

- **Scope:** the close-out items still open after the first pass. C4 (Windows launch) is
  parked: no Windows machine is available. Source `9c96cea`.
- **Environment:** Ollama 0.34.3 reinstalled locally, with `qwen2.5:3b`.
- **Pass — C3:** a test proves the context compiled into the app carries the strict CSP. It
  fails when `"csp": null` is put back; I checked by doing exactly that. Tauri's page server
  (`protocol/tauri.rs`) sets the header from that context. WebKit enforcing it inside the
  native window was not observed.
- **Pass — C5:** a headless test drives the real PTY path.
  - The first output reaches a listener registered before the spawn.
  - Typed input runs with `TERM=xterm-256color`.
  - Kill emits exit and frees the session.
  - In the browser, the production frontend showed the first output even when it was
    emitted inside `terminal_spawn`. It kept the same shell across a Chat round-trip:
    spawned once, killed zero times.
- **Pass — C7 exit condition:** `the_same_chat_task_switches_providers_live`.
  - The same chat task went through the Ollama adapter and the OpenAI adapter (via Ollama's
    `/v1`), changing only the provider id.
  - Both replied "ready"; both were metered (36 in / 2 out).
  - This is the first time the OpenAI adapter has run against a real server.
  - Two adapters on one server, not two vendors: Gemini and OpenRouter need keys.
- **Found live, fixed:**
  - **No per-turn output cap:** one degenerate turn generated 3,600+ tokens for half an hour.
    Now `max_output_tokens`, mapped per adapter. The agent caps plans at 1,024 tokens and
    turns at 4,096.
  - **Ollama's 4,096-token context** silently dropped the system prompt (`truncated = 1`).
    The adapter now asks for 8,192.
  - **An in-root absolute path was refused** as an escape. It now resolves; the escape
    cases stay refused.
  - **An unflagged failing `python3 -m unittest` turned "failed" into "unverified".** Known
    test, build and lint runners now always count as checks.
  - **A rejected call produced a result with no visible call.** It is now shown as
    `rejected`.
- **Observed live through the OpenAI adapter:** three parallel tool calls in one streamed
  turn, reassembled correctly.
- **Pass — C6:** `agent_live_test` passed on `9c96cea`, in attempt 3 of 3. The model's
  unflagged test run failed; the new known-check rule made the runtime push back rather
  than accept "unverified". The model then fixed `calc.py`, the tests exited 0, the
  verdict was `passed` and the state `completed`. The original tests passed independently,
  and the audit log holds 28 events.

**Every live agent run today** (`qwen2.5:3b` unless noted):

| Code | Runs | Passed | Failure modes |
|------|------|--------|---------------|
| `2597c00` | 3 | 0 | overwrote `calc.py` and deleted `add`; test file content written into `calc.py`; inspected and stopped |
| `2597c00`, OpenAI adapter | 1 | 0 | absolute path refused (since fixed); malformed edit |
| `9c96cea` | 3 | **1** | inspected and stopped (×2) |

The runtime's verdict matched independent reality in every run, and none was reported as
passing when it wasn't. Reliability belongs to the model: a stronger one is needed before
anyone should expect the agent to succeed routinely.

- **Pass — gates:** 109 Rust tests at the root (3 ignored); the desktop crate 3 passed (2
  ignored, live); 15 frontend tests; clippy and fmt clean in both cargo workspaces.
- **Not run:** C4 (no Windows machine). Gemini and OpenRouter live (no keys).
- **Fail found by CI, fixed:** Windows `Test` failed
  `accepts_an_absolute_path_inside_the_root`. Windows canonicalises the root to a verbatim
  `\\?\C:\…` path: `/` is not a separator there, and a plain `C:\…` spelling never
  matched. In that form `..` also arrives as an ordinary component. Both spellings are now
  recognised, and `..` and `.` are refused by name. CI run on `d0c89cb` passed all 7 jobs.

### 2026-09-24 — Phase 3 close-out (C1–C10)

- **Scope:** the close-out list in docs/ROADMAP.md, which follows from the 2026-09-24 audit.
  Source `2597c00` (code) and `80d6392` (release workflow permission).
- **Pass — security:**
  - **S1:** filesystem tools take the risk of the path they name. `.env*`, credential files
    and `.git/config` are High; private keys are Critical. Shell commands naming those paths,
    or printing the environment, are raised the same way.
  - **S2:** `decide()` no longer lets a standing grant cover High. The old test asserting that
    it did was rewritten. Shell grants are keyed on the exact command; the runtime refuses a
    grant for anything but Medium, whatever the renderer sends. Tool-wide shell grants are
    migrated away.
  - **S3:** a strict CSP replaces `"csp": null`.
  - Each has tests that fail without the fix.
- **Pass — CSP in practice:** the production build, served in a browser under the identical
  policy. No CSP violations; every Monaco worker loaded; a file opened and rendered 7 lines
  with 21 syntax tokens; the `.env.production` prompt showed High, the reason, and no
  "always allow" button. *Not* observed in the native WebView; this environment cannot see
  the app window.
- **Fail found, fixed — the credential vault never stored a key.** `keyring` 3 has no default
  store and, with no platform feature enabled, silently uses an in-memory mock. Every API
  key saved in Settings since Phase 2 was lost the moment it was written, so OpenAI and
  Anthropic were never usable in the app. Found by the first real-keychain test (it returned
  `None` straight after a successful write). Now Keychain, Credential Manager or Secret
  Service; the round trip passes against the real macOS Keychain and leaves no entry.
- **Fail found, fixed — `send_chat` subscribe race.** This is the third instance, after
  `run_task` and `terminal_spawn`: an instant failure was emitted before the panel listened,
  and the chat waited forever. The error listener also leaked.
- **Pass — invariants:**
  - **#9:** usage is recorded by one `Metered` wrapper around every adapter, including
    cancelled requests, which were previously not recorded at all. 4 tests.
  - **#10:** provider connect and disconnect are audited. A test confirms the key never
    appears, whole or in part.
- **Pass — Phase 2 carry-over (code):**
  - Gemini: API-key mode, through Google's OpenAI-compatible endpoint.
  - OpenRouter.
  - A generic OpenAI-compatible endpoint: https anywhere, plain http only to localhost,
    validated in Rust. Tested.
  - Settings no longer says "Connected" for a local server it has never reached.
- **Pass — tests:** 104 Rust (was 90), 3 ignored live; first frontend suite, 15 Vitest tests,
  now in CI. CI run `35934057203` passed all 7 jobs, including Linux against the real Secret
  Service and Windows against Credential Manager.
- **Release workflow — first ever runs, four defects found and fixed:**

  | Run | Result | Cause | Fix |
  |---|---|---|---|
  | `35934070914` | all 3 fail | Built fine, then "Resource not accessible by integration" — token read-only | `permissions: contents: write` |
  | `35935062802` | macOS fails | An unset secret arrives as `""`; Tauri tried to import an empty certificate | Export only credentials that have a value |
  | `35936049920` | pass | — but the Mac build was **arm64-only**: it cannot start on an Intel Mac | `--target universal-apple-darwin` |
  | `35936666105` | macOS fails | `openssl-sys` won't cross-compile to x86_64; it came only from git2's unused network features | `git2` without default features — no OpenSSL or libssh2 in the tree |
  | `35937745006` | **pass** | | |

- **Pass — draft release `app-v0.1.0`:** private and pre-release. Contents:
  - Windows: `.exe` (NSIS) and `.msi`.
  - macOS: universal `.dmg`.
  - Linux: `.deb` and `.AppImage`.
- **Pass — the CI-built macOS app runs.** It was downloaded from the draft and checked with
  `lipo`: `x86_64 arm64`. It launched on this Intel Mac, created a 1280×800 window, ran for
  15 s, wrote no crash report, and quit cleanly.
  - It is unsigned: the download-quarantine flag had to be cleared first. A real user will
    see Gatekeeper's "unidentified developer" warning until the build is signed.

**Not run / open:**

- **C6 and the live half of C7 are blocked:** the local model runtime and its models were
  removed from this machine after the previous session. Both tests are written and ready:
  - `agent_live_test` (either adapter, via `ANYCODE_LIVE_PROVIDER`);
  - `the_same_chat_task_switches_providers_live`.
  Running them needs Ollama plus a ~2 GB model, or API keys.
- **Gemini and OpenRouter have never been called live.** Unverified without keys:
  - Whether Gemini's compatible endpoint accepts `stream_options`.
  - The format of the model IDs it returns.
- **C4:** the Windows installer is built; **the owner has to launch it** on a Windows
  machine — nobody has run the app on Windows.
- **C5:** the terminal needs the owner.

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
