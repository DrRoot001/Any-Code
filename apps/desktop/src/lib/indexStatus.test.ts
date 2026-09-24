import { describe, expect, it } from "vitest";
import { indexStatusLabel, indexStatusTooltip } from "./indexStatus";

describe("indexStatusLabel", () => {
  it("shows nothing for none", () => {
    expect(indexStatusLabel({ state: "none" })).toBeNull();
  });

  it("says indexing for building", () => {
    expect(indexStatusLabel({ state: "building" })).toBe("Indexing…");
  });

  it("counts files for ready, with correct pluralisation", () => {
    expect(
      indexStatusLabel({ state: "ready", files: 1, chunks: 3, symbols: 5, lastRefreshMs: 40 }),
    ).toBe("Indexed 1 file");
    expect(
      indexStatusLabel({ state: "ready", files: 240, chunks: 3, symbols: 5, lastRefreshMs: 40 }),
    ).toBe("Indexed 240 files");
  });

  it("says the index failed, without the error inline", () => {
    expect(indexStatusLabel({ state: "failed", error: "disk full" })).toBe("Index failed");
  });
});

describe("indexStatusTooltip", () => {
  it("has nothing to add for none or building", () => {
    expect(indexStatusTooltip({ state: "none" })).toBeNull();
    expect(indexStatusTooltip({ state: "building" })).toBeNull();
  });

  it("reports chunks, symbols and refresh time for ready", () => {
    expect(
      indexStatusTooltip({ state: "ready", files: 240, chunks: 900, symbols: 1500, lastRefreshMs: 812 }),
    ).toBe("900 chunks · 1,500 symbols · last refreshed in 812 ms");
  });

  it("surfaces the real error for failed", () => {
    expect(indexStatusTooltip({ state: "failed", error: "disk full" })).toBe("disk full");
  });
});
