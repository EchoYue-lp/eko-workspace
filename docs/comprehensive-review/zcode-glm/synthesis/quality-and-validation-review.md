# S-QA-01: Quality and Validation Synthesis

> Synthesis task: S-QA-01 (5th and final synthesis)
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Baseline: `echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`
> Sources synthesized: `Q-STA-01.md`, `Q-DEP-01.md`, `Q-TST-01.md`, `Q-DOC-01.md`, `Q-PERF-01.md`
> Synthesis date: 2026-08-13

This document merges, deduplicates, and prioritizes every finding produced by
the five completed Q-phase validation tasks. Canonical finding IDs retain their
origin-task prefix (e.g. `Q-PERF-01-P1-01`) so every claim is traceable back to
its evidence report. Cross-references into the earlier syntheses
(`framework-review.md`, `application-review.md`, `cross-repository-review.md`,
`iteration-roadmap.md`) resolve overlaps; the net new findings unique to the
Q-phase are listed in Section 1.

---

## 1. Executive Summary

### Finding inventory

| Priority | Count | Notes |
|---|---:|---|
| P0 | 0 | No unrecoverable-system-level defects surfaced by the Q-phase. |
| P1 | 4 | 1 static-safety panic (STA), 2 test-coverage gaps (TST), 1 perf (PERF). |
| P2 | 14 | 3 static hygiene (STA), 1 dep (DEP), 2 test (TST), 4 docs (DOC), 4 perf (PERF). |
| P3 | 13 | 1 static (STA), 2 dep (DEP), 3 test (TST), 5 docs (DOC), 2 perf (PERF). |
| **Total** | **31** | Of which 2 P3 (DOC-01-P3-04/P3-05) are positive confirmations. |

Per-task breakdown:

| Task | P1 | P2 | P3 | Total | Layer focus |
|---|---:|---:|---:|---:|---|
| Q-STA-01 | 1 | 3 | 1 | 5 | static safety, hygiene |
| Q-DEP-01 | 0 | 1 | 2 | 3 | dependency / license health |
| Q-TST-01 | 2 | 2 | 3 | 7 | test credibility, coverage |
| Q-DOC-01 | 0 | 4 | 5 | 9 | documentation drift |
| Q-PERF-01 | 1 | 4 | 2 | 7 | resource-lifecycle, growth |
| **Totals** | **4** | **14** | **13** | **31** | |

### Headline verdict

**The Q-phase confirms the framework and application are structurally sound and
the AGENTS.md panic-safety invariant holds, but it exposes four P1 defects that
each ship through the green gate today:** one process-crashing panic on
non-ASCII paths, two untested production seams that make known P1 defects
invisible, and one O(E²) trace-store write on the live chat path. None are P0
(no unrecoverable data-corruption-with-secret-exposure), and every one of them
is a localized fix — but the combination "the non-streaming answer loop has zero
tests AND the trace store is O(E²) on every event" means the default EKO chat
path is simultaneously uninstrumented and quadratic.

**What is clean (do not re-litigate):**

- **Panic safety is otherwise clean.** Zero production `.unwrap()`/`.expect()`/
  `panic!`/`unreachable!`/`todo!`/`unimplemented!` in either repo (V01-01 of
  Q-STA-01). The one UTF-8 violation (globstar) is the sole panic family
  defect. AGENTS.md "禁止任何会导致系统 panic 的 API" holds end-to-end.
- **`unsafe` is minimal and guarded.** 2 production sites, both
  `std::env::set_var` under Rust edition 2024, both single-threaded-guarded with
  SAFETY comments. No raw pointers / FFI / unions / inline assembly.
- **Licenses are clean.** All 10 Rust crates declare `MIT`; no native system
  dependencies; the only build script is Tauri's GUI-gated `build.rs`.
- **Frontend dependencies are current and maintained** (React 19 / Vite 6 /
  Tailwind v4 / Zustand 5 / TS 5.8 / Vitest 4), no deprecated packages.
- **No flaky or silently-skipped tests.** The ignored/flaky/platform inventory
  is clean (6 documented `#[ignore]`, all with reason strings).
- **Documentation is free of `worker` terminology** (Q-DOC-01-P3-04); the
  Subagent unification holds at the documentation layer.

### The four P1 findings at a glance

| ID | Layer | One-line | Fix size |
|---|---|---|---|
| Q-STA-01-P1-01 | application | `globstar_match` byte-slices a `&str`, panics on non-ASCII paths under `**` patterns; reachable via project file-scan. | S |
| Q-TST-01-P1-01 | framework | The non-streaming `run_react_loop` has zero tests; `react/tests.rs`'s 81 tests never drive a real loop, so the `Ok("")` empty-answer swallow class is invisible. | S (test) |
| Q-TST-01-P1-02 | adapter | Both Anthropic and OpenAI streaming response-parse paths (`convert_response`/`chat_stream`/SSE event handling) are compile-tested only; the F-LLM-03 defects and any future streaming regression ship invisible. | S (test) |
| Q-PERF-01-P1-01 | framework (+ adapter wiring) | `JsonlRunStore.append_event` is load-modify-save of a full `Run` snapshot per event → O(E²) write + O(E²) read; wired live on the chat path (`infra.rs:377`); cache grows unbounded. | M |

---

## 2. Static Safety — Q-STA-01

Q-STA-01 inspected ~690 `.rs` files (490 framework + 200 application), excluding
`target/`, `.worktrees/`, `examples/`, `benches/`, and test-only files for the
production-reachability classification. Each panic-family and slicing match was
traced from definition to enclosing scope (`#[cfg(test)]` / `#[test]` /
production module) to classify reachability.

### 2.1 Findings

| ID | Pri | Layer | Defect |
|---|---|---|---|
| Q-STA-01-P1-01 | P1 | application | `globstar_match` byte-slices `&remaining[j..]` in a `for j in 0..=remaining.len()` loop (`echo-agent-app-core/src/project/gitignore.rs:178-179`). Any multi-byte UTF-8 path (Chinese, emoji) meeting a `**` pattern panics with `byte index … is not a char boundary`. Reachable from `ProjectContext::should_ignore` during file scanning. Fix: `char_indices()` or operate on `as_bytes()`. |
| Q-STA-01-P2-01 | P2 | framework | ~50 production `#[allow(dead_code)]` annotations suppress dead-code lint (struct fields populated but never read, channel-protocol fields, agent-runtime debug fields). Violates AGENTS.md "代码清理…过时代码可直接删"; per-module incremental cleanup. |
| Q-STA-01-P2-02 | P2 | application (largest) + framework | 25 files >1000 lines, 10 >2000, 5 >3000, 2 >5000 (`task_runtime/executor.rs` 6272, `tui/events.rs` 5746). Maintainability / review-surface concern; not a runtime defect. |
| Q-STA-01-P2-03 | P2 | both | Duplicate crate versions: 38 in framework `Cargo.lock`, 76 in CLI (≈30 are platform-gated `windows-*`, not compiled on macOS). High-impact: `hashbrown` 5 versions, `rand`/`rand_core` 3, `thiserror`/`syn` 2, `reqwest` 2, `quick-xml` 4. Detailed attribution → Q-DEP-01. |
| Q-STA-01-P3-01 | P3 | both | No clippy guard for numeric `as` casts (`cast_possible_truncation`/`cast_possible_wrap` not in gate). 318 casts in production; sampled casts all operate on bounded values (no current defect). Optional lint tightening. |

### 2.2 Positive confirmations (already-clean)

- **Panic-family calls: 0 in production.** All ~1710 raw `unwrap`/`expect`/`panic`
  matches are inside `#[cfg(test)]` modules, `#[test]` functions, dedicated
  test files, or example demonstrations (Q-STA-01 V01-01).
- **`unsafe`: 2 sites, both guarded `env::set_var`.** No FFI / raw-pointer /
  union / inline-assembly risk surface (V03-01).
- **UTF-8 slicing: otherwise compliant.** ~45 other `str`-slice sites use
  char-boundary-safe indices; only `gitignore.rs:179` violates the rule.

### 2.3 Cross-reference

`Q-STA-01-P1-01` is the same defect filed as `APP-CROSS-P1-01` / `FW-QUAL-001`
in the application and framework syntheses. `Q-STA-01-P2-03` is consumed and
attributed by Q-DEP-01 (below). `Q-STA-01-P2-02` feeds Q-PERF-01's
resource-lifecycle analysis of the oversized modules.

---

## 3. Dependency Health — Q-DEP-01

Q-DEP-01 attributes each meaningful (compiled-on-macOS) duplicate version in
the two lockfiles to its pulling direct dependency, inspects the frontend
`package.json` for currency/duplication, enumerates all `build.rs` and
native-dep declarations, and reviews license fields across all 10 Rust crates.

### 3.1 Findings

| ID | Pri | Layer | Defect |
|---|---|---|---|
| Q-DEP-01-P2-01 | P2 | both | `hashbrown` resolves to 5 major versions (0.12 / 0.14 / 0.15 / 0.16 / 0.17) in both lockfiles. Leaf-hot crate; inflates compile time + binary size on every build; risk of cross-version type mismatch. Convergence: update the pulling direct deps so the resolver collapses to 1–2 majors; verify with `cargo tree -d`. |
| Q-DEP-01-P3-01 | P3 | framework (research/rag) + application | `quick-xml` resolves to 4 (framework) / 5 (CLI) versions. The research/RAG feature pins a different major than other transitive crates. Lower impact than hashbrown; easy convergence candidate. |
| Q-DEP-01-P3-02 | P3 | application | `@tailwindcss/vite` declared in both `dependencies` (`^4.1.4`) and `devDependencies` (`^4.1.8`) in `web-frontend/package.json`. npm dedupes by hoisting; manifest-hygiene issue only. (Re-flag of `B-BASE-01-P3-02`.) |

### 3.2 Positive confirmations (already-clean)

- **Licenses: all MIT.** All 10 Rust crates (`echo_agent` + 7 sub-crates +
  `echo-agent-app-core` + Tauri/CLI crates) declare `MIT`; no GPL/AGPL/MPL
  anywhere (V04-01).
- **Build scripts: minimal.** Only `echo-agent-cli/build.rs` (Tauri, gated on
  `CARGO_FEATURE_GUI`). No `pkg-config`, `cmake`, or native system-dependency
  crates. The project builds without system prerequisites beyond the Rust
  toolchain (+ WebView for GUI) (V03-01).
- **Frontend currency: clean.** React 19.1.0, Vite 6.3.5, Tailwind v4.1.7,
  Zustand 5.0.6, TypeScript ~5.8.3, Vitest 4.1.10. No deprecated packages (V02-01).
- **AGENTS.md "echo-agent-cli 不需要 SQLite" holds at the dependency layer** — no
  `libsqlite3-sys`/`rusqlite` compiled on the CLI's active feature set.

### 3.3 Inconclusive / not-run

- **RUSTSEC / `cargo audit` advisory scan: NOT RUN.** Requires network access
  to the advisory database, unavailable in the static pass. Recommended
  follow-up outside this catalog. No known advisories are implied or excluded.
- **Full transitive license tree (`cargo-license`):** only direct crate
  `license` fields read; transitive licenses not enumerated.

---

## 4. Test Credibility — Q-TST-01

Q-TST-01 maps production modules to tests, grades 10 critical tests A/B/C,
inventories ignored/flaky/platform-gated tests, and cross-references the
F-TST-01 mock-fidelity baseline. It reviewed at `echo-agent` `3aa7929` — one
post-baseline commit ("M1 test-credibility re-basing — mock 隐身衣 removal")
ahead of the card's `9b0e0fa`; `echo-agent-cli` at baseline `b3b2e81`.

> **Reviewed-commit divergence (per REPORTING.md "Each report records the actual
> reviewed commits").** Q-TST-01 and Q-PERF-01 both reviewed `echo-agent` at
> `3aa7929` (one `fix(tests)` commit ahead of `9b0e0fa`); the remaining three
> Q-tasks reviewed at the baseline `9b0e0fa`. This is recorded for accuracy.
> The M1 commit's nature (test-only "mock invisibility cloak" removal) means it
> *improves* the test-credibility picture relative to baseline; reviewing at
> baseline would have re-reported already-resolved mock-seam findings.

### 4.1 Findings

| ID | Pri | Layer | Defect |
|---|---|---|---|
| Q-TST-01-P1-01 | P1 | framework | **Non-streaming `run_react_loop` has zero tests.** `react/tests.rs`'s 81 tests use `MockAgent` (23 hits) and zero `MockLlmClient`/`then_text`/`then_tool_call` — they never drive the real loop. The `Ok("")` empty-answer/error-swallow class (F-RCT-02-P1-01) and any future non-streaming regression are invisible. `react_smoke.rs:9-12` still carries a stale "deferred" header. |
| Q-TST-01-P1-02 | P1 | adapter | **Both Anthropic AND OpenAI streaming response-parse paths have zero tests.** All provider tests are request-side conversion (`convert_request`, cache plan, attachment). `convert_response`, `chat_stream`, `AnthropicStreamEvent`/`MessageDelta` handling, and the `stream_post` cancel/error loop are compile-tested only. No SSE wire fixture exists. The F-LLM-03 P1/P2 parse defects and any future streaming regression ship invisible — the most defect-dense untested seam. (Broader than zcode-ds P1-03, which covered Anthropic only.) |
| Q-TST-01-P2-01 | P2 | adapter | `revisioned_adapter.rs` (388 lines, the EKO→framework revisioned task-graph boundary) has **zero tests** and no round-trip/field-level conversion test, violating AGENTS.md "适配器必须保持薄且转换无损…转换必须有 round-trip/字段级测试". The framework side has its own round-trip tests; the EKO conversion boundary is compile-tested only. |
| Q-TST-01-P2-02 | P2 | framework | `MockTool` is still text-only. M1 added `with_delay` but no `ToolFailure`/`bytes`/`data`/`truncated`/pagination builder. Structured-failure routing (`category`→`recovery`) and bounded-output/artifact-spill are untested via the mock. F-TST-01-P2-03 remains open. |
| Q-TST-01-P3-01 | P3 | framework | `MockAgent` still emits only `FinalAnswer`; `FailingMockAgent` returns one error variant (`InitializationFailed`). `mock_agent.rs` unchanged by M1. Orchestration tests cannot assert on subagent intermediate-event ordering or diverse failure modes. |
| Q-TST-01-P3-02 | P3 | framework | Mid-stream cancellation is unmodelled (`with_cancel_after_chunks(n)` absent; real transport polls `is_cancelled()` between chunks). `MockTool` inherits the default `execute_with_context` that drops `ToolContext`. Preventive; no audited test depends on these paths today. |
| Q-TST-01-P3-03 | P3 | framework | `test_sliding_window_compressor` is a print-only fixture (zero assertions) — passes regardless of compressor behavior. Net risk is low: the real sliding-window contract IS covered by `invariants.rs` (13 meaningful tests). The defect is "misleading dead test," not "no coverage." |

### 4.2 Positive confirmations (already-clean)

- **M1 lifted the three highest-impact mock-invisibility seams.** The
  streaming-shape (real two-chunk `Delta`+`Terminal` wire shape + `with_stream_script`),
  usage-on-content-chunk accounting (two new `usage_reported` fixtures), and
  batch-ordering (`FuturesUnordered` completion-order → call-order fix +
  `concurrent_batch_results_follow_call_order`) contracts are now mock-faithful.
- **Ignored/flaky/platform-gated inventory is clean.** 6 `#[ignore]` tests, all
  with explicit reason strings (1 pinned-red Q-FLT-01 placeholder, 5 opt-in
  live/credential-gated); 0 frontend skips; platform gates are legitimate
  production branches. No silently-skipped or hidden-flaky test inflates the
  pass rate (V03-01).
- **No duplicate test infrastructure.** The four framework mock types are
  defined exactly once in `src/testing/`; the CLI reuses them via
  `echo_agent::testing::`.

### 4.3 The credibility picture

The store/executor layer (claims, terminal monotonicity, atomic rollback, stale
revisions), subagent recovery, compression `invariants.rs`, the builtin tools,
and the post-M1 streaming channel carry genuinely meaningful invariant tests
(V02 grade-A). Three production seams are compile-tested only and must not be
trusted as regression nets: the non-streaming `run_react_loop`, both provider
streaming response-parse paths, and `revisioned_adapter.rs`.

---

## 5. Documentation Drift — Q-DOC-01

Q-DOC-01 validates READMEs, feature/config references, examples, EKO setup
docs, and architecture claims against the reviewed code and executable commands.
Static, read-only (no `cargo run`/`cargo build` execution; CLI-command defects
established by diffing documented commands against the clap parser).

### 5.1 Findings

| ID | Pri | Layer | Defect |
|---|---|---|---|
| Q-DOC-01-P2-01 | P2 | framework | **echo-agent README feature-flag tables are materially inaccurate.** 2 phantom features (`plan-execute`, `self-reflection` listed in BOTH tables but absent from `Cargo.toml [features]`); 5 wrong `full`-membership flags (`research`/`database`/`content-guard`/`project-rules` marked "no" but in `full`; `sandbox` marked "yes" but not in `full`); 10–14 omitted real features (`lsp`, `statistics`, `eval`, `improve`, `testing`, …). A user copying `features = ["plan-execute"]` gets a Cargo error. |
| Q-DOC-01-P2-02 | P2 | application | **Docs claim SQLite but code doesn't use it — contradicts AGENTS.md.** 7 distinct doc locations (`README.md:56,239,495,557`; `architecture.md:160,190,222`) claim "SQLite + FTS" for session/memory persistence and list `sqlite` (plus `eval`, `improve`) as an enabled echo-agent feature. `Cargo.toml:50` enables only `mcp/lsp/human-loop/subagent/tasks`; `sessions/search.rs:3` explicitly states "EKO is local — no SQLite/FTS5"; AGENTS.md forbids SQLite in CLI. Invites AGENTS.md-violating changes. |
| Q-DOC-01-P2-03 | P2 | application | **`getting-started.md` documents non-existent CLI subcommands and a broken GUI command.** `onboard`, `run`, `sessions list/show/export/delete`, and `--headless` are documented but `args.rs` is flag-only with no `#[command(subcommand)]` field and no `--headless` flag. The GUI command `cargo run --bin echo-agent-tauri` errors under default features (binary requires `gui` feature). Contradicts the CLI README's own correct parameter table (lines 420-428). Every documented setup command errors when copied verbatim. |
| Q-DOC-01-P2-04 | P2 | framework + application | **10 broken relative-path targets.** 4 broken doc links (`docs/en/16-plan-execute.md`, `19-self-reflection.md` — files don't exist), 3 broken example links (`demo14_memory_isolation.rs`, `demo16_testing.rs`, `demo22_plan_execute.rs` — absent), 1 phantom workspace crate (`echo-agents/` in the Workspace Structure diagram — `Cargo.toml` declares 7 members, no such crate), 1 missing LICENSE (CLI badge/footer link `LICENSE` — no file), 1 wrong asset path (`demo42` `cp examples/mcp.json.example` — file is at repo root). |
| Q-DOC-01-P3-01 | P3 | framework | Example-count claims are inconsistent and wrong: "64 runnable demos" (`:294`) vs "66 runnable examples" (`:389`) vs filesystem truth (67 `demoXX.rs` + 1 smoke = 68). The two claims also contradict each other. |
| Q-DOC-01-P3-02 | P3 | framework | README "# Full (default) — all features enabled" inline comment (`:240`) contradicts `default = []` (`:104`) in the same file — bare `echo-agent = "0.2.0"` enables ZERO features, not `full`. Users expecting a working agent from the bare dep get a no-op build. |
| Q-DOC-01-P3-03 | P3 | application | AGENTS.md still references `echo-agent-eval` as a current submodule (`:139` positioning table, `:370` worktree-path rule). Directory absent; CLI workspace members = `["echo-agent-app-core"]` only. Re-flag of `B-DOC-01-P2-02`; unfixed since the 2026-08-12 baseline. |
| Q-DOC-01-P3-04 | P3 (positive) | both | **Docs are free of `worker` terminology** — `grep -rni 'worker[s]?'` across all inspected docs returns 0 concept hits. AGENTS.md "只有 Subagent,没有 Worker" fully reflected in the documentation layer. |
| Q-DOC-01-P3-05 | P3 (positive) | framework | **5 sampled root examples reference current APIs** (demo05/25/42/57/70 — every imported type, macro, builder method, and `AgentEvent` variant resolves to a real definition). Root examples are reliable API documentation; the only nits are non-API (demo42 `cp` path slip). Extends `F-API-01-P3-03` (demo00-03) to 5 more examples. |

### 5.2 Drift pattern

Every documentation defect in Q-DOC-01 is the same root-cause class: **the docs
predate a code change (feature modularization, SQLite removal, CLI redesign,
file deletions, crate folding) and were never refreshed.** The READMEs and
setup doc cannot be trusted as the feature list, CLI surface, or persistence
model without opening the manifest / `args.rs` / `sessions/search.rs` directly.

### 5.3 What downstream tasks must NOT do

- Do NOT use the echo-agent README feature tables as the feature list — read
  `Cargo.toml [features]` directly (precondition for F-FEAT-01's compile matrix).
- Do NOT copy commands from `getting-started.md` — use the `args.rs` flag surface.
- Do NOT believe the "SQLite + FTS" persistence claims — EKO is file/memory-backed.

---

## 6. Performance Risks — Q-PERF-01

Q-PERF-01 traces the write/fanout/cancellation graph on the default EKO chat
path, then reasons analytically about cost (no live model/network fixture).
Reviewed at `echo-agent` `3aa7929` (re-verified all perf anchors at HEAD;
`stream_channel.rs` anchors unchanged by the M1 commit).

### 6.1 Findings

| ID | Pri | Layer | Defect |
|---|---|---|---|
| Q-PERF-01-P1-01 | P1 | framework (defect) + adapter (live wiring) | **`JsonlRunStore` is O(E²) per run, unbounded on disk, and on the live chat path.** `append_event` = `load → push_event → save`: `load_last_line` reads the entire file (`read_to_string`) to find the last line (O(k) per event k), `save` opens the per-run file in `append(true)` and writes `serde_json::to_string(&run)` — a full `Run` snapshot including `events: Vec<RunEvent>`. A run with E events → O(E²) bytes written and O(E²) read work; the on-disk file is E full snapshots; the in-memory `cache` (`RwLock<HashMap<String,Run>>`) gains one entry per run_id, never evicted. Wired live at `infra.rs:377` → every event of every run. For a tool-heavy 100-iteration run (≈300 events) the trace file reaches ~45k-snapshot-lines' worth of cumulative bytes and every event re-reads the whole growing file. Sharpenes `F-OPS-01-P2-01` (adds O(E²) cost + live-path confirmation). |
| Q-PERF-01-P2-01 | P2 | framework (`TaskSpawner`) + application (`FileTaskShadow`, run-dir retention) | In-process run/task registries grow without bound. `TaskSpawner.tasks` DashMap (`background_task.rs:381`) — `prune_completed` exists but has **zero live callers**. `FileTaskShadow.seq_cache` + `run_write_locks` (`file_shadow.rs:26,32`) have no remove/prune/evict/retain calls. Run directories under `~/.eko/tasks/{run_id}/` are never deleted (only temp dirs and `#[cfg(test)]` use `remove_dir_all`). Slow leak, not runaway (concurrency is bounded by semaphore `max_concurrent: 5`). |
| Q-PERF-01-P2-02 | P2 | application | `app.log` and other append-only journals grow without rotation. `app_log_file` opens `~/.eko/logs/app.log` with `create(true).append(true)`; the code comment concedes "rotate/truncate manually if it grows too large." No `tracing_appender` rolling, no size check anywhere. Tool-execution journals (`tool_execution.rs`), checkpoint reflections (`runtime.rs` → `PROJECT.md`), evidence log all append without rotation. Only the TUI log truncates each start. |
| Q-PERF-01-P2-03 | P2 | application (`save_session`) + framework (`FileConversationStore`) | Conversation/session persistence is a full rewrite per save → O(M²) over long chats. `save_session` rebuilds the full `SavedSession` and `write_json`s it atomically; `FileConversationStore.save_messages` rebuilds the whole `ConversationRecord` and `write_record`s it. `chatStore` autoSave fires per message-add/streaming-append. A 1000-message conversation does ~1000 full rewrites. Not a data-loss issue (atomic_write is safe); the clearest "long-chat gets slow" cause. |
| Q-PERF-01-P2-04 | P2 | application | `SessionSearchEngine` holds ~2× conversation content in memory (`IndexedSession` stores both `content_lower` and `raw_content` = two full copies of every indexed session) and scans linearly per query (`String::contains` over all sessions' full content → O(total chars) per keystroke). `reindex_all` reads every conversation JSON file into memory at startup. Memory ≈ 2× the sum of all conversation transcripts. |
| Q-PERF-01-P3-01 | P3 | framework | `max_iterations == 0` silently maps to `usize::MAX` (unlimited ReAct loop). Config footgun; EKO's default is 100 (safe), so latent for EKO. The dead `LoopDetector` (F-RCT-02-P2-01) would have been the secondary guard but is unwired. |
| Q-PERF-01-P3-02 | P3 | framework | ReAct core-loop driver is a detached task (`tokio::spawn` with `JoinHandle` dropped). Cleanup on stream-drop is cooperative (relies on every await noticing a send error), not guaranteed. Explicit cancel works; implicit cancel (drop `rx`) has no `abort()` fallback if a future await becomes non-cancellable. Robustness gap, not an observed deadlock. |

### 6.2 The unbounded-growth map

Q-PERF-01 confirms six distinct unbounded-growth axes, ranked by impact:

1. **`JsonlRunStore` O(E²) write + O(E²) read per run** (P1) — per-event, live
   path, super-linear. Highest impact.
2. **`JsonlRunStore.cache` + `TaskSpawner.tasks` + `FileTaskShadow` maps** (P2-01)
   — monotonic in-process memory over app lifetime.
3. **`~/.eko/runs/`, `~/.eko/tasks/`** (P2-01) — monotonic on-disk growth,
   never deleted.
4. **`app.log` + JSONL journals** (P2-02) — append-only, never rotated.
5. **Conversation/session files** (P2-03) — O(M²) cumulative write bytes per chat.
6. **`SessionSearchEngine`** (P2-04) — 2× transcript resident + linear scan per query.

### 6.3 What is bounded (confirmed clean)

- **Streaming channel buffer** — bounded `mpsc::channel` (default 256,
  `config.rs:235`).
- **DAG/subagent concurrency** — semaphore `max_concurrent: 5` (`executor.rs:69`).
- **Child-process cleanup** — `kill_on_drop` + deadline + `select!` `tx.closed()`
  (`local.rs:406,637-655`); agent pool has idle eviction.
- **Frontend arrays** — capped (`MAX_MESSAGES=500`, `MAX_EVENTS=500`); the
  A-FE-03-P2-01 MessageBubble O(N·T) re-render cost is orthogonal (render cost,
  not array growth).

---

## 7. Unexecuted Matrix Audit

Five of thirteen Q-phase tasks were completed (`Q-STA-01`, `Q-DEP-01`,
`Q-TST-01`, `Q-DOC-01`, `Q-PERF-01`). **Eight Q-phase tasks were NOT executed.**
This section records what each was supposed to cover, per the canonical
`TASKS.md`, and the risk each gap introduces. None of these gaps invalidate the
five completed reports, but they bound confidence in "the code actually
builds and the invariants survive fault injection."

### 7.1 The eight unexecuted tasks

| Task ID | Status | Intended scope (from `TASKS.md`) | Risk introduced by omission |
|---|---|---|---|
| **Q-FW-01** | not run | Framework submission gate: `cargo fmt --check`, all-feature Clippy, panic-safety Clippy (`-D clippy::unwrap_used`/`expect_used`/`panic`/`unreachable`), all-target/all-feature tests, no-default library check on `echo-agent`. | The Q-STA-01 panic-safety audit was *static* (grep + reachability classification); Q-FW-01 would have **executed** the panic-safety Clippy gate. Q-STA-01 explicitly notes (Coverage And Uncertainty) that "whether the explicit unwrap/expect subset is in CI is an open question, partially answered by B-BASE-01-P2-04 (CI does not run conditional matrices)." Suite greenness for the framework is carried from `F-TST-01 V04` + the M1 commit's own pre-commit gate, not independently re-executed. |
| **Q-FW-02** | not run | Framework feature, examples, and docs matrix: one report per independent feature command; examples grouped by identical required features; doctest/document link validation. | Feature-isolation compile errors are NOT independently verified. Q-DOC-01 confirmed the README feature tables are wrong, so any feature-matrix work must read `Cargo.toml` directly first. A feature-gate compile failure (e.g. a `#[cfg]` branch that doesn't compile under some feature combination) could exist undetected. |
| **Q-CLI-01** | not run | EKO Rust submission gate: fmt, all-feature Clippy, panic-safety Clippy, all-feature tests, `echo-agent-app-core` no-default check; dependency-tree SQLite-absence validation. | Same as Q-FW-01 for the application. The SQLite-absence validation is *partially* covered statically by Q-DEP-01 (no `libsqlite3-sys`/`rusqlite` compiled on the CLI's active feature set), but the no-default-feature and all-feature builds were not executed. |
| **Q-GUI-01** | not run | Tauri/GUI Rust matrix: GUI bin check and GUI tests under the conditional feature matrix; system-dependency failures recorded, not silently skipped. | The GUI target was never compiled or tested. Q-DOC-01-P2-03 confirmed `cargo run --bin echo-agent-tauri` errors under default features (requires `gui`); whether the GUI target even compiles under `--features gui` is unverified. The AGENTS.md conditional-matrix gate for GUI is not validated. |
| **Q-WEB-01** | not run | Frontend submission gate: Prettier check, unit/integration tests, production build. | The frontend was never built or tested in this review. Q-DEP-01 confirmed dependency currency but not that `npm test` / `npm run build` / `npx prettier --check` pass. A-FE-03 owns frontend coverage depth; suite greenness is assumed, not verified. |
| **Q-FLT-01** | not run | ReAct and tool fault-injection suite: malformed LLM output, Unicode, huge output, timeout, cancellation, disconnect, crash, partial effects. One report per fault scenario with seeds/fixtures and exact terminal sequence. | The Q-STA-01-P1-01 globstar panic and Q-PERF-01-P3-02 detached-driver cleanup are **not empirically confirmed**. Q-STA-01's Handoff explicitly says "Q-FLT-01 should include a Unicode-path fault scenario." The non-streaming `Ok("")` swallow (behind Q-TST-01-P1-01) and provider streaming parse (behind Q-TST-01-P1-02) have no fault-injection coverage. This is the single largest unmitigated risk class: many of the F-RCT/F-EXT/F-LLM defects are only *statically* characterized, never reproduced. |
| **Q-FLT-02** | not run | Task and Subagent fault-injection suite: stale revisions, old attempts, cancel, timeout, crash, restart, worktree conflict, failed review. | The DAG/claim/Subagent invariants (well-tested at the store level per Q-TST-01) are not stressed under fault. The untested `revisioned_adapter.rs` (Q-TST-01-P2-01) has no fault fixture. Worktree-conflict and restart-recovery behavior is unverified. |
| **Q-E2E-01** | not run | Real multi-surface smoke and parity suite: Chat, Task, Subagent, tool, HITL, attachment, Browser/MCP, restart, large-output scenarios across applicable surfaces with equivalent facts. | No end-to-end or cross-surface parity was executed. The Q-PERF-01 asymptotic cost claims (O(E²), O(M²)) are *analytical*, not measured — a real long-session timing pass would refine magnitudes. Multi-surface fact equivalence (the AGENTS.md "TUI 与 GUI 功能对等" mandate) is unverified dynamically. |

### 7.2 Risk classification of the gaps

- **Execution gates (Q-FW-01, Q-FW-02, Q-CLI-01, Q-GUI-01, Q-WEB-01):** These
  verify "the code builds and the lints/tests pass." Their omission means the
  static findings (Q-STA-01 panic safety, Q-DEP-01 deps, Q-DOC-01 docs) stand,
  but compile-time and test-suite greenness is *assumed from upstream reports*,
  not independently confirmed in the Q-phase. Highest concrete risk: a
  feature-gate compile failure or a GUI build break that no static task can see.
- **Fault injection (Q-FLT-01, Q-FLT-02):** These empirically reproduce the
  defect classes that Q-STA-01/Q-TST-01/Q-PERF-01 characterize statically. Their
  omission means the four P1 defects are *reasoned about*, not *demonstrated*.
  This is the largest unmitigated risk class — the globstar panic is
  analytically certain (byte index on a multi-byte char cannot succeed) but the
  streaming/cancellation/lifecycle defects are probabilistic and need fixtures.
- **End-to-end (Q-E2E-01):** This measures real cost and surface parity. Its
  omission means Q-PERF-01's O(E²)/O(M²) claims are asymptotic (sufficient to
  establish super-linearity, but constant factors and real footprints are
  unmeasured), and the AGENTS.md mode-equivalence mandate is unverified
  dynamically.

### 7.3 What the completed tasks DO cover that partially offsets the gaps

- The ignored/flaky/platform-gated inventory (Q-TST-01 V03) is clean, so the
  *existing* suite is not silently skipping tests.
- Q-DEP-01 statically confirmed the SQLite-absence invariant that Q-CLI-01 would
  have executed.
- Q-STA-01 statically confirmed panic-safety (0 production unwrap/expect/panic)
  that Q-FW-01's panic-safety Clippy would have executed — but only for the
  panic-family calls; Q-FW-01 would also catch compiler-level issues the grep
  cannot.
- Q-DOC-01-P2-03 statically confirmed the GUI binary requires the `gui` feature
  (a `[[bin]]` with `required-features = ["gui"]` cannot build under `default =
  ["tui"]`), which is a robust static determination even without compiling.

---

## 8. Flaky / Inconclusive Classification

### 8.1 Flaky tests: none

Q-TST-01 V03 explicitly classified the ignored/flaky/platform-gated inventory
across both repos + frontend: **6 `#[ignore]` tests, all documented with reason
strings; 0 frontend skips; platform gates are legitimate production branches.
No silently-skipped or hidden-flaky test inflates the pass rate.** The existing
suite has no flakiness to discount.

### 8.2 Inconclusive / not-run validations

These validations could not be completed in their static pass and are recorded
as inconclusive, not silently skipped:

| Validation | Task | Why not run | Status |
|---|---|---|---|
| RUSTSEC / `cargo audit` advisory scan | Q-DEP-01 | Requires network access to the advisory database. | Inconclusive — no known advisories implied or excluded. `cargo audit` is the recommended follow-up. |
| Full transitive license tree (`cargo-license`) | Q-DEP-01 | Only direct crate `license` fields read in the static pass. | Partial — direct licenses all MIT; transitive tree unenumerated. |
| `cargo run --example` / `cargo build` / CLI command execution | Q-DOC-01 | Read-only review. | Statically substituted — CLI-command defects established by diffing documented commands against the clap parser and `required-features` (a robust static determination). |
| Live model/network fixture timing | Q-PERF-01 | No model credentials available. | Analytical only — asymptotic cost claims (O(E²), O(M²)) are established from the write loops; constant factors unmeasured. A real `Q-E2E-01` timing pass would refine magnitudes. |
| Mutation / negative-control test execution | Q-TST-01 | Static review; V04 used negative-control *reasoning* anchored to full reads of the tested code, not executed mutations. | Static judgment; sufficient to identify the compile-tested-only seams. |

### 8.3 Net

**No flaky results. No silently-skipped validations.** The inconclusive items
are all network- or credential-gated checks (RUSTSEC scan, live model timing)
that are honestly recorded as not-run with a recommended follow-up, never
silently passed. The credibility of the five completed Q-phase reports is not
undercut by hidden non-determinism.

---

## 9. Cross-Cutting Observations

### 9.1 The two highest-impact Q-phase findings interact

`Q-PERF-01-P1-01` (O(E²) trace store on the live chat path) and
`Q-TST-01-P1-01` (non-streaming loop has zero tests) compound: the default EKO
chat path is both quadratic in event count AND its answer-returning loop is
uninstrumented. A long non-streaming run that silently returns `Ok("")` (the
F-RCT-02-P1-01 defect class) would also be writing O(E²) trace bytes the whole
time, with no test to catch either failure. Fixing P1-01 (perf) without fixing
P1-01 (test) leaves the regression invisible; fixing the test without fixing
the perf leaves the chat path slow. They should be addressed together.

### 9.2 The "compile-tested only" seams cluster

Three production seams are compile-tested only and must not be trusted as
regression nets until they have failing-then-passing fixtures:

1. Non-streaming `run_react_loop` (Q-TST-01-P1-01).
2. Both provider streaming response-parse paths (Q-TST-01-P1-02) — the most
   defect-dense untested seam, broader than the Anthropic-only zcode-ds
   framing (OpenAI `stream_chat` is equally untested).
3. `revisioned_adapter.rs` (Q-TST-01-P2-01) — violates the AGENTS.md
   adapter-losslessness rule explicitly.

Any fix in these areas must land its own fixture; the green gate cannot be
trusted to catch regressions there today.

### 9.3 The documentation drift is systematic, not incidental

Every Q-DOC-01 defect shares one root cause: docs predate a code change and
were never refreshed. The drift is broad enough that the READMEs and setup doc
cannot be trusted as the feature list, CLI surface, or persistence model. The
most consequential is `Q-DOC-01-P2-02` (SQLite claim) because it actively
invites AGENTS.md-violating changes — a contributor reading the architecture
doc believes EKO uses SQLite and may re-introduce the dependency.

### 9.4 Layer distribution of the four P1s

- **Application:** Q-STA-01-P1-01 (globstar panic, `gitignore.rs` in app-core).
- **Framework:** Q-TST-01-P1-01 (react loop test gap), Q-PERF-01-P1-01 (trace
  store defect, `trace/mod.rs` — made live by the adapter wiring at
  `infra.rs:377`).
- **Adapter:** Q-TST-01-P1-02 (provider streaming parse seam).

This matches the layering pattern in the earlier syntheses: the framework
provides the primitives, the application wires them, and the defects cluster at
wiring/adapter boundaries and in untested core loops.

---

## 10. Action List (folded into the iteration roadmap)

These findings are already sequenced in `iteration-roadmap.md` (S-RDM-01). The
mapping below records the Q-phase's contribution to that roadmap so this
synthesis is self-contained.

### 10.1 Tier-0 (P1 — data integrity / panics / critical perf)

| Finding | Action | Roadmap slot |
|---|---|---|
| Q-STA-01-P1-01 | Rewrite `globstar_match` over `char_indices()` / `Vec<char>`; add Chinese-path regression test. | Milestone 1 (T0-2, = FW-QUAL-001) |
| Q-PERF-01-P1-01 | Make `append_event` a real single-line append (write only the new event); reconstruct `Run` lazily/on-read; add retention bound to `runs/` dir + cache eviction. | Milestone 1 (new — framework + adapter) |
| Q-TST-01-P1-01 | Add `MockLlmClient`-driven test family for `run_react_loop` (text-only turn; core-loop error → typed error not `Ok("")`; max-iteration); delete stale `react_smoke.rs` header. | Milestone 1 / F-RCT-02 |
| Q-TST-01-P1-02 | Add wire-fixture tests with literal SSE/JSON strings for Anthropic `message_delta` usage, interleaved tool/text blocks, malformed-event drop; OpenAI `[DONE]`-terminated stream. | Milestone 1 / F-LLM-03 |

### 10.2 Tier-1/2 (P2 — hygiene, drift, slow leaks)

| Finding cluster | Action |
|---|---|
| Q-DOC-01-P2-01/P3-01/P3-02 | Regenerate echo-agent README feature tables from `Cargo.toml [features]`; fix example count; remove "Full (default)" comment. |
| Q-DOC-01-P2-02 | Remove SQLite/FTS claims from CLI README + `architecture.md`; correct "依赖 Features" table to the 5 enabled features. |
| Q-DOC-01-P2-03 | Rewrite `getting-started.md` against the real `args.rs` surface (flag-only; GUI via `cargo tauri dev`). |
| Q-DOC-01-P2-04/P3-03 | Remove dead doc/example links + phantom `echo-agents` crate; add CLI LICENSE; remove `echo-agent-eval` from AGENTS.md. |
| Q-TST-01-P2-01 | Add `revisioned_adapter.rs` round-trip + field-level test module. |
| Q-TST-01-P2-02 | Add `MockTool::with_failure_structured` / `with_bytes` / `with_data` / `with_truncated` / paginated builders. |
| Q-PERF-01-P2-01 | Wire `prune_completed` into executor run-completion; add eviction to `FileTaskShadow` maps; add run-dir retention policy. |
| Q-PERF-01-P2-02 | Replace `app.log` append with `tracing_appender::rolling` (daily/size); add JSONL journal retention caps. |
| Q-PERF-01-P2-03 | Either accept M² rewrite for local scale (document it) or make message save append a single record + periodic compaction. |
| Q-PERF-01-P2-04 | Store one normalized copy in `SessionSearchEngine`; cap indexed session count/size. |
| Q-DEP-01-P2-01 | Collapse `hashbrown` (5 majors) by updating pulling direct deps; verify with `cargo tree -d`. |

### 10.3 Tier-3 (P3 — incremental, "随手清理")

- Q-STA-01-P2-01: incremental cleanup of ~50 `#[allow(dead_code)]` per AGENTS.md.
- Q-STA-01-P2-02 / Q-PERF-01 oversized files: split 5000+ line files along
  sub-responsibility boundaries (behavior-preserving).
- Q-STA-01-P3-01: optionally add `clippy::cast_possible_truncation`/`cast_possible_wrap`.
- Q-DEP-01-P3-01/P3-02: converge `quick-xml`; fix `@tailwindcss/vite` duplicate.
- Q-TST-01-P3-01/P3-02: enrich `MockAgent`/`FailingMockAgent`; add mid-stream
  cancel + `ToolContext`-aware mock.
- Q-TST-01-P3-03: delete or convert `test_sliding_window_compressor` to assertions.
- Q-PERF-01-P3-01/P3-02: reject/document `max_iterations(0)`; retain detached
  driver `JoinHandle` for `abort()` on stream-drop.
- RUSTSEC `cargo audit`: run as a follow-up outside this catalog.

---

## 11. Conditions That Invalidate This Synthesis

This synthesis is stale if any of the following occurs:

- Any baseline commit change underneath `echo-agent` `9b0e0fa` (or `3aa7929`
  for Q-TST-01/Q-PERF-01) or `echo-agent-cli` `b3b2e81` requires re-running the
  affected source task's validations before trusting its finding references.
- A fix to `gitignore.rs:179` resolves Q-STA-01-P1-01.
- A rewrite of `JsonlRunStore::append_event` to a real single-line append (or
  removal of the live wiring at `infra.rs:377`) resolves Q-PERF-01-P1-01.
- New `MockLlmClient`-driven tests for `run_react_loop` resolve Q-TST-01-P1-01.
- New wire-fixture tests for provider streaming parse resolve Q-TST-01-P1-02.
- A documentation-refresh pass over the echo-agent README (feature tables),
  CLI README + `architecture.md` (SQLite claims), and `getting-started.md`
  (CLI surface) resolves the Q-DOC-01 P2 cluster.
- Execution of any of the eight unexecuted Q-phase tasks
  (Q-FW-01/02, Q-CLI-01, Q-GUI-01, Q-WEB-01, Q-FLT-01/02, Q-E2E-01) would
  sharpen or refute the risk characterizations in Section 7.

## 12. Source Cross-Reference

| This synthesis section | Source task report |
|---|---|
| Section 2 (Static Safety) | `Q-STA-01.md` + validations `Q-STA-01/V01-01..V04-01` |
| Section 3 (Dependency Health) | `Q-DEP-01.md` + validations `Q-DEP-01/V01-01..V04-01` |
| Section 4 (Test Credibility) | `Q-TST-01.md` + validations `Q-TST-01/V01-01..V04-01` |
| Section 5 (Documentation Drift) | `Q-DOC-01.md` + validations `Q-DOC-01/V01-01..V04-01` |
| Section 6 (Performance Risks) | `Q-PERF-01.md` + validations `Q-PERF-01/V01-01..V04-01` |
| Section 7 (Unexecuted Matrix) | canonical `TASKS.md` Phase Q definitions (lines 919-1039) |
| Section 10 (Action List) | `iteration-roadmap.md` (S-RDM-01) Milestones 1-7 |
