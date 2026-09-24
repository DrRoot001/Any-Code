/**
 * Rebuilds a past task's timeline from its audit-log events (ADR 0004 "Session resume"),
 * so a restart does not lose it. Pure, so it can be tested (history.test.ts). The log is
 * the only source: nothing here fills a gap with a guess — a task that never finished is
 * shown as interrupted, and a result too large to keep says so.
 */
import { argumentSummary, resultSummary, type Entry } from "./timeline";
import type { TaskEvidence, TaskUsage, TaskVerdict } from "./tauri";

/** One audit-log row as the backend returns it (anycode_core::Event). */
export interface AuditEvent {
  id: string;
  timestamp: string;
  kind: string;
  payload?: Record<string, unknown>;
}

/** A value the runtime stored truncated (`for_audit`): only its opening bytes survive. */
interface Truncated {
  truncated: true;
  bytes: number;
  prefix: string;
}

const isTruncated = (value: unknown): value is Truncated =>
  typeof value === "object" && value !== null && (value as Truncated).truncated === true;

const text = (value: unknown): string =>
  typeof value === "string" ? value : isTruncated(value) ? `${value.prefix}…` : "";

/**
 * A result kept only as a prefix. Shell results serialise `exitCode` first, so whether the
 * command passed survives truncation; anything else that was cut short was a large success
 * (errors are small), and is labelled as not kept in full rather than summarised.
 */
function truncatedResult(value: Truncated): { ok: boolean; detail: string } {
  const exit = value.prefix.match(/^\{"exitCode":(-?\d+|null)/);
  if (exit) {
    const code = exit[1] === "null" ? null : Number(exit[1]);
    return { ok: code === 0, detail: `exit ${code ?? "—"} · output not kept in full` };
  }
  return { ok: true, detail: `${value.bytes.toLocaleString()} bytes · not kept in full` };
}

export function eventsToEntries(events: AuditEvent[]): Entry[] {
  const entries: Entry[] = [];
  let finished = false;

  for (const event of events) {
    const p = event.payload ?? {};
    switch (event.kind) {
      case "task.created":
        entries.push({ kind: "instruction", text: text(p.instruction) });
        break;
      case "task.plan":
        entries.push({
          kind: "plan",
          text: text(p.text),
          steps: Array.isArray(p.steps) ? (p.steps as string[]) : [],
        });
        break;
      case "task.turn":
        entries.push({ kind: "text", text: text(p.text) });
        break;
      case "task.replan":
        entries.push({ kind: "replan", reason: text(p.reason) });
        break;
      case "task.tool.call":
      case "task.tool.rejected": {
        const args = isTruncated(p.arguments)
          ? {}
          : ((p.arguments as Record<string, unknown>) ?? {});
        entries.push({
          kind: "tool_call",
          id: String(p.id ?? ""),
          name: String(p.name ?? ""),
          risk: event.kind === "task.tool.rejected" ? "rejected" : ((p.risk as never) ?? "medium"),
          summary: argumentSummary(args),
          reason: typeof p.reason === "string" ? p.reason : null,
        });
        break;
      }
      case "task.tool.result": {
        const { ok, detail } = isTruncated(p.result)
          ? truncatedResult(p.result)
          : resultSummary((p.result as Record<string, unknown>) ?? {});
        entries.push({
          kind: "tool_result",
          id: String(p.id ?? ""),
          name: String(p.name ?? ""),
          ok,
          detail,
        });
        break;
      }
      case "task.finished":
        finished = true;
        entries.push({
          kind: "done",
          evidence: p.evidence as TaskEvidence,
          verdict: p.verdict as TaskVerdict,
          usage: p.usage as TaskUsage,
        });
        break;
      case "task.error":
        finished = true;
        entries.push({ kind: "error", message: text(p.message) });
        break;
      case "task.state":
        if (p.state === "cancelled") {
          finished = true;
          entries.push({ kind: "cancelled" });
        }
        break;
      // task.approval is shown through the call it answered; other kinds are skipped, so a
      // newer build's events never break an older build's replay (ADR 0002).
    }
  }

  if (entries.length > 0 && !finished) entries.push({ kind: "interrupted" });
  return entries;
}
