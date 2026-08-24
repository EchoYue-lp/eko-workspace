# A-MEM-01: Instructions, hot memory, and Dreaming

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Does EKO own only its instruction/memory protocol while projecting updates
immediately and consistently to primary and pooled Agents?

## Scope

- `echo-agent-app-core/src/instruction_provider.rs` (full, 438 lines),
  `unified_memory.rs` (full, 268 lines), `utils.rs` (`find_project_root` /
  `strip_yaml_frontmatter`), `infra.rs` (dreaming task :1132-1212, memory
  store resolution :1269-1364, `refresh_dynamic_context` :527-531, agent
  creation :440-444), `state.rs` (`switch_workspace` :844-1032,
  `exit_workspace` :1053-1185), `agent_pool.rs` (`apply_working_dir` :534,
  `apply_memory_store*` :597-684, `refresh_instruction_context` :687-710,
  `create_agent` :824-978), `evolution/{mod,rule_promoter,review_integration,
  hook_fire}.rs`, `workspace/layout.rs` (state dir / memory store paths),
  `tasks/task_runtime/memory_bridge.rs` (memory candidate writes), project
  context projections (`project/prompt.rs`, `workspace_routing.rs`).
- `echo-agent-cli/src`: `tauri/commands/memory.rs` (add/delete/list/search),
  `tauri/commands/panels.rs` (`promote_rule` :1515-1570), `tui/events.rs`
  (/remember /forget :2830-2930, /memory-review :3662-3841), `tui/commands.rs`,
  `cli/cmd_impls/all.rs` (/remember /forget), `cli/cmd_impls/evolution.rs`
  (/rule-promote :1425-1490), `cli/repl.rs` (:100-112 Dreaming spawn),
  `tui/mod.rs` (:1996-2013), `tauri/desktop.rs` (:244-260), `cli/channels.rs`
  (channels mode wiring).
- Framework (cross-referenced for the boundary): `echo-agent/src/evolution/
  dreaming.rs` (full), `layer.rs` (write_memory/consider_promotion/
  promote_warm_to_hot/enforce_hot_budget, hot-file lock),
  `runtime_integration.rs` (HookEvolutionObserver), `recall.rs`,
  `memory_promoter.rs` (L3), `echo-agent/src/tools/builtin/memory.rs`
  (remember tool), `echo-agent/echo-state/src/compression/mod.rs`
  (projection machinery :540-745, split/merge_protected :747-815).

## Out Of Scope

- Store/conversation durability, atomicity, path safety → F-MEM-01 (complete).
- Compression correctness, canonical reinjection, verifier → F-CMP-01 /
  F-CTX-01 (complete); only the EKO projection-survival and cross-reference
  arms are handled here.
- L3 memory promotion bypass of audit/security → F-EVO-01-P2-02 (complete);
  cross-referenced, not duplicated.
- Workspace-switch config/hook/watcher scope and CWD restore → A-CFG-01
  (complete; P1-01/P1-02/P2-02 used as dependencies).
- Task-run memory policy (`MemoryPolicy`, blocking writes) → A-TSK-06.
- Skill/curator wiring on switch → A-SUB-01.
- CLI/TUI/GUI command surface parity in general → A-SRF-01..04, X-SRF-01.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-MEM-01 card), `zcode-ds/README.md`, templates.
- Dependency reports read in full: `A-CFG-01` (workspace switch/exit
  semantics, CWD staleness, pool sync), `F-CMP-01` (compression pipeline,
  P1-02 summary accumulation, canonical chain), `F-MEM-01` (store durability).
- Cross-reference reports read: `F-EVO-01` (P2-02 StoreMemoryPromoter),
  `F-CTX-01` (P2-02 canonical reinjection — referenced via F-CMP-01's
  independent confirmation).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  (:68, :245-261, :466-467), `docs/2026-07-23-memory-self-evolution-closure.md`
  (:10-21, :45-52), `docs/system-deep-dive/04-memory.md`,
  `echo-agent/docs/zh|en/25-self-improvement.md` (:258/:260),
  `echo-agent/docs/zh/03-memory.md` (sampled).

## Layering Decision

- Generic mechanism (framework): `MemoryLayerManager` (hot/warm layers,
  MEMORY.md format, promotion/demotion/budget, change log, security guard),
  `Dreaming` deterministic driver, `MemoryRecaller`, `MemoryReviewer`,
  `StoreMemoryPromoter`, `HookEvolutionObserver`, projection machinery in
  `echo-state::compression` (replace_projection / protected markers /
  split-merge) — correctly placed.
- EKO product policy (application): the instruction file protocol (five tiers
  + hot MEMORY.md read), the two replaceable projections
  (`eko:instruction-context`, `eko:hot-memory-context`), the refresh policy
  and its triggers, `RulePromoter` (learned-rules.md writer + security scan),
  `ReviewIntegration` (review-inbox gate, rebind, curator, trigger sink that
  captures inferred memories as review-only), Dreaming scheduling and the
  per-surface spawn, workspace-scoped store resolution and rebinding,
  `discover_echo_agent_dir` — correctly placed.
- Adapter boundary: `UnifiedMemory` (thin wrap), `refresh_*_projection`
  helpers (thin projections onto the framework envelope), `ReviewIntegration`
  layer-manager factory (`create_layer_manager*`), `apply_memory_store*`
  pool rebinding — thin and lossless; no scheduling/state authority in the
  adapters.
- Duplicate search (terms + results in V01-01): `InstructionProvider`,
  `UnifiedMemory`, `refresh_instruction_projection`,
  `refresh_hot_memory_projection`, `refresh_memory_projections`,
  `refresh_instruction_context`, `MemoryLayerManager`, `MemoryRecaller`,
  `MemoryPromoter`, `Dreaming`, `spawn_dreaming_task`, `RulePromoter`,
  `promote_rule`, `ReviewIntegration`, `discover_echo_agent_dir`,
  `workspace_curator`, `Curator`, `MEMORY.md`, `learned-rules.md`,
  `WARM_NAMESPACE`, `replace_projection`, `has_projection`, `worker`
  (terminology). One authority per semantic; recorded divergences: three
  projection helpers with `refresh_hot_memory_projection` dead in production;
  MEMORY.md read by two independent parsers (framework serde_yaml entries vs
  EKO body strip); three project-root resolvers with different marker sets
  (`utils::find_project_root`, `discover_echo_agent_dir`,
  framework `InstructionResolver`).

## Current Path

Verified data flow (anchors in V02-01):

1. Boot: `infra::create_agent` → `refresh_dynamic_context` (infra.rs:444) →
   `refresh_memory_projections` (unified_memory.rs:170) sets BOTH
   projections from `InstructionProvider::load_for(root)` — five instruction
   tiers (user.md / AGENTS chain / project.md / learned-rules.md / local.md)
   into `eko:instruction-context`, MEMORY.md body into
   `eko:hot-memory-context`. Pooled agents get the same via `create_agent`.
2. Hot layer writes: `MemoryLayerManager::write_memory` (layer.rs:837) →
   security guard → warm put → `consider_promotion` (:685) → when
   `is_hot_eligible` + trust: `promote_warm_to_hot` (:1109) writes MEMORY.md
   (`add_to_hot` :1059), deletes the warm entry, `enforce_hot_budget` (:711)
   may demote other hot entries; then `notify_memory_layer_change` →
   `HookEvolutionObserver` fires user hooks only. Writers: the `remember`
   tool (tools/builtin/memory.rs:222), GUI add_memory, TUI /remember, CLI
   /remember, task memory bridge (memory_bridge.rs:88-149), Dreaming.
3. Dreaming: `spawn_dreaming_task` (infra.rs:1143; spawn sites desktop.rs:247,
   tui/mod.rs:1999, repl.rs:106) → 60 s after boot, then daily →
   `run_dreaming_pass` (:1203) creates a fresh layer manager from
   `ReviewIntegration` → `Dreaming::run` (framework) promotes high-recall
   (incl. revived Archived) memories to hot, demotes stale low-recall to
   Archived → on `report.promoted > 0` the primary agent and the pool refresh
   (infra.rs:1175-1192).
4. Workspace switch: `AppState::switch_workspace` (state.rs:844) → primary
   `set_working_dir` + `refresh_dynamic_context(Some(root))` (:883), pool
   `apply_working_dir` (agent_pool.rs:534 → :553 refreshes both projections),
   memory store swap + `ReviewIntegration::rebind` + primary
   `install_memory_store`/`install_memory_layer_manager` (:946-1013), pool
   `apply_memory_store` (:597). `exit_workspace` (state.rs:1053) mirrors with
   global scope but never restores the process CWD (A-CFG-01-P1-02).
5. Rule promotion: `RulePromoter::scan_for_proposals` (WARM_NAMESPACE,
   confidence/type/age gates, PROMOTED_TO_RULE marker) → `promote_rule`
   appends to learned-rules.md via CWD-resolved `agents_instructions_path`,
   marks the memory, records a change → GUI panels.rs:1561 / CLI
   evolution.rs:1489 refresh the instruction projection (correct target).
6. Compression: EKO projections are wrapped envelope system messages
   (`wrap_projection_message`, compression/mod.rs:725); `is_protected`
   (:679) → `split_protected`/`merge_protected` keep them through every
   `prepare`; EKO projections survive repeated compression (V03-01/V04-01).

## Findings

### A-MEM-01-P1-01: Every in-session hot-layer (MEMORY.md) mutation refreshes the wrong projection — the hot-memory projection stays stale in the live context of the primary and pooled Agents until the next boot, workspace switch, or exit; `refresh_hot_memory_projection` has zero production callers

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - Two distinct projections: `unified_memory.rs:28-29`
    (`eko:instruction-context`, `eko:hot-memory-context`);
    `instruction_provider.rs:138-140` — `get_instruction_suffix` "Excludes
    hot-layer memory"; `:170-174` — `get_memory_suffix` = hot memory only.
  - `refresh_instruction_projection` (unified_memory.rs:138-151) replaces
    only `eko:instruction-context`; `refresh_hot_memory_projection`
    (:154-167) exists for the hot memory and has NO production caller (only
    its own test, :261); `refresh_memory_projections` (:170-186, both) is
    called only from `refresh_dynamic_context` (infra.rs:528-531) — boot
    (:444), switch (state.rs:883), exit (state.rs:1072), pool
    `apply_working_dir` (agent_pool.rs:553).
  - Wrong-projection refresh sites, each fired exactly when MEMORY.md
    changed: Dreaming pass (infra.rs:1175-1192, `report.promoted > 0`);
    GUI `add_memory` on `promotion.is_some()` (memory.rs:126-145); GUI
    `delete_memory` on hot deletion (memory.rs:221-239); TUI `/remember`
    (tui/events.rs:2838-2852); TUI `/forget` hot (tui/events.rs:2913-2927);
    CLI `/remember` (all.rs:120-141); CLI `/forget` hot (all.rs:195-205);
    pool `refresh_instruction_context` (agent_pool.rs:687-710, doc comment
    claims "hot-memory and instruction projections", body refreshes
    instruction only).
  - No refresh at all on: `remember` tool writes that auto-promote
    (framework tools/builtin/memory.rs:222 → write_memory →
    consider_promotion, layer.rs:837-886/:685-707); task memory bridge
    candidate writes (memory_bridge.rs:88-149); hot-budget demotions
    (layer.rs:711).
- Reachability: every live EKO surface — GUI memory panel add/delete,
  TUI/CLI /remember /forget, the daily Dreaming pass, the `remember` tool,
  and completed task runs — mutates MEMORY.md in a running session; the hot
  projection is read by the model every turn from the boot/switch snapshot.
- Expected invariant (documented contract): `docs/2026-07-23-memory-self-evolution-closure.md:11,20`
  and `MASTER-PLAN.md:251-256` — after Dreaming hot promotion or explicit
  hot memory mutation, the primary AND pooled Agents refresh their
  projections immediately.
- Observed behavior: after any in-session hot-layer change, `eko:hot-memory-context`
  keeps the boot/switch content: newly promoted memories are absent from the
  model's "Active Memories", deleted memories remain present (the model can
  still cite them), and budget-demoted entries stay visible. The refresh code
  fires, but on the instruction projection, which does not contain hot memory.
- Impact: the agent reasons with stale active-memory context on every
  surface; Dreaming's core output (promotion) never reaches the live context;
  the task's central invariant ("projecting updates immediately and
  consistently to primary and pooled Agents") fails on the most frequent
  memory paths. No file/store data loss (MEMORY.md and the warm store are
  correct) — hence P1, not P0.
- Root cause: the design doc (2026-07-23 closure, line 19) originally
  planned ONE projection (`eko:instruction-context`) for instructions AND hot
  memory; the implementation deliberately split hot memory into its own
  projection (unified_memory.rs:28-29, instruction_provider.rs:118-120) but
  the refresh call sites were written against the single-projection plan and
  were never updated — the unwired `refresh_hot_memory_projection` helper is
  the residue of the intended mechanism.
- Direction: at the eight hot-change sites, replace
  `refresh_instruction_projection` with `refresh_memory_projections`
  (primary) and make `agent_pool::refresh_instruction_context` refresh both
  projections (or rename/repurpose it); consider wiring the refresh through
  the `EvolutionObserver::on_memory_layer_change` hook so write-time
  auto-promotion and budget demotions also project; delete
  `refresh_hot_memory_projection` or make it live. Update the closure doc's
  single-projection sentence.
- Regression validation: unit fixture — agent with `refresh_memory_projections`
  loaded from dir A, then a hot-eligible `write_memory` on the installed
  layer manager, then the fix path, asserting `has_projection(HOT_MEMORY_CONTEXT_PROJECTION)`
  content contains the new entry; a Dreaming pass with `promoted > 0`
  asserting the hot projection (not just instruction) changed; hot
  `delete_memory` asserting the tombstone/updated projection; a pool variant
  asserting `refresh_instruction_context` updates hot memory too. Add the
  hot-memory twin of `instruction_projection_replaces_previous_workspace`.
- Validation reports: [V01-01](../validations/A-MEM-01/V01-01.md), [V02-01](../validations/A-MEM-01/V02-01.md), [V03-01](../validations/A-MEM-01/V03-01.md), [V05-01](../validations/A-MEM-01/V05-01.md)

### A-MEM-01-P3-01: Channels mode never spawns Dreaming — the documented "GUI, TUI, and CLI run the same Dreaming schedule" does not hold for the channels surface

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: spawn sites only `src/tauri/desktop.rs:247`, `src/tui/mod.rs:1999`,
  `src/cli/repl.rs:106`; channels mode receives `review_integration`
  (`src/cli/channels.rs:38,50,56`) and uses its layer manager per channel
  agent (:258) but contains no `spawn_dreaming_task` call (V01 grep).
- Reachability: `echo-agent-cli --channels` sessions (background IM surface).
- Expected invariant: `MASTER-PLAN.md:256` — every mode runs the same
  Dreaming schedule; AGENTS.md surface-parity rule.
- Observed behavior: in channels-only mode no Dreaming pass ever runs; hot
  promotion depends entirely on write-time auto-promotion.
- Impact: channels-only deployments get no recall-driven promotion/revival/
  demotion — a silent parity gap on a background surface.
- Root cause: Dreaming was wired into the three interactive surfaces; the
  channels entry point was not included.
- Direction: spawn the dreaming task in `run_channels_mode` (same
  `spawn_dreaming_task` with the shared `review_integration`, primary handle
  or pool, and a cancel token on shutdown).
- Regression validation: channels-mode fixture — assert a dreaming task is
  spawned at boot and cancelled at shutdown (or document channels as
  intentionally excluded and fix the doc).
- Validation reports: [V02-01](../validations/A-MEM-01/V02-01.md), [V05-01](../validations/A-MEM-01/V05-01.md)

### A-MEM-01-P3-02: Rule promotion and Dreaming hot promotion have no cross-channel dedup — a rule-marked memory is still hot-eligible, so the same fact appears in both always-loaded projections (learned-rules.md + MEMORY.md)

- Priority: P3
- Confidence: medium
- Layer: application (adapter between the two promotion channels)
- Evidence: `RulePromoter::scan_for_proposals` skips memories already
  containing `<!-- PROMOTED_TO_RULE` (rule_promoter.rs:137) and keeps them in
  warm; `Dreaming::run` (dreaming.rs:126-176) checks only status/recall_count/
  recency — no PROMOTED_TO_RULE marker check — and `consider_promotion`
  (layer.rs:685-707) + `promote_warm_to_hot` (:1109-1140) then move the same
  entry into MEMORY.md (deleting it from warm). The reverse is impossible
  (rule promotion scans warm only), so the overlap is one-directional.
- Reachability: a high-confidence memory that also accumulates recall_count
  ≥ 5 within 30 days while remaining Active in warm — the default criteria of
  both channels.
- Expected invariant: a fact survives in at most one always-loaded
  projection (mirrors the L3 content-hash dedup spirit; MASTER-PLAN:259-260).
- Observed behavior: the same fact can be simultaneously in learned-rules.md
  (as a rule) and MEMORY.md (as a hot memory); edits to one are not reflected
  in the other.
- Impact: duplicated prompt tokens (both are always in context), potentially
  contradictory rule-vs-memory statements; minor.
- Root cause: the two promotion channels were designed independently and no
  shared "already promoted elsewhere" signal exists (the rule marker is
  content-based and only consulted by RulePromoter).
- Direction: have `Dreaming::run` skip entries whose content carries the
  PROMOTED_TO_RULE marker (single shared marker constant), or move rule
  promotion to a shared eligibility layer; keep the rule path as the
  preferred permanent tier when both gates pass.
- Regression validation: fixture — warm memory with the marker + high recall,
  run one Dreaming pass, assert it is not promoted to hot (stays in warm);
  existing dreaming + rule_promoter tests stay green.
- Validation reports: [V03-01](../validations/A-MEM-01/V03-01.md)

### A-MEM-01-P3-03: `RulePromoter`'s write target is CWD-derived and can land outside the active instruction scope after `exit_workspace`, whose CWD is never restored (A-CFG-01-P1-02) — a promoted rule can be invisible to the agent

- Priority: P3
- Confidence: medium (depends on the A-CFG-01-P1-02 CWD staleness arm)
- Layer: application
- Evidence: `InstructionProvider::agents_instructions_path` resolves from
  `std::env::current_dir()` + `find_project_root` (instruction_provider.rs:291-297);
  `promote_rule` writes there (rule_promoter.rs:197-222); `exit_workspace`
  (state.rs:1053-1185) resets the projection scope to global (refresh with
  root=None, :1072) but never restores the process CWD; the only production
  exit caller is `delete_workspace` (A-CFG-01 P1-02), which removes the
  workspace directory afterwards.
- Reachability: GUI delete of the active workspace, then any rule promotion
  in the same session.
- Expected invariant: promoted rules land in the file that is actually
  projected for the current scope (workspace `.eko/learned-rules.md` or the
  global `~/.eko`).
- Observed behavior: after exit, the agent's instruction projection is
  global, but promotion writes into the exited workspace's
  `.eko/learned-rules.md` — a directory that is about to be (or already)
  deleted; the command reports success with no visible effect.
- Impact: silent no-op rule promotion after workspace deletion; the same
  root as A-CFG-01-P1-02 (CWD restore) and A-CFG-01-P1-01 (scope-bound
  subsystems not part of workspace state).
- Root cause: rule promotion resolves its target from the process CWD while
  the projection scope is determined by `working_dir`/workspace state — the
  two resolutions diverge exactly when CWD and scope disagree.
- Direction: resolve the rule target from the same scope the projection uses
  (agent `working_dir` → project root), i.e. add a
  `save_agents_instructions_for(root)` or pass the agent's working_dir into
  `RulePromoter`; coordinate with the A-CFG-01-P1-02 CWD-restore fix.
- Regression validation: fixture — `exit_workspace` then `promote_rule`,
  asserting the write lands in the global scope file (or fails explicitly);
  after the A-CFG-01 fix, assert CWD restore makes the current resolution
  correct again.
- Validation reports: [V03-01](../validations/A-MEM-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (instruction protocol / memory manager / Dreaming, both repos) | yes | passed | [V01-01](../validations/A-MEM-01/V01-01.md) |
| V02 | Registration and runtime reachability (boot/switch/exit/Dreaming/memory commands/pool refresh) | yes | passed (P1-01 evidence) | [V02-01](../validations/A-MEM-01/V02-01.md) |
| V03 | Invariants/edges: layer/precedence map, compression survival, refresh triggers, duplication/promotion, workspace-switch fixtures | yes | passed (violations → P1-01, P3-02, P3-03) | [V03-01](../validations/A-MEM-01/V03-01.md) |
| V04 | Targeted tests + compile: app-core instruction/unified_memory/rule_promoter/review_integration/agent_pool suites, `cargo check -p echo-agent-app-core`, framework dreaming/layer/compression suites | yes | passed, exit 0 each | [V04-01](../validations/A-MEM-01/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN :68/:245-261/:466, memory closure doc, deep-dive 04, framework self-improvement doc) | yes | passed (regressed rows → P1-01; stale row → P3-01) | [V05-01](../validations/A-MEM-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:251-256` — boot/switch/exit/Dreaming/hot-mutation/rule-promotion refresh the primary Agent immediately; pooled Agents refresh too | regressed | boot/switch/exit correct (state.rs:883/:1072, agent_pool.rs:553); Dreaming and hot-mutation paths refresh the instruction projection only → P1-01 |
| `MASTER-PLAN.md:256` — GUI, TUI, and CLI run the same Dreaming schedule | stale | channels mode never spawns Dreaming → P3-01 |
| `2026-07-23-memory-self-evolution-closure.md:11` — after Dreaming or manual hot-layer change the current and pooled Agents immediately refresh projections | regressed | refresh fires the wrong projection (P1-01) |
| `2026-07-23-memory-self-evolution-closure.md:19` — all instruction + hot memory in one `eko:instruction-context` projection | stale | two projections (unified_memory.rs:28-29); the split is the root cause of P1-01's wrong-refresh sites |
| `2026-07-23-memory-self-evolution-closure.md:20` — trigger list incl. Dreaming hot promotion and manual hot add/delete refresh current Agent + pool | regressed | each listed trigger fires instruction-only or nothing → P1-01 |
| `2026-07-23-memory-self-evolution-closure.md:21` — Dreaming first pass 60 s after boot, then daily | current | infra.rs:1156-1158 |
| `MASTER-PLAN.md:68` — instruction and hot memory use distinct replaceable projections | current | unified_memory.rs:28-29 |
| `echo-agent/docs/zh/25-self-improvement.md:258` — Dreaming is deterministic promotion/revive/archive, no semantic merge | current | dreaming.rs:126-176 |
| `MASTER-PLAN.md:259-260` — one content-derived key so the same fact is not persisted twice | current for L3; not extended to hot/rule channels | L3 dedup in memory_promoter; hot vs rule promotion overlap → P3-02 |
| `MASTER-PLAN.md:466-467` — verify instruction projection replacement, Dreaming's first pass, hot-memory budget | current as a plan; hot-memory projection replacement arm is exactly the regressed path | P1-01 |

## Coverage And Uncertainty

- No runtime session was launched; all behavioral claims are static traces
  (V02/V03). The P1-01 stale-projection effect (model seeing stale Active
  Memories) is argued from the code chain: projection content is fixed at
  refresh time, and no in-session refresh of the hot marker exists.
- `switch_workspace`/`exit_workspace` have no unit tests (state.rs has no
  test module — same gap as A-CFG-01); the workspace-switch fixture exists
  only for the instruction projection (unified_memory.rs:218-246), not for
  hot-memory replacement.
- The channels-mode Dreaming gap (P3-01) was verified by grep; no channels
  run was executed (requires IM credentials).
- The two MEMORY.md parsers (framework serde_yaml entries vs EKO body strip)
  were compared for body compatibility only; entry-metadata drift between
  them is a maintenance risk, not a current divergence.
- F-CMP-01-P1-02 (summary accumulation) and F-CTX-01-P2-02 (canonical
  reinjection) are owned by their framework findings; this task confirmed
  EKO projections neither aggravate nor mitigate them and are themselves
  protected from compression (V03-01/V04-01).
- F-EVO-01-P2-02 (L3 unaudited writes) cross-reference: EKO's RulePromoter
  scans the same warm store that L3 fills, so unaudited L3 content can be
  promoted into learned-rules.md; the promote-time security guard
  (rule_promoter.rs:184-193) preserves the secret-scan arm. Fix ownership
  stays with F-EVO-01-P2-02.

## Handoff

- Downstream tasks may rely on: single-authority instruction protocol
  (InstructionProvider + two projections) and single Dreaming driver with
  EKO-side scheduling (V01); boot/switch/exit refresh both projections on
  primary and pooled agents (V02); ALL in-session hot-layer mutations refresh
  the wrong projection or nothing — hot memory is stale until boot/switch/
  exit (P1-01); channels mode has no Dreaming (P3-01); hot/rule promotion
  channels have no cross-dedup (P3-02); RulePromoter's CWD-derived write
  target diverges from the projection scope after exit (P3-03, depends on
  A-CFG-01-P1-02); projections survive compression (V03).
- Reports to read: this report + V01-01..V05-01; A-CFG-01 (P1-01/P1-02/P2-02
  workspace scope arms); F-CMP-01 (P1-01/P1-02 pipeline stability);
  F-MEM-01 (store durability); F-EVO-01 (P2-02 L3 audit bypass).
- Conditions that make this report stale: any change to
  `instruction_provider.rs` / `unified_memory.rs` refresh functions or
  markers; `infra.rs` dreaming task or `refresh_dynamic_context`; the
  memory-command refresh sites (memory.rs, events.rs, all.rs); the pool
  refresh helpers (`agent_pool.rs:534-710`); `state.rs` switch/exit memory
  rebinding; framework `dreaming.rs`/`layer.rs` promotion semantics; the
  closure-doc/MASTER-PLAN projection rows.
- Follow-up task IDs (fixes are not implemented in this review):
  X-MEM-01 (instruction/memory/context/compression conformance — consumes
  this report's P1-01), A-EVO-01 (evolution product scope — Dreaming
  scheduling + review-only gate), A-SRF-04/X-SRF-01 (channels Dreaming
  parity row), Q-TST-01 (hot-projection refresh fixture coverage), Q-DOC-01
  (closure-doc single-projection wording), Q-CLI-01 (gate unaffected).
