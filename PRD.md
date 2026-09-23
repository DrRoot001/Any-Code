# Any Code

## Product Requirements Document

**Product:** Any Code
**Category:** Universal Agentic Software Engineering Platform
**Document Version:** 1.0
**Status:** Proposed implementation specification
**Primary Platforms:** Windows, macOS
**Secondary Platforms:** Linux, iOS, Android, Web
**Core Runtime:** Rust
**Primary Client:** Tauri 2 + React + TypeScript
**Product Principle:** Any model. Any codebase. Any tool. One workspace.

---

# 1. Executive Summary

Any Code is a universal agentic development environment designed to replace the fragmented workflow developers currently experience across Claude Code, Codex, Gemini CLI, Cursor, Windsurf, IDE extensions, terminals, browser tools, MCP clients, AI subscriptions, API accounts, plugins, skills, and cloud coding agents.

The problem is not simply that developers need another AI coding assistant.

The problem is that modern developers increasingly operate a collection of disconnected AI products:

| Current problem                                        | Any Code response                  |
| ------------------------------------------------------ | ---------------------------------- |
| Several AI subscriptions                               | Unified Provider Center            |
| Several API accounts                                   | Universal Model Gateway            |
| Different usage limits                                 | Usage and Budget Center            |
| Different MCP configurations                           | Universal Capability Registry      |
| Different skills and rules                             | Shared Skills System               |
| Different project memories                             | Persistent Workspace Intelligence  |
| Repeated repository indexing                           | Shared Context Engine              |
| Different coding agents                                | Universal Agent Runtime            |
| Different terminals                                    | Integrated PTY                     |
| IDE switching                                          | Unified Workbench                  |
| Browser testing separately                             | Integrated Browser Agent           |
| Different cloud agents                                 | Local and Cloud Execution          |
| Provider outages                                       | Automatic fallback routing         |
| Expensive models used for trivial work                 | Cost aware model routing           |
| Cheap models used for difficult work                   | Capability aware routing           |
| No unified spend visibility                            | Provider usage ledger              |
| Repeated authentication                                | Connection Vault                   |
| Secrets exposed to tools                               | Permission controlled secret vault |
| Agents changing dangerous files                        | Sandboxed execution policies       |
| Parallel agents conflicting                            | Git worktree isolation             |
| AI saying work is complete without proof               | Verification pipeline              |
| Cannot continue work from phone                        | Mobile control plane               |
| Context disappears between sessions                    | Persistent inspectable memory      |
| Constant copying between GitHub, Linear, Slack and IDE | Connectors and MCP                 |
| Plugins have excessive permissions                     | Capability sandbox                 |

Any Code will therefore function as a **software engineering operating layer**, not merely a chat interface.

---

# 2. Product Vision

Any Code should become the environment through which a developer accesses AI rather than forcing the developer to choose a single AI vendor.

The user should be able to connect:

1. OpenAI
2. Anthropic
3. Google
4. OpenRouter
5. Azure OpenAI
6. Amazon Bedrock
7. Google Vertex AI
8. Local Ollama compatible models
9. OpenAI compatible endpoints
10. Future model providers

The user should then decide whether Any Code selects the model automatically or whether a specific model should be used.

The long term mental model is:

```text
Developer
    │
    ▼
Any Code
    │
    ├── Models
    ├── Agents
    ├── Skills
    ├── MCP
    ├── Connectors
    ├── Plugins
    ├── Terminal
    ├── Browser
    ├── Git
    ├── Cloud
    └── Code Intelligence
```

The underlying provider becomes infrastructure.

The developer stays inside Any Code.

---

# 3. Positioning

## 3.1 Product category

**Universal Agentic Development Environment**

Secondary enterprise positioning:

**Autonomous Software Engineering Platform**

## 3.2 Core promise

> Any model. Any codebase. Any tool. One workspace.

## 3.3 Secondary message

> Stop paying for disconnected coding workflows.

Any Code does not promise that a user can illegally transfer or reuse subscriptions belonging to another product.

Instead, Any Code centralizes every provider connection that the provider officially permits.

Where subscription based authentication is officially available, Any Code may support it.

Where it is not available, Any Code uses:

1. BYOK API access
2. Enterprise provider credentials
3. Local models
4. Approved provider integrations
5. Optional Any Code managed credits in a future release

---

# 4. Brand Identity

Any Code is powered by [heptagram-ai.com](https://heptagram-ai.com). This attribution appears in
the application shell, website and repository entry points, release metadata, and formal brand
materials. It must remain secondary to the Any Code product name and must not be incorporated into
the core logo mark.

## 4.1 Name

**Any Code**

Recommended canonical forms:

```text
Any Code
ANY CODE
anycode
anycode.dev
```

CLI binary:

```bash
anycode
```

Examples:

```bash
anycode
anycode init
anycode run
anycode agents
anycode models
anycode connect
anycode mcp
anycode skills
anycode usage
anycode doctor
```

---

# 5. Founder Signature and Product Mark

The numerical signature is:

```text
207402
```

Its binary representation is:

```text
110010101000101010
```

Its hexadecimal representation is:

```text
32A2A
```

The binary string should become a hidden or secondary visual signature throughout the product.

Examples:

```text
AC / 207402
110010101000101010
AC:32A2A
```

## 5.1 Logo direction

The logo may use the aggressive geometric language associated with tactical game interfaces, but it must not copy the VALORANT logo.

The Any Code mark should be independently ownable intellectual property.

Recommended construction:

1. Two angular shapes create an abstract `A`.
2. Negative space forms part of a `C`.
3. Small binary cuts or notches correspond to `110010101000101010`.
4. The mark remains recognizable at 16 × 16 pixels.
5. The full binary signature appears only in detailed treatments.
6. The symbol must work monochromatically.
7. Avoid gradients in the primary logo.
8. Avoid obvious robot, brain, sparkle, terminal prompt, or generic AI imagery.

Canonical assets:

```text
assets/brand/any-code-mark.png       transparent master mark
assets/brand/any-code-app-icon.png   dark app-icon treatment
```

The standard attribution is:

```text
Powered by heptagram-ai.com
```

The UI itself should remain professional and restrained rather than adopting a gaming aesthetic.

---

# 6. Target Users

## 6.1 Individual Developer

Uses multiple AI coding products and wants one environment.

Needs:

1. Fast coding assistance
2. Cheap model selection
3. Existing subscriptions where officially usable
4. BYOK
5. Local models
6. Git
7. Terminal
8. Browser
9. MCP
10. Persistent project context

## 6.2 AI Power User

Currently jumps between different models because each is stronger at different tasks.

Needs:

1. Model comparison
2. Automatic routing
3. Parallel agents
4. Context control
5. Cost control
6. Custom skills
7. Custom workflows

## 6.3 Professional Software Engineer

Needs reliability more than AI novelty.

Needs:

1. Correct code
2. Tests
3. Git safety
4. Diff review
5. Workspace isolation
6. Reproducible execution
7. Debugging tools
8. Security controls

## 6.4 Development Team

Needs:

1. Shared rules
2. Shared skills
3. Shared connectors
4. Centralized budgets
5. Role based permissions
6. Team analytics
7. Audit trails
8. Standardized workflows

## 6.5 Enterprise

Needs:

1. SSO
2. RBAC
3. Audit logging
4. Data residency controls
5. Private model endpoints
6. Model allowlists
7. MCP policies
8. Secret management
9. On premises or VPC execution
10. Compliance controls

---

# 7. Primary Jobs to Be Done

The user should be able to say:

```text
Implement Stripe subscriptions.
```

Any Code should be capable of:

```text
Understand repository
        ↓
Create implementation plan
        ↓
Determine required context
        ↓
Select models
        ↓
Create isolated agent tasks
        ↓
Modify code
        ↓
Run migrations
        ↓
Run tests
        ↓
Launch application
        ↓
Verify using browser
        ↓
Inspect errors
        ↓
Fix failures
        ↓
Review final diff
        ↓
Present evidence
        ↓
Request approval
```

A completed task means verified engineering work, not merely generated text.

---

# 8. Product Principles

## 8.1 Provider independence

No major subsystem may directly depend on a specific model vendor.

## 8.2 Local first

Repositories, indexes, secrets, terminals, and agent execution should remain local unless the user explicitly enables cloud functionality.

## 8.3 Cloud optional

A user must be capable of running Any Code without uploading the repository to Any Code Cloud.

## 8.4 Explainable routing

When Any Code automatically chooses a model, the user can inspect why.

Example:

```text
Selected: GPT coding model

Reason
Complex TypeScript implementation
Strong tool calling required
128k context required
OpenAI monthly API budget still healthy
Expected cost: $0.41
```

## 8.5 Human authority

The LLM never becomes the security boundary.

The runtime determines what the model may do.

## 8.6 Evidence before completion

An agent cannot report success solely because code was written.

## 8.7 No artificial provider lock in

Users own:

1. Their configuration
2. Their prompts
3. Their skills
4. Their memories
5. Their MCP configurations
6. Their workflows
7. Their usage records

All should be exportable.

---

# 9. Product Scope

## 9.1 V1

V1 includes:

1. Windows desktop application
2. macOS desktop application
3. Repository workspace
4. Integrated Monaco editor
5. Integrated terminal
6. Git integration
7. Universal AI provider system
8. BYOK
9. Provider routing
10. Agent runtime
11. Parallel subagents
12. Code intelligence
13. MCP client
14. Skills
15. Connectors foundation
16. Plugin foundation
17. Browser automation
18. Usage monitoring
19. Budget monitoring
20. Dark theme
21. Light theme
22. System theme
23. Any Code account
24. Optional cloud sync
25. Staging backend
26. Auto update infrastructure
27. Crash reporting
28. Application telemetry controls

## 9.2 V1.5

1. Linux desktop
2. Web control center
3. Remote cloud agents
4. GitHub integration
5. GitLab integration
6. Linear
7. Jira
8. Slack
9. Sentry
10. Vercel
11. Supabase
12. Marketplace
13. Team workspaces

## 9.3 V2

1. iOS
2. Android
3. Enterprise SSO
4. Organization policy engine
5. Cloud sandboxes
6. Shared agents
7. Agent workflow builder
8. Private marketplace
9. Enterprise MCP management
10. Any Code managed model credits

---

# 10. Non Goals for V1

Any Code should not attempt to reproduce every feature of VS Code immediately.

V1 does not require:

1. Full VS Code extension compatibility — V1 runs a `.vsix`'s colour themes, TextMate grammars
   and bundled language servers, and does **not** implement the `vscode` extension API or run an
   extension's JavaScript. Packages come from Open VSX; Microsoft's Marketplace terms restrict it
   to Microsoft's own products. See [ADR 0003](docs/adr/0003-extension-compatibility-and-registry.md).
2. Every programming language debugger
3. Arbitrary local plugin execution on iOS
4. Full local development environments on iPhone
5. Complete cloud IDE infrastructure
6. Own foundation model
7. Own vector database product
8. Own Git hosting
9. Own CI platform
10. Replacement for GitHub

---

# 11. Platform Architecture

For Any Code, the recommended client architecture is:

```text
                    React + TypeScript
                           │
                        Tauri 2
                           │
                           ▼
                      Rust Runtime
                           │
           ┌───────────────┼────────────────┐
           │               │                │
      Local Runtime     Cloud API      Native OS
           │               │                │
           ▼               ▼                ▼
        Agents          Sync           Keychain
        Models          Teams          PTY
        Tools           Usage          Filesystem
        Git             Remote Tasks   Notifications
        MCP             Marketplace    Updates
```

Tauri 2 officially targets Windows, macOS, Linux, Android, and iOS while allowing the application logic to remain in Rust and the interface to remain web based.

This is better suited to Any Code than Flutter because the desktop product heavily depends on web based developer components such as:

```text
Monaco
xterm.js
virtualized trees
markdown renderers
diff viewers
browser developer tooling
React based UI libraries
```

---

# 12. Technology Stack

## 12.1 Desktop and shared client

```text
Tauri 2
React
TypeScript
Vite
TanStack Router
TanStack Query
Zustand
Tailwind CSS
Radix primitives
Monaco Editor
xterm.js
Framer Motion only where useful
```

Zustand handles fast local application state.

TanStack Query handles asynchronous remote and Rust command state.

Do not put the complete application state inside React Context.

---

# 13. Rust Core

Recommended crates and technologies:

```text
Rust
Tokio
Serde
Reqwest
SQLx
rusqlite where appropriate
tracing
tower
axum
notify
portable-pty
git2
tree-sitter
regex
keyring
secrecy
zeroize
argon2 where required
```

Core responsibilities:

```text
Agent execution
Model routing
Provider adapters
Filesystem
Terminal
Git
Repository indexing
MCP
Plugins
Capability security
Secret access
Usage metering
Context construction
Event storage
Cloud synchronization
```

JavaScript must not directly execute arbitrary shell commands.

All privileged operations pass through Rust.

---

# 14. Mobile Architecture

The iOS and Android applications use the same Tauri and React architecture where practical.

However, mobile is primarily a control plane.

Primary mobile functions:

```text
View running tasks
Start cloud task
Chat with workspace agent
Review diff
Approve changes
Reject changes
Inspect logs
View usage
Change model
Manage connections
Receive notifications
View files
View pull requests
Stop agents
Resume agents
```

Mobile should not attempt to recreate the complete desktop workbench.

A lightweight read only code viewer should replace full Monaco functionality where appropriate.

Local arbitrary plugins must not be a requirement on iOS.

Desktop or cloud workers execute those capabilities.

---

# 15. Any Code Account Authentication

Any Code identity is separate from AI provider authentication.

Recommended architecture:

```text
OIDC
OAuth 2.1
Authorization Code + PKCE
Passkeys
Email verification
MFA
Device management
```

Native applications must be treated as public OAuth clients.

No client secret may be embedded in the application.

Authorization Code with PKCE is appropriate for desktop and mobile public clients.

Recommended first identity provider:

**ZITADEL**

Reasons:

```text
OIDC
OAuth
SAML
MFA
Passkeys
Organizations
Auditability
Self hosting capability
```

ZITADEL currently supports OIDC, SAML, OAuth2 and passkey based authentication.

The identity layer must nevertheless be abstracted behind Any Code's `IdentityProvider` interface so migration remains possible.

---

# 16. AI Provider Authentication

This requires strict separation between:

```text
Any Code authentication

and

AI Provider authentication
```

A user being logged into Any Code does not mean Any Code automatically has access to Claude, OpenAI, Gemini or another provider.

---

# 17. Provider Connection Modes

Every provider adapter declares supported authentication modes.

```typescript
type ProviderAuthMode =
  | "oauth"
  | "device_code"
  | "api_key"
  | "access_token"
  | "service_account"
  | "cloud_credentials"
  | "local"
```

Example metadata:

```typescript
interface ProviderManifest {
  id: string
  name: string
  authModes: ProviderAuthMode[]
  supportsUsageApi: boolean
  supportsCostApi: boolean
  supportsSubscriptionUsage: boolean
  supportsStreaming: boolean
  supportsTools: boolean
  supportsVision: boolean
  supportsReasoning: boolean
}
```

---

# 18. OpenAI Integration

Current Codex supports browser based ChatGPT sign in and API key authentication. OpenAI also documents access tokens for certain trusted automation scenarios.

Any Code must not assume that Codex's first party ChatGPT login flow can automatically be embedded into an unrelated third party application.

Supported Any Code implementation must therefore prioritize:

```text
OpenAI API key
Official approved OAuth where available
Enterprise credentials
Approved future provider integration
```

Never scrape:

```text
ChatGPT cookies
browser sessions
private OAuth tokens
Codex auth files
```

unless OpenAI explicitly documents such interoperability.

---

# 19. Anthropic Integration

Claude Code currently supports subscription credentials as well as API access. Anthropic documents OAuth tokens and API keys as distinct authentication mechanisms.

Any Code V1 should support:

```text
Anthropic API key
Amazon Bedrock
Vertex supported Claude access
Enterprise gateway support
```

Subscription based Anthropic authentication should only be implemented when Anthropic explicitly permits the Any Code client to use the relevant OAuth flow.

No authentication reverse engineering.

---

# 20. Google Integration

Gemini CLI currently supports:

```text
Google sign in
Gemini API key
Vertex AI
```

Google specifically recommends Google account sign in for many Gemini CLI users, while API key and Vertex authentication are separate options.

Any Code V1 supports:

```text
Gemini API key
Vertex AI service credentials
Vertex ADC where appropriate
Approved Google OAuth integration
```

Google's API key architecture is also evolving toward authorization keys, so the provider adapter must not hardcode assumptions about key format.

---

# 21. Other Provider Modes

Initial adapter targets:

| Provider                  | Authentication     |
| ------------------------- | ------------------ |
| OpenRouter                | API key            |
| Ollama                    | Local              |
| LM Studio                 | Local endpoint     |
| Azure OpenAI              | Azure credentials  |
| Amazon Bedrock            | AWS credentials    |
| Vertex AI                 | Google credentials |
| OpenAI compatible         | Base URL + API key |
| Custom enterprise gateway | Configurable       |

---

# 22. Secure Credential Vault

Provider secrets must never live inside normal application state.

Desktop storage:

```text
macOS Keychain
Windows Credential Manager
Linux Secret Service
```

Cloud synced credentials require envelope encryption.

Architecture:

```text
User credential
      │
      ▼
Local encryption
      │
      ▼
Device key
      │
      ▼
OS secure storage
```

Cloud:

```text
Credential
    │
    ▼
Data Encryption Key
    │
    ▼
Encrypted credential
    │
    ▼
KMS encrypted DEK
```

Application logs must redact credentials automatically.

---

# 23. Provider Center UX

Main screen:

```text
Connections
─────────────────────────────────────────────

OpenAI
Connected
Authentication: API
Models: 12
This month: $14.82
Health: Good
[Manage]

Anthropic
Connected
Authentication: API
This month: $8.17
Health: Good
[Manage]

Gemini
Connected
Authentication: Google Cloud
This month: $3.91
Health: Good
[Manage]

Ollama
Local
Models: 4
Cost: $0
Health: Available
[Manage]
```

Each connection displays:

```text
Authentication method
Connection status
Model availability
Last successful request
Latency
Rate limit health
Usage
Spend
Configured budget
Failure rate
```

---

# 24. Subscription Manager

This is one of Any Code's principal differentiators.

Developers frequently pay separately for:

```text
ChatGPT
Claude
Gemini
Cursor
Windsurf
GitHub Copilot
API providers
other coding agents
```

Any Code should provide a **Subscription Ledger**.

Example:

```text
AI Spend
─────────────────────────────────────────────
ChatGPT              $20
Claude               $100
Gemini                $20
Cursor                $20
OpenRouter             $8.42
OpenAI API            $13.81

Estimated monthly total
$182.23
```

Closed products may not expose billing information programmatically.

Therefore costs can originate from:

```text
Provider billing API
User entered monthly subscription
API usage events
Imported invoice where supported
Estimated token pricing
```

Any Code must indicate the data source.

Never invent provider remaining quota.

---

# 25. Subscription Intelligence

Any Code should answer:

```text
Which provider did I actually use?

What am I paying per month?

Which subscriptions overlap?

Which model handles most of my work?

Which provider is cheapest for my tasks?

Which expensive subscription am I barely using?

How much did this project cost?

How much did this feature cost?

How much did each agent cost?

Which model gives me the best success rate?
```

Example insight:

```text
Claude subscription
$100/month

Observed Any Code usage
8%

Your OpenAI and Gemini connections handled 92%
of Any Code tasks this month.

Potential redundant spend: $100/month
```

This must be presented as an observation, not an automatic cancellation recommendation unless sufficient usage data exists.

---

# 26. Model Router

The Model Router is one of the core systems.

Routing modes:

```text
Manual
Smart
Subscription First
Balanced
Maximum Quality
Cost Saver
Fastest
Privacy First
Local Only
Custom Policy
```

## 26.1 Routing inputs

```text
Task type
Language
Repository size
Context requirement
Tool requirement
Vision requirement
Model availability
Provider health
Rate limits
Estimated price
Latency history
Past task success
User preference
Workspace privacy rules
Budget
Subscription availability
```

## 26.2 Example

```text
Task
Rename variable across repository

Router
Cheap deterministic task

Selected
Fast low cost model
```

Different task:

```text
Task
Redesign distributed transaction architecture

Router
High reasoning requirement

Selected
High capability reasoning model
```

---

# 27. Fallback Chains

A user can configure:

```text
Primary
Claude

Fallback 1
OpenAI

Fallback 2
Gemini

Fallback 3
Local
```

Automatic failover may occur for:

```text
429
provider outage
model unavailable
budget exhausted
context limit
temporary server error
```

Any Code should preserve task state across provider fallback.

---

# 28. Usage Metering

Every model request must generate a normalized usage event.

```typescript
interface UsageEvent {
  id: string
  workspaceId: string
  sessionId: string
  taskId?: string
  agentId?: string

  provider: string
  model: string

  inputTokens: number
  outputTokens: number
  cachedInputTokens?: number
  reasoningTokens?: number

  latencyMs: number
  estimatedCostUsd: number

  status: "success" | "error"
  timestamp: string
}
```

OpenAI exposes organization usage and cost endpoints that can be used for reconciliation where credentials and account permissions permit it.

Anthropic provides organization cost reporting through its administrative API.

Google provides billing and usage visibility for Gemini API projects, although Any Code must treat the provider's own billing data as authoritative when reconciling costs.

---

# 29. Budget Controls

Users can define:

```text
Daily global budget
Monthly global budget
Per provider budget
Per workspace budget
Per agent budget
Per task budget
Per organization budget
```

Example:

```text
EstateOS

Monthly budget
$100

Used
$63.42

Warning
80%

Hard limit
$100
```

Policies:

```text
warn
ask before exceeding
fallback to cheaper model
fallback to local
stop execution
```

---

# 30. Agent Runtime

Core loop:

```text
Intent
   ↓
Planner
   ↓
Task Graph
   ↓
Context Builder
   ↓
Model Router
   ↓
Agent
   ↓
Tool Invocation
   ↓
Observation
   ↓
Verification
   ↓
Replan if required
   ↓
Complete
```

Agent state machine:

```text
created
planning
waiting
running
blocked
awaiting_approval
verifying
completed
failed
cancelled
```

---

# 31. Multi Agent System

Every agent contains:

```typescript
interface Agent {
  id: string
  role: string
  goal: string

  modelPolicy: string
  permissions: Permission[]

  contextScope: ContextScope
  tokenBudget?: number
  costBudget?: number

  tools: string[]
  skills: string[]

  status: AgentStatus
}
```

Example task:

```text
Build organization invitations
```

Any Code can create:

```text
Architect
Backend Agent
Frontend Agent
Database Agent
Testing Agent
Security Reviewer
Browser Verification Agent
Final Reviewer
```

---

# 32. Agent Isolation

Parallel agents must not edit the same working tree directly.

Use Git worktrees.

Example:

```text
main workspace
      │
      ├── .anycode/worktrees/backend
      ├── .anycode/worktrees/frontend
      └── .anycode/worktrees/tests
```

Agents create isolated changes.

A merge coordinator combines results.

This prevents agents overwriting each other's modifications.

---

# 33. Verification Pipeline

A task is not complete merely because the agent says so.

Default engineering completion:

```text
Code implemented
        +
Compilation succeeds
        +
Lint succeeds
        +
Tests succeed
        +
Runtime starts
        +
Expected behavior verified
        +
Diff reviewed
```

Frontend tasks additionally support:

```text
Browser launch
Console inspection
Network inspection
Responsive viewport checks
Screenshot evidence
DOM assertion
User flow test
```

---

# 34. Integrated Browser Agent

Use Playwright and browser debugging capabilities.

Agent functions:

```text
navigate
click
type
scroll
inspect
screenshot
query DOM
read console
read network
resize viewport
upload
download
authenticate test account
```

Example:

```text
User
Fix mobile signup.

Agent
Starts development server.

Agent
Opens signup page at 390 x 844.

Agent
Runs signup.

Agent
Finds overflow.

Agent
Locates component.

Agent
Modifies CSS.

Agent
Reloads page.

Agent
Repeats signup.

Agent
Captures screenshot.

Agent
Runs tests.

Agent
Reports verified completion.
```

---

# 35. Code Intelligence Engine

Any Code must not send the entire repository blindly to a model.

Repository intelligence:

```text
Filesystem tree
AST index
Symbols
Definitions
References
Imports
Dependencies
Call relationships
Git history
Git blame
Tests
Configuration
Documentation
Semantic search
Full text search
Workspace rules
```

Technology:

```text
tree-sitter
LSP
ripgrep
SQLite FTS5
Git
optional embeddings
```

Do not introduce a graph database into V1 unless repository scale proves relational edge storage insufficient.

---

# 36. Local Index

Suggested SQLite entities:

```text
files
symbols
symbol_edges
imports
references
commits
file_embeddings
search_index
workspace_metadata
```

Incremental indexing should respond to filesystem events.

Only changed files are reprocessed.

---

# 37. Context Builder

Context building stages:

```text
User query
    ↓
Intent extraction
    ↓
Symbol identification
    ↓
Lexical retrieval
    ↓
Structural retrieval
    ↓
Semantic retrieval
    ↓
Git retrieval
    ↓
Reranking
    ↓
Token budgeting
    ↓
Context package
```

The user should be able to inspect:

```text
What files the agent saw
Why they were included
What was excluded
Estimated context token count
```

This becomes the **Context Inspector**.

---

# 38. Memory System

Memory scopes:

```text
Global
Organization
Workspace
Repository
Session
Task
```

Examples:

Global:

```text
Prefer pnpm.
```

Repository:

```text
Never modify production migrations.
```

Task:

```text
Stripe migration approved.
```

Every memory must be:

```text
visible
editable
deletable
exportable
```

No hidden permanent user profiling.

---

# 39. Workspace Rules

Canonical project file:

```text
.anycode/
```

Structure:

```text
.anycode/
    project.md
    architecture.md
    rules.md
    agents.yaml
    tools.yaml
    models.yaml
    permissions.yaml

    skills/
    workflows/
    memory/
```

Example:

```yaml
project:
  package_manager: pnpm

rules:
  require_tests: true
  strict_typescript: true

protected:
  files:
    - .env.production
    - migrations/production/**

approval_required:
  - git.push
  - production.deploy
  - database.drop
```

---

# 40. Import Existing Developer Configuration

Any Code should attempt to reduce migration friction.

Where legally and technically permitted, users can import their own configuration from:

```text
CLAUDE.md
Claude project rules
Gemini project configuration
Codex project instructions
MCP configurations
VS Code settings
Git configuration
environment templates
```

Any Code should normalize these into its internal representation.

Original files remain untouched unless the user explicitly chooses synchronization.

---

# 41. Universal Capability System

MCP, connectors, plugins, skills and native tools must share a common registry.

```text
External Capability
       │
       ▼
Capability Adapter
       │
       ▼
Capability Registry
       │
       ▼
Permission Engine
       │
       ▼
Agent Runtime
```

---

# 42. Capability Interface

```typescript
interface Capability {
  id: string
  name: string

  type:
    | "native"
    | "mcp"
    | "connector"
    | "plugin"
    | "api"
    | "cli"

  inputSchema: JSONSchema
  outputSchema?: JSONSchema

  permissions: Permission[]

  execute(
    input: unknown,
    context: ExecutionContext
  ): Promise<CapabilityResult>
}
```

The agent should not care whether:

```text
github.create_pull_request
```

comes from:

```text
MCP
REST
plugin
native integration
CLI
```

---

# 43. MCP

Any Code must be a first class MCP client.

Support:

```text
Remote HTTP MCP
Local stdio MCP on desktop
OAuth protected MCP
Unauthenticated MCP
MCP discovery
MCP tools
MCP resources
MCP prompts where supported
MCP apps where useful
MCP tasks/extensions as supported
```

The July 2026 MCP specification moved the core toward stateless operation and added further authorization hardening, caching and extension support. Any Code should target the current specification through a versioned MCP adapter rather than embedding assumptions from older versions.

OAuth protected MCP connections must follow the protocol's authorization requirements rather than implementing custom token forwarding.

---

# 44. MCP Manager UX

```text
MCP Servers
──────────────────────────────────────────

GitHub
Connected
14 tools
OAuth
Healthy

PostgreSQL
Local stdio
8 tools
Healthy

Figma
Connected
11 tools
OAuth
Healthy

Custom Dev MCP
localhost:8421
3 tools
Warning
```

Actions:

```text
Add
Connect
Disconnect
Restart
Inspect tools
Inspect schemas
View logs
Change permissions
Remove
```

---

# 45. Connectors

Connector means account connection.

Examples:

```text
GitHub
GitLab
Bitbucket
Linear
Jira
Slack
Notion
Figma
Sentry
Vercel
Cloudflare
Supabase
AWS
GCP
Azure
PostgreSQL
```

Connector stores:

```text
identity
OAuth authorization
scopes
resource access
account metadata
```

Tools depend on connectors.

Credentials are never embedded inside skill files.

---

# 46. Skills

Skills add expertise and operating procedures.

Structure:

```text
skill/
    SKILL.md
    manifest.yaml
    references/
    workflows/
    tests/
```

Manifest:

```yaml
id: react-reviewer
name: React Reviewer

requires:
  tools:
    - code.search
    - code.references
    - filesystem.read
    - git.diff

optional:
  tools:
    - browser

permissions:
  filesystem:
    read: workspace
```

A skill cannot grant itself additional privileges.

---

# 47. Plugins

Plugins can contain executable functionality and therefore require stronger isolation.

Plugin execution options:

```text
WASM sandbox
Separate plugin process
Remote plugin
Signed native plugin
```

Never execute arbitrary third party plugin code inside the primary Any Code process.

Plugin crash:

```text
Plugin terminates
Any Code remains running
```

Resource policy:

```text
CPU limit
RAM limit
execution timeout
network permission
filesystem permission
secret permission
```

---

# 48. Marketplace

Marketplace categories:

```text
Skills
Agents
MCP Servers
Connectors
Plugins
Workflows
Themes
Model Providers
Extensions
```

Extensions are VS Code packages sourced from Open VSX, installed through this same package page
and permission model rather than a second store. Any Code applies their themes, grammars and
language servers; it does not run their JavaScript. See
[ADR 0003](docs/adr/0003-extension-compatibility-and-registry.md).

Marketplace package page displays:

```text
Publisher
Version
Permissions
Required connections
Installation count
Source availability
Signature status
Last update
Compatibility
```

Security review status should be prominent.

---

# 49. Tool Permission System

Permission examples:

```text
filesystem.read.workspace
filesystem.write.workspace
filesystem.write.outside_workspace

shell.execute
shell.admin

network.access
network.domain.github

secret.read.github

git.commit
git.push

database.read
database.write
database.destructive

deployment.staging
deployment.production
```

Policy states:

```text
allow
ask
deny
```

---

# 50. Approval UX

Example:

```text
Approval Required

Agent wants to execute:

npm run migrate:production

Risk
Production database modification

Workspace
EstateOS

[Reject]

[Allow Once]

[Always Allow For This Workspace]
```

High risk operations must never allow an `Always Allow Globally` shortcut.

---

# 51. Shell Security

Commands receive risk classification.

Examples:

Low risk:

```text
ls
git status
npm test
cargo check
```

Medium:

```text
npm install
git commit
docker compose up
```

High:

```text
git push
terraform apply
kubectl delete
database migration
```

Critical:

```text
rm against broad path
production database drop
secret extraction
privilege escalation
```

The model may request execution.

The runtime decides whether execution occurs.

---

# 52. Secrets Protection

Agent access to environment variables is scoped.

The agent should not automatically receive:

```text
complete .env
SSH private keys
cloud root credentials
browser cookies
system keychains
```

Secrets can be injected into a subprocess without exposing their plaintext to the model when possible.

Example:

```text
Agent requests deploy.

Runtime knows VERCEL_TOKEN is required.

Runtime launches deployment process with token.

Model never sees token value.
```

---

# 53. Frontend Information Architecture

Desktop navigation:

```text
Any Code
│
├── Home
├── Workspaces
├── Agents
├── Search
├── Connections
├── Usage
├── Marketplace
└── Settings
```

Workspace:

```text
Workspace
│
├── Code
├── Agent
├── Changes
├── Terminal
├── Problems
├── Browser
└── Tasks
```

---

# 54. Desktop Workbench Layout

Recommended structure:

```text
┌─────────────────────────────────────────────────────────────┐
│ Workspace / Repo / Branch     Command Bar      Model   User │
├───────┬─────────────┬──────────────────────────┬─────────────┤
│       │             │                          │             │
│ Rail  │ Explorer    │ Editor                   │ Agent Dock  │
│       │             │                          │             │
│       │             │                          │             │
├───────┴─────────────┴──────────────────────────┴─────────────┤
│ Terminal | Problems | Output | Browser | Tasks              │
├─────────────────────────────────────────────────────────────┤
│ Git     Branch     Agent Status     Tokens     Cost   Sync   │
└─────────────────────────────────────────────────────────────┘
```

Every panel must be:

```text
resizable
hideable
keyboard accessible
restorable
```

Layouts persist per workspace.

---

# 55. Agent Dock

The right panel is not a generic chat sidebar.

It is an engineering task interface.

Displays:

```text
conversation
current plan
active task
agent state
tool execution
approvals
context
cost
model
verification
artifacts
```

Example:

```text
Implementing organization invitations

✓ Repository analysis
✓ Database design
● Backend implementation
● Frontend implementation
○ Tests
○ Browser verification
○ Review

2 agents running

Cost
$0.38
```

---

# 56. Task Timeline

Every important action becomes an event.

```text
10:42:01  Task started
10:42:03  Repository context loaded
10:42:07  Planner completed
10:42:09  Backend agent created
10:42:09  Frontend agent created
10:43:14  Backend modified 4 files
10:43:47  Test failed
10:44:03  Backend agent replanning
10:44:51  Tests passed
10:45:20  Browser verification started
```

The user can expand any event.

---

# 57. Monaco Editor

Desktop editor requirements:

```text
Syntax highlighting
LSP completion
Go to definition
References
Rename
Code actions
Diagnostics
Breadcrumbs
Outline
Multiple tabs
Split editor
Diff editor
Search
Replace
Multi cursor
Minimap optional
Formatting
Git indicators
```

Agents must not silently overwrite a currently edited unsaved buffer.

---

# 58. Changes Review

Changes view:

```text
Files changed                 14

src/api/team.ts                +84  -12
src/components/Invite.tsx      +61   -4
db/migrations/...              +28    0
tests/team.test.ts             +72    0
```

Actions:

```text
Accept file
Reject file
Accept hunk
Reject hunk
Edit
Ask agent about change
Revert
Stage
Commit
```

---

# 59. Universal Command Bar

Shortcut:

```text
Cmd/Ctrl + K
```

Example commands:

```text
Ask Any Code
Open file
Run task
Change model
Run command
Connect provider
Install skill
Open browser
View usage
Start cloud agent
Create branch
```

Natural language commands supported.

---

# 60. Command Palette

Shortcut:

```text
Cmd/Ctrl + Shift + P
```

Designed for deterministic product actions.

The command bar and command palette are conceptually distinct.

---

# 61. Home Screen

Home should not be a marketing screen.

It should immediately support work.

Sections:

```text
New Task
Recent Workspaces
Running Agents
Recent Sessions
Usage Snapshot
Connection Health
```

Example:

```text
Good morning.

What are you building?

[ Ask Any Code... ]

Running
2 agents

Recent
EstateOS
Heptagram
Any Code

AI spend this month
$37.18
```

---

# 62. Onboarding

First launch should take less than several minutes for a technically capable user.

Flow:

```text
Welcome
   ↓
Any Code sign in or Local Only
   ↓
Theme
   ↓
Connect AI provider
   ↓
Select repository
   ↓
Import existing configuration
   ↓
Workspace scan
   ↓
Permission profile
   ↓
First task
```

A user must be able to skip account creation and use local mode.

Cloud features can require Any Code account authentication later.

---

# 63. Provider Onboarding

Screen:

```text
Choose your intelligence

OpenAI
[Connect]

Anthropic
[Connect]

Gemini
[Connect]

OpenRouter
[Connect]

Ollama
[Detect]

OpenAI Compatible
[Configure]
```

A user only needs one provider to start.

Do not force five integrations during onboarding.

---

# 64. Theme System

Themes:

```text
System
Light
Dark
High Contrast
```

## 64.1 Dark palette

```text
Canvas          #080A0D
Surface         #0F1318
Elevated        #151A21
Border          #232B34

Primary Text    #F5F7FA
Secondary Text  #98A4B1
Muted Text      #6F7A86

Accent          #7C5CFF
Success         #34D399
Warning         #FBBF24
Danger          #FB7185
Info            #38BDF8
```

## 64.2 Light palette

```text
Canvas          #F6F7F9
Surface         #FFFFFF
Elevated        #FAFBFC
Border          #DDE2E7

Primary Text    #101317
Secondary Text  #5E6873
Muted Text      #84909C

Accent          #6746E8
Success         #087F5B
Warning         #A15C00
Danger          #C93450
Info            #0875BE
```

Accent use should remain restrained.

The application must not become a purple neon AI dashboard.

---

# 65. Typography

UI:

```text
Inter
```

Code:

```text
JetBrains Mono
```

Recommended base UI size:

```text
13px to 14px
```

The interface should optimize density without becoming visually exhausting.

---

# 66. Spacing

Use 4 px base spacing.

Primary increments:

```text
4
8
12
16
20
24
32
40
48
```

Avoid arbitrary spacing values.

---

# 67. Radius

```text
Small control     6px
Standard control  8px
Panel             10px
Modal             12px
```

Avoid excessive pill shaped controls.

---

# 68. Motion

Motion communicates hierarchy or state only.

Recommended durations:

```text
fast     100 to 120 ms
normal   150 to 180 ms
panel    180 to 220 ms
```

Respect operating system reduced motion settings.

---

# 69. Accessibility

Target:

**WCAG 2.2 AA**

Requirements:

```text
Keyboard navigation
Visible focus states
Screen reader labels
Sufficient contrast
Reduced motion
Non color status indicators
Zoom compatibility
Accessible modals
Accessible menus
```

---

# 70. Performance Targets

Desktop targets:

| Metric                         |                   Target |
| ------------------------------ | -----------------------: |
| Warm application launch        |                ≤ 1.5 sec |
| Main navigation response       |                 ≤ 100 ms |
| Keyboard input latency         |        ≤ 16 ms perceived |
| Terminal input latency         | ≤ 16 ms local processing |
| Incremental index changed file |          ≤ 1 sec typical |
| Idle memory target             | < 300 MB where realistic |
| UI animation                   |                   60 FPS |
| Crash free sessions            |                  > 99.5% |
| Agent event streaming          |            Near realtime |

Large repository operations must run outside the UI thread.

---

# 71. Backend Architecture

Initial cloud architecture:

```text
                     Any Code Clients
                            │
                            ▼
                       API Gateway
                            │
                            ▼
                        Rust API
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
   PostgreSQL             Redis             Object Store
        │                   │                   │
        ▼                   ▼                   ▼
 Persistent state       ephemeral cache    artifacts
                            │
                            ▼
                         NATS
                            │
                            ▼
                      Worker Runtime
```

---

# 72. Cloud Stack

```text
Rust
Axum
Tokio
SQLx

PostgreSQL

Redis

NATS JetStream

S3 compatible object storage

Docker

Caddy or Traefik

OpenTelemetry

Prometheus

Grafana

Loki
```

Do not introduce Kubernetes for the first production deployment unless scale requires it.

---

# 73. VPS Deployment

The provided `.env.vps` is treated as deployment configuration.

It must never be committed to Git.

Expected staging Docker services:

```text
anycode-api
postgres
redis
nats
object-storage if needed
identity
reverse-proxy
observability
```

Required deployment workflow:

```text
Repository
    ↓
CI
    ↓
Build
    ↓
Tests
    ↓
Container images
    ↓
Staging VPS
    ↓
Health checks
    ↓
Integration tests
```

Generate:

```text
.env.example
```

containing only variable names and safe examples.

---

# 74. Cloud Database

Core entities:

```text
users
organizations
organization_members

devices

workspaces
workspace_members
repositories

sessions
tasks
agents
agent_events

provider_connections
provider_connection_metadata

models
model_capabilities

usage_events
budgets

connectors
connector_installations

capabilities
plugins
skills
workflows

approvals
audit_events

artifacts
cloud_runs
```

Do not store raw provider keys in normal relational columns.

---

# 75. Local Database

Local SQLite stores:

```text
workspace settings
repository index
symbol index
local sessions
local agent events
local usage events
cached model catalog
cached capability manifests
MCP server metadata
workspace memory
```

Local only mode must function when Any Code Cloud is unreachable.

---

# 76. Cloud Synchronization

Syncable:

```text
workspace metadata
sessions
tasks
usage metadata
settings
skills
workflows
agent events
team policies
```

Not synced by default:

```text
repository source
terminal history containing secrets
raw environment variables
local provider keys
unapproved private artifacts
```

---

# 77. Event Architecture

Everything meaningful generates a typed event.

Examples:

```text
session.created
task.created
task.started

agent.created
agent.started
agent.completed
agent.failed

model.request.started
model.request.completed
model.request.failed

tool.requested
tool.approved
tool.executed
tool.failed

file.read
file.modified

test.started
test.failed
test.passed

browser.started
browser.verified

git.commit.created

task.completed
```

Benefits:

```text
replay
debugging
sync
analytics
audit
recovery
timeline UI
```

---

# 78. Cloud API

Suggested routes:

```text
POST   /v1/auth/device
GET    /v1/me

GET    /v1/workspaces
POST   /v1/workspaces
GET    /v1/workspaces/:id

GET    /v1/sessions
POST   /v1/sessions
GET    /v1/sessions/:id

POST   /v1/tasks
GET    /v1/tasks/:id
POST   /v1/tasks/:id/cancel

GET    /v1/connections
POST   /v1/connections
DELETE /v1/connections/:id

GET    /v1/models

GET    /v1/usage
GET    /v1/budgets
PUT    /v1/budgets/:id

GET    /v1/skills
POST   /v1/skills

GET    /v1/capabilities

GET    /v1/events/stream
```

Streaming:

```text
SSE initially
WebSocket where bidirectional realtime control is required
```

---

# 79. Rust Provider Interface

```rust
#[async_trait]
pub trait ModelProvider {
    fn id(&self) -> &'static str;

    async fn models(
        &self
    ) -> Result<Vec<ModelDefinition>>;

    async fn stream(
        &self,
        request: ModelRequest,
    ) -> Result<ModelStream>;

    async fn health(
        &self
    ) -> Result<ProviderHealth>;

    fn capabilities(
        &self
    ) -> ProviderCapabilities;
}
```

Provider implementation details may never leak into the agent orchestration layer.

---

# 80. Normalized Model Request

```typescript
interface ModelRequest {
  model: string

  messages: Message[]

  tools?: ToolDefinition[]

  temperature?: number

  reasoning?: {
    effort?: "low" | "medium" | "high"
  }

  responseFormat?: JSONSchema

  metadata: {
    sessionId: string
    taskId?: string
    agentId?: string
  }
}
```

Provider adapters translate normalized requests into provider specific calls.

---

# 81. Agent Tool Runtime

Core native tools:

```text
filesystem.read
filesystem.write
filesystem.patch

code.search
code.definition
code.references

shell.execute
shell.spawn
shell.kill

git.status
git.diff
git.commit
git.branch
git.worktree

browser.open
browser.click
browser.type
browser.inspect
browser.screenshot

http.request

test.run

build.run

lint.run

mcp.call
```

---

# 82. Terminal

Use a native PTY backend.

Frontend:

```text
xterm.js
```

Backend:

```text
Rust portable PTY
```

Features:

```text
multiple terminals
shell profile detection
working directory
search
copy
links
exit codes
agent terminal
user terminal
command history
```

Agent commands must be visually distinguishable from user commands.

---

# 83. Git

Requirements:

```text
status
stage
unstage
diff
commit
branch
checkout
worktree
merge
conflicts
history
blame
remote information
```

Git push defaults to approval required.

---

# 84. Cloud Agents

Future architecture:

```text
Task
    ↓
Scheduler
    ↓
Ephemeral Workspace
    ↓
Clone repository
    ↓
Inject approved credentials
    ↓
Agent runtime
    ↓
Tests
    ↓
Artifact
    ↓
Pull request
    ↓
Destroy workspace
```

Initially use containers or isolated virtual machines.

Firecracker becomes relevant only when operational scale warrants it.

---

# 85. Monitoring and Observability

Monitor:

```text
API latency
model latency
token usage
provider errors
agent failures
tool failures
MCP failures
plugin crashes
desktop crashes
sync errors
database latency
queue latency
cloud worker utilization
```

Tracing:

```text
OpenTelemetry
```

Metrics:

```text
Prometheus
```

Visualization:

```text
Grafana
```

Logs:

```text
Loki
```

Error reporting may use a hosted product such as Sentry if preferred.

---

# 86. Product Analytics

Analytics must answer:

```text
How many users complete onboarding?

How many connect at least one provider?

How many create their first task?

What percentage of tasks complete successfully?

Which model providers are used?

How often does automatic routing override defaults?

How often are agent changes accepted?

How often are they reverted?

How many users use MCP?

How many users return weekly?

What causes abandonment?
```

Source code and prompts must not be included in analytics by default.

---

# 87. Usage Dashboard

Main metrics:

```text
AI Spend
Tokens
Requests
Agent Runs
Success Rate
Average Task Cost
Average Task Duration
Provider Distribution
Model Distribution
Workspace Distribution
```

Views:

```text
Today
7 days
30 days
Current billing month
Custom
```

---

# 88. Usage Detail

Example:

```text
EstateOS

Cost this month
$31.74

OpenAI
$14.29

Anthropic
$11.02

Gemini
$6.43

Most expensive task
Implement property matching
$2.18

Most expensive agent
Architecture reviewer
$4.72 total
```

---

# 89. Provider Health

Status example:

```text
OpenAI
Healthy
p50 1.4 sec
p95 3.9 sec
Failures 0.8%

Anthropic
Degraded
p50 3.1 sec
p95 9.7 sec
Failures 7.2%
```

Router may automatically avoid unhealthy providers according to policy.

---

# 90. Security Model

Threats include:

```text
Malicious repository instructions
Prompt injection
Malicious MCP server
Malicious plugin
Secret exfiltration
Destructive shell execution
Compromised model response
Dependency attack
OAuth token theft
Cloud worker escape
Cross workspace access
```

Every untrusted data source must be tagged.

Repository text is data.

MCP output is data.

Browser content is data.

None receives authority simply because an LLM reads instructions from it.

---

# 91. Data Privacy Modes

Workspace privacy:

```text
Standard
Private
Local Only
Enterprise Restricted
```

Local Only:

```text
No cloud sync
No Any Code cloud processing
No remote telemetry beyond opt in crash data
Only explicitly configured model endpoints
```

Privacy First routing can require local models.

---

# 92. Audit Log

Enterprise audit events:

```text
login
provider connected
provider disconnected
secret changed
agent started
permission granted
production command approved
MCP installed
plugin installed
workflow modified
budget changed
organization policy changed
```

---

# 93. Auto Updates

Desktop builds require signed updates.

Channels:

```text
stable
beta
nightly
```

Enterprise administrators may pin versions.

---

# 94. Repository Structure

Recommended monorepo:

```text
any-code/
│
├── apps/
│   ├── desktop/
│   ├── web/
│   └── mobile/
│
├── crates/
│   ├── anycode-core/
│   ├── anycode-agent/
│   ├── anycode-models/
│   ├── anycode-context/
│   ├── anycode-code-intelligence/
│   ├── anycode-tools/
│   ├── anycode-mcp/
│   ├── anycode-git/
│   ├── anycode-terminal/
│   ├── anycode-browser/
│   ├── anycode-security/
│   ├── anycode-plugin-runtime/
│   ├── anycode-usage/
│   └── anycode-sync/
│
├── services/
│   ├── api/
│   ├── worker/
│   └── identity/
│
├── packages/
│   ├── ui/
│   ├── protocol/
│   ├── sdk-typescript/
│   └── design-tokens/
│
├── infrastructure/
│   ├── docker/
│   ├── staging/
│   └── production/
│
├── docs/
│
└── .github/
```

If the provided repository already contains a viable structure, do not restructure it merely to match this diagram.

Adapt rather than rewrite unnecessarily.

---

# 95. Testing Strategy

## Unit tests

Required for:

```text
router
usage calculator
permission engine
context selection
provider adapters
event reducer
Git operations
capability registry
```

## Integration tests

Required for:

```text
provider connections
MCP
terminal
Git
database
cloud API
authentication
sync
```

## End to end tests

Use Playwright.

Critical flows:

```text
onboarding
connect provider
open repository
send task
agent modifies code
diff appears
run test
approve action
view usage
restart application
resume session
```

---

# 96. Provider Contract Tests

Every provider adapter must pass a standard contract suite:

```text
authentication
model discovery
streaming
cancellation
tool calls
usage parsing
error normalization
rate limit handling
timeout
fallback
```

This prevents provider specific behavior leaking throughout the product.

---

# 97. Fault Testing

Test:

```text
provider outage
internet disconnect
database unavailable
MCP crash
plugin crash
terminal child crash
agent process crash
cloud sync interruption
rate limit
token expiration
disk full
repository deleted
Git conflict
```

The application should fail predictably rather than corrupt state.

---

# 98. Implementation Phases

## Phase 0: Foundation

Deliver:

```text
monorepo
Tauri shell
React shell
Rust core
CI
formatting
linting
testing
SQLite
event system
design system
dark/light themes
```

Exit condition:

Application launches correctly on Windows and macOS.

---

## Phase 1: Workbench

Deliver:

```text
workspace selection
file explorer
Monaco
tabs
terminal
Git status
diff
command palette
settings
```

Exit condition:

Developer can realistically use Any Code as a lightweight coding environment.

---

## Phase 2: AI Provider Layer

Deliver:

```text
Provider abstraction
OpenAI
Anthropic
Gemini
OpenRouter
Ollama
streaming
model selector
secure credential storage
usage events
```

Exit condition:

Same chat task can switch between providers without changing the rest of the application.

---

## Phase 3: Agent Runtime

Deliver:

```text
planner
task state machine
tool calls
filesystem tools
terminal tools
Git tools
approval system
event timeline
verification
```

Exit condition:

Agent can implement and verify a simple repository task.

---

## Phase 4: Code Intelligence

Deliver:

```text
tree-sitter
ripgrep
LSP integration
symbol index
SQLite FTS
context builder
context inspector
memory
```

Exit condition:

Agents retrieve targeted repository context instead of broad file dumping.

---

## Phase 5: Multi Agent

Deliver:

```text
task DAG
subagents
Git worktrees
parallel execution
merge coordinator
agent dashboard
```

Exit condition:

Frontend and backend agents can work concurrently without corrupting the workspace.

---

## Phase 6: Browser Verification

Deliver:

```text
Playwright
browser panel
screenshots
console
network
DOM tools
responsive verification
```

Exit condition:

Frontend agents can prove functionality.

---

## Phase 7: Capability Platform

Deliver:

```text
MCP
connectors
skills
plugin host
capability permissions
installation UX
```

Exit condition:

External capabilities operate through a single registry.

---

## Phase 8: Subscription Intelligence

Deliver:

```text
usage dashboard
budgets
provider analytics
model analytics
subscription ledger
routing policies
cost insights
```

Exit condition:

User can understand where AI money is going and change behavior accordingly.

---

## Phase 9: Cloud

Deliver:

```text
Any Code identity
cloud API
PostgreSQL
sync
devices
remote sessions
web dashboard
```

Exit condition:

Sessions continue across authorized devices.

---

## Phase 10: Mobile

Deliver:

```text
iOS
Android
task management
diff review
approvals
usage
connections
notifications
cloud agent control
```

Exit condition:

Developer can supervise real work without a laptop.

---

# 99. V1 Success Criteria

A V1 release is successful when a new user can:

```text
Install Any Code
Open repository
Connect one provider
Ask for a feature
Watch agent execution
Review commands
See modified files
Run tests
Inspect diff
See task cost
Restart Any Code
Resume session
```

without requiring manual configuration outside Any Code except provider authorization.

---

# 100. Quality Bar

The product must not ship merely because all intended screens exist.

Required quality:

```text
No obvious placeholder UI
No fake metrics
No fake provider quotas
No fake agent state
No fake terminal
No mock production flows
No hardcoded demo records
No exposed secrets
No unhandled loading states
No unhandled empty states
No silent failures
No blocking filesystem work on UI thread
No unbounded agent execution
```

---

# 101. Definition of Done

A feature is done when:

```text
Functional implementation complete

UI matches design system

Dark mode verified

Light mode verified

Keyboard navigation verified

Loading state implemented

Empty state implemented

Error state implemented

Permissions implemented

Analytics implemented where applicable

Unit tests pass

Integration tests pass

End to end path passes

No secrets logged

Documentation updated
```

---

# 102. Repository Implementation Contract

When an implementation repository is supplied, execution should proceed as follows:

```text
1. Inspect complete repository.

2. Determine current stack and architecture.

3. Preserve good existing architecture.

4. Create an implementation parity checklist against this PRD.

5. Identify missing infrastructure.

6. Implement foundational architecture first.

7. Never insert fake production data merely to make UI look complete.

8. Build real provider abstractions before provider specific UI.

9. Build permission boundaries before unrestricted agent execution.

10. Use provided .env.vps only for staging infrastructure.

11. Never commit .env.vps.

12. Generate a safe .env.example.

13. Run local tests.

14. Build desktop packages.

15. Deploy required backend services to staging.

16. Run staging integration tests.

17. Run end to end flows.

18. Record unresolved limitations explicitly.
```

---

# 103. Architectural Rules

The implementation must follow these invariants:

```text
UI never owns secrets.

LLM never owns permissions.

Provider adapter never owns orchestration.

Agent never bypasses capability runtime.

Skills never grant permissions.

Plugins never execute in the main process.

Cloud is never required for Local Only mode.

Model specific code stays inside provider adapters.

Repository indexing stays local by default.

Every expensive model request creates usage telemetry.

Every privileged action is auditable.

Every agent task is cancellable.

Every long running task is resumable where possible.

Every automated completion is verifiable.
```

---

# 104. Critical Product Differentiator

The biggest mistake would be marketing Any Code as:

> Another AI code editor that supports several models.

That market is crowded and easy to copy.

The stronger product is:

> **Any Code is the universal execution environment for AI software engineering.**

The system owns:

```text
context
orchestration
permissions
tools
verification
usage intelligence
provider routing
memory
integrations
workflow
```

Models remain replaceable.

That is the architectural and commercial foundation on which Any Code can expand beyond a coding assistant into a real engineering platform.

---

# 105. Product North Star

The final experience should eventually be this simple:

```text
Open Any Code.

Open repository.

Describe outcome.

Any Code decides:
what context is required
what models should work
what agents should exist
what tools they need
what can run automatically
what requires approval
how to verify the result
how much it costs

Developer reviews the result.

Done.
```

Everything underneath can be extremely sophisticated.

The user's interaction with it should not be.
