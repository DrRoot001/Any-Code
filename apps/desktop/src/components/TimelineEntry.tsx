import type { TaskEvidence, TaskUsage, TaskVerdict } from "../lib/tauri";
import type { Entry } from "../lib/timeline";

/**
 * Renders one timeline entry — shared by the live Agent Dock (AgentPanel) and the
 * read-only replay of a past task (TaskHistoryPanel), so a finished task looks exactly
 * as it did while it ran.
 */
export function TimelineEntry({ entry }: { entry: Entry }) {
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
