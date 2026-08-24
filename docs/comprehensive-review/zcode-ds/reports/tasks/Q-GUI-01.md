# Q-GUI-01: Tauri/GUI Rust matrix

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: both repositories clean before and after every execution;
> `web-frontend/src/generated/` clean (0 modified files) at baseline and after
> each run — no build side effect required a Correction.

## Question

Does the GUI target compile and test under its conditional feature matrix?

**Answer: yes.** Both required validations pass at the reviewed commits with
known exit codes: `cargo check --no-default-features --features gui --bin
echo-agent-tauri --locked` exits 0 (zero warnings), and `cargo test
--no-default-features --features gui --locked` exits 0 with 48/48 unit tests
green, zero warnings, and the `echo-agent-tauri` bin test harness compiled and
linked (proof of a complete GUI link on this host — no missing Tauri system
library). No environment or system-dependency failure occurred, so nothing was
silently skipped.

## Scope

- V01: `cargo check --no-default-features --features gui --bin
  echo-agent-tauri --locked` — full GUI binary type-check under the `gui`
  feature with `tui` excluded (6 m 46 s, exit 0).
- V02: `cargo test --no-default-features --features gui --locked` — GUI
  feature test suite (10 m 45 s incl. compile, exit 0, 48 passed).
- Feature wiring inspected: `echo-agent-cli/Cargo.toml` `gui` feature
  (:29-36: tauri + tauri-plugin-shell/dialog/notification/fs/
  global-shortcut + portable-pty + dashmap + similar + `channels` →
  `echo-agent/channels`), `[[bin]] echo-agent-tauri` with
  `required-features = ["gui"]` (:43-46), `src-tauri/src/main.rs` entry,
  app-core `[features]` (telemetry only; no `gui`), `__ts_rs` check-cfg
  wiring (echo-agent-app-core/Cargo.toml:56 — a cfg for the generation test,
  not a feature enabled by `gui`).
- Baseline protocol per task instruction: `git status --short
  web-frontend/src/generated/` recorded before execution (0 lines) and
  re-checked after each run (0 lines) — the gui build does not trigger
  ts-rs regeneration (`__ts_rs` is not part of the `gui` feature), so no
  `git checkout --` restore was needed.

## Out Of Scope

- Dynamic GUI launch, webview behavior, event flow at runtime -> Q-E2E-01.
- Frontend formatting/tests/build gate -> Q-WEB-01.
- Full workspace all-features gate incl. app-core tests and panic-safety
  Clippy -> Q-CLI-01 (this task's commands intentionally select the root
  package only).
- Tauri command semantics/state/authority, setup composition, duplicate
  projection -> A-SRF-02 (dependency; its P1-01/P2-01 defects are known and
  cross-referenced, not re-filed).
- Frontend/Rust type contracts and generated-artifact drift -> A-FE-01
  (dependency; P3-02 formatting drift not triggered by gui builds).

## Inputs

- Root `AGENTS.md` (full; GUI conditional matrix under "验证分层"), shared
  `README.md`, `REPORTING.md` (validation granularity, completion rule,
  `not_run` policy), `TASKS.md` (Q-GUI-01 card only), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read: `A-SRF-02` (complete — V04-02 gui bin check,
  V04-03 gui lib tests, P1-01 double-setup, P2-01 duplicate projection),
  `A-FE-01` (complete — P3-02 generated-artifact drift, P2-01/P2-02 type
  divergence).
- Historical documents treated as hypotheses: AGENTS.md gate commands and
  A-SRF-02/A-FE-01 validation claims (classified in "Historical Claim
  Status").

## Layering Decision

Gate task; no new abstraction proposed, so no framework-vs-application
placement decision. Classification of what this gate exercises:

- Generic mechanism: Cargo feature/`required-features` machinery,
  `echo-agent/channels` feature toggling (framework, reused as-is).
- EKO product policy (application, correct placement): the `gui` feature set
  and the `echo-agent-tauri` bin are application packaging; the Tauri adapter
  modules under `src/tauri/**` and `src-tauri/` are the reviewed surface.
- Adapter boundary: the GUI matrix is the compile/test gate for the Tauri
  adapter; it does not itself implement any adapter logic.
- Duplicate search: `gui` feature (single definition, Cargo.toml:29-36);
  `echo-agent-tauri` bin (single definition, Cargo.toml:43-46);
  `__ts_rs` (single check-cfg declaration, app-core Cargo.toml:56); no
  duplicate gate commands or parallel GUI feature definitions found.

## Current Path

Verified at the reviewed commits (V01-01, V02-01):

1. Feature topology: root package `echo-agent-cli` defines `gui`; enabling it
   also enables `channels` → `echo-agent/channels` in the framework. The
   workspace member `echo-agent-app-core` has no `gui` feature (telemetry
   only), so `--features gui` applies to the root package alone.
2. Bin wiring: `echo-agent-tauri` (src-tauri/src/main.rs) is gated by
   `required-features = ["gui"]` — the bin cannot be built without the
   feature (the conditional matrix is enforced by Cargo itself).
3. V01 result: the full dependency graph (echo-core/state/tools/execution/
   integration/orchestration/echo_agent, app-core, tauri 2.x tree incl.
   polars/portable-pty) type-checks; `Finished dev profile in 6m 46s`; zero
   warnings.
4. V02 result: 48 unit tests green (tauri chat execution projector, files,
   mcp allowlist/URL validation, conversations projection merge, memory
   namespaces, path_validator denylist, execution-identity helper, plus
   non-gated CLI-module tests), 0 failed, 0 ignored; `echo_agent_tauri` bin
   harness compiled/linked (0 tests in the bin itself); 3 doctests ignored
   (cli/command.rs, logging/mod.rs — pre-existing, CLI-side); whole log: 0
   warnings, 0 errors.
5. No side effects: `web-frontend/src/generated/` unchanged after both runs
   (`__ts_rs` regeneration is not part of the gui build path), whole repo
   clean.

## Findings

### Q-GUI-01-P3-01: The GUI matrix gate is green with zero tests covering boot/setup composition — the known double-`.setup()` defect (A-SRF-02-P1-01) is invisible to this gate

- Priority: P3
- Confidence: high
- Layer: application (test coverage of the adapter surface)
- Evidence:
  - V02-01: the `echo_agent_tauri` bin test harness runs 0 tests; all GUI
    tests are module-level units under `src/tauri/**` (chat.rs, files.rs,
    mcp.rs, conversations.rs, memory.rs, path_validator.rs, mod.rs).
  - `echo-agent-cli/src/tauri/mod.rs:40-68` (first `.setup()`: DevTools +
    `browser://event` forwarder) vs `:311-772` (second `.setup()`: shortcut +
    SubagentEventBus bridge) — no test asserts the builder's setup-slot
    composition (A-SRF-02-P1-01, confirmed again at these commits; same
    deviation already noted in A-SRF-02 V04-03).
- Reachability: every execution of the AGENTS.md GUI matrix — the gate runs
  but cannot fail on boot-composition regressions.
- Expected invariant: the GUI gate should fail (or at least be able to fail)
  when boot-time bridge composition breaks, e.g. via a builder-level test
  asserting exactly one setup closure registering all forwarders.
- Observed behavior: gate passes (exit 0) while `browser://event` has no live
  producer at runtime — the pass gives no evidence about GUI boot/runtime
  composition.
- Impact: a regression of the double-`.setup()` class would sail through this
  matrix; the gate's green status can be misread as "GUI healthy" beyond unit
  level.
- Root cause: no app-level fixture builds/launches the Tauri `Builder` in
  tests; unit tests cover leaf modules only, and the two live setups are a
  composition the unit suite never inspects.
- Direction: add a builder-level test asserting `build_tauri_app`'s builder
  contains exactly one setup closure that registers both the browser
  forwarder and the bridge (mirrors A-SRF-02-P1-01 regression validation);
  keep the matrix commands unchanged. Dynamic smoke remains Q-E2E-01.
- Regression validation: a test that starts a browser session and asserts the
  webview receives `browser://event`; or a unit assertion on the builder's
  single setup closure.
- Validation reports: [V01-01](../validations/Q-GUI-01/V01-01.md), [V02-01](../validations/Q-GUI-01/V02-01.md)

No other findings. The gate question is answered affirmatively with known
exit codes; the matrix behavior itself (feature gating via
`required-features`, no silent skip, no env failure, no build side effects)
is as documented.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | GUI bin check: `cargo check --no-default-features --features gui --bin echo-agent-tauri --locked` | yes | passed | [V01-01](../validations/Q-GUI-01/V01-01.md) (exit 0) |
| V02 | GUI tests: `cargo test --no-default-features --features gui --locked` | yes | passed | [V02-01](../validations/Q-GUI-01/V02-01.md) (exit 0; 48 passed) |
| V03 | Static definition/reachability | not applicable | - | - |
| V04 | Targeted executable check | not applicable | - | - |
| V05 | Historical-document drift | not applicable | - | - |

V03-V05 not applicable by design: the task card declares exactly two required
validations (a compile gate and a test gate); no new abstraction is proposed
(no duplicate/reachability claim beyond the feature wiring documented above,
which the gate itself proves by execution), and the feature wiring was
verified directly by V01/V02 rather than by a separate static pass. No fake
reports are created (REPORTING.md).

All required validations executed; every reported command has a known exit
code; no validation is missing, failed, or `not_run`.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md GUI matrix: `cargo check --no-default-features --features gui --bin echo-agent-tauri` and `cargo test --no-default-features --features gui` | current | both commands executed verbatim (plus `--locked`); V01 exit 0, V02 exit 0 |
| A-SRF-02 V04-02: gui bin check passes at b3b2e81 | current | re-run at same commit, exit 0 (V01-01) |
| A-SRF-02 V04-03: 21 Tauri-layer lib tests pass under gui at b3b2e81 | current | superset re-run: 48/48 green incl. those 21 (V02-01) |
| A-SRF-02 V04-03 deviation: no test covers builder/setup composition or bridge+projector dual path | current | reconfirmed in V02-01 (bin harness 0 tests; no setup test) — Q-GUI-01-P3-01 |
| A-FE-01-P3-02: a fresh ts-rs regeneration writes unformatted output (79 files drift) | current (but not triggered by gui builds) | no regeneration during V01/V02: `__ts_rs` is an app-core check-cfg (Cargo.toml:56), not part of the `gui` feature; generated/ clean before and after |

## Coverage And Uncertainty

- Both commands ran on the same host as the reviewed commits (no code changed
  between dependency reports and this gate).
- `cargo test` (no `--workspace`) selects the root package only; the
  `echo-agent-app-core` workspace member's tests are not part of this gate
  (they belong to the all-features workspace gate, Q-CLI-01 scope). Noted,
  not a defect of the documented command.
- No GUI process was launched: runtime composition, event flow, and webview
  behavior are Q-E2E-01 scope; this gate is compile+unit-test only
  (Q-GUI-01-P3-01).
- Environment: macOS arm64; the `echo_agent_tauri` test harness linked
  successfully, so Tauri system libraries are present on this host — no
  `not_run` was needed for environmental reasons.
- Network-independent run (`--locked`); no crates.io access needed beyond the
  existing Cargo.lock + local cache.

## Handoff

- Conclusions downstream tasks may rely on: the GUI target compiles and tests
  green under its conditional feature matrix at b3b2e81/9b0e0fa (V01/V02);
  the matrix is enforced by Cargo `required-features`; gui builds have no
  `__ts_rs`/generated-file side effects; the gate does not exercise boot
  composition (P3-01) — dynamic verification of `browser://event`, duplicate
  projection, and HITL flow remains Q-E2E-01's job.
- Reports to read: this report + V01-01/V02-01; A-SRF-02 (P1-01 double-setup
  is the known composition defect behind P3-01), A-FE-01 (P3-02 generated
  drift), A-SRF-02 V04-02/V04-03 (prior runs of the same commands).
- Stale triggers: any change to `echo-agent-cli/Cargo.toml` `gui` feature or
  `required-features`, `src-tauri/src/main.rs` entry, or the `src/tauri/**`
  surface re-invalidates these results; a change of reviewed commits makes
  the report stale for the new commit.
- Follow-up task IDs (fixes are not implemented in this review): Q-E2E-01
  (dynamic GUI smoke: browser panel session, duplicate event counts, HITL
  back-to-back turns), Q-CLI-01 (workspace all-features gate incl. app-core
  tests), A-SRF-02 fix milestone (single `.setup()`), Q-WEB-01 (frontend
  gate).
