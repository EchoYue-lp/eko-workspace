# X-MEM-01: Instruction/memory layers over generic context and compression

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

Can EKO-specific instruction/memory layers use generic context and compression
without duplicate persistence or lost updates?

## Scope

Primary source paths and behaviors inspected (cross-cutting framework +
application):

- `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs` (full) —
  five-tier file protocol + `MEMORY.md` hot layer, `get_instruction_suffix`
  (excludes hot memory) / `get_memory_suffix` (hot memory only) /
  `get_system_prompt_suffix` (both).
- `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs` (full) —
  `UnifiedMemory` wrapper, the two projection markers
  (`eko:instruction-context`, `eko:hot-memory-context`), the three refresh
  helpers, two projection-independence tests.
- `echo-agent/echo-state/src/compression/mod.rs:30-62, 540-793, 1295-1450` —
  projection envelope (`PROJECTION_ENVELOPE_PREFIX`,
  `is_context_projection_message`, `projection_envelope_text`),
  `replace_projection` / `apply_projections`, `is_protected` /
  `split_protected` / `merge_protected`, the compression `prepare` loop.
- `echo-agent/src/agent/react/run/context.rs:490-555` — per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall injection (tail projection).
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:444, 520-531, 1132-1212` —
  bootstrap `refresh_dynamic_context`, `spawn_dreaming_task` refresh wiring.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:844-1032` —
  `switch_workspace` store + projection rebind.
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:585-710` —
  `apply_memory_store` / `apply_memory_store_inner` / pool refresh helper.
- `echo-agent-cli/src/tauri/commands/memory.rs:105-157, 190-245` — GUI
  add/delete memory refresh sites.
- `echo-agent-cli/src/cli/cmd_impls/all.rs:105-215` — CLI /remember//forget
  refresh sites.
- All `refresh_*_projection` / `refresh_instruction_context` call sites
  enumerated via repository-wide grep (see V01).

## Out Of Scope

Deferred to named task IDs (all complete, read as dependencies):

- **F-CTX-01** — token budget arithmetic, phantom reservations, protected-token
  deduction. X-MEM-01 consumes F-CTX-01's conclusion that protected content
  survives compression but its token cost is observability-only.
- **F-MEM-01** — `Store` / `ConversationStore` trait durability, `FileStore` /
  `FileConversationStore` path safety. X-MEM-01 treats these as stable inputs.
- **F-CMP-01** — compressor matrix, summary accumulation, sanitize correctness.
  X-MEM-01 consumes F-CMP-01's conclusion that projections never enter
  compressor input and re-verifies only the EKO-projection-specific interaction.
- **A-MEM-01** — application memory-policy refresh wiring. X-MEM-01
  cross-references A-MEM-01-P1-01 (the wrong-projection defect) and re-verifies
  it at the X-MEM-01 commits; it does not re-derive the full refresh table.
- Framework `MemoryLayerManager` hot/warm/cold scoring and `Dreaming` recall
  logic — framework evolution scope, taken as given.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder — especially the framework-vs-application layering gate,
  the "first check if it already exists" rule, the no-duplicate-persistence
  intent, and the local-personal-assistant threat model).
- `docs/comprehensive-review/REPORTING.md` and
  `docs/comprehensive-review/templates/{task-report,validation-report}.md`.

Dependency task reports read (in full):

- `zcode-glm/tasks/F-CTX-01.md` — projection-survival via markers + envelope +
  canonical re-injection + tool-pair sanitise (V02); protected-token is
  observability-only (F-CTX-01-P2-02).
- `zcode-glm/tasks/F-MEM-01.md` — single `Store`/`ConversationStore` authority
  per concept (V01); projection round-trip losslessness (V04).
- `zcode-glm/tasks/F-CMP-01.md` — projections never enter compressor input;
  summary accumulation (F-CMP-01-P2-01) affects `[对话历史摘要]` system
  messages, not protected projections.
- `zcode-glm/tasks/A-MEM-01.md` — EKO owns only its file protocol; hot-layer
  edits refresh the wrong projection (P1-01); pool helper doc mismatch
  (P2-01); CLI no pool fan-out (P2-02); workspace switch is the ground-truth
  refresh path.

Historical documents treated as hypotheses:

- `unified_memory.rs:1-22` module doc claims the loader is "static, file-only"
  with "dynamic memories handled separately". Verified current.
- `unified_memory.rs:137-148` docstrings claim instruction and hot-memory are
  "two independently replaceable projections". Verified current at framework
  layer; the application refresh wiring does not exercise hot-memory
  independence (re-affirmed A-MEM-01-P1-01).
- `infra.rs:1140-1142` (`spawn_dreaming_task` doc) claims "when a pass changes
  the hot layer, the primary and pooled agents refresh their replaceable
  instruction projection immediately". Verified **regressed**: code refreshes
  `eko:instruction-context`, but Dreaming writes `MEMORY.md` which is projected
  by `eko:hot-memory-context`. The comment names the wrong projection
  (re-affirmed A-MEM-01-P1-01).

## Layering Decision

This task **spans both repositories** and verifies that the application rides
correctly on the framework's generic context/compression machinery.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework owns projection survival: the `<echo-agent-context-projection-v1>` envelope, `is_context_projection_message` / `is_protected` / `split_protected` / `merge_protected`, `replace_projection` / `apply_projections`, and the per-turn `TURN_MEMORY_CONTEXT_PROJECTION` tail-projection recall. Any `echo-agent` consumer may project stable-prefix or per-turn context through these primitives. Correctly in `echo_state::compression`. |
| EKO product policy | EKO owns its **file protocol** (`InstructionProvider`: `~/.eko/user.md`, the `AGENTS[.override].md` chain, `<root>/.eko/{project,local,learned-rules}.md`, `<root>/.eko/MEMORY.md`), the two projection markers (`eko:instruction-context`, `eko:hot-memory-context`), the `UnifiedMemory` wrapper, the `refresh_*_projection` helpers, and the refresh wiring in Dreaming / TUI / GUI / CLI. Correctly in `echo-agent-app-core`. |
| Adapter boundary | The three refresh helpers (`refresh_instruction_projection`, `refresh_hot_memory_projection`, `refresh_memory_projections`) are thin: they read files via `InstructionProvider::load_for`, wrap the body in `Message::system`, and call the framework's `context.replace_projection(marker, message)`. No scheduling authority, no state ownership, no semantic loss. The framework's `replace_projection` is the seam being adapted to. |
| Duplicate search | Searched terms across both repos: `InstructionProvider`, `UnifiedMemory`, `refresh_instruction_projection`, `refresh_hot_memory_projection`, `refresh_memory_projections`, `refresh_instruction_context`, `eko:instruction-context`, `eko:hot-memory-context`, `get_instruction_suffix`, `get_memory_suffix`, `load_hot_memory`, `agents_instructions_path`, `PROJECTION_ENVELOPE_PREFIX`, `replace_projection`, `apply_projections`, `replace_tail_projection`, `TURN_MEMORY_CONTEXT_PROJECTION`. Result: one canonical definition per concept. `UnifiedMemory` wraps `InstructionProvider` (no parallel loader). The framework's two recall/runtime projections (`TURN_MEMORY_CONTEXT_PROJECTION`, `WORKSPACE_CONTEXT_PROJECTION`) are distinct per-turn/runtime projections, not duplicates of the two EKO stable-prefix projections. |
| Migration deletion | No deletion recommended. The two-layer model (hot `MEMORY.md` markdown + warm `store.json` typed KV) is intentional, not duplication. The framework's projection envelope is a single primitive consumed by EKO's two markers. |

**Synthesis:** EKO owns only its instruction/memory file protocol. The
framework owns projection survival and the dynamic recall machinery. No
duplicate authority exists on either side. The defects surfaced below are in
the **application refresh wiring** (wrong projection target on hot-layer
edits), not in ownership, layering, or the framework's compression path.

## Current Path

Verified instruction/memory flow at commits `9b0e0fa` / `b3b2e81`:

```text
                          EKO file protocol (application)
                          ─────────────────────────────────
  ~/.eko/user.md                  ┐
  <root>/AGENTS[.override].md      │   InstructionProvider::load_for
  <root>/.eko/project.md           ├─►  (instruction_provider.rs:61)
  <root>/.eko/learned-rules.md     │        │
  <cwd>/.eko/local.md             ┘        │
       │                                    │
       │   <root>/.eko/MEMORY.md ───────────┤  (hot layer, loaded separately)
       │                                    │
       ▼                                    ▼
  get_instruction_suffix()        get_memory_suffix()
  (5 tiers, EXCLUDES hot)         (MEMORY.md body only)
  instruction_provider.rs:141-163  instruction_provider.rs:170-174
       │                                    │
       ▼                                    ▼
  refresh_instruction_projection  refresh_hot_memory_projection
  → eko:instruction-context       → eko:hot-memory-context
  unified_memory.rs:138-151       unified_memory.rs:154-167
       │                                    │
       └──────────────┬─────────────────────┘
                      ▼
           refresh_memory_projections (BOTH)
           unified_memory.rs:170-186
                      │
                      ▼  (single framework seam)
           ContextManager::replace_projection(marker, message)
           compression/mod.rs:605-611
                      │
                      ▼  wraps in <echo-agent-context-projection-v1>
                      │  envelope at system/history boundary
                      │  (mod.rs:725-741, 567-572)
                      │
          ┌───────────┴─────────────┐
          ▼                         ▼
   is_context_projection_message  → true
   is_protected                   → true (mod.rs:678-691)
          │
          ▼  on every ContextManager::prepare()
   split_protected removes projections from compressible set (mod.rs:747-773)
   compressor runs on compressible ONLY (mod.rs:1336-1345)
   merge_protected re-inserts projections after (mod.rs:1353, 781-793)
   ⇒ projections NEVER enter compressor input
   ⇒ immune to F-CMP-01-P2-01 summary accumulation
```

**Bootstrap** (`infra.rs:440-444`): `create_agent` builds the ReactAgent then
calls `refresh_dynamic_context(&mut agent, root)` which calls
`refresh_memory_projections` (BOTH projections). So at agent creation time,
both projections are installed correctly via the single framework seam.

**Per-turn recall** (`run/context.rs:514-532`): a separate tail projection
`TURN_MEMORY_CONTEXT_PROJECTION` is refreshed every turn from the warm store
(`recall_long_term_memories` → `format_memory_context` →
`replace_tail_projection`). This is a DISTINCT projection from the two
stable-prefix projections — it sits at the tail (not the system/history
boundary) to preserve the longest stable prefix for provider KV caches. It is
not a duplicate of `eko:hot-memory-context`.

**Workspace switch** (`state.rs:844-1032`): the single fully-consistent rebind
path — sets CWD, calls `refresh_dynamic_context` on primary (both projections),
`apply_working_dir` on pool, rebuilds conversation + runtime-state stores,
`ReviewIntegration::rebind` then `create_layer_manager`, `install_memory_store`
+ `install_memory_layer_manager` on primary, `apply_memory_store` on pool (with
`memory_store_override` for future pool agents).

The graph exposes one defect class (see Findings): the application refresh
wiring routes MEMORY.md-mutating edits to the wrong projection marker,
defeating the "lost updates" half of the task question. The "duplicate
persistence" half is clean.

## Findings

### X-MEM-01-P1-01: Hot-layer (MEMORY.md) edits refresh the wrong projection — promoted/deleted hot memories are a lost update until workspace switch

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:28-29, 138-167`
    — the two markers are distinct; `refresh_instruction_projection` targets
    `eko:instruction-context` only (reads `instruction_prompt_suffix()` which
    excludes hot memory); `refresh_hot_memory_projection` targets
    `eko:hot-memory-context` only (reads `memory_prompt_suffix()`).
  - `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:141-163`
    — `get_instruction_suffix` deliberately excludes `hot_memory`; only
    `get_memory_suffix` (`:170-174`) emits the `## Active Memories (Hot
    Layer)` block.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:1175-1192`
    (`spawn_dreaming_task`) — on `report.promoted > 0`, calls
    `refresh_instruction_projection` (primary) +
    `pool.refresh_instruction_context()`. Dreaming promotes via
    `consider_promotion → promote_warm_to_hot`, which writes `MEMORY.md`.
  - `echo-agent-cli/src/tauri/commands/memory.rs:126-145` (add_memory) and
    `:219-238` (delete_memory on `MemoryLayer::Hot`) — same wrong-target
    pattern.
  - `echo-agent-cli/src/tui/events.rs:2839-2857` (`/remember`) and
    `:2913-2927` (`/forget` Hot) — same pattern.
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:123-138` (`/remember`) and
    `:194-209` (`/forget` Hot) — same pattern.
  - Repository-wide grep (V02): `refresh_hot_memory_projection` has **zero**
    production callers; its only caller outside its own definition is the unit
    test at `unified_memory.rs:261`. The only production path that refreshes
    `eko:hot-memory-context` is `refresh_memory_projections` ←
    `refresh_dynamic_context` (`infra.rs:528-529`) ← workspace switch
    (`state.rs:883`) and bootstrap (`infra.rs:444`).
- Reachability: any `write_memory` that promotes to hot, any `delete_memory`
  on a hot entry, or any Dreaming pass with `promoted > 0`. All eight call
  sites re-read the file via `InstructionProvider::load_for` but push the
  result into the **instruction** projection. The hot-memory projection
  retains its previous value until the next workspace switch or process
  restart.
- Expected invariant (the task's "without lost updates" requirement): when the
  memory protocol changes, the agent observes the change immediately. A memory
  promoted to `MEMORY.md` should appear in the agent's `## Active Memories
  (Hot Layer)` stable prefix on the next turn.
- Observed behavior: the `## Active Memories (Hot Layer)` segment of the
  agent's context is frozen at agent-creation time. Promoted memories do not
  appear there; deleted hot memories do not disappear from there. The
  `eko:instruction-context` projection is needlessly re-written with identical
  content (instructions do not change when only `MEMORY.md` changes), masking
  the omission. This is a **lost update**: the write to `MEMORY.md` is durable
  on disk but never reaches the agent's stable prompt prefix.
- Impact: the headline capability of Dreaming (recall-driven promotion to a
  stable prompt prefix) and of `/remember` auto-promotion is silently broken
  on the primary surface it was built for — the stable prefix. Promoted
  memories may still surface via the framework's per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall (`context.rs:514-532`) when they
  match the current query, but that is the recall path (query-dependent,
  transient), not the hot-layer stable-prefix injection the feature
  advertises. `spawn_dreaming_task`'s own doc comment (`infra.rs:1140-1142`)
  promises "agents refresh their … projection immediately" — false for the
  hot-memory projection.
- Root cause: the refresh helpers were written when the two projections were
  separated, but the MEMORY.md-mutating call sites were never updated to pick
  the right one. Every MEMORY.md-mutating site copied the same
  `refresh_instruction_projection` + `pool.refresh_instruction_context()`
  snippet used by the learned-rules.md sites, without noticing that
  `MEMORY.md` is excluded from `get_instruction_suffix`.
- Direction: at every MEMORY.md-mutating site, call
  `refresh_hot_memory_projection` (primary) and add a pool-level
  `refresh_hot_memory_context` that mirrors `refresh_instruction_context` but
  calls `refresh_hot_memory_projection`. The simplest mechanical fix is to
  replace the eight wrong-target call sites with `refresh_memory_projections`
  (which refreshes both, idempotently) and widen the pool helper's doc + body
  to match. The learned-rules.md sites (`panels.rs:1561`, `evolution.rs:1489`)
  are already correct and should stay on `refresh_instruction_projection`.
- Regression validation: seed a workspace, call
  `layer_manager.write_memory` with a high-confidence entry that triggers
  promotion, then assert the agent's `eko:hot-memory-context` projection
  contains the new key (existing test
  `instruction_and_hot_memory_use_distinct_projections` at
  `unified_memory.rs:248-267` covers projection independence but not the
  promotion-refresh path; add a promotion-refresh test).
- Validation reports: [V02-01](../validations/X-MEM-01/V02-01.md),
  [V01-01](../validations/X-MEM-01/V01-01.md). This finding re-affirms
  **A-MEM-01-P1-01** at the X-MEM-01 commits; no new defect is claimed beyond
  it.

### X-MEM-01-P2-01: `AgentPool::refresh_instruction_context` doc claims hot-memory refresh but only refreshes instructions

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:686-710` —
  doc comment "Refresh hot-memory and instruction projections on every
  existing agent"; body calls only `refresh_instruction_projection`
  (`:701`).
- Reachability: every pool refresh triggered by Dreaming (`infra.rs:1191`),
  GUI memory commands (`memory.rs:143,236`, `panels.rs:1570`), and TUI memory
  commands (`events.rs:2855,2926`).
- Expected invariant: a pool helper whose doc claims both projections should
  refresh both, or its doc should name only instructions.
- Observed behavior: the helper refreshes only instructions. This is the
  pool-side half of P1-01: even if a caller wanted to refresh hot memory on
  the pool, the helper does not expose that capability.
- Impact: misleading API; the pool has no path to refresh hot memory short of
  `apply_working_dir` (workspace switch). Compounds P1-01 — there is currently
  no single call that refreshes hot memory across the pool.
- Root cause: the helper predates the instruction/hot-memory split (or was
  never updated when `refresh_hot_memory_projection` was added at
  `unified_memory.rs:154`).
- Direction: either (a) widen the helper to refresh both (rename to
  `refresh_memory_context`, call `refresh_memory_projections` per agent), or
  (b) keep it instruction-only and fix the doc. (a) is preferred because it is
  the minimal building block P1-01's fix needs.
- Regression validation: covered by the P1-01 promotion-refresh test extended
  to assert pooled agents also reflect the promoted key.
- Validation reports: [V02-01](../validations/X-MEM-01/V02-01.md). Re-affirms
  **A-MEM-01-P2-01**.

### X-MEM-01-P2-02: CLI /remember, /forget, and rule-promote refresh only the primary agent — pooled/background agents diverge

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:123-142` (`/remember`) —
    refreshes `ctx.agent` only; no `pool.refresh_*` call.
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:194-214` (`/forget` Hot) — same.
  - `echo-agent-cli/src/cli/cmd_impls/evolution.rs:1471-1496` (rule promote)
    — same.
  - Contrast: `tauri/commands/memory.rs:142-144`, `panels.rs:1569-1571`, and
    `tui/events.rs:2854-2856,2925-2927` all follow the primary-agent refresh
    with `pool.refresh_instruction_context()`.
- Reachability: any CLI-driven memory or rule promotion/deletion while a
  background/pool agent exists.
- Expected invariant: the refresh fan-out should be the same regardless of
  which surface (CLI/TUI/GUI) initiated the change.
- Observed behavior: CLI paths mutate the file and refresh the primary agent,
  but pooled agents (including the always-on `__background__` agent) keep the
  stale projection until the next workspace switch or restart.
- Impact: a user who runs `/remember <fact>` in the CLI and then dispatches a
  background task sees the background agent operate against the
  pre-`/remember` protocol. Inconsistent with the GUI/TUI behavior.
- Root cause: the CLI snippets predate the pool-fan-out convention and were
  not updated when GUI/TUI gained it.
- Direction: after each CLI refresh, mirror the GUI/TUI pattern with
  `if let Some(pool) = &pool { pool.refresh_instruction_context().await; }`
  (and, once P2-01 is fixed, the corresponding hot-memory call). Better:
  extract a single `refresh_memory_after_edit(primary, pool, root)` helper in
  `unified_memory.rs` and call it from all eight sites so the fan-out cannot
  drift again.
- Regression validation: seed a pool with a background agent, run CLI
  `/remember`, assert the background agent's instruction projection reflects
  the new rule within the same command.
- Validation reports: [V02-01](../validations/X-MEM-01/V02-01.md). Re-affirms
  **A-MEM-01-P2-02**.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Source/precedence map: instruction/memory flow EKO→framework, single path or duplicates | yes | **passed** (single path, no duplicate persistence; framework per-turn recall is a distinct tail projection) | [V01-01](../validations/X-MEM-01/V01-01.md) |
| V02 | Immediate refresh: memory changes reflected in agent context or stale | yes | **failed** (hot-layer edits refresh wrong projection → P1-01; pool helper mismatch → P2-01; CLI no pool fan-out → P2-02) | [V02-01](../validations/X-MEM-01/V02-01.md) |
| V03 | Repeated compression: EKO memory survives repeated compression cycles (per F-CMP-01 summary accumulation) | yes | **passed** (projections never enter compressor input; immune to F-CMP-01-P2-01; 69 echo_state compression tests green) | [V03-01](../validations/X-MEM-01/V03-01.md) |
| V04 | Workspace switch: memory store replacement, duplicate promotion | yes | **passed** (clean store + projection rebind; `memory_store_override` for future pool agents; no duplicate-store defect) | [V04-01](../validations/X-MEM-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three module docs make auditable claims) | done — see Historical Claim Status table below | — |

Targeted executable checks run as part of V03-01:

| Command | Exit | Result |
|---|---:|---|
| `cargo test -p echo_state --lib compression:: --locked` | 0 | 69 passed, 0 failed |

No conditional feature/GUI/frontend matrix was run: this task touches framework
compression code (already covered by F-CMP-01's run) and application refresh
wiring (static analysis of call sites). No new feature flag or public API
change is in scope.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `instruction_provider.rs:14-23` "Static, file-only loader: no DB, no embeddings, no recall … dynamic memories handled separately" | current | V01 confirms; the no-watcher consequence is documented in A-MEM-01-P3-01 (not re-litigated here). |
| `unified_memory.rs:137-148` "instruction and hot-memory projections are independently replaceable" | current at framework layer, regressed at application layer | The framework supports independent replacement (`replace_projection` per marker); no application MEMORY.md-edit call site exercises hot-memory replacement outside workspace switch / bootstrap (P1-01). |
| `infra.rs:1140-1142` (`spawn_dreaming_task`) "When a pass changes the hot layer, the primary and pooled agents refresh their replaceable instruction projection immediately" | regressed | Code refreshes `eko:instruction-context`, but Dreaming writes `MEMORY.md` which is projected by `eko:hot-memory-context`. Comment names the wrong projection (P1-01). |
| `agent_pool.rs:686` "Refresh hot-memory and instruction projections on every existing agent" | stale | Body refreshes only instructions (P2-01). |
| F-CTX-01 handoff: "Protected content survives compression" | current | V03 re-confirms for both EKO projections via `is_context_projection_message` → `is_protected`. |
| F-CMP-01 handoff: "Projections never enter compressor input; summary accumulation affects `[对话历史摘要]` system messages, not protected projections" | current | V03 re-confirms: `split_protected` (`mod.rs:1336`) removes projections before compress; `merge_protected` (`:1353`) re-inserts after. |
| F-MEM-01 handoff: "Store/FileStore identities are single authority; projection round-trip lossless" | current | V01/V04 confirm no parallel store authority and no duplicate persistence in the application layer. |
| A-MEM-01-P1-01 / P2-01 / P2-02: hot-layer refresh wrong projection; pool helper mismatch; CLI no pool fan-out | current (re-affirmed) | V02 re-confirms all three at the X-MEM-01 commits (`9b0e0fa` / `b3b2e81`); `refresh_hot_memory_projection` still has zero production callers. |

## Coverage And Uncertainty

- **All four validations are grounded.** V01, V02, V04 are static call-graph
  analyses (grep + targeted reads). V03 adds an executable test run
  (`cargo test -p echo_state --lib compression::`) confirming 69/69 pass at
  `9b0e0fa`.
- **P1-01 not exercised at runtime.** The defect is verified by unambiguous
  code trace: eight call sites refresh the wrong projection marker;
  `refresh_hot_memory_projection` has no production caller outside workspace
  switch. A runtime test that promotes a memory and asserts the agent's
  `eko:hot-memory-context` envelope changes would raise P1-01 from
  high-confidence to confirmed; the existing test
  `instruction_and_hot_memory_use_distinct_projections`
  (`unified_memory.rs:248-267`) proves the projections are independent but does
  not exercise the promotion-refresh path.
- **Per-turn recall mitigation.** P1-01's impact is softened by the
  framework's `TURN_MEMORY_CONTEXT_PROJECTION` recall
  (`context.rs:514-532`), which dynamically surfaces warm-store memories that
  match the current query each turn. So promoted memories are not wholly
  invisible — they are invisible *as the stable hot-layer prefix*, which is
  the specific surface Dreaming and `/remember` auto-promotion target.
- **Framework layer machinery not re-audited.** `MemoryLayerManager`'s
  hot/warm promotion scoring, `Dreaming`'s recall-frequency logic, and
  `EvidenceStore`'s inbox semantics are taken as given (framework scope).
- **`replace_tail_projection` vs `replace_projection`.** The framework offers
  both a system-boundary replace (`replace_projection`, used by EKO's two
  stable prefixes) and a tail append (`replace_tail_projection`, used for
  per-turn recall). EKO correctly uses the boundary variant for its stable
  prefixes and the framework correctly uses the tail variant for recall; this
  was verified but not promoted to a finding because it is correct.
- **Compression token-accounting gap (F-CTX-01-P2-02) is not a lost update.**
  The protected-token cost is not deducted from `effective_limit`, which can
  cause provider 400s (out of window), but it does not lose or duplicate the
  projection content. Out of scope for the "duplicate persistence or lost
  updates" question; tracked by F-CTX-01.

## Handoff

Conclusions downstream tasks may rely on:

1. **No duplicate persistence.** EKO rides on the framework's single
   `replace_projection` seam via two markers (`eko:instruction-context`,
   `eko:hot-memory-context`). The hot/warm file split (`MEMORY.md` markdown
   + `store.json` typed KV) is intentional and single-authority per layer.
   Downstream tasks reasoning about "who owns the prompt suffix" can rely on
   `InstructionProvider` + `UnifiedMemory` being the single application
   authority and `ContextManager::replace_projection` being the single
   framework seam.
2. **Projections survive repeated compression.** Both EKO markers carry the
   `<echo-agent-context-projection-v1>` envelope, are protected by
   `is_context_projection_message` → `is_protected`, and never enter
   compressor input. They are immune to F-CMP-01-P2-01 (summary accumulation).
   Downstream tasks that model long-session behavior can treat the protocol
   suffix as stable across compression.
3. **Lost updates exist on the hot-layer refresh path (P1-01).** Any
   downstream task that promises "memory is immediately visible after
   promotion" must consume P1-01 as a prerequisite. The per-turn recall path
   partially compensates. The two-layer split is correct; the refresh wiring
   is the defect.
4. **Workspace switch is the single consistent refresh + rebind path.** It
   refreshes both projections on primary and pool, rebinds the store, and
   rebuilds the layer manager. Downstream tasks should model switch as the
   ground-truth refresh and treat the other surfaces as best-effort (and, for
   MEMORY.md edits, currently broken).
5. **CLI surfaces do not fan out refresh to the pool (P2-02).** Downstream
   consistency tests should not assume CLI-driven changes reach background
   agents.

Reports downstream tasks must read:

- This report's V01 (source/precedence map) and V02 (refresh-wiring table) for
  any task touching the instruction/memory refresh paths.
- `zcode-glm/tasks/A-MEM-01.md` for the full refresh-wiring table and the
  original derivation of P1-01 / P2-01 / P2-02.
- `zcode-glm/tasks/F-CMP-01.md` for projection-survival and summary-
  accumulation context (cross-referenced in V03).
- `zcode-glm/tasks/F-CTX-01.md` for the budget/protected-token accounting gap
  that compounds window-overruns but is not a lost update.

Conditions that make this report stale:

- Any change to the eight MEMORY.md-mutating call sites that switches them to
  `refresh_hot_memory_projection` or `refresh_memory_projections` (resolves
  P1-01; re-run V02-01).
- Widening `AgentPool::refresh_instruction_context` to refresh both
  projections (resolves P2-01; re-run V02-01).
- Adding pool fan-out to the three CLI sites (resolves P2-02; re-run V02-01).
- Any change to `split_protected` / `merge_protected` / `is_protected` that
  alters projection survival (re-run V03-01).
- Any change to `switch_workspace` store/projection rebind ordering (re-run
  V04-01).

Follow-up task IDs (no fixes implemented in this review):

- A dedicated memory-refresh-coherence task should land P1-01 + P2-01 + P2-02
  together: they share a single fix shape (route every edit through one
  `refresh_memory_after_edit(primary, pool, root)` helper that calls
  `refresh_memory_projections` on the primary and a widened pool helper).
  P1-01 is the highest-impact and the smallest semantic change.
- This task does not modify any code; it only cross-validates the framework
  and application layers and re-affirms the A-MEM-01 findings at the X-MEM-01
  commits.
