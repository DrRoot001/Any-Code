import { useDialogFocus } from "../hooks/useDialogFocus";
import { grantLabel, requestSummary } from "../lib/approval";
import type { ApprovalResponse, RiskLevel, TaskApprovalRequest } from "../lib/tauri";

const RISK_EXPLANATION: Record<RiskLevel, string> = {
  low: "Read-only — cannot change anything.",
  medium: "Can modify files in this workspace.",
  high: "High impact — asked every time, never covered by “always allow”.",
  critical: "Denied by policy — this cannot be approved.",
};

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
              {request.reason && <p className="approval-reason">{request.reason}</p>}
            </dd>
            <dt>Workspace</dt>
            <dd>{workspaceName}</dd>
          </dl>
        </div>

        <div className="approval-actions">
          <button className="button" onClick={() => onRespond("deny")}>
            Deny
          </button>
          {request.grantable && (
            <button className="button" onClick={() => onRespond("allow_workspace")}>
              {grantLabel(request)}
            </button>
          )}
          <button className="button button--primary" onClick={() => onRespond("allow_once")}>
            Allow once
          </button>
        </div>
      </div>
    </div>
  );
}
