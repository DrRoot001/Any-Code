import { describe, expect, it } from "vitest";
import { grantLabel, requestSummary } from "./approval";
import type { TaskApprovalRequest } from "./tauri";

const request = (overrides: Partial<TaskApprovalRequest>): TaskApprovalRequest => ({
  id: "c1",
  name: "shell.execute",
  arguments: {},
  risk: "medium",
  reason: null,
  grantable: true,
  grantScope: "shell.execute:npm test",
  ...overrides,
});

describe("requestSummary", () => {
  it("shows the exact command", () => {
    expect(requestSummary(request({ arguments: { command: "npm test" } }))).toBe("npm test");
  });

  it("shows what a whole-file write will put in the file, not just its path", () => {
    // Approving a path without seeing the content is not an informed decision.
    const summary = requestSummary(
      request({
        name: "filesystem.write.workspace",
        arguments: { path: "calc.py", content: "def f():\n    pass\n" },
      }),
    );
    expect(summary).toContain("calc.py — replaces the whole file with:");
    expect(summary).toContain("def f():");
  });

  it("shows an edit as removed and added lines", () => {
    const summary = requestSummary(
      request({
        name: "filesystem.edit.workspace",
        arguments: { path: "a.py", old_text: "x = 1\ny = 2", new_text: "x = 3" },
      }),
    );
    expect(summary).toBe("a.py\n\n- x = 1\n- y = 2\n+ x = 3");
  });
});

describe("grantLabel", () => {
  it("names the exact command a shell grant covers", () => {
    // Audit S2: the button used to read "Always allow in this workspace" while granting
    // every shell command. It now says what it grants.
    expect(grantLabel(request({ grantScope: "shell.execute:npm test" }))).toBe(
      "Always allow “npm test” here",
    );
  });

  it("keeps a command containing a colon intact", () => {
    expect(grantLabel(request({ grantScope: "shell.execute:npm run test:unit" }))).toBe(
      "Always allow “npm run test:unit” here",
    );
  });

  it("names the capability for a tool-wide grant", () => {
    expect(grantLabel(request({ grantScope: "filesystem.write.workspace" }))).toBe(
      "Always allow filesystem.write.workspace in this workspace",
    );
  });
});
