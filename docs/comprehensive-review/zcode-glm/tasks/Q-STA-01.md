# Q-STA-01: Static safety and dependency audit

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

What panic, direct-index, UTF-8 slicing, overflow, unsafe, dead-code,
duplicate dependency, and oversized-module risks remain?

## Scope

Primary source paths and behaviors inspected (read-only static analysis):

- All `src/` trees under `echo-agent/echo-*/src/`, `echo-agent/src/`,
  `echo-agent-cli/src/`, `echo-agent-cli/echo-agent-app-core/src/`,
  `echo-agent-cli/src-tauri/src/`.
- `echo-agent/Cargo.lock` (564 crates) and `echo-agent-cli/Cargo.lock`
  (787 crates) for duplicate-version analysis.
- AGENTS.md "Rust 编码硬性约束" sections (UTF-8 safety, panic prohibition)
  as the invariant source.

Search coverage across ~690 `.rs` files (490 framework + 200 application),
excluding `target/`, `.worktrees/`, `examples/`, `benches/`, and test-only
files for the production-reachability classification.

## Out Of Scope

Deferred to named task IDs:

- Per-crate executable clippy gate verification (fmt/clippy/test/build) →
  `Q-FW-01`, `Q-CLI-01`.
- Feature-isolation compile matrix → `Q-FW-02`.
- Duplicate dependency attribution (which direct dep pulls each version),
  advisory/RUSTSEC scan, license review → `Q-DEP-01`.
- Test-suite quality and coverage of the invariants found here → `Q-TST-01`.
- Performance/resource-lifecycle analysis of the oversized modules →
  `Q-PERF-01`.

## Inputs

- Repository documents read: root `AGENTS.md` (Rust coding constraints,
  cleanup policy, framework-vs-application layering), `REPORTING.md`,
  both report templates.
- Dependency task reports read: `B-BASE-01` (topology, feature plumbing,
  CI gates). Its finding B-BASE-01-P2-04 (CI does not run conditional
  matrices) and the file/line-count handoff note are consumed here.
- Historical documents treated as hypotheses: none directly; the
  README.md file-count baseline table (deferred from B-BASE-01) is
  confirmed here — 490 framework + 200 application `.rs` files.

## Layering Decision

This task spans both repositories and is purely static. The AGENTS.md
invariants verified:

- **Generic mechanism**: panic safety (unwrap/expect/panic), UTF-8-safe
  slicing, `unsafe` hygiene, dead-code cleanup — these apply to both
  framework and application equally.
- **EKO product policy**: the one UTF-8 defect found
  (`gitignore.rs:179`) is in application-layer code (`echo-agent-app-core`)
  that filters project files by `.gitignore` patterns — a local-desktop-
  assistant concern.
- **Adapter boundary**: n/a (no cross-repo adapter code inspected).

Repository-wide duplicate search terms used: `.unwrap()`, `.expect(`,
`panic!(`, `unreachable!(`, `todo!(`, `unimplemented!(`, `unsafe`,
`#[allow(dead_code)]`, `#[allow(unused`, ` as (u8|u16|u32|u64|usize|i*)`,
byte-slice patterns `\[[^]]*\.\.[^]]*\]`.

## Current Path

The audit traced each panic-family and slicing match from its definition
site to its enclosing scope (`#[cfg(test)]` mod, `#[test]` fn, or
production module) to classify reachability. For the one defect found
(Q-STA-01-P1-01), the verified call graph is:

```
context.rs:62  ProjectContext::should_ignore
  → gitignore.rs:81  GitIgnore::is_ignored(relative_path, is_dir)
    → gitignore.rs:97   glob_matches(pattern, relative_path)
      → gitignore.rs:118  globstar_match(pattern, path)   [when pattern has "**"]
        → gitignore.rs:179  &remaining[j..]   ← PANIC on non-ASCII path
```

`is_ignored` is the public API on `GitIgnore`, called from
`ProjectContext` during file-listing/scanning operations that the agent
performs when working with project files.

## Findings

### Q-STA-01-P1-01: `globstar_match` byte-slices a `&str`, panicking on non-ASCII paths

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:178-179`
- Reachability: definition (`globstar_match` fn, line 159) → registration
  (called from `glob_matches` at line 118 when pattern contains `**`) →
  live caller (`is_ignored` at line 81, called from
  `context.rs:62` `ProjectContext::should_ignore` during project file
  scanning).
- Expected invariant (AGENTS.md "字符串处理:UTF-8 安全,禁止字节级截断"):
  `&str` slicing must use char-boundary-safe indices; byte-index slicing
  panics on multi-byte characters (Chinese, emoji).
- Observed behavior: line 178 iterates `for j in 0..=remaining.len()`
  (every byte position) and line 179 does `let candidate = &remaining[j..]`
  where `remaining: &str` is a relative file path. If the path contains
  any multi-byte UTF-8 character (e.g., `"测试/x.rs"` where `测` is 3
  bytes), `&remaining[1..]` slices inside the first character and panics
  with `byte index 1 is not a char boundary`.
- Impact: any project file-scan operation (the agent listing, searching,
  or filtering project files) will panic — crashing the process — if (1)
  a file/directory path in the project contains non-ASCII characters
  (Chinese, Japanese, emoji — common for a China-targeted product) AND
  (2) a loaded `.gitignore` pattern contains `**` (e.g.,
  `**/node_modules`, `**/.DS_Store`, `**/target`). The project's own
  `.gitignore` files currently do not use `**`, which lowers immediate
  likelihood but not the defect's severity.
- Root cause: the loop was written treating `&str` as a byte buffer
  (`[u8]`), forgetting that `str` slicing requires char boundaries. The
  adjacent `simple_glob` function correctly uses `as_bytes()` for
  byte-level comparison — `globstar_match` should have done the same or
  used `char_indices()`.
- Direction: replace `for j in 0..=remaining.len()` with
  `for (j, _) in remaining.char_indices()` (and handle the terminal
  `remaining.len()` case if the match must also be tried at end-of-string).
  Alternatively, operate on `remaining.as_bytes()` and avoid slicing.
  Add a regression test with a non-ASCII path and a `**` pattern.
- Regression validation: unit test `globstar_match("**/x", "目录/x")`
  must not panic and must return `true`; add to
  `gitignore.rs` `#[cfg(test)] mod tests`.
- Validation reports: [V02-01](../validations/Q-STA-01/V02-01.md)

### Q-STA-01-P2-01: ~50 production `#[allow(dead_code)]` annotations in framework, violating cleanup policy

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: see V04-01 table; representative sites:
  `echo-agent/src/agent/react/run/pipeline.rs:58,921,927`,
  `echo-agent/src/agent/react/run/execution.rs:17,195,226,403`,
  `echo-agent/src/agent/react/run/context.rs:156,172,199`,
  `echo-agent/src/agent/subagent/team/agent_box.rs:119,127`,
  `echo-agent/echo-tools/src/media/image_fetch.rs:16,46,70`,
  `echo-agent/echo-integration/src/channels/channels/qq/gateway.rs:26,37,39,41,44`.
- Reachability: definition (annotated struct fields/functions) → these
  compile but are never read/called in non-test builds, suppressed only by
  the `#[allow]` attribute.
- Expected invariant (AGENTS.md "代码清理:无需兼容,过时代码可直接删"):
  dead code should be deleted, not annotated. Per the framework-deletion
  rule, internal dead code (non-`pub`, or `pub` but not a reasonable API
  option) should be removed; only legitimate framework API options may be
  retained.
- Observed behavior: ~50 production `#[allow(dead_code)]` annotations
  suppress the dead-code lint. Most are struct fields populated during
  deserialization/execution but never read (LLM response fields, channel
  protocol fields, agent-runtime debug fields). One has an explicit
  "future integration" comment
  (`echo-orchestration/src/human_loop/classifier.rs:396`).
- Impact: accumulated dead code increases maintenance burden, misleads
  readers into thinking fields are used, and bloats compile time. Not a
  runtime defect.
- Root cause: fields were added speculatively or for future use and
  suppressed with `#[allow]` rather than being added when needed (YAGNI
  violation) or deleted when the need evaporated.
- Direction: incremental cleanup per AGENTS.md "随手清理是强制要求" —
  when touching a module, audit its `#[allow(dead_code)]` and either use
  the field, delete it, or (for genuine framework API options) document
  why it is retained. Do not batch-delete framework `pub` items without
  the framework-wide search required by AGENTS.md.
- Regression validation: `cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings` must remain green after each
  removal.
- Validation reports: [V04-01](../validations/Q-STA-01/V04-01.md)

### Q-STA-01-P2-02: 25 source files exceed 1000 lines; 2 exceed 5000 lines

- Priority: P2
- Confidence: high
- Layer: application (largest files), framework
- Evidence: V04-01 oversized-modules table. Top offenders:
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs` (6272),
  `echo-agent-cli/src/tui/events.rs` (5746),
  `echo-agent/echo-tools/src/data.rs` (3751),
  `echo-agent/src/agent/subagent/executor.rs` (3672),
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` (3496).
- Reachability: these are live, compiled, production modules (not dead).
- Expected invariant: AGENTS.md does not set a hard line limit, but files
  of this size impede review, increase merge-conflict surface, and make
  static analysis (like this task) harder to scope.
- Observed behavior: 25 files over 1000 lines, 10 over 2000, 5 over 3000,
  2 over 5000.
- Impact: maintainability and review difficulty. The two 5000+ line files
  (`executor.rs`, `events.rs`) are single-responsibility modules that have
  accreted logic without splitting.
- Root cause: organic growth; the task-runtime executor and TUI event
  handler each centralize a large state machine / dispatch table.
- Direction: optional future refactor — split along sub-responsibility
  boundaries (e.g., `executor.rs` into planning vs execution vs cleanup;
  `events.rs` into input-handling vs render-state vs command-dispatch).
  Not blocking; recorded for `Q-PERF-01` and future maintainability work.
- Regression validation: behavior-preserving split verified by existing
  tests.
- Validation reports: [V04-01](../validations/Q-STA-01/V04-01.md)

### Q-STA-01-P2-03: Duplicate crate versions in both lockfiles (38 framework / 76 CLI)

- Priority: P2
- Confidence: high
- Layer: framework and application
- Evidence: `echo-agent/Cargo.lock`, `echo-agent-cli/Cargo.lock`; full
  duplicate list in V04-01.
- Reachability: all duplicates are compiled (except platform-gated
  `windows-*` crates, which are not compiled on macOS but still parsed by
  cargo).
- Expected invariant: AGENTS.md does not set a dedup target, but duplicate
  majors inflate compile time, binary size, and can cause subtle type-
  mismatch errors across crate boundaries.
- Observed behavior: high-impact duplicates (compiled on macOS target):
  `hashbrown` (5 versions: 0.12/0.14/0.15/0.16/0.17), `rand`/`rand_core`
  (3 versions each), `thiserror`/`thiserror-impl` (2: v1/v2),
  `syn` (2: 1.0/2.0, CLI only), `toml`/`toml_edit`/`winnow` (3 each, CLI),
  `schemars` (3, CLI), `reqwest` (2: 0.12/0.13, CLI), `quick-xml` (4).
  ~30 of the 76 CLI duplicates are `windows-*` crates (platform-gated,
  not compiled on macOS).
- Impact: compile-time and binary-size cost; potential for cross-version
  type incompatibility (e.g., two `indexmap` types). Not a correctness
  defect today.
- Root cause: transitive dependencies pulling different majors of shared
  crates; the project's direct deps have not all converged.
- Direction: detailed attribution and remediation deferred to `Q-DEP-01`.
  Quick wins likely include ensuring all direct deps request the same
  major of `thiserror`, `syn`, `indexmap`, `itertools`.
- Regression validation: `cargo tree -d` before/after dep updates.
- Validation reports: [V04-01](../validations/Q-STA-01/V04-01.md)

### Q-STA-01-P3-01: No clippy guard for numeric `as` casts (pedantic lints not in gate)

- Priority: P3
- Confidence: medium
- Layer: framework and application
- Evidence: AGENTS.md "提交前门禁" clippy gate uses
  `-D warnings` (default lints) plus `-D clippy::unwrap_used` etc., but
  does not include `clippy::cast_possible_truncation`,
  `clippy::cast_possible_wrap`, or `clippy::cast_precision_loss`
  (pedantic lints). 318 `as` casts exist in production source (246
  framework + 72 CLI).
- Reachability: the casts are live; the lints are not enforced.
- Expected invariant: AGENTS.md "禁止任何会导致系统 panic 的 API" covers
  overflow, but numeric truncation via `as` does not panic (it silently
  wraps/truncates), so it is not strictly a panic risk. It is a silent-
  correctness risk.
- Observed behavior: sampled casts (V03-01) all operate on bounded values
  (token counts, page counts, indices) safe on 64-bit targets. No current
  defect found. But future risky casts would not be caught.
- Impact: low. No current defect. A future cast like `huge_u64 as u8`
  would silently truncate without warning.
- Root cause: pedantic lints were not added to the gate.
- Direction: optional — add
  `#![warn(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]`
  to crate roots, or to the clippy gate command, for incremental tightening.
- Regression validation: enabling the lints should produce a manageable
  number of warnings to audit.
- Validation reports: [V03-01](../validations/Q-STA-01/V03-01.md)

### Positive confirmation: panic safety is clean (no finding ID needed)

- Across both repositories, **zero** production `.unwrap()`, `.expect()`,
  `panic!`, `unreachable!`, `todo!`, or `unimplemented!` calls exist
  outside test/example/comment context. All ~1710 raw matches are inside
  `#[cfg(test)]` modules, `#[test]` functions, dedicated test files, or
  example demonstrations. This satisfies AGENTS.md "禁止任何会导致系统
  panic 的 API". See [V01-01](../validations/Q-STA-01/V01-01.md).

### Positive confirmation: `unsafe` is minimal and guarded

- Only 2 production `unsafe` blocks exist
  (`echo-core/src/plugin/variables.rs:190`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1491`), both
  `std::env::set_var` under Rust edition 2024, both with single-threaded-
  execution guards (`init-time` / `call_once`) and SAFETY comments. No
  raw pointers, FFI, unions, or inline assembly. See
  [V03-01](../validations/Q-STA-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Panic-family search + reachability classification | yes | passed | [V01-01](../validations/Q-STA-01/V01-01.md) |
| V02 | UTF-8 slicing + direct-index audit | yes | passed | [V02-01](../validations/Q-STA-01/V02-01.md) |
| V03 | `unsafe` blocks + numeric `as` cast sample | yes | passed | [V03-01](../validations/Q-STA-01/V03-01.md) |
| V04 | Dead-code annotations + oversized modules + duplicate deps | yes | passed | [V04-01](../validations/Q-STA-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | Q-STA-01 is a static-current-state audit; no historical document claims to revalidate. B-BASE-01's deferred file/line counts are confirmed here (490+200 .rs files). |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-BASE-01 handoff: "echo-agent ≈ 490 Rust files / 183k LOC" (deferred from README baseline) | current | confirmed: 490 `.rs` files (excl target/worktrees); LOC not separately counted but consistent with 25 files >1000 lines |
| B-BASE-01 handoff: "echo-agent-cli ≈ 200 Rust files" | current | confirmed: 200 `.rs` files |
| B-BASE-01-P2-04: CI does not run conditional matrices | current | not directly re-tested here, but the absence of `clippy::cast_*` pedantic lints (P3-01) is consistent with CI running only default lint subsets |
| AGENTS.md "禁止任何会导致系统 panic 的 API" | current (clean) | V01-01: 0 production panic-family calls in either repo |
| AGENTS.md "UTF-8 安全,禁止字节级截断" | regressed (one site) | V02-01: `gitignore.rs:179` violates this invariant; all other ~45 str-slice sites comply |

## Coverage And Uncertainty

- **Classifier heuristic**: the panic-safety classifier (V01) uses
  brace-based scope tracking, not a full Rust parser. Spot-checks on 4
  files confirmed accuracy. Residual misclassification risk is low and
  would be caught by the clippy gate at compile time (if the gate is
  actually run with the `-D clippy::unwrap_used` flags — CI runs
  `-D warnings` which covers default lints; whether the explicit
  unwrap/expect subset is in CI is a V04-01 open question, partially
  answered by B-BASE-01-P2-04).
- **`as` cast sample**: 318 casts exist; only ~25 were sampled (the
  `as u32` and `as usize` subsets). A risky cast in the unsampled majority
  could exist but is unlikely given the codebase patterns.
- **Dead code without annotation**: this audit counts only
  `#[allow(dead_code)]`. Actual un-annotated dead code (detectable only
  via compiler warnings) is not inventoried. A full
  `cargo +nightly rustc -- -W dead_code` pass would find more.
- **Duplicate dependencies**: lockfile-based; no attribution to which
  direct dependency pulls each duplicate version (Q-DEP-01's scope).
- **Frontend**: `.ts`/`.tsx` static safety (TypeScript strictness, any
  casts) is out of scope; deferred to frontend-specific tasks.

## Handoff

Conclusions downstream tasks may rely on:

1. **Panic safety is clean**: zero production unwrap/expect/panic in
   either repo. `Q-TST-01`, `Q-FLT-01`, `Q-FLT-02` can assume the agent
   runtime will not panic via these mechanisms. The one panic risk is
   the UTF-8 slicing defect below.
2. **One UTF-8 panic defect exists** at `gitignore.rs:179`
   (Q-STA-01-P1-01), reachable via project file-scanning when a `**`
   gitignore pattern meets a non-ASCII path. `Q-FLT-01` should include a
   Unicode-path fault scenario; the fix is straightforward
   (`char_indices()`).
3. **`unsafe` is minimal and sound**: 2 production sites, both guarded
   `env::set_var`. No FFI/raw-pointer risk for security tasks to worry
   about.
4. **~50 framework `#[allow(dead_code)]`** need incremental cleanup.
   Framework `F-*` tasks touching these modules should clean them in the
   same change per AGENTS.md "随手清理".
5. **Duplicate dependencies are significant** (38/76) but detailed
   remediation belongs to `Q-DEP-01`. The high-impact targets are
   `hashbrown` (5 versions), `rand` (3), `thiserror`/`syn` (2 each),
   `reqwest` (2).

Reports downstream tasks must read: this task report plus the four
validation reports under `validations/Q-STA-01/`.

Conditions that make this report stale:

- Any commit adding `.unwrap()`/`.expect()`/`panic!` to production `src/`.
- A fix to `gitignore.rs:179` (would resolve Q-STA-01-P1-01).
- Any change to `Cargo.toml` dependency versions (affects duplicate counts).
- Addition/removal of `#[allow(dead_code)]` annotations.
- Significant restructuring of the 25 oversized files.

Follow-up task IDs (no fixes implemented in this review):

- `Q-DEP-01` — duplicate dependency attribution, advisory scan, license
  review (consumes P2-03).
- `Q-FLT-01` — add a Unicode-path fault scenario to cover Q-STA-01-P1-01.
- `Q-TST-01` — assess test coverage of the panic-safety and UTF-8
  invariants (V01/V02 show the code is clean; tests should guard against
  regression).
- `Q-PERF-01` — resource-lifecycle analysis of the oversized modules
  (consumes P2-02).
- Future maintainability task — split the 5000+ line files and clean
  `#[allow(dead_code)]` incrementally.
