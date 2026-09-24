/**
 * Wording for the index status badge in the StatusBar (Phase 4). Kept out of the
 * component so it can be tested directly (indexStatus.test.ts).
 */
import type { IndexStatus } from "./tauri";

/**
 * The workspace index's state, in words — never left to a colour alone. `null` means
 * nothing is shown: `none` is not an error, just "no index has been built".
 */
export function indexStatusLabel(status: IndexStatus): string | null {
  switch (status.state) {
    case "none":
      return null;
    case "building":
      return "Indexing…";
    case "ready":
      return `Indexed ${status.files.toLocaleString()} file${status.files === 1 ? "" : "s"}`;
    case "failed":
      return "Index failed";
  }
}

/** Extra detail for the tooltip. Only `ready` and `failed` have anything to add. */
export function indexStatusTooltip(status: IndexStatus): string | null {
  switch (status.state) {
    case "ready":
      return (
        `${status.chunks.toLocaleString()} chunks · ${status.symbols.toLocaleString()} symbols · ` +
        `last refreshed in ${status.lastRefreshMs.toLocaleString()} ms`
      );
    case "failed":
      return status.error;
    default:
      return null;
  }
}
