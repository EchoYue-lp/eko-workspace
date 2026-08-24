# B-BASE-01: Repository and build topology

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean

## Question

What packages, workspace members, targets, features, optional dependencies, and
cross-repository path dependencies exist now, and does CI enforce the declared
repository gates?

## Scope

- All `Cargo.toml` manifests and normalized `cargo metadata --locked` output in
  both repositories.
- Both `Cargo.lock` files as resolution inputs.
- `echo-agent-cli/web-frontend/package.json` and its lockfile.
- `echo-agent-cli/build.rs`.
- Both `.github/workflows/rust-ci.yml` files.
- Root `AGENTS.md` submission and conditional gate definitions.

## Out Of Scope

- Compilation of individual feature combinations (`F-FEAT-01`, `Q-FW-02`).
- Dependency advisories, licenses, and duplicate versions (`Q-DEP-01`).
- Runtime entry-point composition (`B-PATH-01`).
- Test quality or actual gate execution (`Q-FW-01`, `Q-CLI-01`, `Q-GUI-01`,
  `Q-WEB-01`).

## Inputs

- Root `AGENTS.md` read in full.
- `docs/comprehensive-review/README.md`, `REPORTING.md`, and the
  `B-BASE-01` task card.
- No dependency task reports; this task has no dependencies.
- No historical audit finding was accepted as evidence.

## Layering Decision

- Generic mechanism: the eight-package `echo-agent` workspace and its feature
  forwarding are independent framework build topology.
- EKO product policy: the CLI workspace, GUI/TUI feature split, app-core feature
  selection, and frontend scripts belong to the application.
- Adapter boundary: relative path dependencies from EKO to the framework are
  the build-time adapter boundary. They enable framework capabilities but do
  not themselves own runtime state.
- Duplicate search: every non-target `Cargo.toml`, `package.json`, build script,
  workflow, absolute/worktree path, and normalized path dependency was searched
  across both repositories. No second active Rust workspace or duplicate
  frontend package was found.

## Current Path

`echo-agent/Cargo.toml:1` makes the root package and seven subcrates one resolver
v3 workspace. The root facade depends on all seven members at
`echo-agent/Cargo.toml:105`; lower-crate dependencies are captured in
[V01](../validations/B-BASE-01/V01-02.md) and become the input to
`B-ARCH-01`. Root features at `echo-agent/Cargo.toml:65` forward integration,
state, and domain-tool capabilities to their owner crates while root-owned
capabilities use local feature gates.

`echo-agent-cli/Cargo.toml:1` makes `echo-agent-app-core` the only submember;
the root package itself is also a workspace member. Both packages depend on the
sibling framework through relative paths (`echo-agent-cli/Cargo.toml:49` and
`echo-agent-cli/echo-agent-app-core/Cargo.toml:10`). Neither selects `sqlite`.
The app exposes a default TUI/CLI binary and a `gui`-required Tauri binary
(`echo-agent-cli/Cargo.toml:21`, `:39`, `:43`). The React/Vite frontend is an
npm package with test and build scripts at
`echo-agent-cli/web-frontend/package.json:6`.

## Findings

### B-BASE-01-P2-01: CLI CI leaves the entire frontend outside its gate

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/.github/workflows/rust-ci.yml:11`;
  `echo-agent-cli/web-frontend/package.json:6`; root `AGENTS.md:301`
- Reachability: every push/PR enters the sole `ci` job, which ends after Rust
  commands at workflow line 45 and never enters `web-frontend`.
- Expected invariant: frontend changes are checked with Prettier, tests, and a
  production build before merge.
- Observed behavior: no Node setup or frontend command exists in CI.
- Impact: type errors, failed frontend tests, formatting drift, or an invalid
  Vite production build can merge while CI remains green.
- Root cause: the workflow models only the Rust workspace even though the
  repository contains a shipped React/Tauri frontend.
- Direction: add a frontend job using the committed npm lockfile and the three
  commands required by `AGENTS.md`; do not fold all outcomes into one opaque
  command.
- Regression validation: deliberately introduce one formatting, one test, and
  one type/build failure in separate CI negative controls, then revert them.
- Validation reports: [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P2-02: CLI CI's framework dependency is a floating external input

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/.github/workflows/rust-ci.yml:15`;
  `echo-agent-cli/Cargo.toml:50`;
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:10`
- Reachability: every CI run checks out `EchoYue-lp/echo-agent` without a `ref`,
  symlinks that checkout to the exact sibling path used by both manifests, and
  compiles against whatever commit is then at the framework default branch.
- Expected invariant: rerunning CI for an unchanged CLI commit tests the same
  framework contract, or an explicitly declared compatibility lane.
- Observed behavior: the framework revision is neither pinned in the workflow
  nor represented by a commit-bearing dependency; it can change between runs.
- Impact: the same CLI commit can move from green to red without any CLI change,
  and a result cannot be reproduced later from the CLI commit and lockfile
  alone.
- Root cause: independent repositories are connected by a local path dependency
  while CI supplies that path from an unpinned checkout.
- Direction: make the intended contract explicit: pin a compatible framework
  revision for the required gate, and optionally add a separate allowed-to-fail
  or scheduled latest-main compatibility lane.
- Regression validation: rerun the pinned gate twice from the same CLI commit
  and verify the resolved framework SHA is identical and logged.
- Validation reports: [V01](../validations/B-BASE-01/V01-02.md),
  [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P2-03: Framework CI weakens the documented all-target test gate

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/.github/workflows/rust-ci.yml:35`;
  `echo-agent/.github/workflows/rust-ci.yml:60`; root `AGENTS.md:259`
- Reachability: the test matrix runs `cargo test -p <package> --lib --tests` for
  each package. The lint job compiles all targets but does not execute test
  harnesses belonging to examples or the benchmark.
- Expected invariant: CI enforces `cargo test --workspace --all-targets
  --all-features --locked`, the mandatory framework submission gate.
- Observed behavior: target execution is restricted to libraries and
  integration tests.
- Impact: a test placed in an example/bench target, or behavior only exercised
  by such a target harness, is not run by CI even though the repository gate
  promises all-target execution.
- Root cause: the package-split test optimization changed target semantics while
  the workflow comment treats Clippy compilation as equivalent coverage.
- Direction: preserve package-level parallelism if useful, but use
  `--all-targets` for each matrix package or add a separate exact gate command.
- Regression validation: add a temporary failing target-local test as a negative
  control and verify the CI test job detects it.
- Validation reports: [V03](../validations/B-BASE-01/V03-02.md),
  [V04](../validations/B-BASE-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Manifest/member inventory | yes | passed | [V01](../validations/B-BASE-01/V01-02.md) |
| V02 | Feature-to-dependency graph | yes | passed | [V02](../validations/B-BASE-01/V02-02.md) |
| V03 | Target/required-feature inventory | yes | passed | [V03](../validations/B-BASE-01/V03-02.md) |
| V04 | CI-versus-AGENTS gate comparison | yes | failed | [V04](../validations/B-BASE-01/V04-01.md) |
| V05 | Primary recount and finding-anchor acceptance | yes | passed | [V05](../validations/B-BASE-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md`: framework is a root package plus seven subcrates | current | [V01](../validations/B-BASE-01/V01-02.md) |
| Root `AGENTS.md`: CLI is a workspace and EKO does not enable SQLite | current | [V01](../validations/B-BASE-01/V01-02.md), [V02](../validations/B-BASE-01/V02-02.md) |
| Root `AGENTS.md`: CLI contains `echo-agent-eval` | stale | no directory or manifest at `b3b2e81f2b2d`; [V01](../validations/B-BASE-01/V01-02.md) |
| Root `AGENTS.md`: CI depends on the listed mandatory gates | regressed | [V04](../validations/B-BASE-01/V04-01.md) |

## Coverage And Uncertainty

No build or test command was executed; later quality tasks own those expensive
validations. Cargo's resolved third-party graph was not audited. The meaning of
facade marker features and the omissions from `full` require source-level cfg
inspection in `F-FEAT-01`. This task establishes topology, not runtime
reachability.

## Handoff

- `B-ARCH-01` may rely on the eight-package framework inventory and normalized
  path-dependency graph, but must independently inspect module/re-export
  ownership.
- `B-PATH-01` may rely on the EKO target inventory, but not infer feature parity
  from target existence.
- `B-DOC-01` should classify the stale `echo-agent-eval` overview and the CI
  gate drift.
- This report becomes stale if either manifest tree, either workflow, the
  frontend package scripts, or the reviewed commits change.
