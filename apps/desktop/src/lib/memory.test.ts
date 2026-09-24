import { describe, expect, it } from "vitest";
import { scopeLabel, sourceLabel } from "./memory";

describe("scopeLabel", () => {
  it("labels global and workspace scopes as text", () => {
    expect(scopeLabel("global")).toBe("Global");
    expect(scopeLabel("workspace")).toBe("This workspace");
  });
});

describe("sourceLabel", () => {
  it("reads an adopted instruction file's source plainly", () => {
    expect(sourceLabel("adopted:CLAUDE.md")).toBe("adopted from CLAUDE.md");
  });

  it("is null when there is no source, not an empty string", () => {
    expect(sourceLabel(null)).toBeNull();
  });

  it("shows an unrecognised source as written rather than hiding it", () => {
    expect(sourceLabel("something-else")).toBe("something-else");
  });
});
