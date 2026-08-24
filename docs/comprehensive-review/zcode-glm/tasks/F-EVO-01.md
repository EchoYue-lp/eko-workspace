# F-EVO-01: Eval, improvement, and evolution framework APIs

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only inspection + targeted `cargo check`/`cargo test`)

## Question

Are eval/improve/evolution capabilities valid optional framework APIs
with explicit side effects and without coupling to EKO product policy?

## Scope

Primary source paths and behaviors inspected (read-only unless noted):

- `echo-agent/src/eval/` — all 9 files (`mod.rs`, `runner.rs`, `grader.rs`,
  `comparator.rs`, `regression.rs`, `replay.rs`, `report.rs`, `server.rs`,
  `trigger.rs`); 2905 lines.
- `echo-agent/src/improve/` — all 6 files (`mod.rs`, `trajectory.rs`,
  `analyzer.rs`, `eval_improvement.rs`, `generator.rs`, `loop.rs`); 1453
  lines.
- `echo-agent/src/evolution/` — all 17 files; 11096 lines.
- `echo-agent/src/lib.rs` — module gating for `eval`/`improve`/`evolution`.
- `echo-agent/Cargo.toml` — `[features]` (`eval = []`, `improve = []`, no
  `evolution` feature) and `examples/` `required-features`.
- `echo-agent/src/agent/react/run/context.rs` and `react_loop.rs` — where
  the react engine consumes `evolution::MemoryLayerManager` and the trigger
  detector during a run (the live mutation surface).
- `echo-agent/src/tools/builtin/memory.rs` — `remember`/`forget` tools that
  route through `MemoryLayerManager`.
- `echo-agent-cli/echo-agent-app-core/src/evolution/` — the EKO adapter
  layer (`review_integration.rs`, `evidence.rs`, `rule_promoter.rs`,
  `dashboard.rs`, `hook_fire.rs`) and its wiring in `agent_pool.rs`,
  `runtime.rs`, `infra.rs`, `state.rs`.
- Executable checks: `cargo check -p echo_agent --no-default-features
  --features {eval, improve, eval,improve} --locked`; `cargo test -p
  echo_agent --features eval --locked eval::` (14 tests).

## Out Of Scope

Deferred to named task IDs:

- Full memory-authority unification between `memory_promoter`
  (compression-driven L3 salvage) and `evolution::MemoryLayerManager`
  → `F-MEM-01` (the listed dependency owns memory store/promoter
  authority). F-EVO-01 records the overlap as a coverage note and
  handoff only.
- Per-feature standalone compile matrix for the WHOLE feature menu →
  `Q-FW-02` and the AGENTS.md "条件矩阵". F-EVO-01 runs only the three
  eval/improve combinations directly relevant to the task.
- The EKO Review Inbox / Evidence Store product behavior (UI, persistence
  of `EvidenceCandidate`) → application-layer tasks (`A-*`).
- Skill lifecycle promotion policy (Draft→Active thresholds) product
  tuning → application-layer.

## Inputs

- Repository documents read: root `AGENTS.md` (sections "产品定位与安全边界",
  "统一术语", "删除框架代码的判定", "实现前门禁", "条件矩阵"; the
  "No self-evolution metric platform. Evolution = explicit diagnostics and
  user-triggered review only." line is the V03 anchor), `docs/comprehensive-
  review/README.md`, `REPORTING.md`, both report templates, the `F-EVO-01`
  card in `TASKS.md`.
- Dependency task reports read: `F-FEAT-01` (feature topology; established
  that `eval = []` and `improve = []` are live marker features, and that the
  CLI enables `default-features = false` excluding both — see F-FEAT-01
  Current Path and Handoff).
- Cross-referenced for memory-authority context: `F-MEM-01` task card
  (dependency; not yet read as a report — it is pending).
- Historical documents treated as hypotheses: none.

## Layering Decision

F-EVO-01 spans framework and adapter layers. The layering conclusions:

- **Generic mechanism** (framework, keep): `eval` (case definition, runner,
  scorers, regression-from-traces, replay contract), `improve` (trajectory
  export + eval-driven critique/suggestion loop), and the bulk of
  `evolution` (typed memory primitives, change audit, security guard,
  staleness/conflict/health scorers, skill candidate/merge/patch
  proposals, dreaming maintenance pass) are all generic agent capabilities.
  None reference EKO, Tauri, app-core, or any product type (verified by
  repository-wide grep — see V02). A non-EKO consumer can use any of them
  against the framework's own `Agent`/`Store`/`RunStore` traits.
- **EKO product policy** (application): the Review Inbox / Evidence Store,
  the `MemoryTriggerSink` that routes triggers to review
  (`ReviewIntegration::on_trigger` returns `Captured`), the `RulePromoter`,
  Dreaming scheduling (`infra.rs` "Spawn Dreaming after boot settles, then
  repeat it on a daily cadence"), and the `evolution run` CLI command are
  all application-layer and live in `echo-agent-cli/echo-agent-app-core/`.
  They correctly consume the framework primitives; none leak back into the
  framework.
- **Adapter boundary**: `ReviewIntegration` (app) implements the framework's
  `MemoryTriggerSink` trait and `SkillLoadPolicy` trait — thin trait
  implementations with no rescheduling, no second layer manager, no second
  audit log. Conversion is lossless (trigger → `EvidenceCandidateDraft`).
  `HookEvolutionObserver` (framework) publishes framework events through a
  `HookRegistry`; it does not mutate. The adapter boundary is clean.

Repository-wide duplicate-search terms used: `cfg(feature = "eval")`,
`cfg(feature = "improve")`, `cfg(feature = "evolution")`, `eko`,
`echo.agent.cli`, `echo_agent_cli`, `app.core`, `app_core`, `tauri`,
`use crate::eval`, `use crate::improve`, `use crate::evolution`,
`MemoryLayerManager`, `write_memory`, `memory_promoter`,
`extract_observations`. Results: zero EKO/product-type references inside
`echo-agent/src/{eval,improve,evolution}`; one live overlap between
`memory_promoter` and `evolution::MemoryLayerManager` (both write memories
during runs — see Coverage And Uncertainty and handoff to F-MEM-01).

## Current Path

### Feature wiring (commit `9b0e0fa`)

- `eval` and `improve` are empty marker features (`Cargo.toml:95-96`).
  `eval` gates `pub mod eval` (`lib.rs:36-38`) and the eval-driven half of
  `improve` (`improve/mod.rs:42-62`, 13 `cfg(feature = "eval")` gates on
  `analyzer`/`eval_improvement`/`generator`/`loop` and their re-exports).
  `improve` gates `pub mod improve` (`lib.rs:43-45`); its base
  `trajectory` submodule is available with `improve` alone. `evolution` is
  NOT feature-gated — `pub mod evolution` at `lib.rs:40` is unconditional
  (0 `cfg(feature = "evolution")` matches workspace-wide). See V01.
- The CLI consumes `echo_agent` with `default-features = false` and enables
  neither `eval` nor `improve` (app-core `Cargo.toml:10-15`; root CLI
  `Cargo.toml:50`). So `eval` and `improve` are framework-only options here;
  they are NOT compiled into EKO. `evolution` is compiled into EKO because
  it is unconditional.
- `examples/demo50_eval.rs` (`required-features = ["eval"]`) and
  `examples/demo51_self_improvement.rs` (`required-features = ["eval",
  "improve"]`) are the only in-tree consumers of those features.

### Mutation surface (the V03 crux)

`evolution` exposes a clean **propose → apply** split for skills and
semantic memory:

- Proposals (analysis output, no side effect): `SkillMergeProposal`
  (`merge.rs:34`, via `scan_and_propose`), `SkillPatch` (`patch.rs`,
  via `analyze_and_propose`), `MemoryConflictProposal` (`review.rs:266-277`,
  documented "Analysis-only proposal describing a conflict that needs an
  explicit choice"), `ReviewCandidate`/`ReviewOutcome`
  (`background_review.rs:128-152`, "default behavior is proposal-only"),
  `TriggerMatch` (`triggers.rs`, detection only), `SkillCandidate`
  (`candidate.rs`), `SkillHealthReport`/`StalenessReport` (read-only
  scores).
- Explicit apply primitives (pub, take a proposal, consumer-invoked):
  `SkillMerger::execute_merge`, `SkillPatcher::apply_patch`,
  `MemoryReviewer::merge_group`, `Curator::promote_to_{draft,active}`,
  `Curator::apply_transitions`, `MemoryLayerManager::{write_memory,
  promote, demote, delete_memory}`. All mutations flow through the audit
  log (`JsonlChangeLog`) and the `EvolutionSecurityGuard` (secret scan +
  injection detection + trust assignment) inside `write_memory`
  (`layer.rs:843-886`).

BUT the framework's **react engine** auto-invokes the memory-write
primitives during a run when a `MemoryLayerManager` is installed. Two
paths, both in `echo-agent/src/agent/react/run/context.rs`:

1. **Trigger auto-write** (`context.rs:34-137`, called from
   `react_loop.rs:558`, `context.rs:509,579`). `TriggerDetector::detect`
   runs synchronously on each user message; for each `TriggerMatch` the
   engine asks the optional `memory_trigger_sink` for a disposition. If
   the sink returns `Captured`, the framework skips its write. If there
   is **no sink** (the framework default), or the sink returns `Persist`,
   the engine calls `layer_manager.write_memory(...)` directly
   (`context.rs:100-101`). EKO installs `ReviewIntegration` as the sink
   (`agent_pool.rs:924`, `runtime.rs:256`) which **always returns
   `Captured`** and routes the trigger to the Evidence Inbox
   (`review_integration.rs:277-321`, with the comment "EKO treats
   inferred memory as review-only. Do not let an inbox storage failure
   fall through to the framework's direct durable write path").
2. **Pre-compaction flush** (`context.rs:676-798`). When
   `ContextManager::should_compress()` is true, the engine makes a
   bounded (15s) LLM call to extract durable facts from the about-to-be-
   compressed transcript and calls `layer_manager.write_memory(...)`
   directly for each fact (`context.rs:794`). This path is gated only by
   `self.llm_client` and `self.memory_layer_manager` being `Some`
   (`context.rs:681-683`). It is **NOT** intercepted by
   `memory_trigger_sink`. So in EKO's deployment, triggers are
   review-gated but pre-compaction facts are auto-written to the warm
   layer.

Both auto-write paths land in the warm typed-namespace and go through the
security guard + audit log. Warm→hot promotion (which would put a memory
into `MEMORY.md` / the system-prompt stable prefix) still requires
`consider_promotion`'s deterministic confidence+stability gate
(`layer.rs:884-886`); the auto-writes themselves do not force promotion.
Rule promotion (`RulePromoter`) and skill promotion are application-layer
and proposal-based; no framework path auto-mutates rules or skills.

## Findings

### F-EVO-01-P2-01: React engine auto-writes memory during runs, in tension with the "evolution = diagnostics-only / no automatic memory mutation" boundary

- Priority: P2
- Confidence: high (fact), medium (severity — depends on reading of AGENTS.md)
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/context.rs:34-137` (trigger detect +
    write), `:676-798` (pre-compaction flush + write), `:100-101`,
    `:794`.
  - `echo-agent/src/agent/react/run/react_loop.rs:558` (call site in the
    react loop).
  - `echo-agent/src/evolution/layer.rs:837-886` (`write_memory`, the
    primitive both paths invoke).
  - `echo-agent-cli/echo-agent-app-core/src/evolution/review_integration.rs:277-321`
    (EKO sink returns `Captured`; triggers review-gated in EKO).
- Reachability: definition (`MemoryLayerManager::write_memory`,
  `TriggerDetector::detect`) → registration (react engine field
  `memory_layer_manager: Option<...>` on `ReactAgent`, set by
  `install_memory_layer_manager`) → live caller
  (`detect_and_write_memory_triggers` and `pre_compaction_flush` inside
  the always-compiled react loop). Both compile (`--features` eval check
  is irrelevant — this is core) and run in EKO because the app installs a
  layer manager (`agent_pool.rs:672,923`, `state.rs:970`).
- Expected invariant: AGENTS.md states "No self-evolution metric
  platform. Evolution = explicit diagnostics and user-triggered review
  only." and the task's V03 rephrases it as "no automatic
  memory/rule/skill mutation." A framework whose react engine writes
  memory each turn without a per-write user action sits on the boundary
  of that statement.
- Observed behavior:
  - Trigger path: framework default (no sink) **auto-writes** every
    detected trigger to the warm layer. EKO overrides to review-only via
    its sink, so EKO's trigger behavior is compliant — but the
    *framework default* is auto-write.
  - Pre-compaction path: **always auto-writes** extracted facts directly,
    with no sink interception. This runs in EKO's deployment too.
- Impact: two distinct. (1) For non-EKO consumers that install a
  `MemoryLayerManager` but no sink, the framework silently auto-persists
  memories every turn — surprising for an API documented as
  "diagnostics-only". (2) For EKO, triggers are correctly review-gated,
  but pre-compaction facts bypass the Review Inbox and are written
  directly. Neither path causes data loss (writes are atomic, security-
  scanned, audit-logged) and neither auto-promotes to the system prompt,
  so the user-facing blast radius is bounded — hence P2 rather than P1.
- Root cause: evolution's stated boundary ("diagnostics + user-triggered
  review") was drawn at the `evolution/` module level, but the react
  engine (core) wires `MemoryLayerManager` into the hot path for
  compression salvage and trigger persistence. The "diagnostics-only"
  label describes the `evolution/` API surface, not the full system
  behavior once a layer manager is installed.
- Direction (pick one, do NOT implement in this review):
  (a) Make the documented boundary precise: state explicitly in
  `evolution/mod.rs` and `lib.rs` that installing a
  `MemoryLayerManager` opts the react engine into automatic warm-layer
  salvage writes (triggers by default, pre-compaction always), and that
  `Captured` from a `MemoryTriggerSink` is the opt-out for the trigger
  path only. (b) Route `pre_compaction_flush` through the
  `memory_trigger_sink` as well (or a sibling sink) so a consumer can
  gate both paths uniformly. (c) Add an explicit
  `auto_persist_triggers: bool` / `auto_persist_compaction_facts: bool`
  config on the react agent (default false) so auto-write is opt-in,
  aligning the framework default with the "diagnostics-only" statement.
- Regression validation: a test that constructs a `ReactAgent` with a
  layer manager and a recording sink, runs one turn that produces a
  trigger, and asserts the sink disposition is honored; plus a test
  that pre-compaction facts are review-gated (or auto-written) per the
  chosen direction.
- Validation reports: [V03-01](../validations/F-EVO-01/V03-01.md).

### F-EVO-01-P3-01: `evolution` is unconditionally compiled while `eval`/`improve` are feature-gated

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/lib.rs:40` (`pub mod evolution;` with no
  `#[cfg(...)`); 0 `cfg(feature = "evolution")` matches across the
  workspace (V01). Contrast `lib.rs:36-38` (`eval`) and `:43-45`
  (`improve`). `Cargo.toml` defines `eval = []` and `improve = []` but
  no `evolution` feature.
- Reachability: definition → unconditional compilation for every
  `echo_agent` consumer → live caller (the react engine and the CLI both
  use it).
- Expected invariant: optional capabilities that are explicitly listed
  alongside `eval`/`improve` in the framework's own module-level doc
  (`lib.rs:8-20`) should be isolatable behind a feature for consumers
  that do not want them, consistent with the framework's feature-menu
  design (F-FEAT-01).
- Observed behavior: `evolution` (17 files, 11096 lines) is compiled
  into every consumer including minimal `--no-default-features` builds.
  It pulls only always-on core crates (`error`, `memory`, `trace`,
  `skills`, `llm` — V01), so it does not drag optional dependencies,
  but it cannot be compiled out.
- Impact: low. No optional-dep weight, no correctness effect. The cost
  is compile time for consumers that do not use evolution, and an
  inconsistency with `eval`/`improve` gating that makes the
  "capabilities menu" story (F-FEAT-01) less uniform. Runtime
  mitigation exists: a consumer that never calls
  `install_memory_layer_manager` gets no evolution behavior, so the
  module is runtime-opt-in even though it is compile-time-mandatory.
- Root cause: evolution predates the feature-menu discipline applied to
  eval/improve, or was judged "always wanted". No `Cargo.toml` feature
  was ever added.
- Direction: if isolation consistency is desired, add `evolution = []`
  to `Cargo.toml`, gate `pub mod evolution` behind
  `#[cfg(feature = "evolution")]` (with a `doc(cfg)` attr like the
  others), and add it to the `full` aggregator and to the CLI's feature
  list (the CLI already depends on it via `MemoryLayerManager`). Per
  AGENTS.md "删除框架代码的判定" this is a cleanup, not a deletion
  question — evolution is live and used.
- Regression validation: `cargo check -p echo_agent
  --no-default-features --locked` (must still compile without
  evolution) and `cargo check -p echo_agent --features full --locked`;
  add `evolution` to the AGENTS.md conditional matrix list.
- Validation reports: [V01-01](../validations/F-EVO-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Feature/reachability inventory + standalone compile | yes | passed | [V01-01](../validations/F-EVO-01/V01-01.md) |
| V02 | API genericity / no EKO coupling | yes | passed | [V02-01](../validations/F-EVO-01/V02-01.md) |
| V03 | Mutation/review boundaries (propose vs apply; auto-write paths) | yes | passed_with_findings | [V03-01](../validations/F-EVO-01/V03-01.md) |
| V04 | Deterministic fixture/test reproducibility | yes | passed | [V04-01](../validations/F-EVO-01/V04-01.md) |
| V05 | Historical-document drift | no | not_run | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md` "No self-evolution metric platform. Evolution = explicit diagnostics and user-triggered review only." | partially_current | The `evolution/` module's propose/apply split and the app's Review Inbox are diagnostics+review. BUT the framework react engine auto-writes memory during runs (F-EVO-01-P2-01), so the statement is accurate for the `evolution/` API surface and inaccurate for the full system once a layer manager is installed. |
| `AGENTS.md` "no automatic memory/rule/skill mutation" (task V03 phrasing) | partially_current | Rules and skills: confirmed not auto-mutated by the framework (RulePromoter and skill promotion are app-layer/proposal-based). Memory: framework react engine auto-writes (F-EVO-01-P2-01). |
| `F-FEAT-01` "eval and improve are live marker features; CLI excludes both via default-features = false" | current | Re-verified: app-core `Cargo.toml:10-15` and root CLI `Cargo.toml:50` enable neither; standalone `--features eval`, `--features improve`, `--features eval,improve` all compile (V01). |
| `evolution/mod.rs` "All mutations ... are recorded in the audit log. High-risk changes ... require human review." | current | Verified: every `write_memory`/promote/demote/delete calls `record_change`; merge/patch produce proposals consumed by explicit apply fns; `BackgroundReviewConfig::auto_persist_user_preferences` defaults false. |
| `improve/mod.rs` "This module does NOT automatically ... Modify core runtime code / Relax security policies / Change permission rules / Publish or deploy" | current | Verified: `ImprovementLoop` produces `LoopResult { suggestions }`; loop.rs:36-39 doc explicitly "does NOT automatically apply suggestions". |

## Coverage And Uncertainty

- **Covered**: full file inventory of all three modules; feature wiring
  for eval/improve/evolution; standalone compile of all three feature
  combinations; genericity grep (zero EKO coupling); propose/apply split
  for skills+semantic memory; the two react-engine auto-write paths and
  EKO's sink override; eval test determinism (14/14 pass); CLI feature
  selection.
- **Not executed**: a whole-repo `--features full` test run (disk at
  ~46 GiB; only targeted eval tests + three `cargo check`s were run to
  stay within the AGENTS.md disk guidance). The `Q-FW-02` matrix and the
  AGENTS.md conditional matrix own the full sweep. The conditional
  matrix list in AGENTS.md does not currently include `eval`/`improve`/
  `evolution`, so they are not mandated there either.
- **Memory-authority overlap (handoff, not a finding here)**:
  `echo-agent/src/memory_promoter.rs` (always-compiled) salvages facts
  from compression-evicted messages via `StoreMemoryPromoter`, while
  `evolution::MemoryLayerManager::write_memory` is the typed/layered
  write path with security+audit. The react engine wires
  `StoreMemoryPromoter` as the `MemoryPromoter`
  (`react/mod.rs:1033,1047,1123,1177`) AND runs `pre_compaction_flush`
  through the layer manager (`context.rs:676-798`). These are two
  memory-write mechanisms with different security/audit characteristics
  coexisting in the framework. Whether they are duplicate authority or
  complementary (L3 salvage vs typed lifecycle) is `F-MEM-01`'s call;
  F-EVO-01 flags it and stops.
- **Uncertain claims**: the severity of F-EVO-01-P2-01 hinges on whether
  "no automatic memory mutation" is read strictly (any auto-write during
  a run is a violation) or pragmatically (opt-in, security-scanned,
  audit-logged, warm-layer-only, sink-interceptable trigger writes are
  acceptable; only the non-interceptable pre-compaction path is a real
  gap). Both readings are documented in the finding for the user to
  adjudicate.

## Handoff

- **Conclusions downstream tasks may rely on**:
  - `eval` and `improve` are valid, generic, properly feature-gated
    framework options. They are NOT compiled into EKO (CLI excludes
    both) and are consumed only by the two demo examples. They are
    legitimate framework options per AGENTS.md "删除框架代码的判定"
    (not deletion candidates just because the CLI doesn't use them).
  - `improve`'s base trajectory export works with `improve` alone; its
    eval-driven analyzer/loop/generator activate only when `eval` is
    also on (verified by standalone compile). The cross-feature coupling
    is correct and compiles.
  - `evolution` is generic (zero EKO coupling) and its propose/apply
    split for skills + semantic memory is clean: proposals are
    analysis-only; apply primitives are pub, consumer-invoked,
    audit-logged, security-scanned.
  - No framework path auto-mutates **rules** or **skills**. Auto-mutation
    is confined to **memory** (warm layer), via two react-engine paths
    documented in F-EVO-01-P2-01.
  - EKO's `ReviewIntegration` sink review-gates trigger-detected
    memories (`Captured`); it is a thin, correct adapter.

- **Reports downstream tasks must read**:
  - This report for the eval/improve/evolution boundary and the
    react-engine auto-write finding.
  - `F-FEAT-01` for why `eval`/`improve` are kept despite no CLI use.
  - `F-MEM-01` (when complete) for the memory-authority question,
    including the `memory_promoter` vs `MemoryLayerManager` overlap
    flagged above.

- **Conditions that make this report stale**:
  - Any change to `echo-agent/src/agent/react/run/context.rs`
    (`detect_and_write_memory_triggers`, `pre_compaction_flush`).
  - Adding/removing a `#[cfg]` on `pub mod eval`/`improve`/`evolution`
    in `lib.rs`, or adding an `evolution` feature to `Cargo.toml`.
  - Changes to `MemoryTriggerSink` disposition semantics or EKO's
    `ReviewIntegration::on_trigger` return value.
  - Any new auto-write path added to the react engine.

- **Follow-up task IDs** (fixes not implemented in this review task):
  - F-EVO-01-P2-01 resolution (document / sink-route / opt-in config
    for the react-engine auto-writes).
  - F-EVO-01-P3-01 (optional `evolution` feature gate for isolation
    consistency).
  - `F-MEM-01` to adjudicate the `memory_promoter` vs
    `MemoryLayerManager` memory-write authority overlap.
