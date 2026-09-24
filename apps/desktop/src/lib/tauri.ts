import { invoke } from "@tauri-apps/api/core";

/** Typed wrappers around every Rust command — the one place their names and shapes live. */

export interface WorkspaceInfo {
  path: string;
  name: string;
}

export interface FsEntry {
  name: string;
  path: string;
  isDir: boolean;
}

export type GitFileStatus =
  "modified" | "added" | "deleted" | "renamed" | "untracked" | "conflicted";

export interface GitStatusEntry {
  path: string;
  status: GitFileStatus;
}

export interface FileDiff {
  headContent: string | null;
  workingContent: string | null;
}

export const commands = {
  getTheme: () => invoke<string>("get_theme"),
  setTheme: (theme: string) => invoke<void>("set_theme", { theme }),

  getLastWorkspace: () => invoke<WorkspaceInfo | null>("get_last_workspace"),
  openWorkspace: (path: string) => invoke<WorkspaceInfo>("open_workspace", { path }),

  listDir: (relative: string) => invoke<FsEntry[]>("list_dir", { relative }),
  readFile: (relative: string) => invoke<string>("read_file", { relative }),
  writeFile: (relative: string, contents: string) =>
    invoke<void>("write_file", { relative, contents }),

  gitStatus: () => invoke<GitStatusEntry[]>("git_status"),
  gitDiff: (relative: string) => invoke<FileDiff>("git_diff", { relative }),
  gitBranch: () => invoke<string | null>("git_branch"),

  terminalSpawn: (id: string, cols: number, rows: number) =>
    invoke<void>("terminal_spawn", { id, cols, rows }),
  terminalWrite: (id: string, data: string) => invoke<void>("terminal_write", { id, data }),
  terminalResize: (id: string, cols: number, rows: number) =>
    invoke<void>("terminal_resize", { id, cols, rows }),
  terminalKill: (id: string) => invoke<void>("terminal_kill", { id }),
};

export interface ProviderStatus {
  id: string;
  name: string;
  requiresKey: boolean;
  hasKey: boolean;
  /** Configured by base URL (an OpenAI-compatible endpoint); a key is optional. */
  needsEndpoint: boolean;
  endpoint: string | null;
  /** Everything it needs is configured. Not a claim that it is reachable. */
  ready: boolean;
}

export interface ModelDefinition {
  id: string;
  displayName: string;
}

export type ChatRole = "system" | "user" | "assistant";

export interface ChatMessage {
  role: ChatRole;
  content: string;
}

export const providerCommands = {
  listProviders: () => invoke<ProviderStatus[]>("list_providers"),
  setProviderKey: (provider: string, key: string) =>
    invoke<void>("set_provider_key", { provider, key }),
  removeProviderKey: (provider: string) => invoke<void>("remove_provider_key", { provider }),
  /** `null` clears it. Validated in Rust: https anywhere, plain http only to localhost. */
  setProviderEndpoint: (provider: string, baseUrl: string | null) =>
    invoke<void>("set_provider_endpoint", { provider, baseUrl }),
  listModels: (provider: string) => invoke<ModelDefinition[]>("list_models", { provider }),
  /** `requestId` is the caller's, so it can subscribe before the stream starts. */
  sendChat: (
    requestId: string,
    provider: string,
    model: string,
    sessionId: string,
    messages: ChatMessage[],
  ) => invoke<void>("send_chat", { requestId, provider, model, sessionId, messages }),
};

export type RiskLevel = "low" | "medium" | "high" | "critical";

/** Payload of `task:tool_call:{taskId}`. */
export interface TaskToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  /** `rejected`: refused before the permission gate (unknown tool, missing argument). */
  risk: RiskLevel | "rejected";
  /** Why this call is riskier than the tool usually is, e.g. a secret-bearing path. */
  reason: string | null;
}

/** Payload of `task:tool_result:{taskId}`. */
export interface TaskToolResult {
  id: string;
  name: string;
  result: Record<string, unknown>;
}

/** Payload of `task:approval_requested:{taskId}`. */
export interface TaskApprovalRequest {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  risk: RiskLevel;
  reason: string | null;
  /** Whether "always allow" may be offered. The runtime enforces this regardless. */
  grantable: boolean;
  /** What an "always allow" covers, e.g. `shell.execute:npm test`. */
  grantScope: string;
}

/** One command the agent actually ran, with the exit code the process returned. */
export interface CommandRecord {
  command: string;
  exitCode: number | null;
  /** The agent ran it to check its work (build/lint/test), not to look around. */
  verification: boolean;
}

/**
 * The runtime's judgement of whether the work passed, from each check's latest run
 * (anycode_agent::verdict). Authoritative — the UI displays it, never recomputes it.
 */
export type TaskVerdict =
  | { kind: "passed"; checks: string[] }
  | { kind: "failed"; failing: CommandRecord[] }
  | { kind: "unverified" };

/** Mirrors anycode_agent::TaskState. */
export type TaskState =
  | "created"
  | "planning"
  | "running"
  | "awaiting_approval"
  | "verifying"
  | "completed"
  | "failed"
  | "cancelled";

/** Payload of `task:plan:{taskId}`. `steps` is empty when the reply held no list. */
export interface TaskPlan {
  steps: string[];
  text: string;
}

/**
 * What the runtime measured during a task — not what the model claimed. `filesChanged`
 * is a real `git status` delta; `commands` are real exit codes.
 */
export interface TaskEvidence {
  filesChanged: string[];
  commands: CommandRecord[];
}

/** Tokens a task consumed. Reported in tokens, not currency — see TaskUsage in Rust. */
export interface TaskUsage {
  inputTokens: number;
  outputTokens: number;
}

/** Payload of `task:done:{taskId}`. */
export interface TaskDone {
  text: string;
  evidence: TaskEvidence;
  verdict: TaskVerdict;
  usage: TaskUsage;
}

export type ApprovalResponse = "allow_once" | "allow_workspace" | "deny";

export const agentCommands = {
  /**
   * `taskId` is supplied by the caller so it can subscribe to this task's event
   * channels before the task starts — see run_task's docs in Rust.
   */
  runTask: (
    taskId: string,
    provider: string,
    model: string,
    sessionId: string,
    instruction: string,
  ) => invoke<string>("run_task", { taskId, provider, model, sessionId, instruction }),
  respondToApproval: (id: string, response: ApprovalResponse) =>
    invoke<void>("respond_to_approval", { id, response }),
  cancelTask: (taskId: string) => invoke<void>("cancel_task", { taskId }),
};

export type MemoryScope = "global" | "workspace";

/** A memory is the user's own words (PRD §38); `source` is set when it was adopted from a file. */
export interface Memory {
  id: string;
  scope: MemoryScope;
  workspace: string | null;
  content: string;
  source: string | null;
  createdAt: string;
  updatedAt: string;
}

/** An instruction file found in the repository. Untrusted until the user adopts it. */
export interface InstructionFile {
  path: string;
  bytes: number;
  adopted: boolean;
}

/** A past task, from the audit log. `outcome` is null when it never finished. */
export interface TaskSummary {
  taskId: string;
  sessionId: string;
  instruction: string;
  provider: string;
  model: string;
  workspace: string;
  startedAt: string;
  outcome: "completed" | "failed" | "cancelled" | null;
}

export const memoryCommands = {
  listMemories: () => invoke<Memory[]>("list_memories"),
  addMemory: (scope: MemoryScope, content: string) =>
    invoke<Memory>("add_memory", { scope, content }),
  updateMemory: (id: string, content: string) => invoke<void>("update_memory", { id, content }),
  deleteMemory: (id: string) => invoke<void>("delete_memory", { id }),
  /** `path` comes from the native save dialog. */
  exportMemories: (path: string) => invoke<void>("export_memories", { path }),
  repositoryInstructions: () => invoke<InstructionFile[]>("repository_instructions"),
  /** The text adopting `path` would add — shown to the user before they adopt it. */
  previewInstruction: (path: string) => invoke<string>("preview_instruction", { path }),
  /** Adopts exactly `content`, the text the user was shown; refused if the file changed since. */
  adoptInstruction: (path: string, content: string) =>
    invoke<Memory>("adopt_instruction", { path, content }),
};

export const historyCommands = {
  listTasks: () => invoke<TaskSummary[]>("list_tasks"),
  /** Raw audit events; turn them into a timeline with `eventsToEntries` (lib/history.ts). */
  taskEvents: (taskId: string) =>
    invoke<import("./history").AuditEvent[]>("task_events", { taskId }),
};

/** Why a passage is in a context package (anycode_context::Reason). */
export type ContextReason =
  | { kind: "named_in_instruction" }
  | { kind: "defines_symbol"; symbol: string }
  | { kind: "matches_terms"; terms: string[] }
  | { kind: "imported_by"; path: string }
  | { kind: "changed_in_working_tree" };

export interface ContextItem {
  path: string;
  startLine: number;
  endLine: number;
  /** For an outline item, the file's symbol list rather than its text. */
  text: string;
  outline: boolean;
  reasons: ContextReason[];
  score: number;
  /** Estimated (bytes ÷ 4), never measured. */
  estTokens: number;
}

export interface ContextExcluded {
  path: string;
  startLine: number;
  endLine: number;
  reasons: ContextReason[];
  estTokens: number;
  why: string;
}

/** Payload of `task:context:{taskId}` — what the agent was given, and what it was not. */
export interface ContextPackage {
  intent: { paths: string[]; identifiers: string[]; terms: string[] };
  items: ContextItem[];
  excluded: ContextExcluded[];
  estTokens: number;
  budgetTokens: number;
  repoEstTokens: number;
  repoFiles: number;
}

/**
 * The workspace index (Phase 4). Returned by `index_status` and pushed as `index:status`
 * whenever it changes. `ready` counts are what the index holds, not estimates.
 */
export type IndexStatus =
  | { state: "none" }
  | { state: "building" }
  | { state: "ready"; files: number; chunks: number; symbols: number; lastRefreshMs: number }
  | { state: "failed"; error: string };

export const indexCommands = {
  status: () => invoke<IndexStatus>("index_status"),
};
