import { useDialogFocus } from "../hooks/useDialogFocus";
import type { ApprovalResponse, RiskLevel, TaskApprovalRequest } from "../lib/tauri";

const RISK_EXPLANATION: Record<RiskLevel, string> = {
  low: "Read-only — cannot change anything.",
  medium: "Can modify files in this workspace.",
  high: "Affects state outside this workspace.",
  critical: "Denied by policy — this cannot be approved.",
};

/** The exact thing being requested, as text the user can actually evaluate. */
function requestSummary(request: TaskApprovalRequest): string {
  const args = request.arguments ?? {};
  if (typeof args.command === "string") return args.command;
  if (typeof args.path === "string") return args.path;
  return JSON.stringify(args, null, 2);
}

/**
 * PRD §50. Shows the exact command or path, its risk, and the workspace it applies to.
 * There is deliberately no "always allow globally" — a standing grant never reaches
 * past the workspace it was given in (docs/SECURITY.md).
 */
export default function ApprovalDialog({
  request,
  workspaceName,
  onRespond,
}: {
  request: TaskApprovalRequest;
  workspaceName: string;
  onRespond: (response: ApprovalResponse) => void;
}) {
  // Escape denies rather than dismissing: closing a permission prompt must not be
  // mistaken for granting it.
  const dialogRef = useDialogFocus<HTMLDivElement>(true, () => onRespond("deny"));

  return (
    <div className="overlay approval-overlay">
      <div
        className="approval"
        ref={dialogRef}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="approval-title"
        aria-describedby="approval-body"
      >
        <h2 id="approval-title">Approval required</h2>

        <div id="approval-body">
          <p className="muted">The agent wants to run:</p>
          <pre className="approval-subject">{requestSummary(request)}</pre>

          <dl className="approval-facts">
            <dt>Capability</dt>
            <dd>
              <code>{request.name}</code>
            </dd>
            <dt>Risk</dt>
            <dd>
              <span className={`risk risk--${request.risk}`}>{request.risk}</span>{" "}
              {RISK_EXPLANATION[request.risk]}
            </dd>
            <dt>Workspace</dt>
            <dd>{workspaceName}</dd>
          </dl>
        </div>

        <div className="approval-actions">
          <button className="button" onClick={() => onRespond("deny")}>
            Deny
          </button>
          <button className="button" onClick={() => onRespond("allow_workspace")}>
            Always allow in this workspace
          </button>
          <button className="button button--primary" onClick={() => onRespond("allow_once")}>
            Allow once
          </button>
        </div>
      </div>
    </div>
  );
}
