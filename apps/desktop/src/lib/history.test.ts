import { describe, expect, it } from "vitest";
import { eventsToEntries, type AuditEvent } from "./history";

/** Builds one audit row with sane defaults, overriding only what a test cares about. */
function event(kind: string, payload: Record<string, unknown> = {}): AuditEvent {
  return { id: "e", timestamp: "2026-01-01T00:00:00Z", kind, payload };
}

describe("eventsToEntries", () => {
  it("yields no entries for an empty log", () => {
    expect(eventsToEntries([])).toEqual([]);
  });

  it("rebuilds a full task in order", () => {
    const events: AuditEvent[] = [
      event("task.created", { instruction: "add a test" }),
      event("task.plan", { text: "1. write it\n2. run it", steps: ["write it", "run it"] }),
      event("task.turn", { text: "Looking at the file now." }),
      event("task.tool.call", {
        id: "c1",
        name: "shell.execute",
        arguments: { command: "cargo test" },
        risk: "medium",
      }),
      event("task.tool.result", {
        id: "c1",
        name: "shell.execute",
        result: { exitCode: 0, stdout: "ok", stderr: "" },
      }),
      event("task.finished", {
        evidence: { filesChanged: [], commands: [] },
        verdict: { type: "unverified" },
        usage: { inputTokens: 10, outputTokens: 20 },
      }),
    ];

    const entries = eventsToEntries(events);
    expect(entries.map((e) => e.kind)).toEqual([
      "instruction",
      "plan",
      "text",
      "tool_call",
      "tool_result",
      "done",
    ]);
    expect(entries[0]).toEqual({ kind: "instruction", text: "add a test" });
    expect(entries[1]).toEqual({
      kind: "plan",
      text: "1. write it\n2. run it",
      steps: ["write it", "run it"],
    });
    expect(entries[3]).toMatchObject({ kind: "tool_call", id: "c1", name: "shell.execute", risk: "medium" });
    expect(entries[4]).toMatchObject({
      kind: "tool_result",
      id: "c1",
      ok: true,
      detail: "exit 0 · ok",
    });
  });

  it("marks a rejected tool call with risk 'rejected' and its reason", () => {
    const entries = eventsToEntries([
      event("task.tool.rejected", {
        id: "c1",
        name: "shell.execute",
        arguments: {},
        reason: "shell.execute is missing required argument(s): command",
      }),
    ]);
    expect(entries).toEqual([
      {
        kind: "tool_call",
        id: "c1",
        name: "shell.execute",
        risk: "rejected",
        summary: "",
        reason: "shell.execute is missing required argument(s): command",
      },
      { kind: "interrupted" },
    ]);
  });

  it("reports a truncated shell result as a failure carrying its exit code", () => {
    const entries = eventsToEntries([
      event("task.tool.result", {
        id: "c1",
        name: "shell.execute",
        result: { truncated: true, bytes: 50000, prefix: '{"exitCode":1,"stdout":"lots of ou' },
      }),
    ]);
    expect(entries[0]).toMatchObject({ kind: "tool_result", ok: false });
    expect((entries[0] as { detail: string }).detail).toContain("exit 1");
  });

  it("reports a truncated non-shell result as ok and not kept in full", () => {
    const entries = eventsToEntries([
      event("task.tool.result", {
        id: "c1",
        name: "filesystem.read.workspace",
        result: { truncated: true, bytes: 90000, prefix: '{"content":"a very long file...' },
      }),
    ]);
    expect(entries[0]).toMatchObject({ kind: "tool_result", ok: true });
    expect((entries[0] as { detail: string }).detail).toContain("not kept in full");
    expect((entries[0] as { detail: string }).detail).toContain("90,000");
  });

  it("shows the prefix of a truncated task.turn text", () => {
    const entries = eventsToEntries([
      event("task.turn", {
        text: { truncated: true, bytes: 20000, prefix: "Here is what I found so far" },
      }),
    ]);
    expect(entries[0]).toMatchObject({ kind: "text", text: "Here is what I found so far…" });
  });

  it("ends an unfinished task as interrupted", () => {
    const entries = eventsToEntries([event("task.created", { instruction: "do it" })]);
    expect(entries[entries.length - 1]).toEqual({ kind: "interrupted" });
  });

  it("ends a cancelled task as cancelled, not interrupted", () => {
    const entries = eventsToEntries([
      event("task.created", { instruction: "do it" }),
      event("task.state", { state: "cancelled" }),
    ]);
    expect(entries[entries.length - 1]).toEqual({ kind: "cancelled" });
  });

  it("skips unknown event kinds without throwing", () => {
    expect(() =>
      eventsToEntries([
        event("task.created", { instruction: "do it" }),
        event("task.approval", { id: "c1", name: "shell.execute", decision: "allowed_by_policy" }),
        event("some.future.kind", { anything: true }),
        event("task.state", { state: "cancelled" }),
      ]),
    ).not.toThrow();
    const entries = eventsToEntries([
      event("task.created", { instruction: "do it" }),
      event("some.future.kind", { anything: true }),
      event("task.state", { state: "cancelled" }),
    ]);
    expect(entries.map((e) => e.kind)).toEqual(["instruction", "cancelled"]);
  });
});
