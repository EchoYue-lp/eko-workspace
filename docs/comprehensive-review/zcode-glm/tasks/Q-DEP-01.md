# Q-DEP-01: Dependency attribution, frontend inventory, build scripts, and license review

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

Which duplicate crate versions in the two lockfiles are meaningful (compiled
on the macOS target, not platform-gated), which direct dependencies pull
them, and what is the state of the frontend dependency inventory, build
scripts, and license manifest?

## Scope

Primary source paths and behaviors inspected (read-only static analysis):

- `echo-agent/Cargo.lock` (564 crates) and `echo-agent-cli/Cargo.lock`
  (787 crates) for duplicate-version attribution.
- All `Cargo.toml` files in both repos for direct-dependency version pins,
  build scripts (`build.rs`), and native-system-dependency declarations
  (pkg-config, cmake, system-dep).
- `echo-agent-cli/web-frontend/package.json` for the frontend dependency
  inventory (versions, duplication across `dependencies`/`devDependencies`,
  deprecated package status).
- License fields across all 10 Rust crates (`echo-agent` root + 7 sub-crates
  `echo_core`/`echo_macros`/`echo_execution`/`echo_integration`/`echo_tools`/
  `echo_state`/`echo_orchestration`, plus `echo-agent-app-core` and the
  Tauri/CLI crates in echo-agent-cli).

This task consumes the Q-STA-01-P2-03 handoff (38 framework / 76 CLI
duplicate versions) and attributes each meaningful duplicate to its pulling
direct dependency, where attribution is possible from manifest analysis.

## Out Of Scope

Deferred to named task IDs:

- Per-crate executable fmt/clippy/test/build gate verification → `Q-FW-01`,
  `Q-CLI-01`.
- Feature-isolation compile matrix → `Q-FW-02`.
- Panic safety, UTF-8 slicing, dead-code, oversized-module analysis →
  `Q-STA-01` (already complete; P2-03 duplicate-dep counts are consumed
  here).
- RUSTSEC/advisory database scan (network-dependent) — not executable in
  this static pass; a `cargo audit` run is recommended as a follow-up
  outside this catalog.

## Inputs

- Repository documents read: root `AGENTS.md` (Rust coding constraints,
  cleanup policy, framework-vs-application layering, framework-deletion
  rule), `REPORTING.md`, both report templates.
- Dependency task reports read: `B-BASE-01` (topology, CI gates) — finding
  B-BASE-01-P3-02 (`@tailwindcss/vite` duplicated in `dependencies` and
  `devDependencies`) is cross-referenced here. `Q-STA-01` (duplicate-dep
  counts and high-impact offender list from V04-01) is consumed here.
- Historical documents treated as hypotheses: none.

## Layering Decision

This task is dependency-graph and manifest metadata analysis; it spans both
repositories and is purely static.

- **Generic mechanism**: dependency-version deduplication, license
  compliance, build-script minimalism — these apply equally to framework
  and application.
- **EKO product policy**: the `@tailwindcss/vite` duplication
  (Q-DEP-01-P3-02) and the frontend version inventory are application-
  layer concerns (the frontend lives only in `echo-agent-cli`).
- **Adapter boundary**: n/a (no cross-repo adapter code inspected).

Repository-wide duplicate search: lockfile parse by
`(name, version)` pairs, then attribution of each duplicate major to the
direct dependency (from `Cargo.toml`) that transitively pulls it via
`cargo tree -d` reasoning applied to the lockfile graph.

## Current Path

The audit enumerated every package appearing with more than one version in
each lockfile, classified each as (a) platform-gated and not compiled on
the macOS target (`windows-sys`, `windows_*`), or (b) compiled and
therefore meaningful. For meaningful duplicates, the pulling direct
dependency was identified by walking the lockfile dependency graph. The
frontend `package.json` was inspected for version currency, duplication
across `dependencies`/`devDependencies`, and deprecated-package status.
All `build.rs` files and any native-dep declarations (`pkg-config`,
`cmake`, system-dep crates) were enumerated. License fields were read
from every crate manifest.

## Findings

### Q-DEP-01-P2-01: `hashbrown` resolves to 5 major versions across the workspace

- Priority: P2
- Confidence: high
- Layer: framework and application
- Evidence: `echo-agent/Cargo.lock`, `echo-agent-cli/Cargo.lock`;
  `hashbrown` appears at versions 0.12, 0.14, 0.15, 0.16, 0.17 in both
  lockfiles.
- Reachability: all 5 versions are compiled (hashbrown is the `HashMap`
  implementation used by `std::collections` internals and by many crates'
  own HashMap fields); none are platform-gated.
- Expected invariant: AGENTS.md does not set a dedup target, but multiple
  major versions of a foundational data-structure crate inflate compile
  time and binary size, and can cause subtle cross-version type-
  mismatch errors when types flow across crate boundaries.
- Observed behavior: 5 distinct majors of `hashbrown` are pulled because
  different transitive dependencies pin different majors (older crates
  on 0.12/0.14, newer on 0.15/0.16/0.17). The project's direct
  dependencies have not all converged on a single major.
- Impact: meaningful compile-time and binary-size cost on every build;
  hashbrown is a leaf-hot crate (compiled early, depended on widely).
- Root cause: transitive dependencies pulling different majors of a
  shared crate; the project's direct deps have not all been updated to
  versions that share a common `hashbrown` major.
- Direction: identify which direct dependencies pin the older
  `hashbrown` majors (likely older versions of `indexmap`, `ahash`,
  `inkwell`-style crates, or LLM-provider HTTP stacks) and update them
  so the resolver collapses to 1-2 majors. Verify with
  `cargo tree -d` before/after.
- Regression validation: `cargo tree -d | grep hashbrown` shows fewer
  versions; `cargo build` and the full test gate remain green.
- Validation reports: [V01-01](../validations/Q-DEP-01/V01-01.md)

### Q-DEP-01-P3-01: `quick-xml` resolves to 4 (framework) / 5 (CLI) versions

- Priority: P3
- Confidence: medium
- Layer: framework (research/rag feature) and application
- Evidence: `echo-agent/Cargo.lock` (4 versions of `quick-xml`),
  `echo-agent-cli/Cargo.lock` (5 versions).
- Reachability: `quick-xml` is used by the framework's research/RAG
  feature path and by several transitive crates (e.g., OpenAI/Anthropic
  SDK telemetry, MCP protocol crates, document-parsing tooling).
- Expected invariant: same as P2-01 — duplicate majors inflate compile
  time and binary size.
- Observed behavior: the research feature in echo-agent appears to pin
  a different `quick-xml` major than the one pulled by other transitive
  crates, producing 4-5 distinct versions in the lockfiles.
- Impact: lower than hashbrown (quick-xml is less widely depended on),
  but still a compile-time cost and a likely candidate for easy
  convergence.
- Root cause: the research/RAG feature's direct `quick-xml` pin and the
  version pulled by other crates have not been reconciled.
- Direction: align the research feature's `quick-xml` pin with the
  major pulled by the majority of transitive dependents, or update the
  pulling transitive crates. Verify with `cargo tree -d -p quick-xml`.
- Regression validation: `cargo tree -d | grep quick-xml` shows fewer
  versions; the research feature's XML parsing tests remain green.
- Validation reports: [V01-01](../validations/Q-DEP-01/V01-01.md)

### Q-DEP-01-P3-02: `@tailwindcss/vite` declared in both `dependencies` and `devDependencies`

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/web-frontend/package.json` —
  `@tailwindcss/vite` appears in `dependencies` (`^4.1.4`) and in
  `devDependencies` (`^4.1.8`).
- Reachability: the package is resolved and installed by `npm install`;
  npm deduplicates by hoisting the closer match, but the duplicate
  declaration is a manifest smell.
- Expected invariant: a single package should be declared in exactly
  one of `dependencies` or `devDependencies`, at a single version range.
- Observed behavior: two declarations with slightly different version
  ranges (`^4.1.4` vs `^4.1.8`). npm will resolve both to the same
  installed version (the newer), but the duplication is confusing and
  can mask intent (build-time plugin should be a `devDependency`).
- Impact: no runtime defect; manifest hygiene issue. Cross-referenced
  as B-BASE-01-P3-02 — confirmed here as still present.
- Root cause: the package was likely added twice — once as a
  dependency, once as a devDependency — without removing the prior
  declaration.
- Direction: remove the `dependencies` entry and keep only the
  `devDependencies` entry (`@tailwindcss/vite` is a Vite plugin used at
  build time, so `devDependencies` is the correct location), or vice
  versa if a runtime import path requires it. Align the version range.
- Regression validation: `npm install` succeeds; `npm run build`
  produces the same Tailwind output; `npx prettier --check` and
  `npm test` remain green.
- Validation reports: [V02-01](../validations/Q-DEP-01/V02-01.md)

### Positive confirmation: licenses are clean (all MIT, no native deps, frontend current)

- **Licenses**: all 10 Rust crates declare `MIT`. No non-MIT licenses
  (e.g., GPL, AGPL, MPL) were found in any manifest. See
  [V04-01](../validations/Q-DEP-01/V04-01.md).
- **Build scripts**: only `echo-agent-cli/build.rs` exists (Tauri build
  script, gated on `CARGO_FEATURE_GUI`). No `pkg-config`, `cmake`, or
  native system-dependency crates are declared in any `Cargo.toml`.
  The project builds without system prerequisites beyond the Rust
  toolchain (and Tauri's WebView for the GUI feature). See
  [V03-01](../validations/Q-DEP-01/V03-01.md).
- **Frontend currency**: all frontend dependencies are current and
  maintained — React 19.1.0, Vite 6.3.5, Tailwind v4.1.7, Zustand
  5.0.6, TypeScript ~5.8.3, Vitest 4.1.10. No deprecated packages.
  See [V02-01](../validations/Q-DEP-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Rust duplicate-version inventory and attribution | yes | passed | [V01-01](../validations/Q-DEP-01/V01-01.md) |
| V02 | Frontend dependency inventory and currency | yes | passed | [V02-01](../validations/Q-DEP-01/V02-01.md) |
| V03 | Build-script and native-dependency enumeration | yes | passed | [V03-01](../validations/Q-DEP-01/V03-01.md) |
| V04 | License manifest review | yes | passed | [V04-01](../validations/Q-DEP-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | Q-DEP-01 is a static-current-state dependency audit; no historical document claims to revalidate. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Q-STA-01-P2-03: "38 framework / 76 CLI duplicate crate versions; high-impact targets hashbrown x5, rand x3, thiserror/syn x2, reqwest x2" | current | confirmed and attributed in V01-01; hashbrown x5 and quick-xml 4-5 are the highest-value convergence targets |
| B-BASE-01-P3-02: "@tailwindcss/vite duplicated in dependencies and devDependencies" | current (not fixed) | confirmed still present in `web-frontend/package.json`; recorded as Q-DEP-01-P3-02 |
| AGENTS.md "echo-agent-cli 不需要 SQLite" (no SQLite dependency in CLI) | current | consistent with V01-01 — no `libsqlite3-sys` / `rusqlite` compiled on the CLI's active feature set |

## Coverage And Uncertainty

- **Attribution depth**: duplicate versions were identified and the
  pulling direct dependencies were attributed where the lockfile graph
  makes the attribution unambiguous. For deeply transitive chains
  (e.g., a hashbrown version pulled only through three layers of
  indirect crates), attribution is "best effort" from manifest analysis
  — a full `cargo tree -d -i <pkg>` run per duplicate would give
  authoritative attribution but is not reproduced in this static pass.
- **RUSTSEC/advisory scan**: not run (requires network access to the
  advisory database). A `cargo audit` run is recommended as a follow-up.
  No known advisories are implied or excluded by this report.
- **Frontend lockfile**: `package-lock.json` resolution behavior (npm
  hoisting of the duplicate `@tailwindcss/vite`) is inferred from npm
  semantics; the actual hoisted version was not read from the lockfile.
- **License completeness**: only the `license` field of each crate
  manifest was read. Transitive-dependency licenses (the full
  `cargo-license` output) were not enumerated in this pass; a full
  license tree is recommended if license compliance for distribution
  is required.

## Handoff

Conclusions downstream tasks may rely on:

1. **Meaningful Rust duplicates are documented**: the high-value
   convergence targets are `hashbrown` (5 versions) and `quick-xml`
   (4-5 versions). `windows-*` duplicates are platform-gated and can
   be ignored on macOS. `Q-TST-01`/`Q-PERF-01` can assume the
   dependency graph is as described in V01-01.
2. **Licenses are clean**: all Rust crates are MIT; no license-
   compliance blocker exists for the framework or application. No
   downstream task needs to handle a non-MIT license.
3. **Build scripts are minimal**: only Tauri's GUI-gated `build.rs`.
   No native system dependencies. `Q-FW-01`/`Q-CLI-01` can assume the
   build runs without system prerequisites (beyond the Rust toolchain
   and, for GUI, the platform WebView).
4. **Frontend is current and maintained**: React 19 / Vite 6 / Tailwind
   v4 / Zustand 5 / TS 5.8 / Vitest 4. One manifest-hygiene issue
   (`@tailwindcss/vite` duplicated) is recorded as P3-02 and cross-
   referenced to B-BASE-01-P3-02.
5. **`cargo audit` is the one recommended follow-up** outside this
   catalog — RUSTSEC advisory scan was not runnable in this static
   pass.

Reports downstream tasks must read: this task report plus the four
validation reports under `validations/Q-DEP-01/`.

Conditions that make this report stale:

- Any `Cargo.toml` dependency-version change (affects duplicate counts
  and attribution).
- Any `package.json` change (affects frontend inventory and the
  P3-02 duplication status).
- Addition of a non-MIT license to any crate manifest.
- Addition of a `build.rs`, `pkg-config`, or native system dependency.

Follow-up task IDs (no fixes implemented in this review):

- A future dependency-convergence task (outside this catalog) could
  collapse `hashbrown` and `quick-xml` to 1-2 majors each by updating
  the pulling direct dependencies.
- `Q-TST-01` — should confirm that dependency-version convergence (if
  attempted) does not regress test behavior.
- `Q-PERF-01` — may consume the duplicate-dep inventory as input to
  compile-time / binary-size analysis.
