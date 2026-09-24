import { useEffect, useState } from "react";
import { eventsToEntries } from "../lib/history";
import { formatStartedAt, outcomeClass, outcomeLabel } from "../lib/taskHistory";
import { historyCommands, type TaskSummary } from "../lib/tauri";
import type { Entry } from "../lib/timeline";
import { Icon } from "./Icons";
import { TimelineEntry } from "./TimelineEntry";

/**
 * Past tasks for this workspace (ADR 0004 "Session resume"): the audit log is the only
 * source, rebuilt into the same timeline the live Agent Dock shows, but read only.
 * Reachable only between runs (AgentPanel hides the button while a task is running).
 */
export default function TaskHistoryPanel({ onClose }: { onClose: () => void }) {
  const [tasks, setTasks] = useState<TaskSummary[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [selected, setSelected] = useState<TaskSummary | null>(null);
  const [entries, setEntries] = useState<Entry[] | null>(null);
  const [entriesError, setEntriesError] = useState<string | null>(null);

  useEffect(() => {
    historyCommands
      .listTasks()
      .then(setTasks)
      .catch((reason) => setListError(String(reason)));
  }, []);

  const open = (task: TaskSummary) => {
    setSelected(task);
    setEntries(null);
    setEntriesError(null);
    historyCommands
      .taskEvents(task.taskId)
      .then((events) => setEntries(eventsToEntries(events)))
      .catch((reason) => setEntriesError(String(reason)));
  };

  return (
    <div className="panel">
      <div className="panel-header">
        <span>{selected ? "Past task" : "Past tasks"}</span>
        <button className="icon-button" onClick={onClose} aria-label="Back to the agent">
          <Icon name="close" size={14} />
        </button>
      </div>

      {selected ? (
        <>
          <div className="history-banner" role="status">
            <span>Read only — replaying a past task, not a running one.</span>
            <button className="button" onClick={() => setSelected(null)}>
              Back to list
            </button>
          </div>
          <div className="panel-scroll agent-timeline">
            {entriesError && (
              <p className="danger" role="alert">
                {entriesError}
              </p>
            )}
            {!entriesError && entries === null && <p className="muted">Loading…</p>}
            {!entriesError &&
              entries?.map((entry, i) => <TimelineEntry key={i} entry={entry} />)}
          </div>
        </>
      ) : (
        <div className="panel-scroll">
          {listError && (
            <p className="danger" role="alert">
              {listError}
            </p>
          )}
          {!listError && tasks === null && <p className="muted">Loading past tasks…</p>}
          {!listError && tasks?.length === 0 && (
            <p className="muted">No past tasks in this workspace yet.</p>
          )}
          {!listError && tasks && tasks.length > 0 && (
            <ul className="history-list">
              {tasks.map((task) => (
                <li key={task.taskId}>
                  <button className="history-row" onClick={() => open(task)}>
                    <span className="history-row-instruction">{task.instruction}</span>
                    <span className="history-row-meta">
                      <span>
                        {task.provider} / {task.model}
                      </span>
                      <span>{formatStartedAt(task.startedAt)}</span>
                      <span className={`history-outcome history-outcome--${outcomeClass(task.outcome)}`}>
                        {outcomeLabel(task.outcome)}
                      </span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
