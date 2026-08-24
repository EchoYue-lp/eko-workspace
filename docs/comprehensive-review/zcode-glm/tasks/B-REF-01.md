# B-REF-01: Mature implementation reference matrix

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (external reference lookup; no repository code inspected)
> `echo-agent-cli` commit: not-applicable (external reference lookup; no repository code inspected)
> Worktree state: clean (read-only research task)

## Question

What current cross-system patterns should constrain architecture, state,
Plan, Subagent, event, permission, skill/plugin, and recovery findings?

## Scope

External reference lookup against first-party primary sources for five
systems, plus one cross-system convergence synthesis:

- Claude Code (Anthropic) — plan mode, subagents, hooks, sessions,
  checkpointing.
- OpenAI Codex — `codex exec --json` event stream, thread/turn/item model,
  rollout persistence.
- Cursor — Plan Mode, background (Cloud) Agents.
- Devin (Cognition) — Managed Devins delegation, the "Don't Build
  Multi-Agents" architectural principles.
- Temporal — durable execution, Activity retry/idempotency, replay.

Topics covered per system: architecture, state model, plan mode,
subagent/delegation, events/streaming, permissions, skills/plugins,
recovery.

## Out Of Scope

- Any echo-agent / echo-agent-cli source inspection (this task establishes
  external baselines only; code inspection belongs to F-*, A-*, X-* tasks).
- Full per-item-type JSON schema for Codex's stream (primary doc gives the
  envelope only; per-type fields are secondarily documented).
- Cursor's and Devin's internal state/event mechanisms (not publicly
  documented at the Codex/Claude-Code level).
- Non-coding-agent workflow systems other than Temporal.

## Inputs

- `AGENTS.md` read in full (required reading); its "先调研业界优秀实现"
  section and product-positioning sections frame this task.
- `docs/comprehensive-review/README.md`, `REPORTING.md`, the `B-REF-01` task
  card, and both report templates.
- No dependency task reports (B-REF-01 has no declared dependencies).
- No historical audit finding was accepted as evidence.

## Layering Decision

This task produces *external reference constraints*, not code findings, so
the framework/application/adapter classification applies to **how downstream
tasks should classify the patterns**, not to the references themselves:

- **Generic mechanism (framework-level)** — the transferable primitives any
  `echo-agent` consumer may need: typed event envelope (C6), retry/idempotency
  primitives (C-contrast), isolation-first delegation shape (C5).
- **EKO product policy (application-level)** — plan-as-reviewable-artifact
  approval UX (C1), permission-mode non-restoration on resume (C4), the
  choice of trajectory-resume over event-sourced-replay for an interactive
  local agent (C3, C-contrast).
- **Adapter boundary** — N/A for this task (no adapter code inspected).

Repository-wide duplicate search: not applicable (no repository symbols
inspected). The relevant duplicate-authority concern for downstream tasks is
captured in C1/C5: the mature pattern is *one* plan authority (an artifact)
and *one* subagent authority (TaskRun→PlanTask→SubagentRun), not parallel
plan/todo/store implementations.

## Current Path

The "current path" for a reference task is the evidence chain per system.
Each system's claims are anchored to first-party primary URLs (accessed
2026-08-12) in its validation report. The convergence claims (C1–C7) in
`V06-01` are each backed by ≥3 independent systems. Two systems were
researched by two reviewers concurrently (a "ZCode subagent (B-REF-01)"
process and this reviewer); where their attempts disagree, the disagreement
and its resolution are recorded in `V06-01`'s table.

## Findings

The findings below are *reference constraints*: they state the mature cross-
system pattern and how it constrains echo-agent review findings. They are
not code defects. "Observed behavior" = what the external systems document;
"expected invariant" = the pattern downstream echo-agent design should not
violate.

### B-REF-01-P1-01: Plan is an artifact/data, not a runtime approval state machine

- Priority: P1
- Confidence: high
- Layer: framework (mechanism) / application (approval UX)
- Evidence: Claude Code `EnterPlanMode`/`ExitPlanMode` tools + `plan` mode
  non-restored on resume (`V01-02` A); Codex `plan updates` item type
  (`V02-02` A); Cursor reviewable/editable/saved plan (`V03-01` A); Devin
  approved decomposition (`V04-02` B).
- Reachability: 4/4 coding agents independently exhibit plan-as-artifact;
  0/4 use a multi-state runtime approval graph.
- Expected invariant: plan is a versioned artifact the user can review/edit/
  save/share; approval is a UI/tool-call flow over the artifact; runtime
  tracks at most a small mode flag, and privileged modes (plan,
  bypassPermissions) are NOT restored across resume.
- Observed behavior: all four coding agents treat plan as artifact/data.
- Impact: any echo-agent design that introduces a multi-state plan-approval
  runtime machine (e.g., Planning→AwaitingApproval→Ready→…) diverges from
  every mature reference. This is exactly the class of bug AGENTS.md records
  (the rejected 13-state design).
- Root cause (of the historical divergence): designing plan approval as a
  runtime state machine rather than researching mature implementations first.
- Direction: echo-agent `TaskPlan` must remain a versioned, editable
  artifact; approval must be prompt/permission-driven, not a run-state
  column. Do not reintroduce `plan_create/plan_patch/plan_execute` parallel
  CRUD (AGENTS.md rule 6).
- Regression validation: any future design that adds a plan-approval state
  to the run state machine must be checked against C1.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md),
  [V02-02](../validations/B-REF-01/V02-02.md),
  [V03-01](../validations/B-REF-01/V03-01.md),
  [V04-02](../validations/B-REF-01/V04-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C1).

### B-REF-01-P1-02: A typed, append-only event surface is the parity enabler

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: Codex JSONL thread/turn/item stream with stable per-item ids
  (`V02-02` A); Claude Code hooks at three cadences with exit-2 blocking
  (`V01-02` E); Temporal typed History Events (`V05-01` §5).
- Reachability: 3/5 systems expose a typed, externally-consumable surface;
  Codex's is the most explicit and machine-readable.
- Expected invariant: the agent loop is exposed as a deterministic, append-
  only surface (typed stream and/or lifecycle hooks) that TUI/GUI/CLI can
  consume identically.
- Observed behavior: Codex's stable per-item id lets a reducer update an
  in-flight row in place rather than append; Claude Code's hooks fire inside
  subagents with `agent_id`/`agent_type`.
- Impact: AGENTS.md mandates TUI/GUI/CLI feature parity. Without a typed
  event contract, parity is not achievable. echo-agent's event envelope
  should support the thread/turn/item shape with stable per-item ids.
- Direction: adopt Codex-style event identity (stable across
  started/updated/completed) as the reference for F-RCT-03 and X-EVT-01.
- Regression validation: a streaming/non-streaming conformance fixture
  (F-RCT-03) should assert the typed event identity.
- Validation reports: [V02-02](../validations/B-REF-01/V02-02.md),
  [V01-02](../validations/B-REF-01/V01-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C6).

### B-REF-01-P1-03: Subagent/delegation is isolation-first with bounded caps

- Priority: P1
- Confidence: high
- Layer: framework (shape) / application (policy)
- Evidence: Claude Code own-context subagents, `isolation: worktree`,
  `MAX_CONCURRENT_SUBAGENTS=20`, `MAX_SPAWN_DEPTH=3`, output scanning
  (`V01-02` D); Cursor isolated-VM Cloud Agents (`V03-01` B); Devin isolated-
  VM Managed Devins with coordinator-owned plan (`V04-02` A,B).
- Reachability: 3/4 coding agents parallelize via isolation (own context/VM/
  worktree), not via shared-context peer agents.
- Expected invariant: delegation gives each unit an isolated execution
  context and a narrow focus; the coordinator owns the plan and compiles from
  full trajectories; concurrency and depth are bounded and configurable.
- Observed behavior: isolation-first delegation is the convergent pattern;
  shared-context parallel agents are explicitly rejected by Cognition's
  first-party principles.
- Impact: matches echo-agent's `TaskRun→PlanTask→SubagentRun` model and the
  "only Subagents, not Workers" rule. A shared-context parallel-subagent
  design would diverge from all references and reintroduce the context-
  fragmentation problems Cognition documents.
- Direction: keep one subagent authority; make recursion bounded and
  configurable (Claude Code's depth-3 default is a reasonable reference), not
  forbidden; treat subagent output as untrusted (scan before parent reads).
- Regression validation: F-SUB-* should assert bounded concurrency/depth and
  output scanning.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md),
  [V03-01](../validations/B-REF-01/V03-01.md),
  [V04-02](../validations/B-REF-01/V04-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C5).

### B-REF-01-P1-04: Cognition's two principles are the anti-fragmentation rule

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: "Share context, and share full agent traces, not just individual
  messages" and "Actions carry implicit decisions, and conflicting decisions
  carry bad results" — verbatim from the first-party Cognition essay
  (`V04-02` C).
- Reachability: first-party articulation of why shared-context parallel
  agents are fragile.
- Expected invariant: when work is split across agents, either (a) share full
  traces (not just messages) so implicit decisions propagate, or (b) split
  into independent packages so no cross-agent implicit decisions conflict.
- Observed behavior: Devin's Managed Devins choose (b) — independent packages
  with post-hoc trajectory reading by the coordinator.
- Impact: directly informs when EKO should vs should not fan out subagents.
  Splitting one coherent artifact across live-shared-context parallel
  subagents is the anti-pattern.
- Direction: F-SUB-02 (subagent execution modes/teams) and F-TSK-03 (runtime
  DAG) should cite these principles when judging whether a fan-out is sound.
- Regression validation: any fan-out design review should check the two
  principles.
- Validation reports: [V04-02](../validations/B-REF-01/V04-02.md).

### B-REF-01-P1-05: Temporal is the contrast case — orchestrator replay is NOT transferable to LLM agents

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: Temporal event-sourced replay does not re-execute completed
  Activities (`V05-01` §4, `V05-02` C); this requires a deterministic
  orchestrator (Workflow), which an LLM ReAct loop is not (`V05-02` D); all
  4 coding agents converge on trajectory+resume instead (C3).
- Reachability: Temporal is the only event-sourced-replay system in the
  matrix; the 4 coding agents are the trajectory+resume tier.
- Expected invariant: echo-agent recovery should be trajectory persistence +
  resume into a new turn (the agent tier), plus best-effort file-state rewind
  for the user, NOT orchestrator replay over recorded tool results.
- Observed behavior: the agent convergence on trajectory+resume is forced by
  LLM non-determinism, not an oversight.
- Impact: prevents over-engineering echo-agent's `Snapshot` into a Temporal-
  style replay engine (which would be both infeasible and over-design for a
  local interactive agent, per AGENTS.md). The transferable Temporal parts
  are the retry/idempotency primitives only (see B-REF-01-P2-05).
- Direction: F-RCT-05 and X-STA-01 should adopt trajectory+resume +
  file-rewind; explicitly reject orchestrator-replay.
- Regression validation: a `Snapshot`-resume test must not assume re-running
  the orchestrator skips completed tools.
- Validation reports: [V05-01](../validations/B-REF-01/V05-01.md),
  [V05-02](../validations/B-REF-01/V05-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C3, C-contrast).

### B-REF-01-P2-01: Agent-tier persistence is file/JSONL trajectory, not a database

- Priority: P2
- Confidence: high
- Layer: application (CLI policy)
- Evidence: Claude Code JSONL transcripts under `~/.claude/` with separate
  subagent files (`V01-02` B); Codex rollout under `~/.codex/sessions`
  (`V02-02` B).
- Reachability: 2/4 coding agents publicly inspectable, both JSONL/file.
- Expected invariant: the CLI stores conversation/task state as files, not a
  relational DB.
- Observed behavior: both inspectable agents use JSONL files.
- Impact: supports AGENTS.md's "echo-agent-cli does not need SQLite"
  positioning. (The framework may still offer SQLite for other consumers —
  AGENTS.md framework rule; this finding does not argue for deleting
  `SqliteStore`.)
- Direction: do not introduce SQLite into echo-agent-cli; keep FileStore as
  the CLI default.
- Regression validation: none (reference constraint).
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md),
  [V02-02](../validations/B-REF-01/V02-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C2).

### B-REF-01-P2-02: Checkpointing is best-effort file-state rewind, with documented gaps

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: Claude Code keeps 100 most-recent per-prompt file snapshots;
  explicitly does NOT track bash/subagent/external/symlink changes; "not a
  replacement for version control" (`V01-02` C).
- Expected invariant: a session-level rewind facility reverts file edits
  made by the agent's own edit tools, best-effort, distinct from the
  conversation store.
- Observed behavior: Claude Code separates code restore from conversation
  restore and from compaction.
- Impact: echo-agent's rewind/checkpoint should be scoped to agent-made file
  edits and must document its gaps honestly; it is a recovery convenience,
  not a transactional guarantee.
- Direction: F-RCT-05 rewind design should mirror this scoped, best-effort
  model.
- Regression validation: rewind tests should cover the documented gaps.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md).

### B-REF-01-P2-03: Subagent recursion IS allowed and bounded (corrects V01-01)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: Claude Code `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` default 3;
  recursion withheld only at the depth limit (`V01-02` D). `V01-01` §4
  asserted "generally cannot spawn."
- Expected invariant: subagent recursion is bounded and configurable, not
  forbidden.
- Observed behavior: primary doc confirms default depth 3.
- Impact: echo-agent should make subagent nesting bounded/configurable
  (Claude Code's model), not unconditionally disabled.
- Direction: F-SUB-* should model bounded recursion.
- Regression validation: a depth-limit test.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md)
  (supersedes V01-01 §4 on this point).

### B-REF-01-P2-04: Recovery = persisted rollout + resume-by-ID into a new turn

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: Codex `codex exec resume --last` / `resume <SESSION_ID>`
  (`V02-02` B, corrects V02-01's "known gap"); Claude Code `--resume` re-reads
  transcript (`V01-02` B).
- Expected invariant: a run can be resumed by ID, restoring the trajectory
  and continuing in a new turn, without re-running completed side effects.
- Observed behavior: both agents resume by ID into a new turn.
- Impact: echo-agent run/turn identity should support resume-by-ID; the
  `thread_id`/session-id is the join key.
- Direction: F-CORE-01 identity + F-RCT-05 resume should support this.
- Regression validation: a resume-by-ID round-trip test.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md),
  [V02-02](../validations/B-REF-01/V02-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C3).

### B-REF-01-P2-05: Activity retry + idempotency-key is the transferable Temporal primitive

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: Temporal at-least-once default; idempotency key = Workflow Run ID
  + Activity ID; at-most-once via `maximumAttempts=1`; retry backoff tracked
  across worker crashes (`V05-01` §8, `V05-02` B).
- Expected invariant: tool calls retry at-least-once with an idempotency key
  keyed by (run-id, call-id); destructive/non-idempotent tools opt to
  at-most-once.
- Observed behavior: Temporal documents this precisely.
- Impact: the one transferable Temporal lesson for echo-agent tool-batch
  execution (F-RCT-04) and retry primitives (F-REL-01).
- Direction: adopt the (run-id, call-id) idempotency-key pattern and the
  at-most-once escape hatch.
- Regression validation: F-RCT-04 partial-side-effect fixtures.
- Validation reports: [V05-01](../validations/B-REF-01/V05-01.md),
  [V05-02](../validations/B-REF-01/V05-02.md).

### B-REF-01-P3-01: Permission is a launch-time mode/flag, not an approval state machine

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: Codex `--sandbox` + `--ask-for-approval`, `--full-auto`
  deprecated (`V02-02` C); Claude Code session modes, plan/bypass not
  restored (`V01-02` A,E); Cursor/Devin isolation boundaries (`V03-01` B,
  `V04-02` E).
- Expected invariant: permission is set at launch (mode/flag) plus an
  isolation boundary; hooks/gates act per-call but the policy itself is not a
  runtime approval graph.
- Impact: supports AGENTS.md's rule that permission modes govern automated
  agent paths and should not gate user-interactive tools (terminal, file
  picker).
- Direction: F-HITL-01 should treat permission as launch mode + isolation.
- Validation reports: [V01-02](../validations/B-REF-01/V01-02.md),
  [V02-02](../validations/B-REF-01/V02-02.md),
  [V06-01](../validations/B-REF-01/V06-01.md) (C4).

### B-REF-01-P3-02: Revert-and-replan is a valid recovery flow

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence: Cursor recommends reverting changes and re-planning over patching
  an in-progress agent (`V03-01` A).
- Impact: reinforces that plans are re-runnable artifacts and side effects
  are disposable via VCS.
- Validation reports: [V03-01](../validations/B-REF-01/V03-01.md).

### B-REF-01-P3-03: Plan-approval and execution-approval are separable

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence: Devin approves the proposed decomposition, then each managed
  Devin executes autonomously (`V04-02` B).
- Impact: a product may approve the *plan decomposition* while leaving
  *per-step* execution agent-driven — a useful design option for EKO.
- Validation reports: [V04-02](../validations/B-REF-01/V04-02.md).

### B-REF-01-P3-04: At-most-once for destructive/non-idempotent tools

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: Temporal `maximumAttempts=1` for at-most-once (`V05-02` B).
- Impact: destructive shell commands should not be auto-retried; relevant to
  the full-auto/default permission distinction.
- Validation reports: [V05-02](../validations/B-REF-01/V05-02.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Claude Code primary-source lookup | yes | passed | [V01-02](../validations/B-REF-01/V01-02.md) (supersedes [V01-01](../validations/B-REF-01/V01-01.md) on subagent recursion) |
| V02 | Codex primary-source lookup | yes | passed | [V02-02](../validations/B-REF-01/V02-02.md) (supersedes [V02-01](../validations/B-REF-01/V02-01.md) on resume-gap framing) |
| V03 | Cursor primary-source lookup | yes | passed | [V03-01](../validations/B-REF-01/V03-01.md) |
| V04 | Devin + Cognition primary-source lookup | yes | passed | [V04-02](../validations/B-REF-01/V04-02.md) (upgrades [V04-01](../validations/B-REF-01/V04-01.md) to first-party) |
| V05 | Temporal primary-source lookup | yes | passed | [V05-02](../validations/B-REF-01/V05-02.md) + [V05-01](../validations/B-REF-01/V05-01.md) (facts agree; transferability framing reconciled) |
| V06 | Cross-system convergence | yes | passed | [V06-01](../validations/B-REF-01/V06-01.md) |

Note on dual attempts: a concurrent reviewer ("ZCode subagent (B-REF-01)")
produced V01-01, V02-01, V04-01, V05-01; this reviewer produced V01-02,
V02-02, V03-01, V04-02, V05-02, V06-01. Both sets are retained per the
immutability rule; disagreements and resolutions are tabulated in V06-01.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: "Claude Code's plan mode is prompt injection, not runtime enforcement" | current (with refinement) | Plan mode is tool-mediated (`EnterPlanMode`/`ExitPlanMode`), `plan` mode not restored on resume; the official docs describe behavior, not the internal mechanism. V01-02 A. |
| AGENTS.md: the rejected 13-state plan-approval runtime machine was an outlier | current (corroborated) | 4/4 coding agents treat plan as artifact; none use a runtime approval graph. V06-01 C1; finding P1-01. |
| AGENTS.md: "echo-agent-cli does not need SQLite" | current (supported) | 2/2 inspectable coding agents use JSONL/file trajectory, not a DB. V06-01 C2; finding P2-01. |
| AGENTS.md: permission modes apply to automated agent paths, not user-interactive tools | current (supported) | Permission is launch mode + isolation in all references; no user-vs-agent carve-out documented. V06-01 C4; finding P3-01. |
| V01-01 §4: "subagents generally cannot spawn further subagents" | regressed → corrected | Primary doc shows recursion allowed, depth 3 default. V01-02 D; finding P2-03. |
| V02-01 §8: Codex resume is "a known gap" (#1991) | stale → corrected | `codex exec resume` is now a documented shipped feature. V02-02 B. |

## Coverage And Uncertainty

- **Skills sampled thinly.** Claude Code's skill system (progressive
  disclosure, model-invocation default) is documented primarily in `V01-01`
  §7 from secondary sources; this reviewer did not re-fetch the primary
  skills doc. The convergence claim (C7) is moderate-confidence.
- **Codex per-item-type schema** is documented at the envelope level only;
  per-type field tables come from a secondary cheatsheet (`V02-01` §5). The
  envelope itself is primary-confirmed.
- **Cursor/Devin internal mechanisms** (state machine, event schema,
  concurrency limits) are not publicly documented; only product behavior is.
- **Temporal Workflow determinism constraints** are stated at concept level
  here; the full determinism rule set is in Temporal's encyclopedia (read by
  V05-01) and is standard doctrine.
- **Two reviewers ran concurrently.** Their factual claims are consistent;
  the two analytical divergences (Codex resume, Temporal transferability)
  are resolved in favor of the primary-source reading and recorded.
- No repository code was inspected; all findings are external-reference
  constraints, not code defects.

## Handoff

- **Conclusions downstream tasks may rely on:** the seven convergent
  patterns (C1–C7) and the Temporal contrast case (C-contrast) in V06-01 are
  the canonical external-reference anchors. Downstream tasks should cite the
  relevant convergence ID rather than re-deriving the comparison.
- **Reports downstream tasks must read:** V06-01 (convergence) plus the
  per-system report for any system they cite.
- **Task-to-convergence mapping:**
  - F-TSK-01 (canonical task/plan model) → C1, finding P1-01.
  - F-TSK-03 (runtime DAG) → C5, finding P1-03; Cognition principles P1-04.
  - F-RCT-03 (streaming events) / X-EVT-01 → C6, finding P1-02.
  - F-RCT-04 (tool batch) → C-contrast, finding P2-05, P3-04.
  - F-RCT-05 (snapshot/resume) → C3, C-contrast, findings P1-05, P2-02,
    P2-04.
  - F-HITL-01 (permissions) → C4, finding P3-01.
  - F-SUB-01/02 (subagents) → C5, findings P1-03, P2-03.
  - F-SKL-01 (skills) → C7.
  - X-STA-01 (persistence/recovery/identity) → C2, C3, C-contrast.
- **Conditions that make this report stale:** if Claude Code, Codex, Cursor,
  Devin, or Temporal publishes a contradictory revision of the cited
  behavior (e.g., Codex reverting/deprecating `codex exec resume`, or Claude
  Code introducing a plan-approval state machine), the affected convergence
  claim must be revalidated.
- **Follow-up task IDs (no fixes implemented in this review task):** none
  new — the findings here constrain existing F-*/A-*/X-* tasks as mapped
  above. A targeted revalidation task should be opened only if a cited
  primary source contradicts this report later.
