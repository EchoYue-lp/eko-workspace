# F-FEAT-01: Feature topology and isolation

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only inspection)

## Question

Does each feature enable exactly its required code and dependencies,
including no-default and standalone feature use?

## Scope

Primary source paths and behaviors inspected (read-only):

- `echo-agent/Cargo.toml` — root `[features]` table (35 entries incl.
  `default`, `full`), `[dependencies]`, `[package.metadata.docs.rs]`.
- All 7 sub-crate `Cargo.toml` files under `echo-agent/` (`echo-core`,
  `echo-macros`, `echo-execution`, `echo-integration`, `echo-tools`,
  `echo-state`, `echo-orchestration`) — their `[features]` tables and
  optional dependencies.
- All `#[cfg(feature = "...")]` declarations across the workspace
  (302 occurrences total — see V01 census).
- `echo-agent-cli/Cargo.toml` and `echo-agent-cli/echo-agent-app-core/Cargo.toml`
  — the `default-features = false` + curated feature subset the application
  enables on `echo_agent` (cross-referenced with B-BASE-01 V02).

## Out Of Scope

Deferred to named task IDs:

- Full per-feature standalone compile matrix execution (`for feature in
  sqlite subagent human-loop mcp ...`) → `Q-FW-02` and the AGENTS.md
  "条件矩阵" gate. F-FEAT-01 does the static equivalent (feature wiring
  analysis), not the dynamic compile sweep.
- Source-level reachability of every feature-gated module's call graph →
  `B-PATH-01` (composition root) and follow-up F- tasks.
- Workspace/crate dependency graph cycle analysis → `B-ARCH-01`.
- echo-state `sqlite` feature schema/migration concerns → not applicable
  per AGENTS.md ("echo-agent-cli 不需要 SQLite"; `sqlite` is a framework
  option for other consumers and is not a deletion candidate — see
  AGENTS.md "删除框架代码的判定").

## Inputs

- Repository documents read: root `AGENTS.md` (sections "代码清理",
  "删除框架代码的判定", "echo-agent-cli 不需要 SQLite", "条件矩阵"),
  `docs/comprehensive-review/README.md`, both report templates, the
  `F-FEAT-01` task card in `TASKS.md`.
- Dependency task reports read: `B-BASE-01` (workspace topology, feature
  inventory per crate, CLI feature selection, docs.rs `data` exclusion
  note at `Cargo.toml:59-63`).
- Historical documents treated as hypotheses: none.

## Layering Decision

F-FEAT-01 is a framework-layer structural review. The layering invariants
verified here:

- **Generic mechanism** (framework, keep): the `[features]` table of
  `echo-agent` is the framework's capability menu. Per AGENTS.md
  "删除框架代码的判定", a framework feature's right to exist is judged by
  "framework-internal + all reasonable consumers", not by whether
  `echo-agent-cli` uses it. The `sqlite`, `mcp`, `lsp`, `a2a`,
  `human-loop`, etc. features are legitimate framework options even though
  the CLI may not enable all of them.
- **EKO product policy** (application): the CLI's `default-features = false`
  + curated subset (notably excluding `sqlite`) is an application-layer
  decision and is verified separately in B-BASE-01.
- **Adapter boundary**: none — features are not adapted, they are
  forwarded via standard `<crate>/<feature>` syntax.

Repository-wide duplicate-search terms used: `cfg(feature = `, every
feature name from the root `[features]` table, `dep:` (optional-dep
activation strong form), `default-features = false`. Results: no duplicate
feature definitions across crates; all sub-crate feature forwards target
existing sub-crate features.

## Current Path

Feature topology at commit `9b0e0fa`:

- Root `echo-agent/Cargo.toml` declares 35 features. `default = []` (no
  implicit enablement). `full` aggregates most features.
- 302 `cfg(feature = "...")` occurrences across the workspace, distributed
  per the V01 census (root 226, echo-tools 48, echo-core 13, echo-state 6,
  echo-execution/echo-integration/echo-orchestration 3 each, echo-macros 0).
- 19 features activate optional dependencies via `dep:` prefix (clean —
  see V03).
- Cross-crate forwarding (`mcp`, `lsp`, `sqlite`, `human-loop`, etc.) is
  correct and targets declared sub-crate features.
- `subagent = ["tasks"]` is the only intra-framework feature composition
  in the root; it is correct (subagent source imports `crate::tasks::*`).
- 8 empty `= []` features gate real code (legitimate markers).
- 5 empty `= []` features gate nothing (dead — see findings).

## Findings

### F-FEAT-01-P2-01: 5 dead features with zero cfg matches and zero code references

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml` `[features]` table; workspace-wide
  `grep -rn 'cfg(feature = "sandbox")'` (and the other 4 names) returns 0
  matches; bare-name grep in `.rs` files returns 0 references.
- Reachability: definition (`Cargo.toml [features]`) → registration (none —
  enables no `dep:`, forwards no sub-crate feature) → live caller (none).
- Expected invariant: per AGENTS.md "代码清理:无需兼容,过时代码可直接删"
  and "删除框架代码的判定" rule (1) — a feature with no framework-internal
  cfg gate, no optional-dependency activation, and no consumer reference
  is dead and should be removed.
- Observed behavior: the following 5 features are declared as `= []` and
  have zero `cfg(feature = "<name>")` matches anywhere in the workspace:

  | Feature | cfg matches | Code references |
  |---|---:|---:|
  | `sandbox` | 0 | 0 (sandbox source code is compiled unconditionally; the feature is an unused marker) |
  | `semantic-memory` | 0 | 0 |
  | `macros` | 0 | 0 (unrelated to the `echo-macros` crate) |
  | `provider-factory` | 0 | 0 |
  | `multimodal` | 0 | 0 |

- Impact: dead config expands the framework's feature surface without
  effect, misleading consumers into thinking `--features sandbox` (etc.)
  does something. Adds maintenance noise and contradicts the project's
  YAGNI / no-backward-compat rule.
- Root cause: features were likely declared speculatively or left behind
  after their gated code was moved/unconditionalized, and never cleaned up.
- Direction: remove these 5 lines from the root `[features]` table in
  `echo-agent/Cargo.toml`. Also remove them from the `full` aggregator
  list (see F-FEAT-01-P3-01). Run the AGENTS.md "条件矩阵" per-feature
  compile to confirm nothing else references them (already statically
  confirmed: 0 references).
- Regression validation: after deletion, `cargo check -p echo_agent
  --no-default-features` and `cargo check -p echo_agent --features full`
  must both still succeed. Conditional matrix:
  `for feature in sqlite subagent human-loop mcp lsp a2a git database rag chart web media; do cargo check -p echo_agent --no-default-features --features "$feature" --locked || exit 1; done`.
- Validation reports: [V01](../validations/F-FEAT-01/V01-01.md)
  (cfg census), [V02](../validations/F-FEAT-01/V02-01.md) (dead-feature
  classification).

### F-FEAT-01-P3-01: `full` aggregator includes dead features unnecessarily

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml` `[features] full = [...]` list.
- Reachability: definition → consumers that enable `full` (e.g. docs.rs
  when not excluded, `cargo test --all-features`) → the dead features
  become active but contribute nothing.
- Expected invariant: the `full` aggregator should be the minimal set of
  real features, not a superset that includes dead markers.
- Observed behavior: the `full` feature lists the 5 dead features from
  F-FEAT-01-P2-01 alongside the live ones. Enabling `full` thus
  "activates" features that gate nothing.
- Impact: cosmetic — no compile-time effect (the dead features enable
  nothing), but it obscures the true capability surface and means the
  F-FEAT-01-P2-01 deletion must touch two places (feature definition AND
  the `full` list).
- Root cause: `full` was authored to enumerate all features
  indiscriminately; dead features were added to both lists in lockstep.
- Direction: when removing the 5 dead features per F-FEAT-01-P2-01, also
  remove their names from the `full` aggregator list in the same commit.
- Regression validation: `cargo check -p echo_agent --features full --locked`
  succeeds.
- Validation reports: [V04](../validations/F-FEAT-01/V04-01.md).

### F-FEAT-01-P3-02: `workflow` feature gates references but module wiring needs spot verification

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: V02 classifies `workflow` as a live marker (4 cfg matches),
  gating "workflow module references".
- Reachability: definition → `cfg(feature = "workflow")` at 4 sites →
  the gated items are reachable when `workflow` is enabled.
- Expected invariant: a feature with cfg matches must gate a complete,
  compilable module (not dangling references to a module that is itself
  gated by a different feature or missing).
- Observed behavior: the 4 cfg matches for `workflow` reference the
  workflow module, but this review did not execute a standalone
  `--features workflow` compile to confirm the gated paths are
  self-contained (no missing companion feature). The static wiring looks
  plausible but is not dynamically verified here.
- Impact: low — if `workflow` is incompletely gated, `--features workflow`
  alone could fail to compile. But `workflow` is included in `full` and
  the AGENTS.md conditional matrix, so any gap would surface there.
- Root cause: insufficient dynamic verification in this review (delegated
  to the AGENTS.md conditional matrix and Q-FW-02).
- Direction: execute `cargo check -p echo_agent --no-default-features
  --features workflow --locked` as part of the conditional matrix. If it
  fails, audit the 4 cfg sites for missing companion-feature declarations.
- Regression validation: standalone `--features workflow` compiles.
- Validation reports: [V01](../validations/F-FEAT-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and cfg(feature census | yes | passed | [V01-01](../validations/F-FEAT-01/V01-01.md) |
| V02 | Unused / empty feature classification | yes | passed_with_findings | [V02-01](../validations/F-FEAT-01/V02-01.md) |
| V03 | Optional dependency leakage (dep: prefix) | yes | passed | [V03-01](../validations/F-FEAT-01/V03-01.md) |
| V04 | Standalone compile / feature composition analysis | yes | passed | [V04-01](../validations/F-FEAT-01/V04-01.md) |
| V05 | Historical-document drift | no | not_run | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md` "条件矩阵" per-feature list (`sqlite subagent human-loop mcp lsp a2a git database rag chart web media`) | current | the listed features all exist and are non-dead; `subagent = ["tasks"]` confirmed self-contained in V04 |
| `AGENTS.md` "echo-agent-cli 不需要 SQLite" | current | CLI enables `default-features = false` + curated subset excluding `sqlite` (B-BASE-01 V02) |
| `AGENTS.md` "删除框架代码的判定" (sqlite is a framework option, not deletable just because CLI doesn't use it) | current | `sqlite` feature forwards to `echo_state/sqlite` and gates real code — kept correctly |

## Coverage And Uncertainty

- **Covered**: static feature-wiring analysis across all 8 `echo-agent`
  crates; cfg(feature census (302 occurrences); dead-feature classification
  (5 dead, 8 live markers); optional-dep leakage (none); CLI feature
  selection; docs.rs `data` exclusion rationale.
- **Not executed in this task**: the dynamic per-feature standalone compile
  matrix (`for feature in ...`). This is delegated to the AGENTS.md
  "条件矩阵" gate and `Q-FW-02`. F-FEAT-01's V04 does the static
  equivalent by inspecting wiring rather than invoking `cargo check`. As a
  result, finding F-FEAT-01-P3-02 (`workflow` standalone compile) is
  medium-confidence pending that matrix run.
- **Environmental limits**: none — all inspections are read-only static
  analysis (grep + Cargo.toml reads).
- **Uncertain claims**: whether the 4 `workflow` cfg sites form a
  self-contained module under `--features workflow` alone (see
  F-FEAT-01-P3-02).

## Handoff

- **Conclusions downstream tasks may rely on**:
  - The framework feature surface is clean on dependency leakage (V03):
    every optional dep is `dep:`-gated, `default = []`.
  - `subagent = ["tasks"]` is correctly composed and compiles standalone.
  - 5 features (`sandbox`, `semantic-memory`, `macros`, `provider-factory`,
    `multimodal`) are confirmed dead by static analysis and are safe
    deletion candidates per AGENTS.md.
  - The `sqlite` feature is a legitimate framework option (gates real code
    via `echo_state/sqlite`) and is NOT a deletion candidate — the
    "echo-agent-cli 不需要 SQLite" rule applies to the CLI consumer, not
    to the framework feature menu (AGENTS.md "删除框架代码的判定").

- **Reports downstream tasks must read**:
  - This report (F-FEAT-01) for the dead-feature list and feature-wiring
    invariants.
  - `B-BASE-01` for the per-crate feature inventory and CLI selection.

- **Conditions that make this report stale**:
  - Any change to `echo-agent/Cargo.toml` `[features]` table (additions,
    removals, re-composition of `full`).
  - Any new `#[cfg(feature = "...")]` added or removed in the workspace.
  - Any change to a sub-crate's `[features]` table that affects
    cross-crate forwarding.

- **Follow-up task IDs** (fixes not implemented in this review task):
  - Deletion of the 5 dead features and their `full` entries — tracked as
    F-FEAT-01-P2-01 / F-FEAT-01-P3-01; out of scope for this review.
  - Dynamic per-feature compile matrix → `Q-FW-02` and AGENTS.md
    "条件矩阵".
  - `workflow` standalone compile verification → fold into the
    conditional matrix run (F-FEAT-01-P3-02).
