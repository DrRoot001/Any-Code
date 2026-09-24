/** Wording for the Context Inspector (PRD §37), kept out of the component and tested. */
import type { ContextPackage, ContextReason } from "./tauri";

/** Mirrors anycode_context::describe, so the inspector and the prompt say the same thing. */
export function describeReason(reason: ContextReason): string {
  switch (reason.kind) {
    case "named_in_instruction":
      return "named in the instruction";
    case "defines_symbol":
      return `defines \`${reason.symbol}\``;
    case "matches_terms":
      return reason.terms.length ? `matches ${reason.terms.join(", ")}` : "full-text match";
    case "imported_by":
      return `imported by ${reason.path}`;
    case "changed_in_working_tree":
      return "changed in the working tree";
  }
}

/**
 * How targeted the package is, in words. Every number is an estimate and says so; a
 * percentage under 0.1 is shown as "under 0.1%" rather than rounding a real share to zero.
 */
export function describeScale(p: ContextPackage): string {
  if (p.items.length === 0) return "No repository context was selected for this task.";
  const share = p.repoEstTokens > 0 ? (100 * p.estTokens) / p.repoEstTokens : 0;
  const pct = share > 0 && share < 0.1 ? "under 0.1%" : `${share.toFixed(1)}%`;
  const files = new Set(p.items.map((i) => i.path)).size;
  return (
    `${p.items.length} passage${p.items.length === 1 ? "" : "s"} from ${files} file` +
    `${files === 1 ? "" : "s"} · ~${p.estTokens.toLocaleString()} of ` +
    `~${p.repoEstTokens.toLocaleString()} repository tokens (${pct}, estimated)`
  );
}

export function lineRange(start: number, end: number): string {
  return start === end ? `line ${start}` : `lines ${start}–${end}`;
}
