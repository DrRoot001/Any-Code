import { useQueryClient } from "@tanstack/react-query";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useProviderModel } from "../hooks/useProviderModel";
import {
  agentCommands,
  type ApprovalResponse,
  type RiskLevel,
  type TaskApprovalRequest,
  type TaskDone,
  type TaskEvidence,
  type TaskToolCall,
  type TaskToolResult,
  type TaskUsage,
} from "../lib/tauri";
import { useWorkbenchStore } from "../state/workbenchStore";
import ApprovalDialog from "./ApprovalDialog";
import { Icon } from "./Icons";
import ModelPicker from "./ModelPicker";

type Entry =
  | { kind: "instruction"; text: string }
  | { kind: "text"; text: string }
  | { kind: "tool_call"; id: string; name: string; risk: RiskLevel; summary: string }
  | { kind: "tool_result"; id: string; name: string; ok: boolean; detail: string }
  | { kind: "done"; text: string; evidence: TaskEvidence; usage: TaskUsage }
  | { kind: "error"; message: string }
  | { kind: "cancelled" };

function argumentSummary(args: Record<string, unknown>): string {
  if (typeof args?.command === "string") return args.command;
  if (typeof args?.path === "string") return args.path;
  const json = JSON.stringify(args ?? {});
  return json === "{}" ? "" : json;
}

/**
 * A tool result is a success only when the runtime said so — never assumed. A shell
 * result with no exit code (killed by a signal) is a failure, not an unknown we round up.
 */
function resultSummary(result: Record<string, unknown>): { ok: boolean; detail: string } {
  if (typeof result?.error === "string") return { ok: false, detail: result.error };

  const isShellResult = "exitCode" in (result ?? {});
  if (isShellResult) {
    const exitCode = typeof result.exitCode === "number" ? result.exitCode : null;
    const stderr = typeof result.stderr === "string" ? result.stderr.trim() : "";
    const stdout = typeof result.stdout === "string" ? result.stdout.trim() : "";
    const output = stderr || stdout;
    const tail = output ? ` · ${output.split("\n").slice(-3).join("\n")}` : "";
    return {
      ok: exitCode === 0,
      detail: `exit ${exitCode ?? "—"}${tail}`,
    };
  }

  if (typeof result?.content === "string") {
    return { ok: true, detail: `${result.content.length} chars` };
  }
  return { ok: true, detail: "" };
}

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

  const appendText = (text: string) =>
    setEntries((current) => {
      const last = current[current.length - 1];
      if (last?.kind === "text") {
        return [...current.slice(0, -1), { kind: "text", text: last.text + text }];
      }
      return [...current, { kind: "text", text }];
    });

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
      listen<{ text: string }>(`task:delta:${id}`, (e) => appendText(e.payload.text)),
      listen<TaskToolCall>(`task:tool_call:${id}`, (e) =>
        setEntries((current) => [
          ...current,
          {
            kind: "tool_call",
            id: e.payload.id,
            name: e.payload.name,
            risk: e.payload.risk,
            summary: argumentSummary(e.payload.arguments),
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
          text: e.payload.text,
          evidence: e.payload.evidence,
          usage: e.payload.usage,
        }),
      ),
      listen<{ message: string }>(`task:error:${id}`, (e) =>
        finish({ kind: "error", message: e.payload.message }),
      ),
      listen(`task:cancelled:${id}`, () => finish({ kind: "cancelled" })),
    ]);
    unlistenRef.current = offs;
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
          Add an API key in Settings → Providers, or run Ollama locally.
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
        {taskId && !approval && <p className="muted agent-working">Working…</p>}
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
      return (
        <div className="agent-entry">
          {entry.text && <p>{entry.text}</p>}
          <Evidence evidence={entry.evidence} usage={entry.usage} />
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
  }
}

/**
 * What the runtime measured. The verdict is computed from real exit codes — a task
 * where nothing ran is reported as unverified, never as success (PRD §8.6).
 */
function Evidence({ evidence, usage }: { evidence: TaskEvidence; usage: TaskUsage }) {
  const { filesChanged, commands } = evidence;
  const failed = commands.filter((c) => c.exitCode !== 0);

  return (
    <section className="evidence" aria-label="Evidence">
      <h4>Evidence</h4>

      {commands.length === 0 ? (
        <p className="evidence-verdict evidence-verdict--unverified">
          Nothing was verified — the agent ran no commands.
        </p>
      ) : failed.length > 0 ? (
        <p className="evidence-verdict evidence-verdict--failed" role="alert">
          Verification failed — {failed.length} of {commands.length} command
          {commands.length === 1 ? "" : "s"} exited non-zero.
        </p>
      ) : (
        <p className="evidence-verdict evidence-verdict--passed">
          {commands.length} command{commands.length === 1 ? "" : "s"} ran, all exited 0.
        </p>
      )}

      {commands.length > 0 && (
        <ul className="evidence-list">
          {commands.map((c, i) => (
            <li key={i} className={c.exitCode === 0 ? "" : "danger"}>
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
