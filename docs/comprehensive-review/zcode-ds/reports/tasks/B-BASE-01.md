# B-BASE-01: Repository and build topology

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

What packages, workspace members, targets, features, optional dependencies,
and cross-repository path dependencies exist now, and does CI enforce the
declared repository gates?

## Scope

- All 8 framework `Cargo.toml` files, both CLI `Cargo.toml` files, both
  `Cargo.lock` files, normalized `cargo metadata --locked` output.
- `echo-agent-cli/build.rs`, `web-frontend/package.json`, `.prettierrc`,
  `.prettierignore`.
- Both `.github/workflows/rust-ci.yml` files.
- Root `AGENTS.md` gate definitions and CLI composition claim.

## Out Of Scope

- Per-feature compilation (`F-FEAT-01`, `Q-FW-02`).
- Dependency advisories / licenses / duplicate versions (`Q-DEP-01`).
- Runtime entry-point composition (`B-PATH-01`).
- Actual gate execution results (`Q-FW-01`, `Q-CLI-01`, `Q-GUI-01`, `Q-WEB-01`).

## Inputs

- Root `AGENTS.md` read in full.
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md` (task
  B-BASE-01 only), this track's `README.md`.
- No dependency reports; task has no dependencies.
- No historical audit conclusion accepted as evidence.

## Layering Decision

- Generic mechanism: the eight-package framework workspace, its feature
  forwarding, and its CI topology are independent framework build facts.
- EKO product policy: the CLI workspace, TUI/GUI feature split, app-core
  feature selection, frontend scripts, and the CLI CI belong to the
  application.
- Adapter boundary: the relative path dependencies
  (`../echo-agent`, `../echo-agent/echo-core`) are the build-time adapter
  boundary; CI supplies that path from a checkout and is where the
  framework/application contract is tested.
- Duplicate search: every `Cargo.toml`, `package.json`, build script,
  workflow, and lockfile in both repositories; greps for `worktrees` and
  `/Users/` absolute paths (zero hits); grep for SQLite crates in CLI
  lockfiles. No second active workspace or duplicate frontend package found.

## Current Path

`echo-agent/Cargo.toml:1-21`: root package + 7 sub-crates, resolver v3, all 8
in `default-members`. Root facade depends on all 7 members
(`echo-agent/Cargo.toml:106-112`); sub-crates depend only on
`echo_core`/`echo_macros` (full graph in [V01](../validations/B-BASE-01/V01-01.md)).
Root features (`Cargo.toml:65-103`) forward integration/state/tool
capabilities to owner crates; 13 are empty marker features.

`echo-agent-cli/Cargo.toml:1-5`: root package + `echo-agent-app-core`, resolver
v3. Both depend on the sibling framework via relative paths
(`echo-agent-cli/Cargo.toml:50`, `echo-agent-app-core/Cargo.toml:10`);
neither enables `sqlite` or `database`. Default binary is the TUI/CLI
`echo-agent-cli` (`default-run`, `Cargo.toml:12`, `:39-41`); the Tauri desktop
binary is `gui`-gated (`Cargo.toml:43-46`) and built only when
`CARGO_FEATURE_GUI` is set (`build.rs:3-11`). The React/Vite frontend is an
npm package with `test`/`build` scripts (`web-frontend/package.json:6-14`).

## Findings

### B-BASE-01-P2-01: CLI CI never validates the shipped frontend

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/.github/workflows/rust-ci.yml:37-45`;
  `echo-agent-cli/web-frontend/package.json:6-14`; root `AGENTS.md:301-305`
- Reachability: every push to main/master and every PR enters the sole `ci`
  job, which ends at the Rust commands (workflow line 45) and contains no
  Node step, no `npm` command, and no reference to `web-frontend`.
- Expected invariant: frontend changes are checked with `npx prettier
  --check`, `npm test`, and `npm run build` before merge (the declared
  conditional gate for any `web-frontend/` change).
- Observed behavior: no setup-node, no prettier/test/build command anywhere
  in the workflow.
- Impact: type errors, failing vitest suites, formatting drift, or an invalid
  `tsc -b && vite build` production build can merge while CI stays green; the
  repository's own gate is unenforced.
- Root cause: the workflow was written for the Rust workspace only, before
  the React frontend became a shipped product surface.
- Direction: add a frontend job (setup-node with the committed
  `package-lock.json`, then the three AGENTS.md commands as separate steps);
  do not fold them into one opaque command.
- Regression validation: introduce one formatting, one test, and one
  type/build failure in separate negative controls, verify each fails CI,
  then revert.
- Validation reports: [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P2-02: CLI CI's framework input is unpinned and non-reproducible

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/.github/workflows/rust-ci.yml:15-21`;
  `echo-agent-cli/Cargo.toml:50`; `echo-agent-cli/echo-agent-app-core/Cargo.toml:10`
- Reachability: every CI run checks out `EchoYue-lp/echo-agent` without a
  `ref` (line 18), symlinks it to the exact sibling path both manifests
  resolve (`ln -s` at line 21), and compiles against whatever commit is at
  the framework default branch at that moment.
- Expected invariant: rerunning CI for an unchanged CLI commit tests the same
  framework contract, or an explicitly declared compatibility lane.
- Observed behavior: the framework revision is neither pinned in the workflow
  nor represented by a commit-bearing dependency; it can differ between runs.
- Impact: the same CLI commit can go green→red without any CLI change; a
  result cannot be reproduced from the CLI commit + lockfile alone. A
  dependency-graph change on framework main can additionally make the CLI's
  `--locked` gate fail without CLI edits.
- Root cause: two independent repositories are connected by a local path
  dependency, and CI substitutes an unpinned checkout for that path.
- Direction: pin a compatible framework revision for the required gate, and
  optionally add a separate scheduled/allow-failure latest-main lane.
- Regression validation: rerun the pinned gate twice from the same CLI commit
  and verify the resolved framework SHA is identical and logged.
- Validation reports: [V01](../validations/B-BASE-01/V01-01.md),
  [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P2-03: Framework CI does not execute the declared all-target test gate

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/.github/workflows/rust-ci.yml:61-62`;
  `echo-agent/.github/workflows/rust-ci.yml:27-30`; root `AGENTS.md:259`
- Reachability: the test job matrix runs `cargo test -p <package> --lib
  --tests --all-features --locked` per package; the lint job compiles
  examples/benches with `--all-targets` but never executes their test
  harnesses; doctests are not executed either (`--lib --tests` excludes
  `--doc`).
- Expected invariant: CI enforces `cargo test --workspace --all-targets
  --all-features --locked`, the mandatory framework submission gate.
- Observed behavior: target execution is restricted to lib unit tests and
  integration tests (`cache_user_id_test`, `react_smoke`); the 68 examples
  and the `agent_bench` (harness=false) are compiled but not run.
- Impact: a test placed in an example/bench target, or behavior only
  exercised by such a harness, is not executed by CI even though the
  repository gate promises all-target execution.
- Root cause: the package-split test optimization changed target semantics;
  the workflow comment treats Clippy compilation as equivalent coverage.
- Direction: keep package-level parallelism if useful, but add `--all-targets`
  (and `--doc`) to each matrix invocation or add one exact gate command.
- Regression validation: add a temporary failing test inside an example as a
  negative control and verify the CI test job detects it.
- Validation reports: [V03](../validations/B-BASE-01/V03-01.md),
  [V04](../validations/B-BASE-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Manifest/member inventory | yes | passed | [V01](../validations/B-BASE-01/V01-01.md) |
| V02 | Feature-to-dependency graph | yes | passed | [V02](../validations/B-BASE-01/V02-01.md) |
| V03 | Target/required-feature inventory | yes | passed | [V03](../validations/B-BASE-01/V03-01.md) |
| V04 | CI-versus-AGENTS gate comparison | yes | failed | [V04](../validations/B-BASE-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md:139`: CLI contains `echo-agent-eval` (评测) submodule | stale | no such directory at `b3b2e81f2b2d`; empty `evals/` dir exists instead; [V01](../validations/B-BASE-01/V01-01.md) |
| Root `AGENTS.md`: framework is a root package plus seven subcrates | current | [V01](../validations/B-BASE-01/V01-01.md) |
| Root `AGENTS.md`: CLI is a workspace; EKO does not enable SQLite | current | [V01](../validations/B-BASE-01/V01-01.md), [V02](../validations/B-BASE-01/V02-01.md) |
| Root `AGENTS.md`: CI depends on the listed mandatory gates | regressed | [V04](../validations/B-BASE-01/V04-01.md) |
| Review README baseline: CLI "377 TS/TSX files" | stale | `web-frontend/src` contains 218 `.ts/.tsx` files (219 including root tsconfigs is not the claim); 200 Rust files confirmed; [V03](../validations/B-BASE-01/V03-01.md) |

## Coverage And Uncertainty

No build or test command was executed; `Q-*` quality tasks own those
validations. The third-party resolved graph was not audited (Q-DEP-01). The
meaning of the 13 empty marker features requires source-level cfg inspection
(F-FEAT-01). This task establishes build topology, not runtime reachability.

## Handoff

- `B-ARCH-01` may rely on the eight-package inventory and the acyclic path
  dependency graph, but must independently inspect module/re-export
  ownership.
- `B-PATH-01` may rely on the EKO target inventory (lib + `echo-agent-cli`
  bin + `gui`-gated Tauri bin), but not infer feature parity from target
  existence.
- `B-DOC-01` should classify the stale `echo-agent-eval` claim, the
  TS/TSX-count drift, and the CI gate regressions.
- `F-FEAT-01` owns marker-feature reachability and per-feature compile checks.
- This report becomes stale if either manifest tree, either workflow, the
  frontend package scripts, or the reviewed commits change.
