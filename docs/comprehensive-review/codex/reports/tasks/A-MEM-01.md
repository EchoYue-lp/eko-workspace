# A-MEM-01: Instructions, hot memory, and Dreaming

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent-cli` remained clean; unrelated concurrent
> `echo-agent` source changes were excluded without reading, modification, or
> rollback. Framework conclusions use committed source and completed Codex
> dependency reports.

## Question

Does EKO own only its instruction/memory protocol while projecting updates
immediately and consistently to primary and pooled Agents?

## Scope

- User/project/repository/local/learned-rule/hot-memory file precedence and
  projection markers.
- Framework Store and MemoryLayerManager wiring into primary and pooled Agents.
- Workspace switch/exit resource identity, ReviewIntegration rebinding, live
  memory mutations, Dreaming, rule promotion, and existing/future pool refresh.
- Compression survival, definition/registration/reachability, duplicate
  authority search, historical claims, and static test inventory.

## Out Of Scope

- Framework FileStore corruption, multi-instance lost updates, namespace, and
  lock defects are owned by [F-MEM-01](F-MEM-01.md). This report proves the EKO
  production topology that activates the accepted multi-instance defect.
- Compression ordering and verifier defects are owned by
  [F-CMP-01](F-CMP-01.md). This report verifies only projection identity and
  survival.
- General workspace root/config authority is owned by
  [A-CFG-01](A-CFG-01.md). This report adds memory-binding-specific evidence.
- Evolution proposal correctness, Skill evolution, UI design, source changes,
  and dynamic execution.
- Cargo, rustc, tests, builds, fixtures, and network activity, per the user's
  review-only instruction.

## Inputs

- Root `AGENTS.md`; review `README.md`, `REPORTING.md`, exact A-MEM-01 card in
  `TASKS.md`, and templates.
- Completed Codex dependencies [A-CFG-01](A-CFG-01.md),
  [F-CMP-01](F-CMP-01.md), and [F-MEM-01](F-MEM-01.md).
- Current clean CLI source plus committed framework source at the commits above.
  No other reviewer directory was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Store traits/implementations, typed memory, layered promotion/demotion, compression-safe projections, Dreaming mechanics, and evolution observers correctly belong in `echo-agent`. File, in-memory, and SQLite implementations remain valid framework options. |
| EKO product policy | `.eko`/`~/.eko` file names and precedence, project isolation, workspace generation, which Agents receive which projections, surface commands, and Dreaming scheduling belong in `echo-agent-cli`. EKO must remain file/memory-only and must not enable SQLite. |
| Adapter boundary | EKO should prepare one immutable `MemoryBinding { generation, root, store }`, install the same Store Arc into ReviewIntegration, primary, existing pool, and future pool, then reconcile instruction and hot projections through one receipt-producing adapter. The adapter must not reimplement Store, promotion, compression, or recall. |
| Duplicate search | Searched both repositories for Store construction/installation, MemoryLayerManager, InstructionProvider, projection markers, refresh helpers, workspace switch/exit, Dreaming, remember/forget, evidence acceptance, rule promotion, and pool overrides/callers. |
| Migration deletion | Keep framework memory implementations. Replace EKO's path-based pool store reopen and split ReviewIntegration locks with one binding; delete the misleading instruction-only refresh facade after all callers use one projection reconciler. |

## Current Data Flow

```text
instruction files ----------------------> instruction projection marker
project/global .eko/MEMORY.md ----------> hot-memory projection marker
                                                |
Store -- MemoryLayerManager -- writes/promotes -+-- expected refresh receipt
  |                                             |
  +--> primary Agent                            +--> existing pooled Agents
  +--> ReviewIntegration/Dreaming               +--> future pooled Agents
```

Bootstrap initially shares one Store Arc among primary, ReviewIntegration, and
the pool. Workspace switch breaks that identity: `AppState` opens one Store for
primary/ReviewIntegration, while `AgentPool::apply_memory_store(root)` opens a
second Store for the same `store.json`. Exit repeats the split for the global
path. Live hot-layer changes also do not converge projections: call sites either
invoke the instruction-only helper or perform no refresh.

## Findings

### A-MEM-01-P0-01: Workspace switch and exit bind primary and pooled Agents to distinct FileStore snapshots of the same file

- Priority: P0
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:941`, `:946`,
  `:949`, `:969`, `:1011`, `:1013`; `:1120`, `:1130`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:586`, `:597`, `:598`,
  `:609`, `:613`, `:614`, `:622`, `:627`; F-MEM-01-P0-02.
- Reachability: every successful workspace switch creates the primary Store,
  installs it, then passes only the root path to the pool. The pool independently
  calls `create_memory_store_for_workspace`. Exit similarly creates one global
  Store for primary and a second in `apply_memory_store_global`.
- Expected invariant: all Agents and memory services targeting one physical
  memory file share one synchronization/snapshot owner.
- Observed behavior: bootstrap shares one Arc, but switch/exit create two
  independent FileStore instances for the same file. F-MEM-01-P0-02 established
  that such instances retain independent full snapshots and a later write can
  erase the other's successful update.
- Impact: a memory saved by the primary or one pooled Agent can disappear after
  another Agent writes. Both operations can report success; restart makes the
  overwritten state permanent.
- Root cause: the pool adapter accepts a path and reopens storage instead of
  accepting the already-prepared Store identity.
- Direction: create one application `MemoryBinding` per workspace generation and
  pass its exact `Arc<dyn Store>` to ReviewIntegration, primary, pool override,
  existing Agents, and future Agents. Delete path-based reopen methods. Do not
  replace or delete the framework FileStore here.
- Regression validation: switch/exit, write distinct keys through primary and
  two pooled Agents in interleaved order, restart, and assert the union plus Arc
  identity/generation agreement.
- Validation reports: [V06](../validations/A-MEM-01/V06-01.md)

### A-MEM-01-P1-02: Hot-memory mutations refresh the instruction marker or no marker, leaving stale facts in live model context

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:137`,
  `:153`, `:169`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:686`,
  `:701`; `echo-agent-cli/src/tauri/commands/memory.rs:126`, `:134`, `:219`,
  `:227`; `echo-agent-cli/src/tui/events.rs:2839`, `:2846`, `:2910`, `:2917`;
  `echo-agent-cli/src/cli/cmd_impls/memory.rs:126`; framework
  `LayeredRememberTool`/forget and application evidence/task-memory callers.
- Reachability: GUI, TUI, and CLI remember/forget commands are live. Framework
  memory tools are installed on Agents. Dreaming and evidence/task paths can
  also promote or delete hot entries.
- Expected invariant: every committed hot-layer change replaces
  `eko:hot-memory-context` for primary and existing pool Agents before the next
  model request; future Agents load the same generation.
- Observed behavior: promotion and hot deletion call
  `refresh_instruction_projection`; pool's documented “hot-memory and
  instruction” method also calls only that helper. Tool/evidence/task mutations
  have no application refresh callback. The independent hot marker therefore
  retains deleted facts and omits newly promoted facts.
- Impact: model behavior depends on Agent age and mutation entry point. Deleted
  memory can continue influencing an active Agent, while a new Agent sees the
  new file state.
- Root cause: Store mutation, layer promotion, and EKO projection reconciliation
  have no shared typed receipt/observer contract.
- Direction: consume framework evolution/memory change receipts in one EKO
  projection reconciler; refresh both domains from one snapshot where ownership
  is uncertain, and specifically the hot marker for hot changes. Apply it to
  primary and all existing pool Agents; future Agents use the binding snapshot.
  Delete/rename the misleading instruction-only pool facade.
- Regression validation: exercise every remember, forget, auto-promotion,
  Dreaming, evidence accept/undo/merge, and task-memory entry point; compare
  exact marker content on primary, existing pool, and newly created pool Agent.
- Validation reports: [V05](../validations/A-MEM-01/V05-01.md)

### A-MEM-01-P1-03: ReviewIntegration's separately locked path and Store can form a cross-workspace pair

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/evolution/review_integration.rs:39`,
  `:43`, `:45`, `:70`, `:75`, `:76`, `:79`, `:82`, `:133`, `:138`, `:144`,
  `:241`, `:245`, `:250`; `echo-agent-cli/echo-agent-app-core/src/state.rs:948`.
- Reachability: workspace switching calls `rebind` on a shared Arc while manual
  review, Dreaming, layer-manager construction, evidence access, and pool Agent
  creation can read the same object.
- Expected invariant: directory, Store, and generation are one atomic snapshot;
  no consumer can combine resources from different workspaces.
- Observed behavior: `rebind` writes two independent locks and always logs
  success even if one write lock is poisoned. `run_review_inner` and
  `runtime_builder` read the locks separately. A reader can observe new path +
  old Store or old path + new Store despite comments claiming atomic rebinding.
- Impact: review/Dreaming can read one project's memories and write its
  `MEMORY.md`, evidence, change log, candidates, or Skills under another
  project's directory.
- Root cause: related resource identity was represented as independently mutable
  fields rather than one immutable generation value.
- Direction: store one `MemoryBinding` under one lock/ArcSwap and make rebind
  fallible with an explicit generation/receipt. Every operation clones that one
  snapshot once. Fold this into the broader A-CFG workspace resource transaction
  without creating another state authority.
- Regression validation: pause a review/layer-manager creation at every binding
  read, switch A->B, and assert every artifact/Store access remains entirely A
  or entirely B; inject lock/rebind failure and require truthful failure.
- Validation reports: [V07](../validations/A-MEM-01/V07-01.md)

### A-MEM-01-P1-04: A project without `.eko/MEMORY.md` silently receives global hot memory despite project-local Store isolation

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:248`,
  `:251`, `:252`, `:253`, `:254`, `:255`, `:256`;
  `echo-agent-cli/echo-agent-app-core/src/state.rs:941`, `:943`, `:946`.
- Reachability: initial Agent creation and every projection refresh call
  `InstructionProvider::load_for(Some(project_root))`.
- Expected invariant: explicit project scope reads project hot memory; absence is
  empty. Global hot memory is used only in explicit global mode, unless a
  separately named user-global tier exists.
- Observed behavior: when the project file does not exist, `load_hot_memory`
  falls back to `~/.eko/MEMORY.md`. Warm Store remains project-local. When the
  first project promotion creates the file, the entire hot projection abruptly
  changes from global to project.
- Impact: one project's learned facts can influence another new project without
  disclosure, and hot/warm layers disagree about scope.
- Root cause: filesystem absence is used as an implicit scope-selection policy.
- Direction: when `project_root` is `Some`, read only that project's hot file;
  use global only when root is `None`. If user-global learned memory is desired,
  model it as an explicit independently rendered tier with user controls.
- Regression validation: global MEMORY present with project file absent,
  unreadable, empty, and later created; assert deterministic non-leaking scope.
- Validation reports: [V03](../validations/A-MEM-01/V03-01.md)

### A-MEM-01-P1-05: CLI rule promotion refreshes only the primary Agent, so pooled Agents retain old rules

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/cmd_impls/evolution.rs:1415`, `:1471`;
  `echo-agent-cli/src/tauri/commands/panels.rs:1515`, `:1546`, `:1557`, `:1568`;
  CLI `CommandContext` construction and AgentPool refresh callers.
- Reachability: `/rule promote` is a live CLI command; GUI exposes the same rule
  acceptance through Tauri. Both write `.eko/learned-rules.md`.
- Expected invariant: after a reported successful promotion, primary, existing
  pool, and future pool Agents use the same instruction projection.
- Observed behavior: GUI refreshes primary and pool. CLI refreshes only the
  primary because its command context has no pool refresh capability. Existing
  pooled Agents keep the old rules; later-created Agents load the updated file.
- Impact: background/multi-session behavior diverges from the interactive Agent,
  and the same accepted rule may be applied or ignored depending on Agent age.
- Root cause: each surface owns post-mutation side effects instead of consuming
  one application mutation receipt.
- Direction: move rule acceptance behind one app-core service returning a
  durable mutation generation, then reconcile primary and pool through the same
  projection adapter. Delete surface-specific refresh sequences after migration.
- Regression validation: promote from every surface while two pool Agents exist,
  create a third afterwards, and compare exact rule marker/generation.
- Validation reports: [V08](../validations/A-MEM-01/V08-01.md)

### A-MEM-01-P2-06: Instruction and hot-memory read errors are treated as absence and can tombstone the last good projection

- Priority: P2
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/instruction_provider.rs:196`,
  `:202`, `:210`, `:213`, `:256`, `:282`;
  `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:142`, `:143`, `:146`,
  `:158`, `:159`, `:162`, `:174`, `:183`.
- Reachability: bootstrap, workspace refresh, promotion, and memory commands load
  these files before replacing projection markers.
- Expected invariant: only a confirmed `NotFound` removes a projection. Invalid
  UTF-8, permission, and I/O errors preserve last-known-good context and surface
  a diagnostic/refresh failure.
- Observed behavior: file loads use `.ok()` and collapse all errors to `None`.
  Projection refresh interprets `None` as a marker tombstone and reports no
  failure.
- Impact: critical user/project rules or hot facts silently disappear from the
  next model request during a refresh, while the surface reports unrelated
  mutation/switch success.
- Root cause: the file protocol has no typed snapshot result or last-known-good
  generation.
- Direction: return a typed `InstructionSnapshot`/error set; distinguish missing
  from unreadable, retain the last committed projection on error, and expose a
  surface-visible degraded state. Keep this in EKO, not the framework Store API.
- Regression validation: permission, invalid UTF-8, partial replacement, and
  transient read errors for each tier; assert no tombstone and truthful status.
- Validation reports: [V03](../validations/A-MEM-01/V03-01.md)

## Positive Conclusions

- EKO's instruction and hot-memory context use distinct framework-owned marker
  envelopes. Replacement removes only the selected marker and prevents duplicate
  copies across ordinary refreshes.
- Projection messages are compression-protected and reinserted. F-CMP-01-P1-02
  still owns incorrect protected-message reinsertion ordering; this report does
  not claim semantic ordering correctness.
- Project structural context no longer embeds AGENTS/instruction content, so the
  old direct project-context duplication has been removed.
- Dreaming is now launched by GUI, TUI, and CLI paths; the older GUI-only claim
  is fixed.
- Workspace working-directory and routing refreshes reach primary, existing
  pooled Agents, and future pool configuration. The defects are specifically
  memory Store identity and mutation projection receipts.
- EKO does not enable SQLite. Framework SQLite remains a valid independent
  option and is not a deletion target.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Inputs, commits, dirty-source isolation, and exact scope | yes | passed | [V00](../validations/A-MEM-01/V00-01.md) |
| V01 | Layering, definition, and duplicate-authority map | yes | passed | [V01](../validations/A-MEM-01/V01-01.md) |
| V02 | Bootstrap/surface/Dreaming production reachability | yes | passed | [V02](../validations/A-MEM-01/V02-01.md) |
| V03 | Instruction precedence, isolation, and file error semantics | yes | failed | [V03](../validations/A-MEM-01/V03-01.md) |
| V04 | Projection identity and compression survival | yes | passed | [V04](../validations/A-MEM-01/V04-01.md) |
| V05 | Hot-memory mutation-to-projection trigger matrix | yes | failed | [V05](../validations/A-MEM-01/V05-01.md) |
| V06 | Store instance identity across bootstrap/switch/exit | yes | failed | [V06](../validations/A-MEM-01/V06-01.md) |
| V07 | ReviewIntegration binding/generation atomicity | yes | failed | [V07](../validations/A-MEM-01/V07-01.md) |
| V08 | Rule promotion and primary/pool surface parity | yes | failed | [V08](../validations/A-MEM-01/V08-01.md) |
| V09 | Existing tests and missing regression inventory | yes | passed | [V09](../validations/A-MEM-01/V09-01.md) |
| V10 | Historical claim classification and finding ownership | yes | passed | [V10](../validations/A-MEM-01/V10-01.md) |
| V11 | Targeted dynamic fault/concurrency matrix | conditional | not_run | [V11](../validations/A-MEM-01/V11-01.md) |
| V12 | Exact links/IDs/executor/path/source-isolation integrity | yes | passed | [V12](../validations/A-MEM-01/V12-02.md) |

## Historical Claim Status

| Claim | Classification | Current evidence |
|---|---|---|
| Memory hot and warm layers are physically isolated by workspace and never diverge | regressed | Store roots are project-local, but project hot-memory absence falls back global and Store instances split; V03/V06. |
| Instruction and dynamic context survive compression | current with framework caveat | Separate marker envelopes are protected; F-CMP-01 owns ordering defects; V04. |
| Dreaming runs only in desktop/GUI | fixed | GUI, TUI, and CLI launch Dreaming; V02/V10. |
| Pool memory refresh updates “hot-memory and instruction projections” | regressed | Method comment says both; implementation refreshes only instructions; V05. |
| ReviewIntegration rebinds directory and Store atomically | regressed | Two separate locks and reads permit cross-generation pairs; V07. |

## Coverage And Uncertainty

- Constructor identity, missing calls, wrong helper calls, fallback branching,
  and separate-lock interleavings are source-conclusive.
- No filesystem or concurrency fixture was run. V11 is a future implementation
  gate, not a blocker to static review completion.
- External framework consumers are not assumed to use EKO's `.eko` protocol;
  recommendations stay in the application adapter.
- Whether user-global learned memory should exist is a product decision. The
  current implicit absence fallback is still invalid because it conflicts with
  the explicit project Store/isolation contract and has no visible tier.
- Current unrelated framework edits did not affect the clean CLI topology used
  for A-MEM findings. Canonical framework FileStore behavior is consumed from
  the accepted F-MEM report rather than rereading dirty memory source.

## Handoff

- `A-EVO-01`: consume the missing memory/rule mutation receipt and surface parity;
  do not invent a second evolution store.
- `A-SUB-01`/`A-TSK-05`: require primary and pooled/Subagent execution to share
  the same `MemoryBinding` generation.
- `X-MEM-01`/`Q-STA-01`: make cross-agent write preservation, workspace
  generation, hot projection convergence, and file-read degradation measurable.
- Roadmap order: one Store identity and atomic binding first; one mutation receipt
  and projection reconciler second; remove path reopen/surface refresh duplicates
  third. Keep framework mechanisms and EKO's no-SQLite boundary intact.
