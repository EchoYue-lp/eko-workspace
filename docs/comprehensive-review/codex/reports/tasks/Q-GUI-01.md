# Q-GUI-01: Tauri/GUI Rust matrix

> Status: needs_evidence
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework had extensive external source changes and was used only through committed blobs. CLI had an external `Cargo.lock` change, which was excluded. Only Codex Q-GUI reports were added; source, index, README and shared catalog were not changed.

## Question

Does the Tauri/GUI target compile and test under its conditional feature matrix?

## Scope

- Root CLI feature declarations, GUI-specific optional dependencies and binary targets.
- GUI and package entry-point conditional compilation.
- Root Tauri build script/config, frontend lifecycle references, capability files and bundle assets.
- Rust CI's Linux system prerequisites and feature/test matrix.
- Existing committed Tauri test inventory and current macOS host prerequisites.

## Out Of Scope

- Cargo, rustc, tests, builds, frontend commands, dynamic fixtures and network,
  all explicitly forbidden for this task.
- Desktop command/runtime behavior owned by `A-SRF-02`.
- Frontend DTO/event/error behavior owned by `A-FE-01` and later A-FE tasks.
- Frontend formatting/tests/build owned by `Q-WEB-01`.
- Framework source behavior; dirty framework file bodies and diffs were excluded.
- CI dependency pinning or general submission-gate findings owned by other Q tasks.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `TASKS.md`, `REPORTING.md`; Codex `README.md`; report templates.
- Authorized dependency reports actually read: `A-SRF-02`, `A-FE-01`.
- One unauthorized Codex report, `Q-STA-01`, was accidentally displayed for
  formatting. V00-03 excludes its entire content; no conclusion here depends on it.
- Fixed committed CLI blobs and tree inventory; dirty CLI `Cargo.lock` excluded.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework source is not involved in the Tauri target matrix and was not reassessed. |
| EKO product policy | GUI feature composition, binary selection, native prerequisites, Tauri capabilities and CI feature combinations belong to EKO. |
| Adapter boundary | The dedicated Tauri bin is a thin application entry into the shared desktop module; Q-GUI checks only whether that target is selected and validated. |
| Duplicate search | Searched the committed CLI for `gui`, `echo-agent-tauri`, `tauri_build`, Tauri config/capability paths, CI feature commands, test attributes, frontend script names and bundle assets. |
| Migration deletion | No authority moves repositories. Delete only the stale nested capability copy after confirming root capability ownership. |

## Current Path

```text
Cargo root package
  gui feature -> optional Tauri/plugins/PTY + channels
  echo-agent-tauri (required-features=gui)
    -> src-tauri/src/main.rs -> desktop entry

build.rs + CARGO_FEATURE_GUI
  -> root tauri.conf.json
  -> runner cargo --bin echo-agent-tauri + feature gui
  -> frontend dev/build lifecycle + bundle assets

Rust CI on Linux
  -> installs declared Tauri native libraries
  -> all-target/all-feature Clippy + all-feature tests
  -X-> no dedicated gui && !tui check/test steps
```

Static positive conclusions:

- Manifest, dedicated GUI bin, build script and Tauri runner agree on `gui` and
  `echo-agent-tauri`.
- Configured frontend scripts and bundle icons exist in the committed tree.
- CI declares its Linux Tauri native libraries; current macOS host exposes Xcode
  command-line tools plus `clang`, `cargo` and `npm` command paths.
- Existing Tauri command test modules are present, but were not executed.
- `A-SRF-02` retains ownership of setup/terminal/workflow behavior; `A-FE-01`
  retains ownership of DTO/event/error projection. Those findings are not repeated.

## Findings

### Q-GUI-01-P2-01: CI never validates the GUI-only feature combination

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/Cargo.toml:21`, `:39`; `echo-agent-cli/src/main.rs:72`; `echo-agent-cli/.github/workflows/rust-ci.yml:22`, `:35`
- Reachability: the root manifest defaults to `tui`; GUI packaging selects the
  `echo-agent-tauri` bin and `gui`. CI installs native libraries, then uses only
  all-feature Clippy/tests. In the package-name bin, all features choose the TUI
  branch, while GUI-only chooses the separately compiled desktop branch.
- Expected invariant: the conditionally supported `gui && !tui` product target
  has explicit compile and test gates matching the documented submission matrix.
- Observed behavior: CI contains neither
  `cargo check --no-default-features --features gui --bin echo-agent-tauri` nor
  `cargo test --no-default-features --features gui`. All-feature gates are not
  feature-isolation checks.
- Impact: a missing GUI-only import, cfg edge or test compile regression can merge
  while the all-feature CI remains green. Current pass/fail status is unknown.
- Root cause: GUI validation was folded into a broad all-feature workspace gate,
  despite materially different conditional entry paths.
- Direction: add two distinct CI steps for the exact GUI-only check and tests;
  retain the all-feature workspace gate rather than replacing it. Keep native
  dependency failures visible on the individual step.
- Regression validation: run the exact two commands as separate immutable
  validations on the CI platform and macOS packaging host.
- Validation reports: [V02](../validations/Q-GUI-01/V02-01.md), [V06](../validations/Q-GUI-01/V06-01.md), [V07](../validations/Q-GUI-01/V07-01.md)

### Q-GUI-01-P3-02: Two divergent capability files claim the same `default` identity

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence: `echo-agent-cli/build.rs:4`; `echo-agent-cli/capabilities/default.json:1`; `echo-agent-cli/src-tauri/capabilities/default.json:1`
- Reachability: the only Cargo package/build script resolves root
  `tauri.conf.json`, placing the root `capabilities/` directory at the active
  configuration boundary. A second conventional copy remains under `src-tauri/`.
- Expected invariant: each capability identifier has one committed authority.
- Observed behavior: both files declare identifier `default`, but the root copy
  includes `core:event:allow-emit` and `fs:default` while the nested copy omits
  them and uses a different schema URL.
- Impact: contributors can update or inspect the wrong capability definition,
  producing misleading review and future packaging drift. No current permission
  failure is claimed.
- Root cause: the package/config moved to the repository root without deleting
  the older conventional `src-tauri/capabilities` copy.
- Direction: confirm root generated-context ownership in the next Tauri build,
  then delete the nested duplicate; do not maintain two synchronized copies.
- Regression validation: build generated Tauri context and assert the single
  `default` capability contains the intended permission set.
- Validation reports: [V03](../validations/Q-GUI-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Fixed commits, dirty isolation and report absence | yes | passed | [report](../validations/Q-GUI-01/V00-01.md) |
| V00-02 | Incorrect standalone `src-tauri`/root package assumptions | disclosure | inconclusive, excluded | [report](../validations/Q-GUI-01/V00-02.md) |
| V00-03 | Unauthorized Q-STA report read | disclosure | inconclusive, excluded | [report](../validations/Q-GUI-01/V00-03.md) |
| V01 | GUI feature/bin/build/config topology | yes | passed | [report](../validations/Q-GUI-01/V01-01.md) |
| V02 | CI conditional feature matrix | yes | failed -> P2-01 | [report](../validations/Q-GUI-01/V02-01.md) |
| V03 | Tauri capability authority | yes | failed -> P3-02 | [report](../validations/Q-GUI-01/V03-01.md) |
| V04 | Native/system environment prerequisites | yes | passed with limits | [report](../validations/Q-GUI-01/V04-01.md) |
| V05 | Existing GUI test/dependency finding inventory | yes | passed | [report](../validations/Q-GUI-01/V05-01.md) |
| V06 | GUI bin check | yes | not_run by instruction | [report](../validations/Q-GUI-01/V06-01.md) |
| V07 | GUI-only tests | yes | not_run by instruction | [report](../validations/Q-GUI-01/V07-01.md) |
| V08 | Tauri frontend/icon reference completeness | yes | passed static | [report](../validations/Q-GUI-01/V08-01.md) |
| V99-01 | Initial integrity gate with invalid top-level Git assumption | disclosure | inconclusive | [report](../validations/Q-GUI-01/V99-01.md) |
| V99-02 | Corrected report/source integrity gate | yes | passed | [report](../validations/Q-GUI-01/V99-02.md) |
| V30 | Primary committed-source sampling | yes | passed | [report](../validations/Q-GUI-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root AGENTS requires separate GUI bin check and GUI tests after Tauri/GUI changes | current policy, unexecuted | V06 and V07 preserve both as explicit `not_run`; V02 shows CI also lacks them. |
| `A-SRF-02` desktop command/setup/terminal/workflow findings | current dependency ownership, not reassessed | V05; no Q-GUI finding repeats them. |
| `A-FE-01` GUI DTO/event/error projection findings | current dependency ownership, not reassessed | V05; no Q-GUI finding repeats them. |
| README says Tauri runner selects `echo-agent-tauri` with `gui` | current | V01 and V08 match the committed manifest/config. |

## Coverage And Uncertainty

- The principal question remains unanswered dynamically: no GUI-only bin check
  or test ran, so status is `needs_evidence` rather than complete.
- Host command presence is not an SDK/linker compatibility test. Any future
  native dependency failure must receive a new failed validation, not be skipped.
- Static ownership strongly identifies root capability configuration, but only a
  Tauri generated-context/build inspection can prove which capability files are
  consumed. P3-02 therefore has medium confidence and claims drift, not failure.
- Frontend lifecycle/build output and packaging were not executed; `Q-WEB-01`
  owns frontend gates.
- V00-03 excludes an accidentally read non-dependency report. Primary acceptance
  requires independent reconstruction from fixed source and authorized inputs.

## Handoff

- Primary should first reconstruct P2-01 from manifest cfg branches and CI, then
  sample the two capability files before accepting P3-02.
- Required future evidence is exactly V06 and V07 as new attempts. Preserve native
  environment failures verbatim if either command cannot start or link.
- Downstream `Q-E2E-01`, `S-APP-01` and `S-QA-01` may rely on static topology only
  after primary acceptance; they must continue to treat GUI compile/test as unknown.
- This report becomes stale if GUI/TUI feature topology, CI, build.rs,
  `tauri.conf.json`, capability paths or either fixed commit changes.
- Primary independently reconstructed both findings from committed manifest,
  cfg, CI, build script and capability blobs in V30. Dynamic status remains
  unknown because V06/V07 were intentionally not run.
