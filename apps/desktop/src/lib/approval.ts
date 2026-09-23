/** What the approval dialog shows, kept free of React so it can be tested (approval.test.ts). */
import type { TaskApprovalRequest } from "./tauri";

/** What the "always allow" button would authorise, in words the user can check. */
export function grantLabel(request: TaskApprovalRequest): string {
  const [capability, ...rest] = request.grantScope.split(":");
  const command = rest.join(":");
  return command
    ? `Always allow “${command}” here`
    : `Always allow ${capability} in this workspace`;
}

const prefixLines = (text: string, prefix: string) =>
  text
    .split("\n")
    .map((line) => prefix + line)
    .join("\n");

/**
 * The exact thing being requested, as text the user can actually evaluate. For a file
 * change that means the change itself — approving a path without seeing what will be
 * written into it is not an informed decision.
 */
export function requestSummary(request: TaskApprovalRequest): string {
  const args = request.arguments ?? {};
  if (typeof args.command === "string") return args.command;
  if (typeof args.path === "string") {
    if (typeof args.old_text === "string" && typeof args.new_text === "string") {
      return `${args.path}\n\n${prefixLines(args.old_text, "- ")}\n${prefixLines(args.new_text, "+ ")}`;
    }
    if (typeof args.content === "string") {
      return `${args.path} — replaces the whole file with:\n\n${args.content}`;
    }
    return args.path;
  }
  return JSON.stringify(args, null, 2);
}
