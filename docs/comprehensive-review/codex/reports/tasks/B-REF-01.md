# B-REF-01: Mature implementation reference matrix

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both repositories clean at inspection time

## Question

What current, first-party cross-system patterns should constrain architecture, state,
Plan, Subagent, event, permission, Skill/Plugin, and recovery findings?

## Scope

Official documentation and first-party repositories for Claude Code, OpenAI Codex,
Cursor, Devin, and Temporal. Each lookup records its URL, access date, supported claim,
and evidence limit. The comparison uses behavior published as of 2026-08-12.

## Out Of Scope

- Recommending source changes or selecting a concrete EKO implementation.
- Reverse engineering closed-source internal state machines.
- Treating Temporal's infrastructure/storage architecture as a required EKO dependency.
- Secondary articles, social posts, vendor comparisons, and marketing inference.

## Inputs

- Root `AGENTS.md`.
- Shared `README.md`, `REPORTING.md`, task card in `TASKS.md`.
- `codex/README.md`.
- No dependency reports; this task has none.

## Layering Decision

| Classification | Reference constraint |
|---|---|
| Generic mechanism | Stable run/turn/item/attempt identity, terminal lifecycle events, cancellation/retry/idempotence, explicit Subagent context/result handoff, and recoverable event/history authority are reusable framework mechanisms. |
| EKO product policy | Whether an action prompts, which paths/commands are trusted, reviewer/worktree policy, local UI presentation, and user-interactive terminal/MCP behavior are EKO decisions. Cloud VM, enterprise RBAC, and PR auto-approval are not generic defaults. |
| Adapter boundary | Application adapters should project framework identities/events losslessly and inject EKO policy. They must not own a second scheduler, retry loop, or Plan approval state machine. |

This is external research, so repository duplicate search is not applicable. Downstream
tasks must still perform their required whole-repository searches before recommending
new types or moving ownership.

## Current Path

The mature systems do not expose one universal agent state machine. They expose
separable artifacts and protocols:

1. An instruction/artifact layer: editable Plan documents, Skills, rules, manifests.
2. An execution layer: root session/turn plus identified Subagent/tool attempts.
3. An event/projection layer: start/update/terminal events and UI/JSONL projections.
4. A policy layer: allow/ask/deny or mode selection.
5. An enforcement layer: sandbox/writable roots/network limits where applicable.
6. A recovery layer: stable IDs plus persisted history/checkpoints and replay/resume.

### Cross-System Matrix

| Topic | Claude Code | OpenAI Codex | Cursor | Devin | Temporal | Constraint for review |
|---|---|---|---|---|---|---|
| Plan | Internal representation not confirmed | Todo is a typed event item/projection; no approval-state evidence | Editable saved Markdown/chat artifact, explicitly built later | Public artifact/state not confirmed | Not an agent Plan system | Treat Plan as artifact/projection; do not infer approval runtime states. |
| Subagent | Plugin agent definitions and tool profiles are first-party | Persisted Subagent lineage/session identity | Clean context, explicit prompt/result, foreground/background, tool/model policy | Own context/inference, foreground/background, parent summary | Child Workflow has explicit start/result identity | Require identity, lineage, context handoff, terminal result, cancellation and attempt ownership. |
| Events | Hook event surface includes tool/Subagent/session lifecycle | Typed thread/turn/item JSONL with IDs and terminal states | Cloud run events/diagnostics and lifecycle hooks | Session/message API and completion notification | Event History is durable authority | Keep canonical events typed and replayable; UI is a projection. |
| Permission | Managed allow/ask/deny and Bash approval examples | Approval policy is separate from sandbox policy | Scoped agent/review policies | Tiered Auto/Prompt modes and scoped rules | Platform authorization is a different domain | Separate automated-action policy from user-interactive feature availability. |
| Sandbox/security | Bash-only sandbox boundary is explicit | Read-only/workspace-write/full-access process/filesystem policy | Cloud VMs, branches, network/secrets controls | OS writable/read roots; fail closed when requested sandbox unavailable | Durable platform security not a desktop model | Apply isolation to execution paths; do not copy cloud/multi-tenant gates into EKO. |
| Skills/Plugins | Manifest plus separately owned agents/skills/hooks/MCP | Skill roots/snapshots and Plugin-contributed Skills/MCP | Versioned on-demand Skills, hooks, MCP | Skills bundle prompts/tools/permissions/workflows; Plugins govern bundles | Plugins/interceptors exist but are not agent Skills | Preserve source ownership, discovery precedence, enable/disable and cleanup boundaries. |
| Recovery | Not established by inspected public core | Rollout history resume/fork; root/Subagent persisted identity | Cloud environment snapshots and run diagnostics | Stable session automatically resumes on message when suspended | Deterministic replay from persisted Event History | Stable identity plus persisted authoritative history; reject stale attempt results. |

## Findings

No repository findings. This task establishes external constraints and uncertainty rather
than identifying a defect in current `echo-agent` or EKO code.

The following design risks should be used as falsifiable hypotheses in later tasks, not
copied as findings without current-code evidence:

- A Plan approval state machine that blocks the runtime is likely duplicate authority.
- A Subagent lacking stable identity, lineage, bounded context handoff, terminal result,
  or cancellation ownership diverges from multiple mature systems.
- A UI status or relational projection used as canonical recovery state is weaker than
  persisted event/history authority.
- Approval policy and sandbox enforcement collapsed into one permission mode obscure
  whether a decision is product policy or technical isolation.
- Skills/Plugins without source-scoped registration and unload cleanup risk leaked or
  stale capabilities.
- Retrying a whole orchestration for a failed external/tool operation risks repeating
  committed effects; attempt-scoped retry/idempotence is the mature pattern.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Official OpenAI search attempt | yes | failed (superseded by first-party repo evidence) | [V00](../validations/B-REF-01/V00-01.md) |
| V01 | Claude Plugin/Skill/agent lifecycle | yes | passed | [V01](../validations/B-REF-01/V01-01.md) |
| V02 | Claude permission/sandbox separation | yes | passed | [V02](../validations/B-REF-01/V02-01.md) |
| V03 | Codex event lifecycle | yes | passed | [V03](../validations/B-REF-01/V03-01.md) |
| V04 | Codex persistence and resume | yes | passed | [V04](../validations/B-REF-01/V04-01.md) |
| V05 | Codex approval/sandbox separation | yes | passed | [V05](../validations/B-REF-01/V05-01.md) |
| V06 | Codex Subagent/Skill/Plugin boundaries | yes | passed | [V06](../validations/B-REF-01/V06-01.md) |
| V07 | Cursor Plan artifact | yes | passed | [V07](../validations/B-REF-01/V07-01.md) |
| V08 | Cursor Subagent model | yes | passed | [V08](../validations/B-REF-01/V08-01.md) |
| V09 | Cursor cloud execution/extensions/policy | yes | passed | [V09](../validations/B-REF-01/V09-01.md) |
| V10 | Devin Subagent model | yes | passed | [V10](../validations/B-REF-01/V10-01.md) |
| V11 | Devin permission/sandbox model | yes | passed | [V11](../validations/B-REF-01/V11-01.md) |
| V12 | Devin Skills/session recovery | yes | passed | [V12](../validations/B-REF-01/V12-01.md) |
| V13 | Temporal event history/replay | yes | passed | [V13](../validations/B-REF-01/V13-01.md) |
| V14 | Temporal retry/child execution scope | yes | passed | [V14](../validations/B-REF-01/V14-01.md) |
| V15 | Claude internal Plan representation | yes | inconclusive | [V15](../validations/B-REF-01/V15-01.md) |
| V16 | Devin Plan artifact/state | yes | inconclusive | [V16](../validations/B-REF-01/V16-01.md) |
| V17 | Cross-system convergence | yes | passed | [V17](../validations/B-REF-01/V17-01.md) |
| V18 | Report completeness and isolation | yes | passed | [V18-01](../validations/B-REF-01/V18-01.md), [V18-02](../validations/B-REF-01/V18-02.md) |
| V19 | Primary source/convergence/uncertainty acceptance | yes | passed | [V19](../validations/B-REF-01/V19-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: Claude Code Plan mode is prompt injection rather than runtime enforcement | stale/unconfirmed as stated | V15 could not establish the closed-source internal representation; the architectural direction is independently supported by Cursor V07. |
| `AGENTS.md`: Codex exec uses item in-progress/completed/failed events | stale in exact naming | V03 confirms start/update/completed with failure inside item status; `turn.failed` is explicit, but there is no literal `item.failed` envelope at reviewed commit. |
| `AGENTS.md`: mature systems treat Plan as editable artifact | current for Cursor; unconfirmed for Devin/Claude internal form | V07, V15, V16. |
| `AGENTS.md`: permissions should distinguish automated action from interaction | current as a project decision and consistent with mature policy/enforcement separation | V02, V05, V11, V17. |

## Coverage And Uncertainty

- `developers.openai.com` returned HTTP 403 and the OpenAI search tool returned 404;
  Codex claims therefore use official open-source commit `2230d64`.
- `code.claude.com` timed out in browser/curl; Claude claims are limited to official
  repository commit `681a8be`. Internal Plan and recovery details remain unknown.
- Cursor and Devin are primarily first-party product documentation; their closed-source
  implementation details are not inferred.
- Product documentation changes over time. Any architecture decision made after a major
  vendor release should revalidate only the smallest relevant report.
- Temporal validates durable orchestration principles, not EKO's storage technology or
  product threat model.

## Handoff

- Downstream tasks may rely on the matrix and V17 convergence, but must link the
  topic-specific validation for precise claims.
- Treat V15/V16 as explicit evidence limits, not negative findings about those products.
- The report becomes stale if any cited official page changes materially, or if the
  reviewed Codex/Claude commits are no longer representative of current contracts.
- Primary consumers: `F-TSK-*`, `F-SUB-*`, `F-HITL-01`, `F-SKL-01`, `F-PLG-01`,
  `F-SEC-01`, `A-TSK-*`, `A-STATE-01`, `A-PLG-01`, `X-TSK-01`, `X-EVT-01`, and the
  final roadmap synthesis.
