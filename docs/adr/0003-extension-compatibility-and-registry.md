# 3. VS Code extension compatibility and the package registry

- **Status:** Accepted
- **Date:** 2026-09-23

## Context

The product owner has stated that Any Code needs VS Code extension capability and a store.
Three repository documents currently say otherwise, and they disagree with each other about
what "a store" even means:

- **PRD §10** lists "Full VS Code extension compatibility" as a **Non-Goal for V1**.
- **PRD §48** specifies a marketplace whose categories are Skills, Agents, MCP Servers,
  Connectors, Plugins, Workflows, Themes and Model Providers. VS Code extensions are not one
  of them. That marketplace is **V1.5** (PRD §9.2), and its foundation — plugin host,
  capability permissions, install UX — is **Phase 7**.
- **PRODUCT-SCOPE.md** explicitly defers "a marketplace before the capability and permission
  contracts are stable" and "arbitrary unsandboxed plugin code in the main application
  process".

So there are two separate asks wearing one name: *run other people's VS Code extensions*, and
*Any Code's own package store*. They share an install and permission surface and nothing else.

PROJECT-RULES.md ranks "applicable law, provider terms" above the PRD, which forces one part
of this decision regardless of scope.

## Decision

**1. The registry is Open VSX, never Microsoft's Marketplace.**

Microsoft's Marketplace Terms of Use restrict marketplace access to Microsoft's own products;
Microsoft has additionally moved to block non-VS Code clients and to restrict its first-party
extensions (C/C++, Pylance, .NET, Remote-*) to VS Code itself. Every comparable product —
Cursor, Windsurf, VSCodium, Gitpod, Theia — consumes Open VSX (Eclipse Foundation) for this
reason. Under our own authority order this is not a preference we can trade against
convenience, and it is a fixed input to every option below. It also caps what compatibility
can ever mean: the most-wanted extensions are precisely the ones not licensed to us.

**2. Compatibility is a ladder, and Any Code picks a rung rather than promising the top.**

| Rung | What runs | Needs | Rough cost |
|------|-----------|-------|-----------|
| A | Themes and TextMate grammars from `.vsix` | A zip reader and a theme/grammar mapper. No extension host. | Days |
| B | A + language servers from LSP extensions | Phase 4's LSP client, pointed at a server an extension ships | Weeks, mostly Phase 4 work we want anyway |
| C | B + a real extension host for a declared `vscode` API subset | Node sidecar, activation events, contribution points, per-extension sandbox and permissions | Months, ongoing |
| D | Full `vscode` API compatibility | Tracking an API surface Microsoft changes monthly | Not achievable as a side quest |

Rung D is what "VS Code extension compatibility" is usually taken to mean, and it is the thing
PRD §10 declines. Rungs A and B deliver most of what users actually install extensions for.

**Any Code commits to rung B.** The Open VSX constraint is what decides it: the extensions
users miss most — Pylance, C/C++, the Remote pack — are precisely the ones Microsoft licenses
only to VS Code, so a full extension host buys the long tail rather than the headliners, at a
cost of months plus a permanent third-party-code attack surface. Rung B's expensive half is the
LSP client, which Phase 4 ships regardless; the marginal cost is a `.vsix` reader and a
theme/grammar mapper. Rung C is additive to B rather than a rewrite, so this choice forecloses
nothing.

**3. The store stays Phase 7, and extensions become a category inside it, not a second store.**

PRD §48's package page already specifies publisher, permissions, required connections,
signature status and security-review status. An Open VSX extension is describable in exactly
those terms. Building a second install surface for extensions would duplicate the permission
model that PRODUCT-SCOPE.md says must be stable *before* any marketplace ships.

## Consequences

- PRD §10's non-goal and PRD §48's category list both need editing; this ADR does not itself
  change scope. Per PRODUCT-SCOPE.md "Change control", the same change must carry the reason,
  affected release and phase, security/privacy/cost/platform/migration impact, and updated
  acceptance criteria and roadmap.
- Any rung above A introduces third-party code execution. PROJECT-RULES.md already requires it
  to run out of process with explicit filesystem, network, CPU, memory and time limits, and to
  enter through the capability registry and permission engine like every other capability. Rung
  C's extension host is that sandbox and is the bulk of its cost.
- Users will ask for extensions Open VSX does not carry. That is permanent, not a gap to close.
- Nothing here is buildable in Phase 3. Phase 3's own exit condition — an agent implementing and
  verifying a task against a live provider — is still not demonstrated.
- Revisit rung C if Open VSX coverage or the LSP subset proves insufficient in real use, rather
  than on the assumption that it will.

## Scope change record

Required by PRODUCT-SCOPE.md "Change control".

**Reason and user outcome.** A developer moving from VS Code keeps their colour theme, their
syntax highlighting, and the language intelligence they rely on, without Any Code maintaining a
clone of an API surface Microsoft revises monthly.

**Affected release and phase.** V1. Rung B lands in **Phase 4 · Code intelligence** (it is that
phase's LSP work with an additional install source). The Extensions category lands in **Phase 7
· Capability platform** alongside the rest of PRD §48. No work enters Phase 3.

**Impact.**

- *Security:* rung B executes no third-party JavaScript. It does run third-party **language
  server binaries**, which is new. They enter through the capability registry and the permission
  engine like any other capability, and a `.vsix` is untrusted input — its manifest, theme JSON
  and grammars are data, parsed defensively, never instructions.
- *Privacy:* Open VSX queries reveal which extensions a user installs. Registry access is
  network I/O and must be absent in Local Only mode.
- *Cost:* no model cost. Engineering cost is the `.vsix` reader and theme/grammar mapper on top
  of Phase 4's LSP client.
- *Platform:* language server binaries are per-platform; an extension carrying a macOS-only
  server must fail visibly on Windows, not silently.
- *Migration:* none. Nothing ships today that this changes.

**Acceptance criteria.**

1. A `.vsix` from Open VSX installs, and its colour theme and TextMate grammar apply to an open
   file in Monaco.
2. An LSP-providing extension installs and its server answers hover, go-to-definition and
   diagnostics through the Phase 4 LSP client.
3. An extension whose server has no binary for the current platform reports that plainly and
   installs nothing.
4. Installing an extension while Local Only mode is on is refused with a reason, not attempted.
5. A `.vsix` containing an extension host entry point (`main`) installs its themes, grammars and
   server if present, and states plainly that its JavaScript will not run.
