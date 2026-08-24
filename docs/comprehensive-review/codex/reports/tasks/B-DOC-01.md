# B-DOC-01: Historical audit and design drift index

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean

## Question

Which existing audit and plan claims still point at current code, which are
fixed/stale/regressed, and which require targeted revalidation?

## Scope

- Root `docs/MASTER-PLAN.md`, `PROJECT-ANALYSIS.md`,
  `deep-iteration-plan.md`, and the current `AGENTS.md` recovery contract.
- `echo-agent-cli/docs/MASTER-PLAN.md`, the July 28 app-core full audit, and the
  July 3 application code review.
- `echo-agent/AUDIT_REPORT.md` dated 2026-05-31.
- Representative completed-milestone anchors: framework file runtime state,
  conversation restore/file store, EKO PluginRuntimeService, and typed
  TaskRuntime hook delivery.
- Dependency reports `B-ARCH-01`, `B-PATH-01`, and the external-evidence limits
  from `B-REF-01`.

## Out Of Scope

- Re-reviewing the behavior behind all 63 historical security/application
  findings. They are routed to current atomic tasks instead.
- Public README/API correctness beyond the architecture drift already proven by
  `B-ARCH-01`; `F-API-01` and `Q-DOC-01` own that surface.
- Editing or deleting historical documents during this read-only review.
- Treating historical online-service threat assumptions as current EKO
  priorities without the local threat model in `AGENTS.md`.

## Inputs

- Root `AGENTS.md`; shared comprehensive-review `README.md`, `REPORTING.md`, and
  `B-DOC-01` task card; Codex track `README.md`.
- Completed Codex dependencies [B-ARCH-01](B-ARCH-01.md) and
  [B-PATH-01](B-PATH-01.md).
- Completed [B-REF-01](B-REF-01.md) was read only to constrain whether old
  external implementation claims may be repeated; undocumented internals were
  not promoted to facts.
- The historical audits above were treated as hypotheses, never copied as
  current findings.

## Layering Decision

- Generic mechanism: public framework documentation and historical framework
  audits may describe reusable stores, tools, sandboxing, hooks, and event
  contracts, but current subsystem reports must validate those independently.
- EKO product policy: master-plan authority, local threat model, mode parity,
  storage selection, milestone status, and operational handoff belong to the
  application/repository documentation layer.
- Adapter boundary: document references may point across repositories, but no
  ownership move is recommended by this indexing task.
- Duplicate search: both master-plan names/authority statements, storage type
  names, plugin/hook runtime names, legacy module/crate paths, `worker` terms,
  `cargo clean` imperatives, and all local Markdown targets in the curated
  corpus were searched. Definitions, re-exports, and EKO consumers were
  distinguished from mere mentions.

## Current Path

The intended recovery path is:

```text
new task/context
  -> read AGENTS.md
  -> AGENTS.md:421 directs reader to docs/MASTER-PLAN.md
  -> docs/MASTER-PLAN.md:3 declares itself the single source
```

The repository currently exposes a competing path:

```text
echo-agent-cli/docs/MASTER-PLAN.md:5
  -> independently declares itself the cross-session source of truth
  -> completion table through 2026-07-29
  -> active Next Step partly copied from the pre-migration July 28 audit
```

The sampled current implementation is unambiguous even where prose is not:

- Framework `FileRuntimeStateStore` is defined at
  `echo-agent/src/state/file.rs:34`, implements `RuntimeStateStore` at line 92,
  is re-exported at `src/state/mod.rs:294`, and EKO constructs it at
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1254`.
- Framework `restore_messages` is defined at
  `echo-state/src/memory/conversation.rs:197`, re-exported at
  `echo-state/src/memory/mod.rs:42`, and used by CLI/TUI/Tauri.
- Framework `FileConversationStore` is defined at
  `echo-state/src/memory/file_conversation.rs:68`, re-exported at
  `echo-state/src/memory/mod.rs:45`, and constructed by EKO at
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1218`.
- EKO's shared `PluginRuntimeService` exists at
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:130`, is retained by
  runtime/state, and serves Tauri commands.
- `HookEventDispatcher` at
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/hook_event_dispatcher.rs:49`
  translates the typed Task/Subagent lifecycle with correlation fields.

## Findings

### B-DOC-01-P1-01: Competing master plans can send a fresh implementation task backwards

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `AGENTS.md:421`; `AGENTS.md:424`;
  `docs/MASTER-PLAN.md:3`; `docs/MASTER-PLAN.md:396`;
  `docs/MASTER-PLAN.md:980`; `docs/MASTER-PLAN.md:983`;
  `echo-agent-cli/docs/MASTER-PLAN.md:5`;
  `echo-agent-cli/docs/MASTER-PLAN.md:70`;
  `echo-agent-cli/docs/MASTER-PLAN.md:446`
- Reachability: every new review/implementation context is explicitly told by
  `AGENTS.md` to load the root master plan. A developer working inside EKO can
  instead encounter the CLI master plan, which makes the same authority claim.
  Both contain imperative “next” text used to resume work.
- Expected invariant: there is exactly one current cross-session authority; it
  gives a monotonic status and never schedules work already marked complete.
- Observed behavior: the CLI plan marks S1/S2/S3 complete at line 70, then says
  all three remain and will migrate at lines 446-451. The root plan retains
  “应用层接入待做” and PluginRuntimeService “下一步” at lines 980/983 despite
  later completion entries, and mandates unconditional `cargo clean` contrary
  to current `AGENTS.md:324-337`.
- Impact: a fresh AI coding context can recreate removed adapters, repeat a
  completed high-risk migration, use obsolete lifecycle semantics, or waste
  large build time/disk churn. This is a direct execution hazard because the
  repository mandates document-based context recovery.
- Root cause: milestone history and operative status are appended in place,
  while a second master plan was introduced without an authority/archival
  boundary. Completion entries do not retire older imperative prose.
- Direction: designate root `docs/MASTER-PLAN.md` as the only current authority
  (matching `AGENTS.md`), reduce it to current invariants/status/next bounded
  work, and move dated narrative to explicitly historical snapshots. Replace
  `echo-agent-cli/docs/MASTER-PLAN.md` with a pointer or rename it as an archived
  domain-roadmap snapshot. Delete completed `待做` prose instead of preserving
  it below a completion table; link operational gates back to `AGENTS.md`.
- Regression validation: a documentation test must assert one source-of-truth
  declaration, one current next-step section, no item present as both complete
  and pending, and no duplicated mandatory gate commands.
- Validation reports: [V02](../validations/B-DOC-01/V02-01.md),
  [V03](../validations/B-DOC-01/V03-01.md),
  [V09](../validations/B-DOC-01/V09-01.md)

### B-DOC-01-P2-01: The authoritative M10 parity completion claim has regressed

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `docs/MASTER-PLAN.md:25`; `docs/MASTER-PLAN.md:380`;
  `echo-agent-cli/src/main.rs:365`;
  `echo-agent-cli/src/cli/modes.rs:118`
- Reachability: the channels-only binary compiles and `main` calls
  `run_channels_mode` directly. Accepted `B-PATH-01` source and compile evidence
  proves that branch bypasses common headless services.
- Expected invariant: a milestone marked “five-entry parity complete” either
  remains true or is reopened when a supported surface loses shared runtime
  composition.
- Observed behavior: the current plan says channel/cron gaps are closed, while
  pure channel mode starts no BackgroundTaskService, SchedulerRunner, plugin
  monitor binding, or Dreaming lifecycle.
- Impact: future planning treats a P1 product invariant as already closed and
  may omit the channels-only regression from the roadmap.
- Root cause: milestone status is manually asserted and the parity test checks
  prose evidence rather than constructor behavior.
- Direction: reopen only the affected M10 composition item and link it to
  canonical `B-PATH-01-P1-01`; do not duplicate the runtime finding in a new
  subsystem. Close it only after constructor-level and real channels-only cron/
  background tests pass.
- Regression validation: the `B-PATH-01` recommended mode-constructor matrix and
  pure-channel persisted-cron/background scenario.
- Validation reports: [V07](../validations/B-DOC-01/V07-01.md),
  [B-PATH V09](../validations/B-PATH-01/V09-01.md)

### B-DOC-01-P3-02: Current project guides retain deleted paths and the forbidden Worker vocabulary

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `AGENTS.md:139`; `AGENTS.md:370`;
  `docs/PROJECT-ANALYSIS.md:62`; `docs/PROJECT-ANALYSIS.md:253`;
  `docs/deep-iteration-plan.md:26`; `docs/deep-iteration-plan.md:230`
- Reachability: `AGENTS.md` is mandatory input and the two root documents read
  as current architecture/iteration guides rather than immutable historical
  artifacts.
- Expected invariant: current documentation uses the sole Subagent model and
  names existing workspace components.
- Observed behavior: `AGENTS.md` lists removed `echo-agent-eval` and its manifest
  path. The two root guides retain internal `Worker` types/roles/prompts and
  current-looking “待做” work even though the product terminology was globally
  migrated.
- Impact: search-driven implementation starts from nonexistent modules and
  reintroduces a second execution vocabulary into designs, reports, or code.
- Root cause: terminology/path migration updated code and newer deep dives but
  did not classify or update older root-level guides.
- Direction: fix `AGENTS.md` immediately; either update the two root guides to
  current Subagent names/paths or rename them as dated historical snapshots and
  remove them from current onboarding. Do not rewrite quoted third-party wire
  names in genuine protocol archives.
- Regression validation: current-doc scan for `echo-agent-eval` and internal
  `\bworker(s)?\b`, with an explicit allowlist limited to external fixed names
  and dated historical snapshots.
- Validation reports: [V04](../validations/B-DOC-01/V04-01.md)

### B-DOC-01-P3-03: The cross-context authority links to non-resolvable prior subagent IDs

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `docs/MASTER-PLAN.md:931`
- Reachability: a fresh context following the mandated root master plan reaches
  its “探索存档” links when verifying the memory/evolution assessment.
- Expected invariant: evidence links in a cross-context authority resolve to
  durable repository artifacts or stable URLs.
- Observed behavior: three UUID-like destinations are rendered as relative
  Markdown links, but no target exists in the repository and no URI scheme or
  task-system locator is supplied.
- Impact: the claimed evidence cannot be inspected after context handoff, so
  the historical conclusion is not reproducible.
- Root cause: ephemeral harness task identifiers were persisted as if they were
  repository-relative documents.
- Direction: replace each with a durable report path if the artifact exists; if
  not, retain the identifier as plain historical text and state that the raw
  exploration is unavailable.
- Regression validation: local-link checker over current authority documents,
  with URI-scheme-aware parsing.
- Validation reports: [V05-01](../validations/B-DOC-01/V05-01.md),
  [V05-02](../validations/B-DOC-01/V05-02.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Curated document corpus | yes | passed after inconclusive broad attempt | [V01-01](../validations/B-DOC-01/V01-01.md), [V01-02](../validations/B-DOC-01/V01-02.md) |
| V02 | Authority and internal-status consistency | yes | failed | [V02](../validations/B-DOC-01/V02-01.md) |
| V03 | Completed-milestone current-code anchors | yes | failed (stale pending prose) | [V03](../validations/B-DOC-01/V03-01.md) |
| V04 | Obsolete path and terminology search | yes | failed | [V04](../validations/B-DOC-01/V04-01.md) |
| V05 | Local document targets | yes | failed after corrected URI classification | [V05-01](../validations/B-DOC-01/V05-01.md), [V05-02](../validations/B-DOC-01/V05-02.md) |
| V06 | Historical-finding classification and routing | yes | passed | [V06](../validations/B-DOC-01/V06-01.md) |
| V07 | M10 parity status against accepted entry-path evidence | yes | failed | [V07](../validations/B-DOC-01/V07-01.md) |
| V08 | First milestone-sampling orchestration cell | evidence integrity | failed before nested execution | [V08](../validations/B-DOC-01/V08-01.md) |
| V09 | Current cleanup policy consistency | yes | failed | [V09](../validations/B-DOC-01/V09-01.md) |
| V10 | Primary report/link/finding-boundary acceptance | yes | passed after failed assembly attempt | [V10-01](../validations/B-DOC-01/V10-01.md), [V10-02](../validations/B-DOC-01/V10-02.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| July 28 audit S1/S2/S3 are framework gaps | fixed | [V03](../validations/B-DOC-01/V03-01.md) |
| July 28 audit cleanup of four app-core paths | fixed | [V04](../validations/B-DOC-01/V04-01.md) |
| CLI master “only 3 gaps remain” in active Next Step | stale and internally contradicted | [V02](../validations/B-DOC-01/V02-01.md), [V03](../validations/B-DOC-01/V03-01.md) |
| Root master M10 five-entry parity complete | regressed for pure-channel common services | [V07](../validations/B-DOC-01/V07-01.md) |
| Root memory/evolution P0/P1 list | partially fixed; unverified remainder | route memory claims to `A-MEM-01`, plugin/hook claims to `A-PLG-01`/`A-TSK-04` |
| May framework audit's 32 findings | stale hypotheses pending targeted revalidation | [V06](../validations/B-DOC-01/V06-01.md) |
| July application review's 31 findings and online XSS/SSRF priority model | stale hypotheses; priority incompatible with current local threat boundary unless local impact is shown | [V06](../validations/B-DOC-01/V06-01.md) |
| Root `AGENTS.md` says CLI contains `echo-agent-eval` | stale | [V04](../validations/B-DOC-01/V04-01.md) |
| Root/split framework READMEs describe current architecture | regressed/stale | [B-ARCH-01](B-ARCH-01.md) |

## Unresolved Historical-Finding Index

| Historical family | Do not assume | Current owner |
|---|---|---|
| SQL/database/web/media validation and provenance | May audit priority or old line numbers | `F-EXT-03` |
| shell/file/Git/process cleanup and partial effects | old implementation still exists | `F-EXT-02`, `Q-FLT-01` |
| sandbox, secrets, unsafe, panic, path traversal | online-service threat model or current reachability | `F-SEC-01`, `Q-STA-01` |
| MCP disconnect/pending request behavior | old transport remains unchanged | `F-INT-01` |
| memory/Dreaming/workspace refresh | root July list remains current | `A-MEM-01`, `X-MEM-01` |
| plugin/hook lifecycle | old P0/P1 list remains open | `F-PLG-01`, `A-PLG-01`, `X-PLG-01` |
| frontend listener/render/accessibility issues | old component paths imply current behavior | `A-FE-03`, `Q-WEB-01` |
| production panic/UTF-8 compliance assertions | broad historical counts are current | `Q-STA-01` |

## Coverage And Uncertainty

The review did not reopen every historical behavior; that would duplicate the
atomic catalog and violate this task's scope. A source file still existing does
not make its old finding current. Conversely, a deleted path can classify that
exact anchor stale/fixed but does not prove an equivalent defect did not move.
The document link script covers the five curated authority/audit files, not all
Markdown. External URLs were not live-checked here; `B-REF-01` already records
the accepted current first-party evidence and limits.

## Handoff

- `F-API-01` may rely on this authority map and must read `B-ARCH-01` for public
  contract drift.
- `Q-DOC-01` owns consolidation, current-doc link/path checks, and executable
  README/config examples. It should not mechanically rewrite historical audit
  snapshots.
- Subsystem tasks should use the unresolved index only as search hypotheses and
  create current source/runtime evidence before reporting a finding.
- `S-ROAD-01` should place documentation authority consolidation early because
  all later AI iterations consume it, but retain canonical runtime defect
  ownership under the relevant subsystem task.
- This report becomes stale when either master plan, `AGENTS.md` project map,
  the sampled storage/plugin/hook anchors, or `B-PATH-01` findings change.
