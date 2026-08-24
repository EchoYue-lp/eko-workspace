# F-MAC-01: Procedural macro contract

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (not inspected in depth; zero macro usage confirmed)
> Worktree state: both source repositories clean (scratch fixtures in /tmp, outside the repositories)

## Question

Do derive/attribute macros generate Tool and Agent code that obeys public
schemas, error handling, generics, hygiene, and feature boundaries?

## Scope

- `echo-macros/src/lib.rs` (full read, 791 lines): `#[tool]`, `#[callback]`,
  `#[guard]`, `#[handler]`, `#[compressor]`, `#[permission_policy]`,
  `#[audit_logger]`, crate-path resolution, shared helpers
  (`extract_fn_params`, `lifetimed_params`, `add_lifetime_a`,
  `extract_doc_comments`, `to_pascal_case`).
- `echo-macros/src/derive_tool.rs` (full read, 496 lines): `#[derive(Tool)]`
  incl. unit-struct path, attribute parsing, risk/permission overrides,
  `deserialize_params`.
- Root macro facade: `echo-agent/src/lib.rs:115-117` (re-exports),
  `src/tools/mod.rs:104-114` (facade exports), `src/macros.rs` (declarative
  macros, for the "Agent code" part of the question), `prelude` (:137-276).
- Target trait contracts: `Tool`/`ToolRunner` (echo-core/src/tools/mod.rs:
  733,739), `AgentCallback` (echo-core/src/agent/mod.rs:920), `Guard`
  (echo-core/src/guard/mod.rs:62), `HumanLoopHandler`
  (echo-orchestration/src/human_loop/mod.rs:522), `ContextCompressor`
  (echo-core/src/compression.rs:446), `PermissionPolicy`
  (echo-core/src/tools/permission.rs:508), `AuditLogger`
  (echo-core/src/audit.rs:137).
- Production macro users: `echo-tools/src/git.rs:24` (GitStatusTool),
  `echo-tools/src/statistics.rs:22` (ExploratoryStatisticsTool), registry
  registration `echo-tools/src/registry.rs:44-49,157-158,257-263,380-382`,
  tests `:477,491`.
- Compile-test fixtures: **none exist in-repo** (no trybuild, no ui tests);
  scratch fixtures created under `/tmp/mac_review/` for this review.
- Examples using macros: demo01/03/04/10/13/25/35/45 (+ `agent!` etc.).

## Out Of Scope

- Behavior of the hand-written builtin tools (`F-EXT-01/02/03`) except where
  they use macros.
- The `agent!`/`messages!`/`tool_params!`/`chat_request!` declarative macros'
  runtime behavior (they have unit tests in `src/macros.rs:240-364`; only
  their macro-count and feature wiring are relevant here).
- Real doc-test compilation of macro docs (`Q-FW-02`).
- Marker-feature reconciliation (`F-FEAT-01` owns the `macros` feature).
- CLI-side macro usage: none exists (grep verified).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-MAC-01 card), `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-API-01` (complete; facade export
  inventory, `ToolRunner` unreachability P3-01, `TypedTool` phantom P2-01),
  `F-EXT-01` (complete; Tool trait/registry contract, macro schema path
  `derive_tool.rs:330-415`). Both verified independently against source.
- Historical documents treated as hypotheses: `echo-agent/README.md`,
  `echo-macros/README.md`, `docs/MASTER-PLAN.md`, `echo-agent/AUDIT_REPORT.md`,
  `docs/deep-iteration-plan.md`, `docs/PROJECT-ANALYSIS.md`.

## Layering Decision

- Generic mechanism (framework): `echo-macros` is a framework build-time
  capability; proc-macro crate-path resolution via `proc-macro-crate` is the
  hygiene mechanism; the facade re-export of the 8 macros is framework
  build architecture.
- EKO product policy: none (framework-only task).
- Adapter boundary: `echo_macros` ↔ `echo_agent` facade — the generated code
  must resolve every `::echo_agent::...` path against the facade's public
  surface; the missing `ToolRunner` export breaks this boundary (P1-01).
- Duplicate-search terms (both repositories): `proc_macro_derive`,
  `proc_macro_attribute`, `macro_export`, `derive(Tool)`, `#[tool(`,
  `#[guard(`, `#[callback(`, `#[handler(`, `#[compressor(`,
  `#[permission_policy(`, `#[audit_logger(`, `ToolRunner`, `TypedTool`,
  `echo_macros::`, `derive_tool`. Results: single macro definitions; no
  second macro system; no code-level `TypedTool`; hand-written `impl Tool`
  blocks coexist with macro-generated ones as the intended trait-impl
  pattern (V01-01).

## Current Path

Verified flows:

1. **Attribute `#[tool]`** (lib.rs:192-297): fn → `ToolAttrs` parse (name/
   description required, permissions optional) → `extract_fn_params` builds a
   `Deserialize + JsonSchema` params struct (doc comments → schemars
   description) → `impl Tool` with `parameters()` = `schema_for!` → JSON,
   `validate_parameters`/`execute` via `deserialize_params` →
   `ToolError::InvalidParameter { name, message }` (field name extracted by
   string heuristic). Works through the facade (V04-01); exercised by
   default example builds (V04-04).
2. **`#[derive(Tool)]`** (derive_tool.rs:226-415): struct → params struct
   `{Struct}Params` (skip fields excluded) → `impl Tool`; `execute`
   delegates to `<Self as <crate>::tools::ToolRunner<{Struct}Params>>::run`.
   Crate resolution tries `echo_core` first, then `echo_agent`
   (:37-62). Production users (echo-tools) have `echo_core` as a dep →
   `::echo_core::tools::ToolRunner` resolves. **Facade-only consumers get
   `::echo_agent::tools::ToolRunner`, which the facade does not export →
   E0405 (P1-01).**
3. **Impl-block macros** (`#[callback]`, `#[handler]`, `#[audit_logger]`,
   lib.rs:321-340, 448-467, 590-609): emit only user-defined methods wrapped
   as `BoxFuture<'a, ...>`; trait defaults fill the rest. Compile through the
   facade (V04-01); zero production usage.
4. **Fn macros** (`#[guard]`, `#[compressor]`, `#[permission_policy]`,
   lib.rs:387-419, 487-515, 532-561): fixed parameter names and types
   (`content`/`direction`, `input`, `tool_name`/`permissions`); body moved
   into `Box::pin(async move #body)`. Zero production usage; `#[guard]` used
   in demo25.
5. **Declarative macros** (src/macros.rs): `agent!` builds
   `ReactAgentBuilder` chains, `messages!`/`tool_params!`/`chat_request!`
   build data — unit-tested (:240-364).
6. **Feature boundary**: `macros = []` (Cargo.toml:100) gates nothing; the
   8 proc macros are unconditionally re-exported (src/lib.rs:115-117);
   the feature only drives example `required-features` (Cargo.toml:200,301)
   (V04-04).

## Findings

### F-MAC-01-P1-01: `#[derive(Tool)]` generates code that cannot compile through the documented `echo_agent` facade — `ToolRunner` is not exported

- Priority: P1
- Confidence: high (reproduced compile failure)
- Layer: framework (echo_macros ↔ echo_agent facade adapter boundary)
- Evidence: `echo-macros/src/derive_tool.rs:387` (`<Self as #echo_crate::tools::ToolRunner<#params_ident>>::run(self, params).await`); `echo-agent/src/tools/mod.rs:109-114` (explicit facade re-export list — `Tool, ToolFailure, ... ToolStreamEvent` — no `ToolRunner`; also absent from `echo_execution::tools::*`); crate resolution `derive_tool.rs:37-62` (echo_core first, then echo_agent); the macro's own doc example `derive_tool.rs:5-24` and `lib.rs:73-91` use the facade import path.
- Reachability: any consumer that imports the derive via the documented `use echo_agent::Tool` and depends only on `echo_agent` (no direct `echo_core` dep) hits `error[E0405]: cannot find trait ToolRunner in module ::echo_agent::tools` inside the derive expansion (reproduced, V04-02 `facade_derive`, exit 1). In-repo usage (echo-tools) always has `echo_core` as a direct dep, so nothing in the workspace exercises the facade-only path; no example uses `#[derive(Tool)]` (V01-01, V04-04).
- Expected invariant: a macro documented as "use via echo_agent" must generate code resolvable against `echo_agent`'s public surface.
- Observed behavior: the derive is unusable through the facade; the only working configuration is the undocumented one (add `echo_core` as a dependency). Users following the docs get a confusing error pointing at a non-existent path.
- Impact: half of the Tool-generation macro surface is broken for the documented consumer configuration — a public API capability failure for external consumers; also the derive doc examples (which reference `ToolRunner`) cannot compile as written.
- Root cause: the facade's `src/tools/mod.rs` export list was curated (F-API-01-P3-01 already recorded `ToolRunner` as unreachable through `echo_agent`) without re-checking the derive's generated paths; `resolve_echo_crate_path` falls back to `echo_agent` for facade-only consumers, and the derive is the only macro whose generated code references `ToolRunner` (the attribute macro implements `execute` directly and does not need it).
- Direction: export `ToolRunner` from `echo-agent/src/tools/mod.rs` (with `ToolRegistrar`, also required by the `ToolRunner: Tool + Sized` bound context — at minimum `ToolRunner`) so `::echo_agent::tools::ToolRunner` resolves; add a compile fixture that derives `Tool` through `echo_agent` only (no echo_core dep) — e.g. add a derive usage to demo25_macros or a dedicated test; alternatively, make `resolve_echo_crate_path` prefer `echo_agent` and hard-fail with a targeted diagnostic when the facade lacks the needed type. Align with F-API-01-P3-01 (facade export decision).
- Regression validation: a facade-only crate (deps: echo_agent + serde/schemars/futures) containing `#[derive(echo_agent::Tool)]` + `impl ToolRunner<...>` must compile (exit 0); `cargo test -p echo_tools --features git,statistics` stays green; doc example `derive_tool.rs:5-24` compiles.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md), [V04-02](../validations/F-MAC-01/V04-02.md)

### F-MAC-01-P2-01: `#[tool]` silently drops an `&self`/`&mut self` receiver — the fn compiles with changed semantics

- Priority: P2
- Confidence: high (reproduced silent compile)
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:621-645` (`extract_fn_params` matches only `FnArg::Typed`; `FnArg::Receiver` falls through silently); `lib.rs:203-297` (`tool_impl` never checks `func.sig.inputs` for receivers).
- Reachability: any `#[tool]` attribute on a fn with a receiver (the fn then still compiles as long as the body does not require the receiver's type — reproduced `cf2_tool_self`, exit 0, V04-02). `self` inside the body binds to the generated unit struct (`{Pascal}Tool`), not the author's intended type.
- Expected invariant: either the macro supports stateful tool fns (receiver → tool state) or it rejects a receiver with a targeted diagnostic.
- Observed behavior: the receiver is removed from the params struct without error; a body that ignores `self` compiles and runs — the author's state access silently disappears; a body that calls a method on `self` fails with a confusing error about the generated unit struct.
- Impact: silent semantic divergence — a tool author can ship a tool whose schema and `execute` differ from the fn they wrote; the failure mode is invisible in the common "self unused" case.
- Root cause: `extract_fn_params` treats non-typed args as no-ops; no receiver guard exists.
- Direction: return `syn::Error` for any `FnArg::Receiver` ("#[tool] functions cannot take a receiver; use a free function or #[derive(Tool)] with ToolRunner") or implement receiver support; add a compile-fail fixture.
- Regression validation: compile-fail fixture `#[tool] async fn t(&self, ...)` must fail with the targeted message; all existing examples stay green.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md), [V04-02](../validations/F-MAC-01/V04-02.md)

### F-MAC-01-P3-01: No compile-fail test suite exists, and the fixed-name contracts of `#[guard]`/`#[compressor]`/`#[permission_policy]` are unvalidated — invalid input yields confusing errors

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: zero trybuild/`*.stderr`/ui fixtures in either repository (V01-01 search); `echo-macros` has no test module (MASTER-PLAN:616 "echo_macros 0" — current); fixed param names in `guard_impl` (lib.rs:410-416), `compressor_impl` (:506-513), `permission_policy_impl` (:551-559) are not checked against the user's fn signature.
- Reachability: reproduced `cf4_guard_names` — `#[guard]` with params named `text`/`dir` fails with `cannot find value text in this scope` (V04-02, exit 1), leaving the user to discover the fixed-name contract by reading generated code.
- Expected invariant: the macro validates the user's signature shape (param names/types) or emits a diagnostic naming the expected contract; error paths have automated compile-fail coverage.
- Observed behavior: diagnostics are generic name-resolution errors; the documented error paths (missing name/description, unknown attribute, tuple/enum structs, invalid risk_level) do produce clean messages (verified cf5/cf6/cf7/cf9) but only by manual testing.
- Impact: developer friction for the impl-helper macros; regression risk — nothing in CI exercises any macro error path, so a future macro change that breaks diagnostics (or the P1-01 class of break) ships unnoticed.
- Root cause: no compile-fail harness (trybuild) was ever added; the fixed-name contracts are implicit.
- Direction: add a `trybuild`-based compile-fail suite in echo-macros covering: missing name/description, unknown attribute, receiver fn, generic fn, tuple/enum derive, invalid risk_level, wrong guard/compressor/policy param names; optionally add signature validation with targeted errors instead of relying on name resolution.
- Regression validation: `cargo test -p echo_macros` (trybuild) passes with expected `.stderr` files; existing fixtures green.
- Validation reports: [V04-02](../validations/F-MAC-01/V04-02.md), [V05-01](../validations/F-MAC-01/V05-01.md)

### F-MAC-01-P3-02: `ToolError::InvalidParameter.name` extraction is a fragile string heuristic — wrong-type errors report `(deserialization)`

- Priority: P3
- Confidence: high (reproduced at runtime)
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:280-286` and `derive_tool.rs:398-404` (`strip_prefix("missing field `")` / `split("at `")` heuristic, fallback `"(deserialization)"`).
- Reachability: reproduced (V04-01): missing field `b` → name `b` (works); `invalid type: string "not-a-number", expected f64` → name `(deserialization)`. Live on every macro-generated tool's invalid-input path.
- Expected invariant: `InvalidParameter.name` names the offending parameter (or the error carries structured field info), enabling field-level attribution by consumers (UI highlight, targeted retry).
- Observed behavior: type-mismatch errors degrade the name to a placeholder; only exact "missing field `x`" strings extract correctly; serde path errors (nested fields) are not extracted at all.
- Impact: consumers that match on `name` cannot attribute wrong-type errors to a field; message text still carries the full serde detail, so impact is diagnostic quality.
- Root cause: string-parsing instead of structured deserialization error handling.
- Direction: use `serde_path_to_error` for field paths, or validate/deserialize per-field to construct `InvalidParameter` with the actual field name; add a wrong-type and nested-field fixture asserting extracted names.
- Regression validation: runtime fixture (V04-01 style) asserting `name == "a"` for a wrong-typed `a`; missing-field case stays green.
- Validation reports: [V04-01](../validations/F-MAC-01/V04-01.md)

### F-MAC-01-P3-03: echo-macros README quickstart and macro docs do not compile as written

- Priority: P3
- Confidence: high (reproduced)
- Layer: framework (docs)
- Evidence: `echo-macros/README.md:9-24` (quickstart deps `echo_macros` + `echo_core` only, `-> ToolResult` return, `Ok(serde_json::json!(...))` body); `README.md` (echo-macros) table `:28` ("`#[tool]` | Generate `TypedTool` from an async fn").
- Reachability: reproduced (V04-02 quickstart, exit 1): `error: Could not find echo_agent in dependencies` — the `#[tool]` attribute macro's `echo_agent_crate_path()` (lib.rs:43-51) requires `echo_agent` in the consumer's manifest (unlike the derive, which tries `echo_core` first). Even with `echo_agent` added, `-> ToolResult` contradicts the macro's `Result<ToolResult>` contract (lib.rs:261) — a second break.
- Expected invariant: README quickstart compiles verbatim; macro docs reference real symbols.
- Observed behavior: following the README fails at build time; `TypedTool` is the same phantom name already flagged by F-API-01-P2-01 (README.md:446) and P3-01 (echo-macros/src/lib.rs:29-31 prelude guidance).
- Impact: broken onboarding for the macro crate; doc-only, no runtime impact.
- Root cause: README predates the facade/split-crate era; `TypedTool` naming survived an API rename.
- Direction: rewrite `echo-macros/README.md` quickstart (add `echo_agent` dep, `-> Result<ToolResult>`), replace `TypedTool` with `Tool`, and fix the `echo_agent::prelude::*` guidance in `echo-macros/src/lib.rs:29-31` (prelude exports no macros) — align with F-API-01-P2-01/P3-01 and Q-DOC-01.
- Regression validation: compile the corrected quickstart verbatim (exit 0); grep for `TypedTool` in both READMEs returns zero.
- Validation reports: [V04-02](../validations/F-MAC-01/V04-02.md), [V05-01](../validations/F-MAC-01/V05-01.md)

### F-MAC-01-P3-04: Generic/lifetime/borrowed params on `#[tool]` fns fail with low-quality diagnostics (no validation)

- Priority: P3
- Confidence: high (reproduced)
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:203-297` (`tool_impl` ignores `func.sig.generics`/where-clauses; params struct fields reuse param types verbatim, `lib.rs:639-643`).
- Reachability: reproduced (V04-02 `cf3_tool_generic`, exit 1): `async fn t<T: ToString>(a: T)` → `error[E0425]: cannot find type T in this scope` at the generated params struct — the generic parameter is copied into the struct field without declaration. Reference params (`a: &str`) fail with missing-lifetime errors (static inspection, same mechanism). `#[derive(Tool)]` on a generic struct has the same class of issue (derive_tool.rs:226 ignores `input.generics`).
- Expected invariant: the macro either supports generics (declaring them on generated types) or rejects them with a targeted diagnostic naming the limitation.
- Observed behavior: silent generation of uncompilable code; the user sees generic "cannot find type" errors with no hint that the macro dropped the generics.
- Impact: confusing diagnostics for a plausible input; no wrong runtime behavior (fails loudly at compile time).
- Root cause: `tool_impl`/`derive_tool_impl` never inspect generics or borrow types.
- Direction: reject generic fns/structs and reference-typed params with `syn::Error` diagnostics (or properly forward generics); add compile-fail fixtures (fold into the P3-01 trybuild suite).
- Regression validation: compile-fail fixtures for generic fn, generic struct, and `&str` param all fail with the targeted message.
- Validation reports: [V03-01](../validations/F-MAC-01/V03-01.md), [V04-02](../validations/F-MAC-01/V04-02.md)

### F-MAC-01-P3-05: Macro showcase `demo25_macros` is excluded from default example builds by a no-op feature, and README macro counts/commands are stale

- Priority: P3
- Confidence: high
- Layer: framework (build/docs)
- Evidence: `echo-agent/Cargo.toml:100` (`macros = []`), `:200,301` (`required-features = ["macros"]`); `echo-agent/README.md:96` (`cargo run --example demo25_macros` — fails as written, V04-04 exit 1), `:200` ("11 macros" — actual 12: 8 proc + 4 user-facing declarative).
- Reachability: default `cargo build --examples`/CI skips demo25 (V04-04); the macro showcase is never compiled by default even though the `macros` feature gates nothing — which is exactly how the P1-01 facade break shipped unnoticed (no example uses `#[derive(Tool)]`).
- Expected invariant: the macro showcase is compiled in default CI; README commands and counts match the manifest.
- Observed behavior: feature gate is a marker with no code effect; README command fails without `--features macros`; count is off by one.
- Impact: reduced regression protection for the macro surface; doc drift; marker-feature confusion (F-FEAT-01 owns the marker audit).
- Root cause: `macros` was declared as a no-op feature and demo25 was gated on it before the feature became a marker; README examples were not re-verified.
- Direction: either remove `required-features` from demo25 (and let it build by default), or add the missing gates the feature implies and fix the README command/count; add a derive usage to demo25 to cover the facade path (ties to P1-01 regression validation).
- Regression validation: `cargo check --example demo25_macros` under default features exits 0; README count corrected to 12 (or the inventory trimmed).
- Validation reports: [V04-04](../validations/F-MAC-01/V04-04.md), [V05-01](../validations/F-MAC-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (macro inventory, no second authority) | yes | passed | [V01-01](../validations/F-MAC-01/V01-01.md) |
| V02 | Registration and runtime reachability (registry trace, zero-usage macros, feature gates) | yes | passed | [V02-01](../validations/F-MAC-01/V02-01.md) |
| V03 | Invariant/edge-case inspection (trait contracts, hygiene, schema, error mapping) | yes | passed | [V03-01](../validations/F-MAC-01/V03-01.md) |
| V04 | Compile-pass + runtime fixture (facade macros, error handling, schema) | yes | passed | [V04-01](../validations/F-MAC-01/V04-01.md) |
| V04 | Compile-fail diagnostics suite + crate-rename fixtures | yes | passed | [V04-02](../validations/F-MAC-01/V04-02.md) |
| V04 | Production derive path: `cargo test -p echo_tools --features "git,statistics" --locked --lib registry` | yes | passed (exit 0) | [V04-03](../validations/F-MAC-01/V04-03.md) |
| V04 | Example/feature boundary: demo25 with/without `macros`, demo01 default | yes | passed (exit 0/1/0) | [V04-04](../validations/F-MAC-01/V04-04.md) |
| V05 | Historical-document drift (README/echo-macros README/MASTER-PLAN/AUDIT) | conditional | passed | [V05-01](../validations/F-MAC-01/V05-01.md) |

All required validations have reports; every command has a recorded exit code.
Compile-fail coverage exists only via the scratch fixtures in /tmp (the
in-repo absence of such a suite is itself a finding, P3-01).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:616 "echo_macros 0 [tests]" | current | echo-macros has no test module; confirmed by repo scan ([V01-01](../validations/F-MAC-01/V01-01.md)) |
| README:200 "11 macros" | stale | 8 proc + 4 declarative = 12 ([V05-01](../validations/F-MAC-01/V05-01.md)) |
| README:1044 "`#[tool]` ... `TypedTool` from async fn" | stale | `TypedTool` has 0 code matches; macro generates `impl Tool` ([V05-01](../validations/F-MAC-01/V05-01.md); same phantom as F-API-01-P2-01) |
| README:266 "`macros` feature gates proc macros" | stale | `macros = []` gates nothing; re-exports unconditional (Cargo.toml:100, src/lib.rs:115-117; [V04-04](../validations/F-MAC-01/V04-04.md)) |
| README:96 `cargo run --example demo25_macros` works | stale | fails without `--features macros` ([V04-04](../validations/F-MAC-01/V04-04.md)) |
| echo-macros README quickstart compiles | stale | "Could not find echo_agent in dependencies"; `-> ToolResult` mismatch ([V04-02](../validations/F-MAC-01/V04-02.md), P3-03) |
| echo-macros lib.rs:29-31 "import via echo_agent::prelude::*" | stale | prelude exports no proc macros (F-API-01-P3-01) |
| AUDIT_REPORT / deep-iteration-plan / PROJECT-ANALYSIS macro claims | current (none found) | grep zero macro claims ([V05-01](../validations/F-MAC-01/V05-01.md)) |
| git.rs:29-30 "TODO(v0.3): integrate into default tool registry" | stale | tool already registered at registry.rs:49,263 ([V02-01](../validations/F-MAC-01/V02-01.md)) |

## Coverage And Uncertainty

- The `#[handler]`, `#[compressor]`, `#[permission_policy]`, `#[audit_logger]`
  macros were verified against trait signatures statically (V03) and by
  contract comparison only — they have zero in-repo usage, so no compile
  fixture exercised them; the fixed-name contracts of guard/compressor/policy
  are shared with `#[guard]` (cf4 fixture covers the guard instance of the
  class).
- The unit-struct derive path was compile-checked (`cf10_unit_derive`, exit 0)
  but not run; its `parameters()` returns a hand-rolled empty-object schema
  that bypasses schemars — not flagged (matches the documented contract).
- Runtime checks were executed against the exact baseline commits in scratch
  crates; the polars-linked echo-tools test run compiles from the baseline
  source tree.
- `echo-agent-cli` was not read beyond the macro-usage grep (zero usage); no
  CLI-side macro contract exists.
- Doc compile coverage (cargo doc) was not run; macro doc snippets are
  `rust,ignore` (Q-FW-02 owns real doctests).
- The `agent!` declarative macro's generated builder chain was not traced
  against `ReactAgentBuilder` option-by-option (F-RCT-01 owns the builder).

## Handoff

- Conclusions downstream tasks may rely on:
  - Single macro authority confirmed; the derive is broken through the facade
    (P1-01) — `X-BND-01`/`F-API-01` facade-export decision must include
    `ToolRunner` (or a deliberate removal of the derive's facade story).
  - The `macros` marker feature and demo25 `required-features` (P3-05) feed
    `F-FEAT-01`'s marker audit.
  - Macro error paths are untested in CI (P3-01) — `Q-TST-01`/`Q-FW-02` should
    plan a trybuild suite; `Q-DOC-01` owns the README rewrites (P3-03, P3-05,
    and F-API-01's P2-01/P3-01).
  - `F-EXT-01`'s P1-01/P2-01 (plan-mode filter, WRITE_TOOLS) do not interact
    with macro-generated tools today (macro tools are not write-classified
    differently), but a future `derive`-based write tool would inherit the
    P2-01 drift; the shared root cause (per-tool classification vs name lists)
    is unchanged.
- Reports to read: this report plus the 8 validation reports; `F-API-01`
  (facade export map, P3-01 ToolRunner), `F-EXT-01` (tool contract).
- Stale triggers: any change to `echo-macros/src/*`, `echo-agent/src/tools/
  mod.rs` exports, `src/lib.rs` re-exports, `echo-tools` registry/git/
  statistics modules, Cargo.toml features or example declarations, or the two
  READMEs invalidates the corresponding claims.
- Follow-up task IDs (fixes not implemented in this review): `F-FEAT-01`
  (marker features), `F-API-01`/`X-BND-01` (facade export decision for
  `ToolRunner`), `Q-DOC-01` (README rewrites), `Q-TST-01`/`Q-FW-02` (compile-
  fail suite and real doc compilation).
