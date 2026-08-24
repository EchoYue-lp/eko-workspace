# Q-STA-01: Static safety and dependency audit

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (verified via `git status
> --porcelain`, empty output)

## Question

What panic, direct-index, UTF-8 slicing, overflow, unsafe, dead-code,
duplicate dependency, and oversized-module risks remain in `echo-agent` and
`echo-agent-cli`?

**Answer: the macro-level panic-safety invariant holds in both repositories
(executable clippy gate, zero diagnostics), and direct-index/overflow surfaces
are guarded; the UTF-8 slicing invariant has **two additional live violations**
beyond the two already filed by X-INV-01 — `percent_decode`
(`echo-tools/src/web/providers/utils.rs:32`, live in EKO via `web_search`,
reproduced panic) and `parse_guard_response` (`echo-core/src/guard/llm.rs:101`,
reproduced panic). Unsafe usage is minimal (two live edition-2024 `set_var`
sites + one dead). Dead code is small and classified. Duplicate dependencies
are numerous but benign except reqwest/crossterm splits in the CLI. 18
framework + 13 CLI files exceed 1500 lines, dominated by single-authority
engines.**

## Scope

- Full production regions (all member crates, both repos; `tests/` dirs,
  `examples`, `benches`, and the TS frontend excluded) scanned for: panic
  macros, direct indexing, range slicing, unchecked arithmetic, `unsafe`.
- `#[allow(dead_code)]` inventory + caller greps; Cargo.lock version splits
  with reverse-dependency traces for notable cases; line counts for all Rust
  sources.
- Executable checks: the AGENTS.md panic-safety clippy gate on both
  workspaces; two standalone `rustc` reproductions of the new panic sites.

## Out Of Scope

- Advisory/license scan and full supply-chain review — `Q-DEP-01`.
- Submission-gate execution beyond the panic-safety lints — `Q-FW-01`,
  `Q-CLI-01`, `Q-GUI-01`.
- Frontend (TS) static analysis — explicitly excluded by the task card.
- Codex / zcode-glm tracks (independence rule).

## Inputs

- Root `AGENTS.md` in full (panic/UTF-8 hard rules, layering gates, dead-code
  and duplicate-authority rules).
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md`
  (Q-STA-01 card), `zcode-ds/README.md`, both report templates.
- Dependency report read: `B-BASE-01` (build topology, both lockfiles,
  manifest inventory, workspace membership).
- Cross-reference reports read: `X-INV-01` task report + its V04/V05
  validation reports; F/A-phase findings referenced via canonical IDs from
  `zcode-ds/README.md` (F-HITL-01-P1-01/02, F-EXT-03-P1-03, F-EXT-02-P1-01,
  A-PROJ-01-P3-01, X-INV-01-P1-01/02/P3-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Panic/UTF-8/overflow safety, unsafe inventory, and duplicate-dependency health are framework-wide contract properties (AGENTS.md hard rules). Both new findings sit in framework code (`echo-tools` web provider, `echo-core` guard). |
| EKO product policy | The `web` feature (which makes `percent_decode` reachable in the shipped product) is EKO's tool-selection decision (`echo-agent-app-core/Cargo.toml:10-15`); the defect itself is framework-level. |
| Adapter boundary | `load_shell_env` (infra.rs, GUI startup) is an EKO startup adapter whose unsafe `set_var` is Once-guarded and whitelisted — sound. |
| Duplicate search | Searched: `unwrap|expect|panic!|unreachable!|todo!|unimplemented!|get_unchecked`, `\w+[var]` indexes, all `[..]`/`[..n]`/`[n..]`/`[a..=b]` ranges, `len() - X` / narrowing casts, `unsafe`, `allow(dead_code)`, Cargo.lock package-name groups, `wc -l` over all sources — across both repos, all members. |
| Migration deletion | Delete targets from this audit: `react/mod.rs:1765-1799` (dead read-before-edit duplicate pair), `agent/mod.rs:47` (SubAgentMap alias), `tool_exec.rs:59-88` (four dead accessors), `plugin/variables.rs:186-203` (dead unsafe `export_to_env`), plus archived items F-HITL-01-P1-01/02 (approval.rs dead impl block). |

## Current Path

Verified per-rule-family current state (each with its own validation report):

1. **Panic macros**: both workspaces pass `cargo clippy --workspace --lib
   --bins --all-features --locked -- -D clippy::unwrap_used -D
   clippy::expect_used -D clippy::panic -D clippy::unreachable` with **zero
   diagnostics** (exit 0, 3m33s / 8m59s). Scripted classification confirms all
   remaining hits are `#[cfg(test)]`/`#[test]`-only or doc comments
   (e.g. `tools.rs:1178,1187` test mock, `layer.rs:1411` test helper
   `make_manager`). Cross-reference: X-INV-01-V04 conclusions **current**.
2. **Direct indexing**: every variable-index candidate in both repos carries a
   verifiable guard (loop bound, len check, validated map lookup, or
   by-construction node sets in `workflow/dag.rs`). The only production index
   panic is the archived `F-EXT-03-P1-03` (`data_quality.rs:249-255`), still
   present, with **additional failure modes** observed: `n == 1` divides by
   zero, `n == 2` OOB on both quartiles, `n == 0` underflow.
3. **UTF-8 slicing**: the invariant is **violated in four live sites**:
   X-INV-01-P1-01 (`pdf.rs:225-227`), X-INV-01-P1-02 (`eval/runner.rs:728`),
   plus **Q-STA-01-P1-01** (`web/providers/utils.rs:32` `percent_decode`,
   reproduced) and **Q-STA-01-P2-01** (`guard/llm.rs:101`
   `parse_guard_response`, reproduced). Archived latent/zero-reachability
   items F-EXT-02-P1-01 (`edit.rs:263-265`), A-PROJ-01-P3-01 (gitignore),
   X-INV-01-P3-01 (`regression.rs:80`) still present. CLI clean.
4. **Overflow**: all `len() - X`/multiplication/cast sites are guarded or
   input-bounded; no new finding. One latent observation (TUI
   `history_down`, `tui/mod.rs:1289`) with no current trigger.
5. **Unsafe**: 3 production sites total, all edition-2024
   `std::env::set_var` — `infra.rs:1491` `load_shell_env` (live, Once-guarded,
   whitelisted — sound), `plugin/variables.rs:190` `export_to_env` (**zero
   callers**), `config.rs:803`/`providers/config.rs:1296` EnvGuard
   (test-only). No pointer/transmute unsafe anywhere.
6. **Dead code**: 11 `#[allow(dead_code)]` sites + 1 module-level; delete
   targets: `react/mod.rs:1765-1799` (dead read-before-edit duplicate pair,
   live twin at `snapshot.rs:871` + `pipeline.rs:363,375,627`),
   `agent/mod.rs:47`, `tool_exec.rs:59-88` (incl. `mcp_manager_arc` returning
   `None`), `variables.rs` `export_to_env`; retain-by-contract:
   `trace/mod.rs:361` (wire enum), `spawn_task.rs:39` (pub API),
   `response.rs:75` (TS DTO), `tui/mod.rs:336` (clipboard lease by design),
   `output/mod.rs` module attr (helpers). Archived F-HITL-01-P1-01/02
   (approval.rs dead impl block, 456 lines) confirmed current.
7. **Duplicate dependencies**: 38 multi-version packages (framework) / 76
   (CLI); all transitive except `reqwest 0.12.28 + 0.13.3` (0.13.3 has no
   default-feature reverse path) and `crossterm 0.28.1 + 0.29.0` (TUI stack
   vs `comfy-table → polars-core`) in the CLI — two HTTP clients and two
   terminal-emulation libraries in the shipped app.
8. **Oversized modules**: 18 framework + 13 CLI files > 1500 lines; extremes
   `tasks/task_runtime/executor.rs` 6272 and `src/tui/events.rs` 5746; all are
   single-authority engines (coherent no-split), but the two 5k+ files are
   material maintainability risk.

## Findings

### Q-STA-01-P1-01: `percent_decode` byte-slices a `&str` at computed offsets — panic on non-ASCII input after `%`

- Priority: P1
- Confidence: high (panic reproduced verbatim, exit 101)
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/web/providers/utils.rs:26-46`
  (`let hex = &input[i + 1..i + 3];` at `:32`); callers
  `echo-agent/echo-tools/src/web/providers/duckduckgo.rs:48,54`
  (`extract_url` → `percent_decode(encoded)`); tool registration
  `echo-agent/echo-tools/src/registry.rs:88,301` (`WebSearchTool`),
  `web_search` feature enabled by EKO (`echo-agent-cli/echo-agent-app-core/
  Cargo.toml:10-15`).
- Reachability: `web_search` tool → DuckDuckGo fallback (`search.rs:74`) →
  parses the **remote third-party** results page → `extract_url` passes the
  href's `uddg=` value → `percent_decode`. Trigger: any `%` byte immediately
  followed by a multibyte char. Reproduction:
  `percent_decode("https://example.com/%中文")` → panic "end byte index 23 is
  not a char boundary; it is inside '中' (bytes 21..24 of string)", exit 101.
- Expected invariant: AGENTS.md — byte slicing of `&str` is forbidden; a
  byte-length guard (`i + 2 < bytes.len()`) is not a char-boundary guard.
- Observed behavior: `input[i + 1..i + 3]` executes whenever `bytes[i] == b'%'`
  and two bytes remain — no `is_char_boundary` check on the `&str`.
- Impact: the registered `web_search` tool aborts the agent run (no
  `catch_unwind` barrier, per F-EXT-02 V02) when DuckDuckGo (or a
  compromised/malformed endpoint) returns a redirect URL with raw non-ASCII
  after `%`; same invariant-violation family as X-INV-01-P1-01/P1-02
  (both P1). X-INV-01-V05 missed this computed-offset slice.
- Root cause: percent-decoding implemented on `&str` byte offsets instead of
  on the byte vec (`input.as_bytes()`), inherited from the web-tools module.
- Direction: decode from `&input.as_bytes()[i+1..i+3]` (u8 window,
  `u8::from_str_radix` on bytes — no `&str` slicing), or gate on
  `input.is_char_boundary(i+1) && input.is_char_boundary(i+3)` before the
  `&str` slice. Add a `%`+CJK fixture.
- Regression validation: unit fixture `percent_decode("…/%中文")` and
  `percent_decode("x%好y")` must not panic and must pass through/percent-decode
  correctly; existing `test_percent_decode` stays green.
- Validation reports: [V03-01](../validations/Q-STA-01/V03-01.md)

### Q-STA-01-P2-01: `parse_guard_response` slices `trimmed[start..=end]` with `start > end` — panic on malformed LLM guard output

- Priority: P2
- Confidence: high (panic reproduced verbatim, exit 101)
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/llm.rs:101`
  (`&trimmed[start..=end]`, `start = trimmed.find('{')`,
  `end = trimmed.rfind('}')`, both `Option`-unwrapped); public facade
  re-export `echo-agent/src/lib.rs:229` (`LlmGuard`), feature `guard`
  (`echo-core/Cargo.toml:24`, non-default).
- Reachability: `LlmGuard` (opt-in framework guard) runs
  `chat_simple(messages)` on the guard prompt and parses the **LLM reply**
  (untrusted). Trigger: a reply containing a `}` before the first `{` —
  e.g. verbose prose then an unclosed JSON block. Reproduction:
  `"Sure, here it is: } actually the JSON is { safe: true"` → panic
  "byte range starts at 41 but ends at 19", exit 101.
- Expected invariant: AGENTS.md — no API panics on abnormal input; a
  `find`-based slice is only safe when the two offsets are ordered.
- Observed behavior: `start <= end` is never checked; `rfind('}') < find('{')`
  yields an inverted range, which panics instead of falling back.
- Impact: an LLM guard that was installed to *protect* the agent instead
  kills the run on a plausible verbose reply; same invariant family as
  X-INV-01-P1-02 (untrusted LLM output → panic).
- Root cause: brace-pair extraction assumed well-formed JSON ordering without
  validating `start <= end`.
- Direction: after computing both offsets, fall back to `trimmed` when
  `start > end` (or scan left-to-right for a balanced pair). Add a
  stray-brace fixture to the guard tests.
- Regression validation: fixtures for `"}x{"`, `"}x{}"`, and
  `"text } then { safe: true"` must return `Err`/fallback without panicking;
  existing guard tests stay green.
- Validation reports: [V03-01](../validations/Q-STA-01/V03-01.md)

### Q-STA-01-P2-02: dead duplicate read-before-edit enforcement in `react/mod.rs` — second authority that silently does nothing

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:1765-1799`
  (`MAX_READ_FILES`, `READ_FILES_TTL`, `record_file_read`, `was_file_read`,
  all `#[allow(dead_code)]`); live twin: `echo-agent/src/agent/snapshot.rs:871`
  `record_file_read` consumed at `echo-agent/src/agent/react/run/pipeline.rs:
  363,375,627`; config `force_read_before_edit`
  (`echo-agent/src/agent/config.rs:126`, `snapshot.rs:95`).
- Reachability: zero callers of the `react/mod.rs` pair (full-repo grep);
  the config knob's real enforcement runs through the `snapshot.rs`/`pipeline.rs`
  path only.
- Expected invariant: one implementation of the read-before-edit safety
  feature; no dead duplicate that a future fix could miss.
- Observed behavior: two implementations exist; the `react/mod.rs` one is
  unreachable and suppressed by `#[allow(dead_code)]`.
- Impact: maintainers grepping `record_file_read` see two authorities and may
  patch the dead one; the documented safety feature's behavior diverges from
  what a reader of `react/mod.rs` would conclude. No runtime impact today.
- Root cause: the feature was moved into the snapshot/pipeline path without
  deleting the older `react/mod.rs` copy.
- Direction: delete `react/mod.rs:1765-1799` (the four items) and the
  `recently_read_files` field usage in `react/mod.rs` if it is not the shared
  Arc (verify against `snapshot.rs:357,459` before deleting the field).
- Regression validation: build with `force_read_before_edit: true` and a
  read-then-edit flow — enforcement still triggers via the pipeline path
  (behavior unchanged after deletion).
- Validation reports: [V06-01](../validations/Q-STA-01/V06-01.md)

### Q-STA-01-P3-01: `export_to_env` — production `unsafe` block with zero callers and an unenforceable SAFETY premise

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/variables.rs:186-203`
  (`pub fn export_to_env`, `unsafe { std::env::set_var(...) }` at `:190`);
  zero callers in the full repository (grep).
- Reachability: none — the function is a pub framework API that no consumer
  (framework or EKO) invokes; plugin env-var injection never happens.
- Expected invariant: production `unsafe` must be reachable and its SAFETY
  comment enforceable; dead unsafe code is a latent trap.
- Observed behavior: the SAFETY comment ("caller must ensure single-threaded
  initialization") has no caller at all; the env vars it would export
  (`ECHO_PLUGIN_ROOT` etc.) are set nowhere.
- Impact: misleading public API; if a consumer wires it later from a
  multi-threaded context the unsafe is unguarded. No runtime impact today.
- Root cause: plugin env-export was designed but never integrated.
- Direction: wire it into the plugin activation path (with the documented
  startup guarantee) or delete it (AGENTS.md: dead code is deleted, not
  kept "for later").
- Regression validation: if wired — plugin activation test asserting the env
  vars are visible to spawned processes; if deleted — `cargo test` green.
- Validation reports: [V05-01](../validations/Q-STA-01/V05-01.md),
  [V06-01](../validations/Q-STA-01/V06-01.md)

### Q-STA-01-P3-02: dead internal items — `SubAgentMap` alias and four `tool_exec` accessors (one a `None` stub)

- Priority: P3
- Confidence: high (zero callers verified by grep)
- Layer: framework
- Evidence: `echo-agent/src/agent/mod.rs:47` (`pub(crate) type SubAgentMap`);
  `echo-agent/src/agent/react/subsystems/tool_exec.rs:59` (`tool_manager_arc`),
  `:74` (`mcp_manager_arc`, **returns `None` unconditionally**), `:80`
  (`subagent_registry`), `:85` (`progressive_skill_registry`) — all
  `#[allow(dead_code)]`, zero callers.
- Reachability: none for all five items.
- Expected invariant: internal items are either reachable or deleted;
  accessors must not return hard-coded `None`.
- Observed behavior: dead internal code; `mcp_manager_arc` documents an
  architecture decision ("McpManager is not Arc-wrapped") that no caller sees.
- Impact: dead code and a misleading `None` stub; deletion burden only.
- Root cause: accessors added during subsystem refactors and never consumed.
- Direction: delete all five items (internal, non-pub).
- Regression validation: `cargo check --all-features` green after deletion.
- Validation reports: [V06-01](../validations/Q-STA-01/V06-01.md)

### Q-STA-01-P3-03: duplicate dependency versions — reqwest 0.12/0.13 and crossterm 0.28/0.29 coexist in the shipped CLI

- Priority: P3
- Confidence: medium (lockfile + `cargo tree -i` evidence; 0.13.3 reverse
  path empty under default features)
- Layer: adapter (dependency graph boundary)
- Evidence: `echo-agent-cli/Cargo.lock` — `reqwest` 0.12.28 + 0.13.3,
  `crossterm` 0.28.1 + 0.29.0; reverse trees: 0.12.28 direct via
  `echo-agent`/`echo-agent-app-core`, 0.13.3 no default-feature reverse path;
  crossterm 0.28 via binary + `ratatui` + `reedline`, 0.29 via
  `comfy-table 7.2.2 → polars-core 0.53.0`. Framework lockfile: 38
  multi-version packages, CLI: 76 — rest are transitive ecosystem splits
  (`thiserror` 1+2, `syn` 1+2, `toml` 0.8/0.9/1.x, `base64` 0.21/0.22,
  `hashbrown` 4 versions, `windows-sys` 4 versions, `rand` 0.8/0.9/0.10).
- Reachability: both reqwest versions and both crossterm versions are linked
  into the EKO binary.
- Expected invariant: one version per dependency in the shipped binary where
  feasible; no duplicate HTTP client or terminal-emulation stack.
- Observed behavior: two HTTP clients (different TLS/streaming semantics) and
  two terminal-emulation libraries (both may touch raw-mode/terminal state)
  coexist; mostly unavoidable transitive splits otherwise.
- Impact: binary-size and audit-surface bloat; potential subtle behavior
  divergence (e.g. different retry/encoding defaults) and a stale lockfile
  entry for reqwest 0.13.3. No confirmed runtime defect.
- Root cause: transitive resolution (polars pulls crossterm 0.29, Tauri build
  pulls reqwest 0.13 for a feature path); no version-alignment effort yet.
- Direction: try `cargo update -p reqwest@0.13.3` alignment or a `[patch]`;
  verify whether anything actually requires reqwest 0.13; consider
  `cargo tree -d` maintenance in CI. Q-DEP-01 owns advisories.
- Regression validation: full CLI test suite + TUI smoke after alignment.
- Validation reports: [V07-01](../validations/Q-STA-01/V07-01.md)

### Q-STA-01-P3-04: oversized modules — 31 files over 1500 lines, two above 5000

- Priority: P3
- Confidence: high (measured)
- Layer: framework + application
- Evidence: `echo-tools/src/data.rs` 3751, `src/agent/subagent/executor.rs`
  3672, `src/agent/react/mod.rs` 3276, `echo-execution/src/skills/hooks.rs`
  2925, `echo-state/src/compression/mod.rs` 2584,
  `echo-orchestration/src/tasks/executor.rs` 2556, `run/stream_channel.rs`
  2161, `echo-tools/src/shell.rs` 1979, `workflow/graph.rs` 1934,
  `src/evolution/layer.rs` 1900, `run/pipeline.rs` 1722, `snapshot.rs` 1678,
  `files/files.rs` 1616, `echo-execution/src/tools.rs` 1602,
  `sandbox/local.rs` 1590, `providers/config.rs` 1579, `hooks/types.rs` 1529,
  `plugin/registry.rs` 1526 (framework); `tasks/task_runtime/executor.rs`
  6272, `src/tui/events.rs` 5746, `tasks/task_runtime/store.rs` 3496,
  `src/tauri/commands/panels.rs` 2238, `tasks/task_runtime/worktree.rs` 2231,
  `research.rs` 2207, `infra.rs` 2184, `tasks/task_runtime/types.rs` 2169,
  `src/tui/mod.rs` 2019, `browser/mod.rs` 1996, `plugin_runtime.rs` 1884,
  `src/cli/cmd_impls/evolution.rs` 1850, `src/tauri/commands/chat.rs` 1723
  (CLI).
- Reachability: all files are on live paths (they host the engines reviewed
  by F/A/X tasks).
- Expected invariant: modules stay reviewable; no single file carries an
  entire subsystem beyond what a coherent engine requires.
- Observed behavior: all oversized files are coherent single-authority
  engines (no accidental accretion), but `task_runtime/executor.rs` and
  `tui/events.rs` exceed the 1500-line threshold by ~4x.
- Impact: review friction and defect density (both 5k+ files already carry
  multiple P1 findings from A-TSK-*/A-SRF-01 — a size/correctness correlation,
  not proof of causation); no runtime defect.
- Root cause: engines grown feature-by-feature without decomposition checkpoints.
- Direction: after the correctness findings in those files land, decompose by
  behavior slice (per review README: "reviewed by behavior slices"), not as a
  mechanical line-split.
- Regression validation: decomposition must preserve the submission gate
  (Q-CLI-01/Q-FW-01) and the existing test suites of the affected modules.
- Validation reports: [V08-01](../validations/Q-STA-01/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | Panic-macro classification + panic-safety clippy gates (both workspaces, lib+bins, all features) | yes | passed | [V01-01](../validations/Q-STA-01/V01-01.md) |
| V02 | Direct-index scan (`\w+[var]` patterns) + guard verification + archived-site recheck | yes | passed | [V02-01](../validations/Q-STA-01/V02-01.md) |
| V03 | UTF-8 range-slice scan (both repos) + reproductions of new candidates | yes | failed (2 new live violations → P1-01, P2-01) | [V03-01](../validations/Q-STA-01/V03-01.md) |
| V04 | Overflow scan (`len()-X`, casts, multiplications) + guard verification | yes | passed | [V04-01](../validations/Q-STA-01/V04-01.md) |
| V05 | Unsafe-block inventory + SAFETY comment/caller verification | yes | passed | [V05-01](../validations/Q-STA-01/V05-01.md) |
| V06 | Dead-code scan (`allow(dead_code)` + caller greps + retain/delete classification) | yes | passed | [V06-01](../validations/Q-STA-01/V06-01.md) |
| V07 | Duplicate-dependency scan (lockfile groups + `cargo tree -i` traces) | yes | passed | [V07-01](../validations/Q-STA-01/V07-01.md) |
| V08 | Oversized-module scan (`wc -l`, >1500) | yes | passed | [V08-01](../validations/Q-STA-01/V08-01.md) |

Every required validation has a report; every command has a known exit code
(0 for gates/scans, reproduced 101 for the two panic reproductions). The
failed V03 does not block completion — it is recorded as findings P1-01/P2-01.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| X-INV-01-V04: macro-level panic safety holds (zero production unwrap/expect/panic!/unreachable!, CLI zero) | current | both clippy panic-safety gates exit 0 with zero diagnostics at these commits (V01-01) |
| X-INV-01-V05: UTF-8 invariant violated exactly at pdf.rs:225-227 and eval/runner.rs:728 | regressed (now 4 sites) | both still present; **two additional live violations**: web/providers/utils.rs:32 and guard/llm.rs:101 (V03-01) |
| X-INV-01-P3-01 (regression.rs:80 latent slice), F-EXT-02-P1-01 (edit.rs), F-EXT-03-P1-03 (IQR), A-PROJ-01-P3-01 (gitignore) | current (unfixed) | present at these commits; referenced not re-filed (V02-01, V03-01) |
| F-HITL-01-P1-01/02: approval machinery dead (`approval.rs` impl block, `process_steps` uncalled) | current (unfixed) | `#[allow(dead_code)]` impl at approval.rs:11 (456 lines) + dead `approval` field at react/mod.rs:119 re-confirmed (V06-01) |
| AGENTS.md: panic/UTF-8 hard rules | regressed | 2 archived + 2 new live UTF-8/range panics (V03-01) |

## Coverage And Uncertainty

- The direct-index and overflow audits are pattern-driven with full context
  inspection of every candidate; a pattern missed by both scans (like
  X-INV-01-V05 missing `percent_decode`) remains possible but unlikely —
  three independent scans (X-INV-01-V05 plus this V03) now cover the surface.
- The two new panic reproductions used standalone `rustc` programs mirroring
  the exact source logic; no project binary was executed and no source was
  modified (read-only review).
- `cargo tree -i` reverse traces ran under default features; `reqwest@0.13.3`
  may have a feature-gated reverse path (lockfile-resolved). Advisory scan
  remains Q-DEP-01.
- The `guard` and `eval` features are non-default; their panic reachability is
  static (compiled when the feature is enabled). `web` is enabled by EKO.
- Test-only `unwrap`/`expect` (fine by policy) were not exhaustively counted
  per file beyond the residual verification.

## Handoff

- Downstream tasks may rely on: macro-level panic safety green on both repos
  (V01); direct-index and overflow surfaces guarded except archived
  F-EXT-03-P1-03 (V02/V04); unsafe usage = 2 live + 1 dead edition-2024
  `set_var` sites (V05); dead-code inventory with delete targets (V06);
  duplicate-dependency split (V07); oversized-module list (V08); and the two
  NEW panic findings P1-01 (percent_decode, live in EKO `web_search`) and
  P2-01 (parse_guard_response, `guard` feature) with reproductions (V03).
- Reports to read: 8 validation reports above; X-INV-01 (P1-01/P1-02/P3-01),
  F-EXT-03 (P1-03), F-EXT-02 (P1-01), A-PROJ-01 (P3-01), F-HITL-01 (P1-01/02)
  for the archived items; Q-DEP-01 (advisories) when available.
- Stale conditions: this report becomes stale if either reviewed commit
  moves, or if any slice/index/unsafe/dead-code site listed changes.
- Follow-up task IDs: S-RDM-01 (roadmap items: P1-01, P2-01, P2-02, P3-01..04,
  plus archived F-EXT-03-P1-03 with its n=1/2 failure modes), Q-FLT-01 (fault
  fixtures for percent_decode and parse_guard_response), Q-DEP-01 (advisories),
  Q-FW-01/Q-CLI-01 (full submission gates; the two clippy runs here are a
  subset).
