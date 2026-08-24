# X-MEM-01: Instruction, memory, context, and compression conformance

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: framework externally dirty and inspected only through committed `HEAD`; CLI `Cargo.lock` externally dirty and excluded

## Question

Can EKO-specific instruction and memory layers use generic context, Store,
promotion, and compression mechanisms without duplicate persistence, mixed
workspace generations, stale projections, or ambiguous partial commits?

## Scope

- EKO user/repository/project/learned/local instruction precedence, project and
  global hot `MEMORY.md`, workspace/global warm FileStore binding, primary and
  pooled Agent projection refresh, workspace switch/exit, rule/evidence/Dreaming
  mutation paths, and surface parity.
- Framework committed ContextManager projection identity, repeated compression,
  StoreMemoryPromoter, MemoryLayerManager warm/hot promotion and budget demotion,
  observer/change-log flow, and current static tests.
- Definition/duplicate/reachability, source/precedence, immediate refresh,
  repeated compression, workspace switch, promotion/dedup, and SQLite boundary.

## Out Of Scope

- Atomic budget/accounting defects owned by `F-CTX-01`.
- FileStore corruption/multi-instance/namespace defects owned by `F-MEM-01`.
- Compressor ordering, accumulated summary, verifier, attachment, and
  fire-and-forget promoter defects owned by `F-CMP-01`.
- EKO precedence fallback, per-surface refresh omissions, duplicate FileStore,
  and split ReviewIntegration locks owned by `A-MEM-01`, except as inputs to an
  independent cross-layer generation/commit invariant.
- SQLite implementation review. SQLite remains a valid optional framework
  capability; EKO must not and currently does not enable it.
- Source fixes, index/README edits, Cargo/rustc/test/build/dynamic fixtures, or
  network execution.

## Inputs

- Root `AGENTS.md`; exact `TASKS.md` card; shared `REPORTING.md`; Codex README;
  report templates.
- Authorized completed Codex reports `F-CTX-01`, `F-MEM-01`, `F-CMP-01`, and
  `A-MEM-01` only.
- Current CLI source at the pinned revision and framework committed blobs through
  `git show HEAD:<path>`/`git grep HEAD`. No other reviewer output was read.

## Layering Decision

Generic Store contracts, MemoryLayerManager promotion/demotion, typed promotion
receipts, projection identity, compression and summary lifecycle belong in the
framework. EKO owns `.eko`/`~/.eko` source precedence, workspace generation,
which primary/pool Agents share one Store, surface-triggered reconciliation, and
user-visible failure. The adapter should publish one immutable
`MemoryBinding { generation, root, store, layer_manager, projection_snapshot }`
and consume framework mutation receipts; it must not reimplement Store,
compression, or promotion. Duplicate searches covered instruction tiers,
projection markers, FileStore constructors, layer managers, memory promoters,
workspace switch/exit, primary/pool overrides, remember/forget/evidence/rule/
Dreaming entry points, promotion/demotion, compression, and SQLite features.
Remediation should delete path-based Store reopen and surface-specific refresh
sequences after cutover, while retaining all reasonable framework Store options.

## Current Path

### Source and precedence map

| Source | Authority/scope | Projection/consumer | Static conclusion |
|---|---|---|---|
| user instructions | `~/.eko/user.md` | `eko:instruction-context` | explicit first tier |
| repository instructions | resolver from working dir/project root | same instruction marker | explicit after user |
| project instructions | `<root>/.eko/project.md` | same instruction marker | explicit project tier |
| learned rules | `<root>/.eko/learned-rules.md` | same instruction marker | explicit after project |
| local instructions | working-dir local source | same instruction marker | explicit final instruction tier |
| hot memory | `<root>/.eko/MEMORY.md`, but absent project file falls back `~/.eko/MEMORY.md` | `eko:hot-memory-context` | separate marker, incorrect implicit fallback (`A-MEM-01-P1-04`) |
| warm memory | `<root>/.eko/memory/store.json` or global `~/.eko/store.json` | memory tools, recall, compression promoter | correct physical path choice; duplicate handles on switch (`A-MEM-01-P0-01`) |
| compressed history | ContextManager + selected compressor | provider-visible context | projections protected, but summary/reinsertion defects remain `F-CMP-01` |

### Live data flow

```text
workspace/root files
  -> InstructionProvider -> instruction + hot projection markers

warm Store <-> MemoryLayerManager <-> hot MEMORY.md
    ^               ^                 |
    |               |                 +-> EKO projection refresh required
    |               +-> change log + observer (notification, not commit receipt)
    +-> memory tools / evidence / Dreaming / compression MemoryPromoter

AppState workspace switch
  current/CWD
  -> primary + pool working_dir and NEW projections
  -> persistence/conversation/checkpoint stores
  -> primary NEW warm Store/promoter/LayerManager
  -> ReviewIntegration binding
  -> pool independently reopened NEW Store/promoters/LayerManagers
```

Construction and all reviewed mutation paths are production-reachable: Agent
creation installs one Store, layered tools, MemoryPromoter and both projections;
GUI/TUI/CLI remember/forget/rule/evidence commands reach the manager; Dreaming
runs from application lifecycle; GUI workspace IPC reaches switch/exit; normal
ContextManager preparation reaches compression and promotion.

### Mutation and refresh matrix

| Mutation | Durable target | Current projection action | Result |
|---|---|---|---|
| GUI/TUI/CLI explicit remember promoted hot | warm then hot file | calls instruction-only helper; GUI/TUI pool helper also instruction-only | hot projection stale (`A-MEM-01-P1-02`) |
| hot forget | hot file | same wrong helper | deleted fact remains pinned (`A-MEM-01-P1-02`) |
| framework memory tool/evidence accept/undo | warm/hot depending policy | no common EKO receipt/reconciler | entry-point/Agent-age divergence |
| Dreaming hot promotion | warm/hot | primary/pool instruction-only when report count > 0 | hot projection stale |
| CLI rule promotion | learned-rules + memory marker + log | primary instruction projection only | pool stale (`A-MEM-01-P1-05`) |
| GUI rule promotion | same | primary + pool instruction helper | correct domain, but mutation can partially commit before error |
| compression eviction | warm Store through StoreMemoryPromoter | no EKO projection needed while warm; no durable receipt | framework metrics can claim unverified promotion (`F-CMP-01-P1-05`) |
| workspace switch | every source/binding | projections publish before Store/manager binding | mixed workspace generation, P1-01 |

### Repeated compression matrix

- Framework marker replacement keeps one instruction and one hot projection per
  marker during ordinary refresh; both are protected from compression.
- Protected reinsertion can move a projection across semantic role regions
  (`F-CMP-01-P1-02`). Summary/IncrementalSummary preserve earlier system
  summaries and append a new one (`F-CMP-01-P1-03`).
- ContextManager installs StoreMemoryPromoter against the active Store; async
  `install_memory_store` correctly rewires it on one Agent. The cross-workspace
  problem is publication order and shared generation, not an absent setter.
- Promotion returns no durable result for compression evictions and application
  hot/rule promotion has no common receipt, so Store state and projected model
  state cannot be reconciled to one committed generation.

## Findings

### X-MEM-01-P1-01: Workspace switch can publish new projections while memory tools and compression still target the old workspace

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:844`; `echo-agent-cli/echo-agent-app-core/src/state.rs:845`; `echo-agent-cli/echo-agent-app-core/src/state.rs:872`; `echo-agent-cli/echo-agent-app-core/src/state.rs:883`; `echo-agent-cli/echo-agent-app-core/src/state.rs:890`; `echo-agent-cli/echo-agent-app-core/src/state.rs:941`; `echo-agent-cli/echo-agent-app-core/src/state.rs:969`; `echo-agent-cli/echo-agent-app-core/src/state.rs:1013`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:534`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:553`
- Reachability: GUI workspace IPC calls `AppState::switch_workspace`; the method
  immediately changes current/CWD, then primary and pooled working directories
  call `refresh_dynamic_context`, which replaces both instruction/hot markers.
  Only later does it install the new warm Store, MemoryPromoter, LayerManager,
  ReviewIntegration binding, and pool Store override.
- Expected invariant: one request observes either the complete old memory scope
  or complete new `MemoryBinding` generation; workspace switching fences active
  requests and publishes projections, Store, promoter, manager, evidence path,
  and future-Agent override atomically.
- Observed behavior: new-workspace projections are visible before the matching
  Store/promoter/manager. Rebinding is sequential and has no generation token,
  request safe point, rollback, or active-turn gate. Pool handles repeat the
  same ordering. `A-MEM-01-P0-01/P1-03` further establish that final binding can
  use duplicate Store handles and a cross-workspace ReviewIntegration pair.
- Impact: a turn racing switch can reason under workspace B instructions/hot
  facts while recalling from or promoting compression facts into workspace A;
  it can persist cross-project memory even if the final switch later succeeds.
- Root cause: workspace resources are mutated field-by-field and immediately
  visible rather than staged as one immutable application generation.
- Direction: construct one EKO `MemoryBinding` from the new root and exact shared
  Store, prepare all primary/pool/future-Agent consumers, enter a request safe
  point, then publish once. Make switch failure preserve the previous binding.
  Delete path-based pool reopening and early `refresh_dynamic_context` once the
  binding reconciler owns publication; generic Store/compression stays framework.
- Regression validation: pause before/after every current switch step while a
  primary and pooled turn recalls and compresses; assert every read/write/projection
  is entirely A or entirely B and failed switch leaves A unchanged.
- Validation reports: [V02](../validations/X-MEM-01/V02-01.md), [V05](../validations/X-MEM-01/V05-01.md), [V08](../validations/X-MEM-01/V08-01.md), [V09](../validations/X-MEM-01/V09-01.md), [V10](../validations/X-MEM-01/V10-01.md)

### X-MEM-01-P1-02: Promotion can durably change memory or rules, return failure, and suppress projection reconciliation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/src/evolution/layer.rs:1109`; `echo-agent/src/evolution/layer.rs:1114`; `echo-agent/src/evolution/layer.rs:1121`; `echo-agent/src/evolution/layer.rs:1123`; `echo-agent/src/evolution/layer.rs:1132`; `echo-agent/src/evolution/layer.rs:1136`; `echo-agent/src/evolution/layer.rs:711`; `echo-agent/src/evolution/layer.rs:773`; `echo-agent/src/evolution/layer.rs:801`; `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs:195`; `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs:220`; `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs:224`; `echo-agent-cli/echo-agent-app-core/src/evolution/rule_promoter.rs:252`; `echo-agent-cli/src/tauri/commands/memory.rs:126`; `echo-agent-cli/src/tauri/commands/memory.rs:127`
- Reachability: GUI/TUI/CLI remember can trigger framework warm-to-hot promotion;
  Dreaming calls the same manager; GUI/CLI users can promote a memory to a rule.
  Surfaces refresh projections only in the final `Ok` branch.
- Expected invariant: promotion returns a typed, idempotent durable receipt
  describing committed source/destination, audit/observer outcome and required
  projection generation; an error either means no durable change or explicitly
  reports partial commit so reconciliation/retry is safe.
- Observed behavior: warm-to-hot writes hot, deletes warm, logs and notifies,
  then calls hot-budget enforcement. A later warm write/log/hot commit failure
  returns `Err` after the original promotion committed, so the surface skips
  projection refresh. Rule promotion first writes learned-rules, then marks the
  Store entry, then records change log; either later failure reports failure
  after the rule is live. Retrying before the marker committed can append the
  same rule again. Neither API returns committed generation or reconciliation
  work.
- Impact: the next model request can retain stale hot/rule context despite a
  durable promotion, while the user is told it failed; retry can duplicate rules
  or produce divergent hot/warm/audit facts across Agents.
- Root cause: multi-authority promotion uses a single `Result` as both mutation
  outcome and transaction result, with side effects committed before all fallible
  stages and projection refresh outside the authority.
- Direction: make framework MemoryLayerManager promotion idempotent and return a
  typed receipt containing committed layer changes, budget demotions, and
  observer/audit status. EKO rule promotion should similarly use a stable
  promotion ID and recoverable staged manifest. One EKO reconciler consumes the
  receipt for primary/pool projections even after partial failure. Delete
  surface `if Ok { refresh }` sequences after cutover; do not create a second
  Store or move EKO rule-file policy into framework.
- Regression validation: inject failure after every hot/warm/audit/rule/marker
  write, retry twice, and assert exactly one rule/fact, truthful receipt/status,
  and identical primary/existing/future pool projections.
- Validation reports: [V03](../validations/X-MEM-01/V03-01.md), [V04](../validations/X-MEM-01/V04-01.md), [V06](../validations/X-MEM-01/V06-01.md), [V08](../validations/X-MEM-01/V08-01.md), [V09](../validations/X-MEM-01/V09-01.md), [V10](../validations/X-MEM-01/V10-01.md)

## Positive Conclusions

- EKO uses distinct instruction and hot-memory projection markers and the
  framework replacement API prevents ordinary duplicate copies per marker.
- Workspace construction selects project-local hot/warm paths; the defect is
  absent-project fallback and publication/binding consistency, not a need for a
  new persistence implementation.
- Async framework `install_memory_store` rewires both memory tools and the
  compression MemoryPromoter to the supplied Store for one Agent.
- Warm-to-hot and hot-to-warm methods write the destination before removing the
  source, reducing loss on early failure. The finding concerns late failure,
  receipt semantics, and projection convergence.
- CLI/app-core manifests use `default-features = false` and do not enable
  `sqlite`; no EKO SQLite store construction was found. Framework SQLite remains
  a legitimate optional capability and is not a deletion target.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Inputs, commits, dependency and dirty-source isolation | yes | passed | [V00](../validations/X-MEM-01/V00-01.md) |
| V01 | Source, scope, and precedence map | yes | failed/deduplicated | [V01](../validations/X-MEM-01/V01-01.md) |
| V02 | Definition, registration, and production reachability | yes | passed | [V02](../validations/X-MEM-01/V02-01.md) |
| V03 | Immediate refresh and all-consumer mutation matrix | yes | failed/deduplicated | [V03](../validations/X-MEM-01/V03-01.md) |
| V04 | Repeated compression, projection, summary, promotion matrix | yes | failed/deduplicated | [V04](../validations/X-MEM-01/V04-01.md) |
| V05 | Workspace switch generation/state publication | yes | failed/finding | [V05](../validations/X-MEM-01/V05-01.md) |
| V06 | Promotion partial commit, retry, and projection receipt | yes | failed/finding | [V06](../validations/X-MEM-01/V06-01.md) |
| V07 | EKO no-SQLite/framework optional-SQLite boundary | yes | passed | [V07](../validations/X-MEM-01/V07-01.md) |
| V08 | Existing test inventory and missing regressions | yes | failed/gaps | [V08](../validations/X-MEM-01/V08-01.md) |
| V09 | Dependency ownership and historical classification | yes | passed | [V09](../validations/X-MEM-01/V09-01.md) |
| V10 | Dynamic refresh/compression/switch/promotion fixtures | future | not_run | [V10](../validations/X-MEM-01/V10-01.md) |
| V11 | Exact links, headers, IDs, isolation, and source state | yes | passed | [V11](../validations/X-MEM-01/V11-01.md) |
| V30 | Primary committed-source acceptance | yes | passed | [V30](../validations/X-MEM-01/V30-01.md) |

## Historical Claim Status

| Dependency claim | Classification | Current evidence |
|---|---|---|
| `F-CTX-01-P1-05`: compression lacks within-budget postcondition | current, orthogonal | [V04](../validations/X-MEM-01/V04-01.md) |
| `F-CTX-01-P2-08`: projection identity is content-forgeable | current; marker replacement still positive | [V04](../validations/X-MEM-01/V04-01.md) |
| `F-MEM-01-P0-02`: distinct FileStore handles lose updates | current and activated by switch topology | [V05](../validations/X-MEM-01/V05-01.md) |
| `F-CMP-01-P1-02/P1-03/P1-05`: projection reorder, summary accumulation, unacknowledged promotion | current | [V04](../validations/X-MEM-01/V04-01.md) |
| `A-MEM-01-P0-01/P1-03`: split Store/binding after switch | current | [V05](../validations/X-MEM-01/V05-01.md) |
| `A-MEM-01-P1-02/P1-05`: hot/rule mutation refresh gaps | current | [V03](../validations/X-MEM-01/V03-01.md) |
| `A-MEM-01-P1-04/P2-06`: global fallback and read-error-as-absence | current | [V01](../validations/X-MEM-01/V01-01.md) |

## Coverage And Uncertainty

- Pure static review only. No Cargo, rustc, frontend test/build, dynamic fixture,
  process pause/kill, browser, or network command ran. V10 records future
  execution evidence and does not block source-conclusive findings.
- Framework was read only from committed blobs because its live worktree was
  externally dirty. CLI `Cargo.lock` was excluded from all inspection.
- Whether user-global memory should be an explicit independent tier is a product
  decision; the current invisible fallback still violates the current scope map.
- Late promotion failures are source-conclusive; exact recovery UX and receipt
  schema belong to the iteration design, not this review.

## Handoff

- Primary reviewer should independently verify V11, especially switch ordering,
  active-turn/generation absence, warm-to-hot late failures, rule partial commit,
  exact report links, and manifest feature boundary before accepting.
- Roadmap order: atomic EKO MemoryBinding generation first; framework typed
  promotion receipt/idempotence and EKO rule manifest second; one all-Agent
  projection reconciler third; delete path reopen and surface refresh duplicates.
- Preserve `F-*`/`A-MEM-01` atomic ownership and merge their fixes into these
  cross-layer milestones rather than creating parallel stores or contexts.
- This report becomes stale if workspace switch/exit ordering, Agent/Pool Store
  installation, projection refresh, MemoryLayerManager promotion/budget handling,
  RulePromoter ordering, or CLI feature lists change.
