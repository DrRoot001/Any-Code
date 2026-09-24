import { describe, expect, it } from "vitest";
import { formatStartedAt, outcomeClass, outcomeLabel } from "./taskHistory";

describe("outcomeLabel", () => {
  it("labels each finished outcome plainly", () => {
    expect(outcomeLabel("completed")).toBe("completed");
    expect(outcomeLabel("failed")).toBe("failed");
    expect(outcomeLabel("cancelled")).toBe("cancelled");
  });

  it("says a task with no outcome did not finish, rather than hiding it", () => {
    expect(outcomeLabel(null)).toBe("did not finish");
  });
});

describe("outcomeClass", () => {
  it("gives an unfinished task its own class rather than reusing null", () => {
    expect(outcomeClass(null)).toBe("unfinished");
    expect(outcomeClass("failed")).toBe("failed");
  });
});

describe("formatStartedAt", () => {
  it("formats a valid ISO timestamp into something other than the raw string", () => {
    const formatted = formatStartedAt("2026-01-15T10:30:00Z");
    expect(formatted).not.toBe("2026-01-15T10:30:00Z");
    expect(formatted.length).toBeGreaterThan(0);
  });

  it("falls back to the raw value when the timestamp cannot be parsed", () => {
    expect(formatStartedAt("not-a-date")).toBe("not-a-date");
  });
});
