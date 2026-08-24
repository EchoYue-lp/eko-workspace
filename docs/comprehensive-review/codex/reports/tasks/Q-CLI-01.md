# Q-CLI-01: EKO Rust submission gate

> Status: needs_evidence
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework external dirt inspected only through committed blobs;
> CLI external dirty Cargo.lock excluded; no source body/diff from live dirt used

## Question

Does the current `echo-agent-cli` Rust workspace pass its mandatory gate
without enabling SQLite?

## Scope

- Five exact mandatory EKO Rust commands: fmt check, all-target/all-feature
  Clippy, panic-safety Clippy, all-feature tests, and app-core no-default check.
- Static command alignment in committed CI.
- Both committed CLI manifests and committed lockfile for SQLite absence; fixed
  framework manifests only to classify SQLite as an optional reusable ability.
- Strict separation between static configuration facts and unexecuted gate
  outcomes.

## Out Of Scope

- Running Cargo, rustc, rustfmt, Clippy, tests, builds, frontend commands,
  dynamic fixtures or network checks.
- GUI conditional matrix (`Q-GUI-01`) and frontend gate (`Q-WEB-01`).
- Re-reporting the floating CI framework revision, frontend omission,
  dependency/advisory or framework gate findings owned by B-BASE/Q-DEP/Q-FW.
- Source, manifest, lock, CI, README or shared-index modifications.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact Q-CLI-01 card in
  `TASKS.md`; Codex `README.md`; report templates.
- Application atomic reviews were considered complete enough per catalog; no
  runtime failure existed to interpret because no gate command ran.
- Codex reports [B-BASE-01](B-BASE-01.md), [Q-FW-01](Q-FW-01.md) and
  [Q-DEP-01](Q-DEP-01.md), used only for ownership/deduplication and the accepted
  framework-vs-EKO SQLite boundary.
- Committed blobs at the two fixed revisions. No other reviewer was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework SQLite/File/InMemory implementations and optional feature topology remain valid reusable `echo-agent` capabilities. |
| EKO product policy | EKO persistence is file/in-memory; both CLI packages must continue selecting framework features explicitly without SQLite. |
| Adapter boundary | Relative framework dependencies and the CI sibling checkout provide the build adapter. They must select one reproducible framework revision without silently broadening features. |
| Duplicate search | Compared all committed CLI Cargo.toml files, committed lock package names, framework sqlite feature forwarding and the exact CI/policy commands. |
| Migration deletion | No framework SQLite code should be deleted. If EKO ever gains an accidental SQLite dependency, delete that CLI selection/dependency and its CLI-only use rather than weakening the product boundary. |

## Current Path

```text
CLI root
  echo-agent default-features=false
  features=mcp,lsp,human-loop,subagent,tasks

app-core
  echo-agent default-features=false
  explicit application features, no sqlite/full

committed CLI Cargo.lock
  no rusqlite/libsqlite3-sys/sqlx/sqlite package

framework
  default=[]
  optional sqlite -> echo_state/sqlite + optional rusqlite
  valid for non-EKO consumers; retained

mandatory gate
  V02 fmt                -> not_run
  V03 all-feature Clippy -> not_run
  V04 panic Clippy       -> not_run
  V05 all-feature tests  -> not_run
  V06 app-core minimal   -> not_run
```

The committed CLI workflow text contains all five exact commands. This is only
configuration evidence: its sibling framework checkout is unpinned under
B-BASE-01-P2-02 and no current-pair outcome was executed here.

## Findings

No new findings.

Static review found no new command mismatch or SQLite leakage. Existing
canonical findings remain:

- `B-BASE-01-P2-02`: CLI CI supplies the sibling framework from an unpinned
  default branch, so a run is not reproducible for this fixed pair.
- `B-BASE-01-P2-01`: frontend is outside CI, but that is not part of this Rust
  task and belongs to Q-WEB follow-up.
- `Q-FW-01` owns framework-only submission-gate drift; `Q-DEP-01` owns
  dependency policy. Neither is re-numbered here.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Fixed commits and dirty-source isolation | yes | passed | [V00](../validations/Q-CLI-01/V00-01.md) |
| V01 | Committed manifest/lock SQLite absence | yes | passed | [V01](../validations/Q-CLI-01/V01-01.md) |
| V02 | `cargo fmt --all -- --check` | yes | not_run | [V02](../validations/Q-CLI-01/V02-01.md) |
| V03 | all-target/all-feature Clippy | yes | not_run | [V03](../validations/Q-CLI-01/V03-01.md) |
| V04 | panic-safety Clippy | yes | not_run | [V04](../validations/Q-CLI-01/V04-01.md) |
| V05 | all-feature workspace tests | yes | not_run | [V05](../validations/Q-CLI-01/V05-01.md) |
| V06 | app-core no-default check | yes | not_run | [V06](../validations/Q-CLI-01/V06-01.md) |
| V07 | CI/policy command alignment and finding deduplication | yes | passed | [V07](../validations/Q-CLI-01/V07-01.md) |
| V99 | Report/link/executor/source-boundary integrity | yes | passed | [V99](../validations/Q-CLI-01/V99-01.md) |
| V30 | Primary static-boundary acceptance | yes | passed; executable gate still missing | [V30](../validations/Q-CLI-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: EKO does not need/enable SQLite | current | [V01](../validations/Q-CLI-01/V01-01.md) |
| AGENTS.md: framework may retain optional SQLite for other consumers | current | [V01](../validations/Q-CLI-01/V01-01.md) |
| B-BASE-01: neither CLI package selects SQLite | current at identical CLI commit | [V01](../validations/Q-CLI-01/V01-01.md) |
| Committed CLI CI represents this exact repository pair | stale/overbroad | Its command set matches, but sibling checkout remains floating under B-BASE-01-P2-02; [V07](../validations/Q-CLI-01/V07-01.md) |

## Coverage And Uncertainty

- The principal gate question remains unanswered: all five executable commands
  are `not_run`, so the task correctly remains `needs_evidence`.
- Static evidence conclusively shows committed EKO feature selection and lock
  resolution omit SQLite. It does not prove the no-default or all-feature
  command type-checks.
- The live modified CLI Cargo.lock was never read or adopted. Future commands
  must use the committed lock in an isolated checkout and a pinned framework
  checkout at the reviewed SHA.
- CI command presence does not prove GitHub executed or passed this commit.
- No runtime finding was created from source patterns or previous atomic review.

## Handoff

- When executable work is authorized, create V02-02 through V06-02, one exact
  command per immutable attempt. Do not overwrite these `not_run` records.
- Pin and log framework SHA before accepting CI evidence for a CLI commit.
- Preserve the EKO no-SQLite invariant; preserve framework optional SQLite.
- This report becomes stale if CLI manifests/lock/workflow, framework sqlite
  features, repository gate policy or either reviewed commit changes.

## Primary Acceptance

Primary independently accepts V01 and V07 only: committed EKO dependency
selection/lock omit SQLite and the five CI command texts match repository policy.
V02-V06 remain unexecuted, so task status correctly remains `needs_evidence`.
