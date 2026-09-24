/**
 * The Agent Dock's timeline logic, kept free of React so it can be tested directly
 * (timeline.test.ts). The component only wires events to these functions.
 */
import type { RiskLevel, TaskEvidence, TaskPlan, TaskUsage, TaskVerdict } from "./tauri";

export type Entry =
  | { kind: "instruction"; text: string }
  /** Streams in as `text`; `steps` arrives once the plan is complete (empty = no list). */
  | { kind: "plan"; text: string; steps: string[] | null }
  | { kind: "replan"; reason: string }
  | { kind: "text"; text: string }
  | {
      kind: "tool_call";
      id: string;
      name: string;
      risk: RiskLevel | "rejected";
      summary: string;
      reason: string | null;
    }
  | { kind: "tool_result"; id: string; name: string; ok: boolean; detail: string }
  | {
      kind: "done";
      evidence: TaskEvidence;
      verdict: TaskVerdict;
      usage: TaskUsage;
    }
  | { kind: "error"; message: string }
  | { kind: "cancelled" }
  /** A past task with no terminal event in the log: the app closed while it ran. */
  | { kind: "interrupted" };

export function argumentSummary(args: Record<string, unknown>): string {
  if (typeof args?.command === "string") return args.command;
  if (typeof args?.path === "string") return args.path;
  const json = JSON.stringify(args ?? {});
  return json === "{}" ? "" : json;
}

/**
 * A tool result is a success only when the runtime said so — never assumed. A shell
 * result with no exit code (killed by a signal) is a failure, not an unknown we round up.
 */
export function resultSummary(result: Record<string, unknown>): { ok: boolean; detail: string } {
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

/** Streamed model text joins the text entry it continues, or starts a new one. */
export function appendText(entries: Entry[], text: string): Entry[] {
  const last = entries[entries.length - 1];
  if (last?.kind === "text") {
    return [...entries.slice(0, -1), { kind: "text", text: last.text + text }];
  }
  return [...entries, { kind: "text", text }];
}

/** Plan text streams into an unfinished plan entry (`steps === null`). */
export function appendPlanText(entries: Entry[], text: string): Entry[] {
  const last = entries[entries.length - 1];
  if (last?.kind === "plan" && last.steps === null) {
    return [...entries.slice(0, -1), { ...last, text: last.text + text }];
  }
  return [...entries, { kind: "plan", text, steps: null }];
}

/** The finished plan replaces the streamed draft rather than appearing a second time. */
export function completePlan(entries: Entry[], plan: TaskPlan): Entry[] {
  const last = entries[entries.length - 1];
  const done: Entry = { kind: "plan", text: plan.text, steps: plan.steps };
  return last?.kind === "plan" && last.steps === null
    ? [...entries.slice(0, -1), done]
    : [...entries, done];
}
