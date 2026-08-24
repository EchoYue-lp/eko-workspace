# F-EVO-01: Eval, improvement, and evolution framework APIs

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at primary acceptance; the
> mid-review external test/mock commit did not change this task's source paths

## Question

Are eval/improve/evolution capabilities valid optional framework APIs with
explicit side effects, reliable review/mutation/rollback boundaries, and no
coupling to EKO product policy?

## Scope

- Root `eval`: cases, runner/fixtures/SWE-bench, criteria/grader, replay,
  regression, trigger metrics, comparator and reports.
- Root `improve`: analyzer, prompt generator, analysis loop/facade, trajectory
  JSONL and reports.
- Root `evolution`: review/proposals, candidate/draft, curator, audit, skill and
  memory merge/patch, background review, triggers/security/runtime wiring.
- Features, exports, examples, tests, scoped history and live EKO adapters needed
  to establish definition, authority and reachability.

## Out Of Scope

- Source fixes and all Cargo/rustc/test/build/dynamic fixture/network execution.
- Generic Store defects already owned by F-MEM-01, except evolution-specific
  multi-record transaction composition.
- EKO's complete product authorization/UI behavior, deferred to A-EVO-01.
- Generic ReAct cancellation/tool semantics and full secret policy.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex protocol
  and report templates.
- [F-FEAT-01](F-FEAT-01.md): accepted feature/export/standalone evidence, reused
  without rerunning compilation.
- [F-MEM-01](F-MEM-01.md): accepted Store/memory authority boundary, used to
  avoid duplicating generic persistence findings.
- Current source and scoped Git history. No other reviewer report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Eval cases/runners/criteria, trace analysis, explicit trajectory export, proposal-only review, typed mutation primitives, stale-source checks, audit and recovery are reasonable reusable framework APIs independent of EKO callers. |
| EKO product policy | Workspace evidence inbox, user accept/reject/undo interactions, concrete `.eko` paths, UI/TUI/CLI projections, automatic-review settings and hook policy remain application-owned. |
| Adapter boundary | EKO may bind framework proposals to workspace/source identity and call explicit mutators; it must not compensate for a framework mutator that declares success before its authoritative state is durable. |
| Duplicate search | Searched features/cfg/exports/examples; eval/improve/evolution type and behavior names; fixture/cwd/criteria; proposal/review/mutation/rollback/audit/curator; store/source/revision/actor/evidence; all callers across both repositories. |
| Migration deletion | Retain public framework options. Replace misleading facades and partial mutation bodies; remove inert fields/dead Store ownership and obsolete success claims when one durable authority is wired. Do not move EKO evidence/UI policy into the framework. |

## Current Path

```text
standalone framework eval consumer
  -> EvalCase -> EvalRunner fixture -> Agent::execute(task)
  -> criteria(cwd, optional trace) -> EvalResult/Report

framework improvement consumer
  -> EvalDrivenImprovement -> ImprovementLoop
  -> repeated unchanged Agent factories -> critiques/suggestions -> reports

framework/EKO evolution
  -> trace/memory evidence -> proposal-only reviewers/detectors
  -> EKO workspace EvidenceStore + user interaction policy
  -> explicit MemoryLayerManager / DraftGenerator / Patcher / Merger mutators
  -> Store/files + Curator + JsonlChangeLog (currently sequential authorities)
```

Positive invariants worth retaining: `eval`/`improve` feature isolation is valid;
trajectory export is explicitly opt-in; MemoryReviewer is analysis-only;
BackgroundReviewer is proposal-only by default; memory conflict proposals recheck
current content/status/confidence before mutation and return exact snapshots; EKO
correctly supplies workspace-scoped evidence and Curator policy.

## Findings

### F-EVO-01-P0-01: Eval case identity can delete a directory outside the workspace

- Priority: P0; confidence: high; layer: framework.
- Evidence: `echo-agent/src/eval/mod.rs:66`, `echo-agent/src/eval/runner.rs:177`.
- Reachability: any deserialized/public EvalCase with `project_fixture` reaches
  `setup_fixture`; `PathBuf::join` accepts absolute and parent-containing IDs.
- Expected invariant: fixture cleanup is contained beneath canonical workspace root.
- Observed behavior: destination is `workspace_root.join(case.id)` and an existing
  destination is passed to `remove_dir_all`; containment and cleanup errors are ignored.
- Impact: malformed case identity can recursively delete unrelated local data.
- Root cause: display identity is reused as an unchecked filesystem path.
- Direction: derive an opaque/sanitized workspace component, canonicalize parent
  and target, verify containment before any removal, and return setup failure.
- Regression validation: absolute, `..`, separators, symlinks, Unicode, cleanup
  failure, and prove no path outside a temporary root changes.
- Validation reports: [V03](../validations/F-EVO-01/V03-01.md)

### F-EVO-01-P0-02: Direct skill candidates can overwrite SKILL.md outside the draft root

- Priority: P0; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/draft.rs:99`, `:108`.
- Reachability: `SkillDraftGenerator::generate_from_candidate` is public and live
  in EKO review/commands; it accepts a public/deserializable SkillCandidate.
- Expected invariant: every generated draft is contained under `_drafts`.
- Observed behavior: candidate name is joined directly as a path component before
  create/write, so absolute or parent-containing names escape the configured root.
- Impact: malformed candidate data can overwrite an unrelated `SKILL.md`.
- Root cause: logical candidate name is used as filesystem authority without validation.
- Direction: require one validated relative component or opaque candidate ID and
  verify canonical containment before staging any file.
- Regression validation: absolute/parent/separator/empty/Unicode/collision names
  and unchanged files outside the temporary root.
- Validation reports: [V09](../validations/F-EVO-01/V09-01.md)

### F-EVO-01-P1-01: Eval Agent execution is detached from its prepared fixture

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/eval/runner.rs:51`, `:177`, `:333`.
- Reachability: all fixture cases and public SweBench criteria use this runner.
- Expected invariant: setup occurs before Agent execution and Agent plus criteria
  share the same explicit working directory.
- Observed behavior: runner computes cwd but calls only `agent.execute(task)`;
  criteria alone receive cwd. SweBench clones/checks out only after Agent returns.
- Impact: coding evals measure work in the wrong repository; SWE-bench is unusable
  as implemented and reused directories add cross-run nondeterminism.
- Root cause: Agent contract/case setup lacks an invocation context carrying cwd,
  and repository initialization was placed in post-execution criteria.
- Direction: initialize/reset one isolated repository first and invoke through an
  explicit working-directory context; delete post-execution setup/fallback-to-root.
- Regression validation: recording Agent cwd/file mutation, setup failure, two
  consecutive cases, clean reset, timeout/cancel and SWE-bench order.
- Validation reports: [V03](../validations/F-EVO-01/V03-01.md)

### F-EVO-01-P1-02: Grader and trigger metrics can report false or out-of-range success

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/eval/grader.rs:135`,
  `echo-agent/src/eval/trigger.rs:50`, `:116`.
- Reachability: public LlmGraded criteria and TriggerAccuracy APIs.
- Expected invariant: one result per expected identity, complete cardinality,
  finite scores within range, and wrong/missing routing counts as failure.
- Observed behavior: duplicate/unknown passed assertion IDs increase pass rate,
  missing rows are dropped by zip, wrong target increments TP, and an empty
  multi-run row divides by zero.
- Impact: invalid envelopes can pass gates or emit NaN/misleading routing quality.
- Root cause: metrics count unvalidated rows instead of joining by authoritative IDs.
- Direction: validate unique exact ID sets/cardinality and finite confidence;
  derive confusion facts for every expected case and reject malformed run counts.
- Regression validation: duplicate/unknown/missing IDs, result length mismatch,
  wrong target, zero/mismatched multi-runs, NaN/inf and score range.
- Validation reports: [V04](../validations/F-EVO-01/V04-01.md)

### F-EVO-01-P1-03: EvalDrivenImprovement does not evaluate an improved candidate

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/improve/eval_improvement.rs:30`, `:81`,
  `echo-agent/src/improve/loop.rs:102`.
- Reachability: public facade exported under `improve+eval`; module/docs advertise
  iterative prompt improvement.
- Expected invariant: public configuration changes execution and each iteration
  evaluates an explicitly updated, reviewable Agent candidate.
- Observed behavior: facade `max_iterations` is never applied; loop generates
  suggestions but calls the unchanged factory on train/test and later iterations.
  Its RunStore is not attached to EvalRunner, invalidating trace criteria.
- Impact: reports can label stochastic repetition as a best improvement and Tool
  criteria fail regardless of the actual run.
- Root cause: analysis and mutation/application are composed only in prose.
- Direction: expose analysis-only naming/result, or accept an explicit
  human-reviewed candidate update callback and candidate identity per iteration;
  delete inert options and iterative claims until wired.
- Regression validation: factory/prompt identity per iteration, max=1/N,
  trace-dependent criteria, no-change analysis mode, report I/O failure.
- Validation reports: [V05](../validations/F-EVO-01/V05-01.md)

### F-EVO-01-P1-04: Public edge inputs contain direct panic and UTF-8 violations

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/improve/loop.rs:78`,
  `echo-agent/src/eval/regression.rs:79`, `echo-agent/src/eval/runner.rs:724`.
- Reachability: one-case criteria groups, public trace run IDs, and ValueMatch keys/output.
- Expected invariant: every input shape and Unicode string returns a result without panic.
- Observed behavior: singleton split calls `clamp(1, 0)`; RegressionSuite byte-slices
  run ID; numeric fallback reuses a lowercased-string byte offset on original text.
- Impact: ordinary small eval sets or Unicode trace/output data can crash review/eval.
- Root cause: unchecked split bounds and byte offsets are treated as character identity.
- Direction: define singleton train/holdout semantics, use `.chars()`/safe match
  ranges on the original string, and use checked/saturating numeric aggregation.
- Regression validation: 0/1/2 per criteria group; every Unicode boundary and
  case-fold length change; extreme counters/durations.
- Validation reports: [V04](../validations/F-EVO-01/V04-01.md),
  [V05](../validations/F-EVO-01/V05-01.md), [V13](../validations/F-EVO-01/V13-01.md)

### F-EVO-01-P1-05: ChangeLog cannot satisfy its advertised complete rollback/durability contract

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/audit.rs:1`, `:56`, `:82`, `:245`.
- Reachability: all layer/draft/candidate/patch/merge mutations accept ChangeLog;
  EKO constructs JsonlChangeLog in live paths.
- Expected invariant: every successful mutation has durable causal before/after
  evidence and a recovery operation; lock/fsync failure returns error.
- Observed behavior: trait has no rollback, most skill entries omit states/source
  identity, lock exhaustion appends unlocked, fsync error is discarded, and
  per-handle caches become stale.
- Impact: recovery/audit cannot reconstruct changes and may report incomplete or
  corrupted history after contention/crash.
- Root cause: log is an append convenience after mutation, not a transaction or
  authoritative recovery record.
- Direction: define typed mutation/proposal/source/revision/actor/undo records;
  fail closed on lock/sync; couple success to audit or compensate. Remove rollback
  and durability claims until the contract exists.
- Regression validation: lock contention/exhaustion, fsync failure, two handles,
  crash points, every mutation round-trip and rollback.
- Validation reports: [V08](../validations/F-EVO-01/V08-01.md)

### F-EVO-01-P1-06: Candidate and draft state is nondeterministic and partially committed

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/candidate.rs:214`, `:230`, `:278`,
  `echo-agent/src/evolution/draft.rs:57`, `:108`.
- Reachability: EKO ReviewIntegration runs detection and optionally draft generation.
- Expected invariant: identical evidence selects identical bounded candidates;
  Store, workspace Curator, draft file and audit agree or no mutation occurs.
- Observed behavior: randomized HashMap iteration selects capped groups; sanitized
  topic collisions share identity; read errors become absence; Store/Curator/audit
  commit sequentially with some failures warned. Draft default Curator is global,
  and file write precedes lifecycle/audit failure.
- Impact: runs choose different candidates, unrelated evidence can merge, and
  partial state/file updates survive returned errors.
- Root cause: display-derived identity and multiple independent authorities lack
  stable ordering, workspace binding and transaction staging.
- Direction: stable sort and collision-resistant source ID, propagate reads,
  require consumer Curator, and stage one recoverable commit; delete global
  fallback and best-effort lifecycle writes from mutation APIs.
- Regression validation: randomized input order, sanitization collision, Store/
  Curator/audit fault at each step, restart and exact state equality.
- Validation reports: [V09](../validations/F-EVO-01/V09-01.md)

### F-EVO-01-P1-07: Skill patch proposals are stale-prone and concurrent apply can lose updates

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/patch.rs:257`,
  `echo-agent/src/evolution/security.rs:467`.
- Reachability: live EKO CLI applies generated patches to registered SKILL.md files.
- Expected invariant: approved proposal binds to exact source revision, one writer
  owns the file, and source plus audit commit or roll back together.
- Observed behavior: proposal has no source hash/revision; apply reads then writes
  a shared fixed temp path without lock, renames source before audit, and records
  no before/after. The exported patch check has no production caller.
- Impact: concurrent/stale approvals can overwrite newer edits; returned audit
  error hides an already-applied patch and leaves no general undo payload.
- Root cause: proposal analysis and file transaction identities are disconnected.
- Direction: bind proposal to canonical path/hash/revision, serialize apply, stage
  audited before/after, and compensate on failure; integrate any automatic-mutation
  guard at the mutator boundary without gating direct user interaction.
- Regression validation: stale source, two concurrent patches, temp collision,
  rename/log failure, retry/idempotence and full Unicode content.
- Validation reports: [V10](../validations/F-EVO-01/V10-01.md)

### F-EVO-01-P1-08: Skill merge reports success without persisting the primary skill

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/merge.rs:214`,
  `echo-agent-cli/src/cli/cmd_impls/evolution.rs:1002`.
- Reachability: live EKO `/skill-merge ... execute` calls the public merger.
- Expected invariant: primary descriptor is durably updated before secondary is
  deprecated and success is returned.
- Observed behavior: Store is dead, only a caller clone is mutated, Curator result
  is ignored, audit records success, and EKO drops the clone while telling the user
  the primary must still be written manually.
- Impact: secondary can be marked deprecated while the primary lacks its triggers/
  tools/paths; reload loses the purported merge.
- Root cause: a pure descriptor merge, lifecycle mutation and persistence were
  combined behind one misleading execute API with no authoritative store writer.
- Direction: separate deterministic pure merge from one caller-owned durable
  commit, or persist canonical source transactionally; delete dead Store field,
  ignored Curator result and false completion text.
- Regression validation: reload after merge, missing/failed Curator, audit failure,
  stale proposal, deterministic ordering and secondary availability.
- Validation reports: [V10](../validations/F-EVO-01/V10-01.md)

### F-EVO-01-P1-09: Memory merge snapshot recovery is not automatic or atomic

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/layer.rs:563`, `:654`,
  `echo-agent/src/evolution/review.rs:386`.
- Reachability: EKO evidence acceptance calls `apply_merge_proposal` and stores
  returned snapshots for undo.
- Expected invariant: stale review is rejected and multi-record mutation/audit
  either fully commits or automatically restores exact before state.
- Observed behavior: stale checks are sound, but primary and secondaries are
  written/audited sequentially; any intermediate failure returns without automatic
  compensation. Restore is also sequential. Revision sum can overflow.
- Impact: accepted or undone merges can leave mixed Active/Superseded facts and
  incomplete audit even though exact snapshots existed before mutation.
- Root cause: recovery payload is returned only after a nontransactional loop,
  instead of being owned by a transaction guard during the loop.
- Direction: stage snapshots first, use commit/compensation with resumable member
  progress and checked counts; surface partial state explicitly if compensation fails.
- Regression validation: failure at every Store/audit member, process restart,
  repeated undo/retry, concurrent stale update and u32 maximum.
- Validation reports: [V11](../validations/F-EVO-01/V11-01.md)

### F-EVO-01-P1-10: Curator corruption is converted to empty state and overwritten

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/evolution/curator.rs:111`, `:254`.
- Reachability: every Curator mutation loads state inside `with_locked_state`; EKO
  supplies workspace Curators and framework defaults also use this implementation.
- Expected invariant: present unreadable/unparseable state blocks mutation and
  preserves recoverable bytes.
- Observed behavior: load logs then returns default; the next dirty mutation saves
  that empty/partial state over the existing file.
- Impact: one malformed/torn curator file can erase all lifecycle identities on
  the next normal operation.
- Root cause: missing-state and corrupt-state share a value-returning load API.
- Direction: return typed Missing versus Corrupt, quarantine/preserve corrupt bytes,
  and refuse mutation until recovery; delete default-on-corruption behavior.
- Regression validation: malformed JSON, unreadable file, subsequent touch/
  transition, preserved original bytes and explicit recovery.
- Validation reports: [V09](../validations/F-EVO-01/V09-01.md),
  [V14](../validations/F-EVO-01/V14-01.md)

### F-EVO-01-P2-01: Prompt/report facades hide operational failure as success

- Priority: P2; confidence: high; layer: framework.
- Evidence: `echo-agent/src/improve/generator.rs:39`,
  `echo-agent/src/improve/eval_improvement.rs:95`.
- Reachability: public PromptGenerator and enabled EvalDrivenImprovement.
- Expected invariant: LLM/format/report I/O failure remains distinguishable from
  a legitimate unchanged prompt or completed report.
- Observed behavior: generator returns current prompt for Agent error or missing
  tags; report directory/write failures only warn while facade returns Some result.
- Impact: automation cannot tell “no change” from failed generation or incomplete artifacts.
- Root cause: fallible side effects are exposed through String/Option success shapes.
- Direction: return typed generation/report outcomes with unchanged-as-data and
  artifact paths/errors; delete silent fallback-as-success.
- Regression validation: Agent error, malformed/multiple tags, zero max chars,
  unwritable directory and partial report writes.
- Validation reports: [V05](../validations/F-EVO-01/V05-01.md)

### F-EVO-01-P2-02: Trajectory JSONL is not an authoritative training-data record

- Priority: P2; confidence: high; layer: framework.
- Evidence: `echo-agent/src/improve/trajectory.rs:25`, `:186`, `:235`, `:264`.
- Reachability: public trajectory option and demo, independent of EKO.
- Expected invariant: append rows do not interleave, corruption is observable,
  lineage is reconstructable, and aggregates cannot overflow.
- Observed behavior: appends have no shared/process lock, malformed rows vanish,
  parent/agent/provider/turn/execution lineage is dropped, Tool output is preview
  only without artifact identity, and stats use unchecked sums.
- Impact: fine-tuning/export consumers can train on silently incomplete,
  misattributed or corrupted records and report incorrect aggregates.
- Root cause: JSONL was modeled as convenience export without dataset manifest,
  locking, diagnostics or typed provenance.
- Direction: append under one durable writer/lock, expose corrupt records, retain
  bounded provenance/artifact references and use checked aggregates.
- Regression validation: concurrent writers, torn/malformed line, full lineage,
  truncated artifact reference, cancelled/failed runs and maximum counters.
- Validation reports: [V06](../validations/F-EVO-01/V06-01.md),
  [V13](../validations/F-EVO-01/V13-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Feature/export and independent option value | yes | passed | [V01](../validations/F-EVO-01/V01-01.md) |
| V02 | Definition, reachability and ownership trace | yes | passed | [V02](../validations/F-EVO-01/V02-01.md) |
| V03 | Eval fixture/workspace/SWE-bench side effects | yes | failed | [V03](../validations/F-EVO-01/V03-01.md) |
| V04 | Grader/trigger/regression metric invariants | yes | failed | [V04](../validations/F-EVO-01/V04-01.md) |
| V05 | Improvement option/iteration/trace flow | yes | failed | [V05](../validations/F-EVO-01/V05-01.md) |
| V06 | Trajectory persistence/provenance/aggregate | yes | failed | [V06](../validations/F-EVO-01/V06-01.md) |
| V07 | Review versus mutation authority | yes | passed | [V07](../validations/F-EVO-01/V07-01.md) |
| V08 | Audit/rollback/durability/source identity | yes | failed | [V08](../validations/F-EVO-01/V08-01.md) |
| V09 | Candidate/draft identity and transaction | yes | failed | [V09](../validations/F-EVO-01/V09-01.md) |
| V10 | Skill patch/merge reviewed mutation | yes | failed | [V10](../validations/F-EVO-01/V10-01.md) |
| V11 | Memory merge stale/rollback transaction | yes | failed | [V11](../validations/F-EVO-01/V11-01.md) |
| V12 | EKO adapter and duplicate-authority boundary | yes | passed | [V12](../validations/F-EVO-01/V12-01.md) |
| V13 | Panic/UTF-8/overflow scan | yes | failed | [V13](../validations/F-EVO-01/V13-01.md) |
| V14 | Existing tests and scoped history | yes | passed | [V14](../validations/F-EVO-01/V14-01.md) |
| V15 | Initial integrity/source-dirty disclosure gate | yes | passed | [V15](../validations/F-EVO-01/V15-01.md) |
| V16 | Final integrity after concurrent dirty change | yes | passed | [V16](../validations/F-EVO-01/V16-01.md) |
| V30 | Primary current-commit acceptance | yes | passed | [V30](../validations/F-EVO-01/V30-01.md) |

No executable fixture was run because the review track and user explicitly ban
new Cargo/rustc/test/build/dynamic fixture/network work. Containment, deterministic
ordering, fault injection and concurrency tests are future implementation
validations, not fake `not_run` attempt reports.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `c71351a` separate review from mutation | current | MemoryReviewer is analysis-only; [V07](../validations/F-EVO-01/V07-01.md) |
| `3b5601e` Curator TOCTOU/deadlock fix | current but incomplete | locked read-modify-write is present, while corruption still defaults/overwrites; [V09](../validations/F-EVO-01/V09-01.md) |
| `fce2f3c` SkillPatcher apply API | current but incomplete | live apply exists without stale/concurrent transaction; [V10](../validations/F-EVO-01/V10-01.md) |
| Evolution docs: complete audit with rollback | regressed/unsupported | ChangeLog has no rollback and skill entries omit undo state; [V08](../validations/F-EVO-01/V08-01.md) |
| Improve docs: iterative prompt optimization | stale for current facade | unchanged factory is repeatedly evaluated; [V05](../validations/F-EVO-01/V05-01.md) |

## Coverage And Uncertainty

- Source-conclusive static inspection covered all eval/improve modules and all
  evolution modules, with deeper call traces on externally mutating paths.
- No command/build/test/fixture/network validation was executed. Runtime-specific
  filesystem error codes, crash timing and concurrency schedules remain future tests.
- Existing tests were inventoried, not run. Accepted F-FEAT-01 supplies prior
  compile isolation evidence at the same reviewed framework commit.
- Evaluation timeout cancellation beyond dropping `Agent::execute` was not
  re-audited here; canonical invocation cancellation belongs to ReAct tasks.
- A-EVO-01 must decide product authorization and cross-surface behavior. This
  task establishes that EKO non-use never justifies deleting a reasonable public API.

## Handoff

- Preserve the feature/export split and proposal-only MemoryReviewer/BackgroundReviewer.
- Fix the two contained-path P0s before using external eval/candidate files.
- Make improvement naming/options truthful before tuning algorithms.
- Treat each mutation as source-revision + staged change + durable audit +
  compensation, while keeping EKO workspace/user review policy in the application.
- Downstream A-EVO-01 must read V07, V10 and V12 plus this task report.
- This report becomes stale after changes to eval fixture invocation, improvement
  facade composition, ChangeLog/Curator, candidate/draft, patch/merge or EKO evidence adapters.
- Primary acceptance at `3aa79299` is recorded in V30; the intervening commit had
  no diff in eval/improve/evolution source.
