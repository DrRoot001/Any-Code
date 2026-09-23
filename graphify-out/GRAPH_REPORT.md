# Graph Report - Any Code  (2026-09-24)

## Corpus Check
- 133 files · ~239,905 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1366 nodes · 2206 edges · 110 communities (94 shown, 16 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 18 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1f86131b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- PRD.md
- README.md
- What You Must Do When Invoked
- What You Must Do When Invoked
- What You Must Do When Invoked
- agent_live_test.rs
- anthropic.rs
- 98. Implementation Phases
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- graphify reference: extra exports and benchmark
- Engineering standards
- Project rules and governance
- tauri.conf.json
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
- tauri.ts
- Any Code — agent operating instructions
- Staging operations and monitoring
- Any Code brand
- Repository knowledge graph
- Roadmap
- AppState
- WorkspaceRoot
- provider_commands.rs
- PtySession
- default.json
- Store
- Project audit — 2026-09-24
- useWorkbenchStore
- .prettierrc.json
- agent_commands.rs
- PROJECT-RULES.md
- SettingsPanel.tsx
- workbenchStore.ts
- types.rs
- ChatPanel.tsx
- .push
- Message
- Explorer.tsx
- anycode-secrets/src/lib.rs
- ollama.rs
- ProviderError
- openai.rs
- state.rs
- live_openai.rs
- filesystem.rs
- anycode-tools/src/lib.rs
- FilesystemEditTool
- shell.rs
- GitStatusTool
- TempDir
- 3. VS Code extension compatibility and the package registry
- anycode-security/src/lib.rs
- ToolContext

## God Nodes (most connected - your core abstractions)
1. `AppState` - 32 edges
2. `useWorkbenchStore` - 20 edges
3. `WorkspaceRoot` - 16 edges
4. `Store` - 16 edges
5. `compilerOptions` - 15 edges
6. `Message` - 14 edges
7. `ToolContext` - 14 edges
8. `ModelRequest` - 13 edges
9. `ProviderError` - 13 edges
10. `StoreError` - 13 edges

## Surprising Connections (you probably didn't know these)
- `anycode-desktop` --depends_on--> `anycode-agent`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-agent/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-core`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-core/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-fs`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-fs/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-git`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-git/Cargo.toml
- `anycode-desktop` --depends_on--> `anycode-models`  [EXTRACTED]
  apps/desktop/src-tauri/Cargo.toml → crates/anycode-models/Cargo.toml

## Import Cycles
- None detected.

## Communities (110 total, 16 thin omitted)

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

### Community 5 - "agent_live_test.rs"
Cohesion: 0.29
Nodes (10): an_agent_implements_and_verifies_a_repository_task(), git(), make_repo(), one_line(), original_suite_passes(), Path, PathBuf, String (+2 more)

### Community 6 - "anthropic.rs"
Cohesion: 0.15
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

### Community 11 - "Engineering standards"
Cohesion: 0.25
Nodes (8): Commits and branches, Definition of done, Dependencies, Engineering standards, Performance targets, Quality bar, Release artifacts, Testing

### Community 12 - "Project rules and governance"
Cohesion: 0.25
Nodes (8): Architecture rules, Authority and source order, Decision rules, Delivery rules, Product rules, Project rules and governance, Quality and release rules, Security and privacy rules

### Community 13 - "tauri.conf.json"
Cohesion: 0.06
Nodes (30): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+22 more)

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
Cohesion: 0.04
Nodes (46): @anycode/design-tokens, dependencies, @anycode/design-tokens, monaco-editor, react, react-dom, @tanstack/react-query, @tauri-apps/api (+38 more)

### Community 53 - "verdict.rs"
Cohesion: 0.12
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
Cohesion: 0.19
Nodes (10): App(), THEMES, Icon(), IconName, StatusBar(), decodeBase64(), TerminalPanel(), WelcomeScreen() (+2 more)

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
Cohesion: 0.14
Nodes (14): 2026-08-23T00:41:22Z — Brand integration baseline, 2026-08-23T00:45:09Z — GitHub CI after branding and governance push, 2026-08-24 — Phase 1 workbench review and UX remediation, 2026-08-24 — Second-pass UI/UX and ledger audit, 2026-09-23 — Integrated terminal defect hunt, 2026-09-23 — Phase 3 completion: planner, state machine, audit log, live exit condition, 2026-09-23 — Phase 3 MVP: agent dock, approvals, cancellation, evidence, 2026-09-23 — Scope decision: extensions and the store (documents only) (+6 more)

### Community 68 - "tauri.ts"
Cohesion: 0.11
Nodes (25): AgentPanel(), argumentSummary(), Entry, resultSummary(), STATE_LABEL, ApprovalDialog(), prefixLines(), requestSummary() (+17 more)

### Community 69 - "Any Code — agent operating instructions"
Cohesion: 0.33
Nodes (6): Any Code — agent operating instructions, Before writing code, Commands, Definition of done, graphify, Non-negotiable

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
Nodes (10): Assigned by the 2026-09-24 audit, Current phase: 3 close-out, Distribution gate, Phase 0 · Foundation, Phase 1 · Workbench, Phase 2 · Provider layer, Phase 3 · Agent runtime, Phase 4 · Code intelligence (next) (+2 more)

### Community 75 - "AppState"
Cohesion: 0.08
Nodes (54): list_dir(), read_file(), require_root(), Option, Result, State, String, Vec (+46 more)

### Community 76 - "WorkspaceRoot"
Cohesion: 0.16
Nodes (21): Entry, FsError, list_dir(), read_file(), rejects_absolute_path(), rejects_parent_dir_traversal(), round_trips_a_file_inside_the_root(), AsRef (+13 more)

### Community 77 - "provider_commands.rs"
Cohesion: 0.20
Nodes (20): build_provider(), ChatDeltaEvent, ChatDoneEvent, ChatErrorEvent, ChatToolCallEvent, list_models(), list_providers(), provider_error_message() (+12 more)

### Community 78 - "PtySession"
Cohesion: 0.16
Nodes (19): Child, default_shell(), login_shell_path(), login_shell_path_is_the_shells_own(), PtySession, resolve_login_shell_path(), Box, Error (+11 more)

### Community 79 - "default.json"
Cohesion: 0.22
Nodes (8): description, identifier, permissions, $schema, windows, core:default, dialog:default, main

### Community 81 - "Store"
Cohesion: 0.06
Nodes (41): Connection, Event, EventScope, omits_empty_scope_and_payload(), roundtrips_through_json(), Into, Option, Self (+33 more)

### Community 82 - "Project audit — 2026-09-24"
Cohesion: 0.15
Nodes (13): 1. Phases 0–3: deliverables, 2. Architecture invariants, 3. V1 product contract (PRODUCT-SCOPE.md), 4. Security findings, 5. PRD scope that belongs to no phase, 6. Quality gaps, 7. What's next, Phase 0 · Foundation — exit: *launches on Windows and macOS* (+5 more)

### Community 83 - "useWorkbenchStore"
Cohesion: 0.21
Nodes (8): DiffPane(), EditorArea(), MonacoPane(), EXTENSION_LANGUAGE, languageForPath(), loadMonaco(), Monaco, useWorkbenchStore

### Community 85 - "agent_commands.rs"
Cohesion: 0.09
Nodes (61): ApprovalResponse, cancel_task(), changed_since(), dirty_paths(), execute_tool(), for_audit(), grant_permission(), has_grant() (+53 more)

### Community 87 - "SettingsPanel.tsx"
Cohesion: 0.20
Nodes (11): Command, CommandPalette(), SettingsPanel(), THEMES, useDialogFocus(), applyTheme(), fonts, motion (+3 more)

### Community 88 - "workbenchStore.ts"
Cohesion: 0.18
Nodes (9): GitPanel(), STATUS_COLOR, STATUS_LABEL, GitFileStatus, WorkspaceInfo, BottomPanel, OpenTab, SidePanel (+1 more)

### Community 89 - "types.rs"
Cohesion: 0.18
Nodes (10): ModelDefinition, ProviderAuthMode, ProviderManifest, Value, StreamEvent, tool_name_from_wire(), tool_name_to_wire(), ToolDefinition (+2 more)

### Community 90 - "ChatPanel.tsx"
Cohesion: 0.27
Nodes (8): ChatPanel(), ModelPicker(), ProvidersSection(), useProviderModel(), ChatMessage, ModelDefinition, providerCommands, ProviderStatus

### Community 91 - ".push"
Cohesion: 0.29
Nodes (10): handles_multiple_events_in_one_chunk(), parses_a_single_data_only_event(), parses_named_events(), reassembles_an_event_split_across_two_chunks(), Option, Self, String, Vec (+2 more)

### Community 92 - "Message"
Cohesion: 0.31
Nodes (11): role_str(), role_str(), Message, ModelRequest, RequestMetadata, Role, Into, Option (+3 more)

### Community 93 - "Explorer.tsx"
Cohesion: 0.29
Nodes (3): Explorer(), FileRow(), FsEntry

### Community 94 - "anycode-secrets/src/lib.rs"
Cohesion: 0.42
Nodes (9): delete_api_key(), entry(), get_api_key(), Error, Option, Result, String, SecretError (+1 more)

### Community 95 - "ollama.rs"
Cohesion: 0.14
Nodes (17): build_chat_request(), OllamaProvider, parse_line(), parses_a_content_line(), parses_a_whole_tool_call_and_decodes_its_name(), parses_the_final_usage_line(), request_carries_tools_and_links_results_to_their_call_by_name(), Client (+9 more)

### Community 96 - "ProviderError"
Cohesion: 0.24
Nodes (7): OpenAiProvider, Client, ModelDefinition, ModelStream, Result, ProviderError, Error

### Community 97 - "openai.rs"
Cohesion: 0.15
Nodes (19): accumulates_argument_fragments_across_chunks(), build_chat_request(), builds_a_streaming_chat_request(), extract_tool_call_fragments(), extracts_tool_call_fragments_by_index(), finish_reason(), history_tool_calls_use_the_wire_name(), includes_tool_definitions_when_present() (+11 more)

### Community 98 - "state.rs"
Cohesion: 0.18
Nodes (7): InvalidTransition, Default, Result, Self, run(), TaskMachine, TaskState

### Community 99 - "live_openai.rs"
Cohesion: 0.67
Nodes (5): a_real_response_produces_a_tool_call(), a_real_response_reports_usage(), api_key(), model(), String

### Community 100 - "filesystem.rs"
Cohesion: 0.36
Nodes (11): context(), edit_cannot_escape_the_workspace_root(), edit_changes_only_the_named_span(), edit_refuses_text_that_is_absent_or_ambiguous(), read(), read_rejects_missing_path_argument(), String, TempDir (+3 more)

### Community 101 - "anycode-tools/src/lib.rs"
Cohesion: 0.16
Nodes (16): every_tool_name_survives_wire_encoding(), every_tool_produces_a_non_empty_spec(), missing_required(), missing_required_arguments_are_named(), Box, Option, Send, Sync (+8 more)

### Community 102 - "FilesystemEditTool"
Cohesion: 0.36
Nodes (3): capability_risk(), FilesystemEditTool, RiskLevel

### Community 104 - "shell.rs"
Cohesion: 0.18
Nodes (9): a_failing_command_is_a_result_not_an_error(), captures_stdout_and_exit_code(), context(), Result, RiskLevel, TempDir, Value, ShellExecuteTool (+1 more)

### Community 105 - "GitStatusTool"
Cohesion: 0.24
Nodes (6): GitStatusTool, reports_an_untracked_file(), Path, RiskLevel, Value, run()

### Community 106 - "TempDir"
Cohesion: 0.25
Nodes (5): Drop, Path, PathBuf, Self, TempDir

### Community 107 - "3. VS Code extension compatibility and the package registry"
Cohesion: 0.33
Nodes (5): 3. VS Code extension compatibility and the package registry, Consequences, Context, Decision, Scope change record

### Community 108 - "anycode-security/src/lib.rs"
Cohesion: 0.19
Nodes (5): classify_shell_command(), decide(), Decision, RiskLevel, StandingGrant

### Community 113 - "ToolContext"
Cohesion: 0.16
Nodes (9): FilesystemReadTool, FilesystemWriteTool, Result, Value, Result, Error, String, ToolContext (+1 more)

## Knowledge Gaps
- **502 isolated node(s):** `printWidth`, `trailingComma`, `name`, `description`, `homepage` (+497 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ToolDefinition` connect `types.rs` to `Message`, `agent_commands.rs`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **Why does `read_file_tool()` connect `types.rs` to `live_openai.rs`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `Store`, `anycode-tools/src/lib.rs`, `agent_commands.rs`, `PtySession`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **What connects `printWidth`, `trailingComma`, `name` to the rest of the system?**
  _502 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `PRD.md` be split into smaller, more focused modules?**
  _Cohesion score 0.021052631578947368 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._