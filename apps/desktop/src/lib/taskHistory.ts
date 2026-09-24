/**
 * Pure formatting for the History view (ADR 0004 "Session resume"). Kept out of the
 * component so it can be tested directly (taskHistory.test.ts).
 */
import type { TaskSummary } from "./tauri";

/** How a past task's outcome reads in the list — never left to a colour alone. */
export function outcomeLabel(outcome: TaskSummary["outcome"]): string {
  return outcome ?? "did not finish";
}

/** A CSS-safe suffix for styling by outcome, alongside the text label. */
export function outcomeClass(outcome: TaskSummary["outcome"]): string {
  return outcome ?? "unfinished";
}

/** A past task's start time, in the viewer's own locale. Falls back to the raw value
 * rather than showing "Invalid Date" if the backend ever sends something unparsable. */
export function formatStartedAt(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}
