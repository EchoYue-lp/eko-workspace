# X-MEM-01: Instruction, memory, context, and compression conformance

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Can EKO-specific instruction/memory layers use generic context and
compression without duplicate persistence or lost updates?

Answer in one sentence: the machinery is structurally sound (single
instruction authority, projections protected from compression, content-hash
dedup, destination-first layer transitions), but today the system does NOT
satisfy the two invariants — in-session hot-memory mutations never reach the
live context (canonical A-MEM-01-P1-01), and the generic compression pipeline
does not guarantee a bounded or within-limit context (canonical
F-CMP-01-P1-01/P1-02) — plus two dormant parallel mechanisms and one split
file authority recorded as new findings below.

## Scope

- EKO instruction/memory protocol: `echo-agent-app-core/src/instruction_provider.rs`
  (full), `unified_memory.rs` (full), `infra.rs` (create_agent system-prompt
  assembly :200-330, `refresh_dynamic_context` :527-531, dreaming task
  :1132-1212, memory store resolution :1269-1367), `state.rs`
  `switch_workspace` :844-1032 / `exit_workspace` :1053-1185, `agent_pool.rs`
  (`apply_working_dir` :534-560, `apply_memory_store*` :597-684,
  `refresh_instruction_context` :687-710, `create_agent` :824-978),
  `evolution/rule_promoter.rs` (:95-268), `utils.rs` (`strip_yaml_frontmatter`
  :60-86), `tasks/task_runtime/memory_bridge.rs` (:88-165),
  `tasks/task_runtime/compact_context.rs` (protected marker :156-160).
- EKO surfaces: `src/tauri/commands/memory.rs`, `src/tauri/commands/panels.rs`
  (:1550-1575), `src/tui/events.rs` (:2830-2930), `src/cli/cmd_impls/all.rs`
  (:110-210), `src/cli/cmd_impls/evolution.rs` (:1480-1495), `src/main.rs`,
  `src/tauri/desktop.rs` (creation sites).
- Framework context/compression machinery: `echo-state/src/compression/mod.rs`
  (`prepare` :1243-1539, projections :546-741, `split_protected`/`merge_protected`
  :747-793, `is_protected` :679-691), `compressor/{sliding_window,summary}.rs`
  (:41-66 / :285-348), `echo-core/src/compression.rs`
  (`to_reinjection_messages` :376-401), `echo-core/src/project_rules.rs`
  (:176-218), `echo-agent/src/agent/react/mod.rs` (`new_inner` :322-384,
  `build_system_prompt` :676-730, `set_working_dir` :907-921),
  `echo-agent/src/evolution/layer.rs` (:685-810, :837-886, :1059-1191,
  :1239-1341), `dreaming.rs` (full), `memory_promoter.rs`,
  `runtime_integration.rs` (observer :127-140).
- Executed tests: echo_state `compression::` (69), app-core
  `instruction_provider` (8) / `unified_memory` (4) / `rule_promoter` (3),
  framework `evolution::dreaming` (4) / `evolution::layer` (19) — all green
  (V03-01, V04-01); one dynamic probe (V03-01) exercising repeated
  compression and count-window stall.

## Out Of Scope

- Store durability/atomicity details → F-MEM-01 (complete); only the warm
  store's role as EKO memory persistence is cross-referenced.
- Compressor fidelity/summary quality, verifier heuristics → F-CMP-01
  (complete); only the stability and projection-survival arms are re-traced.
- Budget allocation percentages and window inference internals → F-CTX-01
  (complete); the 396K wiring is cross-referenced.
- Task-brief capsule semantics (`compact_context.rs` runtime recovery) →
  A-TSK-06; only its protected-marker registration is noted.
- Workspace-switch config/hook/watcher scope and CWD restore → A-CFG-01
  (complete); CWD staleness is used as a dependency for the rule-promotion
  write-target arm.
- Eval/evolution product scope and audit bypass → F-EVO-01 / A-EVO-01.
- No dynamic GUI/TUI/CLI session was launched; behavior claims are static
  traces plus the /tmp probe (V03-01).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (X-MEM-01 card), `zcode-ds/README.md`, templates.
- Dependency task reports read in full: zcode-ds `F-CTX-01` (complete),
  `F-MEM-01` (complete), `F-CMP-01` (complete), `A-MEM-01` (complete); A-MEM-01
  validation reports V01-01..V05-01 sampled (V03-01/V04-01 read).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  (:68, :245-261, :466-467), `docs/2026-07-23-memory-self-evolution-closure.md`
  (:10-21, :45-52), `echo-agent-cli/docs/configuration.md` (:61).

## Layering Decision

- Generic mechanism (framework): `ContextManager` projection/protected-marker
  machinery, all compressors + sanitizer + verifier, `CanonicalContext`
  re-injection, `MemoryLayerManager` (hot/warm layers, promotion/demotion,
  budget, security guard), `Dreaming` driver, `StoreMemoryPromoter` (L3),
  `EvolutionObserver` notification — correctly placed.
- EKO product policy (application): the five-tier instruction file protocol
  (user / repository AGENTS chain / project / learned-rules / local), the two
  replaceable projections, the refresh policy and its trigger sites,
  `RulePromoter` + learned-rules.md protocol, Dreaming scheduling per surface,
  workspace-scoped memory store resolution and rebinding,
  `strip_yaml_frontmatter` read of MEMORY.md — correctly placed.
- Adapter boundary: `UnifiedMemory`, `refresh_*_projection` helpers, pool
  store/layer-manager rebinding, `ReviewIntegration` factories — thin and
  lossless; no scheduling/state authority in the adapters.
- Duplicate search (both repositories; terms in V01-01):
  `InstructionProvider`, `UnifiedMemory`, `refresh_instruction_projection`,
  `refresh_hot_memory_projection`, `refresh_memory_projections`,
  `refresh_instruction_context`, `MemoryLayerManager`, `write_memory`,
  `WARM_NAMESPACE`, `replace_projection`, `eko:instruction-context`,
  `eko:hot-memory-context`, `MEMORY.md`, `learned-rules.md`,
  `memory_context_suffix`, `project_rules`, `auto_project_rules`,
  `PROMOTED_TO_RULE`, `l3_`, `parse_memory_md`, `strip_yaml_frontmatter`.
  Results: one live authority per semantic; three recorded divergences →
  `memory_context_suffix` dormant parallel mechanism (X-MEM-01-P2-01);
  MEMORY.md dual parser (X-MEM-01-P2-02); rule/hot dual-file persistence
  (canonical A-MEM-01-P3-02).

## Current Path

Verified data flow (anchors in V01-01/V02-01/V03-01):

1. Boot/switch/exit: `InstructionProvider::load_for(root)` reads the five
   tiers + hot MEMORY.md (instruction_provider.rs:61-82, :251-259);
   `refresh_memory_projections` (unified_memory.rs:170-186) installs both
   projections on the primary agent (infra.rs:444, state.rs:883/:1072) and
   pooled agents (agent_pool.rs:553) via `refresh_dynamic_context`
   (infra.rs:528-531).
2. Memory writes: `write_memory` (layer.rs:837-886) → warm put → `consider_promotion`
   (:685-707) → `promote_warm_to_hot` (:1109-1144, hot-first, warm deleted
   after) → `enforce_hot_budget` (:711-810, demote to warm); notifications go
   to the `EvolutionObserver` user hooks only (runtime_integration.rs:127-140).
3. Dreaming: 60 s after boot, then daily (infra.rs:1156-1158); on
   `report.promoted > 0` the primary and pool refresh the INSTRUCTION
   projection only (infra.rs:1175-1192, agent_pool.rs:687-710).
4. In-session hot mutations (GUI add/delete, TUI/CLI /remember /forget) all
   fire `refresh_instruction_projection` (memory.rs:134/:227,
   events.rs:2846/:2917, all.rs:130/:201) — the hot projection is never
   updated in-session; `refresh_hot_memory_projection` has zero production
   callers (unified_memory.rs:154-167).
5. Rule promotion: `RulePromoter::scan_for_proposals` (warm, Active,
   confidence/type/age gates, skips `<!-- PROMOTED_TO_RULE` :137) →
   `promote_rule` appends to CWD-derived learned-rules.md and marks the warm
   entry (:178-268); the refresh targets the instruction projection with the
   agent's working_dir (panels.rs:1556-1570, evolution.rs:1485-1495) — the
   correct projection for learned-rules.md.
6. Compression: `prepare` (mod.rs:1243-1539) splits protected projections out
   (:1336), compresses, merges them back (:1353), sanitizes, verifies,
   re-injects canonical context (:1528); projections survive every pass
   exactly once (V03-01 probe), but the count-window compressors can return
   over-limit context unchanged (sliding_window.rs:48-66) and summary
   compressors accumulate one immortal summary per pass (summary.rs:346-348).
7. Window wiring: EKO uses `DEFAULT_CONTEXT_WINDOW = 396_000` (infra.rs:23,
   :215-219, :258-262) without provider inference (canonical F-CTX-01-P1-01).

## Findings

### X-MEM-01-P2-01: `memory_context_suffix` / `PromptAssembler::add_instruction_context` is a dormant parallel instruction-context mechanism that no production entry point exercises — the documented injection never happens

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:226-231` —
  `PromptAssembler::add_instruction_context(memory_suffix)` with the comment
  "Inject the unified instruction/profile context so the agent's system prompt
  reflects EKO user/project/local instruction files"; `infra.rs:131`
  (`memory_context_suffix` param); all four production construction sites pass
  `None`: `echo-agent-cli/src/main.rs:161` and `:492`,
  `src/tauri/desktop.rs:150`, `echo-agent-app-core/src/agent_pool.rs:841`.
- Reachability: definition + registration (the branch exists in
  `create_agent`) but no production caller ever supplies a value; the live
  instruction-context path is the two projections via
  `refresh_dynamic_context` (infra.rs:528-531). Only a hypothetical future
  caller could reach it.
- Expected invariant: one instruction-context mechanism; any second
  mechanism either delegates or is removed (AGENTS.md "严禁平行实现同一
  语义").
- Observed behavior: a full second instruction-injection path (with its own
  budget model inside `PromptAssembler`) is compiled in, documented as live,
  and never executed; the system prompt it would extend is built at
  `infra.rs:254-255` without any instruction tier.
- Impact: misleading API and comment; if a fixer "re-wires" the projection
  refresh through `memory_context_suffix`, the model would receive
  instructions twice (system prompt + projection) — the exact duplication the
  task question asks about; maintenance surface today.
- Root cause: the system-prompt assembly path predates the projection design;
  when instructions moved to replaceable projections, the parameter was left
  in place with its doc unchanged.
- Direction: delete `memory_context_suffix` + the `add_instruction_context`
  branch (or make the comment state that instruction context is
  projection-owned); add a compile-time/grep regression assertion that no
  construction site passes a non-None value.
- Regression validation: grep `memory_context_suffix` returns only the
  declaration site; `cargo check -p echo-agent-app-core --locked` green.
- Validation reports: [V01-01](../validations/X-MEM-01/V01-01.md)

### X-MEM-01-P2-02: MEMORY.md — the hot-layer file — has two independent parsers; when the file diverges from the machine format, the model projection and the layer manager disagree (hot budget/search silently no-op)

- Priority: P2
- Confidence: medium (code facts high; trigger requires a non-conforming file)
- Layer: adapter (application read vs framework write/parse boundary)
- Evidence: EKO projection read = `strip_yaml_frontmatter`
  (`echo-agent-cli/echo-agent-app-core/src/utils.rs:60-86`) — fence-based
  body strip, never parses entries; framework parse =
  `parse_memory_md` (`echo-agent/src/evolution/layer.rs:1239-1284`) — YAML
  frontmatter `entries` via serde_yaml_ng + `- **[key]**` bullet body;
  `extract_hot_entry_content` (layer.rs:1320-1331) keys on the exact bullet
  pattern; `enforce_hot_budget` (layer.rs:711-810) iterates only
  `file.entries` (frontmatter-derived).
- Reachability: normal operation is consistent (the layer manager writes both
  frontmatter and bullets via `format_memory_md`/`add_to_hot`,
  layer.rs:1059-1084). Divergence requires a user-edited MEMORY.md (the file
  is documented as editable) or a malformed/absent frontmatter: then the
  model still sees the body (fence strip is YAML-agnostic), while
  search (`search_layered` :893-931) and budget enforcement
  (:711-810) can silently see zero entries — e.g. frontmatter-less or
  YAML-unparseable MEMORY.md makes `enforce_hot_budget` a no-op and the hot
  layer can grow past `HOT_TOKEN_BUDGET` (layer.rs:95) without demotion.
- Expected invariant: one interpretation of the hot-layer file: whatever the
  model sees as "Active Memories" is addressable by hot search and the
  budget.
- Observed behavior: two interpretations of one file; divergence is silent
  (no warning, no reconciliation).
- Impact: unbounded hot-layer growth on malformed/edited files; hot entries
  invisible to search/demote while visible to the model (or vice versa);
  contradiction risk for the "hot memory" product invariant.
- Root cause: the projection read path and the layer-manager parse path were
  written independently; neither validates the other's assumptions, and
  `parse_memory_md` degrades silently to empty entries on YAML errors.
- Direction: make the projection read reuse the framework parse (or add an
  explicit entry-less/parse-failed signal to the projection so the model and
  the layer manager see the same state); warn on frontmatter-less MEMORY.md;
  add a budget-enforcement test with a frontmatter-less file.
- Regression validation: unit fixture — MEMORY.md without frontmatter, write
  past `HOT_TOKEN_BUDGET`, assert `enforce_hot_budget` either demotes or
  reports the unparseable state; a search fixture asserting hot entries match
  the projected body.
- Validation reports: [V01-01](../validations/X-MEM-01/V01-01.md),
  [V04-01](../validations/X-MEM-01/V04-01.md)

### X-MEM-01-P3-01: `AgentPool::refresh_instruction_context` doc claims "hot-memory and instruction projections" but refreshes instruction only — the misleading comment that seeded the wrong-target refresh sites

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:686` doc
  comment "Refresh hot-memory and instruction projections on every existing
  agent" vs body :699-708 calling only
  `refresh_instruction_projection` (unified_memory.rs:138-151); the pool is
  the only pooled-agent refresh entry point used by Dreaming
  (infra.rs:1191) and all surface commands (V02-01 inventory).
- Reachability: every pooled-agent refresh triggered by a hot-layer change —
  the call always runs, always with the wrong target.
- Expected invariant: documentation matches behavior, and the function named
  by the pool contract covers both projections.
- Observed behavior: the doc promises hot memory; the body delivers
  instruction context only; the mismatch is invisible to callers, which is
  how the eight wrong-target sites (canonical A-MEM-01-P1-01) went unnoticed.
- Impact: contributes to the P1-01 stale-hot-projection defect by making the
  refresh look correct; a fix that renames/repurposes this function must
  first fix the comment.
- Root cause: written during the single-projection design era (closure-doc
  line 19) and never updated after the hot/instruction split
  (unified_memory.rs:28-29).
- Direction: make the body refresh both projections via
  `refresh_memory_projections` (primary fix for the pool arm of
  A-MEM-01-P1-01) or correct the doc; covered by the canonical finding's
  fix.
- Regression validation: pool fixture — hot MEMORY.md change, call
  `refresh_instruction_context`, assert `has_projection(HOT_MEMORY_CONTEXT_PROJECTION)`
  content changed (the fixture does not exist today, V04-01).
- Validation reports: [V02-01](../validations/X-MEM-01/V02-01.md),
  [V04-01](../validations/X-MEM-01/V04-01.md)

### Canonical findings revalidated at the reviewed commits (folded, not re-filed)

| Canonical ID | Status at 9b0e0fa/b3b2e81 | Evidence re-traced | X-MEM-01 relevance |
|---|---|---|---|
| A-MEM-01-P1-01 hot-memory projection never refreshes (every in-session hot mutation refreshes the wrong projection or none; stale until boot/switch/exit) | current | full site inventory rebuilt in V02-01 | the central "lost update" of the task question |
| F-CMP-01-P1-01 count-window compressors never bound tokens; prepare never re-checks | current | statically + dynamically (V03-01 probe: limit 100, 6 msgs, `compressed=true`, output unchanged) | memory/instruction layers ride a pipeline that can send over-limit context |
| F-CMP-01-P1-02 one immortal summary per compression pass | current | summary.rs:292-296/:346-348 | system region (incl. protected projections) grows with repeated compression |
| F-CMP-01-P1-03 adaptive L1 fold breaks tool-call/tool-result contiguity | current | levels.rs:392-396 re-traced | same prepare chain the projections ride |
| F-CTX-01-P1-01 provider window bypassed; EKO hardcodes 396K | current | infra.rs:23/:218/:261, react/mod.rs:336-343 | the window the memory/instruction layers are budgeted against |
| F-CTX-01-P2-02 canonical re-injection truncates rules to 2000 chars; skill-injection staleness | current | echo-core/src/compression.rs:376-401, react/mod.rs:379 | instruction survival after compression (EKO `project-rules` feature off → rules arm dormant in EKO, stale-prompt arm framework-wide) |
| F-EVO-01-P2-02 L3 promotion bypasses audit | current | memory_promoter.rs:71/:91 | L3 fills the same warm namespace the instruction protocol promotes from |
| F-MEM-01-P1-01 FileStore silently discards corrupt store and overwrites it | current | echo-state/src/memory/store.rs:235-238/:254-278; infra.rs:1319-1349 | the warm half of EKO's memory persistence |
| A-MEM-01-P3-02 rule and hot promotion channels have no cross-dedup (fact in learned-rules.md AND MEMORY.md) | current | dreaming.rs:126-176 vs rule_promoter.rs:137 | the "duplicate persistence" arm of the task question (always-loaded duplication) |
| A-MEM-01-P3-03 RulePromoter write target CWD-derived, diverges from projection scope after exit_workspace | current | instruction_provider.rs:291-297, rule_promoter.rs:197-222, state.rs:1053-1185 | write-side lost update (depends on A-CFG-01-P1-02) |
| F-CMP-01-P3-02 EKO doc "token_limit 0 = disabled" regressed | current | infra.rs:258-262 | window wiring the layers depend on |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Source/precedence map + duplicate persistence search (both repos) | yes | passed (2 new findings, 1 canonical fold) | [V01-01](../validations/X-MEM-01/V01-01.md) |
| V02 | Immediate refresh — every hot-layer mutation → correct projection on primary and pool | yes | failed (invariant violated; canonical A-MEM-01-P1-01) | [V02-01](../validations/X-MEM-01/V02-01.md) |
| V03 | Repeated compression — projection survival exactly-once + convergence; echo_state compression:: suite (69) + dynamic probe | yes | passed (survival holds; stall reproduced under canonical F-CMP-01-P1-01) | [V03-01](../validations/X-MEM-01/V03-01.md) |
| V04 | Workspace-switch fixture + duplicate-promotion fixtures; app-core unified_memory/rule_promoter, framework dreaming/layer suites | yes | passed (4+3+4+19 green; coverage gaps recorded) | [V04-01](../validations/X-MEM-01/V04-01.md) |
| V05 | Cross-reference with existing findings (canonical IDs) + historical-document drift | yes | passed (all canonical current; classification table) | [V05-01](../validations/X-MEM-01/V05-01.md) |

All required validations executed; every command has a known exit code; the
failed invariant in V02 is recorded as a finding (canonical A-MEM-01-P1-01)
per REPORTING.md.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:251-256` — boot/switch/exit/Dreaming/hot-mutation refresh the primary Agent immediately; pooled Agents refresh too | regressed | boot/switch/exit correct (state.rs:883/:1072, agent_pool.rs:553); Dreaming + all in-session hot mutations refresh instruction-only or nothing (V02-01) → A-MEM-01-P1-01 |
| `MASTER-PLAN.md:259-260` — one content-derived key so the same fact is not persisted twice | current for L3; not extended to hot/rule channels | `l3_{hash}` dedup (memory_promoter.rs:57,91); hot vs rule overlap → A-MEM-01-P3-02 |
| `2026-07-23-memory-self-evolution-closure.md:11,20` — after Dreaming or manual hot-layer change, current and pooled Agents immediately refresh projections | regressed | every listed trigger fires instruction-only or nothing (V02-01) |
| `2026-07-23-memory-self-evolution-closure.md:19` — all instruction + hot memory in one `eko:instruction-context` projection | stale | two projections (unified_memory.rs:28-29); the split is the root of the wrong-refresh sites |
| `echo-agent-cli/docs/configuration.md:61` — token_limit 0 disables compression | regressed | infra.rs:258-262 (0 → 396K) + react/mod.rs:346-353 → F-CMP-01-P3-02 |
| `MASTER-PLAN.md:68` — instruction and hot memory use distinct replaceable projections | current | unified_memory.rs:28-29; survival verified (V03-01) |

## Coverage And Uncertainty

- No runtime EKO session (GUI/TUI/CLI) was launched; all refresh-latency
  claims are static call-site traces (V02-01). The P1-01 effect (model seeing
  stale Active Memories) is argued from the code chain, not observed in a
  live session.
- The dynamic probe (V03-01) exercised `ContextManager` directly with
  `SlidingWindowCompressor`; the summary accumulation (F-CMP-01-P1-02) is
  static evidence only (needs an LLM), consistent with F-CMP-01's own
  coverage.
- `state.rs` has no test module; switch/exit memory rebinding is verified by
  inspection only (V04-01).
- X-MEM-01-P2-02's trigger (non-conforming MEMORY.md) was not exercised
  dynamically; impact magnitude is argued from the enforcement loop.
- The `project-rules` feature is disabled in EKO builds (app-core
  Cargo.toml:10-15), so the framework auto-project-rules duplication and the
  2000-char rules truncation are dormant on the EKO path — noted to avoid a
  false "duplicate persistence" conclusion (V05-01).
- F-CMP-01-P1-03 (adaptive fold) is re-traced at its anchor only; it is not
  exercised by EKO's default `"summary"` strategy.

## Handoff

- Downstream tasks may rely on: single-authority instruction protocol and
  exactly-once projection survival under repeated compression (V01, V03);
  the full refresh-site inventory (V02) confirming A-MEM-01-P1-01 at these
  commits; the dormant `memory_context_suffix` path (X-MEM-01-P2-01) that
  must not be used to "fix" the refresh; the dual MEMORY.md parser divergence
  (X-MEM-01-P2-02); the rule/hot dual-file persistence (A-MEM-01-P3-02).
- Reports to read: this report + V01-01..V05-01; A-MEM-01 (P1-01, P3-01..03),
  F-CMP-01 (P1-01/02/03, P2-01), F-CTX-01 (P1-01, P2-02), F-MEM-01 (P1-01),
  F-EVO-01 (P2-02).
- Conditions that make this report stale: any change to
  `instruction_provider.rs` / `unified_memory.rs` refresh helpers or markers;
  the refresh call sites (memory.rs, events.rs, all.rs, infra.rs dreaming
  block, agent_pool.rs:534-710); `state.rs` switch/exit memory rebinding;
  `echo-state/src/compression/` projection/split/merge/prepare code; the
  `memory_context_suffix`/PromptAssembler wiring; MEMORY.md parsers
  (utils.rs, layer.rs).
- Follow-up task IDs (fixes are not implemented in this review): A-MEM-01
  (P1-01 fix — refresh both projections at all eight sites and wire the
  observer), F-CMP-01 (P1-01/P1-02 stability), F-CTX-01 (P1-01 window
  wiring), X-BND-01 (P2-01/P2-02 authority decisions), Q-TST-01 (hot-memory
  workspace-switch twin fixture, budget-enforcement fixture for
  frontmatter-less MEMORY.md), Q-DOC-01 (P3-01 comment, infra.rs:226-231
  comment, closure-doc rows).
