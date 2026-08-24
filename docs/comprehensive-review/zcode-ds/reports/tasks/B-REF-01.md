# B-REF-01: Mature implementation reference matrix

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (external reference task)
> `echo-agent-cli` commit: not-applicable
> Worktree state: not-applicable

## Question

What current cross-system patterns should constrain architecture, state,
Plan, Subagent, event, permission, skill/plugin, and recovery findings?

## Scope

- Claude Code: plan mode internals (community reverse-engineering doc with
  source anchors), skills/plugin marketplace model (cited in EKO
  MASTER-PLAN).
- OpenAI Codex: event stream contract (`exec_events.rs`), durable rollout,
  sandbox × approval policy model.
- Cursor / Devin: plan-then-execute, background agents, review agent,
  plan revision.
- Temporal: durable execution, event-history replay, side effects,
  signal ordering.

## Out Of Scope

- Fresh deep reading of Claude Code skills/hooks chapters (repo structure
  confirmed; content-level claims for skills rely on the marketplace doc
  citation in the CLI MASTER-PLAN and will be re-verified if a task needs
  them).
- Full Codex `exec_events.rs` variant enumeration (F-* tasks reference the
  matrix, not the source).
- Per-product version pinning.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (B-REF-01 card), `zcode-ds/README.md`.
- No code review was performed; this task is pure external reference.

## Layering Decision

- Generic mechanism: the converged patterns (artifact plans, event-sourced
  recovery, typed terminal events, sandbox/approval separation, separate
  review) are framework-level guidance.
- EKO product policy: where EKO chooses to differ (e.g., file-only
  persistence instead of SQLite indexing), the difference must be a
  conscious local-product decision.
- Adapter boundary: none.
- Duplicate search: n/a (external references; each lookup records its own
  source and limits).

## Current Path

Five validation reports record per-system evidence (V01-V04) and the
convergence matrix (V05). The matrix constrains downstream findings:
plans as editable artifacts with permission-gated approval; append-only
event record as recovery authority with payload-before-event ordering;
typed terminal events; sandbox/approval policy separation; review separate
from execution; subagents excluded from interactive approval where
possible.

## Findings

### B-REF-01-P3-01: "Safe point" terminology diverges from the industry (informational)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: [V03](../validations/B-REF-01/V03-01.md) (Temporal: no
  first-class "safe point"; closest analogues are event-history append and
  workflow-task completion points); Codex payload-before-event ordering
  ([V02](../validations/B-REF-01/V02-01.md))
- Reachability: EKO's MASTER-PLAN uses "safe points" for RuntimeDagExecutor
  revision reloads (`echo-agent-cli/docs/MASTER-PLAN.md:217-227`); the
  framework uses the term in `echo-orchestration` task docs.
- Expected invariant: terminology should map to a mechanism the industry
  recognizes.
- Observed behavior: "safe point" is a local term; the industry mechanism
  it refers to (durable event append points) is convergent.
- Impact: none functionally; terminology friction in reviews and docs.
- Root cause: independent design vocabulary.
- Direction: keep the term but document it as "revision safe point =
  event-history append point" in the task docs; no code change required.
- Regression validation: n/a.
- Validation reports: [V02](../validations/B-REF-01/V02-01.md),
  [V03](../validations/B-REF-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Claude Code plan-mode lookup | yes | passed | [V01](../validations/B-REF-01/V01-01.md) |
| V02 | Codex event stream/sandbox lookup | yes | passed | [V02](../validations/B-REF-01/V02-01.md) |
| V03 | Temporal durable-execution lookup | yes | passed | [V03](../validations/B-REF-01/V03-01.md) |
| V04 | Cursor/Devin plan-then-execute lookup | yes | passed | [V04](../validations/B-REF-01/V04-01.md) |
| V05 | Cross-system convergence report | yes | passed | [V05](../validations/B-REF-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md` lesson: Claude Code plan mode is prompt/artifact-driven, not a state machine | current | [V01](../validations/B-REF-01/V01-01.md) |
| CLI MASTER-PLAN: skills updates follow "Claude Code's explicit marketplace refresh model" (code.claude.com/docs/en/plugin-marketplaces) | current (cited) | CLI MASTER-PLAN.md:239-243; [V01](../validations/B-REF-01/V01-01.md) |
| Root MASTER-PLAN: `events.jsonl` is the recovery authority | current (matches Codex rollout + Temporal event history) | [V02](../validations/B-REF-01/V02-01.md), [V03](../validations/B-REF-01/V03-01.md) |

## Coverage And Uncertainty

- Official Anthropic/Cursor/Devin docs were not directly fetchable in this
  session; community reverse-engineering (with source anchors) and secondary
  coverage were used, and limits are recorded per lookup.
- Codex event naming: the task card's remembered `item.in_progress/
  completed/failed` wording was not found verbatim; observed names are
  `item.started`, `item.agentMessage/delta`, `thread/started`,
  `turn/completed`; authoritative list is in `exec_events.rs`.
- Skills/plugin lookup for Claude Code was not a full chapter read; the
  marketplace-refresh model is cited from the EKO MASTER-PLAN.

## Handoff

- Downstream tasks (F-HITL-01, F-TSK-03, F-SUB-01, A-TSK-04, A-HITL-01,
  A-INT-01, X-EVT-01, X-AUT-01, S-RDM-01) should treat the V05 matrix as
  the constraint set when judging architecture decisions.
- The matrix is the evidence base for "plan as artifact", "event-sourced
  recovery", "typed terminals", and "review separate from execution" claims
  in synthesis.
- This report becomes stale when the referenced systems materially change
  their documented models (each lookup records its access date).
