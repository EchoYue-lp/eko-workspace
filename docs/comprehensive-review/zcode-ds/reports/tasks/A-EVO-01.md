# A-EVO-01: EKO evolution product scope

> Status: complete
> Reviewer: ZCode (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Has EKO kept evolution as explicit diagnostics/review without hidden metric
loops, automatic semantic mutation, or framework option deletion?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/evolution/` (mod.rs, evidence.rs,
  rule_promoter.rs, review_integration.rs, dashboard.rs, hook_fire.rs),
  `auto_memory/mod.rs`, `infra.rs` (Dreaming task :1143-1212), `state.rs`
  (review-integration rebind :948-1013), `agent_pool.rs` (observer/trigger
  sink wiring :915-927), `runtime.rs` (:231-258 trigger sink, :411-490
  reflection API).
- `echo-agent-cli/src/cli/cmd_impls/evolution.rs` (17 commands),
  `src/cli/repl.rs` (exit hooks :250-262, register :172), `src/tui/events.rs`
  (evolution slash commands), `src/tauri/commands/panels.rs` (review/evidence/
  curator/dashboard/rule/skill commands), `src/tauri/commands/memory.rs`,
  `web-frontend/src/components/evolution/EvolutionPanel.tsx` +
  `src/api/endpoints.ts` (`evolutionApi`).
- Framework cross-references (boundary only): `echo-agent/src/evolution/`
  (triggers, background_review, review, dreaming, curator, layer, security,
  runtime_integration), `src/agent/react/run/{react_loop,context,execution}.rs`
  (trigger capture per turn), `echo-state/src/compression/mod.rs` (L3
  promoter invocation).

## Out Of Scope

- Framework evolution API validity / feature gating → F-EVO-01 (complete).
- Memory protocol, projections, compression survival → A-MEM-01 (complete).
- L3 promotion audit/security bypass itself → F-EVO-01-P2-02 (complete;
  this task records only how EKO's evolution surfaces expose it).
- Hot-memory projection staleness itself → A-MEM-01-P1-01 (complete; this
  task records only the evolution-surface exposure).
- Skill lifecycle internals beyond user-gate verification → A-SUB-01,
  F-SKL-01.
- Full TUI/GUI/CLI parity matrix → A-SRF-01..04, X-SRF-01 (the evolution
  surface gap found here is handed off there).
- `docs/comprehensive-review/codex/` and `zcode-glm/` — not read per review
  protocol.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-EVO-01 card), `zcode-ds/README.md`, templates.
- Dependency reports read in full: `F-EVO-01.md` (framework eval/improve/
  evolution; P2-02 L3 promoter), `A-MEM-01.md` (instruction/memory protocol;
  P1-01 hot projection, P3-01 channels Dreaming, P3-02 hot/rule dedup,
  P3-03 rule-target CWD).
- Historical documents treated as hypotheses:
  `echo-agent-cli/docs/2026-07-15-self-evolution-review-and-roadmap.md`,
  `docs/2026-07-23-memory-self-evolution-closure.md`, `MASTER-PLAN.md` (:62,
  :245-264), `docs/system-deep-dive/04-memory.md` (§9),
  `docs/system-deep-dive/07-cross-cutting.md` (:141-142),
  `echo-agent-cli/docs/configuration.md`, `getting-started.md` (sampled).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Framework owns: `MemoryLayerManager` (hot/warm, promotion, audit+guard), `Dreaming` deterministic driver, `TriggerDetector`/`MemoryTriggerSink`, `BackgroundReviewer` (proposal-only default), `MemoryReviewer`, `Curator`, `SkillCandidateDetector`/`SkillDraftGenerator`/`SkillMerger`/`SkillPatcher`/`SkillHealthMonitor`, `ChangeLog`/`JsonlChangeLog`, `EvolutionSecurityGuard`, `HookEvolutionObserver`, `StoreMemoryPromoter` (L3). Single authority per concept — verified V01-01. |
| EKO product policy | Review Inbox (JSONL evidence-candidates protocol, accept/undo/rollback), `RulePromoter` → learned-rules.md with PROMOTED_TO_RULE marker, `ReviewIntegration` (trigger sink + skill load policy + curator binding + session-end gate), workspace-scoped curation, Dreaming scheduling per surface, on-demand Dashboard, `auto_memory` inbox routing, `/reflect` + REPL exit reflection, `AUTO_MEMORY_ENABLED` session toggle, per-surface command exposure. Correctly application. |
| Adapter boundary | `review_integration.rs`, `auto_memory/mod.rs`, `hook_fire.rs` are thin, lossless adapters (no second scheduling loop, no second state authority; the only product authority is the inbox gate). |
| Duplicate search | Terms: evolution, curator, dreaming, evidence, EvidenceStore, RulePromoter, ReviewIntegration, MemoryTriggerSink, TriggerDetector, BackgroundReviewer, MemoryReviewer, MemoryLayerManager, SkillCandidateDetector, SkillDraftGenerator, SkillMerger, SkillPatcher, SkillHealthMonitor, ChangeLog, HookEvolutionObserver, reflection, PROJECT.md, AUTO_MEMORY_ENABLED, worker. Results: zero semantic duplicates (V01-01); one dead duplicate writer for the reflection concept (P3-02). |
| Migration deletion | None recommended at framework level; EKO-side: delete `runtime.rs:413-423` dead reflection API or wire it to the live REPL path; delete the `/critiques clear` no-op branch. |

## Current Path

Verified data flow (anchors in V02-01/V03-01):

1. **Trigger capture (per turn, review-gated)**: every turn
   `detect_and_write_memory_triggers` (`echo-agent/src/agent/react/run/
   react_loop.rs:558`, context.rs:509/579) → `TriggerDetector` → EKO
   `ReviewIntegration::on_trigger` (review_integration.rs:277-321) upserts an
   `EvidenceCandidate` into the workspace inbox and returns `Captured` (no
   durable memory write); on inbox failure it fail-closes (drops the evidence
   with a warn log) rather than falling through to the framework's direct
   write path (:314-318). Sink installed on the primary agent
   (runtime.rs:256) and pool agents (agent_pool.rs:924).
2. **Review Inbox (user-gated)**: `EvidenceStore` (evidence.rs:335-402)
   dedup by SHA-256 fingerprint, append-only JSONL with fs2 file locks,
   interaction events; `accept` (evidence.rs:478-636) writes through the
   layer manager with rollback on record failure; `undo` (evidence.rs:638)
   restores snapshots; stale merges fail before mutation. Surfaces: CLI
   `/evidence-inbox` (evolution.rs:1677), TUI (events.rs:3832), GUI
   `evidence_candidate_action` (panels.rs:1329).
3. **Rule promotion (user-gated)**: CLI `/rule-promote` (evolution.rs:1415)
   and GUI `promote_rule` (panels.rs:1515) → `RulePromoter::scan_for_proposals`
   (WARM_NAMESPACE, confidence/type/age gates) → `promote_rule`
   (rule_promoter.rs:178) security-checked, appends learned-rules.md, marks
   the memory, records a change, fires `RulePromoted` hook, refreshes the
   instruction projection (correct target — learned-rules.md is part of the
   instruction context).
4. **Dreaming (automatic, deterministic)**: `spawn_dreaming_task`
   (infra.rs:1143) on GUI/TUI/REPL (desktop.rs:247, tui/mod.rs:1999,
   repl.rs:106; channels has none — A-MEM-01-P3-01), 60 s after boot then
   daily, promote/revive/archive via a fresh layer manager (audited+scanned);
   `report.promoted > 0` triggers the primary + pool refresh
   (infra.rs:1175-1192) — of the instruction projection, NOT the hot-memory
   projection (A-MEM-01-P1-01 exposure, P2-02 here).
5. **Automatic un-audited writes**: L3 `StoreMemoryPromoter` fires on every
   compression pass (`echo-state/src/compression/mod.rs:836-838` →
   `echo-agent/src/memory_promoter.rs:70-94`) with direct `put_typed`
   (F-EVO-01-P2-02); EKO evolution surfaces consume that store (P2-01 here).
6. **CLI REPL exit sequence** (repl.rs:250-262): auto-memory extraction →
   inbox queue; **reflection → `.eko/memory/PROJECT.md` (automatic LLM
   write, P1-01)**; memory review (disabled by default).

## Findings

### A-EVO-01-P1-01: CLI REPL session exit runs an automatic LLM "reflection" that appends its output to `.eko/memory/PROJECT.md` — a semantic write outside the review gate, change log, and security guard, undocumented and CLI-only

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/repl.rs:256` (call in the REPL exit
  sequence), `:340-411` (`run_reflection_on_exit`: LLM prompt "summarize key
  learnings in 1-2 sentences", `tokio::time::timeout(2s)` chat call :369-377,
  append to CWD-resolved `.eko/memory/PROJECT.md` :379-400). Guarded only by
  `message_count < 4` (:359) and LLM availability. No user prompt, no
  confirmation, no `ChangeLog` entry, no `EvolutionSecurityGuard` scan, no
  size bound. Compare the documented review-gate paths: rule promotion
  (rule_promoter.rs:183-193), trigger capture (review_integration.rs:314-318),
  Review Inbox accept (evidence.rs:478).
- Reachability: every interactive CLI REPL session exit with ≥4 messages and
  a configured LLM client — the default REPL exit path (`repl.rs:256`); not
  reachable from TUI (`src/main.rs`, `src/tui/mod.rs`) or GUI
  (`src/tauri/desktop.rs`) — grep shows zero reflection calls there.
- Expected invariant: `docs/2026-07-15-self-evolution-review-and-roadmap.md:84`
  ("自动写入只允许用户明确保存、明确纠正等低歧义事件") and :5 (evolution model
  "证据候选 → 用户确认 → 可审计写入"), `docs/2026-07-23-memory-self-evolution-closure.md:12`
  (semantic content changes go through Review Inbox/manual action);
  `system-deep-dive/04-memory.md:428` ("没有自动 post-run hook;用户必须显式
  触发").
- Observed behavior: LLM-generated session learnings are appended to
  PROJECT.md automatically on every REPL exit, un-audited, un-scanned, and
  unbounded; the only reader is the CLI `/memory show` display
  (cmd_impls/all.rs:250-255) — the file is never loaded into agent context.
- Impact: (a) violation of the documented "no automatic semantic mutation
  without user confirmation" product invariant on a live default path — the
  exact behavior the task question asks about; (b) LLM output (which may
  contain secret-like text) lands in a file without `SecretScanner`
  redaction; (c) unbounded append growth; (d) CWD-derived target path can
  land in the wrong workspace after `exit_workspace` (same root as
  A-CFG-01-P1-02); (e) CLI-only — GUI/TUI sessions never produce the
  reflection, a silent surface-parity deviation. Not P0: the file is local
  and not loaded into agent context (no behavior corruption).
- Root cause: the reflection-on-exit was added as a standalone REPL convenience
  (and mirrored as a dead API in app-core, see P3-02) outside the evolution
  review-gate architecture that every other EKO semantic write goes through.
- Direction: either (a) remove `run_reflection_on_exit` and route session
  learnings through the existing auto-memory extraction → Review Inbox path
  (repl.rs:253 already does this correctly), or (b) make it explicit
  opt-in/ask and write through the change log + security guard with a size
  bound; delete the CWD-derived path in favor of the workspace-resolved
  `WorkspaceLayout::memory` root. Keep `/reflect` (user-triggered) or also
  gate it behind review.
- Regression validation: REPL-exit fixture — session with ≥4 messages and a
  scripted LLM client; assert either (a) no PROJECT.md write occurs or the
  write is queued as an EvidenceCandidate in the inbox, or (b) the write is
  preceded by a confirmation and produces a `ChangeEntry`; existing
  `run_auto_memory_on_exit` tests behavior unchanged; `/memory show`
  still renders the tier.
- Validation reports: [V02-01](../validations/A-EVO-01/V02-01.md), [V03-01](../validations/A-EVO-01/V03-01.md), [V05-01](../validations/A-EVO-01/V05-01.md)

### A-EVO-01-P2-01: EKO's evolution surfaces expose the L3 promoter's unaudited/unscanned automatic writes — the dashboard's activity log is incomplete and L3 content can reach rule proposals and Dreaming

- Priority: P2
- Confidence: high
- Layer: adapter (framework root cause, EKO surface exposure)
- Evidence: framework `echo-agent/src/memory_promoter.rs:70-94` (direct
  `put_typed`, no ChangeLog/guard) invoked from
  `echo-agent/echo-state/src/compression/mod.rs:836-838`; EKO enables the
  compressor on the primary agent and pool (`agent_pool.rs:947-949`); EKO
  surfaces consuming the same warm store: `Dashboard::get_recent_activities`
  (dashboard.rs:170-185, change-log derived), `RulePromoter::scan_for_proposals`
  (rule_promoter.rs:95-160, WARM_NAMESPACE scan), Dreaming
  (`infra.rs:1203-1212`).
- Reachability: every compression pass in a live EKO session (all surfaces
  with the compressor configured) writes to the store; the dashboard is
  reachable via `/evolution-dashboard` (evolution.rs:1526) and
  `get_evolution_dashboard` (panels.rs:1462); rule proposals via
  `/rule-promote scan` and `scan_rule_proposals` (panels.rs:1494).
- Expected invariant: `echo-agent/src/evolution/mod.rs:20-22` — "All
  mutations to memories, skills, and rules are recorded in the audit log";
  the dashboard's "recent activities" is presented as the evolution record.
- Observed behavior: L3-promoted facts (the most frequent automatic memory
  writes) are written unrecorded and unscanned; the dashboard's recent
  activities omit them; evicted-message content (potentially containing
  secret-like text from assistant output or tool digests) reaches the store
  unredacted and can then be (a) proposed as a learned-rules.md rule by
  `RulePromoter` (the promote-time security guard still re-scans at
  promotion, so the rule write itself stays guarded) and (b) promoted to the
  hot layer by Dreaming.
- Impact: EKO's evolution surfaces present an incomplete mutation record and
  inherit the security-guard gap on the highest-frequency write path; users
  reviewing the dashboard cannot see automatic promotions, and rollback
  (change-log based) cannot target L3 writes. Local-only content, no network
  exposure — P2, matching the framework finding.
- Root cause: framework-side (F-EVO-01-P2-02 — `StoreMemoryPromoter` wired
  directly to the raw Store instead of `MemoryLayerManager::write_memory`);
  EKO has no compensating surface-side guard.
- Direction: fix ownership stays with F-EVO-01-P2-02 (route promoter writes
  through `MemoryLayerManager::write_memory` or add change-log + security
  scan inside the promoter). EKO-side: no code change required once the
  framework path is fixed; optionally surface a dashboard warning when the
  change log and store counts diverge.
- Regression validation: fixture — promote a batch containing a secret-like
  string via compression, assert (a) stored content is redacted or rejected
  and (b) a `ChangeEntry` exists for the promoted key (same fixture as
  F-EVO-01-P2-02); after the fix, dashboard recent-activities includes
  promoted entries.
- Validation reports: [V03-01](../validations/A-EVO-01/V03-01.md), [V05-01](../validations/A-EVO-01/V05-01.md)

### A-EVO-01-P2-02: Every EKO evolution surface that mutates the hot layer refreshes the instruction projection instead of the hot-memory projection — hot memory stays stale in live agent context until boot/switch; `refresh_hot_memory_projection` has zero production callers

- Priority: P2 (P1 in the canonical finding A-MEM-01-P1-01)
- Confidence: high
- Layer: application
- Evidence: wrong-projection refresh sites on the evolution surfaces:
  Dreaming `report.promoted > 0` (infra.rs:1175-1192), GUI `add_memory`
  (tauri/commands/memory.rs:126-145) and `delete_memory` (:221-239), TUI
  `/remember` (tui/events.rs:2838-2852) and `/forget` hot (:2913-2927), CLI
  `/remember` (cmd_impls/all.rs:120-141) and `/forget` hot (:195-205), pool
  `refresh_instruction_context` (agent_pool.rs:687-710); the correct helper
  `refresh_hot_memory_projection` (unified_memory.rs:154-167) has no
  production caller (only its test :261); two distinct projections
  `eko:instruction-context` / `eko:hot-memory-context` (unified_memory.rs:28-29).
- Reachability: every in-session hot-layer mutation on every surface — the
  most frequent memory paths (Dreaming promotion, memory panel add/delete,
  remember/forget, budget demotions, `remember` tool auto-promotion — the
  latter with no refresh at all).
- Expected invariant: `docs/2026-07-23-memory-self-evolution-closure.md:11,20`
  — after Dreaming or manual hot-layer change, current AND pooled agents
  refresh projections immediately.
- Observed behavior: `eko:hot-memory-context` keeps boot/switch content;
  newly promoted memories are absent from the model's Active Memories,
  deleted memories remain citable, budget-demoted entries stay visible.
- Impact: the agent reasons with stale hot-memory context on every surface;
  Dreaming's core output never reaches the live context. No file/store data
  loss (MEMORY.md and the warm store are correct) — hence not P0.
- Root cause: canonical in A-MEM-01-P1-01 (refresh call sites written against
  a single-projection plan after the implementation split hot memory into its
  own projection).
- Direction: canonical fix in A-MEM-01-P1-01 — replace the eight
  instruction-only refresh sites with `refresh_memory_projections`, make
  `refresh_instruction_context` refresh both, and wire
  `EvolutionObserver::on_memory_layer_change` so write-time auto-promotion
  and demotions also project; then delete `refresh_hot_memory_projection` or
  make it live.
- Regression validation: see A-MEM-01-P1-01 (hot projection content contains
  new entry after write_memory/Dreaming/delete fixtures).
- Validation reports: [V02-01](../validations/A-EVO-01/V02-01.md), [V03-01](../validations/A-EVO-01/V03-01.md)

### A-EVO-01-P2-03: TUI exposes a reduced evolution surface — rule promotion, curator, and skill-lifecycle commands are CLI/GUI-only

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: TUI `SlashCommand` enum (`src/tui/commands.rs:100-131`) has only
  AutoMemory/RunReview/EvidenceInbox/MemoryReview/SkillCandidates/
  EvolutionDashboard; the CLI evolution module registers 17 commands
  (evolution.rs:1831-1850) in the REPL only (repl.rs:172), including
  `/rule-promote`, `/curator`, `/skill-create`, `/skill-promote`,
  `/skill-merge`, `/skill-patch`, `/skill-health`, `/skill-register`,
  `/skill-pin`, `/skill-unpin`, `/profile`, `/review`; the GUI EvolutionPanel
  (EvolutionPanel.tsx) exposes rule adoption, skill draft/activation,
  curator run, evidence inbox. TUI has no rule-promotion, curator, or
  skill-lifecycle command and no `/reflect`.
- Reachability: TUI users cannot reach these evolution mutations at all;
  CLI/GUI users can.
- Expected invariant: AGENTS.md surface-parity rule — TUI and GUI are
  feature-equal complete Agents; the evolution surface is one of the shared
  capabilities.
- Observed behavior: TUI sessions can generate candidates (memory-review,
  evidence-inbox) but cannot promote rules, run the curator, or manage the
  skill lifecycle, while CLI and GUI can.
- Impact: TUI-only users silently lack the review-gate execution half of the
  evolution surface (they can review but not act); inconsistent product
  behavior across surfaces.
- Root cause: evolution commands were implemented in the CLI registry and
  Tauri commands; the TUI slash-command set was not extended to the same
  mutation surface.
- Direction: add TUI slash commands (or a shared command-registry path)
  for `/rule-promote`, `/curator`, and the skill-lifecycle commands,
  reusing the same app-core services; update the TUI help list; full parity
  matrix is X-SRF-01's job — this task hands over the gap.
- Regression validation: TUI fixture — slash-command inventory asserts
  rule-promote/curator/skill-lifecycle entries exist and dispatch to the
  same app-core functions as the CLI counterparts.
- Validation reports: [V01-01](../validations/A-EVO-01/V01-01.md), [V02-01](../validations/A-EVO-01/V02-01.md)

### A-EVO-01-P3-01: CLI `/critiques clear` is a no-op that reports "Critiques cleared."

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/cmd_impls/evolution.rs:277-279` —
  `"clear" => { println!("Critiques cleared."); }` — no storage, no
  operation; `/critiques` list only reads the run store.
- Reachability: every CLI `/critiques clear` invocation; TUI has no
  equivalent command.
- Expected invariant: a command reports success only when it performs the
  stated operation.
- Observed behavior: the message claims critiques were cleared; nothing is
  cleared (no critiques store exists in the current design).
- Impact: misleading output for users; harmless otherwise.
- Root cause: leftover branch from a removed critiques design.
- Direction: delete the `clear` branch (and update the usage string) or
  remove the command; keep `/review` + Evidence Inbox as the actual review
  surfaces.
- Regression validation: no test needed beyond removal; grep
  `CritiquesCommand` zero hits after deletion.
- Validation reports: [V03-01](../validations/A-EVO-01/V03-01.md)

### A-EVO-01-P3-02: `Runtime::reflect_on_session` / `checkpoint_reflection` have zero production callers — a dead duplicate of the live REPL reflection writer

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:413`
  (`reflect_on_session`), `:423` (`checkpoint_reflection`), `:476` (same
  PROJECT.md write target); repo-wide grep finds the definitions only (single
  occurrence each). The live writer is `src/cli/repl.rs:340`.
- Reachability: definition only; no runtime path constructs a call.
- Expected invariant: one authoritative writer per durable artifact; dead
  code is removed per AGENTS.md cleanup rules.
- Observed behavior: two implementations of the same "LLM reflection →
  `.eko/memory/PROJECT.md`" concept, one live (REPL exit) and one dead
  (app-core API) — duplicate authority for the reflection artifact, and the
  dead one would be invisible to users if ever called (no gate either).
- Impact: maintenance hazard — a future caller of the dead API would
  silently duplicate the P1-01 write path.
- Root cause: the reflection feature was prototyped in app-core and then
  reimplemented in the REPL without removing the first version.
- Direction: after the P1-01 decision, keep exactly one implementation:
  if reflection stays, move it into app-core (gated + audited) and have the
  REPL call it; otherwise delete `runtime.rs:411-490`.
- Regression validation: grep `checkpoint_reflection|reflect_on_session`
  zero (or exactly one wired) production occurrence after the fix.
- Validation reports: [V03-01](../validations/A-EVO-01/V03-01.md)

### A-EVO-01-P3-03: Documentation drift — the automatic reflection write is undocumented and contradicts the documented review-gate model; the "no record_execution calls" claim is fixed

- Priority: P3
- Confidence: high
- Layer: application (docs)
- Evidence: `docs/system-deep-dive/04-memory.md:407,428` document that EKO
  queues observations to the Review Inbox and has "没有自动 post-run hook";
  `docs/2026-07-15-self-evolution-review-and-roadmap.md:84` restricts
  automatic writes to low-ambiguity events; neither mentions the REPL exit
  reflection (repl.rs:256) or the `.eko/memory/PROJECT.md` tier shown by
  `/memory show` (cmd_impls/all.rs:250-255). `docs/system-deep-dive/
  07-cross-cutting.md:141-142` claimed `SkillExecutionRecord` had no runtime
  `record_execution` call — current code calls `record_skill_telemetry` per
  tool batch (`echo-agent/src/agent/react/run/execution.rs:320,337,351`).
- Reachability: documentation-only; `MASTER-PLAN.md:62` seam-closure row is
  current (V05-01).
- Expected invariant: product docs describe the implemented evolution model.
- Observed behavior: one automatic write path exists that the docs never
  describe and the documented auto-write policy forbids; one historical
  "no telemetry" claim is stale.
- Impact: maintainers and users have an incomplete model of what writes
  automatically; the roadmap/closure acceptance criteria ("无人确认时不会
  自动改写") appear satisfied while the reflection path violates them.
- Root cause: reflection-on-exit added after the policy docs were written;
  telemetry wiring completed without updating 07-cross-cutting.
- Direction: document (or remove, per P1-01) the reflection path in
  04-memory.md and the closure doc; update 07-cross-cutting.md:141-142 to
  current telemetry wiring.
- Regression validation: doc grep — reflection-on-exit mentioned wherever
  automatic writes are enumerated; `record_execution` claim matches code.
- Validation reports: [V05-01](../validations/A-EVO-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (evolution surface, both repos) | yes | passed | [V01-01](../validations/A-EVO-01/V01-01.md) |
| V02 | Registration and runtime reachability (reachable mutation trigger inventory) | yes | passed (P1-01 evidence) | [V02-01](../validations/A-EVO-01/V02-01.md) |
| V03 | Invariants/edges: user authorization boundaries, dead/aspirational classification, product docs vs code | yes | passed (violations → P1-01, P2-01, P2-02, P2-03, P3-01..03) | [V03-01](../validations/A-EVO-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib evolution::` | yes | passed, exit 0 | [V04-01](../validations/A-EVO-01/V04-01.md) |
| V04 | `cargo check -p echo-agent-app-core --all-features` | yes | passed, exit 0 | [V04-02](../validations/A-EVO-01/V04-02.md) |
| V05 | Historical-document drift (roadmap, closure doc, deep-dive 04/07, MASTER-PLAN) | yes | passed (regressed rows → P1-01/P2-01/P2-02; fixed row → P3-03) | [V05-01](../validations/A-EVO-01/V05-01.md) |

All required validations executed; every command has a known exit code; no
validation pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `2026-07-15-roadmap.md:5` — "证据候选 → 用户确认 → 可审计写入 → 使用反馈" | current for review surfaces; regressed on REPL reflection | evidence.rs/rule_promoter/panels review gates (V03); repl.rs:256 → P1-01 |
| `2026-07-15-roadmap.md:84` — automatic writes only for low-ambiguity explicit events | regressed | reflection-on-exit (P1-01); L3 promoter (P2-01) |
| `2026-07-15-roadmap.md:123` — acceptance: no automatic rewrite of facts/skills without confirmation | current with the two exceptions above | V03 authorization table |
| `2026-07-15-roadmap.md:97-109` — unified candidate protocol, fail-closed trigger inbox, three-surface review gate | current | evidence.rs:335-402/478-636; review_integration.rs:314-318; CLI/TUI/GUI inbox (V02) |
| `2026-07-23-closure.md:11,20` — Dreaming/hot mutation immediately refreshes current + pooled agents | regressed | infra.rs:1175-1192 refreshes instruction projection only → cross-ref A-MEM-01-P1-01 (P2-02) |
| `2026-07-23-closure.md:12` — semantic merge/rule/skill activation through Review Inbox/manual | current | V03 table |
| `2026-07-23-closure.md:21` — Dreaming 60 s after boot then daily; GUI/TUI/CLI same wiring | current (interactive); channels gap | infra.rs:1156/1158; spawn sites desktop.rs:247, tui/mod.rs:1999, repl.rs:106; none in channels.rs → A-MEM-01-P3-01 |
| `2026-07-23-closure.md:45` — staleness suggestions analysis-only | current | evidence.rs stale-fail-before-mutation; Dreaming deterministic archive only |
| `04-memory.md:407,412,428` — auto_memory/BackgroundReviewer → Review Inbox; no automatic post-run hook | current for those; reflection is an undocumented third writer | auto_memory/mod.rs:17-24; background_review.rs:104; repl.rs:256 → P1-01/P3-03 |
| `07-cross-cutting.md:141-142` — no `record_execution` in runtime | fixed | execution.rs:320/337/351 |
| `MASTER-PLAN.md:62` — memory and self-evolution seam closure Complete | current | V01/V02 (workspace curator, shared ReviewIntegration, layered writes) |

## Coverage And Uncertainty

- No live session was launched; no LLM-backed path executed (BackgroundReviewer
  review runs, reflection, LlmGrader) — all behavioral claims for those are
  static traces (V02/V03). The P1-01 write behavior is argued from the code
  chain (unconditional append after LLM success).
- The TUI/CLI binary targets were inspected statically only; V04-02 compiles
  app-core, not the `src/cli`/`src/tui`/`src/tauri` bins (Q-CLI-01/Q-GUI-01
  cover those).
- Channels-mode behavior (no Dreaming, pool trigger sink) verified by grep
  only; no IM session was run (A-MEM-01-P3-01 cross-reference).
- Curator transitions are persisted in curator-state.json but produce no
  change-log entry (only a hook event) — noted as a minor audit
  completeness gap, not raised as a separate finding.
- Evidence-candidate quotes are limited (1000 chars, 16 items) but are not
  security-scanned at inbox write time; content is local conversation text,
  consistent with the local threat model.
- `AUTO_MEMORY_ENABLED` is a process-global static shared across surfaces
  (all.rs:462) — toggling in one mode affects all modes of the process;
  session-scoped by design, minor.
- Cross-task boundary: P2-01 and P2-02 duplicate upstream findings
  (F-EVO-01-P2-02, A-MEM-01-P1-01) by design — this task records the
  EKO-evolution-surface exposure with backlinks; the synthesizer should keep
  the upstream IDs canonical.
- No "framework option deletion" evidence found: EKO consumes framework
  evolution APIs (Dreaming, Curator, MemoryLayerManager, reviewers) and
  leaves eval/improve/evolution feature options intact (F-EVO-01); EKO's own
  eval/trajectory product surfaces were removed as documented product policy
  (2026-07-15-roadmap.md:13), not framework options.

## Handoff

- Downstream tasks may rely on: (1) EKO evolution = explicit diagnostics +
  review gate, single authority per concept (V01); (2) reachable mutation
  trigger inventory with authorization classification (V02/V03) — automatic:
  L3 promoter, Dreaming, per-turn trigger capture, REPL-exit auto-memory
  queue, REPL-exit reflection; user-gated: remember/forget (all surfaces),
  evidence accept/undo, rule promote, curator run, skill lifecycle, review
  runs; (3) one live violation of the no-automatic-semantic-write invariant:
  CLI REPL reflection-on-exit (P1-01); (4) two cross-cutting exposures:
  unaudited L3 writes flowing into EKO surfaces (P2-01 → F-EVO-01-P2-02) and
  hot-memory projection staleness on evolution surfaces (P2-02 →
  A-MEM-01-P1-01); (5) TUI evolution surface parity gap (P2-03); (6)
  Review Inbox accept/undo/stale-merge behavior is test-covered and green
  (V04-01).
- Reports to read: this report + V01-01..V05-01; F-EVO-01 (P2-02 canonical,
  framework fix ownership), A-MEM-01 (P1-01 canonical), A-CFG-01 (P1-02 CWD
  staleness interacting with P1-01/P3-03).
- Conditions that make this report stale: any change to
  `src/cli/repl.rs` exit sequence; `runtime.rs` reflection API;
  `unified_memory.rs` projections/refresh; `infra.rs` Dreaming task or its
  refresh; `evolution/` (evidence, rule_promoter, review_integration,
  dashboard); TUI/CLI/GUI evolution command registration; framework
  `memory_promoter.rs` / compression promoter wiring; the policy docs cited
  in V05-01.
- Follow-up task IDs (fixes are not implemented in this review):
  X-SRF-01 (TUI evolution-surface parity row, P2-03), X-MEM-01 /
  X-BND-01 (P2-01, P2-02 canonical merge), Q-CLI-01/Q-GUI-01 (binary
  targets), Q-DOC-01 (P3-03 wording), S-APP-01/S-X-01 (synthesis).
