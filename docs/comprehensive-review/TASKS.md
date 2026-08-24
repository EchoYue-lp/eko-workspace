# Atomic Review Task Catalog

This catalog is the shared execution queue for three independent reviews. Each
reviewer executes the task IDs independently and stores evidence in its own
reviewer directory. Assign one task ID per fresh context unless an explicitly
listed dependency is strongly coupled to the same validation failure.

## How To Execute One Task

Use this request in a fresh task:

```text
Execute comprehensive review task <TASK-ID> from
docs/comprehensive-review/TASKS.md. Follow AGENTS.md and
docs/comprehensive-review/REPORTING.md. This is the Codex review track: create
the task report under docs/comprehensive-review/codex/reports/tasks and one
separate report for every validation execution under
docs/comprehensive-review/codex/reports/validations. Update only the Codex
review index, and do not implement fixes.
```

The assigned reviewer may split a task before reviewing if its source slice has
grown beyond the context limits in `README.md`. A split updates this catalog and
marks the original task `superseded`; it does not silently narrow scope.

## Dependency Rules

- `B-*` tasks establish current facts and external baselines.
- `F-*` and `A-*` tasks can run after their named baseline dependencies.
- `X-*` tasks consume completed framework and application reports; they do not
  reread whole subsystems.
- `Q-*` tasks execute broader static/dynamic validations after relevant review
  reports identify the invariants and fixtures to exercise.
- `S-*` tasks synthesize reports. They never substitute a new shallow code scan
  for missing atomic reviews.

Status is one of `pending`, `in_progress`, `needs_evidence`, `blocked`,
`complete`, or `superseded`.

## Phase B: Baseline And Architecture

### B-BASE-01 - Repository and build topology

- Status: `pending`
- Question: What packages, workspace members, targets, features, optional
  dependencies, and cross-repository path dependencies exist now?
- Primary paths: both `Cargo.toml` trees, `Cargo.lock`, frontend `package.json`,
  build scripts, `.github/workflows`.
- Dependencies: none.
- Required validations: manifest/member inventory; feature-to-dependency graph;
  target/required-feature inventory; CI-versus-AGENTS gate comparison. Each is a
  separate validation report.

### B-ARCH-01 - Framework crate architecture

- Status: `pending`
- Question: Are the eight `echo-agent` workspace members layered coherently,
  without reverse dependencies or facade leakage?
- Primary paths: every crate `lib.rs`, manifests, root re-exports, architecture
  READMEs.
- Dependencies: `B-BASE-01`.
- Required validations: crate dependency graph; public facade mapping; cycle or
  misplaced-type search; current documentation comparison.

### B-PATH-01 - EKO entry-point and composition inventory

- Status: `pending`
- Question: Which startup constructors and live entry points assemble TUI,
  GUI, CLI, channel, cron, and background capabilities?
- Primary paths: `echo-agent-cli/src/main.rs`, `src/cli`, `src/tui`, `src/tauri`,
  `src-tauri`, app-core `runtime.rs`, `infra.rs`, `state.rs`.
- Dependencies: `B-BASE-01`.
- Required validations: entry-point call graph; composition-root inventory;
  feature-gated reachability; mode-to-service matrix.

### B-DOC-01 - Historical audit and design drift index

- Status: `pending`
- Question: Which existing audit/plan claims still point at current code and
  which need targeted revalidation?
- Primary paths: `echo-agent/AUDIT_REPORT.md`, both master plans, architecture
  and audit documents under both `docs` trees.
- Dependencies: `B-ARCH-01`, `B-PATH-01`.
- Required validations: document-to-symbol link check; completed-milestone code
  anchor sampling; obsolete path/term search; unresolved historical-finding
  index. Do not rereview the code behind every claim in this task.

### B-REF-01 - Mature implementation reference matrix

- Status: `complete`
- Question: What current cross-system patterns should constrain architecture,
  state, Plan, Subagent, event, permission, skill/plugin, and recovery findings?
- Primary sources: official documentation and first-party repositories for
  Claude Code, OpenAI Codex, Cursor, Devin, Temporal, and other directly
  relevant systems.
- Dependencies: none.
- Required validations: one immutable validation report per system/topic
  lookup, recording source URL, access date, exact supported claim, and limits;
  one cross-system convergence report. Prefer primary sources and state when a
  system does not publicly document a behavior.

## Phase F: Independent Framework Review

### F-CORE-01 - Core identities, errors, and event envelope

- Status: `pending`
- Question: Are run/turn/message/tool/event identities and error semantics
  stable, typed, and sufficient for independent consumers?
- Primary paths: `echo-core/src/agent`, `error.rs`, `event_envelope.rs`, root
  `src/error.rs`, `src/event_bus.rs`.
- Dependencies: `B-ARCH-01`.
- Validations: type/variant inventory; producer-consumer reachability; identity
  collision/ordering inspection; serialization round-trip tests.

### F-API-01 - Public facade and documentation contract

- Status: `pending`
- Question: Do root re-exports, crate-level APIs, examples, and docs expose one
  coherent framework rather than accidental internals?
- Primary paths: all `lib.rs`, root facade modules, README and API docs.
- Dependencies: `B-ARCH-01`, `B-DOC-01`.
- Validations: public re-export map; duplicate public concept search; doctest and
  example sampling; feature/documentation consistency.

### F-FEAT-01 - Feature topology and isolation

- Status: `pending`
- Question: Does each feature enable exactly its required code and dependencies,
  including no-default and standalone feature use?
- Primary paths: manifests and all `#[cfg]` declarations.
- Dependencies: `B-BASE-01`.
- Validations: cfg-to-feature search; unused/empty feature classification;
  optional dependency leakage inspection; targeted standalone compile reports.

### F-REL-01 - Retry, budgets, circuit breakers, and utility invariants

- Status: `pending`
- Question: Are generic retry/backoff, budget arithmetic, circuit breaker,
  hashing, time, and JSON parsing primitives deterministic and safe under
  overflow, cancellation, clock, and malformed-input edges?
- Primary paths: `echo-core/src/retry.rs`, `budget.rs`, `circuit_breaker.rs`,
  `utils`, root retry adapters and shared time helpers.
- Dependencies: `F-CORE-01`.
- Validations: caller/duplication map; transition and arithmetic tables;
  cancellation/time edge cases; malformed JSON and property-style fixtures.

### F-MAC-01 - Procedural macro contract

- Status: `pending`
- Question: Do derive/attribute macros generate Tool and Agent code that obeys
  public schemas, error handling, generics, hygiene, and feature boundaries?
- Primary paths: `echo-macros`, root macro facade, compile-test fixtures.
- Dependencies: `F-API-01`, `F-EXT-01`.
- Validations: expansion/API mapping; invalid-input diagnostics; generic/rename/
  crate-path fixtures; compile-pass and compile-fail reports.

### F-LLM-01 - Provider-neutral LLM contract

- Status: `pending`
- Question: Can provider implementations preserve messages, tools, thinking,
  usage, caching, streaming, cancellation, and errors without semantic loss?
- Primary paths: `echo-core/src/llm`, `echo-integration/src/providers/traits.rs`,
  provider configuration types.
- Dependencies: `F-CORE-01`.
- Validations: field/variant matrix; streaming-neutrality trace; usage/cache
  authority check; malformed provider response tests.

### F-LLM-02 - OpenAI provider adapter

- Status: `pending`
- Question: Does the OpenAI adapter faithfully implement the neutral contract
  for request construction, deltas, tool calls, usage, and failures?
- Primary paths: `echo-integration/src/providers/openai.rs`, shared adapter/client
  code and tests.
- Dependencies: `F-LLM-01`.
- Validations: request field mapping; streamed/non-streamed response mapping;
  tool-call assembly edge cases; protocol fixture tests.

### F-LLM-03 - Anthropic provider and prompt cache adapter

- Status: `pending`
- Question: Does the Anthropic adapter preserve the same contract, including
  thinking blocks and cache-control behavior?
- Primary paths: `anthropic.rs`, `anthropic_cache.rs`, thinking translation.
- Dependencies: `F-LLM-01`.
- Validations: request/response mapping; interleaved content blocks; cache usage
  accounting; protocol fixture tests.

### F-RCT-01 - ReAct construction and canonical prompt assembly

- Status: `pending`
- Question: Does builder/config assembly produce a deterministic Agent with one
  tool registry, correct instructions, budgets, hooks, and project rules?
- Primary paths: `src/agent/react/builder.rs`, `src/agent/config.rs`, React
  capabilities/subsystems, core prompt templates.
- Dependencies: `F-CORE-01`, `F-API-01`.
- Validations: builder option-to-runtime map; duplicate registry search; prompt
  section/cardinality fixtures; disabled-feature construction tests.

### F-RCT-02 - Non-streaming ReAct loop

- Status: `pending`
- Question: Does one non-streaming turn transition correctly through thinking,
  tool batches, stopping, errors, limits, and final response?
- Primary paths: React loop, pipeline, phases, run context, non-stream execution.
- Dependencies: `F-RCT-01`, `F-LLM-01`.
- Validations: state transition trace; terminal ownership search; max-step/loop
  detection cases; mocked end-to-end turn tests.

### F-RCT-03 - Streaming ReAct event flow

- Status: `pending`
- Question: Are streaming deltas and terminal events ordered, lossless, bounded,
  and semantically equivalent to non-streaming execution?
- Primary paths: `run/stream_channel.rs`, streaming pipeline/loop paths, event
  bus integration.
- Dependencies: `F-RCT-02`.
- Validations: producer-consumer event sequence; channel close/backpressure;
  duplicate terminal tests; streaming/non-streaming conformance fixture.

### F-RCT-04 - Tool batch execution

- Status: `pending`
- Question: Are tool validation, concurrency, timeout, cancellation, partial
  output, retry, and result insertion correct for a tool batch?
- Primary paths: React tool execution phases/subsystems, `echo-execution/src/tools.rs`,
  tool context.
- Dependencies: `F-RCT-02`, `F-EXT-01`.
- Validations: tool-call/result pairing; concurrent ordering; timeout versus
  cancel; partial-side-effect and oversized-result fixtures.

### F-RCT-05 - Steer, interrupt, snapshot, and resume

- Status: `pending`
- Question: Can a running or interrupted Agent resume without replaying completed
  side effects or losing canonical context?
- Primary paths: `src/agent/steer.rs`, `snapshot.rs`, handle/turn, React resume
  and approval paths.
- Dependencies: `F-RCT-02`, `F-RCT-04`, `F-MEM-01`.
- Validations: snapshot field round-trip; completed-tool skip; interrupt at each
  safe point; corrupted/incomplete snapshot handling.

### F-CTX-01 - Context selection and budget accounting

- Status: `pending`
- Question: Are canonical instructions, history, tools, attachments, memory, and
  reserved output selected deterministically within model limits?
- Primary paths: `src/context`, tokenizers/budgets, React context assembly.
- Dependencies: `F-RCT-01`, `F-LLM-01`.
- Validations: budget arithmetic/overflow; protected-content survival; provider
  window mapping; large-schema and multilingual fixtures.

### F-MEM-01 - General memory and conversation stores

- Status: `pending`
- Question: Are the Store/ConversationStore contracts and in-memory/file
  implementations durable, atomic, path-safe, and semantically aligned?
- Primary paths: `echo-core/src/memory`, `echo-state/src/memory` excluding SQLite.
- Dependencies: `F-CORE-01`.
- Validations: trait implementation matrix; corrupt/truncated file handling;
  path-safe ID and atomic-write inspection; round-trip/search/pagination tests.

### F-MEM-02 - SQLite framework capabilities

- Status: `pending`
- Question: Are `SqliteStore` and `SqliteConversationStore` valid independent
  framework options with correct concurrency, schema, search, and feature gates?
- Primary paths: `echo-state/src/memory/sqlite_*`, root re-exports/examples.
- Dependencies: `F-MEM-01`, `F-FEAT-01`.
- Validations: public-use justification; feature isolation; database error and
  concurrent access tests; semantic-search numerical edge cases. CLI usage is
  explicitly not a deletion criterion.

### F-CMP-01 - Compression correctness

- Status: `pending`
- Question: Do compressors preserve protocol pairs, instructions, active tasks,
  recent evidence, and recovery facts under repeated compression?
- Primary paths: `echo-state/src/compression`, root compression adapters.
- Dependencies: `F-CTX-01`, `F-MEM-01`.
- Validations: compressor/invariant matrix; tool-pair preservation; repeated
  compression stability; multilingual/large-context fixtures.

### F-TSK-01 - Canonical task model and revision tools

- Status: `pending`
- Question: Is `TaskSpec + TaskExecution + TaskStatus` the sole generic dynamic
  task model with coherent revisioned `task_create/update/list` semantics?
- Primary paths: orchestration task/task_tools/revisioned/store modules and root
  task exports.
- Dependencies: `F-CORE-01`, `B-REF-01`.
- Validations: duplicate task/plan/todo model search; transition/revision table;
  tool schema round-trip; stale update tests.

### F-TSK-02 - DAG validation and dependency analysis

- Status: `pending`
- Question: Is there one structural validator and one dependency analysis for
  cycles, missing nodes, readiness, skip, and blocked propagation?
- Primary paths: planning validator/plan spec, task DAG/manager/scheduler.
- Dependencies: `F-TSK-01`.
- Validations: validator/DFS duplicate search; cycle/missing/self-dependency
  fixtures; frontier determinism; status-independent structural checks.

### F-TSK-03 - Runtime DAG execution and claims

- Status: `pending`
- Question: Does `RuntimeDagExecutor` correctly own safe points, bounded
  Subagent waves, claims, retries, cancellation, external polling, and stalls?
- Primary paths: `tasks/runtime_executor.rs`, runtime/executor/hooks/verifier.
- Dependencies: `F-TSK-02`, `F-SUB-02`.
- Validations: authority call graph; stale claim/attempt scenarios; cancellation
  and failure propagation; revision reload and stall fixtures.

### F-SUB-01 - Subagent definitions, registry, and prompt context

- Status: `pending`
- Question: Are Subagent identity, catalog snapshot, role prompts, history
  inheritance, tool/permission selection, and results coherent?
- Primary paths: `src/agent/subagent/types.rs`, registry, prompt/context compiler,
  builtin dispatch tools.
- Dependencies: `F-RCT-01`, `F-CORE-01`.
- Validations: definition-to-registration trace; catalog route validation;
  prompt envelope/cardinality; result protocol round-trip.

### F-SUB-02 - Subagent execution modes and teams

- Status: `pending`
- Question: Do Sync, Fork, Teammate, Team, manager, timeout, cancellation, and
  isolation modes share one lifecycle without detached execution?
- Primary paths: Subagent executor, team modules, agent box/manager, worktree and
  dispatch tooling.
- Dependencies: `F-SUB-01`, `F-RCT-04`.
- Validations: mode lifecycle matrix; parent cancellation propagation; timeout
  ownership; team partial-failure and cleanup fixtures.

### F-MAG-01 - Handoff, topology, and multi-agent coordination

- Status: `pending`
- Question: Are handoff and topology APIs coherent with the Subagent-only model,
  or do they create overlapping identity, routing, ownership, or lifecycle
  authorities?
- Primary paths: root `handoff`, `topology.rs`, related tools/examples and
  multi-agent documentation.
- Dependencies: `F-SUB-01`, `F-SUB-02`.
- Validations: concept/identity overlap search; registration and routing trace;
  handoff result/context preservation; topology failure/cancel fixtures.

### F-HITL-01 - Human loop and permission model

- Status: `pending`
- Question: Does the framework provide generic automated-action approval and
  protected-path primitives without imposing EKO interaction policy?
- Primary paths: `echo-orchestration/src/human_loop`, core tool permission,
  React approval integration.
- Dependencies: `F-RCT-04`, `B-REF-01`.
- Validations: decision/policy/provider mapping; timeout/default behavior;
  approval cache identity; local-versus-generic boundary classification.

### F-EXT-01 - Tool contract, registry, schema, and artifacts

- Status: `pending`
- Question: Is the generic Tool contract typed, cancellable, paginated, and
  capable of bounded model output plus complete artifacts?
- Primary paths: core tools, execution tools/risk, root tool registry and builtin
  tooling.
- Dependencies: `F-CORE-01`.
- Validations: schema/execute contract; name collision/registration; cursor and
  artifact integrity; invalid argument/error classification fixtures.

### F-EXT-02 - Shell, file, code, and Git tools

- Status: `pending`
- Question: Are common local developer tools correct for paths, UTF-8, atomic
  writes, diff application, cancellation, process cleanup, and isolation?
- Primary paths: `echo-tools/src/shell.rs`, `code.rs`, `files`, `git*`, sandbox
  adapters used by these tools.
- Dependencies: `F-EXT-01`, `F-SEC-01`.
- Validations: path/Unicode edge cases; process-tree cancellation; conflicting
  edit/diff checks; worktree and partial-side-effect scenarios.

### F-EXT-03 - Data, research, media, database, and Web tools

- Status: `pending`
- Question: Are domain tool contracts honest about validation, provenance,
  pagination, numerical limits, network failures, and artifact output?
- Primary paths: `echo-tools` data/statistics/research/media/web/database/rag/
  chart/document modules.
- Dependencies: `F-EXT-01`.
- Validations: capability/feature map; representative invalid/empty/large input
  fixtures; provenance/result schema checks; live-network tests only when safe
  and separately reported.

### F-SKL-01 - Skill loading and execution

- Status: `pending`
- Question: Are skill discovery, frontmatter, dependency probing, prompt/script
  execution, source identity, hooks, and reload behavior deterministic?
- Primary paths: `echo-execution/src/skills`, root `src/skills`.
- Dependencies: `F-EXT-01`, `F-RCT-01`.
- Validations: discovery precedence; malformed/frontmatter/path cases; tool name
  collision; reload/unload and script cancellation fixtures.

### F-PLG-01 - Plugin manifest, registry, and lifecycle

- Status: `pending`
- Question: Does the plugin framework resolve dependencies, component ownership,
  activation, replacement, unloading, and rollback without leaked registrations?
- Primary paths: `echo-core/src/plugin`, root plugin facade, generic hook types.
- Dependencies: `F-SKL-01`, `B-REF-01`.
- Validations: manifest/path/dependency graph; source-scoped registration;
  activation failure rollback; reload/unload lifecycle fixtures.

### F-INT-01 - MCP integration

- Status: `pending`
- Question: Does MCP configuration, client/server transport, tool adaptation,
  cancellation, reconnect, and schema handling preserve framework contracts?
- Primary paths: `echo-integration/src/mcp`, root MCP facade.
- Dependencies: `F-EXT-01`, `F-CORE-01`.
- Validations: transport lifecycle; schema adaptation; malformed/disconnected
  server fixtures; reconnect and cancellation tests.

### F-INT-02 - LSP, channels, and A2A integrations

- Status: `pending`
- Question: Do LSP, IM channel, and A2A adapters isolate external protocols while
  preserving typed internal lifecycle and cleanup?
- Primary paths: integration LSP/channels, root LSP/channels/A2A.
- Dependencies: `F-CORE-01`.
- Validations: one separate report per protocol family for lifecycle,
  malformed input, retry/cancel, and internal naming conversion.

### F-WFL-01 - Workflow and pipeline engine

- Status: `pending`
- Question: Are graph, DAG, sequential/concurrent pipelines, checkpoints, and
  state transitions a coherent generic workflow API distinct from dynamic tasks?
- Primary paths: `echo-orchestration/src/workflow` and workflow exports/examples.
- Dependencies: `F-CORE-01`, `F-TSK-01`.
- Validations: task/workflow semantic overlap search; graph validation; concurrent
  state merge; checkpoint/resume fixtures.

### F-INTENT-01 - Intent classification and supervisory routing

- Status: `pending`
- Question: Are intent classification and trigger supervision generic,
  explainable, bounded, and separate from runtime state authority?
- Primary paths: root `src/intent`, related builder/tool registration and tests.
- Dependencies: `F-RCT-01`, `B-REF-01`.
- Validations: definition/registration/reachability; label/decision contract;
  timeout/fallback behavior; representative routing fixtures.

### F-NBK-01 - Notebook and structured working artifacts

- Status: `pending`
- Question: Is the notebook capability a coherent, reachable framework API with
  stable cell/artifact semantics rather than an isolated or aspirational path?
- Primary paths: root `src/notebook`, related tool registration, exports,
  examples, and docs.
- Dependencies: `F-API-01`, `F-EXT-01`.
- Validations: public/reachability map; persistence/artifact mapping; malformed
  cell and execution-order cases; documentation drift.

### F-OPS-01 - Scheduler, headless mode, tracing, and telemetry

- Status: `pending`
- Question: Are operational adapters bounded, observable, cancellable, and free
  of hidden lifecycle ownership?
- Primary paths: orchestration scheduler, root scheduler/headless/trace/telemetry/
  audit modules.
- Dependencies: `F-CORE-01`, `F-RCT-02`.
- Validations: scheduler trigger lifecycle; headless event contract; trace
  redaction/size; telemetry-disabled behavior.

### F-EVO-01 - Eval, improvement, and evolution framework APIs

- Status: `pending`
- Question: Are eval/improve/evolution capabilities valid optional framework
  APIs with explicit side effects and without coupling to EKO product policy?
- Primary paths: root `eval`, `improve`, `evolution`, related features/examples.
- Dependencies: `F-FEAT-01`, `F-MEM-01`.
- Validations: feature/reachability inventory; API option value independent of
  CLI use; mutation/review boundaries; deterministic fixture tests.

### F-TST-01 - Framework test and mock utilities

- Status: `pending`
- Question: Do public/internal mocks and testing helpers faithfully model real
  streaming, tool, usage, error, cancellation, and ordering contracts?
- Primary paths: root `src/testing`, mock modules, shared test fixtures and the
  `testing` feature.
- Dependencies: `F-LLM-01`, `F-RCT-03`, `F-EXT-01`.
- Validations: mock-versus-provider contract matrix; scripted ordering/error
  fixtures; testing feature isolation; production tests relying on unrealistic
  behavior.

### F-SEC-01 - Guards, sandbox, secrets, and panic safety

- Status: `pending`
- Question: Do generic local execution protections prevent framework bugs, data
  loss, secret logging, and sandbox escape without product-specific overreach?
- Primary paths: core guards/sandbox, execution sandbox, root security/guard,
  tool security helpers.
- Dependencies: `B-REF-01`, `F-CORE-01`.
- Validations: threat-boundary classification; secret/log redaction; sandbox
  fallback behavior; unsafe panic/UTF-8/path traversal cases.

## Phase A: EKO Application Review

### A-BOOT-01 - Application composition and startup lifecycle

- Status: `pending`
- Question: Does each EKO entry point construct the same core services exactly
  once with consistent config, working directory, shutdown, and reload behavior?
- Primary paths: app-core `runtime.rs`, `infra.rs`, `state.rs`, root `main.rs`,
  Tauri state construction.
- Dependencies: `B-PATH-01`.
- Validations: constructor/service ownership map; entry-point option diff;
  startup failure rollback; shutdown/resource cleanup trace.

### A-CFG-01 - Configuration, providers, and workspace lifecycle

- Status: `pending`
- Question: Are global/project config discovery, provider selection, workspace
  switching, validation, and hot-reload scope coherent?
- Primary paths: app-core config/model/workspace modules, CLI/Tauri config and
  provider commands.
- Dependencies: `A-BOOT-01`.
- Validations: precedence and path map; invalid/partial config fixtures;
  workspace switch state replacement; restart-required versus live reload.

### A-INP-01 - Prepared user turn, attachments, and input artifacts

- Status: `pending`
- Question: Do all entry points normalize user input once while preserving
  Unicode, attachment identity, long text artifacts, cleanup, and display/model
  projections?
- Primary paths: `prepared_turn.rs`, attachments, input artifact handling and
  entry-point adapters.
- Dependencies: `A-BOOT-01`.
- Validations: five-entry field matrix; long/Unicode/empty inputs; attachment
  round-trip; conversation deletion cleanup.

### A-CHAT-01 - Shared chat driver and sinks

- Status: `pending`
- Question: Does `drive_chat` own one application lifecycle while sinks only
  render/transport events?
- Primary paths: `chat_driver.rs`, response types, runtime integration, sink
  implementations.
- Dependencies: `A-INP-01`, `F-RCT-02`, `F-RCT-03`.
- Validations: caller/reachability map; sink responsibility diff; one-terminal
  invariant; stream error/cancel/steer fixtures.

### A-SRF-01 - TUI integration

- Status: `pending`
- Question: Does the TUI expose and correctly render the complete Agent feature
  set rather than a reduced execution path?
- Primary paths: `src/tui` plus TUI providers/sinks in app-core.
- Dependencies: `A-CHAT-01`, `A-TSK-03`, `A-HITL-01`.
- Validations: TUI capability/reducer matrix; task/Subagent/tool/HITL flows;
  resume/attachment/browser/MCP reachability; terminal event fixtures.

### A-SRF-02 - Tauri command and desktop integration

- Status: `pending`
- Question: Are Tauri commands thin, lifecycle-safe adapters with consistent
  state and no duplicate business authority?
- Primary paths: `src/tauri`, `src-tauri`.
- Dependencies: `A-BOOT-01`, `A-CHAT-01`.
- Validations: command-to-service map; state/lock/await inspection; event
  emission contract; window/terminal cleanup scenarios.

### A-SRF-03 - GUI chat and frontend state integration

- Status: `pending`
- Question: Does the React chat surface consume backend facts without inventing
  lifecycle state or dropping late/duplicate events?
- Primary paths: frontend chat components/hooks/stores and API event types.
- Dependencies: `A-SRF-02`, `A-CHAT-01`.
- Validations: backend-to-store flow; reducer monotonicity; reconnect/reload;
  streaming/tool/result rendering fixtures.

### A-SRF-04 - CLI, channels, cron, and background triggers

- Status: `pending`
- Question: Do non-GUI/TUI triggers enter the same core runtime and preserve
  identity, events, memory, tools, cancellation, and terminal semantics?
- Primary paths: `src/cli`, app channel handlers, cron/background launch paths.
- Dependencies: `A-CHAT-01`, `A-TSK-03`.
- Validations: trigger adapter matrix; identity propagation; noninteractive
  event output; cancel/shutdown and recovery fixtures.

### A-STATE-01 - Conversation persistence and restore

- Status: `pending`
- Question: Are file-backed conversations authoritative, atomic, restorable,
  searchable, and cleaned with their dependent artifacts?
- Primary paths: app persistence/conversation restore adapters, Tauri/CLI
  conversation commands, artifact retention.
- Dependencies: `F-MEM-01`, `A-INP-01`.
- Validations: format/authority map; corrupt file behavior; message/tool/thinking
  round-trip; deletion cascade and concurrent write tests.

### A-MEM-01 - Instructions, hot memory, and Dreaming

- Status: `pending`
- Question: Does EKO own only its instruction/memory protocol while projecting
  updates immediately and consistently to primary and pooled Agents?
- Primary paths: instruction provider, memory manager/evolution integration,
  workspace switching and Agent pool refresh paths.
- Dependencies: `A-CFG-01`, `F-CMP-01`, `F-MEM-01`.
- Validations: layer/precedence map; compression survival; refresh triggers;
  duplication/promotion and workspace-switch fixtures.

### A-TSK-01 - TaskRuntime file authorities and typed adapter

- Status: `pending`
- Question: Do plan/events/run-state files have unambiguous authority, and is
  conversion to framework task types thin and lossless?
- Primary paths: task runtime types/file store/store/revisioned adapter/event
  rebuild.
- Dependencies: `F-TSK-01`, `F-TSK-02`.
- Validations: file-authority table; field-by-field round-trip; duplicate model/
  validator search; corrupt and partial state reconstruction.

### A-TSK-02 - EKO task authoring tools

- Status: `pending`
- Question: Are `task_create/update/list` thin product shells over the one
  revisioned graph without independent Todo/Plan CRUD or hidden global state?
- Primary paths: task runtime task tools/planner/service/tool exposure.
- Dependencies: `A-TSK-01`, `F-TSK-01`.
- Validations: registered tool inventory; create/update/list call paths; schema
  parity; forbidden parallel CRUD and global todo search.

### A-TSK-03 - Task execution controller boundary

- Status: `pending`
- Question: Does EKO inject only product policy into `RuntimeDagExecutor`, with
  no second ready-frontier, retry, cancellation, or stall loop?
- Primary paths: task runtime executor/task_execute_tool/controller integration.
- Dependencies: `A-TSK-01`, `F-TSK-03`.
- Validations: framework/application ownership call graph; scheduling-loop
  duplicate search; controller callback responsibility; basic DAG execution.

### A-TSK-04 - Claims, revisions, recovery, and terminal monotonicity

- Status: `pending`
- Question: Can stale revisions/attempts, cancellation, restart, and event replay
  update state only through valid claims without terminal regression?
- Primary paths: task runtime store/ledger/event rebuild/hook dispatcher and
  executor claim paths.
- Dependencies: `A-TSK-03`.
- Validations: claim identity persistence; stale write rejection; event replay
  ordering; crash/restart/cancel/retry scenarios.

### A-TSK-05 - Worktree, file ownership, and merge policy

- Status: `pending`
- Question: Does EKO safely isolate concurrent writers, reuse logical-task
  worktrees, protect user changes, and finalize/merge deterministically?
- Primary paths: task runtime worktree/file shadow/profiles/ownership policies.
- Dependencies: `A-TSK-03`, `F-EXT-02`.
- Validations: ownership conflict analysis; dirty-tree protection; reuse/repair/
  cleanup; merge failure and cancellation fixtures.

### A-TSK-06 - Task review, artifacts, and parent context

- Status: `pending`
- Question: Are complete Subagent results, checks, acceptance, artifacts, and
  bounded parent summaries preserved without leaking thinking protocol?
- Primary paths: task runtime review/compact_context/memory_bridge/result and
  artifact projection paths.
- Dependencies: `A-TSK-04`, `A-STATE-01`.
- Validations: full-result versus summary map; acceptance/check separation;
  artifact retention; restart-equivalent review input fixtures.

### A-SUB-01 - EKO Subagent catalog, pool, and prompt compilation

- Status: `pending`
- Question: Does EKO add domain definitions and product policy while reusing one
  framework Subagent lifecycle and immutable effective catalog?
- Primary paths: `agent_pool.rs`, subagent loader/prompt/definitions/components.
- Dependencies: `F-SUB-01`, `F-SUB-02`, `A-CFG-01`.
- Validations: definition source precedence; default route startup validation;
  prompt cardinality/language; reload and pooled-Agent refresh.

### A-HITL-01 - Multi-surface human interaction policy

- Status: `pending`
- Question: Does EKO arbitrate TUI/GUI/channel approvals within one shared
  deadline without gating direct user interactions as agent automation?
- Primary paths: app-core `hitl`, permission wiring, terminal/file/MCP commands.
- Dependencies: `F-HITL-01`, `A-BOOT-01`.
- Validations: provider arbitration and cancellation; timeout/default behavior;
  direct-user versus Agent action call paths; default permission mode scenarios.

### A-TOOL-01 - Tool exposure, execution, sandbox, and terminal

- Status: `pending`
- Question: Does each Agent/mode expose the intended tools with common error and
  artifact behavior, while keeping interactive terminal separate from Agent
  `run_code` policy?
- Primary paths: app-core tool exposure/execution/infra, Tauri terminal, Agent
  construction paths.
- Dependencies: `F-EXT-01`, `F-EXT-02`, `A-BOOT-01`.
- Validations: per-role/mode registry diff; sandbox probing and no-bare fallback;
  interactive terminal permission path; large output/cancel/error fixtures.

### A-PLG-01 - Skills, plugins, hooks, and reload lifecycle

- Status: `pending`
- Question: Does EKO discovery/activation/reload correctly apply product
  components while framework registrations unload and roll back cleanly?
- Primary paths: plugin runtime/components, skills hub, hook config/dispatcher,
  config watcher, GUI/CLI plugin commands.
- Dependencies: `F-SKL-01`, `F-PLG-01`, `A-CFG-01`.
- Validations: prepare/activate ownership; real component registration; failed
  activation rollback; reload/unload and hook queue flush/shutdown.

### A-INT-01 - Browser, MCP, and LSP application integration

- Status: `pending`
- Question: Are local browser sessions and user-configured MCP/LSP capabilities
  reachable, recoverable, and not blocked by irrelevant permission gates?
- Primary paths: app-core browser/config discovery, Tauri/CLI MCP and LSP paths.
- Dependencies: `F-INT-01`, `F-INT-02`, `A-TOOL-01`.
- Validations: connect/disconnect/reconnect; session cleanup; invalid config
  handling; default-permission interactive use.

### A-FE-01 - Rust/TypeScript API and event type contract

- Status: `pending`
- Question: Do Tauri command DTOs, emitted payloads, TypeScript endpoint types,
  and stores match field-for-field and variant-for-variant?
- Primary paths: Rust response/command/event types; frontend `types`, `api`,
  stores and event hooks.
- Dependencies: `A-SRF-02`, `A-TSK-04`.
- Validations: DTO field matrix; enum/event variant coverage; optional/null
  semantics; generated/fixture serialization tests.

### A-FE-02 - Task, Subagent, and tool projections

- Status: `pending`
- Question: Do frontend projections preserve attempt identity, terminal
  monotonicity, lazy output, results, and Task acceptance distinctions?
- Primary paths: task/subagent/tool stores and task/chat rendering components.
- Dependencies: `A-FE-01`, `A-TSK-06`.
- Validations: reducer identity keys; duplicate/out-of-order events; old attempt
  completion; collapsed/expanded large-output fixtures.

### A-DOM-01 - Data analysis and research workflows

- Status: `pending`
- Question: Are EKO-specific analysis/research policies, provenance, formal
  inference, connectors, workbench state, and artifact export correctly placed
  and reliable?
- Primary paths: app-core analysis/research/connectors/tools, frontend analysis
  and paper workbench.
- Dependencies: `F-EXT-03`, `A-TOOL-01`.
- Validations: exploratory/formal-analysis boundary; provenance/lineage;
  connector failure; artifact/rendering fixtures.

### A-EVO-01 - EKO evolution product scope

- Status: `pending`
- Question: Has EKO kept evolution as explicit diagnostics/review without hidden
  metric loops, automatic semantic mutation, or framework option deletion?
- Primary paths: app-core evolution, CLI/GUI evolution surfaces and runtime hooks.
- Dependencies: `F-EVO-01`, `A-MEM-01`.
- Validations: reachable mutation triggers; user authorization boundaries;
  dead/aspirational path classification; product docs versus code.

### A-OBS-01 - Diagnostics, webhooks, and operational visibility

- Status: `pending`
- Question: Are diagnostics, run context, webhook events, and logs wired to live
  lifecycle facts without globals, secret leakage, or misleading success?
- Primary paths: app-core observability/webhook/state and GUI observability.
- Dependencies: `A-CHAT-01`, `A-TSK-04`, `F-OPS-01`.
- Validations: event-to-emitter reachability; configuration identity; secret and
  content redaction; failure/retry/reporting scenarios.

### A-OUT-01 - Output formats, export, and file delivery

- Status: `pending`
- Question: Do EKO output profiles and Markdown/document/data export paths retain
  complete content, artifact lineage, error causes, and consistent availability
  across surfaces?
- Primary paths: app-core `output`, analysis/research export paths, CLI/Tauri
  file delivery and corresponding frontend consumers.
- Dependencies: `A-STATE-01`, `A-TOOL-01`, `F-EXT-03`.
- Validations: format/renderer registry; large/Unicode content; missing external
  converter behavior; artifact identity and cross-surface delivery fixtures.

### A-PROJ-01 - Project indexing, diff, and coding workspace services

- Status: `pending`
- Question: Are project indexing, diff, coding commands, and workspace state
  derived from current files without stale caches or a second worktree/file
  authority?
- Primary paths: app-core `project`, `diff.rs`, workspace registry, coding CLI
  commands and related GUI panels.
- Dependencies: `A-CFG-01`, `A-TSK-05`, `F-EXT-02`.
- Validations: index lifecycle and invalidation; diff source-of-truth; workspace
  switch behavior; large repository and conflicting file fixtures.

### A-FE-03 - Frontend architecture, performance, and accessibility

- Status: `pending`
- Question: Are frontend components/stores organized around stable domain facts,
  with bounded rendering, cleanup of listeners/timers, accessible interactions,
  and no monolithic accidental state owners?
- Primary paths: frontend layout, shared hooks/stores/components, route/root
  assembly, especially files over 500 lines.
- Dependencies: `A-SRF-03`, `A-FE-01`, `A-FE-02`.
- Validations: store/component dependency map; subscription cleanup; render
  behavior for large chats/tasks; keyboard/focus/label and responsive smoke
  checks.

## Phase X: Cross-Repository Contract Review

### X-BND-01 - Capability placement and duplicate authority map

- Status: `pending`
- Question: Across both repositories, which concepts are correctly framework,
  EKO policy, or thin adapters, and where do semantic duplicates remain?
- Dependencies: all completed `F-*` and `A-*` reports; may run incrementally and
  finalize after both phase syntheses.
- Validations: type/trait/name search; behavior/call-path search; public framework
  option check; adapter-logic/deletion-target matrix.

### X-TSK-01 - Task graph and adapter conformance

- Status: `pending`
- Question: Is there one revisioned TaskRun graph with lossless EKO projection
  and no second validator/executor/store authority?
- Dependencies: `F-TSK-01..03`, `A-TSK-01..06`.
- Validations: field round-trip; authority call graph; forbidden CRUD search;
  shared fixture executed through framework and EKO adapters.

### X-EVT-01 - Event lifecycle conformance

- Status: `pending`
- Question: Do framework events, EKO persistence, Rust surfaces, and TypeScript
  reducers agree on identity, ordering, terminal status, cancel, and timeout?
- Dependencies: `F-CORE-01`, `F-RCT-03`, `A-CHAT-01`, `A-FE-01..02`.
- Validations: producer-to-all-consumer matrix; variant exhaustiveness; recorded
  event replay; duplicate/out-of-order terminal conformance.

### X-SRF-01 - Surface feature parity

- Status: `pending`
- Question: Are GUI, TUI, CLI, channels, cron, and background modes complete
  Agents differing only in trigger and rendering policy?
- Dependencies: `A-SRF-01..04`, `A-TOOL-01`, `A-PLG-01`, `A-INT-01`.
- Validations: capability matrix with definition/registration/reachability;
  common scenario replay; missing event/tool/attachment/HITL paths.

### X-STA-01 - Persistence, recovery, and identity continuity

- Status: `pending`
- Question: Do conversation, snapshot, task, Subagent, artifact, and frontend
  identities survive restart without duplication or stale overwrite?
- Dependencies: `F-RCT-05`, `F-MEM-01`, `A-STATE-01`, `A-TSK-04`, `A-FE-02`.
- Validations: identity table; crash-point recovery matrix; corrupt/partial files;
  retention and deletion cascade.

### X-TOL-01 - Tool error, artifact, and schema conformance

- Status: `pending`
- Question: Does one tool invocation retain the same schema, classification,
  output integrity, artifact metadata, and terminal reason across all layers?
- Dependencies: `F-RCT-04`, `F-EXT-01..03`, `A-TOOL-01`, `A-FE-02`.
- Validations: field mapping; error taxonomy; long-output checksum/cursor;
  invalid/timeout/cancel/partial-side-effect fixtures.

### X-AUT-01 - Permission and local security boundary

- Status: `pending`
- Question: Are automated Agent actions controlled while direct user terminal,
  file picker, MCP configuration, and browser interactions remain usable?
- Dependencies: `F-HITL-01`, `F-SEC-01`, `A-HITL-01`, `A-INT-01`.
- Validations: call-path classification; default/full-auto mode matrix; local
  data-loss/secret protections; over-gating search.

### X-PLG-01 - Skill/plugin/hook lifecycle conformance

- Status: `pending`
- Question: Are framework lifecycle primitives and EKO activation policy joined
  through reversible, source-scoped, failure-safe adapters?
- Dependencies: `F-SKL-01`, `F-PLG-01`, `A-PLG-01`.
- Validations: component ownership map; load/reload/unload trace; failure rollback;
  stale tool/Subagent/hook registration search.

### X-MEM-01 - Instruction, memory, context, and compression conformance

- Status: `pending`
- Question: Can EKO-specific instruction/memory layers use generic context and
  compression without duplicate persistence or lost updates?
- Dependencies: `F-CTX-01`, `F-MEM-01`, `F-CMP-01`, `A-MEM-01`.
- Validations: source/precedence map; immediate refresh; repeated compression;
  workspace switch and duplicate promotion fixtures.

### X-INV-01 - Repository invariant audit

- Status: `pending`
- Question: Do both repositories obey Subagent-only terminology, CLI no-SQLite,
  no parallel task CRUD, panic safety, UTF-8 safety, and relative path rules?
- Dependencies: `B-BASE-01` and relevant completed subsystem reports.
- Validations: one separate report per invariant search; every match classified
  as violation, test-only, third-party wire exception, or false positive.

## Phase Q: Broad Static And Dynamic Validation

### Q-FW-01 - Framework submission gate

- Status: `pending`
- Question: Does current `echo-agent` pass its mandatory submission gate?
- Dependencies: framework atomic reviews complete enough to interpret failures.
- Validations: separate reports for fmt check, all-feature Clippy, panic-safety
  Clippy, all-target/all-feature tests, and no-default library check. Run `cargo
  fmt --all` only during implementation, not this read-only review.

### Q-FW-02 - Framework feature, examples, and docs matrix

- Status: `pending`
- Question: Do public optional capabilities compile and demonstrate their stated
  contracts independently?
- Dependencies: `F-FEAT-01`, `F-API-01`.
- Validations: one report per independent feature command; examples grouped only
  by identical required features; doctest/document link validation separately.

### Q-CLI-01 - EKO Rust submission gate

- Status: `pending`
- Question: Does current `echo-agent-cli` Rust workspace pass its mandatory gate
  without enabling SQLite?
- Dependencies: application atomic reviews complete enough to interpret failures.
- Validations: separate reports for fmt check, all-feature Clippy, panic-safety
  Clippy, all-feature tests, and app-core no-default check; dependency tree
  SQLite absence is another validation.

### Q-GUI-01 - Tauri/GUI Rust matrix

- Status: `pending`
- Question: Does the GUI target compile and test under its conditional feature
  matrix?
- Dependencies: `A-SRF-02`, `A-FE-01`.
- Validations: GUI bin check and GUI tests are separate reports; environment or
  system-dependency failure is recorded, not silently skipped.

### Q-WEB-01 - Frontend submission gate

- Status: `pending`
- Question: Does the frontend pass formatting, unit/integration tests, and
  production build?
- Dependencies: `A-SRF-03`, `A-FE-01..02`.
- Validations: separate reports for Prettier check, tests, and build.

### Q-STA-01 - Static safety and dependency audit

- Status: `pending`
- Question: What panic, direct-index, UTF-8 slicing, overflow, unsafe, dead-code,
  duplicate dependency, and oversized-module risks remain?
- Dependencies: `B-BASE-01`.
- Validations: one report per rule family; matches classified with reachability
  and test/production context rather than counted blindly.

### Q-TST-01 - Test suite credibility and coverage map

- Status: `pending`
- Question: Which production invariants have meaningful tests, which tests only
  restate implementations, and where do mocks hide integration failures?
- Dependencies: `F-TST-01` and completed subsystem reports.
- Validations: production-module-to-test map; assertion/fixture quality sampling;
  ignored/flaky/platform-gated inventory; mutation or negative-control sampling
  on selected critical tests.

### Q-DEP-01 - Dependency, supply-chain, and license health

- Status: `pending`
- Question: Are duplicate versions, stale/unmaintained crates/packages, build
  scripts, native dependencies, licenses, and advisories understood for both
  repositories?
- Dependencies: `B-BASE-01`.
- Validations: Rust dependency tree duplicates; frontend dependency inventory;
  advisory scan; license/native/build-script review. Network-backed checks each
  get separate reports and may be inconclusive if unavailable.

### Q-PERF-01 - Performance and resource-lifecycle audit

- Status: `pending`
- Question: Where can prompt assembly, event fanout, persistence, DAG execution,
  frontend reducers, logs/artifacts, locks, tasks, processes, or caches grow
  without bound or block critical execution?
- Dependencies: completed React, TaskRuntime, persistence, and frontend reports.
- Validations: static allocation/lock/task lifecycle trace; representative large
  fixture measurements; cancellation cleanup; disk/cache growth analysis.

### Q-DOC-01 - Current public and operator documentation validation

- Status: `pending`
- Question: Do README, feature/config references, examples, EKO setup docs, and
  architecture claims match reviewed code and executable commands?
- Dependencies: `B-DOC-01`, `F-API-01`, completed application reports.
- Validations: link/path checks; command/example execution sampling; feature and
  config option matrix; stale terminology and architecture search.

### Q-FLT-01 - ReAct and tool fault-injection suite

- Status: `pending`
- Question: Do Agent/tool invariants survive malformed LLM output, Unicode,
  huge output, timeout, cancellation, disconnect, crash, and partial effects?
- Dependencies: `F-RCT-02..05`, `F-EXT-01`, `X-TOL-01`.
- Validations: one report per fault scenario or indivisible parameterized test
  family, with seeds/fixtures and exact terminal sequence.

### Q-FLT-02 - Task and Subagent fault-injection suite

- Status: `pending`
- Question: Do DAG/claim/Subagent invariants survive stale revisions, old
  attempts, cancel, timeout, crash, restart, worktree conflict, and failed review?
- Dependencies: `F-TSK-03`, `F-SUB-02`, `A-TSK-04..06`, `X-TSK-01`.
- Validations: separate scenario reports with persisted before/after snapshots.

### Q-E2E-01 - Real multi-surface smoke and parity suite

- Status: `pending`
- Question: Can representative Chat, Task, Subagent, tool, HITL, attachment,
  Browser/MCP, restart, and large-output scenarios complete on applicable
  surfaces with equivalent facts?
- Dependencies: `X-SRF-01`, `X-EVT-01`, `Q-GUI-01`, `Q-WEB-01`.
- Validations: one report per scenario/surface pair; unavailable credentials or
  external services are explicit `not_run` reports only when the scenario was
  attempted and environmental prerequisites were checked.

## Phase S: Synthesis And Iteration Roadmap

### S-FW-01 - Framework review synthesis

- Status: `pending`
- Output: `<reviewer>/reports/synthesis/framework-review.md`.
- Dependencies: all `F-*`, `Q-FW-*`, relevant `Q-STA-01` and fault reports.
- Validations: report coverage against task catalog; duplicate/contradictory
  finding reconciliation; stale-commit check. Each gets a validation report
  under `S-FW-01`.

### S-APP-01 - Application review synthesis

- Status: `pending`
- Output: `<reviewer>/reports/synthesis/application-review.md`.
- Dependencies: all `A-*`, `Q-CLI-01`, `Q-GUI-01`, `Q-WEB-01`.
- Validations: catalog coverage; contradiction reconciliation; stale-commit
  check.

### S-X-01 - Cross-repository review synthesis

- Status: `pending`
- Output: `<reviewer>/reports/synthesis/cross-repository-review.md`.
- Dependencies: all `X-*`, `S-FW-01`, `S-APP-01`.
- Validations: boundary-gate completeness; canonical duplicate merge; adapter
  loss/authority recheck.

### S-QA-01 - Quality and validation synthesis

- Status: `pending`
- Output: `<reviewer>/reports/synthesis/quality-and-validation-review.md`.
- Dependencies: all `Q-*` and relevant task validation reports.
- Validations: command/report count reconciliation; unexecuted matrix audit;
  flaky/inconclusive result classification.

### S-RDM-01 - Prioritized iteration roadmap

- Status: `pending`
- Output: `<reviewer>/reports/synthesis/iteration-roadmap.md`.
- Dependencies: `S-FW-01`, `S-APP-01`, `S-X-01`, `S-QA-01`, `B-REF-01`.
- Required content: canonical finding IDs; P0-P3 order; framework/application/
  adapter placement; dependency DAG; cross-repository merge order; deletion
  targets; estimated scope; regression validations; measurable acceptance;
  proposed implementation milestones small enough for fresh tasks.
- Validations: every roadmap item backlinks to evidence; every critical design
  decision backlinks to mature implementation research; no duplicate authority
  is left as an indefinite migration state.
