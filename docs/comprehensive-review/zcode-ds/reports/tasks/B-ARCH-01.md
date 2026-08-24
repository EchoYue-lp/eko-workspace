# B-ARCH-01: Framework crate architecture

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are the eight `echo-agent` workspace members layered coherently, without
reverse dependencies or facade leakage?

## Scope

- All 8 `lib.rs` files (root 331 lines + 7 sub-crates).
- Root re-export facades (`src/retry.rs`, `error.rs`, `llm.rs`, `tokenizer.rs`,
  `scheduler.rs`, `human_loop.rs`, `tasks.rs`, `mcp.rs`, `lsp.rs`,
  `compression.rs`, `audit.rs`, `sandbox.rs`, `skills/`, `workflow/`,
  `guard/`, `plugin.rs`, `memory.rs`, `tools/mod.rs`).
- Root real-implementation modules (`agent/`, `evolution/`, `trace/`,
  `context/`, `state/`, `notebook/`, `intent/`, `event_bus.rs`).
- `echo-macros` crate-path resolution (`lib.rs`, `derive_tool.rs`).
- README architecture section, `docs/` tree layout.

## Out Of Scope

- Per-module type placement inside the root engine (`F-API-01`, `X-BND-01`).
- EKO-side composition (`B-PATH-01`).
- Macro expansion semantics beyond crate-path resolution (`F-MAC-01`).
- Historical audit documents (`B-DOC-01`).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (B-ARCH-01 card), `zcode-ds/README.md`.
- Dependency report `B-BASE-01` (zcode-ds track).
- No historical audit conclusion accepted as evidence.

## Layering Decision

- Generic mechanism: the 8-crate DAG and the facade re-export pattern are
  framework build architecture.
- EKO product policy: none (framework-only task).
- Adapter boundary: `echo_macros` crate-path resolution is the macro/crate
  boundary contract.
- Duplicate search terms: `use echo_agent`, `extern crate`, `echo_agent` in
  sub-crate manifests, `#[tool(` vs `derive(Tool)` usage, module-name pairs
  root↔sub-crate (agent, llm, tools, memory, retry, guard, sandbox, skills,
  workflow, tasks, scheduler, human_loop, mcp, lsp, compression, audit,
  tokenizer, plugin). Zero reverse dependencies; zero parallel
  implementations; two contract inconsistencies recorded below.

## Current Path

Cargo-level layering is a clean DAG (V01): `echo_core`/`echo_macros` at L0,
`echo_tools`/`echo_integration`/`echo_state`/`echo_orchestration` at L1,
`echo_execution` at L2, root facade at L3. The facade explicitly separates
re-export modules ("The authoritative implementation lives in `<crate>`")
from root-owned engine modules; the `workspace` alias module exposes
sub-crates for migration (V02). The only cross-crate macro consumer
(`echo_tools`) uses the derive path that resolves to `echo_core` (V03).

## Findings

### B-ARCH-01-P2-01: Root facade carries substantial real implementation alongside the split crates

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/src/lib.rs:28-107` (31 always-compiled modules),
  `src/agent/` (65 files), `src/evolution/` (17 files, ~11.1k lines),
  `src/trace/` (2.3k lines), `src/state/`, `src/context/`, `src/notebook/`,
  `src/intent/`, `src/event_bus.rs`; `src/lib.rs:124-130` (`pub mod workspace`)
- Reachability: the root package is both the facade and the engine; every
  consumer of `echo_agent` compiles all of it. `workspace::` is referenced in
  docs only today but is public API forever once published.
- Expected invariant: either the root is a thin facade over the split crates,
  or the split crates are the framework's second home with a defined
  migration plan.
- Observed behavior: root `src/` contains the ReAct engine, evolution
  (11k lines), trace, runtime state (TaskNode state machine), context
  assembly, notebook, intent — while the 7 sub-crates host traits, tools,
  state stores, providers, workflow, tasks. The "8 crates, 1 import" story
  (`README.md:391`) is an 8-crate workspace where the root still owns the
  largest share of framework logic.
- Impact: (a) `echo_agent::workspace::*` exposes every sub-crate public API
  as facade API, making the facade contract unstable and defeating facade
  drift control; (b) new framework code has two candidate homes with no
  written placement rule, so migration may stall or regress silently;
  (c) maintainers of the root engine get no isolation benefit from the split.
- Root cause: workspace split (echo_core/macros/… migration) progressed for
  traits/stores/tools but the engine and its largest subsystems were never
  moved; the facade pattern made the split invisible to consumers, removing
  the pressure to finish it.
- Direction: record a placement rule (engine stays root-owned while
  subsystem ownership is sub-crate-owned), and plan the final shape of
  `workspace` (delete or keep as documented compat surface) in the iteration
  roadmap; `X-BND-01` should produce the authority map.
- Regression validation: after any reorganization, run the full framework
  gate plus a facade-consumer example (`demo01_tools`) to prove the public
  path is unchanged.
- Validation reports: [V01](../validations/B-ARCH-01/V01-01.md),
  [V02](../validations/B-ARCH-01/V02-01.md)

### B-ARCH-01-P3-01: echo_macros has two crate-path resolvers with different contracts

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:44` (`echo_agent_crate_path` —
  `crate_name("echo_agent")`, no fallback);
  `echo-macros/src/derive_tool.rs:37-62` (`resolve_echo_crate_path` —
  `crate_name("echo_core")` first, `echo_agent` fallback)
- Reachability: attribute macros (`#[tool]`, `#[callback]`, `#[guard]`,
  `#[handler]`, `#[compressor]`, `#[permission_policy]`, `#[audit_logger]`)
  require the consumer to depend on `echo_agent`; the derive `#[derive(Tool)]`
  prefers `echo_core`. `echo_tools` (echo_core-only) uses only the derive
  (`echo-tools/src/git.rs:24`, `statistics.rs:22`); attribute macros are used
  only inside the `echo_agent` package itself (`src/tools/mod.rs`, examples).
- Expected invariant: all macro entry points of one crate agree on the
  consumer's required dependency surface.
- Observed behavior: an `echo_core`+`echo_macros` consumer can use
  `#[derive(Tool)]` but every attribute macro fails with a
  `crate_name("echo_agent")` error; the derive's `echo_agent` fallback
  exists but the attribute path has none.
- Impact: divergent contract; currently no consumer is affected, but any
  future echo_core-only consumer using `#[tool]` gets a confusing expansion
  error, and the two resolvers invite drift.
- Root cause: the derive was written for echo_core-first resolution during
  the workspace split while the older attribute macros kept the original
  echo_agent-only resolution.
- Direction: unify on one resolver (prefer `echo_core`, fall back to
  `echo_agent`, then emit a clear error naming the missing dependency) and
  delete the duplicate function.
- Regression validation: compile-fail fixture for an echo_core-only consumer
  using `#[tool]`; compile-pass fixture for `#[derive(Tool)]` from both an
  echo_core-only and an echo_agent-only consumer (F-MAC-01 owns fixtures).
- Validation reports: [V03](../validations/B-ARCH-01/V03-01.md)

### B-ARCH-01-P3-02: `tasks` marker feature is a no-op while the module is unconditional

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:76` (`tasks = []`), `:80`
  (`subagent = ["tasks"]`); `echo-agent/src/lib.rs:96` (`pub mod tasks;`
  with no `#[cfg(feature = "tasks")]`)
- Reachability: `crate::tasks` compiles under default features; enabling or
  disabling the `tasks` feature changes nothing.
- Expected invariant: each declared feature gates exactly the code it names.
- Observed behavior: the `tasks` feature toggles nothing; `subagent` declares
  a dependency (`tasks`) that its code does not need, because `src/tasks.rs`
  is always compiled.
- Impact: misleading feature topology for consumers and for tooling that
  derives capability sets from features; the manifest comment at
  `Cargo.toml:77-80` documents the coupling as if it were required.
- Root cause: the tasks facade was moved to unconditional re-export
  (`echo_orchestration::tasks`) but the marker feature and the `subagent`
  dependency were left behind.
- Direction: delete the `tasks = []` marker (or gate `pub mod tasks` and
  declare the dependency for real), and simplify `subagent = ["tasks"]`
  accordingly; F-FEAT-01 should audit the other 12 markers the same way.
- Regression validation: `cargo check -p echo_agent --no-default-features
  --features subagent` and default-feature checks both stay green after the
  change.
- Validation reports: [V02](../validations/B-ARCH-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Crate dependency graph / reverse-dependency search | yes | passed | [V01](../validations/B-ARCH-01/V01-01.md) |
| V02 | Public facade mapping | yes | passed | [V02](../validations/B-ARCH-01/V02-01.md) |
| V03 | Cycle or misplaced-type search | yes | passed | [V03](../validations/B-ARCH-01/V03-01.md) |
| V04 | Current documentation comparison | yes | passed | [V04](../validations/B-ARCH-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md`: eight-crate framework workspace with facade root | current | [V01](../validations/B-ARCH-01/V01-01.md) |
| Root `AGENTS.md` history note: sub-crates never depend on root facade | current | [V01](../validations/B-ARCH-01/V01-01.md) |
| Sub-crate docs: "most users should depend on echo_agent" | current | [V04](../validations/B-ARCH-01/V04-01.md) |
| README "8 crates, 1 import" | current | [V04](../validations/B-ARCH-01/V04-01.md) |

## Coverage And Uncertainty

- Root engine internals (agent/react, evolution, trace) were classified by
  module ownership only; deep review is delegated to the F-* subsystem tasks.
- Whether `src/state` (RuntimeStateStore/TaskNode) and
  `echo_orchestration::tasks` duplicate state-machine authority is an open
  question for `F-TSK-01`/`F-TSK-03`; not established as a defect here.
- Feature-gate consistency of the other 12 marker features is F-FEAT-01.
- No compilation was executed in this task; all claims are static.

## Correction

> Dated: 2026-08-12. Factual correction following independent re-verification
> in `F-FEAT-01`.

**`B-ARCH-01-P3-02` is retracted as stated.** The claim that the `tasks`
marker feature is a no-op was wrong: `#[cfg(feature = "tasks")]` appears in
9 production locations (`src/tools/builtin/mod.rs:9,14`,
`src/agent/callbacks/mod.rs:3`, `src/agent/react/mod.rs:31,36,44,403,447`,
`src/agent/react/tests.rs:1505`), so the feature gates real code. What
remains true: `pub mod tasks;` at `src/lib.rs:96` is unconditional and
`src/tasks.rs` re-exports `echo_orchestration::tasks` regardless of the
feature, and `subagent = ["tasks"]` is not needed for compilation (the
module is always compiled). The corrected finding is
`F-FEAT-01-P3-02` (facade module declaration decoupled from feature
topology; the manifest comment at `Cargo.toml:77-80` is outdated).

## Handoff

- Downstream F-* tasks may rely on: acyclic DAG (V01); facade re-export map
  (V02); no parallel implementations of sub-crate concepts (V03).
- `F-MAC-01` owns the resolver-unification fix for P3-01.
- `F-FEAT-01` owns marker-feature reconciliation including P3-02.
- `X-BND-01` should produce the final authority map for root-owned vs
  sub-crate-owned framework modules (P2-01).
- This report becomes stale if any `lib.rs`, manifest feature set, or the
  macro resolver changes.
