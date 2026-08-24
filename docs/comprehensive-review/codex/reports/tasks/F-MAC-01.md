# F-MAC-01: Procedural macro contract

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; `echo-agent-cli` concurrently dirty only under `web-frontend/src/generated/*.ts` (not read or modified by this task); review artifacts outside source repositories

## Question

Do derive and attribute macros generate Tool/Agent code that obeys public
schemas, error handling, generics, hygiene, diagnostics, and feature boundaries?

## Scope

- `echo-agent/echo-macros/src/lib.rs` and `derive_tool.rs`.
- Root facade procedural-macro exports and generated trait paths.
- `echo_core::tools::{Tool, ToolRunner}`, callback, guard, compression, audit,
  permission, and human-loop trait signatures used by expansion.
- Real `echo_tools` derive consumers plus isolated external Cargo fixtures for
  dependency rename, hidden deps, generics, diagnostics, cfg, and schema edges.

## Out Of Scope

- Declarative `agent!`, `messages!`, `tool_params!`, and `chat_request!` macros.
- Generic Tool runtime semantics, registry behavior, permission policy, and
  tool-schema policy owned by `F-EXT-01`.
- Public documentation defects owned by `F-API-01` except classification as
  historical input.
- Feature-matrix completeness owned by `F-FEAT-01`.
- Source fixes, index changes, network access, or full workspace submission gate.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, task card, and Codex
  reviewer protocol.
- Accepted [B-ARCH-01](B-ARCH-01.md), limited to its macro layering finding.
- In-progress F-API-01 was inspected only to avoid duplicating its current public
  documentation scope; its findings are not dependencies and are not treated as
  accepted.
- No F-EXT-01 Codex task report existed, so this task did not consume one.
- No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Macro input parsing, crate-path resolution, generated trait/schema code, hygiene, generics, and diagnostics are framework mechanisms. |
| EKO product policy | None. EKO has no direct proc-macro consumer in the reviewed Rust source and should not own repairs. |
| Adapter boundary | The root `echo_agent` re-export is a facade adapter. A re-exported macro must generate only facade-reachable paths or explicitly be absent; split consumers may use `echo_core + echo_macros` directly. |
| Duplicate search | Searched all eight macro names, trait names, re-exports, derive/attribute consumers, resolver helpers, generated external paths, generics APIs, compile-test frameworks, and Markdown claims across both repositories. |
| Migration deletion | Unify the two crate resolvers and one generated Tool template. Delete facade-only attribute resolution already owned by B-ARCH-01, duplicated derive unit/named emission, silent helper parsing, and ignored-only doctest coverage after compile fixtures replace them. |

B-ARCH-01-P1-01 already owns the fact that seven attribute macros require the
facade even where split crates should suffice. This report does not duplicate
that finding; it expands the matrix and reports different generated-contract
failures.

## Current Path

```text
echo_agent facade
  -> pub use echo_macros::{Tool, tool, callback, guard, handler,
                           compressor, permission_policy, audit_logger}

attribute macro
  -> parse syn ItemFn/ItemImpl
  -> echo_agent_crate_path() (facade only)
  -> synthesize trait impl/body
       -> direct ::futures paths
       -> #[tool] also direct ::serde/::schemars/::serde_json

derive(Tool)
  -> resolve_echo_crate_path() (echo_core first, facade fallback)
  -> parse struct/tool/tool_param metadata
  -> synthesize <Struct>Params + Tool impl
       -> direct ::serde/::schemars/::serde_json/::futures
       -> calls <resolved>::tools::ToolRunner<Params>

real split consumer
  echo_tools::{git,statistics}
    -> direct echo_core + echo_macros + helper deps -> compiles

facade derive consumer
  echo_agent::Tool expansion
    -> ::echo_agent::tools::ToolRunner -> absent -> compile failure
```

Valid non-generic inputs for all seven attribute macros compile when every helper
dependency and the `human-loop` feature are explicitly supplied. Renamed facade
and renamed core dependencies also compile. These positive controls bound the
findings to public facade completeness and specific input edges.

## Findings

### F-MAC-01-P1-01: Facade-reexported derive(Tool) always targets a missing trait

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/lib.rs:115`, `echo-agent/src/tools/mod.rs:104`,
  `echo-agent/echo-macros/src/derive_tool.rs:387`,
  `echo-agent/echo-core/src/tools/mod.rs:733`
- Reachability: the root publicly re-exports `Tool`; facade users can invoke it,
  but derive's facade fallback emits `::echo_agent::tools::ToolRunner`. The
  facade does not re-export `ToolRunner`. A facade-only fixture and a second
  fixture with all helper crates plus `workspace::core::tools::ToolRunner` both
  fail; the latter has exactly one remaining E0405 at generated code.
- Expected invariant: every facade-reexported macro compiles against the facade
  paths it generates for a valid supported input.
- Observed behavior: no valid facade-side derive consumer can satisfy the fixed
  generated path without modifying the facade itself. Direct `echo_core +
  echo_macros` consumers do compile.
- Impact: one of eight public proc macros is unusable through the advertised
  single-crate API; consumers must discover and depend on split crates directly.
- Root cause: derive added a facade fallback, but the facade export map did not
  expose the helper trait referenced by that fallback.
- Direction: decide the supported surface explicitly. Prefer re-exporting the
  single core Tool contract consistently through the facade and generating
  against that path, or remove facade derive export/document split-only use.
  Delete the fallback branch if the facade contract is not supported.
- Regression validation: facade-only named/unit derives compile and execute with
  the documented dependency set; direct and renamed core probes remain green.
- Validation reports: [V02-01](../validations/F-MAC-01/V02-01.md),
  [V04-21](../validations/F-MAC-01/V04-21.md),
  [V04-22](../validations/F-MAC-01/V04-22.md),
  [V20-01](../validations/F-MAC-01/V20-01.md)

### F-MAC-01-P2-02: Generated code leaks four undeclared helper crates into consumers

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-macros/src/lib.rs:236`,
  `echo-agent/echo-macros/src/lib.rs:247`,
  `echo-agent/echo-macros/src/lib.rs:254`,
  `echo-agent/echo-macros/src/derive_tool.rs:351`
- Reachability: the root re-exports attribute/derive macros and public examples
  present facade use. Expansions name `::serde`, `::schemars`, `::serde_json`,
  and `::futures`; transitive dependencies are not in an external crate's extern
  prelude. The minimal attribute fixture fails four E0433 errors and passes only
  after declaring all four directly. Minimal derive reports the same four.
- Expected invariant: a re-exported macro works with documented direct
  dependencies, or its package contract explicitly requires each generated path.
- Observed behavior: even a facade-only `#[tool]` requires four hidden direct
  dependencies unrelated to imports in the user's function.
- Impact: canonical consumer code fails despite the facade carrying those
  dependencies transitively; generated implementation details leak into every
  downstream manifest and create version-coupling.
- Root cause: macro output uses global third-party crate paths instead of
  framework-owned re-export paths or generated code using core aliases.
- Direction: establish a private/public macro support module in the owning
  framework crate and generate through its re-exports, or explicitly reduce the
  macro API to split-crate consumers with a complete dependency contract. Delete
  direct helper paths once the authority is centralized.
- Regression validation: facade-only and core+macros-only fixtures compile for
  attribute and derive; renamed helper dependencies do not affect expansion.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md),
  [V04-01](../validations/F-MAC-01/V04-01.md),
  [V04-02](../validations/F-MAC-01/V04-02.md),
  [V04-21](../validations/F-MAC-01/V04-21.md),
  [V20-02](../validations/F-MAC-01/V20-02.md)

### F-MAC-01-P2-03: Macro expansion discards generics and where clauses

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-macros/src/lib.rs:203`,
  `echo-agent/echo-macros/src/lib.rs:330`,
  `echo-agent/echo-macros/src/lib.rs:651`,
  `echo-agent/echo-macros/src/derive_tool.rs:226`,
  `echo-agent/echo-macros/src/derive_tool.rs:349`
- Reachability: all APIs accept general `syn::ItemFn`, `ItemImpl`, or
  `DeriveInput` without rejecting generics. Generated params/impls use only
  identifiers and rewritten arguments; no `split_for_impl`, type parameters, or
  where clauses are emitted. Generic derive, generic tool function, and generic
  callback impl independently fail with `T` absent from generated scope.
- Expected invariant: macros either preserve valid Rust generics/where clauses
  or reject unsupported generic input at the macro span with an explicit message.
- Observed behavior: they accept input then emit secondary E0425/E0107 errors
  that expose generated internals.
- Impact: reusable generic tools/callbacks cannot use the macros, and developers
  receive misleading compiler errors rather than a stable supported-input contract.
- Root cause: expansion reconstructs signatures/impls from fragments instead of
  retaining `syn::Generics`, `split_for_impl`, and where clauses.
- Direction: preserve generics and bounds in every generated type/impl, including
  params generics and lifetimes. Where semantics cannot be supported, reject at
  parse time. Delete ad hoc signature reconstruction once a shared typed emitter
  owns it.
- Regression validation: type, lifetime, const-generic, where-clause, generic
  callback/handler/audit, and generic async tool compile-pass/fail UI fixtures.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md),
  [V04-05](../validations/F-MAC-01/V04-05.md),
  [V04-06](../validations/F-MAC-01/V04-06.md),
  [V04-14](../validations/F-MAC-01/V04-14.md),
  [V20-02](../validations/F-MAC-01/V20-02.md)

### F-MAC-01-P2-04: Invalid tool_param metadata is silently treated as absent

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-macros/src/derive_tool.rs:145`,
  `echo-agent/echo-macros/src/derive_tool.rs:153`,
  `echo-agent/echo-macros/src/derive_tool.rs:189`
- Reachability: every derive field calls `parse_tool_param_attrs`; the inner
  parser can return a precise error, but the caller retains only `Ok`. A runnable
  fixture with `#[tool_param(nonsense)]` compiles and emits a field schema without
  the intended metadata.
- Expected invariant: malformed helper attributes stop compilation at the field
  with an actionable diagnostic.
- Observed behavior: syntax/unknown-key/type errors are indistinguishable from no
  helper attribute and silently change the generated schema.
- Impact: a typo such as `skip`/`description` misspelling can expose internal
  state as an LLM parameter or omit schema guidance while the build remains green.
- Root cause: the helper returns a non-fallible value and intentionally discards
  `attr.parse_args` errors.
- Direction: return `syn::Result<ToolParamAttrs>`, combine multiple attribute
  diagnostics, and propagate errors from derive. Delete the `if let Ok` fallback.
- Regression validation: unknown key, missing value, wrong literal type,
  duplicate/conflicting metadata, and valid doc fallback compile-fail/pass snapshots.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md),
  [V04-07](../validations/F-MAC-01/V04-07.md),
  [V20-03](../validations/F-MAC-01/V20-03.md)

### F-MAC-01-P2-05: User-controlled generated names can panic the proc macro

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-macros/src/lib.rs:400`,
  `echo-agent/echo-macros/src/lib.rs:777`
- Reachability: `#[guard(name = ...)]` transforms the user string then passes it
  to `format_ident!`. `#[guard(name = "1-invalid")]` reaches this path.
- Expected invariant: arbitrary string literals are validated and return a
  `syn::Error`; a proc macro never panics on external input.
- Observed behavior: rustc reports `custom attribute panicked` with
  `"1InvalidGuard" is not a valid identifier`.
- Impact: invalid configuration produces unstable panic diagnostics, violates the
  repository no-panic rule, and prevents compile-fail output from serving as a
  controlled public contract.
- Root cause: infallible identifier-formatting syntax is used before validating
  that transformed external text is a Rust identifier.
- Direction: parse generated names with fallible `syn::parse_str::<Ident>` (or
  derive type names from function identifiers), attach an error to the name
  literal, and remove user-string `format_ident!` calls.
- Regression validation: leading digit, punctuation, whitespace, keyword,
  Unicode, empty string, repeated separators, and valid kebab/snake names never panic.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md),
  [V04-08](../validations/F-MAC-01/V04-08.md),
  [V04-19](../validations/F-MAC-01/V04-19.md),
  [V04-20](../validations/F-MAC-01/V04-20.md),
  [V20-03](../validations/F-MAC-01/V20-03.md)

### F-MAC-01-P2-06: Macro package's green test command executes zero contracts

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-macros/src/lib.rs:20`,
  `echo-agent/echo-macros/src/lib.rs:180`,
  `echo-agent/echo-macros/src/derive_tool.rs:5`,
  `echo-agent/echo-macros/Cargo.toml:1`
- Reachability: `cargo test -p echo_macros` is the package's direct verification
  command and there is no trybuild/UI dependency or test directory.
- Expected invariant: public proc macros have executable compile-pass and
  compile-fail coverage for supported syntax, diagnostics, paths, and hygiene.
- Observed behavior: Cargo exits zero with 0 unit tests and 10 ignored doctests;
  every macro contract is skipped. Current regressions therefore coexist with a
  green package test.
- Impact: facade, generics, metadata, and panic regressions can ship without any
  macro-package test failing; compiler-version diagnostic drift is invisible.
- Root cause: examples were marked `ignore` and no UI harness replaced them.
- Direction: add a compile-test harness with one atomic fixture per public macro,
  supported boundary, rename mode, feature, and failure diagnostic. Convert valid
  ignored doctests to compiling examples and delete redundant ignored snippets.
- Regression validation: package test must execute nonzero pass/fail cases and
  fail when expected diagnostics or generated public paths drift.
- Validation reports: [V01-01](../validations/F-MAC-01/V01-01.md),
  [V04-17](../validations/F-MAC-01/V04-17.md),
  [V04-18](../validations/F-MAC-01/V04-18.md),
  [V20-04](../validations/F-MAC-01/V20-04.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Initial combined inventory | yes | failed procedure | [V01-01](../validations/F-MAC-01/V01-01.md) |
| V01 | Atomic definition inventory | yes | passed | [V01-02](../validations/F-MAC-01/V01-02.md) |
| V02 | Export and consumer reachability | yes | passed | [V02-01](../validations/F-MAC-01/V02-01.md) |
| V03 | Hygiene/generics/parser source boundary | yes | failed | [V03-01](../validations/F-MAC-01/V03-01.md) |
| V03 | Initial trait anchor inspection | additional | inconclusive | [V03-02](../validations/F-MAC-01/V03-02.md) |
| V04 | Facade-only attribute | yes | failed | [V04-01](../validations/F-MAC-01/V04-01.md) |
| V04 | Full-dependency attribute control | yes | passed | [V04-02](../validations/F-MAC-01/V04-02.md) |
| V04 | Renamed facade attribute | yes | passed | [V04-03](../validations/F-MAC-01/V04-03.md) |
| V04 | Renamed core derive | yes | passed | [V04-04](../validations/F-MAC-01/V04-04.md) |
| V04 | Generic derive | yes | failed | [V04-05](../validations/F-MAC-01/V04-05.md) |
| V04 | Generic tool attribute | yes | failed | [V04-06](../validations/F-MAC-01/V04-06.md) |
| V04 | Invalid field metadata | yes | failed | [V04-07](../validations/F-MAC-01/V04-07.md) |
| V04 | Invalid generated identifier | yes | failed | [V04-08](../validations/F-MAC-01/V04-08.md) |
| V04 | cfg propagation compile-fail | yes | passed | [V04-09](../validations/F-MAC-01/V04-09.md) |
| V04 | Unit schema/validation consistency | yes | passed | [V04-10](../validations/F-MAC-01/V04-10.md) |
| V04 | Seven-attribute fixture missing feature | yes | failed fixture | [V04-11](../validations/F-MAC-01/V04-11.md) |
| V04 | Corrected seven-attribute first build | yes | inconclusive | [V04-12](../validations/F-MAC-01/V04-12.md) |
| V04 | Corrected seven-attribute rerun | yes | passed | [V04-13](../validations/F-MAC-01/V04-13.md) |
| V04 | Generic callback impl | yes | failed | [V04-14](../validations/F-MAC-01/V04-14.md) |
| V04 | Real Git derive first build | yes | inconclusive | [V04-15](../validations/F-MAC-01/V04-15.md) |
| V04 | Real Git derive rerun | yes | passed | [V04-16](../validations/F-MAC-01/V04-16.md) |
| V04 | Macro package tests | yes | failed coverage | [V04-17](../validations/F-MAC-01/V04-17.md) |
| V04 | cargo-expand availability | optional | not run | [V04-18](../validations/F-MAC-01/V04-18.md) |
| V04 | Missing-name diagnostic | yes | passed compile-fail | [V04-19](../validations/F-MAC-01/V04-19.md) |
| V04 | Invalid-risk diagnostic | yes | passed compile-fail | [V04-20](../validations/F-MAC-01/V04-20.md) |
| V04 | Facade-only derive | yes | failed | [V04-21](../validations/F-MAC-01/V04-21.md) |
| V04 | Full-dependency facade derive | yes | failed | [V04-22](../validations/F-MAC-01/V04-22.md) |
| V05 | Project-owned documentation/history drift | yes | failed | [V05-01](../validations/F-MAC-01/V05-01.md) |
| V06 | Temp target cleanup | yes | passed | [V06-01](../validations/F-MAC-01/V06-01.md) |
| V07 | Final report/link/executor/source-state gate | yes | passed | [V07-05](../validations/F-MAC-01/V07-05.md); prior attempts [V07-01](../validations/F-MAC-01/V07-01.md), [V07-02](../validations/F-MAC-01/V07-02.md), [V07-03](../validations/F-MAC-01/V07-03.md), [V07-04](../validations/F-MAC-01/V07-04.md) |
| V20 | Primary facade derive reconstruction | yes | failed invariant | [V20-01](../validations/F-MAC-01/V20-01.md) |
| V20 | Primary hygiene/generics reconstruction | yes | failed invariant | [V20-02](../validations/F-MAC-01/V20-02.md) |
| V20 | Primary metadata/identifier diagnostics | yes | failed invariant | [V20-03](../validations/F-MAC-01/V20-03.md) |
| V20 | Primary executable-test inventory | yes | failed coverage | [V20-04](../validations/F-MAC-01/V20-04.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-ARCH-01-P1-01: attribute macros secretly require facade | current; accepted dependency, not duplicated | attribute core-only behavior remains facade-coupled; valid facade/rename controls pass |
| `echo-macros` README quickstart needs only `echo_macros` | regressed | sample also imports core and generated paths need four helper crates; V04-01 |
| README `#[tool]` generates `TypedTool` | stale | source generates a concrete `<Fn>Tool` implementing `Tool`; V05-01 |
| English/Chinese compression guide imports `echo_agent_macros` | stale | package is `echo_macros`, facade re-export is `echo_agent::compressor`; V05-01 |
| Macro package tests validate its public examples | regressed/unrealized | V04-17 executes 0 tests and ignores 10 doctests |

## Coverage And Uncertainty

- Primary independently reconstructed the emitted facade path, helper-crate
  leakage, generic loss, swallowed metadata error, panic-capable identifier
  construction, and absent executable test harness in V20-01 through V20-04.
  Delegated compile fixtures remain the executable confirmation; no new build
  was justified while free disk remained below the repository threshold.
- `cargo-expand` is not installed; stable expansion snapshots were not produced.
  Compile fixtures and source-level emitted paths cover the reviewed behavior.
- Callback generic coverage represents impl-block attribute reconstruction;
  handler/audit generic cases share the same helper but were not independently
  compiled. Primary should sample one before broad acceptance.
- Permission variants, receiver misuse, async/unsafe/const/extern signatures,
  duplicate generated type collisions, multiple macro applications, and hygiene
  against caller-defined names remain for the future UI matrix.
- Unit derive's permissive unknown-field behavior is internally consistent and
  not promoted to a finding until F-EXT-01 defines strict schema policy.
- The two harness-lost build attempts are immutable `inconclusive`; known-exit
  reruns passed and no session remains.
- All executable fixtures used one explicit temp target; it was cleaned after the
  matrix. No network access was performed.
- `echo-agent-cli` was clean at task start. Before the final gate, another
  parallel task modified 38 generated TypeScript files under
  `web-frontend/src/generated/`; F-MAC neither read nor changed them. The final
  gate rejects any dirty CLI path outside that exact generated directory.

## Handoff

- Primary reconstruction and acceptance are complete in V20-01 through V20-04.
- `F-API-01` owns public dependency/import documentation; consume P1-01/P2-02
  only after primary acceptance and do not create a second facade Tool contract.
- `F-EXT-01` should decide strict schema/unknown-field policy and require macro
  schema/validation round trips; it should not own macro parsing.
- `F-FEAT-01` may use V04-13 for the `human-loop` feature consumer boundary.
- This report becomes stale when macro sources, core Tool/ToolRunner traits,
  facade exports, generated trait locations, or macro test configuration changes.
