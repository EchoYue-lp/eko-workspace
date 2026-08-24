# A-EVO-01: EKO evolution product scope

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only inspection + targeted `cargo test`)

## Question

Has EKO kept evolution as explicit diagnostics/review without hidden
metric loops, automatic semantic mutation, or framework option deletion?

## Scope

Primary source paths and behaviors inspected (read-only unless noted):

- `echo-agent-cli/echo-agent-app-core/src/evolution/mod.rs` (full,
  24 lines) — module surface and `pub use` re-exports.
- `echo-agent-cli/echo-agent-app-core/src/evolution/review_integration.rs`
  (full, 673 lines) — `ReviewIntegration`, `MemoryTriggerSink` /
  `SkillLoadPolicy` impls, `run_review`, session-end and capture paths.
- `echo-agent-cli/echo-agent-app-core/src/evolution/evidence.rs`
  (full, 1356 lines) — `EvidenceStore` JSONL protocol, `accept`/`undo`
  with rollback, stale-failure-derived `expired` projection.
- `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs`
  (full, 378 lines) — `RulePromoter::scan_for_proposals` and
  `promote_rule` (writes `learned-rules.md`).
- `echo-agent-cli/echo-agent-app-core/src/evolution/dashboard.rs`
  (full, 319 lines) — on-demand metrics, cross-run tool diagnostics.
- `echo-agent-cli/echo-agent-app-core/src/evolution/hook_fire.rs`
  (full, 54 lines) — best-effort `fire_evolution_hook`.
- `echo-agent-cli/echo-agent-app-core/src/auto_memory/mod.rs` (full,
  72 lines) — observation extraction → `EvidenceStore`.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:222-268` — bootstrap
  `ReviewIntegration` + `MemoryLayerManager` + sink + skill policy.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:1132-1212` — daily
  `spawn_dreaming_task`.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:940-1130` — workspace
  switch / exit memory-store and review rebind.
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:591-710, 880-930`
  — pool `apply_memory_store`, `refresh_instruction_context`, and
  pool-agent creation with layer-manager install.
- `echo-agent-cli/src/cli/cmd_impls/evolution.rs` (full, 1850 lines) —
  every evolution CLI command (`/review`, `/memory-review`,
  `/curator`, `/skill-{candidates,promote,create,merge,patch,health,
  register,pin,unpin}`, `/rule-promote`, `/evolution-dashboard`,
  `/evidence-inbox`).
- `echo-agent-cli/src/cli/cmd_impls/all.rs:460-600` — `/auto-memory`
  command and the `AUTO_MEMORY_ENABLED` global.
- `echo-agent-cli/src/cli/repl.rs:90-110, 265-330` — CLI REPL
  `spawn_dreaming_task` and `run_auto_memory_on_exit`.
- `echo-agent-cli/src/tui/events.rs:3610-3700`, `tui/mod.rs:1985-2020` —
  TUI `/auto-memory` handling and TUI Dreaming spawn.
- `echo-agent-cli/src/tauri/desktop.rs:230-280` — GUI Dreaming spawn.
- `echo-agent-cli/src/tauri/commands/panels.rs:1180-1500, 1298-1450`
  — GUI `curator_action`, `evidence_candidate_action`,
  `get_evolution_dashboard`, `promote_rule`, auto-memory commands.
- `echo-agent-cli/Cargo.toml` and `echo-agent-app-core/Cargo.toml`
  (feature selection; CLI excludes `eval`/`improve`).
- Executable check: `cargo test --lib --locked -p echo-agent-app-core
  evolution::` (19 tests pass; see V04-01).

## Out Of Scope

Deferred to named task IDs:

- Framework `eval`/`improve`/`evolution` API genericity and feature
  gating → `F-EVO-01` (complete, read). This task consumes
  F-EVO-01's conclusions and only re-verifies the EKO touchpoints.
- Framework react-engine auto-write paths (trigger detection and
  pre-compaction flush) → `F-EVO-01-P2-01`. A-EVO-01 inherits the
  finding and records the EKO-specific posture (sink = `Captured` for
  triggers; pre-compaction still auto-writes) but does not re-audit the
  framework code paths.
- Memory-authority unification (`memory_promoter` vs
  `MemoryLayerManager`) → `F-MEM-01`.
- Application-layer instruction/hot-memory projection refresh wiring
  (Dreaming refreshes the wrong projection, etc.) → `A-MEM-01` (complete,
  read). A-EVO-01 cross-references P1-01 only for the Dreaming cadence
  consequence.
- Skill lifecycle promotion thresholds and SKILL.md draft quality →
  product tuning, not in scope for this boundary task.

## Inputs

- Required repository documents read: root `AGENTS.md` (sections
  "产品定位与安全边界" and "统一术语"; the task card's anchor
  sentence "No self-evolution metric platform. Evolution = explicit
  diagnostics and user-triggered review only."),
  `docs/comprehensive-review/REPORTING.md`, both report templates, the
  `A-EVO-01` card in `TASKS.md`.
- Dependency task reports read: `zcode-glm/tasks/F-EVO-01.md` (complete)
  — establishes framework propose/apply split, the react-engine
  auto-write finding (F-EVO-01-P2-01), and the framework feature
  topology; `zcode-glm/tasks/A-MEM-01.md` (complete) — establishes the
  application-layer projection-refresh defects.
- Product design documents read as evidence of intent (not as code
  truth): `echo-agent-cli/docs/2026-07-15-self-evolution-review-and-roadmap.md`
  (the Phase A-F roadmap that defined the current boundary),
  `echo-agent-cli/docs/2026-07-23-memory-self-evolution-closure.md`
  (the closure note that consolidated the workspace/dreaming refresh
  wiring).
- Historical documents treated as hypotheses:
  `echo-agent-cli/docs/memory-evolution-full-audit.md` — dated
  audit; flagged the dead `RulePromoter` namespace since fixed. Used
  only to confirm the fix is in place.

## Layering Decision

This is an **application-layer** task. The layering conclusions:

- **Generic mechanism** (framework, owned by `echo_agent::evolution`):
  `MemoryLayerManager::write_memory`/`delete_memory`/
  `apply_merge_proposal`/`restore_merge_snapshots`,
  `MemoryReviewer::review`, `BackgroundReviewer`, `Dreaming::run`,
  `Curator`, `SkillMerger`/`SkillPatcher`/`SkillCandidateDetector`/
  `SkillDraftGenerator`, `JsonlChangeLog`, `EvolutionSecurityGuard`,
  the `MemoryTriggerSink` trait, the `SkillLoadPolicy` trait. None
  reference EKO; consumed via thin adapters.
- **EKO product policy** (application, owned by `echo-agent-app-core`):
  the Review Inbox JSONL protocol and `EvidenceStore` semantics
  (accept/undo with rollback, stale-derived `expired`), the
  `MemoryTriggerSink` impl that returns `Captured` to force
  review-only triggers, the `SkillLoadPolicy` impl that blocks
  `_drafts/` and foreign-workspace skills, the daily `spawn_dreaming_task`
  schedule (60s initial, 86400s interval), the on-demand `Dashboard`
  filter rules (`occurrence_count >= 3 && distinct_run_count >= 2`,
  max 3 reminders), and the `RulePromoter` threshold (`min_confidence:
  0.95`, `min_age_days: 7`).
- **Adapter boundary**: `ReviewIntegration` implements two framework
  traits (`MemoryTriggerSink`, `SkillLoadPolicy`) plus holds an
  `EvolutionObserver` it threads into `create_layer_manager`. It performs
  no second-layer scheduling, owns no semantic mutation, and routes every
  framework proposal through the JSONL inbox. `hook_fire::fire_evolution_hook`
  is a best-effort lifecycle notifier; it does not mutate.

Repository-wide duplicate-search terms used: `EvidenceStore`,
`RulePromoter`, `Dashboard`, `fire_evolution_hook`, `MemoryTriggerSink`,
`SkillLoadPolicy`, `spawn_dreaming_task`, `run_auto_memory_extraction`,
`AUTO_MEMORY_ENABLED`, `curator_state.json`. Result: one canonical
definition per concept; the only overlap with framework machinery is
intentional (the trait impls above). The single dead public function
`run_auto_memory_extraction` (zero callers workspace-wide) is the V03
finding.

## Current Path

### Mutation triggers (verified — see V01)

Every memory/rule/skill mutation reachable from EKO, classified by
who initiates it:

| Surface | Mutates | User-initiated? | Path |
|---|---|---|---|
| CLI `/remember <text>` | typed memory (warm) | yes | `all.rs:106` → `MemoryLayerManager::write_memory` (direct) |
| TUI `/remember` | typed memory (warm) | yes | `events.rs:2839-2857` → `write_memory` |
| GUI `add_memory` | typed memory (warm) | yes | `panels.rs` → `write_memory` |
| CLI/TUI/GUI `/forget`/`delete_memory` Hot | MEMORY.md entry | yes | `delete_memory` + `refresh_instruction_projection` |
| `/evidence-inbox accept <id>` | typed memory or merge | yes | `evidence.rs:478-636` → `write_memory` / `apply_merge_proposal` |
| `/evidence-inbox undo <id>` | revert applied mutation | yes | `evidence.rs:638-770` → `delete_memory` / `restore_merge_snapshots` |
| `/rule-promote <key>` | learned-rules.md | yes | `rule_promoter.rs:178-268` (writes file + marks memory) |
| `/skill-promote <name>` | curator state + SKILL.md move | yes | `evolution.rs:715-802` |
| `/skill-merge <a> <b>` | merge_proposals store + (on execute) SKILL.md | yes | `evolution.rs:904-1101` |
| `/skill-patch <name> apply <i>` | SKILL.md | yes | `evolution.rs:1249-1378` |
| `/curator run` | curator lifecycle transitions | yes | `evolution.rs:185` / `panels.rs:1393` |
| `/curator pin|unpin` | curator state | yes | `curator.pin_skill`/`unpin_skill` |
| `/review`, `/memory-review` | inbox candidates only | yes | no durable mutation; proposals only |
| `/evolution-dashboard`, GUI `get_evolution_dashboard` | none | yes (on open) | read-only metrics |
| `/auto-memory extract`, GUI `extract_auto_memory` | inbox candidates only | yes | `queue_observations` → `EvidenceStore.upsert` |
| `/auto-memory on\|off` | global flag | yes | `AUTO_MEMORY_ENABLED.store` |
| **Background: Dreaming pass** | MEMORY.md promote/demote; warm revive/archive | **automatic (cron)** | `infra.rs:1143-1200` → `Dreaming::run` |
| **Background: react-engine trigger detect** | typed memory (warm) by way of sink | automatic (per-turn) | sink returns `Captured` in EKO → routes to inbox; no durable write |
| **Background: react-engine pre-compaction flush** | typed memory (warm) | **automatic (per-compression)** | `context.rs:676-798` (framework) → `write_memory`; **NOT sink-interceptable**; runs in EKO |
| **Background: session-exit auto-memory** | inbox candidates only | automatic (if enabled) | `repl.rs:265-330` → `queue_observations` |

The bottom three rows are the only paths that mutate state without a
per-action user click. The first two (Dreaming, trigger detect) and the
last (session-exit) are bounded: Dreaming is recall-frequency-driven
maintenance that does not generate new facts; triggers in EKO are
review-gated; session-exit only queues candidates. The pre-compaction
flush is the only path that bypasses the Review Inbox, and is the
framework-side finding F-EVO-01-P2-01 inherited here.

### Authorization defaults (verified — see V02)

- `ReviewConfig::default()` is `review_on_session_end: false`,
  `auto_generate_drafts: false`, `detect_skill_candidates: true`
  (`review.rs:527-537`). EKO instantiates it verbatim
  (`runtime.rs:236`).
- `BackgroundReviewConfig::default()` has `auto_persist_user_preferences:
  false` and `proposal_only: true` (per F-EVO-01 V03).
- `AUTO_MEMORY_ENABLED` defaults to `true` (`all.rs:462-463`), but its
  only effect is to queue heuristic observations into the Review Inbox
  at session exit — never to write durable memory.
- `MemoryTriggerSink::on_trigger` returns `Captured` unconditionally
  (`review_integration.rs:319`), with the comment at `:314-317`:
  "EKO treats inferred memory as review-only. Do not let an inbox
  storage failure fall through to the framework's direct durable write
  path and silently bypass that review gate."
- `SkillLoadPolicy::allows` blocks `_drafts/`, foreign-workspace
  skills, and any skill whose lifecycle is not `Active` or `Stale`
  (`review_integration.rs:325-347`). It does not auto-promote.
- `EvidenceStore::accept` is the only path from inbox to durable
  mutation; it requires an explicit `candidate_id` argument and runs
  through `layer_manager.write_memory` / `apply_merge_proposal`
  (`evidence.rs:478-636`).

### Dreaming cadence (verified — see V03 + V04)

`spawn_dreaming_task` (`infra.rs:1143-1200`) is invoked from three
sites (`repl.rs:106`, `tui/mod.rs:1999`, `tauri/desktop.rs:247`) —
**always** at session/window start, gated only on
`review_integration.is_some()`. The first pass fires after a 60s
initial delay, then every 86400s (24h) until the cancellation token
fires on session exit. It is the **only** automatic semantic-state
mutator that EKO wires up.

The Dreaming pass calls `MemoryLayerManager::consider_promotion` →
`promote_warm_to_hot` (writes MEMORY.md), the deterministic demote/revive
paths, and the warm-layer writes inside the framework's pre-compaction
flush. None of these add new facts; they re-classify existing ones.
Critically, A-MEM-01-P1-01 already established that the post-Dreaming
refresh targets the wrong projection (`eko:instruction-context` instead
of `eko:hot-memory-context`), so the hot-layer promote is not visible
in the agent's stable prefix until the next workspace switch — but the
mutation itself does land in MEMORY.md on disk.

## Findings

### A-EVO-01-P2-01: `auto_memory::run_auto_memory_extraction` is a dead public function (aspirational API surface)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - Definition: `echo-agent-cli/echo-agent-app-core/src/auto_memory/mod.rs:17-24`
    (`pub fn run_auto_memory_extraction(messages, config) ->
    Result<Vec<EvidenceCandidate>, String>`).
  - Repository-wide caller search:
    `grep -rn "run_auto_memory_extraction" echo-agent-cli/ --include="*.rs"`
    returns exactly one line — the definition itself. No production
    caller, no test caller, no re-export.
  - All actual auto-memory callers (CLI `/auto-memory extract`, TUI
    `/auto-memory extract`, GUI `extract_auto_memory`, REPL session-exit)
    call the underlying `extract_observations` + `queue_observations`
    directly, not this helper.
- Reachability: definition → `pub use echo_agent::evolution::auto_memory::
  {extract_observations, ...}` re-export → no live caller.
- Expected invariant: every `pub fn` in the application crate should
  either have at least one live caller, be a documented public API for
  downstream consumers, or be explicitly marked as such. Per AGENTS.md
  "代码清理: 无需兼容, 过时代码可直接删" — a helper that nothing calls
  is dead code.
- Observed behavior: the function compiles into the crate but is never
  invoked. It differs from the actual call sites in that it constructs
  its own `EvidenceStore` from `discover_echo_agent_dir()` rather than
  using the workspace-bound `ReviewIntegration::evidence_store()` — so
  if anyone *did* call it from a workspace-switched session, it would
  write to the wrong workspace's inbox.
- Impact: low. No correctness effect today (zero callers). The risk is
  drift: someone discovering this helper later could call it and get the
  wrong workspace scope. It is also misleading evidence when reasoning
  about the auto-memory surface ("there is a single entry point" —
  false, there are five and this isn't one of them).
- Root cause: the helper predates the consolidation onto
  `ReviewIntegration.evidence_store()`. When the call sites were
  updated to use the workspace-bound store, this entry point was left
  behind.
- Direction: delete `run_auto_memory_extraction`. Per AGENTS.md "代码
  清理", no backward compatibility is required (application-layer fn,
  not framework API). After deletion, re-run the V04-01 test set to
  confirm the auto-memory surface still compiles and the queue tests
  still pass.
- Regression validation: `cargo check -p echo-agent-app-core --locked`
  and `cargo test -p echo-agent-app-core --lib --locked auto_memory::`
  (no direct tests today, but the `evolution::evidence::tests::*` set
  covers `queue_observations` transitively).
- Validation reports: [V03-01](../validations/A-EVO-01/V03-01.md).

### A-EVO-01-P2-02: Pre-compaction flush is the only Review-Inbox-bypassing auto-write that runs in EKO

- Priority: P2
- Confidence: high
- Layer: application (impact), framework (root cause)
- Evidence:
  - Framework path (inherited): `echo-agent/src/agent/react/run/context.rs:676-798`
    — `pre_compaction_flush` calls `MemoryLayerManager::write_memory`
    directly for each LLM-extracted fact, gated only on
    `self.llm_client` and `self.memory_layer_manager` being `Some`
    (`context.rs:681-683`). NOT intercepted by `memory_trigger_sink`
    (the sink covers only the trigger path).
  - EKO installs a `MemoryLayerManager` on every primary and pool agent
    (`runtime.rs:255`, `agent_pool.rs:672,923`), so the path is live
    in every EKO session that has an LLM client.
  - Contrast with the trigger path: `ReviewIntegration::on_trigger`
    (`review_integration.rs:277-321`) returns `Captured` and routes
    triggers to the inbox, so trigger-detected memories are review-gated.
- Reachability: every react-loop turn whose context triggers
  `ContextManager::should_compress()` and an LLM extraction succeeds.
- Expected invariant: AGENTS.md task-card anchor "Evolution = explicit
  diagnostics and user-triggered review only" — read strictly, every
  memory write during a run should be review-gated.
- Observed behavior: pre-compaction facts are auto-written to the warm
  typed-namespace, with security scanning and audit log but without
  Review Inbox mediation. The doc
  `2026-07-15-self-evolution-review-and-roadmap.md` Phase B/E states
  "TriggerDetector, AutoMemory, Reflection, BackgroundReviewer, 压缩
  promotion 都能生成长期信息" and "压缩两条事实抽取路径共用 content key"
  — i.e. the closure accepted that compression-salvage is a write path
  but bound it by content dedup, not by review gating.
- Impact: medium-strict, low-pragmatic. The write is bounded (warm
  layer only, no auto-promotion to the hot prefix, security-scanned,
  audit-logged, deduplicated against existing entries). It is the only
  EKO reachable path that writes new semantic content into the typed
  store without per-write user action. Inherited from F-EVO-01-P2-01.
- Root cause: the framework `MemoryTriggerSink` trait was designed to
  gate trigger-detected memories only; the older pre-compaction salvage
  path predates the trait and was not retrofitted.
- Direction: do NOT implement in this review. Pick one of the
  F-EVO-01-P2-01 directions: (a) document the boundary precisely in
  `evolution/mod.rs` and the EKO adapter; (b) route the pre-compaction
  path through a sibling sink so EKO can review-gate it uniformly; or
  (c) add an opt-in/opt-out config on the react agent.
- Regression validation: a test that runs a react turn triggering
  compression with a recording layer manager and asserts whether
  extracted facts are written (per the chosen direction).
- Validation reports: [V01-01](../validations/A-EVO-01/V01-01.md),
  [V02-01](../validations/A-EVO-01/V02-01.md).

### A-EVO-01-P3-01: Documentation still calls the rule file `AGENTS.md`, but `InstructionProvider` writes `learned-rules.md`

- Priority: P3
- Confidence: high
- Layer: application (doc drift)
- Evidence:
  - CLI `/rule-promote` user-facing success message:
    `evolution.rs:1473-1476` — "Successfully promoted memory '{}' to
    AGENTS.md".
  - CLI command help string: `evolution.rs:1520` — "Promote high-
    confidence memories to agent rules in AGENTS.md".
  - GUI `promote_rule` source comment: `panels.rs:1513` — "不会改
    AGENTS.md. 复刻 CLI".
  - Reality: `rule_promoter.rs:221` calls
    `InstructionProvider::save_agents_instructions(&new_content)`,
    which (per A-MEM-01) writes `<root>/.eko/learned-rules.md`
    (the file is read by `InstructionProvider::get_instruction_suffix`
    under the "Auto-promoted rules" tier, and the legacy AGENTS.md is
    only a one-time migration source).
- Reachability: any user who runs `/rule-promote scan` or `/rule-promote
  <key>` and reads the output.
- Expected invariant: user-facing strings should name the file the
  user will see modified.
- Observed behavior: the success message and command help say
  "AGENTS.md"; the actual written file is `learned-rules.md`.
- Impact: low. The user can find the rule via `InstructionProvider`'s
  "Auto-promoted rules" tier label, but anyone grepping their `.eko/`
  directory for `AGENTS.md` will be confused (the only AGENTS.md in
  `.eko/` is a one-time migration source that may not exist).
- Root cause: the strings predate the AGENTS.md → learned-rules.md
  rename and were never updated.
- Direction: replace the three "AGENTS.md" references with
  "learned-rules.md" (or with "the auto-promoted rules file"). Per
  AGENTS.md "代码清理", no compatibility wrapper is needed.
- Regression validation: `cargo check -p echo-agent-cli --bin
  echo-agent --locked`; no functional test exists for the string.
- Validation reports: [V04-01](../validations/A-EVO-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Reachable mutation trigger inventory | yes | passed_with_findings | [V01-01](../validations/A-EVO-01/V01-01.md) |
| V02 | User-authorization boundaries (defaults + sink disposition) | yes | passed | [V02-01](../validations/A-EVO-01/V02-01.md) |
| V03 | Dead/aspirational path classification | yes | passed_with_findings | [V03-01](../validations/A-EVO-01/V03-01.md) |
| V04 | Product docs vs code + targeted test run | yes | passed_with_findings | [V04-01](../validations/A-EVO-01/V04-01.md) |
| V05 | Historical-document drift | conditional | see Historical Claim Status | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md` task-card anchor "No self-evolution metric platform. Evolution = explicit diagnostics and user-triggered review only." | partially_current | Rules, skills, semantic memory conflict/merge: confirmed user-gated (V01, V02). Memory: the framework react engine still auto-writes via the pre-compaction flush (F-EVO-01-P2-01 inherited as A-EVO-01-P2-02); Dreaming auto-promotes/demotes within the warm/hot layer (deterministic, no new facts). No metric platform, no benchmark loop, no auto prompt rewrite, no auto skill/rule mutation. |
| `2026-07-15-self-evolution-review-and-roadmap.md` Phase F: "不引入 EvalRunner、SQLite、后台 LLM reviewer 或自动 prompt rewrite; 任何指标都不能自动修改 prompt、skill、rule 或 memory。" | current | Verified by V01/V02: no EvalRunner in CLI (`eval`/`improve` excluded per app-core `Cargo.toml:10-15`), no background LLM reviewer loop, no auto prompt rewrite. Dashboard does not collect accept/reject rates (read-only tool-failure diagnostics). |
| Same doc Phase B/E: "压缩两条事实抽取路径共用 content key" + "压缩 promotion 都能生成长期信息" | current | The compression salvage (framework `pre_compaction_flush` + `StoreMemoryPromoter`) is the inherited F-EVO-01-P2-01 surface; A-EVO-01-P2-02 records it as the only non-review-gated write. Doc claims dedup-by-content-key, not review gating — consistent with code. |
| `2026-07-23-memory-self-evolution-closure.md`: "Dreaming 在启动稳定 60 秒后先执行一次, 再按日执行; GUI、TUI、CLI 使用同一接线" | current | All three spawn sites verified (`repl.rs:106`, `tui/mod.rs:1999`, `tauri/desktop.rs:247`); 60s + 86400s interval (`infra.rs:1156-1158`). |
| `memory-evolution-full-audit.md` v1.1: "RulePromoter namespace 死链已修" | current | Verified by `scan_hits_memories_written_to_warm_namespace` test (`rule_promoter.rs:290-323`) — scans `WARM_NAMESPACE = ["agent","memories"]`. |
| CLI `/rule-promote` help string "Promote high-confidence memories to agent rules in AGENTS.md" | stale | The written file is `learned-rules.md` (A-EVO-01-P3-01). |
| F-EVO-01 handoff: "EKO's ReviewIntegration sink review-gates trigger-detected memories" | current | V02 re-verifies `Captured` disposition (`review_integration.rs:319`). |
| A-MEM-01 handoff: "Hot-layer refresh is broken on every memory-edit surface except workspace switch" | current (cross-reference) | Not re-audited; A-EVO-01 notes only that the Dreaming pass inherits this gap (the post-Dreaming refresh goes to the instruction projection, not the hot-memory projection). |

## Coverage And Uncertainty

- **Covered**: every evolution-related `pub` symbol in `echo-agent-app-core/
  src/evolution/` and `src/auto_memory/`; every CLI evolution command in
  `cli/cmd_impls/evolution.rs`; every GUI Tauri command in `panels.rs`
  related to evidence, curator, skills, rules, auto-memory, and the
  dashboard; the runtime bootstrap (`runtime.rs`), the daily Dreaming
  schedule (`infra.rs`), the workspace-switch rebind (`state.rs`), and
  the pool-agent install path (`agent_pool.rs`); the `Cargo.toml`
  feature selection (CLI excludes `eval`/`improve`); the 19 in-tree
  evolution tests (V04-01, all pass).
- **Not executed**: a workspace-wide `cargo test --workspace --all-features`
  (disk at ~62 GiB; per AGENTS.md disk guidance, only the targeted
  evolution test subset was run, total 10m39s). The conditional feature
  matrix is owned by `Q-FW-02` / F-EVO-01.
- **Framework code paths inherited, not re-audited**: the
  `MemoryLayerManager` write/promote/demote semantics, the
  `MemoryReviewer` staleness/conflict scoring, the `Dreaming::run`
  recall-frequency logic, and the react-engine `detect_and_write_memory_
  triggers` / `pre_compaction_flush` plumbing. These are taken from
  F-EVO-01 (complete) as stable inputs.
- **Uncertain claims**:
  - The severity of A-EVO-01-P2-02 (pre-compaction flush) hinges on
    whether the AGENTS.md anchor is read strictly (any auto-write is a
    violation) or pragmatically (warm-layer, security-scanned, dedup'd,
    no auto-promotion = acceptable salvage). The product design doc
    (`2026-07-15`) explicitly accepts compression salvage as a write
    path, which leans toward the pragmatic reading; but the task-card
    anchor phrasing leans strict. Both readings are documented.
  - Whether the daily Dreaming pass counts as "user-triggered" is a
    labeling question. The user starts the session (which spawns
    Dreaming); the pass itself runs without further user action.
    Classified here as "automatic (cron)" because the user does not
    trigger each pass.
- **Scope excluded**: the `EvidenceStore` JSONL protocol is
  schema-versioned (currently v3); schema evolution and migration are
  not in scope. `EvidenceInteractionEvent` audit fields are read-only
  diagnostics (accept/reject/undo/stale), not metrics that feed back
  into mutation logic — verified by V04 grep for `accept_rate`/
  `rejection_rate`/`reward`/`score_loop` (zero matches in evolution/
  auto_memory).

## Handoff

- **Conclusions downstream tasks may rely on**:
  1. EKO's application-layer evolution surface is **review-gated for
     every semantic mutation**: rules, skills, and memory merges all
     require an explicit user action (`/rule-promote <key>`,
     `/skill-promote <name>`, `/evidence-inbox accept <id>`,
     `/skill-merge <a> <b>`, `/skill-patch <name> apply <i>`). No
     background LLM reviewer auto-writes; no metric platform exists.
  2. The only automatic state mutators EKO wires are: (a) Dreaming's
     deterministic promote/demote/revive (recall-driven, no new facts);
     (b) the framework react-engine's pre-compaction flush (warm-layer
     facts, inherited F-EVO-01-P2-01); (c) trigger detection's review-
     gated inbox capture. There is no automatic rule or skill mutation.
  3. `ReviewIntegration` is a thin, correct adapter (two trait impls,
     no rescheduling). The Review Inbox JSONL protocol is the single
     application authority for evidence candidates.
  4. The daily Dreaming cadence is wired identically across CLI, TUI,
     and GUI (`repl.rs:106`, `tui/mod.rs:1999`, `tauri/desktop.rs:247`),
     satisfying mode parity.
  5. CLI excludes `eval`/`improve` framework features
     (`echo-agent-app-core/Cargo.toml:10-15`); the `evolution` module
     is unconditional (per F-EVO-01-P3-01).

- **Reports downstream tasks must read**:
  - This report for the application-side mutation surface and the
    three findings.
  - `zcode-glm/tasks/F-EVO-01.md` for the framework propose/apply
    split and the inherited pre-compaction auto-write finding.
  - `zcode-glm/tasks/A-MEM-01.md` P1-01 for the projection-refresh
    defect that compounds the Dreaming pass's hot-layer write.

- **Conditions that make this report stale**:
  - Adding a new `pub fn` to `evolution/` or `auto_memory/` that
    mutates state (re-run V01/V03).
  - Changing `ReviewConfig::default()` or `BackgroundReviewConfig::
    default()` away from the conservative defaults (re-run V02).
  - Routing the pre-compaction flush through a sink (resolves
    A-EVO-01-P2-02 and F-EVO-01-P2-01; re-run V01).
  - Deleting `run_auto_memory_extraction` (resolves A-EVO-01-P2-01;
    re-run V03).
  - Adding a background LLM reviewer, an EvalRunner, or any metric
    that feeds back into prompt/skill/rule/memory mutation (would
    violate the AGENTS.md anchor; re-run all four validations).
  - Changing `spawn_dreaming_task` cadence or adding a second
    automatic scheduler (re-run V01/V02).

- **Follow-up task IDs** (no fixes implemented in this review):
  - A-EVO-01-P2-01 (delete `run_auto_memory_extraction`).
  - A-EVO-01-P2-02 resolution rides on F-EVO-01-P2-01 (the framework
    sink/config direction).
  - A-EVO-01-P3-01 (rename user-facing "AGENTS.md" strings to
    "learned-rules.md").
