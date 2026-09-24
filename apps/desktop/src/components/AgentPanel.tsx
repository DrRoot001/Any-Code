import { useQueryClient } from "@tanstack/react-query";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useProviderModel } from "../hooks/useProviderModel";
import {
  agentCommands,
  type ApprovalResponse,
  type TaskApprovalRequest,
  type TaskDone,
  type TaskEvidence,
  type TaskPlan,
  type TaskState,
  type TaskToolCall,
  type TaskToolResult,
  type TaskUsage,
  type TaskVerdict,
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

function TimelineEntry({ entry }: { entry: Entry }) {
  switch (entry.kind) {
    case "instruction":
      return (
        <div className="agent-entry agent-entry--instruction">
          <span className="chat-role">you</span>
          <p>{entry.text}</p>
        </div>
      );
    case "plan":
      return (
        <div className="agent-entry agent-plan">
          <h4>Plan</h4>
          {entry.steps && entry.steps.length > 0 ? (
            <ol>
              {entry.steps.map((step, i) => (
                <li key={i}>{step}</li>
              ))}
            </ol>
          ) : (
            <p>{entry.text}</p>
          )}
        </div>
      );
    case "replan":
      return (
        <div className="agent-entry agent-replan" role="status">
          <span className="chat-role">runtime</span>
          <p>{entry.reason}</p>
        </div>
      );
    case "text":
      return (
        <div className="agent-entry">
          <p>{entry.text}</p>
        </div>
      );
    case "tool_call":
      return (
        <div className="agent-entry agent-step">
          <span className={`risk risk--${entry.risk}`}>{entry.risk}</span>
          <code>{entry.name}</code>
          {entry.summary && <span className="agent-step-detail">{entry.summary}</span>}
          {entry.reason && <span className="agent-step-reason">{entry.reason}</span>}
        </div>
      );
    case "tool_result":
      return (
        <div className={`agent-entry agent-step ${entry.ok ? "" : "danger"}`}>
          <span aria-hidden="true">{entry.ok ? "✓" : "✕"}</span>
          <span className="visually-hidden">{entry.ok ? "succeeded" : "failed"}</span>
          <code>{entry.name}</code>
          {entry.detail && <span className="agent-step-detail">{entry.detail}</span>}
        </div>
      );
    case "done":
      // The final message already streamed in as text entries; rendering `entry.text`
      // again here would show it twice.
      return (
        <div className="agent-entry">
          <Evidence evidence={entry.evidence} verdict={entry.verdict} usage={entry.usage} />
        </div>
      );
    case "error":
      return (
        <p className="danger" role="alert">
          {entry.message}
        </p>
      );
    case "cancelled":
      return <p className="muted">Task stopped.</p>;
    case "interrupted":
      return <p className="muted">Did not finish — the app closed while this task was running.</p>;
  }
}

/**
 * What the runtime measured. The verdict is the runtime's (anycode_agent::verdict),
 * judged from each check's latest exit code — displayed here, never recomputed, so the
 * UI cannot disagree with the audit log. Nothing verified is said plainly (PRD §8.6).
 */
function Evidence({
  evidence,
  verdict,
  usage,
}: {
  evidence: TaskEvidence;
  verdict: TaskVerdict;
  usage: TaskUsage;
}) {
  const { filesChanged, commands } = evidence;

  return (
    <section className="evidence" aria-label="Evidence">
      <h4>Evidence</h4>

      {verdict.kind === "unverified" ? (
        <p className="evidence-verdict evidence-verdict--unverified">
          Not verified — the agent ran no checks.
        </p>
      ) : verdict.kind === "failed" ? (
        <p className="evidence-verdict evidence-verdict--failed" role="alert">
          Verification failed —{" "}
          {verdict.failing
            .map((c) => `${c.command} ${c.exitCode === null ? "was killed" : `exited ${c.exitCode}`}`)
            .join("; ")}
          .
        </p>
      ) : (
        <p className="evidence-verdict evidence-verdict--passed">
          Verified — {verdict.checks.length} check{verdict.checks.length === 1 ? "" : "s"} passed
          on {verdict.checks.length === 1 ? "its" : "their"} latest run.
        </p>
      )}

      {commands.length > 0 && (
        <ul className="evidence-list">
          {commands.map((c, i) => (
            <li key={i} className={c.exitCode === 0 ? "" : c.verification ? "danger" : "muted"}>
              {c.verification && <span className="evidence-tag">check</span>}
              <code>{c.command}</code>
              <span className="muted"> exit {c.exitCode ?? "—"}</span>
            </li>
          ))}
        </ul>
      )}

      <h5>
        {filesChanged.length === 0
          ? "No files changed"
          : `${filesChanged.length} file${filesChanged.length === 1 ? "" : "s"} changed`}
      </h5>
      {filesChanged.length > 0 && (
        <ul className="evidence-list">
          {filesChanged.map((path) => (
            <li key={path}>
              <code>{path}</code>
            </li>
          ))}
        </ul>
      )}

      <p className="muted evidence-usage">
        {usage.inputTokens.toLocaleString()} in · {usage.outputTokens.toLocaleString()} out tokens
      </p>
    </section>
  );
}
