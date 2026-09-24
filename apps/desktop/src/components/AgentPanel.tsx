import { useQueryClient } from "@tanstack/react-query";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useProviderModel } from "../hooks/useProviderModel";
import {
  agentCommands,
  type ApprovalResponse,
  type ContextPackage,
  type TaskApprovalRequest,
  type TaskDone,
  type TaskPlan,
  type TaskState,
  type TaskToolCall,
  type TaskToolResult,
} from "../lib/tauri";
import {
  appendPlanText,
  appendText,
  argumentSummary,
  completePlan,
  resultSummary,
  type Entry,
} from "../lib/timeline";
import { useWorkbenchStore } from "../state/workbenchStore";
import ApprovalDialog from "./ApprovalDialog";
import { Icon } from "./Icons";
import ModelPicker from "./ModelPicker";
import TaskHistoryPanel from "./TaskHistoryPanel";
import { TimelineEntry } from "./TimelineEntry";

/** What the task is doing right now, in words — never just a colour or a spinner. */
const STATE_LABEL: Record<TaskState, string> = {
  created: "Starting…",
  planning: "Planning…",
  running: "Working…",
  awaiting_approval: "Waiting for your approval",
  verifying: "Checking the evidence…",
  completed: "Completed",
  failed: "Failed",
  cancelled: "Stopped",
};

/**
 * The Agent Dock (PRD §55): an engineering task interface, not a chat sidebar. It shows
 * what the agent asked to do, what the runtime allowed, and — when the task finishes —
 * the evidence the runtime actually measured.
 */
export default function AgentPanel() {
  const picker = useProviderModel();
  const workspace = useWorkbenchStore((s) => s.workspace);
  const [entries, setEntries] = useState<Entry[]>([]);
  const [draft, setDraft] = useState("");
  const [taskId, setTaskId] = useState<string | null>(null);
  const [startError, setStartError] = useState<string | null>(null);
  const [approval, setApproval] = useState<TaskApprovalRequest | null>(null);
  const [taskState, setTaskState] = useState<TaskState | null>(null);
  const [historyOpen, setHistoryOpen] = useState(false);
  // Bumped whenever a task finishes, and passed as TaskHistoryPanel's `key` — remounting
  // it forces a fresh fetch, so a task that just finished shows up in its own list.
  const [historyVersion, setHistoryVersion] = useState(0);
  const sessionId = useMemo(() => crypto.randomUUID(), []);
  const scrollRef = useRef<HTMLDivElement>(null);
  const unlistenRef = useRef<UnlistenFn[]>([]);
  const queryClient = useQueryClient();

  const detach = useCallback(() => {
    // Detaching must be total: one listener failing to unregister must not strand the
    // rest, or a finished task keeps receiving events into a dead component.
    for (const off of unlistenRef.current) {
      try {
        off();
      } catch {
        /* already gone */
      }
    }
    unlistenRef.current = [];
  }, []);

  useEffect(() => detach, [detach]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [entries]);

  const finish = useCallback(
    (entry: Entry) => {
      setEntries((current) => [...current, entry]);
      setApproval(null);
      setTaskId(null);
      detach();
      setHistoryVersion((v) => v + 1);
      // The agent may have created or edited files. Drop the cached directory listings
      // and git status so the Explorer and Source Control show what's actually there.
      queryClient.invalidateQueries();
    },
    [detach, queryClient],
  );

  const run = useCallback(async () => {
    const instruction = draft.trim();
    if (!instruction || taskId || !picker.provider || !picker.model) return;

    setStartError(null);
    setEntries((current) => [...current, { kind: "instruction", text: instruction }]);
    setDraft("");

    const id = crypto.randomUUID();

    // Subscribe before starting: a task that fails immediately must not emit its
    // terminal event into a channel nobody is listening on yet.
    const offs = await Promise.all([
      listen<{ state: TaskState }>(`task:state:${id}`, (e) => setTaskState(e.payload.state)),
      // Not emitted at all when the index isn't ready, so this entry is genuinely
      // optional — never a placeholder standing in for context that wasn't built.
      listen<ContextPackage>(`task:context:${id}`, (e) =>
        setEntries((current) => [...current, { kind: "context", package: e.payload }]),
      ),
      listen<{ text: string }>(`task:plan_delta:${id}`, (e) => setEntries((c) => appendPlanText(c, e.payload.text))),
      listen<TaskPlan>(`task:plan:${id}`, (e) => setEntries((c) => completePlan(c, e.payload))),
      listen<{ reason: string }>(`task:replan:${id}`, (e) =>
        setEntries((current) => [...current, { kind: "replan", reason: e.payload.reason }]),
      ),
      listen<{ text: string }>(`task:delta:${id}`, (e) => setEntries((c) => appendText(c, e.payload.text))),
      listen<TaskToolCall>(`task:tool_call:${id}`, (e) =>
        setEntries((current) => [
          ...current,
          {
            kind: "tool_call",
            id: e.payload.id,
            name: e.payload.name,
            risk: e.payload.risk,
            summary: argumentSummary(e.payload.arguments),
            reason: e.payload.reason,
          },
        ]),
      ),
      listen<TaskToolResult>(`task:tool_result:${id}`, (e) => {
        const { ok, detail } = resultSummary(e.payload.result);
        setEntries((current) => [
          ...current,
          { kind: "tool_result", id: e.payload.id, name: e.payload.name, ok, detail },
        ]);
      }),
      listen<TaskApprovalRequest>(`task:approval_requested:${id}`, (e) => setApproval(e.payload)),
      listen<TaskDone>(`task:done:${id}`, (e) =>
        finish({
          kind: "done",
          evidence: e.payload.evidence,
          verdict: e.payload.verdict,
          usage: e.payload.usage,
        }),
      ),
      listen<{ message: string }>(`task:error:${id}`, (e) =>
        finish({ kind: "error", message: e.payload.message }),
      ),
      listen(`task:cancelled:${id}`, () => finish({ kind: "cancelled" })),
    ]);
    unlistenRef.current = offs;
    setTaskState("created");
    setTaskId(id);

    try {
      await agentCommands.runTask(id, picker.provider, picker.model, sessionId, instruction);
    } catch (error) {
      // The task never started, so no terminal event is coming — clean up here.
      setStartError(String(error));
      setTaskId(null);
      detach();
    }
  }, [draft, taskId, picker.provider, picker.model, sessionId, finish, detach]);

  const respond = useCallback(
    (response: ApprovalResponse) => {
      if (!approval) return;
      const id = approval.id;
      setApproval(null);
      agentCommands
        .respondToApproval(id, response)
        .catch((error) => setStartError(`Could not deliver the decision: ${String(error)}`));
    },
    [approval],
  );

  const stop = useCallback(() => {
    if (!taskId) return;
    agentCommands.cancelTask(taskId).catch((error) => setStartError(String(error)));
  }, [taskId]);

  // Past tasks are only reachable between runs — there's nowhere to show them while the
  // live timeline above is doing something.
  if (historyOpen) {
    return <TaskHistoryPanel key={historyVersion} onClose={() => setHistoryOpen(false)} />;
  }

  if (picker.providersError) {
    return (
      <div className="empty-state">
        <div>
          <strong>Could not load providers</strong>
          {picker.providersError}
        </div>
      </div>
    );
  }
  if (!picker.providers) return <div className="empty-state muted">Loading providers…</div>;
  if (picker.connected.length === 0) {
    return (
      <div className="empty-state">
        <div>
          <strong>No provider connected</strong>
          Add an API key or an endpoint in Settings → Providers, or run Ollama locally.
        </div>
      </div>
    );
  }

  return (
    <div className="panel">
      <ModelPicker
        connected={picker.connected}
        provider={picker.provider}
        onProviderChange={picker.setProvider}
        models={picker.models}
        modelsError={picker.modelsError}
        model={picker.model}
        onModelChange={picker.setModel}
        disabled={!!taskId}
      />

      {/* Past tasks make no sense to open mid-run: nothing here would be live. */}
      {!taskId && (
        <div className="agent-toolbar">
          <button className="button" onClick={() => setHistoryOpen(true)}>
            <Icon name="history" size={14} />
            Past tasks
          </button>
        </div>
      )}

      <div className="panel-scroll agent-timeline" ref={scrollRef}>
        {entries.length === 0 && (
          <p className="muted">
            Describe a change to make in this repository. The agent reads and edits files, runs
            commands, and must verify its own work before reporting done.
          </p>
        )}
        {entries.map((entry, i) => (
          <TimelineEntry key={i} entry={entry} />
        ))}
        {taskId && taskState && (
          <p className="muted agent-working" role="status">
            {STATE_LABEL[taskState]}
          </p>
        )}
      </div>

      {startError && (
        <p className="danger chat-error" role="alert">
          {startError}
        </p>
      )}

      <form
        className="chat-input"
        onSubmit={(e) => {
          e.preventDefault();
          run();
        }}
      >
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder={taskId ? "Task running…" : "Describe the change to make…"}
          disabled={!!taskId || !picker.model}
          aria-label="Task instruction"
        />
        {taskId ? (
          <button type="button" className="button" onClick={stop}>
            Stop
          </button>
        ) : (
          <button
            type="submit"
            className="icon-button"
            disabled={!draft.trim() || !picker.model}
            aria-label="Run task"
          >
            <Icon name="send" />
          </button>
        )}
      </form>

      {approval && (
        <ApprovalDialog
          request={approval}
          workspaceName={workspace?.name ?? "this workspace"}
          onRespond={respond}
        />
      )}
    </div>
  );
}

