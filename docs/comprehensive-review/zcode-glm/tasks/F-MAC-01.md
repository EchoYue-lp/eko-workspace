# F-MAC-01: Procedural macro contract

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (9b0e0faf74d35c9a432370b923acabfbb5f32d63)
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Do derive/attribute macros generate `Tool` and `Agent` code that obeys public
schemas, error handling, generics, hygiene, and feature boundaries?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-macros/src/lib.rs` (790 lines) — eight `proc_macro` entry
  points: `#[derive(Tool)]` (delegates to `derive_tool`), `#[tool]`,
  `#[callback]`, `#[guard]`, `#[handler]`, `#[compressor]`,
  `#[permission_policy]`, `#[audit_logger]`. Plus shared helpers
  `extract_fn_params`, `impl_block_to_boxfuture_methods`,
  `extract_boxfuture_methods_with_return`, `lifetimed_params`,
  `add_lifetime_a`, `require_return_type`, `extract_doc_comments`,
  `to_pascal_case`, `echo_agent_crate_path`.
- `echo-agent/echo-macros/src/derive_tool.rs` (496 lines) — `derive_tool_impl`
  and `generate_unit_tool` for `#[derive(Tool)]`, plus
  `resolve_echo_crate_path`, `ToolStructAttrs`, `ToolParamAttrs`,
  `parse_tool_param_attrs`, `extract_tool_attrs`.
- `echo-agent/src/macros.rs` (364 lines) — declarative macros `agent!`,
  `messages!`, `tool_params!`, `chat_request!`, plus private helpers
  `__tool_param_field!`, `__chat_request_field!`, and inline `#[cfg(test)]`
  unit tests.
- `echo-agent/echo-macros/Cargo.toml` — proc-macro crate manifest, deps
  (`syn`/`quote`/`proc-macro2`/`proc-macro-crate`), `[package.metadata.docs.rs]`.
- Target trait definitions the macros codegen against:
  - `echo-core/src/tools/mod.rs:733-903` — `Tool`, `ToolRunner<P>`,
    `ToolParameters = HashMap<String, serde_json::Value>` (line 554),
    `ToolRiskLevel`, `validate_parameters`, `permissions`, `risk_level`.
  - `echo-core/src/agent/mod.rs:920-990` — `AgentCallback` (8 lifecycle methods).
  - `echo-core/src/guard/mod.rs:62-76` — `Guard::check`.
  - `echo-core/src/compression.rs:446-454` — `ContextCompressor::compress`.
  - `echo-core/src/audit.rs:137-142` — `AuditLogger::log`/`query`.
  - `echo-core/src/tools/permission.rs:508-514` — `PermissionPolicy::check`
    (returns `PermissionDecision`, not `Result<PermissionDecision>`).
  - `echo-orchestration/src/human_loop/mod.rs:522-553` — `HumanLoopHandler`.
- Canonical usage fixtures (positive cases):
  - `echo-agent/examples/demo25_macros.rs` — exercises `#[tool]`,
    `#[callback]`, `#[guard]` and the four declarative macros end-to-end.
  - `echo-agent/examples/demo03_approval.rs`, `demo10_streaming.rs`,
    `demo12_resilience.rs`, `demo13_tool_execution.rs`,
    `demo45_customer_service.rs` — additional `#[tool]` call sites.
  - `echo-agent/echo-tools/src/git.rs:23-46`, `echo-tools/src/statistics.rs:21`
    — real `#[derive(Tool)]` usage in a sibling framework crate.

## Out Of Scope

Deferred to named task IDs:

- Declarative-macro behaviour beyond the contract surface (`agent!` builder
  chain completeness, `messages!`/`chat_request!` wire-shape correctness) →
  these are spot-checked for compile correctness here; their semantic
  equivalence to hand-written builders is owned by F-API-01 / F-CORE-01.
- The `Tool` trait's runtime contract (cancellation, pagination, artifacts) →
  owned by F-EXT-01. This task only checks that macros generate code that
  *matches* the trait shape.
- Generated JSON Schema *quality* (schemars 0.8 vs 0.9 dialect, optional vs
  nullable, description provenance) → partially covered in V01; deep schema
  audit belongs to a future tool-schema task.
- Feature-flag topology of the framework itself → F-FEAT-01. The macros do not
  emit `#[cfg(feature = ...)]` so the interaction is minimal (see V03).

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` — "framework vs
  application" rule, "first check if it already exists" rule, no-panic rule,
  UTF-8 safety rule, framework API deletion rule (echo-macros is a framework
  crate; its macros are public API surface and must not be removed without
  framework-wide evidence).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`.
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.

Dependency task reports read:

- `zcode-glm/tasks/F-API-01.md` — confirms the eight macros are re-exported
  at the facade top level (`echo-agent/src/lib.rs:115-117`) and identifies
  the `workspace` module's failure to alias `echo_macros` (F-API-01-P2-01).
  F-API-01's handoff note explicitly directs this task to inspect the
  8-macro top-level re-export as the macro contract surface.
- `zcode-glm/tasks/F-EXT-01.md` — establishes the canonical `Tool` /
  `ToolRunner` / `ToolResult` / `ToolFailure` contract that the generated
  impls must match. Its V01 inventory is the source of truth for trait
  method signatures cross-checked in V01 below.

Historical documents treated as hypotheses: the `echo-macros/src/lib.rs:1-31`
module-level doc table claiming each macro "generates" a specific trait impl,
and the per-macro doc comments promising a specific code shape.

## Layering Decision

Per the AGENTS.md "framework vs application" rule, every observation in this
report is classified at the **framework** layer. `echo_macros` is a generic
procedural-macro helper crate reusable by any consumer of the `echo-agent`
framework; nothing here touches EKO product policy. The framework-API
retention rule applies: this task does not recommend removing any macro; it
identifies contract gaps and hygiene issues whose fixes are local to
`echo-macros/src/`.

Repository-wide duplicate-search terms used (cross-crate):

- Macro entry points: `derive(Tool)`, `#[tool]`, `#[callback]`, `#[guard]`,
  `#[handler]`, `#[compressor]`, `#[permission_policy]`, `#[audit_logger]`.
- Codegen target traits: `Tool`, `AgentCallback`, `Guard`, `HumanLoopHandler`,
  `ContextCompressor`, `PermissionPolicy`, `AuditLogger`, `ToolRunner`.
- Attribute parsers: `ToolAttrs`, `NameAttr`, `ToolStructAttrs`,
  `ToolParamAttrs`, `ToolParamRaw`.
- Crate-path helpers: `echo_agent_crate_path`, `resolve_echo_crate_path`,
  `proc_macro_crate::crate_name`, `FoundCrate::Itself`, `FoundCrate::Name`.
- Result: one canonical definition site per macro in `echo-macros/src/`;
  one canonical definition site per target trait in `echo-core/src/` (and
  `echo-orchestration/src/human_loop/` for `HumanLoopHandler`). No parallel
  implementations found. Two **internal** duplications noted as findings
  (serde-error field extraction in `lib.rs` + `derive_tool.rs`; the `ToolAttrs`
  vs `ToolStructAttrs` parsers).

## Current Path

Macro inventory and codegen shape at commit `9b0e0fa`:

```text
echo_macros::lib.rs (8 proc_macro entries)
  ├─ #[derive(Tool)]      → derive_tool::derive_tool_impl
  │                          resolve_echo_crate_path(): echo_core 1st, echo_agent fallback
  │                          generates <Struct>Params struct + impl Tool
  │                          (named fields, unit struct, risk_level, permissions)
  ├─ #[tool]              → tool_impl
  │                          echo_agent_crate_path(): echo_agent only
  │                          generates <Fn>Params + <Fn>Tool + impl Tool
  │                          body inlined into Box::pin(async move { ... })
  ├─ #[callback]          → callback_impl
  │                          impl_block_to_boxfuture_methods (returns BoxFuture<'a, ()>)
  ├─ #[guard]             → guard_impl
  │                          generates <Name>Guard + impl Guard::check
  ├─ #[handler]           → handler_impl
  │                          extract_boxfuture_methods_with_return
  ├─ #[compressor]        → compressor_impl
  │                          generates <Fn>Compressor + impl ContextCompressor::compress
  ├─ #[permission_policy] → permission_policy_impl
  │                          generates <Fn>Policy + impl PermissionPolicy::check
  │                          (returns PermissionDecision, not Result)
  └─ #[audit_logger]      → audit_logger_impl
                             extract_boxfuture_methods_with_return

echo_agent::src::macros.rs (4 declarative macros, macro_export)
  ├─ agent!               → token-tree muncher → ReactAgentBuilder chain
  ├─ messages!            → Vec<Message> via Message::$role constructor
  ├─ tool_params!         → serde_json::Value JSON Schema
  └─ chat_request!        → ChatRequest struct literal
```

Crate-path resolution flow (the most important contract decision because it
governs whether the macro works in third-party crates):

```text
#[derive(Tool)]    consumer Cargo.toml
                     ↓
   resolve_echo_crate_path()
     ├─ crate_name("echo_core") Ok → ::echo_core (or renamed ::#ident)
     └─ crate_name("echo_core") Err
        ├─ crate_name("echo_agent") Ok → ::echo_agent
        └─ both Err → syn::Error "Cannot find `echo_core` or `echo_agent`"

#[tool], #[callback], #[guard], #[handler], #[compressor],
#[permission_policy], #[audit_logger]
                    consumer Cargo.toml
                     ↓
        echo_agent_crate_path()
        └─ crate_name("echo_agent") Ok → ::echo_agent
           Err → syn::Error (no echo_core fallback)
```

The two helpers disagree on the fallback policy (finding F-MAC-01-P3-02).

Codegen target verification (V01 confirms every signature matches):

```text
Generated impl method shape           Target trait shape (echo-core)
────────────────────────────────────  ─────────────────────────────────
Tool::execute<'a>(&'a self,           Tool::execute<'a>(&'a self,
  parameters: ToolParameters)           parameters: ToolParameters)
  -> BoxFuture<'a, Result<ToolResult>>  -> BoxFuture<'a, Result<ToolResult>>
  (lib.rs:261-267 / derive_tool.rs:    (echo-core/src/tools/mod.rs:754)
   381-389)                            ✓ match

Tool::validate_parameters<'a>(...)    Tool::validate_parameters<'a>(...)
  -> BoxFuture<'a, Result<()>>         -> BoxFuture<'a, Result<()>>
  (lib.rs:254-259 / derive_tool.rs:    (echo-core/src/tools/mod.rs:881)
   371-379)                            ✓ match (override of default impl)

Guard::check<'a>(&'a self,            Guard::check<'a>(&'a self,
  content: &'a str,                     content: &'a str,
  direction: GuardDirection)            direction: GuardDirection)
  -> BoxFuture<'a, Result<GuardResult>>(echo-core/src/guard/mod.rs:71)
  (lib.rs:410-414)                     ✓ match

PermissionPolicy::check<'a>(...)      PermissionPolicy::check<'a>(...)
  -> BoxFuture<'a, PermissionDecision> (echo-core/src/tools/permission.rs:509)
  (lib.rs:552-556)                     ✓ match (NOT Result<...>)

ContextCompressor::compress(         ContextCompressor::compress(
  &self, input: CompressionInput)       &self, input: CompressionInput)
  -> BoxFuture<'_, Result<...>>        -> BoxFuture<'_, Result<...>>
  (lib.rs:506-510)                     (echo-core/src/compression.rs:447)
                                       ✓ match

AgentCallback::on_*<'a>(&'a self, ...)  (echo-core/src/agent/mod.rs:920-990)
  -> BoxFuture<'a, ()>                  ✓ match (lifetime injected by
  (lib.rs:663-666)                       lifetimed_params for plain &self/&T)

HumanLoopHandler::on_approval/on_input (echo-orchestration/src/human_loop/
  via extract_boxfuture_methods_with_   mod.rs:524-532)
  return (lib.rs:691-694)               ✓ match for plain &self/&T params

AuditLogger::log/query                AuditLogger::log/query
  via extract_boxfuture_methods_with_  (echo-core/src/audit.rs:139-141)
  return                               ✓ match for owned-param shape
```

The shape match is the load-bearing positive result of this task: the
generated impls slot cleanly into the public traits.

## Findings

### F-MAC-01-P2-01: `#[derive(Tool)]` silently ignores struct generics, producing a downstream compile error with no macro-side diagnostic

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-macros/src/derive_tool.rs:228-230` — `let struct_ident =
    &input.ident; let params_ident = format_ident!("{}Params",
    struct_name_str);`. `input.generics` is never read.
  - `echo-agent/echo-macros/src/derive_tool.rs:358` — `impl
    #echo_crate::tools::Tool for #struct_ident { ... }` emits the impl with
    no `<...>` generic argument list.
  - `echo-agent/echo-macros/src/derive_tool.rs:387` — `<Self as
    #echo_crate::tools::ToolRunner<#params_ident>>::run(self, params).await`
    has the same gap.
  - `echo-agent/echo-macros/src/derive_tool.rs:353` — the generated
    `<Struct>Params` struct likewise omits any generics, so a field typed
    `T` becomes an unresolved identifier.
- Reachability: definition in `derive_tool.rs:226-415` → live caller in
  `echo-tools/src/git.rs:23` (`GitStatusTool` — non-generic, so the bug is
  dormant in-tree) and `echo-tools/src/statistics.rs:21`. A third-party
  consumer writing `#[derive(Tool)] struct Tool<T> { ... }` would hit it.
- Expected invariant (per AGENTS.md "no panic" and the macro doc string at
  `lib.rs:57-91` advertising derive support for "a struct definition"):
  the macro either supports generics or rejects them with a clear error
  pointing at the struct.
- Observed behavior: generics are dropped on the floor. The compiler error
  ("wrong number of generic arguments", "cannot find type `T`") is emitted
  at the macro-expansion site, not at the user's `struct` declaration.
- Impact: a generic tool struct is a plausible third-party use case (e.g.
  a generic JSON-fetch tool parameterised over response type). Today the
  user gets an opaque post-expansion error and has to read generated code
  to discover the macro does not support generics.
- Root cause: the macro was authored for the non-generic case (the only
  in-tree uses are non-generic) and was never extended to carry
  `input.generics` through to the impl and the params struct.
- Direction: either (a) thread `input.generics` and `input.vis` through
  both the `impl<...> Tool for Struct<...>` and the params struct, plus a
  `where` clause; or (b) detect `!input.generics.params.is_empty()` and
  emit a clear `syn::Error::new_spanned(&input.generics, "#[derive(Tool)]
  does not support generic types; remove the generic parameters")` up
  front. Option (b) is the cheap correctness fix; option (a) is the
  feature-complete fix.
- Regression validation: trybuild compile-fail test on `struct Tool<T> {
    x: T }` (see F-MAC-01-P2-03); existing `echo-tools` callers continue
  to compile under (a).
- Validation reports: [V03](../validations/F-MAC-01/V03-01.md).

### F-MAC-01-P2-02: `#[derive(Tool)]` and `#[tool]` generate `::schemars::*` paths, forcing every consumer crate to declare `schemars` as a direct dependency — undocumented

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-macros/src/lib.rs:236` — `#[derive(::serde::Deserialize,
    ::schemars::JsonSchema)]` on the generated params struct.
  - `echo-agent/echo-macros/src/lib.rs:248` — `::schemars::schema_for!(
    #params_name)` in the generated `parameters()` method.
  - `echo-agent/echo-macros/src/derive_tool.rs:351,363` — same two
    `::schemars::*` emissions for the derive path.
  - `echo-agent/echo-macros/src/lib.rs:249` and `derive_tool.rs:364` —
    `::serde_json::to_value(schema).unwrap_or_default()` also requires
    `serde_json` as a direct dep (but `serde_json` is more commonly
    already present).
  - Dependency manifest check: `schemars = "0.8"` appears in
    `echo-agent/Cargo.toml:121` and `echo-tools/Cargo.toml:49`, but **not**
    in `echo-core/Cargo.toml`, **not** in `echo-macros/Cargo.toml`, and the
    macro never re-exports it.
  - The crate-path helper `resolve_echo_crate_path`
    (`derive_tool.rs:37-62`) resolves `echo_core` or `echo_agent` but the
    literal `::schemars` is never substituted through it — it is hard-coded.
- Reachability: every consumer of either `#[derive(Tool)]` or `#[tool]` is
  affected. In-tree (`echo-tools`, root examples) it works because
  `schemars` is already a workspace dep; for a third-party consumer that
  adds only `echo_agent` (which transitively pulls `schemars` but does
  not re-export it) the path `::schemars::JsonSchema` triggers
  "unresolved import" / "crate `schemars` not found in this scope".
- Expected invariant: a procedural macro should either route its generated
  code through the resolved framework crate path (so the consumer needs
  only the one Cargo dependency they actually named), or its documentation
  must explicitly call out every transitive dependency the consumer must
  add.
- Observed behavior: the `echo-macros/src/lib.rs:1-31` module doc and the
  per-macro doc comments (`lib.rs:57-91, 169-191, 303-320, 372-386,
  425-447, 473-486, 521-531, 567-589`) say nothing about `schemars`.
  Consumers discover the requirement only when the build breaks.
- Impact: a framework that advertises "just add `echo_agent` and use
  `#[derive(Tool)]`" silently requires a second, version-pinned dep
  (`schemars = "0.8"`, not `0.9` — a mismatch fails to compile because
  the macro emits `::schemars::JsonSchema` whose shape differs between
  the two). The `0.8` vs `0.9` schema-dialect cliff is the more
  dangerous half of this finding because the failure mode is a confusing
  trait-bound error, not a missing-crate error.
- Root cause: the macro pre-dates the formalisation of the framework's
  public-dep surface; `schemars` was added to `echo-tools`/root as a
  workspace convenience without updating the macro to route through a
  re-export.
- Direction: re-export `schemars` (and `serde_json` if desired) from
  `echo_core` or `echo_agent` — e.g. `pub use schemars as __schemars;` —
  and substitute the resolved path in the generated code so the consumer
  needs only the framework dep. Cheaper alternative: add a "Required
  dependencies" section to the macro doc comments and the README listing
  `schemars = "0.8"`.
- Regression validation: a trybuild/smoke test in a *fresh* crate with
  only `echo_agent` (or only `echo_core`) in `Cargo.toml` that exercises
  both `#[tool]` and `#[derive(Tool)]`.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md),
  [V03](../validations/F-MAC-01/V03-01.md).

### F-MAC-01-P2-03: No `trybuild` compile-fail tests and no integration tests for procedural macros — only the declarative macros have inline unit tests

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `ls echo-agent/echo-macros/tests/` → directory does not exist.
  - `grep -rn "trybuild" echo-agent/**/Cargo.toml` → zero hits. No
    compile-fail test infrastructure is wired up anywhere in the workspace.
  - `echo-macros/src/lib.rs` and `echo-macros/src/derive_tool.rs` have no
    `#[cfg(test)]` block — no unit tests for any of the 8 proc macros.
  - The only macro tests in the workspace are `src/macros.rs:241-363`
    (`messages_macro_basic`, `messages_macro_single`, `messages_macro_empty`,
    `tool_params_macro_basic`, `tool_params_macro_all_required`,
    `tool_params_macro_none_required`, `chat_request_macro_basic`,
    `chat_request_no_options`, `agent_macro_basic`,
    `agent_macro_with_tools_and_options`) — declarative-macro only.
  - The positive case for the proc macros is covered indirectly by
    `examples/demo25_macros.rs` etc., but examples are not run by
    `cargo test --workspace` unless the caller opts in, and they provide
    no negative coverage.
- Reachability: definition side of the contract. No caller depends on
  these tests, but their absence is the reason several findings in this
  report (P2-01 generics, P3-02 asymmetric crate-path fallback, P3-06
  return-type validation) have stayed latent.
- Expected invariant: a framework shipping 8 procedural macros as public
  API should have (a) compile-pass fixtures for the documented shapes
  and (b) compile-fail fixtures for at least the common error cases
  (missing `name`, wrong attribute identifier, generic struct, missing
  `schemars` dep).
- Observed behavior: zero such fixtures. The macros are tested only by
  the downstream fact that `echo-tools` and the examples compile.
- Impact: regressions in macro expansion (e.g. removing `risk_level`
  support, changing the `Tool::execute` signature) are caught only when
  a downstream caller breaks. The contract documented in
  `echo-macros/src/lib.rs:1-31` is not enforced by any automated check.
- Root cause: proc-macro testing was never set up; the workspace relies
  on integration via `echo-tools` and examples as implicit coverage.
- Direction: add `trybuild = "1"` as a dev-dependency of `echo-macros`,
  create `echo-macros/tests/compile_pass/` (one fixture per macro
  mirroring the doc examples) and `echo-macros/tests/compile_fail/`
  (fixtures for the cases enumerated in V02 and V04). Run as part of
  `cargo test --workspace`.
- Regression validation: the new tests are the regression validation.
- Validation reports: [V04](../validations/F-MAC-01/V04-01.md).

### F-MAC-01-P3-01: Attribute macros `#[callback]`, `#[handler]`, `#[compressor]`, `#[permission_policy]`, `#[audit_logger]` silently discard their attribute arguments

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: all five bind `_attr` and ignore it:
  - `echo-macros/src/lib.rs:322` — `pub fn callback(_attr: TokenStream,
    item: TokenStream)`.
  - `echo-macros/src/lib.rs:449` — `pub fn handler(_attr, item)`.
  - `echo-macros/src/lib.rs:488` — `pub fn compressor(_attr, item)`.
  - `echo-macros/src/lib.rs:533` — `pub fn permission_policy(_attr, item)`.
  - `echo-macros/src/lib.rs:591` — `pub fn audit_logger(_attr, item)`.
- Reachability: live public API. A user writing `#[callback(foo = "bar")]`
  gets the same expansion as `#[callback]` with no warning.
- Expected invariant: an attribute macro that does not consume
  arguments should either reject non-empty attribute tokens with a
  `syn::Error` or document that arguments are ignored. Rust's lint
  `unused_attributes` does not fire here because the tokens are
  syntactically valid attribute arguments to the proc-macro attribute.
- Observed behavior: silent acceptance. Misspelled configuration,
  leftover arguments from a refactor, or copy-paste mistakes all pass
  without comment.
- Impact: low for the documented examples (none of the five advertise
  attribute arguments), but a foot-gun for users who assume the macros
  take arguments by analogy with `#[tool(name = "...")]` and
  `#[guard(name = "...")]` (which do).
- Root cause: the five macros were designed argument-free; the empty
  `()` form is the contract. The silent acceptance is an over-permissive
  default.
- Direction: in each of the five entry points, change to `attr:
  TokenStream` and emit `if !attr.is_empty() { return syn::Error::new(
  Span::call_site(), "this attribute takes no arguments").to_compile_error().into(); }`.
  Cheap, removes the foot-gun.
- Regression validation: trybuild compile-fail fixtures with
  `#[callback(unknown)]` etc. (blocked on F-MAC-01-P2-03).
- Validation reports: [V02](../validations/F-MAC-01/V02-01.md).

### F-MAC-01-P3-02: Crate-path resolution is asymmetric — `#[derive(Tool)]` falls back `echo_core → echo_agent`, attribute macros look up `echo_agent` only

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-macros/src/derive_tool.rs:37-62` — `resolve_echo_crate_path`
    tries `crate_name("echo_core")` first, on `Err` falls back to
    `crate_name("echo_agent")`, on `Err` of both emits a syn error.
  - `echo-macros/src/lib.rs:43-51` — `echo_agent_crate_path` calls only
    `crate_name("echo_agent")` and returns the underlying error on
    `Err`. No `echo_core` attempt.
  - `lib.rs:204, 331, 398, 458, 497, 542, 600` — every attribute macro
    calls `echo_agent_crate_path()?` to obtain the crate path.
- Reachability: a consumer crate that lists `echo_core` but not
  `echo_agent` in `Cargo.toml` can use `#[derive(Tool)]` but not
  `#[tool]`/`#[callback]`/etc. The seven attribute macros all hard-fail
  with "echo_agent not found".
- Expected invariant: all eight macros should resolve the framework
  crate through the same fallback chain. The `Tool`/`ToolRunner` traits
  themselves live in `echo_core`, so a consumer who only needs `Tool`
  codegen should not be forced to pull in the `echo_agent` facade just
  to satisfy a macro path.
- Observed behavior: `#[derive(Tool)]` works in `echo_core`-only crates;
  the attribute macros do not.
- Impact: forces a `echo_agent` dep on consumers who would otherwise be
  fine with the lighter `echo_core`. The asymmetry is undocumented. The
  `echo-tools` crate uses `#[derive(Tool)]` and depends on `echo_core`
  (`echo-tools/Cargo.toml`), which is exactly the case the asymmetric
  resolution advantages — but the same crate would break if it tried to
  use `#[callback]`.
- Root cause: `derive_tool.rs` was hardened to support `echo_core`-only
  consumers; the attribute-macro path in `lib.rs` was not.
- Direction: replace `echo_agent_crate_path` with a call to the existing
  `resolve_echo_crate_path` (move it to a shared `mod` and call from
  both sites), or duplicate the fallback. Either way the seven
  attribute macros gain `echo_core` fallback for free.
- Regression validation: trybuild smoke in an `echo_core`-only crate
  exercising `#[callback]`.
- Validation reports: [V03](../validations/F-MAC-01/V03-01.md).

### F-MAC-01-P3-03: Generated `pub struct` / `pub fn` visibility ignores the input's visibility — a private struct or async fn becomes a public Tool/params type

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-macros/src/lib.rs:236-241` — always `pub struct #params_name`
    and `pub struct #struct_name`, regardless of the input fn's
    visibility. `func.vis` is never consulted.
  - `echo-macros/src/derive_tool.rs:353` — `pub struct #params_ident`,
    regardless of `input.vis`.
  - `echo-macros/src/derive_tool.rs:465` — same in the unit-struct
    branch.
  - The generated `impl Tool for #struct_ident` is fine (impls inherit
    visibility from the self type), but the params struct and the
    generated helper struct (`<Fn>Tool`) leak to `pub`.
- Reachability: live. A user who writes `#[tool(...)] async fn add(...)`
    in a private module of their crate gets `pub struct AddTool` and
  `pub struct AddParams` in that module — exported past the module's
  private boundary if the module is `pub`, or merely over-permissive
  within a private module.
- Expected invariant: the generated items should adopt the visibility
  of the input (or at most one level more permissive). A private async
  fn should not produce a public tool struct.
- Observed behavior: all generated structs are `pub`.
- Impact: minor in current in-tree usage (the examples are at crate
  root, where `pub` is harmless), but a real encapsulation leak for
  library consumers embedding tools in internal modules.
- Root cause: macros were authored with the happy path (top-level
  examples) in mind; visibility propagation was not implemented.
- Direction: capture `let vis = &input.vis;` (derive) / `let vis =
  &func.vis;` (attribute) and substitute into the generated struct
  definitions instead of the literal `pub`.
- Regression validation: trybuild fixture with `pub(crate)` and private
  input, asserting the generated items inherit the same visibility.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md).

### F-MAC-01-P3-04: Error-field extraction parses serde error strings — brittle, duplicated between `#[tool]` and `#[derive(Tool)]`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-macros/src/lib.rs:274-291` — `deserialize_params` body: builds
    a `serde_json::Value::Object`, calls `serde_json::from_value`, on
    error parses `msg.strip_prefix("missing field \`").and_then(|s|
    s.strip_suffix('\`'))` to extract the field name.
  - `echo-macros/src/derive_tool.rs:394-409` — same body, copy-pasted
    verbatim (apart from `#echo_crate` vs `#echo_agent` and a stray
    space in `& parameters` at line 386).
  - Both fall back to the literal string `"(deserialization)"` when the
    pattern does not match.
- Reachability: every `validate_parameters` and `execute` call site
  that receives malformed JSON hits this code path.
- Expected invariant: error field extraction should not depend on
  serde's error-rendering format (which is an internal concern of serde
  and has changed across versions). The pattern is also incomplete: it
  handles "missing field `X`" and "... at line N col M" (via
  `msg.split(" at \`").nth(1)`), but does not handle serde's
  "invalid type: ... expected ..." form, which has no backtick-wrapped
  field name and falls through to the fallback.
- Observed behavior: works for the common missing-field case, silently
  degrades to `(deserialization)` for most other errors. A serde
  point-release that changes the error string shape breaks the
  extraction silently — no panic, just worse field names in
  `ToolError::InvalidParameter`.
- Impact: low today (the framework's primary consumers areEcho's own
  tools, which validate upstream), but the failure mode is silent
  quality degradation of error messages.
- Root cause: code was duplicated when `derive_tool.rs` was split out
  of `lib.rs`; the duplication has been preserved through subsequent
  edits. The string parsing is the simplest workaround for serde not
  exposing structured error metadata.
- Direction: extract a single helper (e.g.
  `fn serde_field_name(err: &serde_json::Error) -> &str`) into a shared
  private module, call from both sites. Optionally track
  `serde_json::Error::classify()` or the path field if/when serde
  exposes it.
- Regression validation: unit test on the helper with a corpus of
  serde error strings; compile-pass tests that the field name appears
  in `ToolError::InvalidParameter.name` for missing-field and
  wrong-type cases.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md),
  [V02](../validations/F-MAC-01/V02-01.md).

### F-MAC-01-P3-05: `generate_unit_tool` emits a duplicate `#[allow(dead_code)]` and a non-interpolated doc comment

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-macros/src/derive_tool.rs:467-468` — two consecutive
    `#[allow(dead_code)]` attributes on the same `impl` block. Cosmetic
    bug; rustc accepts the duplicate silently.
  - `echo-macros/src/derive_tool.rs:350` — `/// Auto-generated parameter
    struct for [\`#struct_ident\`].` is inside a `quote!` block. Doc
    comments in `quote!` become `#[doc = "literal string"]` attributes
    — the literal is *not* scanned for `#ident` interpolation. The
    generated doc literally reads "Auto-generated parameter struct for
    [`#struct_ident`]." with the text `#struct_ident` rather than e.g.
    "ReadFileTool". Same issue is absent from the unit-struct branch
    (`derive_tool.rs:462`) which does not interpolate.
- Reachability: the doc-comment defect is visible to anyone who runs
  `cargo doc` on a crate using `#[derive(Tool)]` on a named-fields
  struct. The duplicate attribute is invisible to consumers.
- Expected invariant: doc comments generated by macros should
  interpolate identifiers correctly (use `SetMutability::doc` /
  `quote_spanned!` or build the doc string via `format!` and emit
  `#[doc = #formatted]`). Duplicate attributes should not accumulate.
- Observed behavior: rustdoc renders the broken placeholder text.
- Impact: cosmetic. Confusing for users browsing generated docs.
- Root cause: copy-paste of the doc-comment template from a context
  where interpolation was assumed; the duplicate `#[allow(dead_code)]`
  is an editing artefact (the line was added twice).
- Direction: replace the literal doc comment with a constructed
  `#[doc = #doc_str]` where `doc_str = format!("Auto-generated
    parameter struct for [`{}`].", struct_name_str)`. Remove one of
  the two `#[allow(dead_code)]` lines.
- Regression validation: `cargo doc` on `echo-tools` and inspection of
  the rendered `GitStatusToolParams` page.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md).

### F-MAC-01-P3-06: `#[tool]` only checks for a non-`()` return type, not that the return type is `Result<ToolResult>` — type mismatches surface at the impl site

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-macros/src/lib.rs:212-217` — `if let ReturnType::Default =
    &func.sig.output { return Err(... "#[tool] function must have a
    return type (e.g., Result<ToolResult>)"); }`. Only the absence of a
    return type is rejected. Any other return type (`String`,
    `Result<String>`, `impl Future<...>`) is accepted and spliced into
    `Box::pin(async move { let params = ...?; let #params_name { ... } =
    params; #body })`.
  - `echo-macros/src/lib.rs:261-267` — the generated `execute` returns
    `BoxFuture<'a, #echo_agent::error::Result<#echo_agent::tools::ToolResult>>`.
    The body's actual return type must unify with that, else rustc
    emits a type error pointing at the expansion.
- Reachability: live. The validation gap is exercised by any user who
  writes `#[tool(...)] async fn foo() -> String { ... }`.
- Expected invariant: a macro that documents "must return
  `Result<ToolResult>`" should reject obviously-incompatible return
  types at the macro site, where the error span points at the user's
  fn signature, rather than letting the error surface in the generated
  `async move { ... }` block.
- Observed behavior: the diagnostic is delayed to the impl site, with
  a span pointing at the macro expansion. The user has to read
  generated code to discover that their `-> String` return type is
  incompatible.
- Impact: usability. No correctness defect (rustc catches it), but
  the error quality is poor.
- Root cause: validating "is `Result<ToolResult>`" requires type
  resolution, which proc macros cannot do; the macro would need to do
  a syntactic heuristic (`fn` returns something whose outermost type
  segment is `Result` and whose first generic argument's last segment
  is `ToolResult`). The check was never added.
- Direction: optional — add a syntactic check that the return type's
  outermost path ends in `Result` and emit a friendlier error.
  Otherwise, expand the existing error message to say "must be
  `Result<ToolResult>` (or compatible); got `<pretty-printed sig>`".
- Regression validation: trybulid compile-fail fixture with
  `async fn foo() -> String`.
- Validation reports: [V02](../validations/F-MAC-01/V02-01.md).

### F-MAC-01-P3-07: `lifetimed_params` / `add_lifetime_a` handle only plain `&self` and top-level `&T` / `&mut T` — exotic receivers and nested references are not supported

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-macros/src/lib.rs:703-717` — `lifetimed_params` matches
    `FnArg::Receiver` → `&'a self` and `FnArg::Typed` → applies
    `add_lifetime_a` to the type. Does not handle `self: Pin<&mut
    Self>`, `self: Box<Self>`, or arbitrary receiver types.
  - `echo-macros/src/lib.rs:723-741` — `add_lifetime_a` matches
    `syn::Type::Reference` at the top level only and recurses no
    further. `&(&str, &str)`, `&[&str]`, `&Vec<&str>`,
    `Option<&str>` (no top-level `&`) all pass through unchanged.
- Reachability: live for `#[callback]`, `#[handler]`, `#[audit_logger]`
  (all use these helpers). A user writing `async fn on_input(&self,
  prompts: &[&str])` for a custom handler gets `prompts: &[&str]` in
  the generated impl — no `'a` added to the inner `&str`. If the trait
  expects `&'a [&'a str]` the impl fails to compile; if the trait
  itself elides the inner lifetime (as `HumanLoopHandler::on_input`
  does for `&'a str`), the user gets away with it.
- Expected invariant: lifetime injection should either fully rewrite
  all elided lifetimes (deep) or refuse to guess and require the user
  to write explicit lifetimes.
- Observed behavior: shallow rewrite. Works for the trait signatures
  in `echo-core`/`echo-orchestration` today because those signatures
  use only top-level `&T` parameters, but any trait evolution toward
  nested-reference parameters would silently mis-inject.
- Impact: low today (all target traits use shallow references); medium
  as a forward-compatibility risk.
- Root cause: the helper was written to satisfy the current trait
  shapes, not to be a general-purpose lifetime rewriter.
- Direction: document the limitation in the macro doc comments ("use
  only top-level `&T` parameters in method signatures"), or extend
  `add_lifetime_a` to recurse into reference elements and slice
  elements. The documentation fix is cheaper and matches actual use.
- Regression validation: trybulid compile-pass fixtures mirroring
  every target trait method exactly.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md),
  [V03](../validations/F-MAC-01/V03-01.md).

### F-MAC-01-P3-08 (positive): Macro-generated method signatures match all seven target traits; crate-path resolution handles crate rename; the documented contract holds for the documented shapes

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - Cross-check of every generated method signature against its target
    trait (full table in V01-01). All seven traits (`Tool`,
    `AgentCallback`, `Guard`, `HumanLoopHandler`, `ContextCompressor`,
    `PermissionPolicy`, `AuditLogger`) have matching lifetime, parameter,
    and return-type shape.
  - Notably, `#[permission_policy]` correctly generates `BoxFuture<'a,
    PermissionDecision>` (not `Result<PermissionDecision>`) at
    `lib.rs:552-556`, matching `PermissionPolicy::check` at
    `echo-core/src/tools/permission.rs:509-513`.
  - `#[derive(Tool)]` correctly delegates to `ToolRunner::run` via
    `<Self as #echo_crate::tools::ToolRunner<#params_ident>>::run(self,
    params).await` at `derive_tool.rs:387`, matching the `ToolRunner<P>`
    helper trait at `echo-core/src/tools/mod.rs:733-736`.
  - Crate-rename handling: both `echo_agent_crate_path` and
    `resolve_echo_crate_path` handle `FoundCrate::Itself` (returns
    `::echo_agent`/`::echo_core`) and `FoundCrate::Name(name)` (returns
    `::#ident`), so a consumer that renames the dep (`ea = { package =
    "echo_agent" }`) gets the correct path substituted.
  - `#[tool]` correctly extracts doc comments from fn args
    (`extract_fn_params` at `lib.rs:615-648`) and emits
    `#[schemars(description = "...")]` so the generated JSON Schema
    carries the doc text.
  - `#[derive(Tool)]` correctly handles `#[tool_param(skip)]`
    (`parse_tool_param_attrs` at `derive_tool.rs:145-167`) so internal
    state fields are not exposed to the LLM-facing params struct.
- Reachability: the documented happy paths all work.
  `echo-tools/src/git.rs:23-46` and `examples/demo25_macros.rs` are
  the in-tree demonstrations.
- Expected invariant: the macro doc table at `echo-macros/src/lib.rs:1-31`
  accurately describes the codegen.
- Observed behavior: it does. The doc claims are current.
- Impact: positive. The contract documented in the module-level rustdoc
  is honoured for the shapes the macros advertise.
- Root cause: n/a (positive finding).
- Direction: keep the doc table in sync if new macros are added; the
  latent gaps are the generics/visibility/schemars issues above.
- Regression validation: the trybulid compile-pass suite recommended
  in F-MAC-01-P2-03 should pin every entry in the doc table.
- Validation reports: [V01](../validations/F-MAC-01/V01-01.md),
  [V03](../validations/F-MAC-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Expansion/API mapping — what macros exist, what code they generate, signature match against target traits | yes | passed_with_notes | [V01-01](../validations/F-MAC-01/V01-01.md) |
| V02 | Invalid-input diagnostics — missing fields, wrong types, unknown attribute idents; clarity of macro-emitted errors | yes | passed_with_notes | [V02-01](../validations/F-MAC-01/V02-01.md) |
| V03 | Generic / rename / crate-path fixtures — struct generics, crate rename, `echo_core`-vs-`echo_agent` fallback, feature-gated codegen | yes | failed | [V03-01](../validations/F-MAC-01/V03-01.md) |
| V04 | Compile-pass and compile-fail report — existing test inventory, edge-case coverage gaps | yes | failed | [V04-01](../validations/F-MAC-01/V04-01.md) |
| V05 | Targeted executable compile check of `cargo check -p echo_macros` / `cargo run --example demo25_macros` | conditional | not_run | See Coverage section |

V05 (the executable compile check) was not run because (a) it is owned by
the F-FEAT-01 / F-TST-01 matrix, and (b) the static signature comparison in
V01 plus the inventory in V04 together establish the contract-status
without requiring a workspace build. The `examples/demo25_macros.rs` file
is registered as an `[[example]]` and is exercised by the CI matrix
already.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-macros/src/lib.rs:9` "`#[tool]` generates `Tool` impl" | current | V01-01: `impl Tool for #struct_name` emitted at `lib.rs:243`. |
| `echo-macros/src/lib.rs:10` "`#[callback]` generates `AgentCallback` impl" | current | V01-01: `impl AgentCallback for #self_ty` at `lib.rs:336`. |
| `echo-macros/src/lib.rs:11` "`#[guard]` generates `Guard` impl" | current | V01-01: `impl Guard for #struct_name` at `lib.rs:407`. |
| `echo-macros/src/lib.rs:12` "`#[handler]` generates `HumanLoopHandler` impl" | current | V01-01: `impl HumanLoopHandler for #self_ty` at `lib.rs:463`. |
| `echo-macros/src/lib.rs:13` "`#[compressor]` generates `ContextCompressor` impl" | current | V01-01: `impl ContextCompressor for #struct_name` at `lib.rs:506`. |
| `echo-macros/src/lib.rs:14` "`#[permission_policy]` generates `PermissionPolicy` impl" | current | V01-01: `impl PermissionPolicy for #struct_name` at `lib.rs:551`, returning `PermissionDecision` (not `Result`). |
| `echo-macros/src/lib.rs:15` "`#[audit_logger]` generates `AuditLogger` impl" | current | V01-01: `impl AuditLogger for #self_ty` at `lib.rs:605`. |
| `echo-macros/src/lib.rs:16` "`#[derive(Tool)]` generates `Tool` impl from a struct" | current with caveats | V01-01: `impl Tool for #struct_ident` at `derive_tool.rs:358`. Caveats: generics unsupported (P2-01), visibility not propagated (P3-03). |
| `echo-macros/src/lib.rs:57-91` "`#[derive(Tool)]` supports `#[tool(risk_level = "ReadOnly\|Standard\|Dangerous")]`" | current | V01-01: `derive_tool.rs:310-332` parses and codegens `risk_level`. Invalid value rejected at `derive_tool.rs:315-323`. |
| `echo-macros/src/lib.rs:65-69` "`#[tool_param(skip)]` and `#[tool_param(description = "...")]`" | current | V01-01: `derive_tool.rs:175-200` (parser) + `derive_tool.rs:276-293` (codegen). |
| `echo-macros/src/lib.rs:20-27` Quick Example for `#[tool]` (`async fn add(a: f64, b: f64) -> Result<ToolResult>`) | current | V03-01: example compiles; verified by `examples/demo25_macros.rs:17-25` (same shape). |
| `echo-macros/src/lib.rs:29-31` "Most users should import these macros via `echo_agent::prelude::*` or `use echo_agent::{tool, callback, guard, handler};`" | current | F-API-01 V01-01: top-level re-export at `src/lib.rs:115-117` exposes all 8 macros; `use echo_agent::tool` works. |
| `echo-macros/src/lib.rs:85-91` `#[derive(Tool)]` doc example uses `ToolRunner<ReadFileToolParams>` | current | V01-01: codegen at `derive_tool.rs:387` matches the documented `ToolRunner` delegation pattern; `echo-tools/src/git.rs:36-46` exercises the same shape. |
| Implicit assumption that consumer needs only `echo_agent` (or `echo_core`) as a direct dep | stale | F-MAC-01-P2-02: consumer must also add `schemars = "0.8"` because the macro emits `::schemars::JsonSchema` and `::schemars::schema_for!`. |

## Coverage And Uncertainty

Code not inspected deeply:

- `echo-macros`'s generated JSON Schema *quality*: V01 confirms the macro
  emits `#[derive(::schemars::JsonSchema)]` + `schema_for!`, but the
  resulting schema's handling of `Option<T>` (nullable vs absent),
  description provenance, and 0.8-vs-0.9 dialect was not exhaustively
  verified. Owned by a future tool-schema task.
- Macro interaction with the application adapter (`echo-agent-app-core`)
  and EKO-side tool registration. Application code never invokes these
  macros directly (EKO tools are written by hand against `impl Tool`),
  so the macro contract is framework-consumer-facing only.
- The `agent!` declarative macro's full option matrix (every
  `(@build ...)` arm at `src/macros.rs:31-93`) was inspected at the
  contract level (each arm maps to a known `ReactAgentBuilder` method)
  but not exhaustively tested against the current `ReactAgentBuilder`
  API surface; that cross-check belongs to F-CORE-01.

Validations not executed at runtime:

- No `cargo check -p echo_macros`, `cargo run --example demo25_macros`,
  or `cargo test` runs. All claims are static inspections of source at
  commit `9b0e0fa`. The signature-match claim in V01 is robust because
  it is a direct side-by-side comparison of generated tokens (read from
  the `quote!` block) against the trait declaration (read from
  `echo-core/src/`). The generics/visibility/schemars findings in P2-01,
  P2-02, P3-03 are structural facts (the relevant field is simply never
  read or the relevant path is hard-coded) and do not require execution
  to confirm.

Claims that remain uncertain:

- F-MAC-01-P3-07 (shallow lifetime rewriting): whether the target
  traits evolve to include nested-reference parameters. Today they do
  not, so the helper works; the finding is forward-looking. Confidence
  medium.
- F-MAC-01-P2-02 (`schemars` direct-dep requirement): whether
  `echo_agent` already re-exports `schemars` under a non-obvious path
  (`echo_agent::schemars::*` or via the prelude). A grep was performed
  and no such re-export was found, but the prelude's 200 items were
  not exhaustively cross-checked against `schemars` types. Confidence
  high that no re-export exists; medium on the practical impact (some
  consumers may already have `schemars` for unrelated reasons).
- Whether any consumer outside the workspace actually uses these macros
  with crate-rename (`ea = { package = "echo_agent" }`). The
  `FoundCrate::Name(name)` branch at `lib.rs:46-49` and
  `derive_tool.rs:40-43` is written for that case but in-tree nothing
  exercises it; the rename path is verified by code reading only.

## Handoff

Conclusions downstream tasks may rely on:

- The 8 procedural macros in `echo_macros` generate trait impls whose
  method signatures match their `echo-core`/`echo-orchestration` target
  traits. Downstream tasks can rely on the contract documented in
  `echo-macros/src/lib.rs:1-31` for the happy-path shapes. (V01)
- Crate-rename handling is correct (`FoundCrate::Itself` and
  `FoundCrate::Name` both produce valid paths). F-FEAT-01 can rely on
  this for the crate-alias feature-topology analysis. (V03)
- The `Tool`/`ToolRunner`/`ToolResult` contract established by F-EXT-01
  is honoured by both `#[tool]` and `#[derive(Tool)]` codegen. F-EXT-02
  (builtin tool implementations) can treat `#[derive(Tool)]` as a
  first-class way to write builtin tools — `echo-tools/src/git.rs:23`
  already demonstrates the pattern.

Reports downstream tasks must read:

- F-EXT-02 (builtin tool implementations) should read V01-01 — the
  `#[derive(Tool)]` codegen pattern (`<Struct>Params` + `impl Tool` +
  `ToolRunner::run` delegation) is the recommended way to write new
  builtin tools and the audit should cross-check that builtin tools
  follow it.
- F-FEAT-01 (feature topology) should read V03-01 — the macros emit no
  `#[cfg(feature = ...)]` themselves, so they do not affect the
  cfg/feature matrix directly; but the `schemars` direct-dep requirement
  (P2-02) is a feature-graph fact that F-FEAT-01's manifest audit
  should account for.
- B-DOC-01 (historical drift) should read V04-01 — the macro test gap
  (P2-03) is the reason the documented macro contract has drifted
  silently in the past (e.g. the duplicate `#[allow(dead_code)]` and
  the broken doc-comment interpolation at `derive_tool.rs:350` would
  have been caught by a compile-pass suite).
- F-API-01 (facade contract) already noted in its handoff that
  F-MAC-01 should read V01-01 for the 8-macro top-level re-export.
  This report confirms the re-export is correct and the macros
  themselves are the contract surface.

Conditions that make this report stale:

- Any commit that adds generic-type support to `#[derive(Tool)]`
  invalidates F-MAC-01-P2-01 and the V03 failure.
- Any commit that re-exports `schemars` from `echo_core` or
  `echo_agent` (or routes the macro's `::schemars::*` paths through
  the resolved framework crate) invalidates F-MAC-01-P2-02.
- Any commit that adds `trybuild` compile-fail tests to `echo-macros`
  invalidates F-MAC-01-P2-03 and the V04 failure.
- Any commit that unifies `echo_agent_crate_path` and
  `resolve_echo_crate_path` into a single helper with `echo_core`
  fallback invalidates F-MAC-01-P3-02.
- Any change to a target trait (`Tool`, `AgentCallback`, `Guard`,
  `HumanLoopHandler`, `ContextCompressor`, `PermissionPolicy`,
  `AuditLogger`) signature invalidates V01 and the corresponding
  match row in the Current Path table.
- Any commit that propagates `input.vis` / `func.vis` into the
  generated structs invalidates F-MAC-01-P3-03.

Follow-up task IDs (recommended, not implemented in this review):

- A trybuild test sweep for `echo-macros` covering the eight macros'
  documented shapes plus the negative cases enumerated in V02 and V04
  (P2-03). This is the single highest-leverage follow-up because it
  pins every other contract claim in this report.
- A two-line fix to thread `input.generics` and `input.vis` through
  `#[derive(Tool)]` (P2-01, P3-03), or alternatively an up-front
  `syn::Error` rejecting generics until proper support lands.
- A one-line re-export (`pub use schemars as __schemars;` in
  `echo_core` or `echo_agent`) plus substitution in the macro's
  `quote!` blocks to retire the `schemars` direct-dep requirement
  (P2-02). Alternatively a doc-comment disclosure.
- Unification of `echo_agent_crate_path` with
  `resolve_echo_crate_path` to give attribute macros `echo_core`
  fallback (P3-02).
