# Graph Report - Any Code  (2026-09-23)

## Corpus Check
- 125 files · ~226,624 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1221 nodes · 1836 edges · 113 communities (88 shown, 25 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 11 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `10aba304`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- PRD.md
- README.md
- What You Must Do When Invoked
- What You Must Do When Invoked
- What You Must Do When Invoked
- Store
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
- anycode-core
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
- event.rs
- AGENTS.md
- DiffPane.tsx
- .prettierrc.json
- agent_commands.rs
- SettingsPanel.tsx
- workbenchStore.ts
- types.rs
- ChatPanel.tsx
- .push
- Message
- useWorkbenchStore
- entry
- ProviderError
- OpenAiProvider
- openai.rs
- anycode-security/src/lib.rs
- live_openai.rs
- Tool
- AppHandle
- State
- ToolError
- ToolContext
- GitStatusTool
- Box
- anycode-tools/src/lib.rs
- Error
- Path
- Self
- Send
- Sync

## God Nodes (most connected - your core abstractions)
1. `AppState` - 27 edges
2. `useWorkbenchStore` - 20 edges
3. `compilerOptions` - 15 edges
4. `WorkspaceRoot` - 14 edges
5. `run_gated_tool()` - 14 edges
6. `run_task()` - 14 edges
7. `Store` - 13 edges
8. `Message` - 13 edges
9. `ModelRequest` - 13 edges
10. `ProviderError` - 13 edges

## Surprising Connections (you probably didn't know these)
- `run_gated_tool()` --calls--> `decide()`  [INFERRED]
  apps/desktop/src-tauri/src/agent_commands.rs → crates/anycode-security/src/lib.rs
- `App()` --calls--> `applyTheme()`  [EXTRACTED]
  apps/desktop/src/App.tsx → packages/design-tokens/src/index.ts
- `AppState` --references--> `PtySession`  [EXTRACTED]
  apps/desktop/src-tauri/src/lib.rs → crates/anycode-terminal/src/lib.rs
- `AppState` --references--> `ToolRegistry`  [EXTRACTED]
  apps/desktop/src-tauri/src/lib.rs → crates/anycode-tools/src/lib.rs
- `AppState` --references--> `Store`  [EXTRACTED]
  apps/desktop/src-tauri/src/lib.rs → crates/anycode-store/src/lib.rs

## Import Cycles
- None detected.

## Communities (113 total, 25 thin omitted)

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

### Community 5 - "Store"
Cohesion: 0.16
Nodes (17): Connection, granting_twice_does_not_error(), permission_grants_are_scoped_per_workspace(), AsRef, Error, Option, Path, Result (+9 more)

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
Nodes (8): App(), THEMES, Icon(), IconName, decodeBase64(), TerminalPanel(), commands, queryClient

### Community 61 - "anycode-git/src/lib.rs"
Cohesion: 0.17
Nodes (21): current_branch(), diff_file(), diff_reports_head_and_working_content(), empty_repository_has_no_branch(), FileDiff, FileStatus, GitError, reports_untracked_and_modified_files() (+13 more)

### Community 64 - "anycode-desktop"
Cohesion: 0.31
Nodes (9): anycode-desktop, anycode-fs, anycode-git, anycode-models, anycode-secrets, anycode-security, anycode-store, anycode-terminal (+1 more)

### Community 65 - "Releasing the desktop app"
Cohesion: 0.29
Nodes (6): Auto-update signature (separate from code signing), macOS — code signing + notarization, Releasing the desktop app, Trigger it, What "just build it" gets you today, Windows — code signing

### Community 66 - "Security model"
Cohesion: 0.25
Nodes (7): Permissions, Reporting, Secrets, Security model, Shell risk classes, The one rule, Threat model

### Community 67 - "Verification history"
Cohesion: 0.18
Nodes (11): 2026-08-23T00:41:22Z — Brand integration baseline, 2026-08-23T00:45:09Z — GitHub CI after branding and governance push, 2026-08-24 — Phase 1 workbench review and UX remediation, 2026-08-24 — Second-pass UI/UX and ledger audit, 2026-09-23 — Integrated terminal defect hunt, 2026-09-23 — Phase 3 MVP: agent dock, approvals, cancellation, evidence, Current quality status, Open QA risks (+3 more)

### Community 68 - "tauri.ts"
Cohesion: 0.14
Nodes (20): AgentPanel(), argumentSummary(), Entry, resultSummary(), ApprovalDialog(), requestSummary(), RISK_EXPLANATION, agentCommands (+12 more)

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
Cohesion: 0.29
Nodes (7): Current phase: 3 · Agent runtime, Distribution gate, Phase 0 · Foundation, Phase 1 · Workbench, Phase 2 · Provider layer, Roadmap, V1 success criterion

### Community 75 - "AppState"
Cohesion: 0.10
Nodes (43): list_dir(), read_file(), require_root(), Option, Result, State, String, Vec (+35 more)

### Community 76 - "WorkspaceRoot"
Cohesion: 0.16
Nodes (21): Entry, FsError, list_dir(), read_file(), rejects_absolute_path(), rejects_parent_dir_traversal(), round_trips_a_file_inside_the_root(), AsRef (+13 more)

### Community 77 - "provider_commands.rs"
Cohesion: 0.20
Nodes (20): build_provider(), ChatDeltaEvent, ChatDoneEvent, ChatErrorEvent, ChatToolCallEvent, list_models(), list_providers(), provider_error_message() (+12 more)

### Community 78 - "PtySession"
Cohesion: 0.11
Nodes (27): AppHandle, PtyDataEvent, PtyExitEvent, Result, String, terminal_kill(), terminal_resize(), terminal_spawn() (+19 more)

### Community 79 - "default.json"
Cohesion: 0.22
Nodes (8): description, identifier, permissions, $schema, windows, core:default, dialog:default, main

### Community 81 - "event.rs"
Cohesion: 0.11
Nodes (19): Event, EventScope, omits_empty_scope_and_payload(), roundtrips_through_json(), Into, Option, Self, String (+11 more)

### Community 83 - "DiffPane.tsx"
Cohesion: 0.33
Nodes (7): DiffPane(), EditorArea(), MonacoPane(), EXTENSION_LANGUAGE, languageForPath(), loadMonaco(), Monaco

### Community 85 - "agent_commands.rs"
Cohesion: 0.12
Nodes (47): ApprovalResponse, cancel_task(), changed_since(), CommandRecord, dirty_paths(), execute_tool(), finish_cancelled(), finish_done() (+39 more)

### Community 87 - "SettingsPanel.tsx"
Cohesion: 0.20
Nodes (11): Command, CommandPalette(), SettingsPanel(), THEMES, useDialogFocus(), applyTheme(), fonts, motion (+3 more)

### Community 88 - "workbenchStore.ts"
Cohesion: 0.40
Nodes (5): WorkspaceInfo, BottomPanel, OpenTab, SidePanel, WorkbenchState

### Community 89 - "types.rs"
Cohesion: 0.22
Nodes (8): ModelDefinition, ProviderAuthMode, ProviderManifest, Value, StreamEvent, ToolDefinition, Usage, read_file_tool()

### Community 90 - "ChatPanel.tsx"
Cohesion: 0.27
Nodes (8): ChatPanel(), ModelPicker(), ProvidersSection(), useProviderModel(), ChatMessage, ModelDefinition, providerCommands, ProviderStatus

### Community 91 - ".push"
Cohesion: 0.29
Nodes (10): handles_multiple_events_in_one_chunk(), parses_a_single_data_only_event(), parses_named_events(), reassembles_an_event_split_across_two_chunks(), Option, Self, String, Vec (+2 more)

### Community 92 - "Message"
Cohesion: 0.31
Nodes (11): role_str(), role_str(), Message, ModelRequest, RequestMetadata, Role, Into, Option (+3 more)

### Community 93 - "useWorkbenchStore"
Cohesion: 0.15
Nodes (8): FileRow(), GitPanel(), STATUS_COLOR, STATUS_LABEL, StatusBar(), FsEntry, GitFileStatus, useWorkbenchStore

### Community 94 - "entry"
Cohesion: 0.42
Nodes (9): delete_api_key(), entry(), get_api_key(), Error, Option, Result, String, SecretError (+1 more)

### Community 95 - "ProviderError"
Cohesion: 0.13
Nodes (17): build_chat_request(), OllamaProvider, parse_line(), parses_a_content_line(), parses_the_final_usage_line(), Client, ModelDefinition, ModelStream (+9 more)

### Community 96 - "OpenAiProvider"
Cohesion: 0.20
Nodes (8): OpenAiProvider, Client, ModelDefinition, ModelStream, Result, ModelProvider, Send, Sync

### Community 97 - "openai.rs"
Cohesion: 0.16
Nodes (18): accumulates_argument_fragments_across_chunks(), build_chat_request(), builds_a_streaming_chat_request(), extract_tool_call_fragments(), extracts_tool_call_fragments_by_index(), finish_reason(), includes_tool_definitions_when_present(), message_to_json() (+10 more)

### Community 98 - "anycode-security/src/lib.rs"
Cohesion: 0.15
Nodes (7): capability_risk(), classify_shell_command(), decide(), Decision, RiskLevel, StandingGrant, RiskLevel

### Community 99 - "live_openai.rs"
Cohesion: 0.67
Nodes (5): a_real_response_produces_a_tool_call(), a_real_response_reports_usage(), api_key(), model(), String

### Community 100 - "Tool"
Cohesion: 0.40
Nodes (4): Option, Send, Sync, Tool

### Community 103 - "ToolError"
Cohesion: 0.14
Nodes (13): context(), FilesystemReadTool, FilesystemWriteTool, read_rejects_missing_path_argument(), Result, RiskLevel, TempDir, Value (+5 more)

### Community 104 - "ToolContext"
Cohesion: 0.20
Nodes (8): ToolContext, a_failing_command_is_a_result_not_an_error(), captures_stdout_and_exit_code(), context(), Result, TempDir, Value, ShellExecuteTool

### Community 105 - "GitStatusTool"
Cohesion: 0.22
Nodes (7): GitStatusTool, reports_an_untracked_file(), Path, Result, RiskLevel, Value, run()

### Community 107 - "anycode-tools/src/lib.rs"
Cohesion: 0.13
Nodes (14): every_tool_produces_a_non_empty_spec(), Box, Drop, Path, PathBuf, Self, Value, Vec (+6 more)

## Knowledge Gaps
- **480 isolated node(s):** `Current quality status`, `Review protocol`, `2026-09-23 — Integrated terminal defect hunt`, `2026-08-23T00:41:22Z — Brand integration baseline`, `2026-08-23T00:45:09Z — GitHub CI after branding and governance push` (+475 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **25 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `Store`, `anycode-tools/src/lib.rs`, `agent_commands.rs`, `PtySession`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Why does `ToolCallRequest` connect `agent_commands.rs` to `types.rs`, `Message`?**
  _High betweenness centrality (0.033) - this node is a cross-community bridge._
- **Why does `run_gated_tool()` connect `agent_commands.rs` to `anycode-security/src/lib.rs`, `WorkspaceRoot`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **What connects `Current quality status`, `Review protocol`, `2026-09-23 — Integrated terminal defect hunt` to the rest of the system?**
  _480 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `PRD.md` be split into smaller, more focused modules?**
  _Cohesion score 0.021052631578947368 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `What You Must Do When Invoked` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._