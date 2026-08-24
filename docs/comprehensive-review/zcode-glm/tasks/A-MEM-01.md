# A-MEM-01: Instructions, hot memory, and Dreaming

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does EKO own only its instruction/memory file protocol while projecting
updates to that protocol immediately and consistently to the primary and
pooled Agents?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs` (full,
  438 lines) — `InstructionProvider`, five-tier file protocol
  (user/repository/project/learned-rules/local) plus `MEMORY.md` hot layer,
  legacy `AGENTS.md → learned-rules.md` migration, `get_instruction_suffix`
  / `get_memory_suffix` / `get_system_prompt_suffix` assembly.
- `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs` (full, 268
  lines) — `UnifiedMemory` wrapper, the four refresh helpers
  (`refresh_instruction_projection`, `refresh_hot_memory_projection`,
  `refresh_memory_projections`), and the two projection markers
  (`eko:instruction-context`, `eko:hot-memory-context`).
- `echo-agent-cli/echo-agent-app-core/src/auto_memory/mod.rs` (full,
  72 lines) — auto-memory observation extraction routed into the
  `EvidenceStore` inbox.
- `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs`
  (full, 378 lines) — high-confidence memory → `learned-rules.md`
  promotion.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:340-360, 527-531,
  1132-1212, 1274-1367` — memory store factories, `refresh_dynamic_context`,
  `spawn_dreaming_task` + refresh wiring, memory path resolution.
- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:530-710, 824-978`
  — `apply_working_dir`, `apply_memory_store`, `refresh_instruction_context`,
  pool-agent creation and store/layer-manager wiring.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:834-1200` —
  `switch_workspace` / `exit_workspace` store + projection rebind.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:222-268` — bootstrap
  `ReviewIntegration` + `MemoryLayerManager` creation.
- `echo-agent-cli/src/{tauri/commands/memory.rs,tauri/commands/panels.rs,
  tui/events.rs,cli/cmd_impls/all.rs,cli/cmd_impls/evolution.rs}` — every
  `refresh_instruction_projection` / `refresh_instruction_context` caller.
- `echo-agent/echo-state/src/compression/mod.rs:30-62, 540-793` — projection
  envelope, `apply_projections` / `replace_projection` / `remove_projection`,
  `is_protected` / `split_protected` / `merge_protected`.
- `echo-agent/src/evolution/{dreaming.rs,layer.rs:480-706}` — Dreaming pass,
  `consider_promotion` → `promote_warm_to_hot` (writes `MEMORY.md`).
- `echo-agent/src/agent/react/run/context.rs:505-540` — per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall injection.
- `echo-agent-cli/echo-agent-app-core/src/workspace/layout.rs` (full) —
  canonical workspace paths.

## Out Of Scope

Deferred to named task IDs:

- **F-MEM-01** (complete, read) — `Store` / `ConversationStore` trait
  contracts, `FileStore` / `FileConversationStore` durability and path
  safety. This task relies on those identities; it does not re-audit them.
- **F-CMP-01** (complete, read) — compression correctness. This task
  consumes F-CMP-01's conclusion that protected content (including
  projections) survives compression and re-verifies only the
  instruction-specific interaction.
- **F-CTX-01** (complete) — context budget / phantom reservation and
  protected-token accounting. Owned there.
- **A-CFG-01** (complete, read) — config watcher stale-after-switch (P1-01)
  and config-not-reloaded-on-switch (P2-04). This task cross-references
  those for the workspace-switch behavior of instruction/memory files.
- Framework `MemoryLayerManager` hot/warm/cold correctness, `Dreaming`
  scoring, and `EvidenceStore` semantics — framework-side, owned by the
  framework evolution tasks. This task only inspects the **application
  refresh wiring** that reacts to their outputs.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` —
  framework/application layering gate, no-duplicate rule, the local-personal
  threat model, and the "first check if it already exists" rule.
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/{task-report,validation-report}.md`.

Dependency task reports read:

- `zcode-glm/tasks/F-CMP-01.md` (complete) — establishes that projections
  survive compression (V02) and that summary system messages accumulate
  (F-CMP-01-P2-01). This task re-verifies that projections are immune to
  that accumulation.
- `zcode-glm/tasks/F-MEM-01.md` (complete) — establishes the `Store` /
  `FileStore` identities and projection round-trip losslessness (V04-01).
  This task treats memory-store durability as a stable input.
- `zcode-glm/tasks/A-CFG-01.md` (complete) — establishes that the config
  watcher is not refreshed on workspace switch (P1-01) and config is not
  reloaded on switch (P2-04). This task cross-references both for
  instruction/memory-file staleness after a switch.

Historical documents treated as hypotheses:

- `instruction_provider.rs:1-23` module docstring claims the loader is
  "Static, file-only … Query-dependent dynamic memories are handled
  separately". Verified current; the implication (no file watcher) is
  confirmed and called out in P3-01.
- `unified_memory.rs:137-186` docstrings claim instruction and hot-memory
  are "two independently replaceable projections". Verified current at the
  framework layer; the application refresh wiring does not exercise the
  hot-memory independence (P1-01).
- `infra.rs:1132-1142` (`spawn_dreaming_task` doc) claims "When a pass
  changes the hot layer, the primary and pooled agents refresh their
  replaceable instruction projection immediately." Verified **regressed**:
  the code refreshes the **instruction** projection, but Dreaming writes
  the **hot** layer (`MEMORY.md`). The comment names the wrong projection
  (P1-01).

## Layering Decision

This is an **application-layer** task with a clean framework touchpoint.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework owns projection survival (`ContextManager::replace_projection` / `is_context_projection_message` / `split_protected` + `merge_protected`), the per-turn `TURN_MEMORY_CONTEXT_PROJECTION` recall, `MemoryLayerManager`'s hot/warm/cold model, and `Dreaming`. Any `echo-agent` consumer may rely on these. Correctly in `echo-state` / `echo-agent`. |
| EKO product policy | EKO owns its **file protocol** (`InstructionProvider`: `~/.eko/user.md`, the `AGENTS[.override].md` chain, `<root>/.eko/{project,local,learned-rules}.md`, and the `<root>/.eko/MEMORY.md` hot layer), the two projection markers (`eko:instruction-context`, `eko:hot-memory-context`), the `UnifiedMemory` wrapper, the `refresh_*_projection` helpers, and the refresh wiring in Dreaming / TUI / GUI / CLI surfaces. Correctly in `echo-agent-app-core`. |
| Adapter boundary | The four refresh helpers are thin: they read files via `InstructionProvider::load_for`, wrap the body in `Message::system`, and call the framework's `context.replace_projection(marker, message)`. No scheduling authority, no state ownership, no semantic loss. |
| Duplicate search | `InstructionProvider`, `UnifiedMemory`, `refresh_instruction_projection`, `refresh_hot_memory_projection`, `refresh_memory_projections`, `refresh_instruction_context`, `eko:instruction-context`, `eko:hot-memory-context`, `get_instruction_suffix`, `get_memory_suffix`, `load_hot_memory`, `agents_instructions_path`. Result: one canonical definition per concept. `UnifiedMemory` wraps `InstructionProvider` (no parallel loader). The framework's `TURN_MEMORY_CONTEXT_PROJECTION` / `WORKSPACE_CONTEXT_PROJECTION` are distinct per-turn/runtime projections, not duplicates of the two EKO stable-prefix projections. |
| Migration deletion | No deletion recommended. The two-layer model (hot `MEMORY.md` markdown + warm `store.json` typed KV) is intentional, not duplication. The legacy `AGENTS.md → learned-rules.md` one-time migration (`migrate_legacy_agents_file`) is correct and tested. |

EKO does own only its instruction/memory file protocol. The framework owns
projection survival and the dynamic recall/layer machinery. The defects
below are in the **application refresh wiring** (wrong projection target,
missing pool propagation), not in ownership or layering.

## Current Path

### Instruction/memory protocol assembly (verified — see V01)

`InstructionProvider::load_for(working_dir)` (`instruction_provider.rs:61`)
loads six independent file tiers and stores them as separate `Option<String>`
fields. There is **no override/merge semantics**: every tier that exists on
disk is concatenated, in a fixed order, into the prompt suffix.

`get_instruction_suffix()` (`:141-163`) concatenates, in order:
`User-level` (`~/.eko/user.md`) → `Repository instructions`
(`AGENTS[.override].md` chain via `echo_core::project_rules::InstructionResolver::agents_files_only`)
→ `Project-level` (`<root>/.eko/project.md`) → `Auto-promoted rules`
(`<root>/.eko/learned-rules.md`, falling back to legacy
`<root>/.eko/AGENTS.md`) → `Local directory` (`<cwd>/.eko/local.md`).

`get_memory_suffix()` (`:170-174`) appends the hot layer
(`## Active Memories (Hot Layer)` + `<root>/.eko/MEMORY.md` body, frontmatter
stripped). `get_system_prompt_suffix()` (`:121-134`) is
`instruction_suffix ++ memory_suffix`.

### Projection into the agent (verified — see V02)

Two independently-replaceable projections carry the protocol into the
agent's context:

- `eko:instruction-context` ← `get_instruction_suffix()` (the five file tiers)
- `eko:hot-memory-context` ← `get_memory_suffix()` (`MEMORY.md` body)

Both are installed by `refresh_memory_projections`
(`unified_memory.rs:170-186`) via `ContextManager::replace_projection`
(`mod.rs:605-611`). `replace_projection` removes any existing message with
the same marker envelope, then inserts the new `Message::system` wrapped in
the framework-reserved `<echo-agent-context-projection-v1>` envelope at the
system/history boundary (`mod.rs:546-573`).

Compression survival: `is_protected` (`mod.rs:678-691`) returns `true` for
any message where `is_context_projection_message` is true. `split_protected`
(`:747-773`) removes projection messages from the compressible set before
the compressor runs; `merge_protected` (`:781-793`) re-inserts them near
their original positions. Projections therefore **never enter the
compressor input** and cannot accumulate like summary system messages
(F-CMP-01-P2-01). Confirmed by V02.

### Refresh wiring (verified — see V03)

Refresh triggers split into two populations:

| Surface | When MEMORY.md changes | When learned-rules.md changes | Pool refreshed? |
|---|---|---|---|
| Workspace switch (`state.rs:883`, `agent_pool.rs:553`) | `refresh_memory_projections` (both) | `refresh_memory_projections` (both) | yes (`apply_working_dir`) |
| Dreaming (`infra.rs:1179-1192`) | `refresh_instruction_projection` (wrong target) | `refresh_instruction_projection` (correct) | yes (`refresh_instruction_context`) |
| GUI add/delete (`memory.rs:134,227`) | `refresh_instruction_projection` (wrong target) | n/a | yes (`refresh_instruction_context`) |
| TUI /remember//forget (`events.rs:2846,2917`) | `refresh_instruction_projection` (wrong target) | n/a | yes (`refresh_instruction_context`) |
| CLI /remember//forget (`all.rs:130,201`) | `refresh_instruction_projection` (wrong target) | n/a | **no** |
| Rule promote (`panels.rs:1561`, `evolution.rs:1489`) | n/a | `refresh_instruction_projection` (correct) | panels: yes; CLI evolution: **no** |

The "wrong target" rows are the root of P1-01: they modify `MEMORY.md`
(which is projected by `eko:hot-memory-context`) but refresh
`eko:instruction-context` (which deliberately excludes `MEMORY.md`, per
`get_instruction_suffix` at `:141-163`). `refresh_hot_memory_projection` is
invoked only from `refresh_memory_projections` (workspace switch) and from
one unit test.

### Pool refresh mechanics

`AgentPool::refresh_instruction_context` (`agent_pool.rs:687-710`) iterates
every pooled agent and calls `refresh_instruction_projection`. Despite its
doc comment ("Refresh hot-memory and instruction projections"), it does not
call `refresh_hot_memory_projection`. `apply_working_dir` (`:534-560`) and
`apply_memory_store` (`:597-684`) propagate working-dir and store/layer
changes to every pooled agent; `apply_memory_store` also records a
`memory_store_override` so future pool agents bind to the new store
(`:632-637`, consumed at `:906-911`).

### Workspace switch (verified — see V04)

`switch_workspace` (`state.rs:844-1032`) replaces: process CWD, primary +
pool agent `working_dir`, persistence, conversation store, runtime state
store, memory store + `MemoryLayerManager` (via `ReviewIntegration::rebind`
then `create_layer_manager`), workspace-curated skills, and workspace
routing. It refreshes **both** projections via `refresh_dynamic_context`
(`state.rs:883`, `agent_pool.rs:553`). `exit_workspace` (`:1052-1180`) is
symmetric, restoring global paths and refreshing via
`refresh_dynamic_context(agent, None)`.

### Memory-store topology (verified — see V04)

Two physically distinct files per workspace, by design (not duplication):

- `<root>/.eko/MEMORY.md` — hot layer, markdown body, read by
  `InstructionProvider::load_hot_memory` and written by
  `MemoryLayerManager::write_memory_file`.
- `<root>/.eko/memory/store.json` — warm layer, typed KV, read/written by
  `FileStore` via `create_memory_store_for_workspace`.

Global fallback layout differs: warm store at `~/.eko/store.json` (no
`memory/` subdirectory) and hot layer at `~/.eko/MEMORY.md`. Both are
internally consistent; the asymmetry is noted in P3-02.

## Findings

### A-MEM-01-P1-01: Hot-layer (MEMORY.md) edits refresh the wrong projection — promoted/deleted hot memories never reach the agent's stable prefix

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:28-29, 138-167`
    — the two markers are distinct; `refresh_instruction_projection` targets
    `eko:instruction-context` only, `refresh_hot_memory_projection` targets
    `eko:hot-memory-context` only.
  - `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:170-186` —
    `refresh_memory_projections` is the only helper that refreshes both.
  - `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:141-163`
    — `get_instruction_suffix` deliberately excludes `hot_memory`
    (`MEMORY.md`); only `get_memory_suffix` (`:170-174`) emits it.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:1175-1192`
    (`spawn_dreaming_task`) — on `report.promoted > 0`, calls
    `refresh_instruction_projection` (primary) + `refresh_instruction_context`
    (pool). Dreaming promotes via `consider_promotion → promote_warm_to_hot`,
    which writes `MEMORY.md` (`echo-agent/src/evolution/layer.rs:685-706`).
  - `echo-agent-cli/src/tauri/commands/memory.rs:126-145` (add_memory) and
    `:219-238` (delete_memory on `MemoryLayer::Hot`) — same pattern.
  - `echo-agent-cli/src/tui/events.rs:2839-2857` (`/remember`) and
    `:2913-2927` (`/forget` Hot) — same pattern.
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:123-138` (`/remember`) and
    `:194-209` (`/forget` Hot) — same pattern.
- Reachability: any `write_memory` that promotes to hot, any `delete_memory`
  on a hot entry, or any Dreaming pass with `promoted > 0`. All three call
  sites re-read the file via `InstructionProvider::load_for` but then push
  the result into the **instruction** projection. The hot-memory projection
  retains its previous value until the next workspace switch
  (`refresh_dynamic_context` → `refresh_memory_projections`) or process
  restart.
- Expected invariant (the task's own question): when the memory protocol
  changes, the primary and pooled Agents observe the change immediately and
  consistently. Concretely, a memory promoted to `MEMORY.md` should appear
  in the agent's `## Active Memories (Hot Layer)` stable prefix on the next
  turn.
- Observed behavior: the `## Active Memories (Hot Layer)` segment of the
  agent's context is frozen at agent-creation time. Promoted memories do
  not appear there; deleted hot memories do not disappear from there. The
  `eko:instruction-context` projection is needlessly re-written with
  identical content (instructions do not change when only `MEMORY.md`
  changes), masking the omission.
- Impact: the headline capability of Dreaming (recall-driven promotion to a
  stable prompt prefix) and of `/remember` auto-promotion is silently
  broken on the primary surface it was built for — the stable prefix.
  Promoted memories may still surface via the framework's per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall (`context.rs:514-532`) when they
  match the current query, but that is the recall path, not the
  hot-layer injection the feature advertises. `spawn_dreaming_task`'s own
  doc comment (`infra.rs:1140-1142`) promises "agents refresh their …
  projection immediately" — false for the hot-memory projection.
- Root cause: the refresh helpers were written when the two projections were
  separated, but the call sites were never updated to pick the right one.
  Every MEMORY.md-mutating site copied the same `refresh_instruction_projection`
  + `pool.refresh_instruction_context()` snippet used by the
  learned-rules.md sites, without noticing that `MEMORY.md` is excluded
  from `get_instruction_suffix`.
- Direction: at every MEMORY.md-mutating site, call
  `refresh_hot_memory_projection` (primary) and add a pool-level
  `refresh_hot_memory_context` that mirrors `refresh_instruction_context`
  but calls `refresh_hot_memory_projection`. The simplest mechanical fix
  is to replace the eight wrong-target call sites with
  `refresh_memory_projections` (which refreshes both, idempotently) and
  fix the pool helper's doc + body to match. The learned-rules.md sites
  (panels.rs:1561, evolution.rs:1489) are already correct and should stay
  on `refresh_instruction_projection`.
- Regression validation: seed a workspace, call
  `layer_manager.write_memory` with a high-confidence entry that triggers
  promotion, then assert the agent's `eko:hot-memory-context` projection
  contains the new key (existing test
  `instruction_and_hot_memory_use_distinct_projections` at
  `unified_memory.rs:248-267` covers projection independence but not the
  promotion-refresh path; add a promotion-refresh test).
- Validation reports: [V03-01](../validations/A-MEM-01/V03-01.md),
  [V01-01](../validations/A-MEM-01/V01-01.md)

### A-MEM-01-P2-01: `AgentPool::refresh_instruction_context` doc claims hot-memory refresh but only refreshes instructions

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:686-710`
  — doc comment "Refresh hot-memory and instruction projections on every
  existing agent"; body calls only `refresh_instruction_projection`
  (`:701`).
- Reachability: every pool refresh triggered by Dreaming (`infra.rs:1191`),
  GUI memory commands (`memory.rs:143,236`, `panels.rs:1570`), and TUI
  memory commands (`events.rs:2855,2926`).
- Expected invariant: a pool helper named `refresh_instruction_context`
  whose doc claims both projections should refresh both, or its doc should
  name only instructions.
- Observed behavior: the helper refreshes only instructions. This is the
  pool-side half of P1-01: even if a caller wanted to refresh hot memory on
  the pool, the helper does not expose that capability.
- Impact: misleading API; the pool has no path to refresh hot memory short
  of `apply_working_dir` (workspace switch). Compounds P1-01 — there is
  currently no single call that refreshes hot memory across the pool.
- Root cause: the helper predates the instruction/hot-memory split (or was
  never updated when `refresh_hot_memory_projection` was added at
  `unified_memory.rs:154`).
- Direction: either (a) widen the helper to refresh both (rename to
  `refresh_memory_context`, call `refresh_memory_projections` per agent), or
  (b) keep it instruction-only and fix the doc. (a) is preferred because it
  is the minimal building block P1-01's fix needs.
- Regression validation: covered by the P1-01 promotion-refresh test
  extended to assert pooled agents also reflect the promoted key.
- Validation reports: [V03-01](../validations/A-MEM-01/V03-01.md)

### A-MEM-01-P2-02: CLI /remember, /forget, and rule-promote refresh only the primary agent — pooled/background agents diverge

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:123-142` (`/remember`) —
    refreshes `ctx.agent` only; no `pool.refresh_*` call.
  - `echo-agent-cli/src/cli/cmd_impls/all.rs:194-214` (`/forget` Hot) —
    same.
  - `echo-agent-cli/src/cli/cmd_impls/evolution.rs:1471-1496` (rule
    promote) — same.
  - Contrast: `echo-agent-cli/src/tauri/commands/memory.rs:142-144`,
    `panels.rs:1569-1571`, and `tui/events.rs:2854-2856,2925-2927` all
    follow the primary-agent refresh with `pool.refresh_instruction_context()`.
- Reachability: any CLI-driven memory or rule promotion/deletion while a
  background/pool agent exists.
- Expected invariant: the refresh fan-out should be the same regardless of
  which surface (CLI/TUI/GUI) initiated the change.
- Observed behavior: CLI paths mutate the file and refresh the primary
  agent, but pooled agents (including the always-on `__background__` agent)
  keep the stale projection until the next workspace switch or restart.
- Impact: a user who runs `/remember <fact>` in the CLI TUI and then
  dispatches a background task sees the background agent operate against the
  pre-`/remember` protocol. Inconsistent with the GUI/TUI behavior and with
  the task's "consistent projection to pooled Agents" requirement.
- Root cause: the CLI snippets predate the pool-fan-out convention and were
  not updated when GUI/TUI gained it.
- Direction: after each CLI refresh, mirror the GUI/TUI pattern with
  `if let Some(pool) = &pool { pool.refresh_instruction_context().await; }`
  (and, once P2-01 is fixed, the corresponding hot-memory call). Better:
  extract a single `refresh_memory_after_edit(primary, pool, root)` helper
  in `unified_memory.rs` and call it from all eight sites so the fan-out
  cannot drift again.
- Regression validation: seed a pool with a background agent, run CLI
  `/remember`, assert the background agent's instruction projection
  reflects the new rule within the same command.
- Validation reports: [V03-01](../validations/A-MEM-01/V03-01.md)

### A-MEM-01-P3-01: Instruction and MEMORY.md files have no file watcher — external edits are not hot-reloaded

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `instruction_provider.rs:14-23` ("Static, file-only loader: no
  DB, no embeddings, no recall"); `refresh_*_projection` are only invoked by
  the explicit triggers enumerated in the Current Path refresh table; the
  config watcher (`config_watcher.rs`, per A-CFG-01) watches only
  `echo-agent.yaml` and `hooks.yaml`, not the instruction/memory files.
- Reachability: any edit to `~/.eko/user.md`, `<root>/.eko/project.md`,
  `<root>/.eko/local.md`, `<root>/.eko/learned-rules.md`, or
  `<root>/.eko/MEMORY.md` made outside the product's own commands.
- Expected invariant: either external edits hot-reload, or the limitation
  is documented at the user-facing surface.
- Observed behavior: external edits are invisible until a workspace switch,
  a Dreaming pass (for `MEMORY.md` only, and even then via the wrong
  projection per P1-01), or a restart. The module doc states the loader is
  "static" but does not state the user-visible consequence.
- Impact: low for a local single-user assistant (most edits flow through
  the product's own commands). Notably compounded by A-CFG-01-P1-01
  (watcher targets not refreshed on switch) and A-CFG-01-P2-04 (config not
  reloaded on switch): after a switch, neither config nor instruction files
  are watched for the new workspace.
- Root cause: by design — the loader is intentionally static; no watcher was
  ever wired for these files.
- Direction: either add a dedicated file watcher for the six instruction
  tiers (mirroring `spawn_config_watcher`) that calls
  `refresh_memory_projections` on change, or document the restart/switch
  requirement in the user-facing instructions UI. The documentation option
  is the lower-risk fix and consistent with the local-assistant threat
  model.
- Regression validation: documentation check; if a watcher is added,
  replay A-CFG-01's V02/V03 to confirm it does not inherit the
  stale-after-switch defect.
- Validation reports: [V04-01](../validations/A-MEM-01/V04-01.md)

### A-MEM-01-P3-02: Global vs workspace memory-store path layout is asymmetric

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:1274-1278`
    (`global_memory_paths`) — global warm store at `~/.eko/store.json`, i.e.
    directly under the `.eko` root.
  - `echo-agent-cli/echo-agent-app-core/src/workspace/layout.rs:47-61`
    — workspace warm store at `<root>/.eko/memory/store.json`, i.e. under a
    `memory/` subdirectory.
  - Hot layer is consistent across both scopes: `~/.eko/MEMORY.md` and
    `<root>/.eko/MEMORY.md` (no subdirectory).
- Reachability: any `exit_workspace` (writes `~/.eko/store.json`) following
  a `switch_workspace` (wrote `<root>/.eko/memory/store.json`).
- Expected invariant: the warm-store filename/dirname convention should be
  the same in global and workspace scope, so tooling and backup recipes can
  treat them uniformly.
- Observed behavior: two different layouts. Each is internally consistent
  (the global factory reads/writes the global path; the workspace factory
  reads/writes the workspace path; `resolve_memory_store_paths`
  (`infra.rs:1288-1313`) returns the matching pair), so this is not a
  correctness bug.
- Impact: minor. Surprising for anyone who greps the filesystem expecting a
  single layout; a future migration that moves the global store under
  `~/.eko/memory/store.json` would need a one-time move but no code change
  beyond `global_memory_paths`.
- Root cause: `global_memory_paths` predates the `WorkspaceLayout::memory`
  convention and was not reconciled.
- Direction: align `global_memory_paths` to `~/.eko/memory/store.json`
  (add a one-time migration that moves any existing `~/.eko/store.json` into
  place), or document the asymmetry on `global_memory_paths`. The migration
  is the cleaner long-term fix; per AGENTS.md "代码清理" no backward
  compatibility is required.
- Regression validation: after migration, bootstrap with an old
  `~/.eko/store.json`, assert it is moved and read; bootstrap fresh, assert
  the new path is used.
- Validation reports: [V04-01](../validations/A-MEM-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Instruction layer/precedence + duplicate authority search | yes | passed | [V01-01](../validations/A-MEM-01/V01-01.md) |
| V02 | Instruction/hot-memory projection compression survival | yes | passed | [V02-01](../validations/A-MEM-01/V02-01.md) |
| V03 | Pool refresh triggers: immediate vs lazy, primary vs pool | yes | failed (P1-01, P2-01, P2-02) | [V03-01](../validations/A-MEM-01/V03-01.md) |
| V04 | Workspace-switch + duplicate store topology | yes | passed (with P3-01/P3-02 notes) | [V04-01](../validations/A-MEM-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three module docs make auditable claims) | done — see Historical Claim Status table below | — |

Targeted executable checks were not run: this task touches only application
wiring and framework read paths already covered by F-CMP-01's
`cargo test -p echo_state --lib compression::` run (which subsumes the
projection-survival tests at `mod.rs:1920-2130`). No new feature flag or
public API change is in scope, so the conditional feature matrix is not
required.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `instruction_provider.rs:14-23` "Static, file-only loader: no DB, no embeddings, no recall … dynamic memories handled separately" | current | V01 confirms; the no-watcher consequence is P3-01. |
| `unified_memory.rs:137-186` "instruction and hot-memory projections are independently replaceable" | current at framework layer, regressed at application layer | The framework supports independent replacement; no application call site exercises hot-memory replacement outside workspace switch (P1-01). |
| `infra.rs:1132-1142` (`spawn_dreaming_task`) "When a pass changes the hot layer, the primary and pooled agents refresh their replaceable instruction projection immediately" | regressed | The code refreshes `eko:instruction-context`, but Dreaming writes `MEMORY.md` which is projected by `eko:hot-memory-context`. The comment names the wrong projection; the hot-memory projection is not refreshed (P1-01). |
| `agent_pool.rs:686` "Refresh hot-memory and instruction projections on every existing agent" | stale | Body refreshes only instructions (P2-01). |
| `instruction_provider.rs:84-113` legacy `AGENTS.md → learned-rules.md` migration "renames if the new file does not already exist; never overwrites user-authored content" | current | Verified by `migrate_legacy_agents_file_*` tests at `:334-379`. |
| F-CMP-01 handoff: "Protected content survives compression" | current | V02 re-confirms for both EKO projections. |
| F-MEM-01 handoff: "Store/FileStore identities are single authority" | current | V04 confirms no parallel store authority in the application layer. |
| A-CFG-01-P1-01 / P2-04: watcher + config not refreshed on workspace switch | current (cross-reference) | Neither config nor instruction/memory files are watched for the new workspace post-switch; P3-01 notes the instruction-file half. |

## Coverage And Uncertainty

- **Not executed at runtime.** All four validations are static reads. P1-01
  is verified by unambiguous code trace (eight call sites refresh the wrong
  projection marker; `refresh_hot_memory_projection` has no production
  caller outside workspace switch). A runtime test that promotes a memory
  and asserts the agent's `eko:hot-memory-context` envelope changes would
  raise P1-01 from high-confidence to confirmed; the existing test
  `instruction_and_hot_memory_use_distinct_projections` proves the
  projections are independent but does not exercise the promotion-refresh
  path.
- **Per-turn recall mitigation.** P1-01's impact is softened by the
  framework's `TURN_MEMORY_CONTEXT_PROJECTION` recall
  (`echo-agent/src/agent/react/run/context.rs:514-532`), which dynamically
  surfaces warm-store memories that match the current query each turn. So
  promoted memories are not wholly invisible — they are invisible *as the
  stable hot-layer prefix*, which is the specific surface Dreaming and
  `/remember` auto-promotion target. The severity classification (P1) rests
  on the stable-prefix feature being defeated, not on total memory loss.
- **Framework layer machinery not re-audited.** `MemoryLayerManager`'s
  hot/warm promotion scoring, `Dreaming`'s recall-frequency logic, and
  `EvidenceStore`'s inbox semantics are taken as given (framework scope).
  This task inspects only the application's reaction to their outputs.
- **`refresh_tail_projection` vs `replace_projection`.** The framework
  offers both a system-boundary replace (`replace_projection`, used by EKO)
  and a tail append (`replace_tail_projection`, used for per-turn recall).
  EKO correctly uses the boundary variant for its stable prefixes; this
  was verified but not promoted to a finding because it is correct.
- **Subagent context.** The subagent context builder
  (`echo-agent/src/agent/subagent/context*.rs`) was not inspected; F-SUB-01
  owns it. P1-01's refresh gap applies to subagents only insofar as they
  inherit pool-agent behavior; formal TaskRuntime subagents build fresh
  context per dispatch and are unaffected by stale projections on
  long-lived pool agents.

## Handoff

Conclusions downstream tasks may rely on:

1. **EKO owns only its instruction/memory file protocol.** The framework
   owns projection survival, dynamic recall, and the layer model. No
   duplicate authority exists. Downstream tasks reasoning about "who owns
   the prompt suffix" can rely on `InstructionProvider` + `UnifiedMemory`
   being the single application authority.
2. **Projections survive compression.** Both `eko:instruction-context` and
   `eko:hot-memory-context` are protected by the framework envelope and
   never enter compressor input (V02). Downstream tasks that model
   long-session behavior can treat the protocol suffix as stable across
   compression, independent of F-CMP-01-P2-01 (summary accumulation).
3. **Hot-layer refresh is broken on every memory-edit surface except
   workspace switch (P1-01).** Any downstream task that promises
   "memory is immediately visible after promotion" must consume P1-01 as a
   prerequisite. The per-turn recall path partially compensates.
4. **Workspace switch is the single consistent refresh path** — it refreshes
   both projections on primary and pool, rebinds the store, and rebuilds
   the layer manager. Downstream tasks should model switch as the
   ground-truth refresh and treat the other surfaces as best-effort.
5. **CLI surfaces do not fan out refresh to the pool (P2-02).** Downstream
   consistency tests should not assume CLI-driven changes reach background
   agents.

Reports downstream tasks must read:

- This report's V01 (precedence/layering) and V03 (refresh-wiring table)
  for any task touching the instruction/memory refresh paths.
- `zcode-glm/tasks/F-CMP-01.md` for projection-survival and summary-
  accumulation context (cross-referenced in V02).
- `zcode-glm/tasks/A-CFG-01.md` P1-01 / P2-04 for the workspace-switch
  watcher/config staleness that compounds P3-01.

Conditions that make this report stale:

- Any change to the eight MEMORY.md-mutating call sites that switches them
  to `refresh_hot_memory_projection` or `refresh_memory_projections`
  (resolves P1-01; re-run V03-01).
- Widening `AgentPool::refresh_instruction_context` to refresh both
  projections (resolves P2-01; re-run V03-01).
- Adding pool fan-out to the three CLI sites (resolves P2-02; re-run
  V03-01).
- Adding a file watcher for the instruction tiers (resolves P3-01; re-run
  V04-01 and cross-check against A-CFG-01 V02/V03).
- Aligning `global_memory_paths` to the `memory/` subdirectory layout
  (resolves P3-02; re-run V04-01).

Follow-up task IDs (no fixes implemented in this review):

- A dedicated memory-refresh-coherence task should land P1-01 + P2-01 +
  P2-02 together: they share a single fix shape (route every edit through
  one `refresh_memory_after_edit(primary, pool, root)` helper that calls
  `refresh_memory_projections` on the primary and a widened pool helper).
  P1-01 is the highest-impact and the smallest semantic change.
- P3-01 and P3-02 are documentation/layout cleanups and can ride along
  with any nearby UX or storage task.
