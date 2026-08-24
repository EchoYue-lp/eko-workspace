# B-DOC-01: Historical audit and design drift index

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Which existing audit/plan claims still point at current code and which need
targeted revalidation?

## Scope

- `echo-agent/AUDIT_REPORT.md` (2026-05-31): all 25 findings + §6 quality
  items, anchor-checked against commit `9b0e0fa`.
- `docs/MASTER-PLAN.md` (root, 1096 lines): baseline claims (三、当前基线 /
  已知需要校正的事实) and milestone archives sampled.
- `echo-agent-cli/docs/MASTER-PLAN.md` (473 lines, updated 2026-07-29):
  milestone status table and current-decisions sections sampled.
- Obsolete term/path search across both repositories (worker, todo_write,
  plan_create/run_dag, sqlite, echo-agent-eval, web mode).

## Out Of Scope

- Re-reviewing the code behind every claim (per task card); the 45 CLI
  design docs and framework docs trees are indexed by ownership only —
  content-level drift is owned by the F-*/A-*/Q-DOC-01 tasks.
- Dependency advisories (§5 → Q-DEP-01), unwrap distribution (§6.4 →
  Q-STA-01).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (B-DOC-01 card), `zcode-ds/README.md`.
- Dependency reports: zcode-ds `B-ARCH-01`, `B-PATH-01`, `B-BASE-01`.

## Layering Decision

- Generic mechanism: AUDIT findings in framework crates (eval runner,
  plugin variables, sandbox, security helpers).
- EKO product policy: `WebConfig` remnant, web-mode removal, CLI no-SQLite.
- Adapter boundary: none new; the audit's file:line anchors are the
  doc↔code link layer.
- Duplicate search terms: every AUDIT-referenced file (21), every
  MASTER-PLAN milestone anchor sampled (10), plus the obsolete-term set
  from V03. No duplicate claim sets found between the two MASTER-PLAN
  documents beyond intentional cross-references.

## Current Path

`AUDIT_REPORT.md` anchors still resolve at `9b0e0fa` for 19/21 files; the
improve module was restructured (6.3 stale). 19 of 25 findings are fixed in
current code. Both MASTER-PLAN documents are actively maintained
(CLI plan last updated 2026-07-29) and their sampled milestone claims
verify against code (V02).

## Findings

### B-DOC-01-P2-01: Eval runner still executes unsanitized `sh -c` commands

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/eval/runner.rs:695-704`
  (`run_command` → `sh -c` on the raw `test_command`); `:340` validates only
  `repo_url` (https), not the command; audit anchor `src/eval/runner.rs:338-347`
- Reachability: `run_command` is called from the SweBench criteria path
  (`runner.rs:227`); `test_command` originates from eval case definitions
  (audit §1.4; confirmed by the call chain `EvalCase::SweBench`).
- Expected invariant: arbitrary command strings from case data must not
  reach a shell without the same checks as the shell tool.
- Observed behavior: unchanged since the 2026-05-31 audit.
- Impact: a hostile or corrupted eval dataset yields arbitrary command
  execution. Under EKO's local threat model (user-controlled machine,
  local eval files) the severity is materially lower than the audit's
  CRITICAL, but the framework is a reusable crate where eval datasets may
  come from shared sources.
- Root cause: never addressed after the audit; the fix list's item 4
  (validate test_command) was not implemented.
- Direction: route `test_command` through the framework's shlex/risk
  classification or execute via argv without a shell; add a regression test
  rejecting metacharacter payloads.
- Regression validation: eval fixture with `test_command =
  "true; touch /tmp/pwned"` must fail closed; a benign command still runs.
- Validation reports: [V01](../validations/B-DOC-01/V01-01.md),
  [V04](../validations/B-DOC-01/V04-01.md)

### B-DOC-01-P3-01: Production `unsafe { set_var }` remains in echo-core plugin variables

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/plugin/variables.rs:179-190`
  (doc comment acknowledges inherent unsafety; `unsafe` block at :190);
  `echo-integration/src/providers/config.rs:1290-1310` is test-only
  (EnvGuard behind `env_test_lock` mutex)
- Reachability: `export_to_env` is called when plugins declare variables
  (plugin lifecycle); runs at startup/plugin-load in single-threaded
  context in practice, but the framework API is not documented as
  startup-only.
- Expected invariant: no data race on `libc environ` from concurrent
  threads.
- Observed behavior: unchanged since audit §1.9; only the in-comment
  acknowledgment was added.
- Impact: theoretical UB if a second thread reads env concurrently; Rust
  1.84+ makes this unsafe by design. Low practical risk, but the API should
  be narrowed or made single-thread-asserted.
- Root cause: plugin env export predates the 1.84 unsafety; no alternative
  mechanism was designed.
- Direction: gate with a mutex + startup-only assertion, or replace with an
  env-provider abstraction; F-SEC-01 to re-rate under the local model.
- Regression validation: unit test spawning a reader thread during
  `export_to_env` under the mutex (or documenting startup-only contract).
- Validation reports: [V01](../validations/B-DOC-01/V01-01.md)

### B-DOC-01-P3-02: `WebConfig` legacy naming remnant from the removed web mode

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:34,42,340`
  (`pub struct WebConfig`, `Default` impl, `AppState.config.web_config`);
  `state.rs:460-467` constructs it from the agent; no server or command
  reads it (V03)
- Reachability: `WebConfig` is built on every `AppState::from_shared` but
  has no live consumer; the web mode hard-exits at
  `echo-agent-cli/src/main.rs:351-353`.
- Expected invariant: state names reflect live product surfaces.
- Observed behavior: a config struct named after a removed surface is
  still constructed at startup.
- Impact: misleading state inventory; costs a construction + RwLock per
  boot; risks future confusion about a "web" mode.
- Root cause: web-mode removal left the config projection behind.
- Direction: delete `WebConfig`/`web_config` (and its IPC consumers if any)
  or rename to a live concept; A-SRF-04 owns the argument cleanup.
- Regression validation: `cargo check` + frontend build after removing the
  struct and any IPC references.
- Validation reports: [V03](../validations/B-DOC-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Document-to-symbol link check | yes | passed | [V01](../validations/B-DOC-01/V01-01.md) |
| V02 | Completed-milestone code anchor sampling | yes | passed | [V02](../validations/B-DOC-01/V02-01.md) |
| V03 | Obsolete path/term search | yes | passed | [V03](../validations/B-DOC-01/V03-01.md) |
| V04 | Unresolved historical-finding index | yes | passed | [V04](../validations/B-DOC-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT 1.1-1.3, 1.5-1.8, 1.10-1.12, 2.1-2.7, 2.9, 3.1 (19 findings) | fixed | [V01](../validations/B-DOC-01/V01-01.md) |
| AUDIT 1.4 (eval `sh -c`) | current | [V01](../validations/B-DOC-01/V01-01.md), finding P2-01 |
| AUDIT 1.9 (production unsafe set_var) | current | [V01](../validations/B-DOC-01/V01-01.md), finding P3-01 |
| AUDIT 2.8 (parse_method hand-rolled) | current (proptest-hardened) | [V01](../validations/B-DOC-01/V01-01.md) |
| AUDIT 3.2 (fallback Client::new) | current | [V01](../validations/B-DOC-01/V01-01.md) |
| AUDIT 6.1 (security TODOs) / 6.2 (dead-code annotations) | current | [V01](../validations/B-DOC-01/V01-01.md) |
| AUDIT 6.3 (improve/store.rs, evolution.rs) | stale | files removed; new improve module layout; [V01](../validations/B-DOC-01/V01-01.md) |
| CLI MASTER-PLAN milestone claims (10 sampled) | current | [V02](../validations/B-DOC-01/V02-01.md) |
| Root MASTER-PLAN "已知需要校正的事实" (run_code isolation, artifact spill) | current | [V02](../validations/B-DOC-01/V02-01.md) |
| AGENTS.md:139 `echo-agent-eval` submodule | stale | [V03](../validations/B-DOC-01/V03-01.md), B-BASE-01 finding |

## Coverage And Uncertainty

- The 45 CLI design docs and the framework `docs/` tree were inventoried by
  ownership only, not read fully — content drift there is Q-DOC-01's scope.
- `parse_method`'s residual risk (first-`"method":`-occurrence heuristic)
  is mitigated by proptests; a full JSON parse would remove the class.
- No tests were executed in this task; all classifications are static.

## Correction

> Dated: 2026-08-12. Factual correction following independent re-verification
> in `F-SEC-01` (see `zcode-ds/reports/tasks/F-SEC-01.md`).

**`B-DOC-01-P2-01` (eval runner `sh -c`) is downgraded and re-scoped.** The
claim that the audit finding 1.4 is "unchanged" was wrong: the SweBench
criteria path now validates `test_command` via `validate_shell_command`
(`src/eval/runner.rs:342-347`, rejecting `;|&$`><`), in addition to the
`repo_url` https check. The remaining gap is narrower: the rejection list
omits the newline character (`\n`/`\r`), which `sh -c` treats as a command
separator — reclassified as `F-SEC-01-P3-01` under the local threat model
(local eval data, user-to-self), with the fix direction being to add
`\n`/`\r` to the dangerous-chars list.

**Other B-DOC-01 "current" items re-rated by F-SEC-01** (all stay open but
local-model severity is P3): 1.9 unsafe `set_var` → F-SEC-01-P3-08
(documented contract + key validation); 2.8 `parse_method` →
F-SEC-01-P3-09; 3.2 fallback `Client::new()` → F-SEC-01-P3-05 (dead field,
live path uses `ssrf_safe_get`); 6.1 security TODOs → F-SEC-01-P3-10; 6.2
dead-code annotations → F-SEC-01-P3-07 (duplicate output guard at
`execution.rs:225` is dead; live path is `snapshot.rs:878`).

## Handoff

- Downstream tasks may rely on: the AUDIT fixed/current/stale index (V04),
  the verified milestone anchors (V02), and the clean term search (V03).
- `F-SEC-01` re-rates the 6 current AUDIT items under the local threat
  model; `F-EVO-01` owns the restructured improve module; `Q-DEP-01`/
  `Q-STA-01` own the deferred audit sections; `A-SRF-04` owns the dead-arg
  cleanup.
- This report becomes stale if AUDIT/MASTER-PLAN documents are rewritten or
  the classified code anchors change.
