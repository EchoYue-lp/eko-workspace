# Q-FW-02: Framework feature, examples, and docs matrix

> Status: needs_evidence
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> Historical execution commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> Worktree state: externally dirty; current evidence uses committed `HEAD` blobs only

## Question

Do public optional capabilities compile and demonstrate their stated contracts
independently?

## Answer

Current static topology is fully inventoried, but the current-commit executable
answer is intentionally unavailable. All eight manifests are byte-identical to
the commit where `F-FEAT-01` independently compiled every 33 non-meta facade
features, 24 split-crate features, and seven split libraries without defaults.
Since then six Rust files changed, including ReAct stream/tool and testing
surfaces, so those historical passes are useful precedent but not current build
evidence. Per the user's explicit instruction, no Cargo, rustdoc, example, or
test command was rerun.

## Current Static Matrix

- 68 Rust example sources are present.
- 58 explicit `[[example]]` targets form 27 distinct required-feature groups;
  the other auto-discovered examples are not separately declared.
- Ten declared examples have no `required-features`; seven are `testing`-gated;
  SQLite is the largest domain group.
- The `full` meta-feature still omits seven flags and cannot select 17 official
  examples, owned by `F-FEAT-01-P2-03`.
- `full`/README example commands and four nonexistent inventory entries remain
  owned by `Q-DOC-01-P2-02`.
- The last executed doctest matrix found facade and core failures plus ignored
  macro/tool examples, owned by `F-API-01-P2-05`; current doctests were not run.
- Facade no-default still resolves execution default file/shell capabilities,
  owned by `F-FEAT-01-P2-01`.

## Findings

No new findings. The matrix independently confirms the current static reach of
the existing canonical findings above. Adding Q-level duplicate IDs would
inflate defect counts without a new root cause.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit, dependency, source-isolation, and historical-evidence boundary | yes | passed | [V00](../validations/Q-FW-02/V00-01.md) |
| V01 | Standalone feature matrix at current commit | yes | not_run; historical evidence classified | [V01](../validations/Q-FW-02/V01-01.md) |
| V02 | Current example target/required-feature grouping | yes | passed | [V02](../validations/Q-FW-02/V02-01.md) |
| V03 | Current example command matrix | yes | not_run | [V03](../validations/Q-FW-02/V03-01.md) |
| V04 | Current per-package doctest matrix | yes | not_run | [V04](../validations/Q-FW-02/V04-01.md) |
| V05 | Current documentation/local-link static matrix | yes | failed/current owner | [V05](../validations/Q-FW-02/V05-01.md) |
| V06 | `full` to official-example subset | yes | failed/current owner | [V06](../validations/Q-FW-02/V06-01.md) |
| V07 | Facade no-default dependency isolation | yes | failed/current owner | [V07](../validations/Q-FW-02/V07-01.md) |
| V99 | Links, headers, IDs, isolation, and status | yes | passed | [V99](../validations/Q-FW-02/V99-01.md) |

## Future Execution Groups

The 27 exact required-feature groups should each receive one independent
example command report, rather than one all-features command that hides feature
isolation. Per-package rustdoc should remain eight independent reports. The
standalone feature matrix should be rerun only after implementation begins or
the user permits dynamic quality gates; historical `F-FEAT-01` reports remain
immutable and must not be relabeled current.

## Handoff

- Keep this task `needs_evidence` until the current commit receives the 27
  grouped example commands, eight rustdoc commands, and required standalone
  feature matrix.
- Fix static topology/documentation owners first; otherwise current dynamic
  execution would mostly reconfirm known selection failures.
- Do not delete useful optional framework capabilities because EKO does not use
  them; this matrix validates the reusable framework contract.
