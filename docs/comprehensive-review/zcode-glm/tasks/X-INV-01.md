# X-INV-01: Repository invariant audit

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

Do both repositories obey Subagent-only terminology, CLI no-SQLite, no
parallel task CRUD, panic safety, UTF-8 safety, and relative path
rules?

## Scope

Primary invariants inspected (one grep-driven validation per
invariant, with cross-references to Q-STA-01 and F-SEC-01):

- V01 — Subagent-only terminology: no `Worker`/`worker` concept in
  any production source file, Cargo.toml, or doc.
- V02 — CLI no-SQLite: zero SQLite dependencies, features, or
  constructor sites in `echo-agent-cli`.
- V03 — No parallel task CRUD: framework's
  `task_create`/`task_update`/`task_list` is the single authority;
  the deleted `todo_write` and `plan_create`/`plan_patch`/`plan_execute`
  tools must not be reintroduced.
- V04 — Relative path rules: all `Cargo.toml` `path =` declarations
  are workspace-relative (no `/Users/...` or `.worktrees/...`).
- V05 — Panic safety: zero production panic-keyword calls
  (cross-ref Q-STA-01); zero computed-index vector out-of-bounds
  (extension over Q-STA-01's `&str` byte-slice audit).
- V06 — UTF-8 safety: no byte-index `&str` slicing, no `len()`-based
  length checks on Unicode strings (cross-ref Q-STA-01 + F-SEC-01).

Search coverage: all `.rs` files under
`echo-agent/**/src/`, `echo-agent-cli/**/src/`,
`echo-agent-cli/src-tauri/src/`; all `Cargo.toml` files; both
`Cargo.lock` files; both `.gitignore` files. `target/` and
`.worktrees/` excluded throughout.

## Out Of Scope

Deferred to named task IDs:

- Full panic-keyword re-audit (1710 matches) → `Q-STA-01` V01 (already
  complete; cross-referenced here).
- Full UTF-8 slicing re-audit (45+ sites) → `Q-STA-01` V02 (already
  complete; cross-referenced here).
- Framework submission gate (fmt/clippy/test execution) → `Q-FW-01`.
- Application Rust submission gate → `Q-CLI-01`.
- Frontend submission gate → `Q-WEB-01`.
- Dependency duplicate/attribution scan → `Q-DEP-01`.

## Inputs

- Repository documents read: root `AGENTS.md` in full (invariants
  sections: "统一术语", "echo-agent-cli 不需要 SQLite", "任务关系
  只有一个权威 API", "Rust 编码硬性约束", "Worktree 并行开发与合并
  规范 §1"), `REPORTING.md`, both report templates, `README.md`
  "Review Invariants" section.
- Dependency task reports read:
  - `zcode-glm/tasks/Q-STA-01.md` (panic-keyword + UTF-8 baseline).
  - `zcode-glm/tasks/F-SEC-01.md` (UTF-8 violation in `rule.rs:55`).
  - `zcode-glm/tasks/B-BASE-01.md` (path hygiene positive, sqlite
    positive — confirmed at the same baseline commits).
- Historical documents treated as hypotheses: none. X-INV-01 is a
  current-state audit.

## Layering Decision

This task spans both repositories and verifies cross-cutting
invariants. The classifications:

| Invariant | Classification |
|---|---|
| Subagent terminology (V01) | Generic mechanism — applies to framework and application equally. |
| CLI no-SQLite (V02) | EKO product policy — `echo-agent-cli` does not enable SQLite; framework may offer it as a reusable option. The application's choice is policy; the framework's offering is a legitimate API menu. |
| No parallel task CRUD (V03) | Generic mechanism — single authority lives in framework (`echo-orchestration/src/tasks/task_tools.rs`); application adds only `task_execute`. |
| Relative path rules (V04) | Adapter boundary — `echo-agent-cli/Cargo.toml` adapts the sibling framework repo via `path = "../echo-agent"`. Relative paths must be restored after worktree development. |
| Panic safety (V05) | Generic mechanism — AGENTS.md "禁止任何会导致系统 panic 的 API" applies to both repos. |
| UTF-8 safety (V06) | Generic mechanism — AGENTS.md "UTF-8 安全,禁止字节级截断" applies to both repos. |

Repository-wide duplicate search terms used:
`\bworker\b`, `\bWorker\b`, `sqlite`/`rusqlite`/`sqlx`,
`SqliteStore`/`SqliteConversationStore`/`SqliteRuntimeStateStore`,
`todo_write`/`plan_create`/`plan_patch`/`plan_execute`,
`worktrees`/`/Users/`, `.unwrap()`/`.expect(`/`panic!(`/`unreachable!`/
`todo!` (cross-ref Q-STA-01), `&s[..]` byte-slice patterns (cross-ref
Q-STA-01).

## Current Path

Six independent grep-driven audits were executed, one per invariant.
Each audit's call graph was traced from definition to live caller:

- V01: no `Worker`/`worker` identifier exists; nothing to trace.
- V02: `echo-agent-cli/Cargo.toml:50` → `echo_agent` dep with features
  `["mcp","lsp","human-loop","subagent","tasks"]` (no `sqlite`) →
  `echo-state` pulled transitively at default features only →
  `infra.rs:1254` constructs `FileRuntimeStateStore`, not any SQLite
  backend. Lockfile confirmed: 0 `rusqlite`/`sqlx`/`libsqlite3-sys`
  packages among 787 total.
- V03: framework authority at
  `echo-agent/echo-orchestration/src/tasks/task_tools.rs:27,94,155`
  (`task_create`/`task_update`/`task_list`); CLI exposure at
  `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs:69` adds
  only `task_execute`. Adapter conversions
  (`TaskUpdateRequest::to_task_plan_patch`) delegate to the framework's
  authoritative `echo_agent::tasks::TaskPlanPatch` /
  `TaskPlanPatchOp` — not a parallel API.
- V04: all 25 `path =` declarations reviewed; every one is
  workspace-relative.
- V05: Q-STA-01 V01-01 baseline reaffirmed (commits unchanged). New
  extension audit on computed vector indices found
  `data_quality.rs:253-254` IQR path:
  `OutlierDetectionTool::execute` (line 168) → guard
  `values.len() < 4` (line 232, admits n=4) → `detect_iqr_outliers`
  (line 249) → `sorted[3 * n / 4.min(n - 1)]` (line 254) evaluates to
  `sorted[4]` when `n == 4`, panicking.
- V06: known sites
  `gitignore.rs:178-180` (Q-STA-01-P1-01) and
  `rule.rs:51-56` (F-SEC-01-P3-01) reaffirmed. No new sites.

## Findings

### X-INV-01-P2-01: IQR outlier detection panics on a numeric column with exactly 4 values

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  `echo-agent/echo-tools/src/data_quality.rs:253-254`
  ```rust
  let q1 = sorted[n / 4.min(n - 1)];
  let q3 = sorted[3 * n / 4.min(n - 1)];
  ```
- Reachability: definition (`detect_iqr_outliers` fn, line 249) →
  registration (called from `OutlierDetectionTool::execute` at line
  237 when `method == "iqr"`) → live caller (the tool is registered
  in `echo-agent/echo-tools/src/registry.rs:151` and again at line
  374, under the `data` feature gate, since `pub mod data_quality` at
  `echo-tools/src/lib.rs:42` is `#[cfg(feature = "data")]`). The
  tool's caller guard at line 232 is `if values.len() < 4 { continue; }`
  — so any column with exactly 4 numeric values reaches
  `detect_iqr_outliers` and panics.
- Expected invariant (AGENTS.md "禁止任何会导致系统 panic 的 API"):
  direct indexing `v[i]` must not be used where `i` can be `>= v.len()`;
  use `v.get(i)` and handle `None`, or precompute a safe index.
- Observed behavior: for `n == 4` the divisor `4.min(n - 1)` evaluates
  to `min(4, 3) = 3`, so `q3_idx = 3 * 4 / 3 = 4`, but `sorted.len()
  == 4` (valid indices 0..=3). `sorted[4]` panics with
  `index out of bounds: the len is 4 but the index is 4`. Reproduced
  in isolation (V05-01). For n=5..=N the index is in bounds.
- Impact: any agent or EKO user invoking the `outlier_detection` tool
  with `method=iqr` (the default) on a CSV/column with exactly 4
  numeric values crashes the process. Realistic likelihood: small
  data samples, sanity-check datasets, or filtered subsets are common
  4-row inputs in exploratory analysis. Severity is bounded by the
  fact that the tool requires opt-in `data` feature and a 4-row
  numeric column.
- Root cause: the divisor `4.min(n - 1)` was intended to clamp the
  quartile stride, but for n=4 it produces a stride of 3 that
  overshoots when multiplied by 3. The off-by-one was missed because
  the existing test `test_outlier_detection_iqr` (line 562) uses 9
  values, far from the boundary.
- Direction: replace both `sorted[...]` direct indexes with
  `sorted.get(...)` and handle the None case, OR replace the body
  with the existing safe `quantile()` helper from
  `echo-agent/echo-tools/src/statistics.rs:195` (which already
  returns `Option<f64>` and is tested at line 257). Add a regression
  test `detect_iqr_outliers_n_equals_4_does_not_panic` that calls the
  tool with a 4-row numeric CSV.
- Regression validation: `cargo test -p echo_tools --features data
  outlier_detection` after the fix must include an n=4 case and pass.
- Validation reports: [V05-01](../validations/X-INV-01/V05-01.md)

### X-INV-01-P3-01: Stale "sqlite-backed" doc comment in CLI infra.rs

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:125`
  ```rust
  /// Shared runtime state store (sqlite-backed). When supplied, the agent
  /// will save `AgentCheckpoint`s + TaskNode DAG entries every iteration.
  pub state_store: Option<Arc<dyn RuntimeStateStore>>,
  ```
- Reachability: the comment is on the public `AgentCreateParams::
  state_store` field, which is constructed in
  `runtime.rs:80` → `infra::create_runtime_state_store()` →
  `infra.rs:1254` returns `FileRuntimeStateStore`, not any SQLite
  backend.
- Expected invariant (AGENTS.md "代码清理:无需兼容,过时代码可直接删"
  + "echo-agent-cli 不需要 SQLite"): doc comments must reflect
  current reality. The CLI never wires SQLite; calling the field
  "sqlite-backed" misleads readers into thinking SQLite is in use,
  which is exactly the kind of stale documentation AGENTS.md says to
  delete.
- Observed behavior: the field's type is the trait object
  `Option<Arc<dyn RuntimeStateStore>>`; the CLI always wires
  `FileRuntimeStateStore`. The comment is leftover from an earlier
  design where SQLite was considered. It is cosmetically wrong but
  has no runtime effect — no SQLite code is reachable.
- Impact: documentation drift; readers may believe the CLI uses
  SQLite, contradicting the no-SQLite invariant (V02). Not a runtime
  defect.
- Root cause: comment not updated when the implementation switched
  from a planned SQLite backend to `FileRuntimeStateStore`.
- Direction: replace "sqlite-backed" with "file-backed" or
  "RuntimeStateStore-backed (file-backed in EKO)". One-line fix; do it
  next time `infra.rs` is touched per AGENTS.md "随手清理是强制要求".
- Regression validation: none (comment-only).
- Validation reports: [V02-01](../validations/X-INV-01/V02-01.md)

### Reaffirmed: Q-STA-01-P1-01 (UTF-8 byte-slice in gitignore.rs)

- Priority: P1 (unchanged)
- Confidence: high
- Layer: application
- Evidence:
  `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:178-180`
  still reads `for j in 0..=remaining.len() { let candidate =
  &remaining[j..]; ... }`.
- This is the Q-STA-01-P1-01 finding, unchanged at commit b3b2e81.
  Cross-referenced from V06-01; not re-promoted to a new X-INV-01
  finding. See [Q-STA-01](Q-STA-01.md) and
  [V06-01](../validations/X-INV-01/V06-01.md).

### Reaffirmed: F-SEC-01-P3-01 (byte-length compare in RuleGuard)

- Priority: P3 (unchanged)
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/rule.rs:51-56` still reads
  `content.len() > max_len` and reports `content.len()` in the block
  reason.
- This is the F-SEC-01-P3-01 finding, unchanged at commit 9b0e0fa.
  Cross-referenced from V06-01; not re-promoted. See
  [F-SEC-01](F-SEC-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Subagent-only terminology (no `worker`/`Worker`) | yes | passed | [V01-01](../validations/X-INV-01/V01-01.md) |
| V02 | CLI no-SQLite (deps, features, constructors, lockfile) | yes | passed (one stale comment, P3-01) | [V02-01](../validations/X-INV-01/V02-01.md) |
| V03 | No parallel task CRUD (framework authority + adapter only) | yes | passed | [V03-01](../validations/X-INV-01/V03-01.md) |
| V04 | Relative path rules in Cargo.toml | yes | passed | [V04-01](../validations/X-INV-01/V04-01.md) |
| V05 | Panic safety (Q-STA-01 cross-ref + computed-index extension) | yes | failed (new finding P2-01) | [V05-01](../validations/X-INV-01/V05-01.md) |
| V06 | UTF-8 safety (Q-STA-01 + F-SEC-01 cross-ref) | yes | failed (two known, no new) | [V06-01](../validations/X-INV-01/V06-01.md) |

The task is `complete` in the sense that every required validation has
a report with a definitive result. Two validations (V05, V06) surface
or reaffirm findings; those findings have known fix directions and are
handed off — fixes are out of scope for this review task.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "统一术语:只有 Subagent,没有 Worker" | current (clean) | V01-01: zero word-boundary `worker`/`Worker` matches in production source; all 31 case-insensitive hits are substrings of `NetworkError`. |
| AGENTS.md "echo-agent-cli 不需要 SQLite" | current (clean) | V02-01: 0 SQLite packages in CLI lockfile (787 total), 0 direct deps, 0 constructor sites, `sqlite` feature not enabled on `echo-state`. One stale comment (P3-01). |
| AGENTS.md "任务关系只有一个权威 API" | current (clean) | V03-01: framework `task_create`/`task_update`/`task_list` trio is the single authority; CLI adds only `task_execute` (permitted); no parallel `todo_write`/`plan_create`/`plan_patch`/`plan_execute` tool registered. |
| AGENTS.md "Worktree 并行开发 §1: relative paths in Cargo.toml" | current (clean) | V04-01: 0 `worktrees` or `/Users/` in any Cargo.toml; all 25 `path =` declarations are workspace-relative. |
| AGENTS.md "禁止任何会导致系统 panic 的 API" | regressed (one new site) | V05-01: Q-STA-01 V01-01 panic-keyword baseline still holds, but computed-index extension finds `data_quality.rs:253-254` (IQR) panics for n=4 (X-INV-01-P2-01). |
| AGENTS.md "UTF-8 安全,禁止字节级截断" | regressed (two known, no new) | V06-01: Q-STA-01-P1-01 (`gitignore.rs:179`) and F-SEC-01-P3-01 (`rule.rs:55`) still present; no new violations. |
| Q-STA-01 V01-01: "0 production unwrap/expect/panic" | current (clean) | V05-01: baseline commits match exactly; conclusion carried forward without re-running the 1710-match classifier. |
| B-BASE-01-P2-01: "Cross-repo path hygiene clean" | current (clean) | V04-01: re-confirmed on same commits. |
| B-BASE-01-P2-02: "CLI never enables sqlite" | current (clean) | V02-01: re-confirmed with deeper lockfile + feature analysis. |
| X-INV-01 task card: "IQR outlier detection" listed under V06 UTF-8 | stale (misclassification) | V06-01: IQR code operates on `Vec<f64>`, not `&str`; no UTF-8 involvement. The IQR site is a direct-index panic, tracked under V05-01 / X-INV-01-P2-01. |

## Coverage And Uncertainty

- **Cross-references to Q-STA-01 V01-01 / V02-01 are taken at face
  value** because the baseline commits match exactly. A full re-run
  of the 1710-match panic-keyword classifier and the 45+-site UTF-8
  audit was not performed — those are Q-STA-01's territory. X-INV-01
  extends coverage only on the computed-index vector dimension
  (V05-01), where one new finding was surfaced.
- **Computed-index vector audit (V05) was targeted, not exhaustive**.
  The IQR site was flagged by the task card and confirmed. A complete
  sweep of all `v[expr_involving_v.len()]` patterns across both repos
  was not done; the residual uncertainty is handed off to a future
  Q-STA-01 V02-02.
- **`eval/runner.rs:728` residual uncertainty** (V06-01): the slice
  `&text[pos..text.len().min(pos + key.len() + 50)]` is safe only if
  `key` is ASCII; current call sites appear to use ASCII keys but
  were not exhaustively enumerated. Deferred to Q-STA-01 V02-02.
- **Frontend (.ts/.tsx) terminology**: a quick
  `grep -rni '\bworker\b' --include='*.ts' --include='*.tsx'` was
  included in V01's broader sweep; no production frontend identifier
  named `Worker` exists. Web-worker API references, if any, would be
  third-party wire names (browser API) and were not separately
  enumerated.
- **Lockfile-based SQLite check (V02)**: a `Cargo.lock` re-generation
  was not forced; the existing lockfile at commit b3b2e81 is the
  evidence. If dependencies change, the lockfile grep must be re-run.
- **No executable validation** was run (no `cargo test`, no
  `cargo clippy`); the IQR panic was reproduced with an isolated Rust
  reproducer that mirrors the index arithmetic, not by exercising the
  real tool end-to-end.

## Handoff

Conclusions downstream tasks may rely on:

1. **Five of six invariants hold cleanly**: Subagent terminology,
   CLI no-SQLite (modulo one stale comment), no parallel task CRUD,
   relative path rules, and Q-STA-01's panic-keyword baseline are
   all clean at commits `9b0e0fa` / `b3b2e81`. These can be relied on
   by subsequent tasks (e.g., `Q-FW-01`, `Q-CLI-01`) without
   re-verification.
2. **One NEW panic-safety violation** exists at
   `echo-agent/echo-tools/src/data_quality.rs:253-254` (IQR
   `q3` index out of bounds for n=4). The `outlier_detection` tool
   is registered under the `data` feature. `Q-FW-01` /
   `Q-FW-02` should include an n=4 fixture when exercising this tool;
   `Q-TST-01` should note that the existing
   `test_outlier_detection_iqr` does not cover the boundary.
3. **Two pre-existing UTF-8 violations** (Q-STA-01-P1-01 in
   `gitignore.rs:179`; F-SEC-01-P3-01 in `rule.rs:55`) remain open
   and unchanged. No new UTF-8 violation was introduced. Any
   fault-injection task (`Q-FLT-01` if opened) should cover both
   sites plus the new IQR panic site.
4. **The X-INV-01 task card's classification of "IQR outlier
   detection" under V06 UTF-8 was a misclassification**: the IQR
   code has no UTF-8 involvement; it is a direct-index panic tracked
   under V05.

Reports downstream tasks must read: this task report plus the six
validation reports under `validations/X-INV-01/`. For panic-keyword
and broader UTF-8 context, also read
`zcode-glm/tasks/Q-STA-01.md` and `zcode-glm/tasks/F-SEC-01.md`.

Conditions that make this report stale:

- Any commit that adds a `worker`/`Worker` identifier in production
  source (would break V01).
- Any commit that adds `rusqlite`/`sqlx` to the CLI lockfile or
  enables `sqlite` on `echo-state` from the CLI (would break V02).
- Any commit that registers a `todo_write`/`plan_create`/
  `plan_patch`/`plan_execute` tool (would break V03).
- Any commit that introduces `/Users/...` or `.worktrees/...` in a
  committed `Cargo.toml` (would break V04).
- A fix to `data_quality.rs:253-254` (would resolve X-INV-01-P2-01).
- A fix to `gitignore.rs:179` (would resolve Q-STA-01-P1-01).
- A fix to `rule.rs:51-56` (would resolve F-SEC-01-P3-01).
- Any commit adding a new `.unwrap()`/`.expect()`/`panic!` to
  production `src/` (would invalidate the Q-STA-01 V01-01
  cross-reference).
- A `Cargo.lock` regeneration that pulls SQLite transitively (would
  invalidate V02's lockfile grep).

Follow-up task IDs (no fixes implemented in this review):

- A targeted **Q-STA-01 V02-02** ("UTF-8 + computed-index re-audit")
  should extend Q-STA-01's direct-index search to all
  `v[expr_involving_v.len()]` patterns across both repos and
  confirm `eval/runner.rs:728` call sites use ASCII `key` values.
- **Q-FW-02** (feature matrix) should exercise the `data` feature's
  `outlier_detection` tool with an n=4 fixture after the IQR fix
  lands.
- The fixes for X-INV-01-P2-01, X-INV-01-P3-01, Q-STA-01-P1-01, and
  F-SEC-01-P3-01 belong to a future implementation milestone, not
  this review task.
