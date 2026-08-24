# Q-FW-01: Framework submission gate

> Status: needs_evidence
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework externally dirty and inspected only through
> committed HEAD; CLI external `Cargo.lock` dirty excluded

## Question

Does current `echo-agent` pass its mandatory submission gate?

## Scope

- The five exact framework gate commands: fmt check, all-target/all-feature
  Clippy, panic-safety Clippy, all-target/all-feature tests, and no-default
  workspace-library check.
- Committed root workspace/feature/target metadata, Cargo.lock, toolchain and
  `.github/workflows/rust-ci.yml` needed to determine automated gate coverage.
- Contributor-facing `CONTRIBUTING.md` and README gate commands.
- Separation of actual command outcomes from static configuration evidence.

## Out Of Scope

- Cargo/rustc/test/build/fixture/network execution, prohibited for this static
  review. No current dirty framework source body or diff was read.
- Per-feature/example/docs matrix owned by Q-FW-02.
- Dependency/advisory/static pattern review owned by Q-STA-01.
- Fault injection owned by Q-FLT tasks.
- Re-reporting the 38 framework atomic reviews' code findings. They are
  prerequisites for interpreting a future failure, not evidence that a gate ran.
- Source, CI, documentation or shared-index changes.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact Q-FW-01 task card;
  Codex README and report templates.
- Codex framework synthesis metadata only to confirm atomic framework reviews
  exist and Q-FW evidence was outstanding. No other reviewer directory was read.
- Framework committed blobs at pinned HEAD via `git show`, `git grep` and
  `git ls-tree`; external worktree modifications were excluded.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Submission-gate commands and CI coverage belong to the reusable framework repository and must validate all workspace members/targets independent of EKO. |
| EKO product policy | None. CLI state and product policy do not determine whether framework HEAD passes. |
| Adapter boundary | CI is the automation adapter for the repository gate; contributor docs are human-facing adapters. Both must reproduce the authoritative gate without weakening target scope. |
| Duplicate search | Compared AGENTS.md, Cargo.toml, Cargo.lock, rust-toolchain, rust-ci workflow, CONTRIBUTING and README for fmt/clippy/test/check flags, workspace packages, targets, features, locks and deny lints. |
| Migration deletion | Consolidate the existing three command descriptions after alignment; do not add a fourth gate script unless it becomes the one generated/authoritative owner. |

## Current Path

```text
authoritative local gate (AGENTS.md)
  fmt --all -- --check
  clippy workspace/all-targets/all-features/locked/-D warnings
  clippy workspace/lib/bins/all-features/locked/panic deny set
  test workspace/all-targets/all-features/locked
  check workspace/lib/no-default/locked

committed CI
  lint job -> exact fmt + both exact Clippy commands
  test matrix -> all 8 packages, but only --lib --tests
  minimal job -> exact no-default check

CONTRIBUTING + README
  independent, weaker command subsets

this review
  V02-V06 exact commands -> not_run by explicit static-only policy
  therefore no current pass/fail gate verdict
```

Static positives: root plus all seven member crates appear in workspace
default-members and CI test matrix; lockfile v4 is committed; the declared stable
toolchain includes rustfmt and Clippy; CI includes `--locked` for resolution-
sensitive commands; default features are empty.

## Findings

### Q-FW-01-P1-01: CI does not execute the mandatory all-target test gate

- Priority: P1; confidence: high; layer: framework.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.github/workflows/rust-ci.yml:35`,
  `:60`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/Cargo.toml:1`,
  `:163`, `:378`.
- Reachability: every push to main/master and pull request runs this committed
  workflow. The package matrix covers all eight crates, and lint compiles all
  targets, but the only test execution is per package `--lib --tests`.
- Expected invariant: CI executes the same mandatory test command as local
  submission policy, including applicable example/binary/benchmark target
  harnesses under all features.
- Observed behavior: CI replaces `cargo test --workspace --all-targets
  --all-features --locked` with eight `cargo test -p ... --lib --tests
  --all-features --locked` commands. The committed repository has 68 Rust
  example sources, 55 explicit examples and one benchmark; Clippy compilation is not test
  execution.
- Impact: a PR can be CI-green while failing the mandatory local all-target test
  gate, so the repository has no automated evidence for the exact submission
  contract and contributors receive contradictory pass/fail signals.
- Root cause: target coverage was optimized independently in CI without changing
  the authoritative gate or preserving equivalent execution semantics.
- Direction: execute the exact mandatory test command in CI, or formally replace
  the authoritative command only after proving the reduced matrix is equivalent.
  Delete the weaker parallel command when converged.
- Regression validation: a deliberately failing example/bench harness must fail
  CI; verify all eight packages and all target kinds under all features/lockfile.
- Validation reports: [V01](../validations/Q-FW-01/V01-01.md),
  [V07](../validations/Q-FW-01/V07-02.md)

### Q-FW-01-P2-02: README and CONTRIBUTING publish two weaker submission gates

- Priority: P2; confidence: high; layer: framework.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/CONTRIBUTING.md:13`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/README.md:1199`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.github/workflows/rust-ci.yml:25`.
- Reachability: README points contributors to CONTRIBUTING and both explicitly
  instruct what to run before PR submission.
- Expected invariant: a contributor following either maintained document runs
  the same mandatory gate and cannot mistake a weaker subset for acceptance.
- Observed behavior: CONTRIBUTING omits lockfile, warning deny, panic-safety,
  all-target/all-feature tests and no-default checks. README omits still more
  flags. CI is a third, partially different definition.
- Impact: contributors can spend review/CI cycles after following documented
  instructions that do not satisfy repository policy; future gate changes must
  be synchronized manually across three authorities.
- Root cause: submission commands are duplicated as prose instead of linked or
  generated from one maintained contract.
- Direction: after resolving P1-01, make one repository-owned gate definition
  authoritative and have README/CONTRIBUTING link to or generate from it; delete
  duplicated weaker command blocks.
- Regression validation: mechanically compare documented/CI commands and fail
  when required flags/steps diverge.
- Validation reports: [V01](../validations/Q-FW-01/V01-01.md),
  [V09](../validations/Q-FW-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Authoritative/CI/document command comparison | yes | failed | [V01-01](../validations/Q-FW-01/V01-01.md) |
| V02 | `cargo fmt --all -- --check` | yes | not_run | [V02-01](../validations/Q-FW-01/V02-01.md) |
| V03 | all-target/all-feature Clippy | yes | not_run | [V03-01](../validations/Q-FW-01/V03-01.md) |
| V04 | panic-safety Clippy | yes | not_run | [V04-01](../validations/Q-FW-01/V04-01.md) |
| V05 | all-target/all-feature tests | yes | not_run | [V05-01](../validations/Q-FW-01/V05-01.md) |
| V06 | no-default workspace library check | yes | not_run | [V06-01](../validations/Q-FW-01/V06-01.md) |
| V07 | CI package and target-execution coverage | yes | failed | [V07-02](../validations/Q-FW-01/V07-02.md) |
| V08 | Manifest/toolchain/lock/CI plumbing | yes | passed | [V08-01](../validations/Q-FW-01/V08-01.md) |
| V09 | Contributor command drift | yes | failed | [V09-01](../validations/Q-FW-01/V09-01.md) |
| V10 | Evidence-chain and source-isolation gate | yes | passed | [V10-02](../validations/Q-FW-01/V10-02.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root AGENTS.md framework submission gate is mandatory | current | Exact five commands are the task's expected gate and separately preserved in V02-V06. |
| CI comment: lint already compiles examples/benches | current but insufficient for test equivalence | V07 confirms all-target Clippy compilation while test execution remains `--lib --tests`. |
| CONTRIBUTING/README checks are sufficient before PR | stale | V09 shows both are strict subsets of the authoritative gate. |
| Framework atomic review is complete enough to interpret failures | current | Codex synthesis metadata records all 38 F tasks complete; no dynamic failure exists yet to interpret. |

## Coverage And Uncertainty

- The principal question is unanswered: no required command ran. The task must
  remain `needs_evidence`, even though static CI/document findings are conclusive.
- No external CI API/network was consulted; the workflow's existence does not
  establish that pinned HEAD had a successful run.
- Framework source is externally dirty. All inspected evidence came from pinned
  committed blobs; future commands must use an isolated checkout or wait for an
  explicitly accepted clean state.
- Static panic-API search was used only to understand why Clippy execution cannot
  be inferred; no match was reported as a gate failure.
- Q-FW-02 must separately validate independent features, examples and docs; this
  task does not claim its coverage.

## Handoff

- Q-FW-01 currently proves no submission-gate pass. When execution is authorized,
  create V02-02 through V06-02, one immutable attempt per exact command.
- Fix/roadmap work should first make CI execute one authoritative gate, then
  delete weaker README/CONTRIBUTING command copies or generate/link them.
- Read V07 before changing CI test sharding; compile coverage is not execution
  equivalence.
- This report becomes stale when gate policy, Cargo workspace/targets,
  rust-ci.yml, CONTRIBUTING/README commands or pinned framework commit changes.
- Primary reviewer must independently verify exact commands and target counts;
  status remains `needs_evidence` regardless of static-report acceptance.
