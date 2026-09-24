import { describe, expect, it } from "vitest";
import { describeReason, describeScale, lineRange } from "./context";
import type { ContextPackage } from "./tauri";

const pkg = (over: Partial<ContextPackage>): ContextPackage => ({
  intent: { paths: [], identifiers: [], terms: [] },
  items: [],
  excluded: [],
  estTokens: 0,
  budgetTokens: 3000,
  repoEstTokens: 100000,
  repoFiles: 200,
  ...over,
});

const item = (path: string, estTokens: number) => ({
  path,
  startLine: 1,
  endLine: 10,
  text: "",
  outline: false,
  reasons: [],
  score: 1,
  estTokens,
});

describe("describeReason", () => {
  it("says why each passage was chosen", () => {
    expect(describeReason({ kind: "defines_symbol", symbol: "multiply" })).toBe("defines `multiply`");
    expect(describeReason({ kind: "matches_terms", terms: ["test", "suite"] })).toBe(
      "matches test, suite",
    );
    expect(describeReason({ kind: "matches_terms", terms: [] })).toBe("full-text match");
  });
});

describe("describeScale", () => {
  it("reports passages, files and the estimated share of the repository", () => {
    const text = describeScale(
      pkg({ items: [item("a.rs", 500), item("a.rs", 300), item("b.rs", 200)], estTokens: 1000 }),
    );
    expect(text).toBe(
      "3 passages from 2 files · ~1,000 of ~100,000 repository tokens (1.0%, estimated)",
    );
  });

  it("never rounds a real share down to zero", () => {
    const text = describeScale(pkg({ items: [item("a.rs", 40)], estTokens: 40, repoEstTokens: 1e6 }));
    expect(text).toContain("under 0.1%");
  });

  it("says plainly when nothing was selected", () => {
    expect(describeScale(pkg({}))).toBe("No repository context was selected for this task.");
  });
});

describe("lineRange", () => {
  it("formats one line and a range", () => {
    expect(lineRange(4, 4)).toBe("line 4");
    expect(lineRange(4, 9)).toBe("lines 4–9");
  });
});
