# X-INV-01: Repository invariant audit

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories externally dirty; all adopted evidence came from committed `HEAD` blobs and CLI `Cargo.lock` was excluded after the V00 isolation incident

## Question

Do both repositories obey Subagent-only terminology, EKO no-SQLite, one Task
authority, panic safety, UTF-8 safety, and relative Cargo path rules?

## Scope

- Committed Rust, TypeScript, configuration, manifests, CI, examples, tests,
  and maintained documentation in both repositories.
- One separately reported static search per required invariant, followed by
  reachability/category sampling of matches.
- Canonical Codex dependencies `B-BASE-01`, `F-CORE-01`, `F-CMP-01`,
  `F-EVO-01`, `F-TSK-01`, `A-PROJ-01`, `X-BND-01`, `X-TSK-01`, and `Q-FW-01`.

## Out Of Scope

- Source fixes or changes to CI.
- Cargo, rustc, Clippy, tests, builds, dynamic fixtures, frontend commands, and
  network access, prohibited for this review phase.
- Treating framework SQLite implementations as dead merely because EKO does
  not enable them.
- Reopening already owned defects under duplicate finding IDs.

## Inputs And Isolation

The root `AGENTS.md`, exact `TASKS.md` card, `REPORTING.md`, Codex README, and
the listed completed Codex dependency reports were read. The framework live
worktree contains extensive external changes and CLI has an external
`Cargo.lock` change, so every adopted source result uses `git grep HEAD` or
`git show HEAD:path`. An initial inventory command explicitly named the dirty
CLI lockfile and exposed dependency names; V00 records the full isolation
deviation. No fact from that output was adopted, and the no-SQLite conclusion
was rebuilt from committed manifests/source with `Cargo.lock` excluded.

## Invariant Matrix

| Invariant | Static verdict | Match classification | Canonical owner |
|---|---|---|---|
| Subagent-only terminology | passed | zero `worker/Worker` word matches across maintained Rust/TS/config/docs at both pinned commits | positive conclusion here |
| EKO does not enable SQLite | passed | zero CLI manifest/source `sqlite`, `SqliteStore`, `SqliteConversationStore`, `rusqlite`, or `sqlx` matches; framework optional SQLite retained | positive conclusion here |
| One Task CRUD/graph authority | failed | deprecated tool names have zero production matches, but public `ManagedTask/TaskManager/TaskStore` still own a second graph/state/store authority | `F-TSK-01-P2-01`, `X-BND-01-P1-01`, `X-TSK-01-P2-05` |
| No panic-capable APIs | failed | raw matches are dominated by inline tests, but sampled public/production edge paths still panic or overflow | `F-CORE-01-P2-06/P3-07`, `F-CMP-01-P1-07`, `F-EVO-01-P1-04`, other atomic owners |
| UTF-8-safe string handling | failed | vector/byte/proven-boundary slices and tests are allowed classifications; sampled human-text byte slices remain reachable | `F-CMP-01-P1-07`, `F-EVO-01-P1-04`, `A-PROJ-01-P2-08` |
| Relative Cargo/worktree paths | passed | zero absolute/tilde/worktree Cargo path dependencies; required relative framework links and `.worktrees/` ignore exist | positive conclusion here |

## Findings

No new findings. Failed invariants are real, but each sampled production defect
already has a canonical atomic or cross-contract owner listed above. Creating
new IDs here would double-count the same root causes. This audit supplies a
repository-wide regression matrix and positive evidence for the three passing
hard constraints.

## Important Classifications

- `worker/Worker`: zero maintained-source matches, so no third-party wire-name
  exception was needed at these commits.
- SQLite names in CLI historical prose describe framework options and explicitly
  say EKO does not enable them; they are documentation, not a linked dependency.
- `panic!`, `unwrap`, and `expect` inside `#[cfg(test)]` modules are test-only,
  while Tauri macro-generated `unreachable!` is an external macro boundary.
  Public constructors/accounting and production human-input slicing remain
  violations when the atomic report proves reachability.
- Most slice matches are vector ranges, byte-protocol framing, delimiter indices
  returned by `find`, or explicit `floor_char_boundary`; these are not UTF-8
  truncation violations. Short opaque IDs and indices derived from a different
  string/byte representation are violations.
- `.worktrees/` paths in Git-isolation docs and a Unicode test literal are
  intentional examples. Absolute paths in a historical implementation plan are
  stale documentation portability issues, not Cargo dependency violations.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commits, dirty-source isolation, and accidental lockfile-read disclosure | yes | inconclusive/corrected boundary | [V00](../validations/X-INV-01/V00-01.md) |
| V01 | Subagent-only terminology | yes | passed | [V01](../validations/X-INV-01/V01-01.md) |
| V02 | EKO no-SQLite with framework option preserved | yes | passed | [V02](../validations/X-INV-01/V02-01.md) |
| V03 | One Task CRUD/graph authority | yes | failed/current owners | [V03](../validations/X-INV-01/V03-01.md) |
| V04 | Panic-capable API classification | yes | failed/current owners | [V04](../validations/X-INV-01/V04-01.md) |
| V05 | UTF-8 slicing classification | yes | failed/current owners | [V05](../validations/X-INV-01/V05-01.md) |
| V06 | Relative Cargo/worktree paths | yes | passed | [V06](../validations/X-INV-01/V06-01.md) |
| V07 | Canonical ownership and duplicate-finding gate | yes | passed | [V07](../validations/X-INV-01/V07-01.md) |
| V08 | Dynamic Clippy/build/test confirmation | future | not_run | [V08](../validations/X-INV-01/V08-01.md) |
| V99 | Exact links, headers, IDs, isolation, and status | yes | passed | [V99](../validations/X-INV-01/V99-01.md) |

## Historical Claim Status

| Claim | Classification | Current evidence |
|---|---|---|
| Subagent terminology is unified | current | [V01](../validations/X-INV-01/V01-01.md) |
| EKO does not enable SQLite; framework may offer it | current | [V02](../validations/X-INV-01/V02-01.md) |
| Framework still has a second Task authority | current | [V03](../validations/X-INV-01/V03-01.md) |
| Panic/UTF-8 edge defects remain | current, atomic ownership preserved | [V04](../validations/X-INV-01/V04-01.md), [V05](../validations/X-INV-01/V05-01.md) |
| Worktree dependency paths are relative and ignore is present | current | [V06](../validations/X-INV-01/V06-01.md) |

## Coverage And Uncertainty

Regex searches are deliberately over-inclusive and cannot replace the required
Clippy submission gate. Each result therefore distinguishes raw candidates from
sampled reachable violations. No dynamic command was run. Q-phase tasks own
future mechanical execution; static conclusions at the pinned commits are
complete.

## Handoff

- Synthesis should count no new defect IDs from this report and retain the
  linked canonical owners.
- Add lightweight repository checks for forbidden terminology, CLI SQLite
  feature/dependency activation, forbidden task-tool names, absolute Cargo path
  dependencies, and known unsafe string-slice patterns. Keep panic Clippy as the
  authoritative mechanical panic gate.
- This report becomes stale when Task authorities, CLI manifests, CI policy, or
  any linked panic/UTF-8 owner changes.
