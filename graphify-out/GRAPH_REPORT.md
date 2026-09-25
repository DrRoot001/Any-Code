# Graph Report - Any Code  (2026-09-25)

## Corpus Check
- 179 files · ~284,424 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2046 nodes · 4005 edges · 146 communities (117 shown, 29 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 41 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4c86da0a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- PRD.md
- anycode-code-intelligence/src/lib.rs
- What You Must Do When Invoked
- What You Must Do When Invoked
- What You Must Do When Invoked
- AppState
- anthropic.rs
- 98. Implementation Phases
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- anycode-context/src/lib.rs
- Project rules and governance
- csp
- 8. Product Principles
- Adding a model provider
- graphify reference: query, path, explain
- Any Code architectural invariants
- graphify reference: query, path, explain
- graphify reference: query, path, explain
- 6. Target Users
- Definition of done
- 1. Record architecture decisions
- 2. Append-only event log with untyped payloads
- graphify reference: add a URL and watch a folder
- graphify reference: commit hook and native AGENTS.md integration
- graphify reference: incremental update and cluster-only
- graphify reference: add a URL and watch a folder
- graphify reference: commit hook and native CLAUDE.md integration
- graphify reference: incremental update and cluster-only
- graphify reference: add a URL and watch a folder
- graphify reference: commit hook and native CLAUDE.md integration
- graphify reference: incremental update and cluster-only
- 3. Positioning
- 95. Testing Strategy
- 9. Product Scope
- graphify reference: GitHub clone and cross-repo merge
- graphify reference: transcribe video and audio
- graphify reference: GitHub clone and cross-repo merge
- graphify reference: transcribe video and audio
- graphify reference: GitHub clone and cross-repo merge
- graphify reference: transcribe video and audio
- 26. Model Router
- 64. Theme System
- dependencies
- .agents/skills/graphify/references/extraction-spec.md
- .claude/CLAUDE.md
- .claude/skills/graphify/references/extraction-spec.md
- .codex/skills/graphify/references/extraction-spec.md
- 12. Technology Stack
- 4. Brand Identity
- 5. Founder Signature and Product Mark
- Any Code
- verdict.rs
- compilerOptions
- Product scope
- package.json
- design-tokens/package.json
- Architecture
- App.tsx
- anycode-git/src/lib.rs
- anycode-desktop
- Releasing the desktop app
- Security model
- Verification history
- AgentPanel.tsx
- Agent Routing Policy
- Staging operations and monitoring
- path_util.rs
- Repository knowledge graph
- Roadmap
- code-intelligence-engineer.md
- WorkspaceRoot
- tests.rs
- PtySession
- default.json
- anycode-store/src/lib.rs
- Project audit — 2026-09-24
- useWorkbenchStore
- .prettierrc.json
- agent_commands.rs
- README.md
- devDependencies
- anycode-security/src/lib.rs
- metered.rs
- tauri.ts
- .push
- types.rs
- Any Code brand
- anycode-secrets/src/lib.rs
- ollama.rs
- lsp.rs
- openai.rs
- index_commands.rs
- live_openai.rs
- filesystem.rs
- Language
- Value
- trust.rs
- memory_commands.rs
- code-reviewer.md
- Tool
- 3. VS Code extension compatibility and the package registry
- ApprovalDialog.tsx
- desktop/package.json
- scripts
- ToolContext
- @xterm/xterm
- MemorySection.tsx
- workspace_policy.rs
- StatusBar.tsx
- ProviderError
- ProviderManifest
- require_root
- react
- search_live
- code.rs
- terminal_commands.rs
- Symbol
- anycode-tools/src/lib.rs
- provider_commands.rs
- GitStatusTool
- git_status
- compatibility-engineer.md
- 4. Code intelligence, context and memory
- debugger.md
- performance-engineer.md
- schema-designer.md
- security-reviewer.md
- test-writer.md
- ui-engineer.md
- ux-reviewer.md
- TaskHistoryPanel.tsx
- current_path
- TimelineEntry.tsx
- fts.rs
- String
- Engineering standards
- AGENTS.md

## God Nodes (most connected - your core abstractions)
1. `AppState` - 56 edges
2. `build()` - 28 edges
3. `Index` - 26 edges
4. `Store` - 26 edges
5. `IndexError` - 25 edges
6. `StoreError` - 24 edges
7. `useWorkbenchStore` - 22 edges
8. `LspClient` - 21 edges
9. `ToolContext` - 21 edges
10. `WorkspaceRoot` - 18 edges

## Surprising Connections (you probably didn't know these)
- `anycode-desktop` --depends_on--> `anycode-agent`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-agent/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-fs`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-fs/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-git`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-git/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-models`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-models/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-secrets`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-secrets/Cargo.toml

## Import Cycles
- 2-file cycle: `crates/anycode-code-intelligence/src/lib.rs -> crates/anycode-code-intelligence/src/lsp.rs -> crates/anycode-code-intelligence/src/lib.rs`

## Communities (146 total, 29 thin omitted)

### Community 0 - "PRD.md"
Cohesion: 0.02
Nodes (94): 100. Quality Bar, 101. Definition of Done, 102. Repository Implementation Contract, 103. Architectural Rules, 104. Critical Product Differentiator, 105. Product North Star, 10. Non Goals for V1, 11. Platform Architecture (+86 more)

### Community 1 - "anycode-code-intelligence/src/lib.rs"
Cohesion: 0.17
Nodes (25): Connection, canonical_root(), Chunk, chunk_lines(), delete_file_rows(), delete_tree_rows(), fetch_file_meta(), FileInfo (+17 more)

### Community 2 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native AGENTS.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 3 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 4 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 5 - "AppState"
Cohesion: 0.13
Nodes (19): app_context(), AppState, get_theme(), Arc, AtomicBool, Context, Mutex, Option (+11 more)

### Community 6 - "anthropic.rs"
Cohesion: 0.14
Nodes (15): AnthropicProvider, build_messages_request(), parse_event(), parses_content_block_delta(), parses_output_usage_from_message_delta(), Client, ModelDefinition, ModelStream (+7 more)

### Community 7 - "98. Implementation Phases"
Cohesion: 0.17
Nodes (12): 98. Implementation Phases, Phase 0: Foundation, Phase 10: Mobile, Phase 1: Workbench, Phase 2: AI Provider Layer, Phase 3: Agent Runtime, Phase 4: Code Intelligence, Phase 5: Multi Agent (+4 more)

### Community 8 - "graphify reference: extra exports and benchmark"
Cohesion: 0.22
Nodes (8): graphify reference: extra exports and benchmark, Step 6b - Wiki (only if --wiki flag), Step 7 - Neo4j export (only if --neo4j or --neo4j-push flag), Step 7a - FalkorDB export (only if --falkordb or --falkordb-push flag), Step 7b - SVG export (only if --svg flag), Step 7c - GraphML export (only if --graphml flag), Step 7d - MCP server (only if --mcp flag), Step 8 - Token reduction benchmark (only if total_words > 5000)

### Community 9 - "graphify reference: extra exports and benchmark"
Cohesion: 0.22
Nodes (8): graphify reference: extra exports and benchmark, Step 6b - Wiki (only if --wiki flag), Step 7 - Neo4j export (only if --neo4j or --neo4j-push flag), Step 7a - FalkorDB export (only if --falkordb or --falkordb-push flag), Step 7b - SVG export (only if --svg flag), Step 7c - GraphML export (only if --graphml flag), Step 7d - MCP server (only if --mcp flag), Step 8 - Token reduction benchmark (only if total_words > 5000)

### Community 10 - "graphify reference: extra exports and benchmark"
Cohesion: 0.22
Nodes (8): graphify reference: extra exports and benchmark, Step 6b - Wiki (only if --wiki flag), Step 7 - Neo4j export (only if --neo4j or --neo4j-push flag), Step 7a - FalkorDB export (only if --falkordb or --falkordb-push flag), Step 7b - SVG export (only if --svg flag), Step 7c - GraphML export (only if --graphml flag), Step 7d - MCP server (only if --mcp flag), Step 8 - Token reduction benchmark (only if total_words > 5000)

### Community 11 - "anycode-context/src/lib.rs"
Cohesion: 0.10
Nodes (59): Candidates, a_backticked_dotted_name_is_kept_as_a_phrase(), a_path_that_is_not_in_the_repository_is_not_invented(), extract_intent(), finds_named_files_backticked_names_and_words(), Intent, is_code_shaped(), known() (+51 more)

### Community 12 - "Project rules and governance"
Cohesion: 0.25
Nodes (8): Architecture rules, Authority and source order, Decision rules, Delivery rules, Product rules, Project rules and governance, Quality and release rules, Security and privacy rules

### Community 13 - "csp"
Cohesion: 0.05
Nodes (42): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+34 more)

### Community 14 - "8. Product Principles"
Cohesion: 0.25
Nodes (8): 8.1 Provider independence, 8.2 Local first, 8.3 Cloud optional, 8.4 Explainable routing, 8.5 Human authority, 8.6 Evidence before completion, 8.7 No artificial provider lock in, 8. Product Principles

### Community 15 - "Adding a model provider"
Cohesion: 0.29
Nodes (6): Adding a model provider, Capabilities are declared, not assumed, Contract, Local providers, Required before merge, Rules

### Community 16 - "graphify reference: query, path, explain"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

### Community 17 - "Any Code architectural invariants"
Cohesion: 0.33
Nodes (5): Any Code architectural invariants, Reporting, The check, Trust tagging, Where code belongs

### Community 18 - "graphify reference: query, path, explain"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

### Community 19 - "graphify reference: query, path, explain"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

### Community 20 - "6. Target Users"
Cohesion: 0.33
Nodes (6): 6.1 Individual Developer, 6.2 AI Power User, 6.3 Professional Software Engineer, 6.4 Development Team, 6.5 Enterprise, 6. Target Users

### Community 21 - "Definition of done"
Cohesion: 0.40
Nodes (4): Definition of done, Evidence, Gate, Quality bar — automatic rejection

### Community 22 - "1. Record architecture decisions"
Cohesion: 0.40
Nodes (4): 1. Record architecture decisions, Consequences, Context, Decision

### Community 23 - "2. Append-only event log with untyped payloads"
Cohesion: 0.40
Nodes (4): 2. Append-only event log with untyped payloads, Consequences, Context, Decision

### Community 24 - "graphify reference: add a URL and watch a folder"
Cohesion: 0.50
Nodes (3): For /graphify add, For --watch, graphify reference: add a URL and watch a folder

### Community 25 - "graphify reference: commit hook and native AGENTS.md integration"
Cohesion: 0.50
Nodes (3): For git commit hook, For native AGENTS.md integration, graphify reference: commit hook and native AGENTS.md integration

### Community 26 - "graphify reference: incremental update and cluster-only"
Cohesion: 0.50
Nodes (3): For --cluster-only, For --update (incremental re-extraction), graphify reference: incremental update and cluster-only

### Community 27 - "graphify reference: add a URL and watch a folder"
Cohesion: 0.50
Nodes (3): For /graphify add, For --watch, graphify reference: add a URL and watch a folder

### Community 28 - "graphify reference: commit hook and native CLAUDE.md integration"
Cohesion: 0.50
Nodes (3): For git commit hook, For native CLAUDE.md integration, graphify reference: commit hook and native CLAUDE.md integration

### Community 29 - "graphify reference: incremental update and cluster-only"
Cohesion: 0.50
Nodes (3): For --cluster-only, For --update (incremental re-extraction), graphify reference: incremental update and cluster-only

### Community 30 - "graphify reference: add a URL and watch a folder"
Cohesion: 0.50
Nodes (3): For /graphify add, For --watch, graphify reference: add a URL and watch a folder

### Community 31 - "graphify reference: commit hook and native CLAUDE.md integration"
Cohesion: 0.50
Nodes (3): For git commit hook, For native CLAUDE.md integration, graphify reference: commit hook and native CLAUDE.md integration

### Community 32 - "graphify reference: incremental update and cluster-only"
Cohesion: 0.50
Nodes (3): For --cluster-only, For --update (incremental re-extraction), graphify reference: incremental update and cluster-only

### Community 33 - "3. Positioning"
Cohesion: 0.50
Nodes (4): 3.1 Product category, 3.2 Core promise, 3.3 Secondary message, 3. Positioning

### Community 34 - "95. Testing Strategy"
Cohesion: 0.50
Nodes (4): 95. Testing Strategy, End to end tests, Integration tests, Unit tests

### Community 35 - "9. Product Scope"
Cohesion: 0.50
Nodes (4): 9.1 V1, 9.2 V1.5, 9.3 V2, 9. Product Scope

### Community 42 - "26. Model Router"
Cohesion: 0.67
Nodes (3): 26.1 Routing inputs, 26.2 Example, 26. Model Router

### Community 43 - "64. Theme System"
Cohesion: 0.67
Nodes (3): 64.1 Dark palette, 64.2 Light palette, 64. Theme System

### Community 44 - "dependencies"
Cohesion: 0.12
Nodes (17): @anycode/design-tokens, dependencies, @anycode/design-tokens, monaco-editor, react-dom, @tanstack/react-query, @tauri-apps/api, @tauri-apps/plugin-dialog (+9 more)

### Community 53 - "verdict.rs"
Cohesion: 0.07
Nodes (27): parse_plan(), String, Vec, InvalidTransition, Default, Result, Self, run() (+19 more)

### Community 54 - "compilerOptions"
Cohesion: 0.10
Nodes (20): compilerOptions, isolatedModules, jsx, lib, module, moduleResolution, noEmit, noFallthroughCasesInSwitch (+12 more)

### Community 55 - "Product scope"
Cohesion: 0.29
Nodes (7): Change control, Product scope, Release boundaries, Scope hierarchy, Scope test for every task, V1 non-goals, V1 product contract

### Community 56 - "package.json"
Cohesion: 0.11
Nodes (18): description, devDependencies, prettier, engines, node, pnpm, homepage, name (+10 more)

### Community 57 - "design-tokens/package.json"
Cohesion: 0.22
Nodes (8): description, exports, ./tokens.css, main, name, private, type, version

### Community 58 - "Architecture"
Cohesion: 0.25
Nodes (8): Architecture, Boundaries between crates, Decisions, Event log, Invariants, Shape, Trust boundary, Verification

### Community 59 - "App.tsx"
Cohesion: 0.12
Nodes (20): App(), THEMES, Command, CommandPalette(), Icon(), IconName, SettingsPanel(), THEMES (+12 more)

### Community 61 - "anycode-git/src/lib.rs"
Cohesion: 0.17
Nodes (21): current_branch(), diff_file(), diff_reports_head_and_working_content(), empty_repository_has_no_branch(), FileDiff, FileStatus, GitError, reports_untracked_and_modified_files() (+13 more)

### Community 64 - "anycode-desktop"
Cohesion: 0.26
Nodes (13): anycode-agent, anycode-code-intelligence, anycode-context, anycode-core, anycode-desktop, anycode-fs, anycode-git, anycode-models (+5 more)

### Community 65 - "Releasing the desktop app"
Cohesion: 0.33
Nodes (6): Auto-update signature (separate from code signing), macOS — code signing + notarization, Releasing the desktop app, Trigger it, What "just build it" gets you today, Windows — code signing

### Community 66 - "Security model"
Cohesion: 0.25
Nodes (7): Permissions, Reporting, Secrets, Security model, Shell risk classes, The one rule, Threat model

### Community 67 - "Verification history"
Cohesion: 0.11
Nodes (18): 2026-08-23T00:41:22Z — Brand integration baseline, 2026-08-23T00:45:09Z — GitHub CI after branding and governance push, 2026-08-24 — Phase 1 workbench review and UX remediation, 2026-08-24 — Second-pass UI/UX and ledger audit, 2026-09-23 — Integrated terminal defect hunt, 2026-09-23 — Phase 3 completion: planner, state machine, audit log, live exit condition, 2026-09-23 — Phase 3 MVP: agent dock, approvals, cancellation, evidence, 2026-09-23 — Scope decision: extensions and the store (documents only) (+10 more)

### Community 68 - "AgentPanel.tsx"
Cohesion: 0.23
Nodes (15): AgentPanel(), STATE_LABEL, agentCommands, ApprovalResponse, TaskDone, TaskPlan, TaskState, TaskToolCall (+7 more)

### Community 69 - "Agent Routing Policy"
Cohesion: 0.33
Nodes (6): Agent Routing Policy, Before writing code, Commands, Definition of done, graphify, Non-negotiable

### Community 70 - "Staging operations and monitoring"
Cohesion: 0.33
Nodes (6): Expected staging services, Incident response, Minimum online checks, Monitoring rules, Operating boundary, Staging operations and monitoring

### Community 71 - "path_util.rs"
Cohesion: 0.12
Nodes (24): best_effort_canonical(), has_git_component(), lexical_relative(), lexically_normalize(), relative_to(), Option, Path, PathBuf (+16 more)

### Community 72 - "Repository knowledge graph"
Cohesion: 0.40
Nodes (5): Installation for a new workstation, Installed integration, Repository knowledge graph, Required agent workflow, What the graph may contain

### Community 73 - "Roadmap"
Cohesion: 0.20
Nodes (10): Assigned by the 2026-09-24 audit, Current phase: 4 · Code intelligence, Distribution gate, Phase 0 · Foundation, Phase 1 · Workbench, Phase 2 · Provider layer, Phase 3 · Agent runtime, Phase 3 close-out — done except C4 (needs a Windows machine) (+2 more)

### Community 76 - "WorkspaceRoot"
Cohesion: 0.15
Nodes (24): accepts_an_absolute_path_inside_the_root(), Entry, FsError, list_dir(), read_file(), rejects_an_absolute_path_outside_the_root(), rejects_parent_dir_traversal(), round_trips_a_file_inside_the_root() (+16 more)

### Community 77 - "tests.rs"
Cohesion: 0.13
Nodes (26): a_directory_swapped_for_a_symlink_takes_its_rows_along(), a_file_swapped_for_a_symlink_leaves_the_index(), an_index_from_an_older_format_is_rebuilt_on_open(), build_fixture(), find(), index_db_path_is_stable_and_lives_inside_the_app_data_dir(), indexed(), one_unreadable_file_does_not_fail_the_batch() (+18 more)

### Community 78 - "PtySession"
Cohesion: 0.15
Nodes (20): default_shell(), login_shell_path(), login_shell_path_is_the_shells_own(), PtySession, resolve_login_shell_path(), Box, Child, Error (+12 more)

### Community 79 - "default.json"
Cohesion: 0.22
Nodes (8): description, identifier, permissions, $schema, windows, core:default, dialog:default, main

### Community 81 - "anycode-store/src/lib.rs"
Cohesion: 0.07
Nodes (51): Event, EventScope, omits_empty_scope_and_payload(), roundtrips_through_json(), Into, Option, Self, String (+43 more)

### Community 82 - "Project audit — 2026-09-24"
Cohesion: 0.15
Nodes (13): 1. Phases 0–3: deliverables, 2. Architecture invariants, 3. V1 product contract (PRODUCT-SCOPE.md), 4. Security findings, 5. PRD scope that belongs to no phase, 6. Quality gaps, 7. What's next, Phase 0 · Foundation — exit: *launches on Windows and macOS* (+5 more)

### Community 83 - "useWorkbenchStore"
Cohesion: 0.09
Nodes (20): DiffPane(), EditorArea(), MonacoPane(), Explorer(), FileRow(), GitPanel(), STATUS_COLOR, STATUS_LABEL (+12 more)

### Community 85 - "agent_commands.rs"
Cohesion: 0.06
Nodes (86): a_path_that_escapes_the_root_resolves_to_none(), a_policy_file_that_exists_but_is_invalid_yaml_fails_closed(), a_relative_path_is_returned_as_is(), a_valid_policy_file_is_loaded_and_applied(), a_workspace_with_no_policy_file_restricts_nothing(), an_absolute_path_inside_the_root_resolves_to_its_relative_form(), ApprovalContext, ApprovalResponse (+78 more)

### Community 86 - "README.md"
Cohesion: 0.18
Nodes (10): Any Code — agent operating instructions, Agent context, Current visual identity, Develop locally, How it works, Project status, Releases, Repository layout (+2 more)

### Community 87 - "devDependencies"
Cohesion: 0.13
Nodes (15): devDependencies, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite, @vitejs/plugin-react, vitest (+7 more)

### Community 88 - "anycode-security/src/lib.rs"
Cohesion: 0.08
Nodes (23): classify_shell_command(), decide(), Decision, path_risk(), RiskLevel, Option, secret_bearing_paths_are_not_ordinary_files(), shell_risk_reason() (+15 more)

### Community 89 - "metered.rs"
Cohesion: 0.11
Nodes (28): a_clean_end_without_usage_is_success_with_unknown_counts(), a_completed_request_is_reported_once_with_real_counts(), an_abandoned_stream_is_reported_as_cancelled(), failures_are_reported_whether_at_open_or_mid_stream(), Metered, MeteredStream, Report, request() (+20 more)

### Community 90 - "tauri.ts"
Cohesion: 0.19
Nodes (14): ChatPanel(), ModelPicker(), ProvidersSection(), statusLabel(), useProviderModel(), ChatMessage, ChatRole, CommandRecord (+6 more)

### Community 91 - ".push"
Cohesion: 0.29
Nodes (10): handles_multiple_events_in_one_chunk(), parses_a_single_data_only_event(), parses_named_events(), reassembles_an_event_split_across_two_chunks(), Option, Self, String, Vec (+2 more)

### Community 92 - "types.rs"
Cohesion: 0.21
Nodes (17): role_str(), role_str(), Message, ModelDefinition, ModelRequest, RequestMetadata, Role, Into (+9 more)

### Community 93 - "Any Code brand"
Cohesion: 0.40
Nodes (4): Accessibility, Any Code brand, Attribution, Canonical assets

### Community 94 - "anycode-secrets/src/lib.rs"
Cohesion: 0.30
Nodes (14): a_missing_key_reads_as_none_not_an_error(), delete_api_key(), entry(), get_api_key(), removing_a_key_that_was_never_set_is_not_an_error(), round_trips_through_the_real_keychain(), Error, Option (+6 more)

### Community 95 - "ollama.rs"
Cohesion: 0.12
Nodes (19): build_chat_request(), caps_output_and_widens_the_context_window(), OllamaProvider, parse_line(), parses_a_content_line(), parses_a_whole_tool_call_and_decodes_its_name(), parses_the_final_usage_line(), request_carries_tools_and_links_results_to_their_call_by_name() (+11 more)

### Community 96 - "lsp.rs"
Cohesion: 0.11
Nodes (39): ChildStdin, an_oversized_content_length_is_refused_without_allocating(), extract_hover_text(), find_in(), find_on_path(), framing_round_trips(), is_executable_file(), language_id_for() (+31 more)

### Community 97 - "openai.rs"
Cohesion: 0.10
Nodes (29): a_compatible_provider_keeps_its_own_identity(), accumulates_argument_fragments_across_chunks(), build_chat_request(), builds_a_streaming_chat_request(), extract_tool_call_fragments(), extracts_tool_call_fragments_by_index(), finish_reason(), history_tool_calls_use_the_wire_name() (+21 more)

### Community 98 - "index_commands.rs"
Cohesion: 0.19
Nodes (23): build(), handle(), index_status(), IndexSlot, IndexStatus, publish(), ready_status(), AppHandle (+15 more)

### Community 99 - "live_openai.rs"
Cohesion: 0.67
Nodes (5): a_real_response_produces_a_tool_call(), a_real_response_reports_usage(), api_key(), model(), String

### Community 100 - "filesystem.rs"
Cohesion: 0.36
Nodes (10): context(), edit_cannot_escape_the_workspace_root(), edit_changes_only_the_named_span(), edit_refuses_text_that_is_absent_or_ambiguous(), read(), read_rejects_missing_path_argument(), TempDir, write() (+2 more)

### Community 101 - "Language"
Cohesion: 0.19
Nodes (15): Language, Candidate, classify(), extract_imports(), extract_symbols(), is_container_candidate(), predicates_hold(), quoted_after() (+7 more)

### Community 102 - "Value"
Cohesion: 0.17
Nodes (9): FilesystemEditTool, FilesystemReadTool, FilesystemWriteTool, reason_for_path(), risk_for_path(), Option, RiskLevel, String (+1 more)

### Community 103 - "trust.rs"
Cohesion: 0.17
Nodes (14): an_origin_cannot_break_out_of_its_attribute_or_line(), closing_tags_in_any_case_or_spacing_cannot_end_the_envelope(), defuse_closing_tags(), prompt_label(), Into, Self, String, Tagged (+6 more)

### Community 104 - "memory_commands.rs"
Cohesion: 0.22
Nodes (17): App, a_file_changed_after_its_preview_is_not_adopted(), a_symlinked_instruction_file_is_neither_listed_nor_adopted(), add_memory_at_workspace_scope_without_an_open_workspace_is_an_error(), adopt_instruction_refuses_a_file_over_64_kib(), adopt_instruction_refuses_a_path_not_in_instruction_files(), adopting_claude_md_creates_a_workspace_memory_and_is_then_reported_adopted(), preview_instruction() (+9 more)

### Community 106 - "Tool"
Cohesion: 0.14
Nodes (8): capability_risk(), CodeDefinitionTool, CodeReferencesTool, CodeSearchTool, RiskLevel, Send, Sync, Tool

### Community 107 - "3. VS Code extension compatibility and the package registry"
Cohesion: 0.33
Nodes (5): 3. VS Code extension compatibility and the package registry, Consequences, Context, Decision, Scope change record

### Community 108 - "ApprovalDialog.tsx"
Cohesion: 0.35
Nodes (7): ApprovalDialog(), RISK_EXPLANATION, grantLabel(), prefixLines(), requestSummary(), RiskLevel, TaskApprovalRequest

### Community 109 - "desktop/package.json"
Cohesion: 0.29
Nodes (6): description, homepage, name, private, type, version

### Community 110 - "scripts"
Cohesion: 0.29
Nodes (7): scripts, build, dev, lint, preview, tauri, test

### Community 111 - "ToolContext"
Cohesion: 0.27
Nodes (11): limit(), required(), Result, Value, Result, Arc, Error, Mutex (+3 more)

### Community 113 - "MemorySection.tsx"
Cohesion: 0.24
Nodes (10): MemoryRow(), MemorySection(), ADR-0004, scopeLabel(), sourceLabel(), ADR-0004, InstructionFile, Memory (+2 more)

### Community 114 - "workspace_policy.rs"
Cohesion: 0.17
Nodes (18): a_listed_capability_is_asked_every_time(), a_policy_can_never_lower_risk(), an_empty_or_unrelated_file_restricts_nothing(), PathAccess, PolicyError, PolicyFile, Protected, Error (+10 more)

### Community 115 - "StatusBar.tsx"
Cohesion: 0.44
Nodes (6): StatusBar(), useIndexStatus(), indexStatusLabel(), indexStatusTooltip(), indexCommands, IndexStatus

### Community 116 - "ProviderError"
Cohesion: 0.33
Nodes (9): done(), ModelDefinition, Result, Vec, Scripted, text(), ProviderError, Error (+1 more)

### Community 118 - "require_root"
Cohesion: 0.38
Nodes (11): list_dir(), read_file(), require_root(), Option, Result, State, String, Vec (+3 more)

### Community 121 - "search_live"
Cohesion: 0.41
Nodes (11): LineMatch, matcher(), references(), Path, Result, String, Vec, search_live() (+3 more)

### Community 122 - "code.rs"
Cohesion: 0.38
Nodes (8): context(), definition_uses_the_index_and_says_when_it_is_not_ready(), fixture(), references_are_whole_words(), TempDir, search_error(), search_finds_lines_and_respects_the_limit(), Display

### Community 123 - "terminal_commands.rs"
Cohesion: 0.29
Nodes (13): a_terminal_session_streams_from_the_first_byte_runs_input_and_cleans_up(), PtyDataEvent, PtyExitEvent, AppHandle, R, Result, State, String (+5 more)

### Community 124 - "Symbol"
Cohesion: 0.38
Nodes (5): row_to_symbol(), Option, Row, Symbol, SymbolKind

### Community 144 - "anycode-tools/src/lib.rs"
Cohesion: 0.12
Nodes (18): every_tool_name_survives_wire_encoding(), every_tool_produces_a_non_empty_spec(), missing_required(), missing_required_arguments_are_named(), Box, Drop, Item, Option (+10 more)

### Community 145 - "provider_commands.rs"
Cohesion: 0.20
Nodes (27): build_adapter(), build_provider(), ChatDeltaEvent, ChatDoneEvent, ChatErrorEvent, ChatToolCallEvent, endpoint_setting(), list_models() (+19 more)

### Community 146 - "GitStatusTool"
Cohesion: 0.22
Nodes (7): GitStatusTool, reports_an_untracked_file(), Path, Result, RiskLevel, Value, run()

### Community 147 - "git_status"
Cohesion: 0.36
Nodes (9): git_branch(), git_diff(), git_status(), Option, Result, State, String, Vec (+1 more)

### Community 149 - "4. Code intelligence, context and memory"
Cohesion: 0.40
Nodes (4): 4. Code intelligence, context and memory, Consequences, Context, Decision

### Community 170 - "TaskHistoryPanel.tsx"
Cohesion: 0.17
Nodes (16): TaskHistoryPanel(), ADR-0004, AuditEvent, eventsToEntries(), isTruncated(), text(), Truncated, truncatedResult() (+8 more)

### Community 171 - "current_path"
Cohesion: 0.38
Nodes (13): current_path(), get_last_workspace(), open_workspace(), open_workspace_at(), AppHandle, Option, PathBuf, R (+5 more)

### Community 173 - "TimelineEntry.tsx"
Cohesion: 0.19
Nodes (12): ContextInspector(), ContextItemRow(), TimelineEntry(), describeReason(), describeScale(), lineRange(), ContextItem, ContextPackage (+4 more)

### Community 174 - "fts.rs"
Cohesion: 0.40
Nodes (3): build_or_query(), neutralises_fts5_operators(), String

### Community 175 - "String"
Cohesion: 0.22
Nodes (22): add_memory(), adopt_instruction(), adopted_source(), delete_memory(), export_memories(), InstructionFile, list_memories(), list_tasks() (+14 more)

### Community 189 - "Engineering standards"
Cohesion: 0.25
Nodes (8): Commits and branches, Definition of done, Dependencies, Engineering standards, Performance targets, Quality bar, Release artifacts, Testing

## Knowledge Gaps
- **550 isolated node(s):** `printWidth`, `trailingComma`, `name`, `description`, `homepage` (+545 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ToolDefinition` connect `types.rs` to `agent_commands.rs`?**
  _High betweenness centrality (0.130) - this node is a cross-community bridge._
- **Why does `read_file_tool()` connect `types.rs` to `live_openai.rs`?**
  _High betweenness centrality (0.128) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `index_commands.rs`, `memory_commands.rs`, `current_path`, `PtySession`, `String`, `anycode-tools/src/lib.rs`, `anycode-store/src/lib.rs`, `provider_commands.rs`, `git_status`, `agent_commands.rs`, `require_root`, `terminal_commands.rs`?**
  _High betweenness centrality (0.094) - this node is a cross-community bridge._
- **What connects `printWidth`, `trailingComma`, `name` to the rest of the system?**
  _550 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `PRD.md` be split into smaller, more focused modules?**
  _Cohesion score 0.021052631578947368 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._