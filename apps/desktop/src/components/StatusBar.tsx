import { useQuery } from "@tanstack/react-query";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { indexStatusLabel, indexStatusTooltip } from "../lib/indexStatus";
import { commands, indexCommands, type IndexStatus } from "../lib/tauri";
import { useWorkbenchStore } from "../state/workbenchStore";
import { Icon } from "./Icons";

/**
 * The workspace index's state (Phase 4). Subscribed before the initial fetch resolves,
 * and a flag guards against that fetch overwriting a newer state the event already
 * delivered — the same ordering hazard as the task event channels, just without a
 * caller-chosen id to key on.
 */
function useIndexStatus(): IndexStatus | null {
  const [status, setStatus] = useState<IndexStatus | null>(null);

  useEffect(() => {
    let cancelled = false;
    let receivedEvent = false;
    let unlisten: UnlistenFn | undefined;

    listen<IndexStatus>("index:status", (e) => {
      receivedEvent = true;
      if (!cancelled) setStatus(e.payload);
    }).then((off) => {
      if (cancelled) off();
      else unlisten = off;
    });

    indexCommands
      .status()
      .then((current) => {
        if (!cancelled && !receivedEvent) setStatus(current);
      })
      .catch(() => {
        /* No status yet — the badge simply stays hidden. */
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return status;
}

export default function StatusBar({ onOpenSettings }: { onOpenSettings: () => void }) {
  const workspace = useWorkbenchStore((s) => s.workspace);
  const bottomPanelOpen = useWorkbenchStore((s) => s.bottomPanelOpen);
  const toggleBottomPanel = useWorkbenchStore((s) => s.toggleBottomPanel);
  const { data: branch } = useQuery({
    queryKey: ["git-branch"],
    queryFn: commands.gitBranch,
    enabled: !!workspace,
  });
  const indexStatus = useIndexStatus();
  const indexLabel = indexStatus ? indexStatusLabel(indexStatus) : null;
  const indexTooltip = indexStatus ? indexStatusTooltip(indexStatus) : null;

  return (
    <footer className="statusbar">
      <div className="status-group">
        <span>{workspace?.name ?? "No workspace"}</span>
        {branch && (
          <span className="status-button">
            <Icon name="branch" size={13} />
            {branch}
          </span>
        )}
        {indexLabel && (
          <span
            className={`status-button ${indexStatus?.state === "failed" ? "danger" : ""}`}
            title={indexTooltip ?? undefined}
            aria-label={indexTooltip ? `${indexLabel}. ${indexTooltip}` : indexLabel}
          >
            <Icon name="files" size={13} />
            {indexLabel}
          </span>
        )}
      </div>
      <div className="status-group">
        <button
          onClick={toggleBottomPanel}
          className="status-button"
          aria-pressed={bottomPanelOpen}
        >
          <Icon name="terminal" size={13} />
          {bottomPanelOpen ? "Hide Terminal" : "Terminal"}
        </button>
        <button onClick={onOpenSettings} className="status-button">
          <Icon name="settings" size={13} />
          Settings
        </button>
      </div>
    </footer>
  );
}
