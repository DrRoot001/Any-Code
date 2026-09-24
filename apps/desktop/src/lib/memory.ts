/**
 * Pure formatting for the Memory view (ADR 0004 "Memory"). Kept out of the component so
 * it can be tested directly (memory.test.ts).
 */
import type { MemoryScope } from "./tauri";

export function scopeLabel(scope: MemoryScope): string {
  return scope === "global" ? "Global" : "This workspace";
}

/**
 * `source` is the raw value the backend stores (e.g. `adopted:CLAUDE.md`, from
 * memory_commands.rs's `adopted_source`). This turns it into the plain sentence the
 * Memory list shows; anything not in that shape is shown as written rather than hidden.
 */
export function sourceLabel(source: string | null): string | null {
  if (!source) return null;
  const [kind, ...rest] = source.split(":");
  const detail = rest.join(":");
  return kind === "adopted" && detail ? `adopted from ${detail}` : source;
}
