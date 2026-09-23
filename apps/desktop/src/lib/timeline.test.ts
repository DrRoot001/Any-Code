import { describe, expect, it } from "vitest";
import {
  appendPlanText,
  appendText,
  argumentSummary,
  completePlan,
  resultSummary,
  type Entry,
} from "./timeline";

describe("resultSummary", () => {
  it("reports a non-zero exit as a failure", () => {
    expect(resultSummary({ exitCode: 1, stderr: "E\nFAILED (errors=1)", stdout: "" })).toEqual({
      ok: false,
      detail: "exit 1 · E\nFAILED (errors=1)",
    });
  });

  it("treats a killed process — no exit code — as a failure, not a pass", () => {
    // This once rendered as ✓: the check was `typeof exitCode === "number"`.
    expect(resultSummary({ exitCode: null, stderr: "", stdout: "" }).ok).toBe(false);
  });

  it("reports a tool error as a failure", () => {
    expect(resultSummary({ error: "denied by user" })).toEqual({
      ok: false,
      detail: "denied by user",
    });
  });

  it("summarises a file read by size, not content", () => {
    expect(resultSummary({ content: "abc" })).toEqual({ ok: true, detail: "3 chars" });
  });

  it("shows only the last three lines of long output", () => {
    const stdout = ["1", "2", "3", "4", "5"].join("\n");
    expect(resultSummary({ exitCode: 0, stdout, stderr: "" }).detail).toBe("exit 0 · 3\n4\n5");
  });
});

describe("argumentSummary", () => {
  it("prefers the command, then the path", () => {
    expect(argumentSummary({ command: "npm test", path: "x" })).toBe("npm test");
    expect(argumentSummary({ path: "src/a.ts" })).toBe("src/a.ts");
    expect(argumentSummary({})).toBe("");
  });
});

describe("timeline assembly", () => {
  it("joins streamed text into one entry until something else intervenes", () => {
    let entries: Entry[] = [];
    entries = appendText(entries, "Hel");
    entries = appendText(entries, "lo");
    expect(entries).toEqual([{ kind: "text", text: "Hello" }]);
    entries = [...entries, { kind: "cancelled" }];
    entries = appendText(entries, "again");
    expect(entries.map((e) => e.kind)).toEqual(["text", "cancelled", "text"]);
  });

  it("replaces the streamed plan draft with the finished plan, not a second copy", () => {
    let entries: Entry[] = [{ kind: "instruction", text: "do it" }];
    entries = appendPlanText(entries, "1. Read");
    entries = appendPlanText(entries, " it");
    entries = completePlan(entries, { steps: ["Read it"], text: "1. Read it" });
    expect(entries).toEqual([
      { kind: "instruction", text: "do it" },
      { kind: "plan", text: "1. Read it", steps: ["Read it"] },
    ]);
  });

  it("does not overwrite an earlier task's finished plan", () => {
    const finished: Entry = { kind: "plan", text: "old", steps: ["old"] };
    const entries = completePlan([finished], { steps: ["new"], text: "new" });
    expect(entries).toHaveLength(2);
    expect(entries[0]).toBe(finished);
  });
});
