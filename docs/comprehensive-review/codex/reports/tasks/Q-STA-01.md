# Q-STA-01: Static safety and dependency audit

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework had extensive external source changes, so every framework anchor came from committed blobs. CLI had an external `Cargo.lock` change, which was excluded. Only Codex Q-STA reports were added; source, index, README and shared catalog were not changed.

## Question

What panic, direct-index, UTF-8 slicing, overflow, unsafe, dead-code, duplicate
dependency, and oversized-module risks remain at the assigned commits?

## Scope

- All committed Rust package source in the eight-package framework workspace.
- Committed CLI/app-core/Tauri Rust source and direct Cargo manifests; CLI
  `Cargo.lock` excluded.
- Rule families: explicit panic APIs, collection indexing, string byte slicing,
  integer/time/capacity arithmetic, unsafe blocks, dead-code suppressions,
  direct dependency duplication/drift, and >=1000-line module concentration.
- Production/test and definition/registration/reachability classification for
  every adopted candidate.

## Out Of Scope

- Cargo, rustc, Clippy, tests, builds, frontend builds, dynamic fixtures and
  network, all forbidden for this review.
- Resolved dependency graph, advisories and licenses (`Q-DEP-01`).
- CI gate execution/quality (`Q-FW-01`, `Q-CLI-01`, `Q-GUI-01`, `Q-WEB-01`).
- Behavioral re-review of ReAct, Tool, Task, HITL, integrations or security
  findings owned by their functional task cards.
- Deletion of framework public API merely because EKO has no caller.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `TASKS.md`, `REPORTING.md`; Codex
  `README.md`; report templates.
- Sole authorized dependency actually read: `B-BASE-01`. Its framework revision
  is older, so it supplied topology/historical context only; all source and
  manifest facts were reconstructed at the fixed Q-STA commits.
- One unauthorized Codex task report (`X-AUT-01`) was accidentally displayed
  while checking report formatting; V00-10 excludes its complete content. No
  other reviewer directory was read, and no X-AUT conclusion supports Q-STA.
- Ten incomplete/failed/isolation discovery commands are preserved in V00
  reports and excluded. Broad unclassified counts were never used as findings.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Panic-free parsing, checked public numeric inputs, UTF-8-safe previews, safe compressor composition and enforceable unsafe contracts belong to the framework module that accepts each input. |
| EKO product policy | Shell-environment bootstrap timing and project ignore/path behavior belong to EKO. |
| Adapter boundary | EKO may translate configuration and environment into framework values, but must normalize numeric bounds and must not establish process-global safety preconditions after starting a multi-thread runtime. |
| Duplicate search | Searched both committed trees for explicit panic APIs, collection/string indexing, arithmetic by risk term, unsafe, dead-code suppressions, direct manifests, exact dependency uses, line counts and adjacent tests. Definition-only or test-only matches were not counted blindly. |
| Migration deletion | No authority moves repositories. Delete only the proven unused CLI `serde_yaml_ng` declaration. Any future private dead-code deletion requires its own feature-aware caller audit. |

## Current Path

```text
external/malformed text
  PDF metadata bytes -> lossy String -> fixed byte slices
  DuckDuckGo href -> percent_decode String byte slice
    => potential UTF-8 boundary panic

GUI executable
  construct Tokio runtime -> run_desktop_entry -> run_desktop
  -> load_shell_env -> Once -> unsafe process set_var
    (`Once` limits calls, not concurrent environment readers)

public Tool/UI numeric values
  image MB and GUI token limit
  -> naked cast/multiply -> byte / threshold limit
    => debug panic or release wrap/weakened limit
```

Positive conclusions:

- No production explicit `.unwrap()`, `.expect()`, `panic!`, `todo!`, or
  `unreachable!` call was established before inline test modules; doc examples
  and safe `unwrap_or*` fallbacks were classified separately.
- Sampled collection indices are bounded by same-collection loops, `position`,
  `enumerate`, or same-cardinality result allocation. Unsafe string slices are
  owned by the UTF-8 finding rather than hidden in raw index counts.
- EKO's custom gitignore globstar is UTF-8-unsafe, but its only wrapper has no
  production caller; current project tree generation bypasses it. It is a
  future regression constraint, not a current-impact finding.
- `allow(dead_code)` is not treated as proof that framework API is obsolete.
  Current ReAct approval/pipeline helpers are reachable, and feature/protocol
  options remain reasonable framework capabilities.
- Repeated dependency names across publishable framework crates are expected.
  Without metadata/lock analysis, manifest version/feature drift is not a
  proven duplicate-version finding.

## Findings

### Q-STA-01-P1-01: PDF-date and percent decoders byte-slice malformed external text

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/pdf.rs:187`, `:196`, `:219`, `:225`; `echo-agent/echo-tools/src/web/providers/utils.rs:26`, `:31`, `:32`; `echo-agent/echo-tools/src/web/providers/duckduckgo.rs:41`, `:48`.
- Reachability: registered `extract_pdf`/`pdf_info` tools parse untrusted PDF metadata dates; default no-key WebSearch parses DuckDuckGo response hrefs through the shared percent decoder.
- Expected invariant: user/model/file/network/plugin strings may be malformed or Unicode and must return a value/error without slicing between UTF-8 code-unit boundaries.
- Observed behavior: PDF metadata is first converted with `from_utf8_lossy`, then sliced at byte positions 4/6/8 after only a byte-length check. Percent decode slices the two bytes after `%` as a `str`; `%` followed by a multibyte character can make the end offset land inside a code point.
- Impact: a malformed/non-ASCII local PDF date or search response can unwind a registered Tool instead of returning a typed parse fallback/error.
- Root cause: previews/parsers encode character or format assumptions as unchecked `str` byte indices rather than validating ASCII grammar or using character/match boundaries.
- Direction: validate the PDF date grammar on bytes/ASCII before slicing and decode percent escapes from the byte buffer without indexing `str` by unvalidated byte endpoints.
- Regression validation: cover malformed/non-ASCII PDF date bytes, `%€`, truncated/invalid percent sequences, and valid encoded UTF-8; assert no panic and explicit fallback/error semantics.
- Validation reports: [V03-02](../validations/Q-STA-01/V03-02.md), [V03-03](../validations/Q-STA-01/V03-03.md), [V09](../validations/Q-STA-01/V09-01.md)

Eval Unicode slicing is already exactly owned by `F-EVO-01-P1-04`; the Hybrid
checkpoint slice is already exactly owned by `F-CMP-01-P1-07`. V03-01/V03-05
reconfirm those defects but do not contribute new Q-STA scope or counts.

### Q-STA-01-P1-02: Safe startup APIs do not enforce the single-thread requirement for process-environment mutation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/plugin/variables.rs:175`, `:181`, `:186`, `:190`; `echo-agent-cli/echo-agent-app-core/src/infra.rs:1409`, `:1478`, `:1482`, `:1491`; `echo-agent-cli/src/main.rs:59`, `:76`; `echo-agent-cli/src-tauri/src/main.rs:3`, `:4`; `echo-agent-cli/src/tauri/desktop.rs:67`, `:124`, `:130`
- Reachability: framework exposes safe public `export_to_env` with a caller-only single-thread promise; GUI main constructs Tokio runtime and then reaches live `load_shell_env`, which performs unsafe `set_var` inside `Once`.
- Expected invariant: a safe public API cannot require an unenforceable unsafe precondition, and process environment writes occur before any other runtime thread can read the environment.
- Observed behavior: framework docs state the precondition but the safe signature does not enforce it. In EKO, both GUI entry shapes construct/enter Tokio before the write. `Once` prevents multiple blocks from running but does not prevent concurrent readers on runtime/Tauri threads.
- Impact: the unsafe block's own documented condition is unproven; concurrent libc environment access can violate Rust's process-environment safety contract and cause undefined behavior during normal GUI startup.
- Root cause: process-global mutation is used as a configuration transport, and call-count serialization is mistaken for whole-process thread exclusion.
- Direction: pass an explicit captured shell-env map into model/provider/MCP subprocess configuration. If mutation is unavoidable, capture and apply before creating Tokio/Tauri runtime threads; make any residual caller precondition an `unsafe fn` or otherwise structurally enforce it. Do not duplicate environment loaders.
- Regression validation: verify GUI boot resolves shell-only credentials through explicit configuration without post-runtime `set_var`; add a startup ordering test/harness and a compile/API test that safe plugin code cannot trigger unsafe global mutation.
- Validation reports: [V05](../validations/Q-STA-01/V05-01.md), [V09](../validations/Q-STA-01/V09-01.md)

### Q-STA-01-P1-03: ImageFetch and GUI token limits multiply unvalidated public values

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/media/image_fetch.rs:153`, `:177`, `:180`, `:185`; `echo-agent-cli/src/tauri/commands/panels.rs:1014`, `:1029`.
- Reachability: the registered ImageFetch Tool accepts Agent-supplied JSON `u64`; GUI compression stats reads the framework-configured public token limit.
- Expected invariant: externally configurable sizes/durations are validated or use checked/saturating arithmetic before conversion; invalid extremes return a typed validation error or clamp to a documented maximum.
- Observed behavior: image MB casts `u64` to `usize` and multiplies by MiB without validation; GUI computes `token_limit * 3 / 4` in overflow-prone order.
- Impact: extreme yet type-valid config/Tool input panics with overflow checks or wraps in optimized builds, producing an incorrect byte or warning threshold.
- Root cause: numeric validation is decentralized at use sites and arithmetic policy differs between adjacent helpers.
- Direction: cap/validate ImageFetch at its Tool schema/parser boundary and use checked byte conversion; compute the GUI ratio with checked arithmetic or division-first without changing the configured token authority.
- Regression validation: exercise zero, documented maximum, maximum+1, `u64::MAX`/`usize::MAX`, 32-bit conversion where supported, and assert image/token behavior is bounded and non-panicking.
- Validation reports: [V04](../validations/Q-STA-01/V04-01.md), [V09](../validations/Q-STA-01/V09-01.md)

Retry arithmetic is already owned by `F-REL-01-P2-05`, compression by
`F-CMP-01-P1-07`, typed-memory oversampling by `F-MEM-01-P3-01`, and channel
session arithmetic by `F-INT-02-P2-01`; V04 reconfirms but does not recount them.

### Q-STA-01-P3-04: CLI root declares an unused second YAML implementation

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/Cargo.toml:67`, `:68`; `echo-agent-cli/echo-agent-app-core/src/hook_config_loader.rs:215`; `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:299`, `:414`; `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:1449`; `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs:410`; `echo-agent-cli/src/cli/keybindings.rs:48`
- Reachability: every committed YAML parser/serializer call uses `serde_yaml`; exact search finds no `serde_yaml_ng` Rust consumer, target or feature use, while root manifest declares it directly.
- Expected invariant: the application has one intentional YAML parser authority and every direct dependency serves a compiled target.
- Observed behavior: app-core/root source uses `serde_yaml`; the root additionally declares unused `serde_yaml_ng`.
- Impact: needless dependency resolution/compile surface and ambiguity about which YAML semantics new code should use.
- Root cause: a dependency survived an implementation migration without a source-use gate.
- Direction: delete root `serde_yaml_ng`; retain the actually used `serde_yaml` owner. Do not infer or modify lockfile in review.
- Regression validation: remove the declaration, run the CLI locked workspace gate, and search source/manifests for zero `serde_yaml_ng` references.
- Validation reports: [V07-02](../validations/Q-STA-01/V07-02.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Fixed commits and dirty-source isolation | yes | passed | [report](../validations/Q-STA-01/V00-01.md) |
| V00-02 | Broad unclassified count attempt | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-02.md) |
| V00-03 | Wrong Hybrid path attempt | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-03.md) |
| V00-04 | Stale ReAct approval symbols | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-04.md) |
| V00-05 | Wrong repository dependency search | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-05.md) |
| V00-06 | Over-narrow PDF pathspec | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-06.md) |
| V00-07 | Missing example validation path | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-07.md) |
| V00-08 | Cross-repository root manifest attempt | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-08.md) |
| V00-09 | Stale ReAct execution symbols | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-09.md) |
| V00-10 | Unauthorized X-AUT task report read | disclosure | inconclusive, excluded | [report](../validations/Q-STA-01/V00-10.md) |
| V01 | Explicit panic API production classification | yes | passed | [report](../validations/Q-STA-01/V01-01.md) |
| V02 | Direct collection indexing | yes | passed | [report](../validations/Q-STA-01/V02-01.md) |
| V03-01 | Eval Unicode slicing scenario | yes | failed -> F-EVO owner | [report](../validations/Q-STA-01/V03-01.md) |
| V03-02 | PDF metadata scenario | yes | failed -> finding | [report](../validations/Q-STA-01/V03-02.md) |
| V03-03 | Percent decoder scenario | yes | failed -> finding | [report](../validations/Q-STA-01/V03-03.md) |
| V03-04 | EKO gitignore scenario/reachability | yes | failed, unreachable | [report](../validations/Q-STA-01/V03-04.md) |
| V03-05 | Hybrid custom stage scenario | yes | failed -> F-CMP owner | [report](../validations/Q-STA-01/V03-05.md) |
| V04 | Overflow/limit family | yes | failed -> finding | [report](../validations/Q-STA-01/V04-01.md) |
| V05 | Unsafe family and startup reachability | yes | failed -> finding | [report](../validations/Q-STA-01/V05-01.md) |
| V06 | Dead-code suppression classification | yes | passed | [report](../validations/Q-STA-01/V06-01.md) |
| V07-01 | Framework direct dependency drift | yes | passed/no finding | [report](../validations/Q-STA-01/V07-01.md) |
| V07-02 | CLI direct dependency use | yes | failed -> finding | [report](../validations/Q-STA-01/V07-02.md) |
| V08 | Oversized module inventory | yes | passed/risk list | [report](../validations/Q-STA-01/V08-01.md) |
| V09 | Existing edge-case test matrix | yes | failed -> regression gaps | [report](../validations/Q-STA-01/V09-01.md) |
| V10 | Dynamic fixtures/Clippy/build | future | not_run by instruction | [report](../validations/Q-STA-01/V10-01.md) |
| V11 | B-BASE dependency/historical classification | yes | passed | [report](../validations/Q-STA-01/V11-01.md) |
| V99-01 | First integrity gate (`rg -L` misuse) | disclosure | inconclusive | [report](../validations/Q-STA-01/V99-01.md) |
| V99-02 | Corrected report/source integrity gate | yes | passed | [report](../validations/Q-STA-01/V99-02.md) |
| V99-03 | Final post-write integrity gate | yes | passed | [report](../validations/Q-STA-01/V99-03.md) |
| V30 | Primary source reconstruction, deduplication and acceptance | yes | passed | [report](../validations/Q-STA-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root AGENTS: arbitrary input strings must use character iteration and never byte truncate | regressed | V03 identifies five framework scenarios; EKO gitignore is defective but currently unreachable. |
| Root AGENTS: panic APIs and possible overflow require safe alternatives | current as policy, incompletely implemented | V01 finds no explicit production panic calls; V02 finds bounded sampled indices; V04 finds unchecked public arithmetic. |
| Root AGENTS: framework public API cannot be deleted because EKO does not call it | current | V06 retains feature/public options and does not equate no EKO caller with dead framework code. |
| `B-BASE-01`: eight-package framework and two-member CLI workspace | current | V11 plus current manifest inventory. |
| `B-BASE-01`: CLI does not enable SQLite | current | current CLI committed manifests; unrelated to Q-STA findings. |
| `B-BASE-01` CI findings | current dependency ownership not reassessed | Out of Q-STA scope and not duplicated. |

## Coverage And Uncertainty

- Pure static review: none of the candidate panics, extreme arithmetic cases or
  GUI startup races was executed. V10 preserves the required future regression
  matrix; it does not block the source-conclusive review after primary acceptance.
- V00-10 records an unauthorized X-AUT report read. Its entire content is
  excluded, so primary must reconstruct all findings from the fixed commits.
- The explicit panic scan stops at a file's first top-level `#[cfg(test)]`; a
  production item unusually placed after that boundary could be missed.
  Independent category searches reduce, but do not eliminate, this risk.
- No Cargo metadata or CLI lockfile was read. This report cannot state whether
  broad version requirements resolve to duplicate versions or quantify build
  cost/security exposure.
- The unsafe environment impact is based on the unsafe block's own documented
  precondition and visible runtime ordering; actual platform thread schedules
  were not observed.
- The gitignore UTF-8 bug is deliberately unnumbered because current committed
  callers never invoke `should_ignore_path`. This conclusion becomes stale when
  that wrapper is wired.
- Module sizes are exact committed line counts, but responsibility/definition
  counts are static triage. No refactor is justified solely by size.

## Handoff

- Primary independently reconstructed and narrowed P1-01/P1-03, verified P1-02
  through both GUI entry shapes, and verified P3-04 via exact manifest/use search
  in V30.
- Fix order: remove panic/unsafe paths first, then normalize numeric inputs, then
  delete the unused dependency. Keep fixes in existing owner modules; do not add
  a cross-repository safety authority.
- Add the Unicode/extreme fixtures from V09 during implementation. The current
  `needs_evidence` status is intentional until primary acceptance; dynamic
  not_run items remain future validation and are not a static-review blocker.
- This report becomes stale if either reviewed commit changes, if `GitIgnore::
  should_ignore_path` gains a caller, if manifests change, or if environment
  bootstrap moves before runtime construction.

## Primary Acceptance

V30 accepts four unique findings after excluding exact Eval/Hybrid/retry/
compression/memory/session duplicates. Dynamic fixtures and Clippy remain
implementation gates, not prerequisites to the completed static review.
