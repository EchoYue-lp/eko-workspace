# B-BASE-01: Repository and build topology

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

What packages, workspace members, targets, features, optional dependencies,
and cross-repository path dependencies exist now?

## Scope

Primary source paths inspected (read-only):

- All 10 `Cargo.toml` files: `echo-agent/Cargo.toml` (root + 7 sub-crates)
  and `echo-agent-cli/Cargo.toml` (root + `echo-agent-app-core`).
- `echo-agent/Cargo.lock` (160 355 B) and `echo-agent-cli/Cargo.lock`
  (240 188 B) — presence and packaging exclude only.
- `echo-agent-cli/web-frontend/package.json`.
- `echo-agent-cli/build.rs`, `echo-agent-cli/tauri.conf.json`,
  `echo-agent-cli/src-tauri/src/main.rs`.
- `.github/workflows/rust-ci.yml` in both repos.
- `.cargo/config.toml` in both repos, `rust-toolchain.toml` in `echo-agent-cli`.
- `AGENTS.md` "提交前门禁" and "条件矩阵" sections (cross-repo gate rules).
- `examples/` and `benches/` directory listings for `echo-agent`.

## Out Of Scope

Deferred to named task IDs:

- Per-feature standalone compile matrix execution → `F-FEAT-01`, `Q-FW-02`.
- Crate dependency graph cycle/facade analysis → `B-ARCH-01`.
- Entry-point call graph and composition-root inventory → `B-PATH-01`.
- Historical audit-document drift → `B-DOC-01`.
- Dependency duplicate/license/advisory scan → `Q-DEP-01`.
- Source-level reachability of any feature-gated module → `F-FEAT-01`.

## Inputs

- Repository documents read in full: root `AGENTS.md`, `docs/comprehensive-review/README.md`,
  `docs/comprehensive-review/REPORTING.md`, both report templates, the
  `B-BASE-01` task card in `TASKS.md`.
- Dependency task reports read: none (B-BASE-01 has no dependencies).
- Historical documents treated as hypotheses: none relevant (this is the
  baseline topology task).

## Layering Decision

This task is purely structural and spans both repositories. The relevant
`AGENTS.md` invariants verified here:

- **Generic mechanism**: workspace topology, feature definitions, build
  scripts, CI gates — these belong to whichever repo hosts them.
- **EKO product policy**: the `default-features = false` + curated feature
  subset that `echo-agent-cli` enables on `echo_agent` (notably excluding
  `sqlite`) is an application-layer decision (AGENTS.md "echo-agent-cli 不需要 SQLite").
- **Adapter boundary**: `echo-agent-cli/build.rs` and the CI symlink hack
  (`ln -s … ../echo-agent`) are thin adaptations that let the application
  consume the sibling framework repo without modifying framework manifests.

Repository-wide duplicate search terms used: `worktrees`, `/Users/` (path
leak rule), `path = "../` and `path = "../../` (cross-repo deps),
`autoexamples`/`autobins`/`autotests`/`autobenches` (target auto-discovery),
`[[bin]]`/`[[example]]`/`[[bench]]`/`[[test]]`/`[lib]` (target declarations).
Results: 0 path leaks; relative paths only; auto-discovery defaults in
effect; targets enumerated in V01/V03.

## Current Path

The build topology, as it exists at the reviewed commits:

```
lp-agent/                              (not a git repo — workspace root only)
├── echo-agent/                        git repo, commit 9b0e0fa, resolver = "3"
│   ├── Cargo.toml                     root PACKAGE echo_agent v0.2.0 + workspace root
│   │                                  members: 7 sub-crates; default-members: . + 7
│   ├── Cargo.lock                     present (160 355 B), excluded from package
│   ├── benches/agent_bench.rs         harness = false, no required-features
│   ├── examples/                      68 .rs files (55 declared, 13 auto-discovered)
│   ├── .cargo/config.toml             macOS rustflags + [toolchain] stable
│   ├── .github/workflows/rust-ci.yml  lint + test(8-matrix) + minimal jobs
│   ├── echo-core/                     echo_core v0.2.0, 6 features
│   ├── echo-macros/                   echo_macros v0.2.0, proc-macro, 0 features
│   ├── echo-execution/                echo_execution v0.2.0, default=["files","shell"]
│   ├── echo-integration/              echo_integration v0.2.0, 4 features
│   ├── echo-tools/                    echo_tools v0.2.0, 12 features
│   ├── echo-state/                    echo_state v0.2.0, 1 feature (sqlite)
│   └── echo-orchestration/            echo_orchestration v0.2.0, 1 feature (websocket)
│
└── echo-agent-cli/                    git repo, commit b3b2e81, resolver = "3"
    ├── Cargo.toml                     root PACKAGE echo-agent-cli v1.0.0 + workspace root
    │                                  members: echo-agent-app-core
    │                                  bins: echo-agent-cli (src/main.rs)
    │                                       echo-agent-tauri (src-tauri/src/main.rs, req ["gui"])
    ├── Cargo.lock                     present (240 188 B)
    ├── build.rs                       runs tauri_build only when CARGO_FEATURE_GUI set
    ├── tauri.conf.json                productName EKO, features ["gui"]
    ├── .cargo/config.toml             gui-* aliases + TS_RS_EXPORT_DIR + macOS rustflags
    ├── rust-toolchain.toml            stable + clippy + rustfmt
    ├── .github/workflows/rust-ci.yml  single ci job (fmt+clippy+test+app-core no-default)
    ├── echo-agent-app-core/           echo-agent-app-core v1.0.0, 1 feature (telemetry)
    ├── src-tauri/                     binary target dir, NOT a workspace member
    └── web-frontend/                  React 19 + Vite 6 + Tailwind v4 + Zustand 5
```

Cross-repo dependency wiring (all relative, all `default-features = false`
on the `echo_agent` dep):

- `echo-agent-cli/Cargo.toml:50-51` → `echo_agent @ ../echo-agent`
  (features: mcp, lsp, human-loop, subagent, tasks) and
  `echo_core @ ../echo-agent/echo-core`.
- `echo-agent-app-core/Cargo.toml:10-16` → `echo_agent @ ../../echo-agent`
  (features: mcp, lsp, human-loop, subagent, git, tasks, shell, files, web,
  data, statistics, chart, research, media, rag) and
  `echo_core @ ../../echo-agent/echo-core`.
- `echo-agent-app-core/Cargo.toml:59` (dev-dep) →
  `echo_agent @ ../../echo-agent` (features: testing).
- `gui` feature at the CLI root forwards `channels` → `echo-agent/channels`.
- `telemetry` feature (CLI root and app-core) forwards to `echo-agent/telemetry`.

CI reproduces the sibling layout via a symlink
(`echo-agent-cli/.github/workflows/rust-ci.yml:21`), which is the only
adaptation needed to honor the relative path on a fresh runner.

State owners and identities: both root manifests are simultaneously the
workspace root and a package. The `echo-agent-cli` workspace has no
`default-members` key, so bare `cargo` commands build the root package and
`echo-agent-app-core`. The `echo-agent` workspace explicitly lists
`default-members` including `.`.

Recovery points: the `--locked` flag is used on every clippy/test/check in
both CIs, anchoring builds to the committed `Cargo.lock`. The
`rust-toolchain.toml` / `.cargo/config.toml [toolchain]` pinning anchors
the compiler to stable.

## Findings

### B-BASE-01-P2-01: Cross-repo path-dependency hygiene is clean (positive confirmation)

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/Cargo.toml:50-51`,
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-16,59`,
  `echo-agent/.gitignore:23`, `echo-agent-cli/.gitignore:17,20`
- Reachability: definition (3 path deps in 2 manifests) → registration
  (`cargo metadata` resolves them) → live caller (CI builds the CLI against
  the sibling repo every push/PR via the symlink adaptation)
- Expected invariant (`AGENTS.md` "Worktree 并行开发与合并规范" rule 1):
  `Cargo.toml` paths must be relative (`../echo-agent` or
  `../../echo-agent`), never absolute or worktree-prefixed; `.gitignore`
  must contain `.worktrees/`.
- Observed behavior: all 5 cross-repo `path =` declarations use the correct
  relative form. `grep -rn "worktrees\|/Users/"` across all 10 `Cargo.toml`
  returns 0 matches. Both `.gitignore` files contain `.worktrees/`. CI uses
  a symlink hack rather than rewriting manifests.
- Impact: any developer fresh-cloning both repos side by side can build
  `echo-agent-cli` without editing manifests. No worktree-path leak is
  pending on `main`.
- Root cause: n/a — this is a confirmation that prior cleanup held.
- Direction: no change required. The CI symlink
  (`ln -s "$GITHUB_WORKSPACE/echo-agent" "$GITHUB_WORKSPACE/../echo-agent"`)
  is the only adaptation; document it if `Q-DOC-01` finds it underexplained.
- Regression validation: re-run
  `grep -rn "worktrees\|/Users/" */Cargo.toml` after any worktree merge.
- Validation reports: [V01](../validations/B-BASE-01/V01-01.md)

### B-BASE-01-P2-02: CLI never enables framework `sqlite`; feature plumbing matches AGENTS.md (positive confirmation)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/Cargo.toml:50`,
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-15,59`
- Reachability: definition → compile-time feature unification across the
  CLI workspace → no `rusqlite`/`sqlx` symbol referenced from CLI crates
  (the framework's `sqlite` feature would otherwise pull `rusqlite` and
  `echo_state/sqlite`)
- Expected invariant (`AGENTS.md` "产品定位与安全边界" → "数据持久化"):
  `echo-agent-cli` must not enable SQLite; the `sqlite` feature,
  `SqliteStore`, and `SqliteConversationStore` remain valid framework
  options whose deletion criterion is framework-wide, not CLI-driven.
- Observed behavior: every CLI-side declaration of `echo_agent` uses
  `default-features = false` with a curated feature list that excludes
  `sqlite`. The union is `mcp, lsp, human-loop, subagent, tasks, git, shell,
  files, web, data, statistics, chart, research, media, rag` (+ `testing`
  dev-only; + `channels` when `gui`; + `telemetry` when `telemetry`).
  `sqlite` and `database` (the latter would pull `sqlx`) are absent.
- Impact: the invariant holds; framework SQLite code is exercised only by
  framework tests/examples, never by the CLI. `F-MEM-02` can rely on this.
- Root cause: n/a — confirmation.
- Direction: no change. Any future PR that adds `sqlite` or `database` to
  the CLI feature list must be rejected per AGENTS.md.
- Regression validation: `grep -n "sqlite\|database" echo-agent-cli/*/Cargo.toml`
  on every CLI dependency change.
- Validation reports: [V02](../validations/B-BASE-01/V02-01.md),
  [V03](../validations/B-BASE-01/V03-01.md)

### B-BASE-01-P2-03: 13 auto-discovered examples have no `required-features` guard

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml` (55 `[[example]]` blocks vs 68 `.rs`
  files in `echo-agent/examples/`); no `autoexamples = false` key present.
- Reachability: definition (the 13 un-declared `.rs` files:
  `demo00_quickstart`, `demo01_tools`, `demo02_tasks`, `demo07_skills`,
  `demo09_file_shell`, `demo10_streaming`, `demo11_callbacks`,
  `demo13_tool_execution`, `demo17_chat`, `demo19_guard`,
  `demo32_token_budget`, `demo33_retry_policy`, `demo40_snapshot`) →
  registration (cargo auto-discovery, `autoexamples` defaults to `true`) →
  live caller (`cargo build --examples`, `cargo test --examples`, and
  `cargo clippy --all-targets` all compile them)
- Expected invariant: an example that calls feature-gated APIs should
  declare `required-features` so that `--no-default-features --examples`
  does not fail to compile.
- Observed behavior: 13 examples are auto-discovered with no
  `required-features`. Several plausibly reference feature-gated APIs
  (e.g. `demo07_skills`, `demo40_snapshot`, `demo32_token_budget`) based on
  their names; whether they actually fail under `--no-default-features` is
  not verified here.
- Impact: `cargo clippy --workspace --all-targets --all-features --locked`
  (CI lint job) compiles them with all features, so the gap is invisible to
  CI. A developer running `cargo build --examples` with default features
  (empty) may hit compile errors that CI does not catch.
- Root cause: examples were added without an explicit `[[example]]` block;
  cargo's auto-discovery silently accepts them.
- Direction: `F-FEAT-01` should attempt `cargo build --examples
  --no-default-features` and either add `required-features` to the missing
  examples or confirm they compile feature-free. Deletion target: none
  (this is a metadata gap, not dead code).
- Regression validation: after fix, `cargo build --examples
  --no-default-features` should succeed or each failing example should have
  a matching `required-features`.
- Validation reports: [V03](../validations/B-BASE-01/V03-01.md)

### B-BASE-01-P2-04: CI does not run any conditional matrix (feature, GUI, frontend)

- Priority: P2
- Confidence: high
- Layer: framework (feature matrix), application (GUI matrix), adapter (frontend)
- Evidence:
  `echo-agent/.github/workflows/rust-ci.yml` (no feature-matrix step),
  `echo-agent-cli/.github/workflows/rust-ci.yml` (no GUI or frontend step,
  despite installing Tauri system deps at lines 23-30)
- Reachability: definition (CI workflow YAML) → registration (GitHub
  Actions triggers on push/PR) → live caller (every push/PR to `main`/
  `master` runs exactly the documented steps, nothing more)
- Expected invariant (`AGENTS.md` "条件矩阵"): when a change touches
  feature definitions, `#[cfg]`, GUI code, or frontend code, the developer
  must locally run the conditional matrix before commit. CI is the
  backstop described as "CI/专项审计兜底".
- Observed behavior: neither CI runs the conditional matrices. Concretely
  missing:
  - `echo-agent`: the per-feature loop
    `for feature in sqlite subagent human-loop mcp lsp a2a git database rag chart web media; do cargo check -p echo_agent --no-default-features --features "$feature"; done`
  - `echo-agent-cli`: `cargo check --no-default-features --features gui --bin echo-agent-tauri`
    and `cargo test --no-default-features --features gui`.
  - `echo-agent-cli`: `npx prettier --check`, `npm test`, `npm run build`
    (no Node toolchain installed).
- Impact: feature-isolation regressions (a feature that no longer compiles
  standalone), GUI build regressions, and frontend type/test/build
  regressions can all merge on `main` without CI catching them. The
  mitigation is developer discipline plus the `Q-*` review tasks.
- Root cause: CI was designed for the mandatory non-conditional gate only;
  the conditional matrices were left to local execution and "CI/专项审计兜底".
- Direction: this is a coverage-gap observation, not a defect in the
  current task's scope. `Q-FW-02`, `Q-GUI-01`, `Q-WEB-01` will exercise
  the matrices. A follow-up implementation task (outside this catalog)
  could add the matrices to CI.
- Regression validation: n/a (read-only review).
- Validation reports: [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P3-01: `echo-agent` test job uses `--lib --tests`, not `--all-targets`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/.github/workflows/rust-ci.yml:62`
  (`cargo test -p "${{ matrix.package }}" --lib --tests --all-features --locked`)
- Reachability: CI test job (matrix of 8 packages) runs on every push/PR.
- Expected invariant (`AGENTS.md` "提交前门禁" for echo-agent):
  `cargo test --workspace --all-targets --all-features --locked`.
- Observed behavior: CI test job uses `--lib --tests` per package, which
  excludes examples, benches, and doctests from test execution. The in-file
  comment justifies this: "The lint job already compiles examples and
  benches with `--all-targets`." So they are type-checked (clippy) but their
  `#[test]` functions (if any) and doctests are not run.
- Impact: low. Most examples are demonstration `fn main()` without `#[test]`
  functions, and the project has few doctests. The gap is a coverage hole,
  not a known failing test.
- Root cause: CI optimization to avoid building 68 examples twice.
- Direction: optional. Either align CI with `--all-targets` or accept the
  trade-off and document it. `Q-FW-01` / `Q-TST-01` can decide.
- Regression validation: n/a.
- Validation reports: [V04](../validations/B-BASE-01/V04-01.md)

### B-BASE-01-P3-02: `@tailwindcss/vite` is duplicated in `package.json` deps and devDeps

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/web-frontend/package.json:16` (dependencies,
  `^4.1.4`) and `:32` (devDependencies, `^4.1.8`)
- Reachability: definition → `npm install` resolves one version (npm
  de-duplicates by name; the devDependencies entry typically wins for the
  installed tree) → live caller (Vite loads the plugin at build time)
- Expected invariant: a package should appear in either `dependencies` or
  `devDependencies`, not both, with a single version spec.
- Observed behavior: `@tailwindcss/vite` is listed in both sections with
  different caret ranges. The build still works because npm picks one, but
  the duplicate is confusing and the two ranges disagree.
- Impact: low. No functional break observed; the discrepancy is a
  maintenance smell and a likely lint warning from `npm ls` or
  `prettier-plugin-packagejson`.
- Root cause: probable copy-paste when adding Tailwind v4.
- Direction: remove the `dependencies` entry (`:16`) and keep the
  `devDependencies` entry (`:32`) — Tailwind Vite plugin is a build-time
  tool. Deletion target: `package.json:16`.
- Regression validation: `npm install && npm run build` should still
  succeed; `npm ls @tailwindcss/vite` should show a single entry.
- Validation reports: [V02](../validations/B-BASE-01/V02-01.md)

### B-BASE-01-P3-03: CI runs on `ubuntu-latest` only; macOS/Windows paths unexercised

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence: both `.github/workflows/rust-ci.yml` files use
  `runs-on: ubuntu-latest`; `.cargo/config.toml` in both repos sets
  `target.'cfg(target_os = "macos")' rustflags`.
- Reachability: CI runner OS selection → every push/PR.
- Expected invariant: the product targets macOS desktop (Tauri) primarily;
  CI should ideally exercise at least one macOS runner.
- Observed behavior: all CI runs on Linux. macOS-specific `rustflags` and
  any macOS-only code paths are not validated in CI. The Tauri Linux build
  works because system deps are installed, but the shipping platform is
  macOS.
- Impact: low-to-medium. A macOS-only regression (e.g., a Cargo flag that
  only emits on macOS) would not be caught. The `.cargo/config.toml`
  `-A linker_messages` flag exists precisely because macOS emits noisy
  linker warnings — confirming macOS builds happen locally but not in CI.
- Root cause: cost / runner-availability choice.
- Direction: optional future CI matrix addition (`runs-on: macos-latest`).
  Outside this read-only review.
- Regression validation: n/a.
- Validation reports: [V04](../validations/B-BASE-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Manifest/member inventory and path-leak search | yes | passed | [V01-01](../validations/B-BASE-01/V01-01.md) |
| V02 | Feature-to-dependency graph + frontend package.json | yes | passed | [V02-01](../validations/B-BASE-01/V02-01.md) |
| V03 | Target/required-feature inventory + CLI feature cross-check | yes | passed | [V03-01](../validations/B-BASE-01/V03-01.md) |
| V04 | CI-versus-AGENTS gate comparison | yes | passed | [V04-01](../validations/B-BASE-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | B-BASE-01 has no historical document dependencies; baseline task |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md` "三个项目的定位" — echo-agent is a workspace with 7 sub-crates + root | current | `echo-agent/Cargo.toml:1-21` lists exactly 7 members; root package `echo_agent` is implicit 8th |
| `AGENTS.md` "echo-agent-cli 不启用 SQLite" | current | `echo-agent-cli/Cargo.toml:50`, `echo-agent-app-core/Cargo.toml:10` both `default-features = false` without `sqlite` |
| `AGENTS.md` "Worktree 并行开发" rule 1 — Cargo.toml paths must be relative | current | 0 `worktrees` or `/Users/` matches across all 10 Cargo.toml |
| `README.md` baseline table — echo-agent ≈ 490 Rust files / 183k LOC | not verified here | file/line counts are out of scope for B-BASE-01; deferred to `Q-STA-01` |
| `README.md` baseline table — echo-agent-cli ≈ 200 Rust + 377 TS/TSX files | not verified here | same as above |

## Coverage And Uncertainty

- **Manifests**: all 10 `Cargo.toml` files read in full. No `[[test]]` or
  `[[bench]]` targets exist outside the root `agent_bench`. Confidence: high.
- **Source-level feature gates**: this task did not open any `.rs` file to
  verify that a feature like `handoff` actually gates non-trivial code.
  That is `F-FEAT-01`'s job. The 12 root features that activate no `dep:`
  (`handoff`, `topology`, `tasks`, `project-rules`, `eval`, `improve`,
  `testing`, `sandbox`, `semantic-memory`, `macros`, `provider-factory`,
  `workflow`, `multimodal`) are recorded as cfg-gate-only based on the
  manifest alone.
- **Auto-discovered examples**: the 13 un-declared examples are identified
  by name; whether each one compiles under `--no-default-features` is not
  verified. Forwarded to `F-FEAT-01`.
- **CI commands**: no CI run was triggered. The comparison is static
  (workflow YAML vs AGENTS.md prose). Actual pass/fail of the gates is
  `Q-FW-01` / `Q-CLI-01`.
- **Frontend**: only `package.json` was inspected. Lockfile
  (`package-lock.json` or `pnpm-lock.yaml`) presence and exact resolved
  versions are not checked; `Q-WEB-01` / `Q-DEP-01` cover that.
- **`Cargo.lock` contents**: only presence and packaging exclude were
  checked. Duplicate-version and advisory scans belong to `Q-DEP-01`.

## Handoff

Conclusions downstream tasks may rely on:

1. **Topology is exactly**: 8 crates in `echo-agent` (root `echo_agent` +
   7 sub-crates), 2 in `echo-agent-cli` (root `echo-agent-cli` +
   `echo-agent-app-core`), both resolver 3, edition 2024, rust-version 1.95,
   MIT. `B-ARCH-01`, `B-PATH-01`, `F-*` tasks can treat this as ground truth.
2. **Feature plumbing is clean**: the CLI enables a known subset of
   framework features with `default-features = false` and never `sqlite`.
   `F-FEAT-01`, `F-MEM-02`, `Q-CLI-01` can rely on this.
3. **Two binaries exist**: `echo-agent-cli` (no gate) and `echo-agent-tauri`
   (gated on `gui`). `B-PATH-01`, `A-SRF-02`, `Q-GUI-01` consume this.
4. **CI does not cover conditional matrices or frontend**. `Q-FW-02`,
   `Q-GUI-01`, `Q-WEB-01` must exercise those locally; do not assume CI
   green implies those configurations pass.
5. **`src-tauri/` is part of the root `echo-agent-cli` package**, not a
   separate crate. `A-SRF-02` should not look for a separate manifest.

Reports downstream tasks must read: this task report plus the four
validation reports under `reports/validations/B-BASE-01/`.

Conditions that make this report stale:

- Any change to `Cargo.toml` in either repo (members, features, deps, bins).
- Addition or removal of `[[example]]`/`[[bench]]`/`[[test]]` blocks.
- Changes to `.github/workflows/rust-ci.yml`.
- Addition of a `src-tauri/Cargo.toml` (would make `src-tauri/` a workspace
  member, invalidating conclusion 5).
- Enabling `sqlite` or `database` on any CLI-side `echo_agent` dependency
  (would invalidate conclusion 2 and the AGENTS.md invariant).

Follow-up task IDs (no fixes implemented in this review):

- `B-ARCH-01` — crate dependency graph and facade analysis.
- `B-PATH-01` — entry-point and composition inventory.
- `F-FEAT-01` — feature isolation matrix (consumes P2-03 and the cfg-gate-only
  feature list).
- `F-MEM-02` — SQLite framework option validation (relies on P2-02).
- `Q-FW-01`, `Q-FW-02`, `Q-CLI-01`, `Q-GUI-01`, `Q-WEB-01` — executable
  gates and the matrices CI skips (consumes P2-04, P3-01).
- `Q-DEP-01` — dependency duplicate/license/advisory scan.
- `Q-STA-01` — file/line counts and static safety audit.
