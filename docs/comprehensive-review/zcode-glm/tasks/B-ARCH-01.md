# B-ARCH-01: Framework crate architecture

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (9b0e0faf74d35c9a432370b923acabfbb5f32d63)
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are the eight `echo-agent` workspace members layered coherently, without
reverse dependencies or facade leakage?

## Scope

Primary source paths and behaviors inspected:

- All 8 workspace `Cargo.toml` files (root + 7 sub-crates).
- All 8 `src/lib.rs` files.
- Root crate facade modules: `src/lib.rs`, `src/prelude` (in lib.rs),
  `src/advanced` (in lib.rs), `src/workspace` (in lib.rs).
- Root crate thin-re-export files: `src/memory.rs`, `src/audit.rs`,
  `src/compression.rs`, `src/error.rs`, `src/retry.rs`, `src/tokenizer.rs`,
  `src/sandbox.rs`, `src/scheduler.rs`, `src/lsp.rs`, `src/mcp.rs`,
  `src/human_loop.rs`, `src/tasks.rs`, `src/channels.rs`, `src/llm.rs`,
  `src/event_bus.rs`.
- Root crate real-implementation modules: `src/agent/`, `src/state/`,
  `src/workflow/`, `src/tools/`, `src/skills/`, `src/guard/`,
  `src/trace/`, `src/evolution/`, `src/eval/`, `src/headless.rs`,
  `src/hooks_bridge.rs`, `src/memory_promoter.rs`, `src/plugin.rs`.
- All 7 sub-crate README files.
- Root README (high-level only; doctest-level audit deferred to F-API-01).

## Out Of Scope

Deferred to named task IDs:

- Per-feature build matrix compile checks → F-FEAT-01 (feature topology
  and isolation). This task is static-manifest only.
- Public-API doctest sampling and root README deep audit → F-API-01
  (public facade and documentation contract).
- Behaviour-level equivalence check between
  `echo_orchestration::workflow::*` and any root-level workflow code →
  F-API-01 or a workflow-specific F-task. (Static review confirmed the
  root re-exports the orchestration types; no parallel impl found.)
- AUDIT_REPORT.md drift → B-DOC-01 (historical audit and design drift
  index). This task only reads sub-crate READMEs.
- echo-agent-cli consumption of the facade → B-PATH-01 and Phase A tasks.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/README.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).
- The B-ARCH-01 task card and the B-BASE-01 dependency declaration in
  `TASKS.md`.

Dependency task reports read: none. B-BASE-01 was declared `complete` in
the original plan but its report is in a parallel reviewer's directory;
this task's evidence comes from direct manifest/source inspection that
subsumes the B-BASE-01 inventory for the `echo-agent` side.

Historical documents treated as hypotheses: the per-crate README files
were read *as documentation under review*, not as authoritative claims
about the code.

## Layering Decision

Per the AGENTS.md "framework vs application" decision rule, every
observation in this report is classified at the **framework** layer.
The `echo-agent` workspace is the reusable framework; nothing in this
task touches EKO product policy or adapter boundaries. The duplicate /
misplaced-type findings below describe internal framework structure, not
framework-vs-application boundaries.

Repository-wide duplicate-search terms used (cross-crate, all 8 crates +
sub-crate source trees):

- Type/trait names: `Store`, `RuntimeStateStore`, `TaskNode`,
  `AgentCheckpoint`, `EventBus`, `GraphBuilder`, `Graph`,
  `SequentialWorkflow`, `DagWorkflow`, `WorkflowDefinition`,
  `ConversationStore`, `Skill`, `SkillRegistry`, `Tool`, `ToolManager`,
  `LlmClient`, `OpenAiClient`, `AnthropicClient`, `McpManager`,
  `ChannelManager`, `ConsoleApproval`, `ConsoleHumanLoopProvider`,
  `ProviderFactory`, `ReactAgent`, `prelude`.
- Module names: `memory`, `compression`, `audit`, `tools`, `agent`,
  `llm`, `guard`, `workflow`, `state`, `tasks`, `scheduler`, `mcp`,
  `channels`, `lsp`, `skills`, `sandbox`, `tokenizer`, `retry`, `error`.
- Path patterns: `echo_agent::`, `echo_core::`, `echo_state::`,
  `echo_orchestration::`, `echo_integration::`, `echo_tools::`,
  `echo_execution::`, `extern crate`, `crate_name("echo_agent")`.

## Current Path

Verified dependency graph and facade shape (full evidence in V01 and V02
reports):

```text
L0  echo_core, echo_macros
       ↑           ↑
       |           |
L1  echo_tools (→ core, macros)
L1  echo_integration (→ core)
L1  echo_state (→ core)
L1  echo_orchestration (→ core)
       ↑
       |
L2  echo_execution (→ core, tools)
       ↑
       |
L3  echo_agent (facade) → all 7 sub-crates
```

- No sub-crate Cargo.toml names `echo_agent`.
- No sub-crate source file statically imports `echo_agent::*`.
- One `extern crate self as echo_agent;` at the root for macro path
  resolution (idiomatic).
- Two `proc_macro_crate::crate_name("echo_agent")` lookups in
  `echo_macros` (runtime consumer resolution, not a Cargo edge).

Facade shape:

- Root exposes 30+ always-compiled `pub mod` declarations plus ~10
  feature-gated ones.
- `prelude` (root `src/lib.rs:137-276`) re-exports ~100 types from
  root's own modules and from `echo_core`/`echo_macros` directly.
- `advanced` (`src/lib.rs:279-331`) re-exports optional-feature types
  plus `tasks` and `agent::critic`.
- `workspace` (`src/lib.rs:124-130`) aliases 5 of the 7 sub-crates
  (missing: `echo_tools`, `echo_macros`).

The facade is structurally sound but is not a pure re-export: the root
crate owns substantial real implementation (`agent/`, `state/`,
`workflow/`, `trace/`, `evolution/`, `eval/`, `guard/`, `headless.rs`,
`event_bus.rs`, `hooks_bridge.rs`, `plugin.rs`, `memory_promoter.rs`).
This is partial migration debt, documented in the next section.

## Findings

### B-ARCH-01-P2-01: Root facade owns real domain implementation that has not migrated to sub-crates

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/state/mod.rs:244` (defines `RuntimeStateStore` trait)
  - `echo-agent/src/state/mod.rs:57` (`TaskNode`)
  - `echo-agent/src/state/mod.rs:117` (`AgentCheckpoint`)
  - `echo-agent/src/state/file.rs` (`FileRuntimeStateStore`)
  - `echo-agent/src/state/sqlite.rs` (`SqliteRuntimeStateStore`)
  - `echo-agent/src/event_bus.rs:12,44` (`EventBus`, `GLOBAL_EVENT_BUS`)
  - `echo-agent/src/llm.rs:116-178` (compat wrappers `assemble_req_header`,
    `chat`, `stream_chat` that delegate to
    `echo_integration::providers::openai::*`)
- Reachability: definition in root crate → registered as `pub mod state;`,
  `pub mod event_bus;`, `pub mod llm;` in `src/lib.rs:58,39,47` → live
  callers throughout the root crate (e.g. `src/agent/`, `src/state/`
  consumers). Reachable from outside via `echo_agent::state::*` and
  `echo_agent::EventBus`.
- Expected invariant (per AGENTS.md layering rule and the crate READMEs'
  stated split): runtime-state persistence and event-bus infrastructure
  belong in `echo_state` or `echo_core`; LLM HTTP wrappers belong in
  `echo_integration::providers::openai`. The facade should re-export them,
  not own them.
- Observed behavior: root crate defines these types and impls in its own
  `src/` tree. The compat wrappers in `src/llm.rs:111-118` are
  explicitly documented as remaining "so older call sites can stay on
  `echo_agent::llm::*`" — acknowledged debt.
- Impact: framework consumers that depend on `echo_state` directly do
  not get `RuntimeStateStore`. Consumers that depend on `echo_core`
  do not get `EventBus`. The split is incomplete, so the sub-crate
  layering promised by the READMEs is not yet real for these concepts.
- Root cause: in-progress crate split. The migration that produced
  `echo_state` extracted `Store`/`ConversationStore`/compressors/audit
  but stopped before extracting runtime-state checkpointing, the global
  event bus, and the LLM HTTP wrappers.
- Direction (recommendation, not a fix in this review): open a follow-up
  refactor task to (a) move `RuntimeStateStore`, `TaskNode`,
  `AgentCheckpoint`, `FileRuntimeStateStore`, `SqliteRuntimeStateStore`
  into `echo_state`; (b) move `EventBus`/`GLOBAL_EVENT_BUS` into
  `echo_core` (it consumes `EventEnvelope` which already lives there);
  (c) delete the `assemble_req_header`/`chat`/`stream_chat` compat
  wrappers in `src/llm.rs` once call sites are migrated to
  `echo_integration::providers::openai::*`. Each move must keep the
  facade re-export in place so external consumers see no breakage.
- Regression validation: a successful `cargo check --workspace
  --all-features` after each move, plus a behavioural test that exercises
  `RuntimeStateStore::save_checkpoint`/`restore_messages` end-to-end.
- Validation reports: [V01](../validations/B-ARCH-01/V01-01.md),
  [V02](../validations/B-ARCH-01/V02-01.md),
  [V03](../validations/B-ARCH-01/V03-01.md).

### B-ARCH-01-P2-02: Facade exposes parallel access paths to the same items

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/lib.rs:119-130` — `workspace` module documents itself
    as "Direct access to split workspace crates **during migration**".
  - Same item reachable through up to 4 paths. Examples verified:
    - `TaskRevisionService`: `echo_agent::tasks::*`,
          `echo_agent::advanced::*`, `echo_agent::workspace::orchestration::tasks::*`,
          `echo_orchestration::tasks::*`.
    - `Store`: `echo_agent::memory::*`,
      `echo_agent::workspace::state::memory::*`, `echo_core::memory::*`.
    - `LocalSandbox`: `echo_agent::sandbox::*`,
      `echo_agent::prelude::*` (re-exported twice within root),
      `echo_execution::sandbox::*`.
    - `McpManager`: `echo_agent::mcp::*`, `echo_agent::advanced::*`,
      `echo_agent::workspace::integration::mcp::*`,
      `echo_integration::mcp::*`.
- Reachability: all paths resolve to the same item at compile time (Rust
  identity is structural, so this is not a name-collision bug).
- Expected invariant: each public item has one canonical facade path so
  that downstream code is uniform and changes touch one site.
- Observed behavior: four paths are live and documented. The `workspace`
  module is intentionally non-uniform (omits `echo_tools` and
  `echo_macros`).
- Impact: maintenance overhead — changing a type's source crate requires
  updating re-exports in multiple facade modules. Consumer code is
  inconsistent: examples in the READMEs use different paths
  interchangeably. Cross-reviewer confusion when grepping for symbol
  definitions.
- Root cause: deliberate migration scaffold. The `workspace` module is
  the documented escape hatch for callers that want to bypass facade
  drift while the split is in progress. It is correct as a transitional
  measure; the debt is that the transition has not been completed.
- Direction: complete the migration (see P2-01) and then remove the
  `workspace` escape hatch and the nested `pub mod state { ... }` /
  `pub mod core { ... }` aliases inside thin-re-export files. Until
  then, no action — the scaffold is doing its job.
- Regression validation: after removal of `workspace::*`, run
  `cargo check --workspace --all-features` and update any consumer
  that still imports through `workspace::*`.
- Validation reports: [V02](../validations/B-ARCH-01/V02-01.md).

### B-ARCH-01-P2-03: echo-orchestration README example does not compile

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/README.md:23-27` — example uses
    `GraphBuilder::new()` with no arguments.
  - `echo-agent/echo-orchestration/src/workflow/graph.rs:214-216` —
    actual signature is `pub fn new(name: impl Into<String>) -> Self`.
- Reachability: README is a published doc on docs.rs (the crate's
  `documentation = "https://docs.rs/echo_orchestration"` link in
  `echo-orchestration/Cargo.toml:10`). Any user copying the example
  gets a compile error.
- Expected invariant: README examples should compile or be marked
  `rust,ignore`/`rust,no_run` if they are intentionally illustrative.
- Observed behavior: example is presented as Rust code without an
  `ignore`/`no_run` annotation and uses an API that does not exist.
- Impact: new users hitting the orchestration crate as a standalone
  dependency (which the README explicitly encourages at line 13-15)
  cannot run the canonical "build a graph" example. They must reverse-
  engineer the real signature.
- Root cause: README predates the `name` parameter addition to
  `GraphBuilder::new` and was never refreshed.
- Direction: update README example to `GraphBuilder::new("pipeline")`
  (matching the root README example) or mark the snippet
  `rust,ignore`. Trivial fix.
- Regression validation: turn the README example into a doctest
  (`cargo test --doc -p echo_orchestration`).
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P2-04: echo-integration README example references nonexistent `ProviderFactory::create_openai`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/README.md:23` — example calls
    `ProviderFactory::create_openai("gpt-5.5", std::env::var("OPENAI_API_KEY")?)?`.
  - `echo-agent/echo-integration/src/providers/config.rs:245` — actual
    method is `pub fn openai(api_key: impl Into<String>, model: impl
    Into<String>) -> Self` (no `create_` prefix; no `Result` return).
  - `grep -rn "create_openai" echo-integration/src/` → zero hits.
- Reachability: published on docs.rs via the crate's `documentation`
  link.
- Expected invariant: README examples compile.
- Observed behavior: example uses a method name that does not exist on
  the type. The example also treats the return as `Result` (uses `?`),
  but the actual `openai(...)` constructor returns `Self` directly.
- Impact: any user copying the example gets a compile error. The same
  users are the audience the README explicitly targets at line 13-15
  ("Most users should depend on `echo_agent`... instead of depending on
  `echo_integration` directly" — implying direct consumers are advanced
  users who will read the README).
- Root cause: API was renamed from `create_openai` to `openai` (and the
  return type simplified from `Result<Self>` to `Self`) without updating
  the README.
- Direction: replace `ProviderFactory::create_openai(model, key)?` with
  `ProviderFactory::openai(key, model)`. Trivial fix.
- Regression validation: turn the README example into a doctest.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-01: echo-core README references a `prelude` module that does not exist

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/README.md:21-22` — `Use the prelude to import
    all core traits: use echo_core::prelude::*;`.
  - `echo-agent/echo-core/src/lib.rs:21-40` — no `prelude` module
    declared. Verified via `grep -n "pub mod prelude\|pub use.*prelude"
    echo-core/src/lib.rs` → zero hits.
- Reachability: README published on docs.rs.
- Expected invariant: README references match the crate's actual public
  API.
- Observed behavior: README instructs users to import
  `echo_core::prelude::*`, which would fail to compile. The only
  framework `prelude` lives in the root facade at
  `echo-agent/src/lib.rs:137`.
- Impact: direct `echo_core` consumers (which the README explicitly
  targets at line 17-29) cannot follow the quickstart. Minor severity
  because most users go through the root facade.
- Root cause: README assumes the root facade's prelude pattern is
  replicated in `echo_core`. It is not.
- Direction: either remove the `prelude` reference from the README
  (preferred — adding a prelude to `echo_core` would expand the public
  API surface for no clear benefit) or add a `prelude` module to
  `echo_core`. Remove option is cheaper and consistent with the
  framework's "one prelude at the facade" design.
- Regression validation: render docs.rs preview and confirm no broken
  intra-doc links.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-02: echo-state README omits `profiles` and `skill_telemetry` modules

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/README.md:30-33` — Contents list mentions
    only Memory, Compression, Audit.
  - `echo-agent/echo-state/src/lib.rs:24,26` — declares `pub mod
    profiles;` and `pub mod skill_telemetry;`. The lib.rs header
    comment (lines 9-13) does mention both.
- Reachability: README published on docs.rs.
- Expected invariant: README Contents covers all `pub mod` items.
- Observed behavior: two public modules missing from README.
- Impact: direct consumers miss two real features. Low severity because
  the crate-level doc comment does cover them and `cargo doc` will
  render them.
- Root cause: README not refreshed when `profiles` and
  `skill_telemetry` were added.
- Direction: extend the Contents table.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-03: echo-orchestration README omits `planning` and `scheduler` modules

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-orchestration/README.md:37-41` — Contents list
    mentions only Workflow, Human-in-the-Loop, Task Management.
  - `echo-agent/echo-orchestration/src/lib.rs:23,25` — declares
    `pub mod planning;` and `pub mod scheduler;`. The lib.rs header
    (lines 8-13) mentions both.
- Reachability: README published on docs.rs.
- Expected invariant: README Contents covers all `pub mod` items.
- Observed behavior: two public modules missing from README. The README
  also has a Feature Flags table that correctly lists `websocket`.
- Impact: direct consumers miss two real features. Compounds with
  P2-03 (broken example) in making the orchestration README unreliable.
- Root cause: README not refreshed when `planning` and `scheduler`
  were added as pub modules.
- Direction: extend the Contents table.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-04: echo-core README contents list mentions `ReActAgent`, which lives in the root facade

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/README.md:34` — "Agent traits: `Agent`,
    `ReActAgent`, `SubAgent`".
  - `echo-agent/echo-core/src/agent/` — defines only the `Agent` trait,
    `AgentEvent`, `AgentCallback`, `CancellationToken`, `EventEnvelope`,
    etc. No `ReactAgent` type.
  - `echo-agent/src/agent/react/` — that is where `ReactAgent` is
    defined (root facade). Verified by reading
    `echo-agent/src/agent/mod.rs:36-41`.
- Reachability: README published on docs.rs.
- Expected invariant: README contents list refers only to types defined
  in the crate.
- Observed behavior: README claims `ReActAgent` is one of echo_core's
  agent traits; it is actually a facade-level type.
- Impact: users looking for `ReActAgent` in `echo_core` will not find
  it. Low severity because the canonical entry point is the facade
  anyway.
- Root cause: README was written before the crate split and conflated
  the trait (`Agent`) with the concrete engine (`ReactAgent`).
- Direction: drop `ReActAgent` from the echo_core README list. The
  root README and root prelude already cover the engine.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-05: echo_macros has two divergent crate-path resolvers

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-macros/src/lib.rs:43-51` — `echo_agent_crate_path()`
    resolves *only* `echo_agent`. Used by the `#[tool]`, `#[callback]`,
    `#[guard]`, `#[handler]`, `#[compressor]`, `#[permission_policy]`,
    and `#[audit_logger]` attribute macros (lib.rs lines 226-605).
  - `echo-agent/echo-macros/src/derive_tool.rs:37-62` —
    `resolve_echo_crate_path()` tries `echo_core` *first*, then falls
    back to `echo_agent`. Used only by `#[derive(Tool)]`.
- Reachability: both functions are private (`fn`, not `pub fn`), called
  only inside `echo_macros`. The behaviour they produce affects every
  consumer of the macros.
- Expected invariant: macro codegen resolves the consumer's crate
  consistently. A consumer that depends on `echo_core` + `echo_macros`
  directly (no `echo_agent`) should be able to use *all* macros, not
  just `#[derive(Tool)]`.
- Observed behavior: only `#[derive(Tool)]` works for `echo_core`-
  only consumers. All attribute macros (`#[tool]`, etc.) generate code
  with `::echo_agent::*` paths that won't resolve unless the consumer
  also depends on `echo_agent`.
- Impact: `echo_tools` (which depends on `echo_core` + `echo_macros`
  only — see `echo-tools/Cargo.toml:40-41`) cannot use the `#[tool]`
  attribute macro for its own internal tool definitions without also
  pulling in `echo_agent`. This silently couples the "standalone
  sub-crate" use case to the facade.
- Root cause: incremental migration. `derive_tool.rs` was updated to
  the new resolver; the older attribute macros were not.
- Direction: replace `echo_agent_crate_path` in `lib.rs` with the
  `resolve_echo_crate_path` logic from `derive_tool.rs`. No external
  API change; purely internal cleanup. Risk: low — the fallback still
  hits `echo_agent` for facade consumers.
- Regression validation: `cargo test --workspace --all-features`;
  specifically add a test that invokes `#[tool]` from inside
  `echo_tools` (or an `examples/` crate that depends only on
  `echo_core`+`echo_macros`).
- Validation reports: [V01](../validations/B-ARCH-01/V01-01.md),
  [V03](../validations/B-ARCH-01/V03-01.md).

### B-ARCH-01-P3-06: Thin-re-export files use non-uniform nested-submodule conventions

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `src/memory.rs:42-45`, `src/compression.rs:27-30`, `src/audit.rs:29-32`
    expose a nested `pub mod state { ... }`.
  - `src/human_loop.rs:8-11`, `src/tasks.rs:8-11` expose
    `pub mod orchestration { ... }`.
  - `src/error.rs:9-12`, `src/retry.rs:6-9`, `src/tokenizer.rs:6-9`
    expose `pub mod core { ... }`.
  - `src/mcp.rs:6-9`, `src/channels.rs:59-62` expose
    `pub mod integration { ... }`.
  - `src/llm.rs:53-82` exposes *five* nested submodules (`core`,
    `integration`, `types`, `config`, `providers`) — the most irregular.
  - `src/lsp.rs:1-11` exposes *no* nested submodule — different from the
    others.
  - `src/sandbox.rs`, `src/scheduler.rs` — no nested submodule.
- Reachability: all are public API surface; each adds a parallel path
  for the same items (e.g. `echo_agent::memory::state::*` resolves to
  the same items as `echo_agent::memory::*`).
- Expected invariant: a single convention for the migration escape
  hatch in thin-re-export files.
- Observed behavior: at least four different naming schemes for the
  nested submodule (`state`, `orchestration`, `core`, `integration`,
  or none).
- Impact: maintenance cost and learning curve. Cosmetic; not a
  correctness issue.
- Root cause: each file was authored by a different pass of the
  migration; no shared convention was enforced.
- Direction: pick one convention (or delete the nested submodules
  entirely — the `workspace::*` module already provides the canonical
  escape hatch). Best handled as part of P2-02 when the migration
  scaffold is removed.
- Regression validation: `cargo check --workspace --all-features` to
  confirm no broken imports.
- Validation reports: [V02](../validations/B-ARCH-01/V02-01.md).

### B-ARCH-01-P3-07: `workspace` module aliases only 5 of 7 sub-crates

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/lib.rs:124-130` — aliases `echo_core`, `echo_execution`,
    `echo_integration`, `echo_orchestration`, `echo_state`. Does **not**
    alias `echo_tools` or `echo_macros`.
- Reachability: public API at `echo_agent::workspace::*`.
- Expected invariant: if the `workspace` module is the documented
  escape hatch for "direct access to split workspace crates", it should
  cover all split crates.
- Observed behavior: `echo_tools` and `echo_macros` are not aliased.
  Consumers wanting direct access must add a direct Cargo dependency
  instead of going through the facade.
- Impact: minor asymmetry. `echo_tools` is a major surface (the largest
  sub-crate by feature count) and its absence from the escape hatch is
  surprising.
- Root cause: likely an oversight when the workspace module was added.
- Direction: either add `pub use echo_tools as tools;` and
  `pub use echo_macros as macros;` to the `workspace` module, or
  document why they are excluded.
- Regression validation: `cargo check --workspace`.
- Validation reports: [V02](../validations/B-ARCH-01/V02-01.md).

### B-ARCH-01-P3-08: echo-tools README Features table omits `statistics` and `full` features

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/README.md:12-23` — Features table lists
    `files`, `shell`, `web`, `media`, `chart`, `data`, `database`,
    `git`, `rag`, `research`.
  - `echo-agent/echo-tools/Cargo.toml:17,34` — defines `full` and
    `statistics` features.
  - `echo-agent/echo-tools/src/lib.rs:72-74` — declares `pub mod
    statistics;` under the `statistics` feature.
- Reachability: README published on docs.rs.
- Expected invariant: README Features table covers all Cargo features.
- Observed behavior: two features missing.
- Impact: users miss the `statistics` tools (exploratory statistics
  module) and the `full` aggregate. Low severity — features are still
  documented in `Cargo.toml` and the lib.rs header.
- Root cause: README predates the `statistics` and `full` features.
- Direction: extend the table.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-09: echo-tools README describes `git` feature as a single `GitTool`

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/README.md:21` — "`git` | GitTool | —".
  - `echo-agent/echo-tools/src/lib.rs:51-86` — declares `git`,
    `git_worktree`, `worktree_tool` modules under the `git` feature.
  - The lib.rs header comment (lines 1-18) says "`git` (6 git CLI
    tools)" — internal doc disagrees with the README.
- Reachability: README published on docs.rs.
- Expected invariant: README feature descriptions match the actual
  module contents.
- Observed behavior: README says one tool; crate header says six tools.
- Impact: users underestimate the git surface. Low severity.
- Root cause: README predates the git-tool expansion.
- Direction: refresh README to match the crate header doc.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

### B-ARCH-01-P3-10: echo-integration README overstates provider breadth

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-integration/README.md:32` — "LLM Providers: OpenAI,
    Anthropic, DeepSeek, Qwen (DashScope), Ollama".
  - `echo-agent/echo-integration/src/providers/` — contains only
    `openai.rs`, `anthropic.rs`, `adapter_client.rs`,
    `anthropic_cache.rs`, `client.rs`, `config.rs`,
    `thinking_translate.rs`, `traits.rs`. No dedicated modules for
    DeepSeek, Qwen, or Ollama.
- Reachability: README published on docs.rs.
- Expected invariant: README provider list matches dedicated provider
  modules.
- Observed behavior: README lists 5 named providers; only 2 have
  dedicated modules. DeepSeek/Qwen/Ollama are presumably reachable via
  the OpenAI-compatible `OpenAiClient` config, but the README does not
  explain that distinction.
- Impact: users may search for an `OllamaClient` that does not exist.
  Low severity.
- Root cause: README conflates "providers configurable via the OpenAI
  client" with "dedicated provider modules".
- Direction: clarify in README that OpenAI-compatible providers
  (DeepSeek, Qwen, Ollama) are configured via `OpenAiClient` with a
  custom base URL, not via dedicated modules.
- Regression validation: render docs.rs preview.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Crate dependency graph (manifest-level DAG, no cycles, no reverse deps, resolver) | yes | passed | [V01-01](../validations/B-ARCH-01/V01-01.md) |
| V02 | Public facade mapping (prelude, advanced, workspace, duplicate paths) | yes | passed_with_notes | [V02-01](../validations/B-ARCH-01/V02-01.md) |
| V03 | Cycle or misplaced-type search (extern crate audit, reverse-import audit, type placement) | yes | passed_with_notes | [V03-01](../validations/B-ARCH-01/V03-01.md) |
| V04 | Documentation comparison (sub-crate READMEs vs actual code) | yes | failed | [V04-01](../validations/B-ARCH-01/V04-01.md) |
| V05 | Targeted executable compile check of broken README examples | conditional | not_run | See Coverage section |

The V05 conditional validation was not run because it would require
modifying README files into doctests, which is a code change and outside
the read-only scope of this task. The V04 static inspection already
establishes that the examples do not compile.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: "echo-agent is reviewed as an independent reusable framework. An API is not dead merely because echo-agent-cli does not call it." | current | Verified: sub-crate READMEs and Cargo.tomls treat each crate as independently consumable; the framework layering is real at the Cargo level. |
| AGENTS.md: "Now workspace has natively solved full-member coverage; resolver = 3." | current | `echo-agent/Cargo.toml:21` — `resolver = "3"`. `members` lists 7 sub-crates; root is the workspace package itself. |
| AGENTS.md: "Standard relationship: TaskRun → PlanTask → SubagentRun; only Subagent, no Worker." | not checked here | Out of scope — terminology audit belongs to a Phase F or Phase A task. (No `worker` symbol references were observed in the files inspected, but no systematic grep was performed.) |
| AGENTS.md: "echo-state's `sqlite` feature is for other framework reusers; echo-agent-cli does not enable it." | current | `echo-state/Cargo.toml:22` — `sqlite` is an optional feature; root `echo-agent/Cargo.toml:71` enables it via the root `sqlite` feature. CLI consumption not checked here. |
| echo-core README: "Use `echo_core::prelude::*`" | stale | `echo_core` has no prelude module. (P3-01) |
| echo-orchestration README: `GraphBuilder::new()` | stale | Actual API requires a name argument. (P2-03) |
| echo-integration README: `ProviderFactory::create_openai` | stale | Method renamed to `openai`. (P2-04) |
| echo-state README contents list (Memory, Compression, Audit) | stale | Missing `profiles`, `skill_telemetry`. (P3-02) |
| echo-orchestration README contents list (Workflow, HITL, Tasks) | stale | Missing `planning`, `scheduler`. (P3-03) |

## Coverage And Uncertainty

Code not inspected deeply:

- Root `src/agent/` (the ReactAgent engine, callbacks, subagent) — file
  listing only. Behaviour review belongs to a Phase F task.
- Root `src/evolution/` (typed memory, candidate, curator, dreaming,
  review, security, triggers) — file listing only.
- Root `src/eval/`, `src/improve/` — file listing only.
- Root `src/plugin.rs`, `src/security.rs`, `src/intent/`, `src/notebook/`,
  `src/context/`, `src/utils/`, `src/paths.rs`, `src/runner.rs`,
  `src/config.rs` (37 KB) — not opened.
- Root `README.md` (50 KB) — not line-by-line audited; high-level only.
  Deferred to B-DOC-01 / F-API-01.
- `AUDIT_REPORT.md` (28 KB) — not opened; B-DOC-01 owns historical-claim
  validation.
- `Cargo.lock` — not inspected; B-BASE-01 owns lockfile inventory.
- `.github/workflows/` — not inspected; B-BASE-01 owns CI inventory.

Validations not run:

- No `cargo`/`cargo doc`/doctest execution (read-only review). All
  "example does not compile" claims are based on direct comparison of
  README text against source signatures. The claims are robust because
  the mismatches are unambiguous (method renamed, argument count
  changed, type renamed), but a doctest sweep by F-API-01 will harden
  them.

Claims that remain uncertain:

- Whether `echo_integration::providers` truly exposes DeepSeek/Qwen/
  Ollama "providers" through configuration only (P3-10 confidence: high
  but not exhaustively verified). A grep for `deepseek`/`qwen`/`ollama`
  in `echo-integration/src/providers/` would confirm; left to F-API-01.
- Whether the `tracing::target: "echo_agent::tool_budget"` string at
  `echo-execution/src/tools.rs:403` is the only non-doc reference to
  the root crate from a sub-crate. The grep was thorough but a single
  string literal could be misread; confidence remains high.

## Handoff

Conclusions downstream tasks may rely on:

- The workspace is a clean acyclic DAG at the Cargo level. Sub-crates
  never depend on the root facade. (V01)
- Layering is `core`/`macros` → mid-layer → `execution` → facade.
  `echo_execution` is at L2 (not L1) because it depends on `echo_tools`.
  (V01)
- The root crate is **not** a pure facade. It owns real implementation
  for `state` (RuntimeStateStore), `event_bus`, parts of `llm`
  (compat wrappers), plus the ReactAgent engine and many supporting
  modules. (V02, V03)
- The `workspace::*` module exposes 5 of 7 sub-crates as direct-import
  escape hatch. `echo_tools` and `echo_macros` are missing. (V02)
- Sub-crate READMEs have multiple compile-breaking examples and stale
  contents lists. (V04)

Reports downstream tasks must read:

- F-API-01 (public facade and documentation contract) should read V02-01
  and V04-01 in full — the duplicate-path inventory and the README drift
  list are the starting points for the doctest sweep.
- F-FEAT-01 (feature topology) should read V01-01's feature-forwarding
  table — it is the baseline for the cfg-to-feature search.
- F-CORE-01 (core identities) should read V03-01's type-placement table —
  `RuntimeStateStore` living in the root affects identity-stability
  claims.
- B-DOC-01 (historical drift) should read V04-01 — it has the concrete
  README-vs-code drift items that the historical-claim audit will need
  to classify.

Conditions that make this report stale:

- Any commit that moves `RuntimeStateStore`/`TaskNode`/`AgentCheckpoint`
  out of the root crate invalidates P2-01.
- Any commit that removes the `workspace` module or unifies the
  thin-re-export file conventions invalidates P2-02 and P3-06.
- Any commit that fixes the echo-orchestration/echo-integration README
  examples invalidates P2-03 and P2-04.
- Any commit that adds `echo_tools`/`echo_macros` to the `workspace`
  module invalidates P3-07.
- Any change to sub-crate `Cargo.toml` path dependencies invalidates
  V01-01 (re-run as V01-02).

Follow-up task IDs (recommended, not implemented in this review):

- A framework-refactor task to migrate `RuntimeStateStore`+impls into
  `echo_state` and `EventBus` into `echo_core`. This is the largest
  item of migration debt surfaced by B-ARCH-01.
- A documentation-cleanup task (likely owned by B-DOC-01 or F-API-01)
  to fix the four broken README examples and the four stale contents
  lists.
- A small internal-cleanup task to unify the two crate-path resolvers
  in `echo_macros` (P3-05).
