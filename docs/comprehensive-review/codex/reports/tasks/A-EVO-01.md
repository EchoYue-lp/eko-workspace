# A-EVO-01: EKO evolution product scope

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent-cli` remained clean. Concurrent changes in
> `echo-agent` were excluded without reading, modification, or rollback;
> framework evidence came from committed `HEAD` and accepted dependency reports.
> Accepted by: Codex primary reviewer after exact-ID, source-anchor,
> reachability, finding-count, link, executor, commit, and isolation checks.

## Question

Has EKO kept evolution as explicit diagnostics/review without hidden metric
loops, automatic semantic mutation, or framework option deletion?

## Scope

- EKO evidence capture, Review Inbox accept/reject/edit/undo, rule promotion,
  skill candidate/draft/activation, Curator actions, dashboard, auto-memory, and
  Dreaming scheduling.
- GUI, TUI, and CLI reachability plus user-authorization and runtime refresh
  boundaries.
- Framework feature consumption only where needed to prove EKO policy. Public
  framework eval/improve/evolution options are reviewed as independent APIs,
  not as dead code because EKO does not enable them.
- Historical product claims and static test coverage.

## Out Of Scope

- Framework eval/improve/evolution implementation defects already owned by
  [F-EVO-01](F-EVO-01.md).
- Generic FileStore, memory projection, and workspace binding defects already
  owned by [F-MEM-01](F-MEM-01.md) and [A-MEM-01](A-MEM-01.md).
- Source edits, Cargo, rustc, tests, builds, dynamic fixtures, and network
  access, per the user's review-only instruction.

## Inputs

- Root `AGENTS.md`; comprehensive review `README.md`, `REPORTING.md`, exact
  A-EVO-01 task card, and templates.
- Accepted Codex dependencies [F-EVO-01](F-EVO-01.md) and
  [A-MEM-01](A-MEM-01.md).
- Current clean `echo-agent-cli` source and committed `echo-agent` source. No
  other reviewer directory was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Proposal-only reviewers, typed memory mutation primitives, Dreaming mechanics, Curator, eval runners, improvement analysis, audit primitives, and optional feature gates are reasonable reusable `echo-agent` capabilities. They remain framework APIs even when EKO deliberately does not enable `eval`/`improve`. |
| EKO product policy | Which analyses run automatically, whether a proposal needs user confirmation, `.eko` paths, Review Inbox state, mutation orchestration, skill catalog refresh, surface parity, and scheduling belong in `echo-agent-cli`. |
| Adapter boundary | EKO should bind a proposal to workspace/source generation, record an operation before invoking a framework mutator, reconcile the durable result, and publish one receipt to all surfaces and Agents. It must not reimplement framework scoring or mutation algorithms. |
| Duplicate search | Searched both repositories for eval/improve/evolution features and exports; Dreaming construction/callers; ReviewConfig; evidence capture/accept/undo; rule promotion; Curator/draft activation; primary/pool skill refresh; TUI/GUI/CLI commands; docs and tests. |
| Migration deletion | Retain framework options. Delete EKO's duplicated CLI/Tauri rule and skill mutation sequences after one app-core mutation service owns journaling, workspace identity, projection/catalog refresh, and surface receipts. Remove default automatic Dreaming startup or convert it to proposal production once the replacement path is live. |

## Current Data Flow

```text
analysis-only sources
  BackgroundReviewer / TriggerDetector / AutoMemory / MemoryReviewer
       -> workspace EvidenceStore JSONL -> explicit user action
       -> MemoryLayerManager mutation -> Evidence snapshot update

explicit rule/skill actions
  GUI or CLI -> direct file/Curator mutation -> primary Agent refresh

automatic path
  GUI + TUI + CLI boot -> 60 s -> daily Dreaming(default thresholds)
       -> recall metrics -> revive/promote/archive immediately
       -> Store/MEMORY.md + live prompt selection
```

The proposal boundary is mostly well designed: heuristic and LLM observations
enter one bounded workspace inbox, BackgroundReviewer is proposal-only by
default, MemoryReviewer emits analysis/conflict proposals, and EKO does not
enable framework eval/improve loops. The exceptions are the scheduled Dreaming
mutator and multi-authority application commits described below.

## Findings

### A-EVO-01-P0-01: Rule promotion can replace a user-editable rules file after a read error and then leave a partially committed mutation

- Priority: P0
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs:178`,
  `:197`, `:198`, `:202`, `:219`, `:223`, `:238`, `:252`;
  `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:291`, `:300`,
  `:305`.
- Reachability: GUI `promote_rule` and CLI `/rule-promote` call this method only
  after an explicit user review, so authorization exists but the mutation path
  is live.
- Expected invariant: unreadable/corrupt existing rules fail closed and preserve
  the original bytes; the file, source-memory marker, change log, and live
  projection commit as one recoverable operation.
- Observed behavior: every `read_to_string` error, including invalid UTF-8, is
  converted to an empty document. `std::fs::write` then truncates/replaces the
  user-editable file. File write, memory marker, and change-log append happen in
  sequence with no journal or compensation; an error/crash after the file write
  returns failure while the rule is already active.
- Impact: a normal reviewed action can erase existing learned rules or leave a
  rule active without its dedup marker/audit record. Retry can append duplicates,
  and restart cannot determine which authority committed.
- Root cause: absence, unreadable data, and invalid UTF-8 share one empty-string
  fallback, while a surface helper owns a multi-resource transaction.
- Direction: move promotion behind one workspace-bound app-core operation;
  require typed Missing versus ReadError, bind expected source/file hashes,
  stage an atomic file replacement, persist operation intent and before state,
  then commit memory/audit/projection with resumable compensation. Never
  overwrite unreadable bytes. Delete direct CLI/Tauri sequencing after migration.
- Regression validation: missing versus invalid UTF-8/permission/partial file,
  crash/error at every file-memory-log-refresh boundary, concurrent user edit,
  retry/restart, and exact original-byte preservation.
- Validation reports: [V07](../validations/A-EVO-01/V07-01.md)

### A-EVO-01-P1-02: Every interactive mode starts an unconfigured recall-metric loop that changes durable memory and model context without review

- Priority: P1
- Confidence: high
- Layer: application policy
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:1143`, `:1156`,
  `:1158`, `:1165`, `:1166`, `:1203`, `:1209`; live callers in
  `src/tauri/desktop.rs:247`, `src/tui/mod.rs:1999`, and `src/cli/repl.rs:106`;
  committed `echo-agent/src/evolution/dreaming.rs:115`, `:142`, `:160`, `:187`.
- Reachability: GUI, TUI, and CLI create the task at startup. After the 60-second
  boot delay, Tokio's first interval tick is immediately ready; subsequent runs
  are daily. EKO always constructs `DreamingConfig::default()` and exposes no
  product opt-in or review receipt.
- Expected invariant: automatic diagnostics may produce bounded proposals, but
  recall metrics alone do not change durable memory status, hot prompt contents,
  or behavior without an explicit product policy and visible decision.
- Observed behavior: the loop revives Archived entries, promotes them into
  `MEMORY.md`, and archives Active entries directly. The resulting report is
  logged only. These changes alter what later model requests see even though
  content text is not rewritten.
- Impact: running EKO for roughly one minute can silently make facts appear in or
  disappear from the model's active context. Users cannot preview, reject, undo,
  or even identify the scheduled decision from the Review Inbox.
- Root cause: a valid generic framework mutator was treated as product-neutral
  maintenance, so EKO bypassed its own proposal/authorization boundary.
- Direction: keep framework Dreaming, but make EKO's scheduled pass produce
  durable bounded proposals/decision receipts only. Apply promote/revive/archive
  through the same explicit Review Inbox service, or require a clearly exposed
  opt-in automation policy with audit and undo. Remove unconditional startup
  once the proposal path is live; do not add online-service permission gates.
- Regression validation: all three modes, first/daily tick, opt-out/default,
  workspace switch, cancel/restart, and exact proof that no Store/status/hot
  projection changes before an authorized receipt.
- Validation reports: [V04](../validations/A-EVO-01/V04-01.md)

### A-EVO-01-P1-03: Evidence accept and undo mutate memory before durably recording the operation state

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/evolution/evidence.rs:478`,
  `:516`, `:528`, `:541`, `:571`, `:590`, `:608`, `:638`, `:662`, `:688`,
  `:718`, `:742`, `:772`; F-EVO-01-P1-09.
- Reachability: GUI, TUI, and CLI Review Inbox actions call the shared
  `EvidenceStore::accept`/`undo`, which is the canonical EKO semantic mutation
  boundary.
- Expected invariant: an operation identity and recoverable phase are durable
  before external mutation; after restart, candidate status/target and memory
  state converge to exactly Applied or Pending.
- Observed behavior: `AcceptAttempt` is only an interaction event. SaveMemory or
  merge mutates the MemoryLayerManager first, then appends Applied/target state.
  Undo deletes/restores memory first, then marks Pending. Compensation handles a
  returned append error but not process termination; startup does not reconcile
  incomplete attempts. Framework merge also has its own member-level partial
  commit defect owned by F-EVO-01-P1-09.
- Impact: a crash can leave a Pending candidate whose memory is active, or an
  Applied candidate whose target was removed/restored. Undo metadata may be
  absent exactly when recovery needs it, so UI state and Agent behavior disagree.
- Root cause: interactions are audit-shaped facts, not a mutation journal with
  operation identity, phases, idempotence, and recovery ownership.
- Direction: persist `Preparing/Mutating/Applied/Undoing` with operation ID,
  candidate revision, target/before state, workspace generation, and idempotency
  key before calling mutators. Reconcile incomplete operations at boot and make
  all compensation durable. Reuse framework mutation receipts; do not create a
  second memory algorithm.
- Regression validation: terminate at every await/append boundary for SaveMemory,
  merge, undo, and compensation; restart repeatedly and assert one terminal state,
  one target, exact before restoration, and monotonic interaction history.
- Validation reports: [V05](../validations/A-EVO-01/V05-01.md)

### A-EVO-01-P1-04: One partial Evidence JSONL append makes the entire Review Inbox and its undo state unreadable

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/evolution/evidence.rs:800`,
  `:821`, `:897`, `:934`, `:941`, `:955`, `:962`, `:975`, `:981`, `:991`.
- Reachability: every inbox list/get/edit/reject/accept/undo and interaction append
  rereads the same append-only file. All three surfaces use this store.
- Expected invariant: a torn final append is detectable and recoverable without
  discarding prior committed candidates, targets, or undo snapshots; interior
  corruption is surfaced with a repair path.
- Observed behavior: `write_all` can leave a partial final record before returning
  error, while `sync_data` occurs only after the full line. Both readers parse
  every nonempty line with `serde_json::from_str`; one malformed line returns an
  error for the whole log. There is no sequence/checksum, tail recovery, snapshot,
  quarantine, or compaction, and every mutation rereads the unbounded history.
- Impact: disk-full or process/power interruption can permanently block review,
  acceptance, rejection, and undo for all otherwise valid records in a workspace.
- Root cause: append-only audit and authoritative current/recovery state share an
  unframed JSONL file without a recovery protocol.
- Direction: add framed records with sequence/checksum and explicit durable
  boundary, recover only a proven incomplete tail while quarantining bytes, and
  checkpoint/compact to a verified snapshot. Fail loudly on interior corruption;
  never silently skip it.
- Regression validation: partial final JSON and newline, partial UTF-8, corrupted
  interior line, disk-full short write, two handles, compaction crash, and exact
  recovery of target/before/interaction facts.
- Validation reports: [V06](../validations/A-EVO-01/V06-01.md)

### A-EVO-01-P1-05: Skill evolution commits lifecycle and files before runtime convergence, while TUI cannot perform the same reviewed actions

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/src/tauri/commands/panels.rs:447`, `:536`, `:572`,
  `:624`, `:1392`, `:1394`, `:1396`, `:1707`, `:1729`, `:1731`, `:1742`,
  `:1762`; `echo-agent-cli/src/cli/cmd_impls/evolution.rs:185`, `:187`, `:713`,
  `:740`, `:743`, `:749`; `echo-agent-cli/src/tui/commands.rs:118`, `:123`,
  `:193`, `:198`; `echo-agent-cli/src/tui/events.rs:4031`.
- Reachability: GUI draft activation and Curator run, plus CLI `/skill-promote`
  and `/curator run`, are live explicit user actions. TUI exposes candidate/draft
  listing but no create/activate/Curator transition command.
- Expected invariant: after a successful reviewed action, file, Curator, primary,
  existing pool, future pool, and every interaction surface share one catalog
  generation; failed runtime activation does not leave durable Active state.
- Observed behavior: GUI and CLI copy the draft and persist Active before loading
  only the primary Agent. Load failure leaves the file/Curator active. Curator
  transitions also reconcile only primary. Tauri already has a pool refresh
  helper for ordinary skill-hub mutations, but evolution paths do not call it.
  TUI only lists candidates/drafts, violating the full-surface product invariant.
- Impact: a skill reported Active may execute in the primary but remain absent or
  stale in pooled Agents, or be durably active while runtime load failed. Users
  cannot complete the same reviewed evolution workflow from TUI.
- Root cause: skill evolution is duplicated in surface handlers instead of one
  app-core catalog transaction and receipt.
- Direction: create one workspace-scoped activation/transition service that
  stages file + Curator state, refreshes primary/existing/future Agent catalogs
  under one generation, compensates or reports explicit partial state, and is
  called by GUI/TUI/CLI. Delete direct surface filesystem/Curator sequences.
- Regression validation: every surface, two existing pooled Agents plus a future
  one, loader/Curator/copy failure at each step, restart, concurrent activation,
  and exact catalog generation equality.
- Validation reports: [V08](../validations/A-EVO-01/V08-01.md)

## Positive Conclusions

- BackgroundReviewer, TriggerDetector, AutoMemory, and MemoryReviewer converge on
  one bounded workspace EvidenceStore; inferred content is not written directly
  to long-term memory.
- `ReviewConfig::default()` disables session-end review and automatic draft
  generation. Memory conflict application requires an explicit inbox action and
  validates stale proposals before starting mutation.
- Evidence IDs are independent UUIDs; normalized fingerprints deduplicate
  sources without reusing identity after edits. Quotes/item counts are bounded.
- Dashboard diagnostics are on demand and bounded; they do not invoke an LLM or
  write semantic state.
- EKO does not enable framework `eval`/`improve`. This is a correct product choice,
  not evidence that the independent public framework APIs should be deleted.
- Rule and skill semantic actions are user initiated. The defects concern commit
  safety, runtime convergence, and surface parity rather than a need for extra
  online-service permission gates.

## Validation Matrix

| ID | Claim | Result | Report |
|---|---|---|---|
| V00 | Scope, commits, dependencies, and dirty-path isolation are explicit | passed | [V00-01](../validations/A-EVO-01/V00-01.md) |
| V01 | Definition/feature/export and duplicate-authority inventory | passed | [V01-01](../validations/A-EVO-01/V01-01.md) |
| V02 | Registration and runtime reachability across GUI/TUI/CLI | passed | [V02-01](../validations/A-EVO-01/V02-01.md) |
| V03 | Proposal-only and user-authorization boundaries | passed | [V03-01](../validations/A-EVO-01/V03-01.md) |
| V04 | Dreaming automatic mutation invariant | failed | [V04-01](../validations/A-EVO-01/V04-01.md) |
| V05 | Evidence accept/undo crash-consistency invariant | failed | [V05-01](../validations/A-EVO-01/V05-01.md) |
| V06 | Evidence JSONL recovery/retention invariant | failed | [V06-01](../validations/A-EVO-01/V06-01.md) |
| V07 | Rule promotion preservation/transaction invariant | failed | [V07-01](../validations/A-EVO-01/V07-01.md) |
| V08 | Skill lifecycle convergence and surface parity | failed | [V08-01](../validations/A-EVO-01/V08-01.md) |
| V09 | Product documentation versus current code | failed | [V09-01](../validations/A-EVO-01/V09-01.md) |
| V10 | Existing static test coverage inventory | passed | [V10-01](../validations/A-EVO-01/V10-01.md) |
| V11 | Dynamic fault/behavior matrix | not_run | [V11-01](../validations/A-EVO-01/V11-01.md) |
| V12 | Final report/link/executor/isolation gate | passed | [V12-01](../validations/A-EVO-01/V12-01.md) |

## Historical Classification

| Claim | Classification | Current evidence |
|---|---|---|
| Background review and auto-memory are proposal-only by default | current | Shared EvidenceStore capture, disabled session-end default, and explicit accept boundary. |
| EKO does not run local benchmark/improvement loops | current | Product manifests and runtime callers do not enable/use framework eval/improve. |
| GUI/TUI/CLI share the Dreaming schedule | current | All three startup paths call one helper. |
| Automatic metrics cannot modify prompt, skill, rule, or memory | regressed | Default Dreaming uses recall/inactivity metrics to change memory status and hot model context after startup. |
| Evidence accept/undo compensation makes Review Inbox recovery safe | incomplete | Returned persistence errors trigger compensation, but process termination has no durable operation phase/reconciliation. |
| Curator/skill lifecycle is fully shared across surfaces and live catalogs | regressed | Evolution mutations refresh primary only and TUI cannot create/activate/run Curator transitions. |

## Coverage Gaps And Residual Risk

- Dynamic crash/fault injection was intentionally not run. Static crash windows
  are explicit; exact filesystem/Store behavior after injected failure remains a
  future implementation validation.
- Framework source had unrelated concurrent changes. A-EVO conclusions use
  accepted F-EVO/A-MEM reports and committed `HEAD`; current dirty framework
  contents were not inspected.
- General hot-memory refresh defects remain owned by A-MEM-01. Fixing only those
  projections would not repair Dreaming authorization or Evidence transactions.
- F-EVO-01 owns framework merge/draft/Curator internals. EKO still needs its own
  operation journal and catalog convergence even after framework fixes.

## Iteration Direction

1. **P0 preservation first:** replace rule promotion with a workspace-bound,
   fail-closed, atomic/journaled application service; preserve all original bytes.
2. **One evolution operation protocol:** persist operation identity, source
   generation, before state, phase, terminal receipt, and recovery for Evidence,
   rule, and skill mutations. Reuse framework mutators behind a thin adapter.
3. **Remove hidden mutation:** change EKO Dreaming scheduling to durable proposals
   or an explicit opt-in automation policy with audit/undo; keep generic framework
   Dreaming available to other consumers.
4. **One catalog generation:** publish accepted skill transitions to primary,
   existing pool, future pool, GUI, TUI, and CLI; delete duplicated handlers.
5. **Harden authority storage:** frame and checkpoint Evidence state with safe tail
   recovery and bounded compaction before adding more evolution features.

## Handoff

- Synthesis should treat A-EVO-01-P0-01 as the application data-preservation
  owner, A-EVO-01-P1-02 as the hidden automatic-mutation owner, and link rather
  than duplicate F-EVO-01-P1-09/A-MEM projection defects.
- Downstream surface/state work should consume a typed evolution operation
  receipt rather than infer success from files, Curator state, or live hooks.
- Re-review triggers: any change to Dreaming startup/config, Evidence persistence,
  rule promotion, skill activation/Curator commands, catalog refresh, workspace
  rebinding, or framework mutation receipts.
