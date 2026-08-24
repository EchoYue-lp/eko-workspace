# F-EVO-01: Eval, improvement, and evolution framework APIs

> Status: complete
> Reviewer: ZCode (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are eval/improve/evolution capabilities valid optional framework APIs with
explicit side effects and without coupling to EKO product policy?

## Scope

- `echo-agent/src/eval/` (9 files: mod, runner, replay, regression, grader,
  comparator, report, server, trigger) and the `eval` feature
  (`echo-agent/Cargo.toml:95`, module gate `src/lib.rs:36-38`).
- `echo-agent/src/improve/` (6 files: mod, trajectory, analyzer,
  eval_improvement, generator, loop) and the `improve` feature
  (`Cargo.toml:96`, gate `src/lib.rs:43-45`).
- `echo-agent/src/evolution/` (17 files: audit, auto_memory,
  background_review, candidate, curator, draft, dreaming, health, layer,
  merge, patch, recall, review, runtime_integration, security, triggers,
  mod) — always compiled (`src/lib.rs:40`).
- Main-path wiring: `src/agent/react/mod.rs` (memory/layer/trigger/skill
  wiring), `src/agent/react/run/{context,execution}.rs`, `src/memory_promoter.rs`,
  `echo-state/src/compression/mod.rs` (promoter invocation), `src/agent/snapshot.rs`.
- EKO consumer side: `echo-agent-app-core/src/evolution/` (dashboard,
  evidence, hook_fire, review_integration, rule_promoter),
  `src/cli/cmd_impls/evolution.rs`, `src/tauri/commands/panels.rs` (rule
  promotion), `echo-agent-app-core/src/{agent_pool,state,infra,run_driver,runtime}.rs`.
- Examples `demo50_eval.rs`, `demo51_self_improvement.rs` and their
  `required-features`; README feature table and docs/en|zh/24-eval-system.md,
  25-self-improvement.md.

## Out Of Scope

- A-EVO-01 (EKO evolution product scope) — this task classifies the
  framework surface and only notes the product adapter boundary.
- EKO task review (`tasks/task_runtime/review.rs`) and framework
  `src/agent/critic` — related-but-distinct feedback mechanisms, not eval
  duplicates (verified in V01-01).
- `src/evolution/curator.rs` skill-lifecycle internals beyond reachability and
  mutation gates (skill lifecycle details belong to F-SKL-01 consumers).
- Live-LLM behavior of `LlmGrader`, `ImprovementLoop`, `PromptGenerator`,
  `SkillDraftGenerator` — no network/LLM fixtures executed (static + unit-test
  coverage only).
- `docs/comprehensive-review/codex/` and `zcode-glm/` — not read per review
  protocol.

## Inputs

- Root `AGENTS.md` (layering, deletion rules, UTF-8/panic safety), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (F-EVO-01 card), `zcode-ds/README.md`.
- Dependency reports (read in full): `F-FEAT-01.md` (feature topology —
  eval/improve are gating features, not no-op markers; README feature-table
  drift P3-01), `F-MEM-01.md` (Store/ConversationStore contracts feeding
  evolution memory layers; FileStore findings).
- Historical documents treated as hypotheses: `echo-agent/AUDIT_REPORT.md`,
  `echo-agent/README.md`, root `AGENTS.md` (echo-agent-eval claim),
  `echo-agent-cli/docs/MASTER-PLAN.md`, `echo-agent-cli/docs/system-deep-dive/06-skills.md`,
  `docs/superpowers/plans/2026-07-10-subagent-parity-roadmap.md`.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | `eval` (EvalCase/SuccessCriteria/EvalRunner/replay/regression/A-B/HTML reports), `improve` (TrajectorySaver + eval-driven improvement loop), `evolution` (typed memory layers, change audit, security guard, curator skill lifecycle, review/merge, dreaming, triggers) are generic framework capabilities, feature-gated or always-compiled, documented with examples and EN/ZH docs. Any unrelated echo-agent consumer would reasonably need them. |
| EKO product policy | EKO's workspace-scoped Review Inbox (`EvidenceStore`, `dashboard`, `hook_fire`), `RulePromoter` criteria and learned-rules.md writes, Dreaming scheduling (`infra.rs:1205`, `spawn_dreaming_task`), CLI slash commands and Tauri `promote_rule` — all product decisions over framework APIs. EKO has **no** eval/improve surface (the historical `echo-agent-eval` crate is removed; the task-briefing premise is stale — see V01-01, V05-01). |
| Adapter boundary | `echo-agent-app-core/src/evolution/` is a thin adapter: `ReviewIntegration` (review_integration.rs:15-60) creates framework plumbing on demand; `agent_pool.rs:662,916` and `state.rs:959,1111` wire `MemoryRuntimeIntegrationBuilder`/`HookEvolutionObserver`; `memory_bridge.rs:450-467` implements `ChangeLog` as a no-op adapter. No second state authority, no scheduling loop duplicated. |
| Duplicate search | Terms: eval, improve, evolution, curator, dreaming, reviewer, review, candidate, promoter, TriggerDetector, CritiqueStore, AbComparator, TrajectoryReplay, RegressionSuite, LlmGrader, echo-agent-eval. Single authoritative definitions in the framework; zero parallel implementations in EKO; `CritiqueStore` exists only in README text; `echo-agent-eval` exists only in docs (removed). See V01-01. |

## Current Path

- **eval (feature-gated)**: `src/lib.rs:36-38` gates the module. `EvalRunner`
  (`src/eval/runner.rs:52-159`) executes `EvalCase` against `dyn Agent` with a
  per-case timeout, evaluates `SuccessCriteria` against output + trace, and
  produces `EvalResult`/`EvalReport`. Side effects are explicit: fixture copy
  to workspace_root (:172-186, :740-748), SWE-bench git clone/checkout/apply
  (:362/:388/:413), `sh -c` test commands (:695-702), agent LLM execution.
  Consumers: `examples/demo50_eval.rs`, the eval-gated improve modules
  (`src/improve/{analyzer,eval_improvement,loop}.rs`), tests. Zero EKO
  consumers — capability menu.
- **improve (feature-gated)**: `src/lib.rs:43-45`. `improve` alone = trajectory
  export (`TrajectorySaver`, default `~/.echo-agent/trajectories/`);
  `improve`+`eval` = Analyzer/PromptGenerator/ImprovementLoop/
  EvalDrivenImprovement (`src/improve/mod.rs:42-49`). All suggestions are
  human-review artifacts (`improve/mod.rs:9-15` safety doc). Zero EKO consumers.
- **evolution (always compiled, main path)**: agent construction wires layer
  manager / trigger sink / curator (`src/agent/react/mod.rs:755-778,1076,1139`);
  every tool batch records sequence/success/failure and curator telemetry
  (`run/execution.rs:54-128`); every turn triggers observation capture with
  persist disposition (`run/context.rs:61-101`); compression promotes evicted
  facts to long-term memory (`echo-state/src/compression/mod.rs:836-838` →
  `src/memory_promoter.rs:70-94`); layer manager writes are security-scanned +
  audited (`src/evolution/layer.rs:845-869`); audit is append+sync JSONL
  (`src/evolution/audit.rs:256`); snapshot persists layer manager/curator
  (`src/agent/snapshot.rs:401-405`). EKO drives review via `ReviewIntegration`
  and schedules Dreaming maintenance (`infra.rs:1205-1213`); rule promotion is
  user-triggered only (CLI `cmd_rule_promote`, Tauri `promote_rule`).

## Findings

### F-EVO-01-P2-01: `SuccessCriteria::TestPass` executes `sh -c` without the shell-command validation the SweBench branch received — inconsistent eval-command safety contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/eval/runner.rs:227` (TestPass → `run_command(command, cwd)` with no validation), `:350` (SweBench branch validates via `validate_shell_command`), `:681-692` (`validate_shell_command` definition), `:695-702` (`run_command` executes `sh -c`)
- Reachability: `EvalRunner::run` (:52) → `check_criteria` (:208) → TestPass branch (:227). Reachable with `--features eval` via any `EvalCase` with `SuccessCriteria::TestPass`; eval cases are authored by the framework consumer (or loaded from an eval dataset — the scenario AUDIT_REPORT.md:119-138 flagged).
- Expected invariant: both criteria variants that execute commands apply the same validation, or the trust boundary is documented per variant; the AUDIT_REPORT recommendation "Sanitize eval runner commands" (AUDIT_REPORT.md:644) applies to all `sh -c` executions.
- Observed behavior: SweBench's `test_command` is rejected when it contains `; | & $ \` > <`, while `TestPass::command` with the same characters executes verbatim via `sh -c`.
- Impact: under the local threat model (eval cases are user-authored) this is defense-in-depth rather than an exploit; but the framework presents an inconsistent command-execution contract (one branch sanitized, the other not), and any shared/imported eval dataset (the scenario the AUDIT_REPORT already called out) turns `TestPass` into arbitrary command execution with no guard. The AUDIT claim is only half-fixed (see V05-01).
- Root cause: `validate_shell_command` was added for the SweBench path only; the pre-existing TestPass path was not migrated.
- Direction: apply `validate_shell_command` in the TestPass branch (return a criteria failure with the rejection message, mirroring :350-355), or explicitly document TestPass as trusted-execution by design and update AUDIT_REPORT.md accordingly. No deletion needed.
- Regression validation: unit fixture — `EvalCase` with `TestPass { command: "echo x; rm -rf /" }` must produce a criteria failure with the metacharacter message and must not execute; existing SweBench validation tests stay green.
- Validation reports: [V03-01](../validations/F-EVO-01/V03-01.md), [V05-01](../validations/F-EVO-01/V05-01.md)

### F-EVO-01-P2-02: L3 `StoreMemoryPromoter` writes typed memories directly to the Store, bypassing the change audit and the security guard — violating the evolution module's documented "all mutations are recorded" invariant on the framework's most frequent automatic write path

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/memory_promoter.rs:70-94` (`TypedMemoryStore::put_typed` directly on the Store; no `MemoryLayerManager`, no `ChangeLog`, no `EvolutionSecurityGuard`); invocation `echo-agent/echo-state/src/compression/mod.rs:836-838` (`promoter.promote(evicted_messages)` after every `prepare()`/`force_compress*()`); wiring `src/agent/react/mod.rs:1033,1047,1123,1177`; framework invariant doc `src/evolution/mod.rs:20-22` ("All mutations to memories, skills, and rules are recorded in the audit log"); contrast audited+scanned write paths `src/evolution/layer.rs:845-869`, `src/tools/builtin/memory.rs:222`, `src/agent/react/run/context.rs:101,794`, `src/evolution/auto_memory.rs:349-379`, `src/evolution/background_review.rs:502`
- Reachability: main path — every ReactAgent with a memory store and compression enabled; promotion fires on every compression pass for every evicted batch of ≥50-char assistant/user/tool messages.
- Expected invariant: every durable memory mutation is recorded in the change log and passes the security guard (secret scan / injection detection / rate limit), as documented for the whole evolution module and as enforced on all layer-manager-mediated writes.
- Observed behavior: L3-promoted facts (the most frequent automatic memory writes) are written unrecorded and unscanned; the audit log is an incomplete mutation record and evicted message content (including any secret-like text in assistant output or tool digests) reaches long-term memory without `SecretScanner` redaction.
- Impact: consumers relying on the audit log as the authoritative mutation record (rollback capability advertised by the module) miss the bulk of automatic writes; the security guard's secret-redaction guarantee silently does not apply to the promotion path. Memory content is still local (no network exposure), so this is not P0/P1.
- Root cause: `StoreMemoryPromoter` predates the layered/audited write path and was wired directly to the raw `Store` instead of through `MemoryLayerManager::write_memory`.
- Direction: route promoter writes through `MemoryLayerManager::write_memory` (WARM_NAMESPACE writes then get security scanning, change-log records, and the observer/hook path), or add change-log recording + security scan inside the promoter; delete the direct `put_typed` in `src/memory_promoter.rs:82-93`. Keep the content-hash dedup key (`durable_memory_content_key`).
- Regression validation: fixture — promote a batch containing a secret-like string (e.g. `sk-...` pattern) and assert (a) the stored content is redacted or the write is rejected, and (b) a `ChangeEntry` exists for the promoted key; existing `memory_promoter` tests (V04-04) and layer tests (V04-02) stay green.
- Validation reports: [V03-01](../validations/F-EVO-01/V03-01.md), [V04-04](../validations/F-EVO-01/V04-04.md)

### F-EVO-01-P3-01: `AbComparator` is an exported public API with zero consumers and zero tests — an unexercised framework option

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/eval/comparator.rs:34` (struct), `:40-52` (`compare` runs two agent factories over eval cases), `src/eval/mod.rs:40` (`pub use comparator::AbComparator`); repo-wide grep (V01-01) finds no caller, no test, no example, no EKO usage
- Reachability: definition + re-export only; no runtime path exercises it.
- Expected invariant: a public exported API has at least one deterministic fixture test or example exercising its contract.
- Observed behavior: `AbComparator::compare` is never called anywhere; the A/B comparison capability is untested and unexercised.
- Impact: consumers copying the API get an untested surface (runs real agents against temp workspace `ab_compare_<uuid>`); a broken A/B path would surface only when a consumer tries it.
- Root cause: capability written as a convenience wrapper over `EvalRunner::run_all` and never integrated into an example or test.
- Direction: per AGENTS.md this pub API is a capability-menu item, not dead code — keep it, but add deterministic tests (two scripted mock agents with distinct outputs over `OutputContains`/`ToolUsed` criteria) or fold it into `demo50_eval.rs`; do not delete.
- Regression validation: `cargo test -p echo_agent --features eval --lib eval::comparator` with the new fixtures green.
- Validation reports: [V01-01](../validations/F-EVO-01/V01-01.md)

### F-EVO-01-P3-02: SweBench git clone/checkout/apply run blocking `std::process::Command` inside the async criteria path — no timeout or cancellation

- Priority: P3
- Confidence: high (static)
- Layer: framework
- Evidence: `echo-agent/src/eval/runner.rs:362,388,413` (`std::process::Command::new("git")...output()`), called synchronously from async `check_criteria` (:208); contrast `run_command` (:695-702) which uses `tokio::process::Command`
- Reachability: `SuccessCriteria::SweBench` with `repo_url`/`base_commit`/`test_patch` — reachable in eval runs.
- Expected invariant: async framework code does not block the runtime worker thread on unbounded synchronous I/O.
- Observed behavior: a hung git operation (large repo, network stall) blocks a runtime worker for the full clone/checkout duration; no per-step timeout; no cancellation from the caller (the eval-level timeout only wraps `agent.execute`, not the git steps).
- Impact: eval runs can stall the async runtime thread; a stuck clone cannot be cancelled even when the caller gives up.
- Root cause: git steps written with `std::process` while the surrounding pipeline is tokio-based.
- Direction: switch the three git steps to `tokio::process::Command` with per-step timeouts (or reuse `run_command`-style helpers); keep the https-only URL check (:338-343).
- Regression validation: fixture test with a deliberately slow/failing git URL asserting the step times out within budget and the eval completes with a failed result.
- Validation reports: [V03-01](../validations/F-EVO-01/V03-01.md)

### F-EVO-01-P3-03: Documentation drift — AGENTS.md and the subagent-parity roadmap still reference the removed `echo-agent-eval` crate; README demo51 description references a nonexistent `CritiqueStore`

- Priority: P3
- Confidence: high
- Layer: application (docs) / framework (README)
- Evidence: root `AGENTS.md:139` (lists `echo-agent-eval` as an EKO submodule) and `:370` (`echo-agent-cli/echo-agent-eval/Cargo.toml` path rule); `docs/superpowers/plans/2026-07-10-subagent-parity-roadmap.md:74,699` (eval crate plans); `echo-agent-cli/docs/system-deep-dive/06-skills.md:427` (documents the crate removal); `echo-agent/README.md:1109` (`CritiqueStore`); workspace reality: `echo-agent-cli/Cargo.toml` `members = ["echo-agent-app-core"]`, `evals/` empty (V01-01)
- Reachability: documentation-only; no code impact. AGENTS.md is the highest-priority constraint document — a maintainer following the "worktree path" rule (:370) would create a nonexistent path; the README demo51 row misleads example readers.
- Expected invariant: constraint documents and README describe only existing workspace members and types.
- Observed behavior: three documents reference a crate/type that does not exist; one document (06-skills.md) correctly records the removal.
- Impact: maintainer confusion, wrong path rules, stale example description; the review-tasking premise "echo-agent-cli has echo-agent-eval sub-crate" derives from this drift.
- Root cause: crate removed (SkillGateway cleanup per 06-skills.md:427) without updating AGENTS.md and the roadmap.
- Direction: update root AGENTS.md (owner: root document maintainer — not editable by this review) to drop the `echo-agent-eval` rows; mark the roadmap items resolved/removed; fix README.md:1109 demo51 description to the actual components (Analyzer, Curator, TrajectorySaver); keep 06-skills.md:427.
- Regression validation: grep `echo-agent-eval|echo_agent_eval` across docs returns zero hits; `CritiqueStore` grep returns zero hits.
- Validation reports: [V01-01](../validations/F-EVO-01/V01-01.md), [V05-01](../validations/F-EVO-01/V05-01.md)

No P0/P1 findings. The overall answer to the task question is affirmative with
two P2 contract gaps: eval/improve/evolution are valid optional framework APIs
with explicit, bounded side effects, no EKO product-policy coupling, and no
duplicate authority; the two P2 findings are about internal invariant
consistency (command-validation parity, audit/security completeness on the
promotion path), not about the API's validity or layering.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories | yes | passed | [V01-01](../validations/F-EVO-01/V01-01.md) |
| V02 | Registration and runtime reachability (feature gating; framework option vs dead code) | yes | passed | [V02-01](../validations/F-EVO-01/V02-01.md) |
| V03 | Invariants/edges: explicit side effects, mutation/review boundaries, EKO decoupling | yes | passed (2 violations → P2-01, P2-02) | [V03-01](../validations/F-EVO-01/V03-01.md) |
| V04 | `cargo test -p echo_agent --features eval,improve --lib eval` | yes | passed, exit 0 | [V04-01](../validations/F-EVO-01/V04-01.md) |
| V04 | `cargo test -p echo_agent --features eval,improve --lib evolution` | yes | passed, exit 0 | [V04-02](../validations/F-EVO-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --features eval,improve --lib improve` | yes | passed, exit 0 | [V04-03](../validations/F-EVO-01/V04-03.md) |
| V04 | `cargo test -p echo_agent --features eval,improve --lib memory_promoter` | yes | passed, exit 0 | [V04-04](../validations/F-EVO-01/V04-04.md) |
| V04 | Feature isolation compile matrix (no-default, eval, improve, eval+improve) | yes | attempt 1 failed (harness quoting), attempt 2 passed, all exit 0 | [V04-05-01](../validations/F-EVO-01/V04-05-01.md), [V04-05-02](../validations/F-EVO-01/V04-05-02.md) |
| V04 | Example compile: demo50_eval (eval), demo51_self_improvement (eval,improve) | yes | passed, exit 0, 0 | [V04-06](../validations/F-EVO-01/V04-06.md) |
| V05 | Historical-document drift (AUDIT_REPORT, AGENTS.md, MASTER-PLAN, README, roadmap, 06-skills) | yes | passed | [V05-01](../validations/F-EVO-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT_REPORT.md:119-138 — eval runner `test_command` passed to `sh -c` without sanitization | partially fixed | SweBench validated (`src/eval/runner.rs:350`); TestPass still unsanitized (:227) → F-EVO-01-P2-01 |
| AUDIT_REPORT.md:592-593 — `src/improve/store.rs` / `src/improve/evolution.rs` silent error discards | stale | files do not exist; current improve surface is trajectory/analyzer/generator/loop/eval_improvement |
| AUDIT_REPORT.md:439,644,655 — trace linkage, sanitize commands, checkout/apply error logging | current | metrics from trace (:116-151), SweBench validation (:338-355) |
| AGENTS.md:139,370 — `echo-agent-eval` is an EKO submodule with Cargo.toml path | stale | crate removed; workspace members = app-core only; `evals/` empty → F-EVO-01-P3-03 |
| docs/superpowers/plans/2026-07-10-subagent-parity-roadmap.md:74,699 — eval crate/new eval plans | stale (aspirational) | references removed crate |
| echo-agent-cli/docs/system-deep-dive/06-skills.md:427 — echo-agent-eval crate removed | current | matches workspace reality |
| echo-agent-cli/docs/MASTER-PLAN.md:62 — "Memory and self-evolution seam closure" Complete | current | workspace-bound curator, shared ReviewIntegration, layered write path, stable dedup keys present |
| echo-agent/README.md:1109 — demo51 description mentions CritiqueStore | stale | no such type → F-EVO-01-P3-03 |
| echo-agent/README.md feature table lacks eval/improve rows | current (pre-existing finding) | F-FEAT-01-P3-01 (cross-referenced, not duplicated) |
| echo-agent/README.md:1180-1181 — eval/self-improvement doc links | current | docs/en|zh/24-eval-system.md, 25-self-improvement.md exist |
| `src/improve/mod.rs:33-38` — storage locations doc | current | `TrajectorySaver::default_dir` → `user_data_path("trajectories")` |
| `src/evolution/mod.rs:20-22` — "All mutations ... recorded in the audit log" | regressed in effect | L3 promoter writes bypass audit + security guard → F-EVO-01-P2-02 |

## Coverage And Uncertainty

- No live-LLM execution: `LlmGrader`, `ImprovementLoop`, `PromptGenerator`,
  `SkillDraftGenerator`, `BackgroundReviewer` review runs were inspected
  statically and their unit fixtures run; their end-to-end behavior with a
  real provider is not exercised (no credentials used by design).
- SweBench end-to-end (git clone + checkout + patch + tests) was not executed
  (network/git dependency); P3-02 is static evidence.
- `src/evolution/curator.rs` lifecycle-transition internals beyond the
  mutation gates (touch/register/promote/deprecate) were checked for callers
  only; the per-transition policy (idle-time windows) is outside this task's
  question.
- The framework's `eval`/`improve` API option value is demonstrated through
  examples + the improve loop + tests; there is no third-party consumer in
  this repository, so "independent consumer value" is argued from the public
  contract (docs, feature gating, examples) rather than observed external use.
- V04-05-01 (harness quoting failure) is kept as immutable evidence; the
  corrected matrix is V04-05-02.
- AGENTS.md drift (P3-03) cannot be fixed by this review (root-owned
  document); it is handed off to the root maintainer/iteration roadmap.

## Handoff

- Downstream tasks may rely on: (1) framework eval/improve/evolution are
  single-authority, feature-isolated, main-path-wired capabilities with no EKO
  product coupling and no duplicate implementation; (2) the two P2 gaps —
  TestPass command validation parity (runner.rs:227) and the L3 promoter
  audit/security bypass (memory_promoter.rs:70-94, compression/mod.rs:836-838);
  (3) the `echo-agent-eval` crate does not exist — A-EVO-01 must review the
  current EKO evolution surface (app-core evolution/ + CLI/Tauri commands)
  against the framework APIs named in this report, not against any eval crate.
- Reports to read: V01-01, V02-01, V03-01, V04-01..06, V05-01 (linked above).
- A-EVO-01 (EKO evolution product scope) should verify the review-inbox
  mutation gate end-to-end and the Dreaming scheduling boundary.
- Q-FW-02/Q-TST-01: the eval/improve/evolution test inventory (V04) is a
  deterministic baseline; the LLM-graded paths remain a coverage gap.
- Iteration roadmap: P2-01 and P2-02 fixes belong in the echo-agent framework
  (src/eval/runner.rs, src/memory_promoter.rs) with the regression fixtures
  specified; P3-01 adds comparator tests; P3-02 migrates git steps to
  tokio; P3-03 updates root AGENTS.md (root-maintainer owned) and README.
- This report becomes stale if: `src/lib.rs` feature gates change; the eval
  runner criteria branches change; `StoreMemoryPromoter`/compression promoter
  wiring changes; the evolution audit/security path changes; the README table
  or AGENTS.md submodule list is rewritten.
- Follow-up task IDs: A-EVO-01, X-BND-01 (facade authority map), X-MEM-01,
  Q-TST-01 (LLM-graded coverage gap).
