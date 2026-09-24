# Graph Report - Any Code  (2026-09-24)

## Corpus Check
- 138 files · ~248,829 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1535 nodes · 2535 edges · 151 communities (100 shown, 51 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 18 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1fce33d0`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- PRD.md
- README.md
- What You Must Do When Invoked
- What You Must Do When Invoked
- What You Must Do When Invoked
- provider_commands.rs
- anthropic.rs
- 98. Implementation Phases
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- Engineering standards
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
- Any Code brand
- Repository knowledge graph
- Roadmap
- AppState
- WorkspaceRoot
- shell.rs
- terminal_commands.rs
- default.json
- anycode-store/src/lib.rs
- Project audit — 2026-09-24
- useWorkbenchStore
- .prettierrc.json
- agent_commands.rs
- PROJECT-RULES.md
- devDependencies
- SettingsPanel.tsx
- Metered
- tauri.ts
- .push
- types.rs
- workbenchStore.ts
- anycode-secrets/src/lib.rs
- ollama.rs
- metered.rs
- openai.rs
- state.rs
- live_openai.rs
- filesystem.rs
- anycode-tools/src/lib.rs
- Value
- event.rs
- anycode-security/src/lib.rs
- require_root
- Send
- 3. VS Code extension compatibility and the package registry
- ApprovalDialog.tsx
- desktop/package.json
- scripts
- react-dom
- @xterm/xterm
- ToolError
- Message
- RiskLevel
- ProviderError
- WorkspaceRoot
- Uuid
- AsRef
- TempDir
- Sync
- TempDir
- AppHandle
- Result
- State
- String
- Message
- ProviderError
- Client
- Context
- AsRef
- TempDir
- Item
- ModelRequest
- ProviderError
- ProviderManifest
- StreamEvent
- Default
- Message
- ModelRequest
- ProviderError
- ProviderManifest
- StreamEvent
- Role
- ToolCallRequest
- ToolDefinition
- Verdict
- agent_live_test.rs
- current_path
- git_status

## God Nodes (most connected - your core abstractions)
1. `AppState` - 36 edges
2. `useWorkbenchStore` - 20 edges
3. `ProviderError` - 17 edges
4. `ModelRequest` - 16 edges
5. `compilerOptions` - 15 edges
6. `WorkspaceRoot` - 15 edges
7. `Store` - 15 edges
8. `StoreError` - 15 edges
9. `Metered` - 15 edges
10. `ToolContext` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Store` --references--> `Connection`  [EXTRACTED]
  crates/anycode-store/src/lib.rs → apps/desktop/src-tauri/src/provider_commands.rs
- `App()` --calls--> `applyTheme()`  [EXTRACTED]
  apps/desktop/src/App.tsx → packages/design-tokens/src/index.ts
- `provider_error_message()` --references--> `ProviderError`  [EXTRACTED]
  apps/desktop/src-tauri/src/provider_commands.rs → crates/anycode-models/src/types.rs
- `send_chat()` --references--> `Message`  [EXTRACTED]
  apps/desktop/src-tauri/src/provider_commands.rs → crates/anycode-models/src/types.rs
- `TaskDoneEvent` --references--> `Verdict`  [EXTRACTED]
  apps/desktop/src-tauri/src/agent_commands.rs → crates/anycode-agent/src/verdict.rs

## Import Cycles
- 2-file cycle: `apps/desktop/src-tauri/src/agent_commands.rs -> apps/desktop/src-tauri/src/lib.rs -> apps/desktop/src-tauri/src/agent_commands.rs`
- 2-file cycle: `crates/anycode-tools/src/filesystem.rs -> crates/anycode-tools/src/lib.rs -> crates/anycode-tools/src/filesystem.rs`

## Communities (151 total, 51 thin omitted)

### Community 0 - "PRD.md"
Cohesion: 0.02
Nodes (94): 100. Quality Bar, 101. Definition of Done, 102. Repository Implementation Contract, 103. Architectural Rules, 104. Critical Product Differentiator, 105. Product North Star, 10. Non Goals for V1, 11. Platform Architecture (+86 more)

### Community 1 - "README.md"
Cohesion: 0.20
Nodes (9): Agent context, Current visual identity, Develop locally, How it works, Project status, Releases, Repository layout, Security and operations (+1 more)

### Community 2 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native AGENTS.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 3 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 4 - "What You Must Do When Invoked"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 5 - "provider_commands.rs"
Cohesion: 0.18
Nodes (29): build_adapter(), build_provider(), ChatDeltaEvent, ChatDoneEvent, ChatErrorEvent, ChatToolCallEvent, Connection, endpoint_setting() (+21 more)

### Community 6 - "anthropic.rs"
Cohesion: 0.10
Nodes (18): AnthropicProvider, build_messages_request(), parse_event(), parses_content_block_delta(), parses_output_usage_from_message_delta(), Client, ModelDefinition, ModelProvider (+10 more)

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

### Community 11 - "Engineering standards"
Cohesion: 0.25
Nodes (8): Commits and branches, Definition of done, Dependencies, Engineering standards, Performance targets, Quality bar, Release artifacts, Testing

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
Nodes (17): @anycode/design-tokens, dependencies, @anycode/design-tokens, monaco-editor, react, @tanstack/react-query, @tauri-apps/api, @tauri-apps/plugin-dialog (+9 more)

### Community 53 - "verdict.rs"
Cohesion: 0.11
Nodes (20): parse_plan(), String, Vec, a_check_that_failed_then_passed_has_passed(), a_check_that_passed_then_failed_has_failed(), check(), CommandRecord, exploration_never_counts_either_way() (+12 more)

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
Cohesion: 0.13
Nodes (14): App(), THEMES, GitPanel(), STATUS_COLOR, STATUS_LABEL, Icon(), IconName, StatusBar() (+6 more)

### Community 61 - "anycode-git/src/lib.rs"
Cohesion: 0.17
Nodes (21): current_branch(), diff_file(), diff_reports_head_and_working_content(), empty_repository_has_no_branch(), FileDiff, FileStatus, GitError, reports_untracked_and_modified_files() (+13 more)

### Community 64 - "anycode-desktop"
Cohesion: 0.25
Nodes (11): anycode-agent, anycode-core, anycode-desktop, anycode-fs, anycode-git, anycode-models, anycode-secrets, anycode-security (+3 more)

### Community 65 - "Releasing the desktop app"
Cohesion: 0.33
Nodes (6): Auto-update signature (separate from code signing), macOS — code signing + notarization, Releasing the desktop app, Trigger it, What "just build it" gets you today, Windows — code signing

### Community 66 - "Security model"
Cohesion: 0.25
Nodes (7): Permissions, Reporting, Secrets, Security model, Shell risk classes, The one rule, Threat model

### Community 67 - "Verification history"
Cohesion: 0.12
Nodes (16): 2026-08-23T00:41:22Z — Brand integration baseline, 2026-08-23T00:45:09Z — GitHub CI after branding and governance push, 2026-08-24 — Phase 1 workbench review and UX remediation, 2026-08-24 — Second-pass UI/UX and ledger audit, 2026-09-23 — Integrated terminal defect hunt, 2026-09-23 — Phase 3 completion: planner, state machine, audit log, live exit condition, 2026-09-23 — Phase 3 MVP: agent dock, approvals, cancellation, evidence, 2026-09-23 — Scope decision: extensions and the store (documents only) (+8 more)

### Community 68 - "AgentPanel.tsx"
Cohesion: 0.21
Nodes (15): AgentPanel(), STATE_LABEL, agentCommands, TaskDone, TaskEvidence, TaskToolCall, TaskToolResult, TaskUsage (+7 more)

### Community 69 - "Agent Routing Policy"
Cohesion: 0.33
Nodes (6): Agent Routing Policy, Before writing code, Commands, Definition of done, graphify, Non-negotiable

### Community 70 - "Staging operations and monitoring"
Cohesion: 0.33
Nodes (6): Expected staging services, Incident response, Minimum online checks, Monitoring rules, Operating boundary, Staging operations and monitoring

### Community 71 - "Any Code brand"
Cohesion: 0.40
Nodes (4): Accessibility, Any Code brand, Attribution, Canonical assets

### Community 72 - "Repository knowledge graph"
Cohesion: 0.40
Nodes (5): Installation for a new workstation, Installed integration, Repository knowledge graph, Required agent workflow, What the graph may contain

### Community 73 - "Roadmap"
Cohesion: 0.20
Nodes (10): Assigned by the 2026-09-24 audit, Current phase: 4 · Code intelligence, Distribution gate, Phase 0 · Foundation, Phase 1 · Workbench, Phase 2 · Provider layer, Phase 3 · Agent runtime, Phase 3 close-out — done except C4 (needs a Windows machine) (+2 more)

### Community 75 - "AppState"
Cohesion: 0.14
Nodes (21): app_context(), AppState, get_theme(), Arc, AtomicBool, Context, Mutex, Option (+13 more)

### Community 76 - "WorkspaceRoot"
Cohesion: 0.16
Nodes (21): AsRef, accepts_an_absolute_path_inside_the_root(), Entry, FsError, list_dir(), read_file(), rejects_an_absolute_path_outside_the_root(), rejects_parent_dir_traversal() (+13 more)

### Community 77 - "shell.rs"
Cohesion: 0.15
Nodes (11): a_failing_command_is_a_result_not_an_error(), a_grant_covers_only_the_command_it_was_given_for(), captures_stdout_and_exit_code(), context(), Option, Result, RiskLevel, String (+3 more)

### Community 78 - "terminal_commands.rs"
Cohesion: 0.10
Nodes (33): AppHandle, a_terminal_session_streams_from_the_first_byte_runs_input_and_cleans_up(), PtyDataEvent, PtyExitEvent, terminal_kill(), terminal_resize(), terminal_spawn(), terminal_write() (+25 more)

### Community 79 - "default.json"
Cohesion: 0.22
Nodes (8): description, identifier, permissions, $schema, windows, core:default, dialog:default, main

### Community 81 - "anycode-store/src/lib.rs"
Cohesion: 0.15
Nodes (20): a_tasks_events_come_back_in_order_and_only_for_that_task(), granting_twice_does_not_error(), permission_grants_are_scoped_per_workspace(), Error, Option, Path, Result, Self (+12 more)

### Community 82 - "Project audit — 2026-09-24"
Cohesion: 0.15
Nodes (13): 1. Phases 0–3: deliverables, 2. Architecture invariants, 3. V1 product contract (PRODUCT-SCOPE.md), 4. Security findings, 5. PRD scope that belongs to no phase, 6. Quality gaps, 7. What's next, Phase 0 · Foundation — exit: *launches on Windows and macOS* (+5 more)

### Community 83 - "useWorkbenchStore"
Cohesion: 0.35
Nodes (8): DiffPane(), EditorArea(), MonacoPane(), EXTENSION_LANGUAGE, languageForPath(), loadMonaco(), Monaco, useWorkbenchStore

### Community 85 - "agent_commands.rs"
Cohesion: 0.09
Nodes (60): ApprovalContext, ApprovalResponse, cancel_task(), changed_since(), dirty_paths(), execute_tool(), for_audit(), GateFacts (+52 more)

### Community 86 - "PROJECT-RULES.md"
Cohesion: 0.27
Nodes (3): graphify, QA record, Any Code — agent operating instructions

### Community 87 - "devDependencies"
Cohesion: 0.13
Nodes (15): devDependencies, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite, @vitejs/plugin-react, vitest (+7 more)

### Community 88 - "SettingsPanel.tsx"
Cohesion: 0.20
Nodes (11): Command, CommandPalette(), SettingsPanel(), THEMES, useDialogFocus(), applyTheme(), fonts, motion (+3 more)

### Community 89 - "Metered"
Cohesion: 0.16
Nodes (16): Metered, Report, Arc, Box, Context, Drop, Into, ModelProvider (+8 more)

### Community 90 - "tauri.ts"
Cohesion: 0.17
Nodes (14): ChatPanel(), ProvidersSection(), statusLabel(), useProviderModel(), ChatMessage, ChatRole, CommandRecord, FileDiff (+6 more)

### Community 91 - ".push"
Cohesion: 0.29
Nodes (10): handles_multiple_events_in_one_chunk(), parses_a_single_data_only_event(), parses_named_events(), reassembles_an_event_split_across_two_chunks(), Option, Self, String, Vec (+2 more)

### Community 92 - "types.rs"
Cohesion: 0.21
Nodes (18): role_str(), role_str(), Message, ModelDefinition, ModelRequest, RequestMetadata, Role, Into (+10 more)

### Community 93 - "workbenchStore.ts"
Cohesion: 0.17
Nodes (8): Explorer(), FileRow(), FsEntry, WorkspaceInfo, BottomPanel, OpenTab, SidePanel, WorkbenchState

### Community 94 - "anycode-secrets/src/lib.rs"
Cohesion: 0.30
Nodes (14): a_missing_key_reads_as_none_not_an_error(), delete_api_key(), entry(), get_api_key(), removing_a_key_that_was_never_set_is_not_an_error(), round_trips_through_the_real_keychain(), Error, Option (+6 more)

### Community 95 - "ollama.rs"
Cohesion: 0.12
Nodes (19): build_chat_request(), caps_output_and_widens_the_context_window(), OllamaProvider, parse_line(), parses_a_content_line(), parses_a_whole_tool_call_and_decodes_its_name(), parses_the_final_usage_line(), request_carries_tools_and_links_results_to_their_call_by_name() (+11 more)

### Community 96 - "metered.rs"
Cohesion: 0.17
Nodes (16): a_clean_end_without_usage_is_success_with_unknown_counts(), a_completed_request_is_reported_once_with_real_counts(), an_abandoned_stream_is_reported_as_cancelled(), done(), failures_are_reported_whether_at_open_or_mid_stream(), MeteredStream, request(), ModelStream (+8 more)

### Community 97 - "openai.rs"
Cohesion: 0.10
Nodes (30): a_compatible_provider_keeps_its_own_identity(), accumulates_argument_fragments_across_chunks(), build_chat_request(), builds_a_streaming_chat_request(), extract_tool_call_fragments(), extracts_tool_call_fragments_by_index(), finish_reason(), history_tool_calls_use_the_wire_name() (+22 more)

### Community 98 - "state.rs"
Cohesion: 0.17
Nodes (8): InvalidTransition, Default, Result, Self, run(), TaskMachine, TaskState, TaskState

### Community 99 - "live_openai.rs"
Cohesion: 0.52
Nodes (6): a_real_response_produces_a_tool_call(), a_real_response_reports_usage(), api_key(), model(), read_file_tool(), String

### Community 100 - "filesystem.rs"
Cohesion: 0.34
Nodes (11): context(), edit_cannot_escape_the_workspace_root(), edit_changes_only_the_named_span(), edit_refuses_text_that_is_absent_or_ambiguous(), read(), read_rejects_missing_path_argument(), write(), write_cannot_escape_the_workspace_root() (+3 more)

### Community 101 - "anycode-tools/src/lib.rs"
Cohesion: 0.11
Nodes (22): every_tool_name_survives_wire_encoding(), every_tool_produces_a_non_empty_spec(), missing_required(), missing_required_arguments_are_named(), Box, Drop, Item, Option (+14 more)

### Community 102 - "Value"
Cohesion: 0.22
Nodes (8): FilesystemEditTool, FilesystemWriteTool, reason_for_path(), risk_for_path(), Option, RiskLevel, String, Value

### Community 103 - "event.rs"
Cohesion: 0.10
Nodes (22): Event, EventScope, omits_empty_scope_and_payload(), roundtrips_through_json(), Into, Option, Self, String (+14 more)

### Community 104 - "anycode-security/src/lib.rs"
Cohesion: 0.09
Nodes (19): capability_risk(), classify_shell_command(), decide(), Decision, path_risk(), RiskLevel, Option, secret_bearing_paths_are_not_ordinary_files() (+11 more)

### Community 105 - "require_root"
Cohesion: 0.33
Nodes (12): list_dir(), read_file(), require_root(), Option, Result, State, String, Vec (+4 more)

### Community 107 - "3. VS Code extension compatibility and the package registry"
Cohesion: 0.33
Nodes (5): 3. VS Code extension compatibility and the package registry, Consequences, Context, Decision, Scope change record

### Community 108 - "ApprovalDialog.tsx"
Cohesion: 0.30
Nodes (8): ApprovalDialog(), RISK_EXPLANATION, grantLabel(), prefixLines(), requestSummary(), ApprovalResponse, RiskLevel, TaskApprovalRequest

### Community 109 - "desktop/package.json"
Cohesion: 0.29
Nodes (6): description, homepage, name, private, type, version

### Community 110 - "scripts"
Cohesion: 0.29
Nodes (7): scripts, build, dev, lint, preview, tauri, test

### Community 113 - "ToolError"
Cohesion: 0.20
Nodes (6): FilesystemReadTool, Result, Error, ToolError, FsError, GitError

### Community 116 - "ProviderError"
Cohesion: 0.48
Nodes (5): ModelDefinition, Vec, Scripted, ProviderError, Error

### Community 148 - "agent_live_test.rs"
Cohesion: 0.26
Nodes (11): an_agent_implements_and_verifies_a_repository_task(), git(), make_repo(), one_line(), original_suite_passes(), Path, PathBuf, String (+3 more)

### Community 149 - "current_path"
Cohesion: 0.45
Nodes (11): current_path(), get_last_workspace(), open_workspace(), open_workspace_at(), Option, PathBuf, Result, State (+3 more)

### Community 150 - "git_status"
Cohesion: 0.36
Nodes (9): git_branch(), git_diff(), git_status(), Option, Result, State, String, Vec (+1 more)

## Knowledge Gaps
- **518 isolated node(s):** `Current quality status`, `Review protocol`, `2026-09-24 — Close-out, continued: C3, C5, C6, C7 verified live`, `2026-09-24 — Phase 3 close-out (C1–C10)`, `2026-09-24 — Full project audit` (+513 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **51 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `provider_commands.rs`, `require_root`, `agent_live_test.rs`, `agent_commands.rs`, `git_status`, `current_path`?**
  _High betweenness centrality (0.040) - this node is a cross-community bridge._
- **Why does `Tool` connect `anycode-tools/src/lib.rs` to `anycode-security/src/lib.rs`, `ToolError`, `shell.rs`, `Value`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._
- **Why does `decide()` connect `anycode-security/src/lib.rs` to `agent_commands.rs`?**
  _High betweenness centrality (0.020) - this node is a cross-community bridge._
- **What connects `Current quality status`, `Review protocol`, `2026-09-24 — Close-out, continued: C3, C5, C6, C7 verified live` to the rest of the system?**
  _518 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `PRD.md` be split into smaller, more focused modules?**
  _Cohesion score 0.021052631578947368 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._