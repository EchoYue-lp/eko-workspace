# F-API-01: Public facade and documentation contract

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (9b0e0faf74d35c9a432370b923acabfbb5f32d63)
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Do root re-exports, crate-level APIs, examples, and docs expose one
coherent framework rather than accidental internals?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/lib.rs` — `prelude` (lines 137-276), `advanced`
  (lines 279-331), `workspace` (lines 124-130), top-level macro
  re-export (lines 115-117), conditional `project_rules` re-export
  (lines 106-107), and the `pub mod` declarations (lines 28-104).
- `echo-agent/src/{memory,audit,compression,error,retry,tokenizer,
  sandbox,scheduler,lsp,mcp,human_loop,tasks,channels,llm,event_bus}.rs`
  — the thin-re-export files backing the prelude's `crate::*`
  references.
- `echo-agent/Cargo.toml` — `[features]`, `[package.metadata.docs.rs]`,
  `[[example]]` registrations.
- `echo-agent/examples/{demo00_quickstart,demo01_tools,demo02_tasks,
  demo03_approval}.rs` — canonical entry-point examples.
- `echo-agent/examples/README.md` — example classification.
- `echo-agent/README.md` — top-level API examples (spot-checked,
  lines 36-37, 84, 181, 391, 409-595).
- Sub-crate source files referenced for canonical type definitions:
  `echo-core/src/{error.rs,memory/store.rs,tokenizer.rs,tools/
  permission.rs}`, `echo-execution/src/sandbox/local.rs`,
  `echo-integration/src/{mcp/mod.rs,providers/config.rs}`,
  `echo-orchestration/src/{tasks/revisioned.rs,human_loop/}`.
- Sub-crate README files where they overlap with the facade contract
  (deferred detail from B-ARCH-01-V04).

## Out Of Scope

Deferred to named task IDs:

- Per-feature build matrix compile checks (`cargo check --no-default-
  features --features <f>`) → F-FEAT-01 (feature topology). This task
  is static-manifest only.
- Sub-crate README deep audit (echo-orchestration/echo-integration
  example breaks, echo-core prelude reference, echo-state/echo-
  orchestration missing-contents lists) → already covered by
  B-ARCH-01-P2-03/P2-04/P3-01/P3-02/P3-03. This task inherits those
  findings and focuses on the root facade contract.
- Macro hygiene and expansion correctness for `#[tool]`/`#[derive(Tool)]`
  → F-MAC-01.
- LLM provider feature parity → F-LLM-01.
- Tool builtin inventory and registration → F-EXT-01.
- Root README doctest sweep beyond the spot-checks noted → B-DOC-01
  (historical drift) and a potential follow-up.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full,
  via system reminder — particularly the "framework vs application"
  decision rule and the "first check if it already exists" rule).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/README.md`
  (referenced via REPORTING.md).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/zcode-glm/tasks/B-ARCH-01.md`
  (in full). B-ARCH-01 is the declared dependency; its P2-01/P2-02/
  P3-06/P3-07 findings directly inform the F-API-01 scope. This task
  sharpens B-ARCH-01's "parallel access paths" finding into item-level
  evidence and adds the public-API/docs.rs contract angle that
  B-ARCH-01 explicitly deferred.

Historical documents treated as hypotheses: the root README's claims
about `use echo_agent::prelude::*` being a single integration surface
and the examples' implicit promise of compiling out-of-the-box.

## Layering Decision

Per the AGENTS.md "framework vs application" rule, every observation in
this report is classified at the **framework** layer. The `echo_agent`
facade is the public API surface of the reusable framework; nothing
here touches EKO product policy. The "framework API is retained unless
framework-wide evidence shows it is obsolete or fully replaced" rule
applies: this task does not recommend removing any public item, only
unifying the path through which it is exported.

Repository-wide duplicate-search terms used (cross-crate, all 8 crates
+ sub-crate source trees):

- Facade items sampled for path enumeration: `Store`, `OpenAiClient`,
  `ProviderFactory`, `McpManager`, `LocalSandbox`, `TaskRevisionService`
  (6 items, chosen to cover all 5 mid-layer sub-crates plus echo_core).
- Module names: `memory`, `compression`, `audit`, `tools`, `agent`,
  `llm`, `tasks`, `mcp`, `human_loop`, `sandbox`, `channels`, `lsp`,
  `error`, `retry`, `tokenizer`, `workflow`, `prelude`, `advanced`,
  `workspace`.
- Path patterns: `echo_agent::prelude::`, `echo_agent::advanced::`,
  `echo_agent::workspace::`, `echo_agent::memory::state::`,
  `echo_agent::tasks::orchestration::`, `echo_agent::llm::providers::`,
  `echo_agent::llm::integration::`, `echo_core::tools::permission::`.

## Current Path

The facade exposes three layers of public surface (full counts and
source-path mapping in V01-01):

```text
Layer 1: prelude::*         — 200 items, 30 source-paths, mostly crate::*-relative
Layer 2: advanced::*        —  69 items, 9 source-paths, all crate::*-relative
Layer 3: workspace::*       — 5 crate aliases (core, execution, integration,
                              orchestration, state); missing tools, macros
Plus:    top-level          — pub use echo_macros::{8 attribute/derive macros};
                              pub use echo_core::project_rules (feature-gated)
```

Items reach the consumer through this chain:

```text
consumer code
   ↓
echo_agent::prelude::<Item>           (curated always-on)
echo_agent::advanced::<Item>          (curated feature-gated)
echo_agent::<thin-module>::<Item>     (e.g. memory, tasks, llm)
echo_agent::<thin-module>::<subcrate>::<Item>
                                      (nested escape hatch in thin-module)
echo_agent::workspace::<subcrate>::<Item or module>
                                      (crate-level alias)
   ↓
crate::<module>::<Item>               (root's own module — sometimes real impl,
                                       sometimes thin re-export)
   ↓
echo_<subcrate>::<module>::<Item>     (canonical definition in sub-crate)
```

The chain is structurally sound (V01-01 confirms every `pub use`
resolves to a real sub-crate). The coherence failure is that the same
item enters through multiple points in the chain simultaneously
(V02-01). The documentation failure is that one feature
(`research`) gating a public module is omitted from the docs.rs
metadata, so the rendered docs hide part of the surface (V04-01).

## Findings

### F-API-01-P2-01: `workspace` module is an incomplete and asymmetric escape hatch

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/lib.rs:124-130` — `pub mod workspace { pub use
    echo_core as core; pub use echo_execution as execution; pub use
    echo_integration as integration; pub use echo_orchestration as
    orchestration; pub use echo_state as state; }`. Five aliases;
    `echo_tools` and `echo_macros` are not aliased.
  - `echo-agent/src/lib.rs:119-123` — module doc claims "Direct access
    to split workspace crates during migration."
  - `echo-agent/Cargo.toml` workspace declaration (lines 6-14) lists
    seven members: echo_core, echo_macros, echo_execution,
    echo_integration, echo_tools, echo_state, echo_orchestration.
- Reachability: definition in `src/lib.rs:124-130` → live callers via
  `echo_agent::workspace::*`. The omission means consumers wanting
  `echo_tools::*` or `echo_macros::*` direct access must add those
  crates as explicit Cargo dependencies; they cannot go through the
  facade.
- Expected invariant (per AGENTS.md "first check if it already exists"
  and the workspace module's own doc): if the facade documents a
  migration escape hatch to "split workspace crates", it should cover
  all split crates, especially the largest one (`echo_tools`, which
  has 12 features — the most of any sub-crate).
- Observed behavior: `echo_tools` and `echo_macros` are silently
  absent. No inline comment explains why.
- Impact: consumer-code asymmetry. The README at line 181 claims
  "67 registered tools … accessible through `use echo_agent::prelude::*`",
  but a consumer who wants a direct `echo_tools::*` import (e.g. to
  avoid pulling the root facade's `tokio`/`reqwest` heavy deps) hits
  a dead end at `echo_agent::workspace::tools` (no such item).
- Root cause: likely an oversight when the workspace module was added.
  The module was authored before/around the `echo_tools` split and
  was not refreshed.
- Direction: either (a) add `pub use echo_tools as tools;` and
  `pub use echo_macros as macros;` to the `workspace` module, or
  (b) document why they are excluded. Option (a) is cheaper and
  consistent with the module's stated purpose.
- Regression validation: `cargo check --workspace --all-features`;
  add a doctest that imports `echo_agent::workspace::tools::shell::*`.
- Validation reports: [V01](../validations/F-API-01/V01-01.md).
- Note: this is the same defect as B-ARCH-01-P3-07, restated from
  the public-API-contract angle. The priority is raised from P3 to P2
  here because the facade-coherence task is the proper owner.

### F-API-01-P2-02: Facade exposes parallel access paths to the same items

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: see V02-01 for the full table. Six high-traffic items each
  reachable through 4-6 facade paths:
  - `Store` (trait): 6 paths — prelude, memory, memory::state,
    workspace::state::memory, echo_state::memory, echo_core::memory
    (canonical: `echo-core/src/memory/store.rs:182`).
  - `OpenAiClient`: 5 paths — prelude, llm, llm::providers,
    llm::integration::providers::openai, workspace::integration::
    providers::openai, echo_integration::providers::openai
    (canonical: `echo-integration/src/providers/openai.rs`).
  - `ProviderFactory`: 5 paths — prelude, llm, llm::integration::
    providers, workspace::integration::providers, echo_integration::
    providers (canonical: `echo-integration/src/providers/config.rs:363`).
  - `McpManager`: 4 paths — advanced, mcp, mcp::integration,
    workspace::integration::mcp, echo_integration::mcp
    (canonical: `echo-integration/src/mcp/mod.rs:56`).
  - `LocalSandbox`: 4 paths — prelude, sandbox,
    workspace::execution::sandbox, echo_execution::sandbox
    (canonical: `echo-execution/src/sandbox/local.rs:70`).
  - `TaskRevisionService`: 4 paths — advanced, tasks,
    tasks::orchestration, workspace::orchestration::tasks,
    echo_orchestration::tasks
    (canonical: `echo-orchestration/src/tasks/revisioned.rs:674`).
- Reachability: all paths resolve to the same item at compile time
  (Rust structural identity). Confirmed by reading each thin-re-export
  file:
  - `src/memory.rs:42-47` emits the same `pub use echo_state::memory::*`
    twice (once inside the nested `pub mod state { … }`, once at
    module root).
  - `src/llm.rs:54-82` is the most irregular, with five nested
    submodules (`core`, `integration`, `types`, `config`, `providers`)
    plus root-level re-exports of the same items.
  - `src/tasks.rs:8-11`, `src/human_loop.rs:8-11`,
    `src/mcp.rs:6-9`, `src/channels.rs:59-62` follow the same
    `pub mod <crate> { pub use echo_<crate>::…::*; }` + root
    `pub use echo_<crate>::…::*;` duplication.
- Expected invariant: each public item has one canonical facade path
  so that downstream code is uniform and changes touch one site.
- Observed behavior: four-or-more paths are live and (for `workspace`)
  documented as intentional migration scaffold.
- Impact: maintenance overhead — changing a type's source crate
  requires updating re-exports in multiple facade modules. Consumer
  code is inconsistent: the root README at lines 36, 84, 446, 563 uses
  different import paths for types that all resolve to the same item.
  Cross-reviewer grep confusion.
- Root cause: deliberate migration scaffold (the `workspace::*` module
  and the nested submodules inside thin-re-export files). Correct as
  a transitional measure; the debt is that the transition has not been
  completed.
- Direction: complete the migration (see B-ARCH-01-P2-01) and then
  remove the `workspace` escape hatch plus the nested submodules
  inside thin-re-export files. Until then, no action — the scaffold
  is doing its job.
- Regression validation: after removal, `cargo check --workspace
  --all-features`; update any consumer that imports through
  `workspace::*` or `<thin-module>::<subcrate>::*`.
- Validation reports: [V02](../validations/F-API-01/V02-01.md).
- Note: B-ARCH-01-P2-02 made this observation at the architecture
  level; this finding supplies the item-level evidence and is the
  proper owner from the public-API-contract perspective.

### F-API-01-P2-03: docs.rs metadata omits the `research` feature, hiding `echo_agent::tools::research` from rendered docs

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/Cargo.toml:60-64` —
    `[package.metadata.docs.rs] features = […22 features…]` does not
    include `research`.
  - `echo-agent/Cargo.toml` `[features]` table defines
    `research = ["echo_tools/research"]`.
  - `echo-agent/src/tools/mod.rs:94-97` —
    `#[cfg(feature = "research")] pub mod research { pub use
    echo_tools::research::*; }`. This is a public module gated by the
    feature.
- Reachability: the module is compiled when the feature is on (e.g.
  under `full`). docs.rs builds with the explicit list of 22 features
  and `no-default-features = true`, so docs.rs renders the crate
  *without* the `research` module. The module's `pub` items (scholarly
  search and reference-manager clients) are invisible to anyone
  browsing `https://docs.rs/echo_agent`.
- Expected invariant: docs.rs metadata covers every feature that gates
  a `pub` item in `src/`.
- Observed behavior: one user-visible `pub mod` (`research`) plus one
  behavioural cfg (`shell`, in `src/tools/builtin/spawn_task.rs:262`)
  are not enabled on docs.rs. See V04-01 for the full diff (11 missing
  features; 2 impactful, 1 justified by inline comment, 8 harmless).
- Impact: users who learn the framework via docs.rs miss the entire
  research-tool surface. The root README at line 181 claims "67
  registered tools across 8 crates", but the rendered docs show 66
  crates-worth (modulo the same feature-gating). Inconsistent
  advertisement.
- Root cause: docs.rs feature list was authored before the `research`
  feature was added (or it was missed in the list refresh).
- Direction: add `research` (and optionally `shell`) to the
  `[package.metadata.docs.rs] features = […]` list in `Cargo.toml`.
  Trivial two-line fix. (Adding `shell` is more conservative — it
  doesn't change visible `pub` items but makes the rendered SpawnTask
  behaviour match the default-config build.)
- Regression validation: render docs.rs preview locally with
  `cargo doc --no-deps --features "research,shell,…" --cfg docsrs`.
- Validation reports: [V04](../validations/F-API-01/V04-01.md).

### F-API-01-P3-01: Prelude mixes `crate::*`-relative and absolute `echo_core::*` imports

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/lib.rs:146` — `pub use echo_core::agent::PromptTemplateManager;`
  - `echo-agent/src/lib.rs:209` — `pub use echo_core::memory::{MemoryMeta,
    MemoryRisk, MemorySource, MemoryStatus, MemoryType};`
  - `echo-agent/src/lib.rs:256` — `pub use echo_core::circuit_breaker::
    {CircuitBreaker, CircuitBreakerConfig};`
  - All other 192 prelude items come from `crate::*`-relative paths.
- Reachability: prelude is the primary consumer surface; these 8 items
  bypass the root's thin-re-export modules.
- Expected invariant: the prelude's "go through the root" pattern
  (documented at `src/lib.rs:204-208` as "keeping the facade as the
  single integration surface") should apply uniformly.
- Observed behavior: 8 items break the pattern. The inline comment
  justifies it ("so downstream products … can reach them through the
  echo_agent facade without depending on echo_core directly"), but
  the same justification applies to every other echo_core type that
  *does* go through a thin-re-export module.
- Impact: cosmetic / maintainability. Inconsistent with the rest of
  the prelude. If `echo_core::agent::PromptTemplateManager` is renamed,
  the prelude breaks directly rather than via the (currently missing)
  root `crate::agent::PromptTemplateManager` indirection.
- Root cause: these were added directly to the prelude without first
  being added to a root thin-re-export module.
- Direction: either (a) re-route these 8 items through a root module
  (e.g. add `pub use echo_core::agent::PromptTemplateManager;` to
  `src/agent/mod.rs` and reference it as `crate::agent::*` from the
  prelude), or (b) accept the inconsistency and document it. Option
  (a) is preferred for consistency.
- Regression validation: `cargo check --workspace --all-features`.
- Validation reports: [V01](../validations/F-API-01/V01-01.md).

### F-API-01-P3-02: `workspace` module aliases use ambiguous short names

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/lib.rs:125-129` — aliases are `core`, `execution`,
    `integration`, `orchestration`, `state`. These collide with
    common Rust stdlib and ecosystem concepts (`core` ≈ Rust `core`
    crate; `state` is a generic word).
  - The thin-re-export files use a *different* short name for the same
    sub-crates: `src/memory.rs:42-45` names the nested submodule
    `state` (matching `workspace`), but `src/error.rs:9-12` names it
    `core`, `src/tasks.rs:8-11` names it `orchestration`,
    `src/mcp.rs:6-9` names it `integration`. Five different naming
    schemes across the facade.
- Reachability: public API at `echo_agent::workspace::*` and
  `echo_agent::<thin>::<subcrate>::*`.
- Expected invariant: one naming convention for the migration escape
  hatch.
- Observed behavior: at least four different naming schemes for the
  nested submodule (`state`, `core`, `integration`, `orchestration`,
  or none — see B-ARCH-01-P3-06 for the full list).
- Impact: maintenance cost and learning curve. Cosmetic; not a
  correctness issue.
- Root cause: each thin-re-export file was authored by a different
  pass of the migration; no shared convention was enforced.
- Direction: pick one convention (or delete the nested submodules
  entirely — the `workspace::*` module already provides the canonical
  escape hatch). Best handled as part of F-API-01-P2-02 when the
  migration scaffold is removed.
- Regression validation: `cargo check --workspace --all-features` to
  confirm no broken imports.
- Validation reports: [V01](../validations/F-API-01/V01-01.md),
  [V02](../validations/F-API-01/V02-01.md).
- Note: restates B-ARCH-01-P3-06 from the public-API angle.

### F-API-01-P3-03: Root `examples/*.rs` compile conceptually (positive finding)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/examples/demo00_quickstart.rs` — uses
    `echo_agent::prelude::*`, `echo_agent::{agent, tool}`, `agent!`
    macro, `ToolResult::success/.error`, `agent.execute(…)`. All
    resolved against current code (V03-01).
  - `echo-agent/examples/demo01_tools.rs` — uses `echo_agent::error::
    Result`, `echo_agent::prelude::*`, `echo_agent::tool`. All resolve.
  - `echo-agent/examples/demo02_tasks.rs` — uses `ReactAgentBuilder`
    methods `.model/.name/.system_prompt/.enable_tools/.enable_planning/
    .max_iterations/.build`, `agent.add_tool(Box::new(AddTool))`,
    `echo_agent::error::ReactError::Other`. All resolve (V03-01).
  - `echo-agent/examples/demo03_approval.rs` — uses
    `echo_agent::human_loop::{…}`, `echo_core::tools::permission::{…}`.
    All resolve (V03-01).
- Reachability: examples are registered as `[[example]]` entries in
  `Cargo.toml` (lines 163+) and can be invoked via
  `cargo run --example <name> --features <…>`.
- Expected invariant: canonical entry-point examples compile.
- Observed behavior: they do. This is a positive contrast to the
  sub-crate README examples, which B-ARCH-01-V04 found to be broken
  (echo-orchestration, echo-integration) or stale (echo-core, echo-
  state, echo-orchestration contents lists).
- Impact: the root examples are reliable. The sub-crate README
  examples are not. Inconsistent quality across documentation
  surfaces.
- Root cause: root examples are exercised by CI; sub-crate README
  snippets are not.
- Direction: convert the broken sub-crate README examples into
  doctests so they fail CI when they drift (already recommended by
  B-ARCH-01-P2-03/P2-04).
- Regression validation: keep the examples compiling; add a CI job
  that compiles all `[[example]]` entries with `--all-features`.
- Validation reports: [V03](../validations/F-API-01/V03-01.md).

### F-API-01-P3-04: `demo03_approval.rs` bypasses the facade for permission rule types

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/examples/demo03_approval.rs:18` —
    `use echo_core::tools::permission::{PermissionRule, RuleMatcher,
    RuleSource};` — direct sub-crate import, bypasses
    `echo_agent::*` facade.
  - `echo-agent/src/lib.rs:163-166` — prelude only re-exports
    `DefaultPermissionPolicy`, `PermissionDecision`,
    `PermissionPolicy`, `ToolPermission` from
    `crate::tools::permission`. The `PermissionRule`/`RuleMatcher`/
    `RuleSource` types are NOT in the facade.
  - `echo-core/src/tools/permission.rs:185,203` — `RuleSource` and
    related types live in `echo_core::tools::permission`.
- Reachability: the example compiles because `echo_core` is a
  transitive dependency, but the example sets a precedent that
  consumers should reach past the facade.
- Expected invariant: the facade documentation says "use
  `echo_agent::prelude::*` as the single integration surface"
  (README line 391). Examples should model that contract.
- Observed behavior: the canonical approval example demonstrates
  facade bypass as the supported pattern.
- Impact: consumers copy the example and acquire a direct `echo_core`
  Cargo dependency, defeating the facade's purpose. Coherence leak.
- Root cause: the facade's permission re-export set was authored
  before the rule-based permission API (PermissionRule/RuleMatcher/
  RuleSource) was added to `echo_core::tools::permission`. The
  facade was not refreshed.
- Direction: either (a) extend the prelude (or a non-gated module
  like `echo_agent::tools::permission`) to re-export `PermissionRule`,
  `RuleMatcher`, `RuleSource` and update the example to use the
  facade path; or (b) document that permission-rule types are only
  available via `echo_core` direct import. Option (a) is preferred.
- Regression validation: update demo03 to import via the facade;
  `cargo run --example demo03_approval --features human-loop`.
- Validation reports: [V03](../validations/F-API-01/V03-01.md).

### F-API-01-P3-05: Root README claims "67 registered tools" without per-feature breakdown

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/README.md:181` — "echo-agent ships with **67 registered
    tools** across 8 crates, all accessible through a single
    `use echo_agent::prelude::*`."
  - `echo-agent/src/lib.rs:173-180` — the prelude only re-exports
    WebFetchTool/WebSearchTool (web), ImageFetchTool/WebFetchToolEnhanced
    (media), and ThinkTool. The other 60+ tools are NOT in the prelude;
    they are reachable via `echo_agent::tools::<module>::*` paths
    gated by individual features (shell, files, git, chart, data,
    database, rag, research, lsp).
- Reachability: README claim.
- Expected invariant: README headline claims match the actual prelude
  export set, or are explicitly qualified ("67 tools across 8 crates,
  accessible via feature-gated module paths after
  `use echo_agent::prelude::*` brings in the agent runtime").
- Observed behavior: the unqualified claim "accessible through a
  single `use echo_agent::prelude::*`" overstates the prelude's role.
- Impact: users who take the claim literally expect all 67 tools to
  appear after the single prelude import. They don't — feature
  flags and explicit module paths are required.
- Root cause: README predates the per-feature modularisation and was
  not refreshed.
- Direction: qualify the claim. Replace "accessible through a single
  `use echo_agent::prelude::*`" with "accessible via
  `echo_agent::tools::<feature>::*` module paths after enabling the
  corresponding Cargo feature".
- Regression validation: render docs.rs preview; cross-check the tool
  count against the registered `[[example]]` and feature set.
- Validation reports: not separately validated (read-only spot check
  against the prelude export set in V01-01).
- Note: this is a documentation-precision issue, not a compile-time
  contract break. Severity is bounded; lower confidence because
  "67 tools" may be defensible as an aggregate count across features.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Facade re-export map (prelude + advanced + workspace counts and source-path mapping) | yes | passed_with_notes | [V01-01](../validations/F-API-01/V01-01.md) |
| V02 | Duplicate public concept search (per-item path enumeration) | yes | failed | [V02-01](../validations/F-API-01/V02-01.md) |
| V03 | Doctest and example sampling (conceptual compile of demo00/01/02/03) | yes | passed_with_notes | [V03-01](../validations/F-API-01/V03-01.md) |
| V04 | Feature / documentation consistency (Cargo `[features]` vs `[package.metadata.docs.rs]`) | yes | failed | [V04-01](../validations/F-API-01/V04-01.md) |
| V05 | Targeted executable compile check of `cargo run --example demoXX` | conditional | not_run | See Coverage section |

The V05 conditional validation was not run because it would require
building the root crate with non-default feature combinations, which
is out of scope for a read-only static review and is owned by F-FEAT-01.
The V03 conceptual check already establishes that the example symbol
set resolves.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root README line 391: "`use echo_agent::prelude::*` is all you need" | current | V01-01: prelude has 200 items covering agent/tools/llm/memory/compression/sandbox/trace/error/retry. The claim is accurate for the always-on surface. |
| Root README line 181: "67 registered tools … accessible through a single `use echo_agent::prelude::*`" | overstated | F-API-01-P3-05. Prelude only exports ~5 tool types directly; the rest are gated behind features and module paths. |
| `src/lib.rs:204-208` comment: "keeping the facade as the single integration surface" | stale | F-API-01-P3-01 (8 prelude items bypass root modules) and F-API-01-P3-04 (demo03 demonstrates facade bypass as a supported pattern). |
| `src/lib.rs:119-123` comment: `workspace` module is for "direct access to split workspace crates during migration" | current | V01-01 confirms 5 of 7 sub-crates are aliased. P2-01 records the asymmetry. |
| B-ARCH-01-P2-02: "Facade exposes parallel access paths to the same items" | current | V02-01 sharpens the B-ARCH-01 observation with item-level evidence (6 items × 4-6 paths each). |
| B-ARCH-01-P3-07: "`workspace` module aliases only 5 of 7 sub-crates" | current | Restated as F-API-01-P2-01; same evidence (`src/lib.rs:124-130`). |
| B-ARCH-01-P3-06: "Thin-re-export files use non-uniform nested-submodule conventions" | current | Restated as F-API-01-P3-02; same evidence. |
| B-ARCH-01-P2-03: echo-orchestration README example uses `GraphBuilder::new()` with no args | stale (not re-checked) | Inherited from B-ARCH-01; out of scope here (sub-crate README audit owned by B-DOC-01). |
| B-ARCH-01-P2-04: echo-integration README references `ProviderFactory::create_openai` | stale (not re-checked) | Inherited from B-ARCH-01; out of scope here. |
| `Cargo.toml:67-72` comment: "`data` feature excluded from docs.rs due to polars nightly-incompat" | current | V04-01 confirms `data` is excluded; polars upstream fix link is live in the comment. |

## Coverage And Uncertainty

Code not inspected deeply:

- Root `src/agent/` (the ReactAgent engine, callbacks, subagent) —
  symbol-level only. Behaviour review belongs to F-CORE-01 / F-LLM-01.
- Root `src/skills/`, `src/guard/`, `src/audit/`, `src/compression/`,
  `src/workflow/`, `src/sandbox/`, `src/trace/` — only the
  `pub use` lines at the module root (for prelude accounting). The
  internal pub surface of these modules is not enumerated.
- All other `examples/demoXX.rs` beyond demo00/01/02/03 — file listing
  only. There are 60+ examples; spot-checking the four canonical
  entry-points is sufficient for the public-facade-contract question.
- Sub-crate README files — already covered by B-ARCH-01-V04; this
  task inherits those findings and does not re-audit.
- `Cargo.lock` — not inspected; B-BASE-01 owns lockfile inventory.
- `.github/workflows/` — not inspected; CI inventory is B-BASE-01's
  scope.

Validations not run:

- No `cargo build`, `cargo doc`, or `cargo run --example` execution
  (read-only review). All "compiles conceptually" claims are based on
  direct comparison of the cited symbol against the source
  definition. The mismatches noted in V04 (research feature missing
  from docs.rs) are robust because they are based on a direct diff of
  two manifest stanzas; the V03 passes are robust because the symbol
  set is small and was exhaustively resolved.

Claims that remain uncertain:

- The exact count of "67 registered tools" in README line 181 was not
  independently verified. F-EXT-01 (tool inventory) owns that count.
- The 8 absolute `echo_core::*` imports in the prelude (P3-01) may
  have a defensible reason (the inline comment offers one) that
  raises the cost of fixing them beyond cosmetic. Confidence is high
  that the inconsistency exists; medium that fixing it is worth the
  churn.
- Whether `echo_tools::shell::CommandSafety` (gated by `shell` in
  `src/tools/builtin/spawn_task.rs:262`) is the *only* behavioural
  cfg hidden by the docs.rs omission. A full grep across all features
  was done for `src/` (V04-01) and found only `research` and `shell`
  with non-zero hits. Confidence: high.

## Handoff

Conclusions downstream tasks may rely on:

- The facade is structurally coherent: 200 prelude items + 69 advanced
  items + 5 crate aliases + 8 macros + 1 conditional module cover the
  public surface, and every `pub use` resolves. (V01)
- The facade is **not** path-coherent: 6 sampled items each have 4-6
  distinct import paths. (V02)
- The root `examples/*.rs` are reliable documentation. The sub-crate
  README examples are not (deferred to B-DOC-01). (V03)
- The docs.rs metadata hides the `research` module. Two-line fix in
  `Cargo.toml`. (V04)

Reports downstream tasks must read:

- F-MAC-01 (procedural macro contract) should read V01-01 to know
  which macros (`tool`, `agent`, `handler`, etc.) are re-exported at
  the facade top level vs. which are prelude-only. The 8-macro
  top-level re-export at `src/lib.rs:115-117` is the macro contract
  surface.
- F-FEAT-01 (feature topology) should read V04-01 — the
  features-vs-docs.rs diff is the starting point for the cfg-to-feature
  search. Specifically, F-FEAT-01 should confirm whether `shell` and
  `research` are the only features gating code in root `src/`.
- F-EXT-01 (tool inventory) should read F-API-01-P3-05 — the "67
  tools" README claim is the public contract that F-EXT-01's
  inventory should validate.
- B-DOC-01 (historical drift) should read V03-01 — the contrast
  between reliable root `examples/*.rs` and broken sub-crate README
  examples is the documentation-drift index.

Conditions that make this report stale:

- Any commit that adds `echo_tools`/`echo_macros` aliases to the
  `workspace` module invalidates P2-01.
- Any commit that removes the `workspace::*` module or unifies the
  thin-re-export file conventions invalidates P2-02 and P3-02.
- Any commit that adds `research` (and optionally `shell`) to the
  docs.rs features list invalidates P2-03.
- Any commit that re-routes the 8 absolute `echo_core::*` prelude
  imports through root modules invalidates P3-01.
- Any commit that refreshes the README to qualify the "67 tools …
  through prelude::*" claim invalidates P3-05.
- Any commit that extends the prelude or `tools::permission` module
  to cover `PermissionRule`/`RuleMatcher`/`RuleSource` invalidates
  P3-04.

Follow-up task IDs (recommended, not implemented in this review):

- A one-line manifest fix to add `research` and `shell` to
  `[package.metadata.docs.rs] features` (P2-03). Trivial; can be done
  in any cleanup commit.
- A one-line manifest fix to add `pub use echo_tools as tools;` and
  `pub use echo_macros as macros;` to the `workspace` module (P2-01).
  Trivial.
- A documentation pass to qualify README line 181 and refresh the
  sub-crate READMEs (P3-05; coordination with B-DOC-01).
- The larger migration-debt item (complete the sub-crate extraction
  so the `workspace` escape hatch and the thin-module nested
  submodules can be removed) is owned by B-ARCH-01's follow-up plan
  and is not in F-API-01's scope.
