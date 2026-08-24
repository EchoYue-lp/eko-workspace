# Q-WEB-01: Frontend submission gate

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean in both repositories (`git status --short` empty,
> verified before and after every validation). Note: the 79-file
> `web-frontend/src/generated/` formatting drift recorded by A-FE-01/A-FE-02
> no longer exists — the tree was restored between those reports and this run;
> see Q-WEB-01-P3-01.

## Question

Does the frontend pass formatting, unit/integration tests, and production
build?

**Answer: yes — all three gates pass at the reviewed commit with a clean
tree.** `npx prettier --check "src/**/*.{ts,tsx}"` exits 0, `npm test`
(vitest) exits 0 (26 files / 101 tests), and `npm run build`
(`tsc -b && vite build`) exits 0 (built in 32.15 s). One P3 build-hygiene
finding (regeneration fragility, canonical A-FE-01-P3-02) is confirmed in
source but does not fail any gate in the current tree state.

## Scope

- `echo-agent-cli/web-frontend/` submission gates only: `prettier --check`
  (`src/**/*.{ts,tsx}`), `npm test` (`vitest run`), `npm run build`
  (`tsc -b && vite build`), per AGENTS.md 条件矩阵.
- Generated-artifact workflow mechanism: `echo-agent-cli/echo-agent-app-core/
  src/workspace/mod.rs:236-239` (`__ts_rs` feature test), `web-frontend/
  .prettierrc`, `package.json` scripts (no generation wrapper).
- Pre/post `git status` of `web-frontend/src/generated/` (no ts-rs command
  executed; no regeneration triggered by vitest/build).

## Out Of Scope

- Tauri/GUI Rust matrix (`build:tauri`, `--mode tauri`, GUI tests) → Q-GUI-01.
- Dynamic GUI smoke / e2e → Q-E2E-01.
- Type-contract drift findings (ToolInfo, SkillInfo/McpServerInfo, dormant
  HTTP types) → A-FE-01 (P2-01/P2-02/P3-01/P3-03), read as dependencies.
- Frontend projection/reducer defects → A-FE-02, A-SRF-03 (dependencies).
- Any code fix or working-tree modification (read-only review).

## Inputs

- Root `AGENTS.md` (frontend gate commands under 条件矩阵; read-only review;
  no ts-rs runs), shared `README.md`, `REPORTING.md`, `TASKS.md` (Q-WEB-01
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read (zcode-ds): `A-SRF-03` (complete),
  `A-FE-01` (complete — P3-02 generated drift, V04-03 prettier failure),
  `A-FE-02` (complete — baseline md5/dirty-state practice).
- Historical documents treated as hypotheses: A-FE-01-P3-02's prediction that
  Q-WEB-01 would observe the same prettier failure; A-FE-01 V04-01 vitest
  baseline; the dirty-tree notes in A-SRF-03/A-FE-02 headers.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | None touched — the gates exercise application-layer build tooling only. |
| EKO product policy (application, correct) | The three gate commands, `package.json` scripts, `.prettierrc`, and the generated-TS workflow are application build hygiene. |
| Adapter boundary | `ts-rs` generation (`__ts_rs` test in workspace/mod.rs:236-239) is the Rust→TS artifact bridge; its formatting gap is the finding below. |
| Duplicate search | `prettier` (single dev-dependency, ^3.8.3; single `.prettierrc`), `vitest` (single, ^4.1.10; `test` script only), build scripts (`build`/`build:tauri` only; no generate/format wrapper exists — verified zero matches for a generation script in package.json), `__ts_rs` (single feature; workspace/mod.rs:236). |

## Current Path

Verified command flow (V01-01/V02-01/V03-01):

1. **Formatting**: `npx prettier --check "src/**/*.{ts,tsx}"` (prettier ^3.8.3,
   `.prettierrc`: singleQuote, trailingComma "es5", printWidth 100, semi,
   bracketSpacing, arrowParens always) — exit 0, "All matched files use
   Prettier code style!". The checked files are the committed HEAD versions
   because the tree is clean; `git -C .. status --short
   web-frontend/src/generated/` was empty before and after.
2. **Tests**: `npm test` = `vitest run` (vitest ^4.1.10) — exit 0,
   26 files / 101 tests passed (8.01 s), identical to the A-FE-01 V04-01
   baseline at the same commit.
3. **Build**: `npm run build` = `tsc -b && vite build` — exit 0 (32.15 s);
   `tsc -b` zero errors, vite emitted `dist/` (gitignored; post-run
   `git status` empty).

## Findings

### Q-WEB-01-P3-01: The prettier gate passes only because the tree is clean — the ts-rs regeneration workflow still writes unformatted output, so any regeneration re-breaks the gate (canonical A-FE-01-P3-02, independently confirmed)

- Priority: P3
- Confidence: high (mechanism verified in source; current pass observed)
- Layer: application (build hygiene / generated-artifact workflow)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/workspace/mod.rs:236-239` —
    `#[cfg(feature = "__ts_rs")] mod ts_bindings` with comment "ts-rs will
    auto-generate TypeScript bindings ... when `cargo test` is run with the
    `__ts_rs` feature enabled" — i.e. the documented regeneration step writes
    `web-frontend/src/generated/`.
  - `web-frontend/.prettierrc` (singleQuote, trailingComma "es5",
    printWidth 100) matches the committed HEAD shape, not raw ts-rs output;
    the committed generated files are prettier-formatted (81 `.ts` files in
    `generated/`).
  - `package.json` scripts: `dev`, `dev:tauri`, `test`, `build`, `build:tauri`,
    `preview` — no generation or post-generation prettier wrapper exists.
  - A-FE-01-P3-02 observed the consequence empirically: a fresh regeneration
    left 79 files dirty and `prettier --check` exit 1 (A-FE-01 V04-03).
- Reachability: any developer/build running `cargo test --features __ts_rs`
  in `echo-agent-app-core` dirties `web-frontend/src/generated/*.ts` and turns
  the formatting gate red until `npx prettier --write` is manually re-run;
  that is the documented workflow (workspace/mod.rs:239 comment).
- Expected invariant: committed generated files are reproducible from the
  documented generation step and the formatting gate stays green (AGENTS.md
  提交前门禁 — "fmt 不干净的提交会让 CI 红").
- Observed behavior: at the reviewed commit (b3b2e81, clean tree) the gate is
  green — `prettier --check` exit 0 (V01-01) — because the working tree was
  restored to the prettier-formatted HEAD state since A-FE-01 ran; the
  mechanism that breaks the gate after regeneration is unchanged in source.
- Impact: low today (all gates green); latent CI/fmt-gate fragility and
  non-reproducible committed artifacts on any regeneration, plus permanent
  dirty status for anyone who regenerates — the same impact as A-FE-01-P3-02.
- Root cause: no generation script wraps ts-rs output with prettier, so the
  committed state depends on an undocumented manual `prettier --write` pass.
- Direction: adopt the A-FE-01-P3-02 direction — add a generation script (or
  document in the `__ts_rs` test comment) that runs
  `npx prettier --write src/generated/**/*.ts` immediately after export, or
  commit raw output and exclude `generated/` from the prettier gate; then
  verify the tree stays clean after a fresh generate→format cycle. No new
  authority — this is the same defect class, canonical ID A-FE-01-P3-02.
- Regression validation: `git status` clean after generate→format;
  `prettier --check` exit 0; `git diff` empty after a fresh generation cycle.
- Validation reports: [V01-01](../validations/Q-WEB-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | `npx prettier --check "src/**/*.{ts,tsx}"` — formatting gate | yes | passed (exit 0) | [V01-01](../validations/Q-WEB-01/V01-01.md) |
| V02 | `npm test` (`vitest run`) — unit/component suite | yes | passed (exit 0; 26 files / 101 tests) | [V02-01](../validations/Q-WEB-01/V02-01.md) |
| V03 | `npm run build` (`tsc -b && vite build`) — production build | yes | passed (exit 0; 32.15 s) | [V03-01](../validations/Q-WEB-01/V03-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending. Pre/post `git status` of
`web-frontend/src/generated/` was recorded for every run (empty both times);
no command that regenerates `generated/*.ts` was executed.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-FE-01-P3-02 "generated-artifact commit-state inconsistency … Q-WEB-01 will observe the same prettier failure" | stale (active failure) / mechanism current | Gate is green at the reviewed commit (V01-01: exit 0) because the tree was restored; the regeneration mechanism is unchanged (workspace/mod.rs:236-239, `.prettierrc`, no wrapper script) — Q-WEB-01-P3-01 keeps the canonical A-FE-01-P3-02 conclusion valid. |
| A-FE-01 V04-01 "vitest 26 files / 101 tests, exit 0" | current | Reproduced identically at the same commit (V02-01: 26 files / 101 tests, exit 0). |
| A-SRF-03 / A-FE-02 worktree notes "79 modified files, all `web-frontend/src/generated/*.ts`" | stale | `git status --short` empty at the reviewed commit; 0 modified files in `generated/` before and after every run. |
| AGENTS.md 条件矩阵 "修改 web-frontend → `npx prettier --check "src/**/*.{ts,tsx}"`、`npm test`、`npm run build`" | current | All three commands executed verbatim and green (V01-01/V02-01/V03-01). |

## Coverage And Uncertainty

- All conclusions are executable-gate results at commit b3b2e81; no GUI
  process was launched (Q-GUI-01/Q-E2E-01 own dynamic confirmation).
- `npm test` covers unit/component tests only (26 files / 101 tests); there
  is no e2e suite in the frontend (Q-E2E-01 scope).
- The prettier glob is exactly the AGENTS.md gate (`src/**/*.{ts,tsx}`);
  root-level config files and non-src code are not covered by the gate.
- The ts-rs regeneration itself was NOT executed (read-only review
  prohibition); the dirty-tree consequence relies on A-FE-01's empirical run
  plus the unchanged source mechanism — not re-produced here.
- `build:tauri` (Tauri production bundle) was not executed; the GUI matrix is
  Q-GUI-01 scope.
- The tree-restoration event between A-FE-01 and this review was not
  observable from git history (no new commit; worktree restored to HEAD
  content) — recorded as a state fact, not analyzed further.

## Handoff

- Conclusions downstream tasks may rely on: the frontend submission gate is
  green at commit b3b2e81 with a clean tree — prettier exit 0, vitest
  26/101 exit 0, production build exit 0; the only open frontend-gate item is
  the regeneration fragility (Q-WEB-01-P3-01 = canonical A-FE-01-P3-02),
  which is latent, not currently failing.
- Reports to read: this report + V01-01..V03-01; dependency reports A-FE-01
  (P3-02 canonical, V04-03), A-FE-02, A-SRF-03.
- Stale triggers: any change to web-frontend source formatting, `.prettierrc`,
  `package.json` scripts, the `__ts_rs` feature or `workspace/mod.rs`, or a
  fresh ts-rs regeneration (tree becomes dirty and the prettier gate turns
  red again — then V01-01's conclusion no longer holds).
- Follow-up task IDs (fixes not implemented in this review): S-APP-01
  (application synthesis consumes this gate result), S-QA-01 (command/report
  reconciliation), S-RDM-01 (roadmap item: generation-script fix for
  A-FE-01-P3-02/Q-WEB-01-P3-01), Q-E2E-01 (dynamic smoke on the green
  build/test baseline).
