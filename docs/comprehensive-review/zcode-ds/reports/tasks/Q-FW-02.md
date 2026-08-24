# Q-FW-02: Framework feature, examples, and docs matrix

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Do public optional capabilities compile and demonstrate their stated contracts independently?

## Scope

- All 12 standalone feature compiles requested by the task card: `cargo check -p echo_agent --no-default-features --features <feature> --locked` for sqlite, subagent, human-loop, mcp, lsp, a2a, git, database, rag, chart, web, media (V01-V12).
- Grouped example compilation: 8 examples sampled across all declared required-feature groups, compiled with exactly their manifest-declared features under no-default (V13).
- Doctest execution: `cargo test --doc -p echo_agent --all-features --locked` (V14).
- Document link validation: `cargo doc -p echo_agent --all-features --no-deps --locked` with unresolved-link warning census (V15).
- Source inspection of every failure: feature gates, manifest `[[example]]` declarations, doctest text, struct definitions.

## Out Of Scope

- Full submission gate (fmt/clippy/workspace tests) — Q-FW-01; workspace-level no-default check — Q-FW-01.
- Runtime behavior of features, tools, examples — owned by F-* atomic tasks (F-MEM-02, F-HITL-01, F-INT-01/02, F-EXT-02/03, F-TST-01).
- README/manifest text drift beyond what compilation proves — Q-DOC-01; docs.rs metadata build (`no-default-features` variant) — F-FEAT-01-P3-04/Q-DOC-01.
- `docs/comprehensive-review/codex/` and `zcode-glm/` — not read.

## Inputs

- Root `AGENTS.md` (feature matrix / conditional gate), `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md` (Q-FW-02 card), `zcode-ds/README.md`, both report templates.
- Dependency reports: zcode-ds `F-FEAT-01` (task + V01-V05) and `F-API-01` (task + V01-V05), per the task card. Their static predictions were re-verified by real compilation here, not copied.

## Layering Decision

- Generic mechanism: feature topology, example `required-features`, doctests, and intra-doc links are framework build/documentation surface — all findings below are framework-layer.
- EKO product policy: not implicated; `echo-agent-cli` is unaffected by any of the three findings (it does not build framework examples or all-feature doctests as such).
- Adapter boundary: none implicated.
- Duplicate-search terms: `RuleGuardBuilder`, `CompressionInput` initializers, `focus_instructions`, each of the 22 unresolved link targets (`ReactAgent`, `ExternalRunContext`, `AgentCheckpoint`, `SkillDescriptor`, `PipelineStage`, etc. — all confirmed to exist exactly once as authoritative definitions; zero parallel implementations).

## Current Path

1. Feature isolation (V01-V12): all 12 standalone feature checks exit 0 with zero warnings on the reviewed commit. The minimal cold build (`sqlite` only, V01) compiled 8 tree-sitter crates (`tree-sitter` + 7 grammars) — real compile evidence that `files`/`shell` are always-on through the `echo_execution` default-feature union (canonical F-FEAT-01-P2-01 / F-API-01-P2-02; re-verified, not re-filed). `subagent` compiles standalone (V02) as F-FEAT-01-P3-02 predicted; `data`+`statistics` polars compile on stable 1.97.1 (V13), consistent with the nightly-only caveat in `Cargo.toml:57-63`.
2. Examples (V13): 7 of 8 sampled examples compile under exactly their declared required-features. `demo45_customer_service` fails E0433 (`RuleGuardBuilder` not found) under its declared `sqlite,human-loop` — it uses the content-guard-gated prelude symbol without declaring `content-guard`; adding `content-guard` makes it compile (exit 0).
3. Doctests (V14): 81 pass, 1 compile-fails — `src/testing/mod.rs` doctest initializes `CompressionInput` without the `focus_instructions` field that `echo-core/src/compression.rs:40-48` now declares; all 9 other initializer sites include it.
4. Doc links (V15): `cargo doc` exits 0 with 28 unresolved intra-doc link warnings across 26 source locations; every target exists in code (one is `pub(crate) PipelineStage`), so all are path/scoping defects in doc comments.

## Findings

### Q-FW-02-P2-01: demo45_customer_service cannot compile under its declared required-features — uses content-guard API without declaring it

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:168-169` (`demo45_customer_service`, `required-features = ["sqlite", "human-loop"]`); `examples/demo45_customer_service.rs:36` (`use echo_agent::prelude::*;`), `:309` (`RuleGuardBuilder::new("content-filter")`); prelude gate `src/lib.rs:230-231` (`#[cfg(feature = "content-guard")] pub use crate::guard::rule::{RuleGuard, RuleGuardBuilder};`); `RuleGuardBuilder` defined at `echo-core/src/guard/rule.rs:82`; `content-guard` absent from `default` (`Cargo.toml:66`) and not implied by sqlite/human-loop.
- Reachability: `cargo check -p echo_agent --example demo45_customer_service --no-default-features --features sqlite,human-loop --locked` → exit 101 with `error[E0433]: cannot find type RuleGuardBuilder in this scope (examples/demo45_customer_service.rs:309:9)`. Under plain default features Cargo refuses with the expected required-features message (exit 101, no source error). With `content-guard` added to the feature set → exit 0. `--all-features` builds pass (full includes content-guard), so CI gates that use all-features never see the break.
- Expected invariant: an example's declared `required-features` must cover every gated symbol its source uses, so consumers can build it with exactly the documented features.
- Observed behavior: the example cannot compile with exactly its declared features; its content-filtering capability is silently gated behind an undeclared feature. This is the reverse of the F-API-01-P3-02 case (undeclared examples with no required-features at all): demo45 is *declared*, and the declaration is incomplete.
- Impact: consumers copying `--features sqlite,human-loop --example demo45_customer_service` get a build error; the example's stated contract ("customer-service with content filtering") is not demonstrated independently; `cargo build --examples` for any feature set that includes the declared features but not `content-guard` breaks.
- Root cause: the example was written against `RuleGuardBuilder` after the content-guard split; the manifest declaration was never updated to include `content-guard`.
- Direction: add `content-guard` to `demo45_customer_service`'s `required-features` (`echo-agent/Cargo.toml:169`), or gate the usage in the example source. No other code changes needed.
- Regression validation: `cargo check -p echo_agent --example demo45_customer_service --no-default-features --features sqlite,human-loop,content-guard --locked` (exit 0, proven in V13 run B) plus a `required-features` sweep of all examples against their `use` statements (Q-DOC-01/Q-FW-02 re-run after fix).
- Validation reports: [V13](../validations/Q-FW-02/V13-01.md)

### Q-FW-02-P2-02: Stale doctest in `echo_agent::testing` — `CompressionInput` example omits `focus_instructions`; all-features doctest gate is red

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `src/testing/mod.rs:24-47` (doctest `## Testing a compressor (MockLlmClient)`, initializer at `:39`); `echo-core/src/compression.rs:40-48` (`pub struct CompressionInput` with `pub focus_instructions: Option<String>`); all 9 other initializer sites include the field (`echo-state/src/compression/mod.rs` ×7, `echo-state/src/compression/invariants.rs` ×8 — grep census).
- Reachability: `cargo test --doc -p echo_agent --all-features --locked` → exit 101; `error[E0063]: missing field focus_instructions in initializer of CompressionInput (src/testing/mod.rs:39:13)`; test result `81 passed; 1 failed; 25 ignored`. The doctest is `rust,no_run`, gated by the `testing` feature (module `src/testing`), so it fails only in feature sets that include `testing` (all-features / full); default-feature doc builds are unaffected.
- Expected invariant: every doc example must compile against the current API; the mandatory gate `cargo test --workspace --all-targets --all-features --locked` (AGENTS.md) must be green, and it includes lib doctests.
- Observed behavior: the flagship example of the public `echo_agent::testing` module cannot compile; consequently the doctest component of the framework's mandatory all-features gate is RED on the reviewed commit (Q-FW-01 should confirm the full gate; V14 already proves the doctest phase fails).
- Impact: users following the documented `MockLlmClient` compression example get a compile error; framework CI/submission gate fails on doctests; the defect hides any further doctest regressions (later failures would be masked by the same red run).
- Root cause: `CompressionInput` gained `focus_instructions` (compression focus feature work) without updating the module doc example.
- Direction: add `focus_instructions: None,` to the initializer at `src/testing/mod.rs:39` (and audit the other 25 ignored doctests for similar staleness — ignored doctests are not compiled).
- Regression validation: rerun `cargo test --doc -p echo_agent --all-features --locked` to exit 0 (after fix), then the full AGENTS.md gate.
- Validation reports: [V14](../validations/Q-FW-02/V14-01.md)

### Q-FW-02-P3-01: 28 unresolved intra-doc links across 26 source locations

- Priority: P3
- Confidence: high
- Layer: framework (documentation)
- Evidence: `cargo doc -p echo_agent --all-features --no-deps --locked` → exit 0, 29 warnings, 28 `unresolved link` warnings at `src/agent/default_factory.rs:5`, `src/agent/mod.rs:76-77`, `src/agent/react/capabilities.rs:429`, `src/agent/react/mod.rs:300,1281,2104,2173,2237,2400,2426`, `src/agent/snapshot.rs:324,409,565`, `src/agent/subagent/events.rs:35,40,43,50`, `src/agent/subagent/team/mod.rs:304`, `src/agent/subagent/usage.rs:4`, `src/evolution/curator.rs:225,248`, `src/evolution/draft.rs:3`, `src/evolution/layer.rs:498`, `src/trace/mod.rs:671`.
- Reachability: every target exists in code — `ReactAgent` (`src/agent/react/mod.rs:104`), `ExternalRunContext` (`echo-core/src/tools/mod.rs:964`), `AgentCheckpoint` (`src/state/mod.rs:117`), `delegate_task`/`delegate_to_agent`/`delegate_to_agent_with_parent_context_and_cancel` (`src/agent/react/mod.rs:2100/2169/2289`), `SkillDescriptor` (`src/lib.rs:217`, `src/evolution/health.rs:19`), `AgentEvent::LlmUsage` (`src/agent/react/run/stream_channel.rs:466`), etc. The single exception is `PipelineStage` (`src/agent/react/run/pipeline.rs:59`) which is `pub(crate)` — rustdoc cannot link private items.
- Expected invariant: rustdoc output has no dead intra-doc links; doc comments reference symbols by a path resolvable from the referencing module.
- Observed behavior: 28 bare-name links in doc comments are not in the referencing module's scope (e.g. `[`ReactAgent`]` inside `src/agent/snapshot.rs`), so published docs (docs.rs embeds the same comments) contain dead links; `cargo doc` under `-D warnings` would fail.
- Impact: broken navigation and unprofessional doc surface in the primary published documentation; doc warnings degrade `cargo doc` cleanliness. No runtime impact.
- Root cause: doc comments written with bare type names assuming a global scope, plus at least one reference to a `pub(crate)` item; rustdoc's scope-based resolution never matched.
- Direction: rewrite the 28 links with explicit paths (e.g. `crate::agent::react::ReactAgent` or the re-export the module intends); make `PipelineStage` public if its doc link is intentional. Owned by Q-DOC-01's doc rewrite.
- Regression validation: rerun `cargo doc -p echo_agent --all-features --no-deps` and require zero `unresolved link` warnings; optionally `RUSTDOCFLAGS="-D warnings"`.
- Validation reports: [V15](../validations/Q-FW-02/V15-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | `cargo check -p echo_agent --no-default-features --features sqlite --locked` | yes | passed (exit 0; cold build compiled 8 tree-sitter crates) | [V01](../validations/Q-FW-02/V01-01.md) |
| V02 | `... --features subagent --locked` | yes | passed (exit 0) | [V02](../validations/Q-FW-02/V02-01.md) |
| V03 | `... --features human-loop --locked` | yes | passed (exit 0) | [V03](../validations/Q-FW-02/V03-01.md) |
| V04 | `... --features mcp --locked` | yes | passed (exit 0) | [V04](../validations/Q-FW-02/V04-01.md) |
| V05 | `... --features lsp --locked` | yes | passed (exit 0) | [V05](../validations/Q-FW-02/V05-01.md) |
| V06 | `... --features a2a --locked` | yes | passed (exit 0) | [V06](../validations/Q-FW-02/V06-01.md) |
| V07 | `... --features git --locked` | yes | passed (exit 0) | [V07](../validations/Q-FW-02/V07-01.md) |
| V08 | `... --features database --locked` | yes | passed (exit 0) | [V08](../validations/Q-FW-02/V08-01.md) |
| V09 | `... --features rag --locked` | yes | passed (exit 0) | [V09](../validations/Q-FW-02/V09-01.md) |
| V10 | `... --features chart --locked` | yes | passed (exit 0) | [V10](../validations/Q-FW-02/V10-01.md) |
| V11 | `... --features web --locked` | yes | passed (exit 0) | [V11](../validations/Q-FW-02/V11-01.md) |
| V12 | `... --features media --locked` | yes | passed (exit 0) | [V12](../validations/Q-FW-02/V12-01.md) |
| V13 | Examples grouped by identical required-features: demo27 (sqlite), demo45 (sqlite+human-loop), demo48 (sqlite+tasks+subagent), demo06 (mcp), demo58 (git), demo41 (web), demo60 (data+statistics), demo55 (lsp); each with `--no-default-features --features <required>`; supplementary default-features and +content-guard runs for demo45 | yes | failed (7/8 pass; demo45 exit 101) | [V13](../validations/Q-FW-02/V13-01.md) |
| V14 | `cargo test --doc -p echo_agent --all-features --locked` | yes | failed (exit 101; 81 passed / 1 failed / 25 ignored) | [V14](../validations/Q-FW-02/V14-01.md) |
| V15 | `cargo doc -p echo_agent --all-features --no-deps --locked` + unresolved-link census | yes | passed (exit 0; 28 unresolved-link warnings → P3-01) | [V15](../validations/Q-FW-02/V15-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-FEAT-01 V04-01 static rows ("likely passes" for sqlite/subagent/human-loop/mcp/lsp/a2a/git/database/rag/chart/web/media; no-default; standalone builds) | current (confirmed) | all 12 rows now have real exit codes 0; the needs_evidence handoff is resolved (V01-V12) |
| F-FEAT-01-P2-01 / F-API-01-P2-02: `files`/`shell` always-on via echo_execution defaults + ungated facade re-exports | current (now compile-confirmed) | cold `--no-default-features --features sqlite` build compiled 8 tree-sitter crates (`/tmp/qfw02-v01-sqlite.log`); a consumer cannot disable them — [V01](../validations/Q-FW-02/V01-01.md) |
| F-FEAT-01-P3-02: `subagent = ["tasks"]` edge unnecessary, stale comment | current | `--features subagent` alone compiles (exit 0) — [V02](../validations/Q-FW-02/V02-01.md) |
| F-FEAT-01-P3-03: root `web`/`media`/`data`/`database`/`statistics` `dep:` entries are dead weight but compile-clean | current | V08/V11/V12 all exit 0 — [V08](../validations/Q-FW-02/V08-01.md), [V11](../validations/Q-FW-02/V11-01.md), [V12](../validations/Q-FW-02/V12-01.md) |
| F-API-01 V03: static-clean example imports; default-feature example builds pass | current, narrow | verified only for the 13 *undeclared* examples; declared example demo45 breaks under its own contract — new finding Q-FW-02-P2-01 |
| README.md:104 "zero default features ... minimal compile time and dependency footprint" | regressed in effect | sqlite-only minimal build still compiles 8 tree-sitter crates (see above) |
| F-API-01 V03: static-clean doctest imports (compile verification deferred to Q-FW-02) | **stale (one failure)** | the `testing` module doctest fails to compile — Q-FW-02-P2-02 |
| F-FEAT-01-P3-04: docs.rs metadata omits 10 features | not verified here | `cargo doc` ran all-features only; the no-default-features metadata build remains for Q-DOC-01 |

## Coverage And Uncertainty

- Examples: 8 of 41 manifest-declared examples compiled (one per distinct required-feature group sampled); the 13 undeclared examples (F-API-01-P3-02) were not compiled individually — they are exercised only by the all-targets gate (Q-FW-01). A full sweep of all 68 example files against their required-features is a recommended follow-up (Q-DOC-01/Q-FW-02 rerun).
- Doctests: root crate only, as the task card specifies (`-p echo_agent`); 25 doctests are `#[ignore]`d and were not executed — ignored doctests may hide additional staleness; sub-crate doctests (echo-core, echo-tools, echo-state, echo-execution, echo-integration, echo-orchestration) are not covered by this task's command.
- `cargo doc` was run all-features only; the docs.rs `no-default-features + metadata` build (F-FEAT-01-P3-04) and README external links (Q-DOC-01) remain unverified.
- `cargo tree -e features -i tree-sitter` was not run literally; the cold-build compile log provides equivalent direct evidence (8 tree-sitter crates compiled under a sqlite-only feature set).
- No workspace-level commands were run (`cargo check --workspace --lib --no-default-features`, clippy, full tests) — Q-FW-01 owns the full gate; V14 already proves its doctest phase fails on the reviewed commit.
- No runtime behavior was exercised; all findings are compile- and source-level.

## Handoff

- Downstream tasks may rely on: all 12 feature rows of F-FEAT-01's V04 now carry real exit codes (0); the always-on files/shell claim is compile-confirmed; the doctest phase of the AGENTS.md all-features gate is RED (Q-FW-02-P2-02); demo45's required-features contract is broken (Q-FW-02-P2-01); the doc surface has 28 broken intra-doc links (Q-FW-02-P3-01).
- Reports to read: this report + V01-V15 validation reports; F-FEAT-01 and F-API-01 task reports for the canonical feature-topology and facade findings referenced above.
- Stale triggers: any change to the 8 manifests' `[features]`/`[[example]]` blocks, `echo-execution` defaults, `src/lib.rs` prelude gates, `src/testing/mod.rs` doc examples, `echo-core/src/compression.rs` struct fields, or any doc comment in the 26 link-warning locations invalidates the corresponding claims.
- Follow-up tasks (fixes are not implemented in this review):
  - `Q-FW-01`: run the full submission gate; expect failure at the doctest phase (V14), confirm and record.
  - `Q-DOC-01`: fix the testing-module doctest (P2-02), the 28 intra-doc links (P3-01), the demo45 manifest declaration (P2-01), and the example `required-features` sweep.
  - `F-FEAT-01`/`F-API-01`: revalidate P2-01/P2-02 after their feature-topology fixes (gate `tools::shell`/`files`, marker cleanup).
  - `S-QA-01`: reconcile report counts — this task produced 15 validation reports + 1 task report; Q-FW-01 must not silently report a green gate without addressing V14.
