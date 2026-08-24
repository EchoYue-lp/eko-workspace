# Q-WEB-01: Frontend submission gate

> Status: needs_evidence
> Reviewer: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework external dirty source excluded through committed-object reads; CLI had only external `Cargo.lock` modification, excluded

## Question

Does the frontend pass formatting, unit/integration tests, and production build?

## Scope

- Static verification of frontend scripts, Prettier/Vitest/Vite/TypeScript configuration, dependency lock roots, Tauri production build entry, runtime prerequisites, and CI lane.
- Three separate immutable executable-gate attempts for Prettier, tests, and ordinary production build.
- Deduplication against completed B-BASE, Q-TST, A-FE, and A-SRF owners.

## Out Of Scope

- Running Prettier, npm tests, Vite/TypeScript builds, Vitest, Cargo, fixtures, or network, explicitly prohibited.
- Source, lockfile, CI, documentation, or shared-index changes.
- Reopening frontend runtime defects, CI omission, or missing mounted transport harness under new IDs.
- Claiming that static command existence proves an executable gate passed.

## Inputs

- Root `AGENTS.md`, shared README/REPORTING/TASKS exact card, Codex README and templates.
- Codex B-BASE-01, Q-TST-01, A-SRF-03, and A-FE-01..03 at the relevant pinned commits.
- Committed CLI `web-frontend/package.json`, `package-lock.json`, formatting/build/test configs, Tauri config, README prerequisite, and workflow. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Tool engine compatibility, lockfile reproducibility, TypeScript/Vite/Vitest/Prettier behavior, and process exit status are ecosystem build contracts. |
| EKO product policy | Which frontend and Tauri modes ship, the mandatory three-command gate, Node/npm version, and CI job belong to `echo-agent-cli`. |
| Adapter boundary | Tauri must invoke the same strict type/build authority with its `tauri` mode; it must not maintain a second unchecked build path. |
| Duplicate search | Searched scripts, package/lock dependency maps, tool engines, runtime-version files, TypeScript references, Vitest environment, Vite modes, Tauri commands, workflow Node/npm steps, and completed finding IDs. |
| Migration deletion | Add one runtime/version authority and CI lane; do not add parallel scripts that bypass `tsc -b`. Keep the pure Vitest tier while adding the mounted tier owned by Q-TST-01. |

## Static Gate Topology

```text
locked npm graph
  -> Prettier 3.8.3 -> npx prettier --check "src/**/*.{ts,tsx}"
  -> Vitest 4.1.10 -> npm test -> vitest run
  -> TypeScript 5.8.3 + Vite 6.4.2
       -> npm run build       -> tsc -b -> vite build
       -> npm run build:tauri -> tsc -b -> vite build --mode tauri
            -> tauri.conf.json beforeBuildCommand

current CI
  -> Rust-only workflow; no setup-node/npm ci/frontend step
     (canonical B-BASE-01-P2-01)
```

Positive static conclusions:

- `package.json` dependency maps equal `package-lock.json`'s root maps, and all four gate tools are integrity-locked.
- TypeScript project references include application source and `vite.config.ts`; strict/no-unused/no-fallthrough/no-unchecked-side-effect settings are enabled.
- Both ordinary and Tauri production scripts type-check before bundling; Tauri's committed config reaches `build:tauri`.
- `.prettierrc` is present and the required source glob is explicit in the repository gate.

## Finding

### Q-WEB-01-P2-01: The declared Node.js floor cannot run the locked frontend test tool

- Priority: P2
- Confidence: high
- Layer: application build boundary
- Evidence: `echo-agent-cli/README.md:71-86`; `echo-agent-cli/web-frontend/package.json:1-43`; `package-lock.json:5010-5043`; `echo-agent-cli/.github/workflows/rust-ci.yml:10-45`
- Reachability: every contributor following the current GUI prerequisites installs the committed package graph and `npm test` resolves the locked Vitest binary.
- Expected invariant: one repository-declared Node/npm version satisfies every locked gate tool and is used by local setup and CI.
- Observed behavior: README supports Node.js >=18, but locked Vitest 4.1.10 declares Node `^20 || ^22 || >=24`. The package has no `engines` or `packageManager`, the repository has no Node version file, and CI has no Node setup. Vite 6 still permits Node 18, so formatting/build may appear usable while the required test gate has a different hidden floor.
- Impact: two developers following the same documented setup can obtain an unsupported or failing test gate, and a future CI frontend lane cannot be reproduced from the commit without choosing an undeclared runtime.
- Root cause: the test dependency was upgraded without making the frontend runtime/toolchain an explicit versioned repository input or updating the operator prerequisite.
- Direction: select a currently supported Node line compatible with all locked tools, pin it in one standard runtime file plus `package.json.engines`/`packageManager`, use `npm ci` and that same runtime in CI, and update the prerequisite. Delete the contradictory Node 18 claim rather than keeping multiple floors.
- Regression validation: on the pinned runtime, fresh-clone `npm ci`, Prettier, tests, ordinary build, and Tauri-mode build all pass; an unsupported Node negative control fails immediately with a clear engine check.
- Validation reports: [V02](../validations/Q-WEB-01/V02-01.md), [V03](../validations/Q-WEB-01/V03-01.md), [V05](../validations/Q-WEB-01/V05-01.md), [V06](../validations/Q-WEB-01/V06-01.md), [V07](../validations/Q-WEB-01/V07-01.md)

## Canonical Owners

| Existing issue | Canonical owner | Q-WEB treatment |
|---|---|---|
| No frontend commands/Node setup in required CI | B-BASE-01-P2-01 | current; do not duplicate |
| No mounted DOM/Tauri/EventSource/WebSocket lifecycle harness | Q-TST-01-P1-01 | current; even a green `npm test` does not close it |
| Frontend DTO/reducer/runtime behavior defects | A-FE-01..03, A-SRF-03 | out of scope; gate should expose fixes, not redefine ownership |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Commit, dirty-state, dependencies and isolation | yes | passed | [V01](../validations/Q-WEB-01/V01-01.md) |
| V02 | Scripts/config/package-lock/Tauri topology | yes | passed | [V02](../validations/Q-WEB-01/V02-01.md) |
| V03 | Node/npm runtime compatibility and pinning | yes | failed | [V03](../validations/Q-WEB-01/V03-01.md) |
| V04 | Completed-owner deduplication | yes | passed | [V04](../validations/Q-WEB-01/V04-01.md) |
| V05 | Prettier check | yes | not_run by explicit constraint | [V05](../validations/Q-WEB-01/V05-01.md) |
| V06 | Frontend tests | yes | not_run by explicit constraint | [V06](../validations/Q-WEB-01/V06-01.md) |
| V07 | Ordinary production build | yes | not_run by explicit constraint | [V07](../validations/Q-WEB-01/V07-01.md) |
| V08 | Exact ID/link/executor/isolation integrity | yes | passed | [V08](../validations/Q-WEB-01/V08-01.md) |
| V30 | Primary static source sampling and acceptance | yes | passed | [V30](../validations/Q-WEB-01/V30-01.md) |

## Historical Claim Status

| Claim | Classification | Current evidence |
|---|---|---|
| B-BASE-01-P2-01: frontend is absent from CI | current | `.github/workflows/rust-ci.yml:10-45`; V04 |
| Q-TST-01-P1-01: no mounted transport/component harness | current | no DOM test dependency/config; V04 |
| B-BASE-01: package and lock define one frontend package | current | root dependency maps match; V02 |
| README: Node.js >=18 is sufficient for GUI frontend work | regressed | locked Vitest requires Node 20+; V03 |

## Coverage And Uncertainty

- No gate outcome is known. V05, V06, and V07 are separate `not_run` attempts, so the task correctly remains `needs_evidence`.
- Static inspection cannot detect formatting drift, TypeScript diagnostics, Vitest failures/unhandled rejections, missing native optional packages, or actual bundle warnings.
- The ordinary build required by Q-WEB and Tauri-mode build are distinct commands. Q-GUI-01 should own executable Tauri-mode validation.
- Existing suite credibility remains constrained by Q-TST-01-P1-01 even after `npm test` eventually passes.

## Handoff

- First fix Q-WEB-01-P2-01 by pinning a compatible Node/npm toolchain; otherwise later gate results are environment-dependent.
- Then execute V05, V06, and V07 as separate immutable attempts with exact exit codes. Keep the CI-lane fix under B-BASE-01-P2-01 and mounted-harness work under Q-TST-01-P1-01.
- Before promotion to `complete`, primary must reproduce V02/V03 and independently run the three executable gates. Any change to package/lock/config/workflow/README prerequisites makes this report stale.
- Primary independently reproduced the static Node/Vitest/CI evidence in V30. The task remains `needs_evidence` solely because V05-V07 were intentionally not run.
