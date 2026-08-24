# Q-DOC-01: Current public and operator documentation validation

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework had unrelated dirty source and was inspected only through committed Git objects; CLI had only unrelated `Cargo.lock` modification, excluded

## Question

Do README, feature/config references, examples, EKO setup docs, and architecture claims match reviewed code and executable commands?

## Scope

- Framework root README, bilingual documentation indexes, getting-started/config chapters, example inventory, example target/feature declarations, and local navigation targets.
- EKO README, getting-started/configuration/architecture/gui-status guides, current command-line arguments, canonical data-root/config discovery, Cargo features, and registered GUI commands.
- Static classification of command examples and current versus historical documents at the pinned commits.

## Out Of Scope

- Running Cargo, rustc, tests, builds, Clippy, frontend commands, dynamic fixtures, or network checks, explicitly prohibited for this task.
- Re-reviewing runtime behavior owned by application/framework atomic tasks.
- Editing documentation, source, shared indexes, or external dirty files.
- Treating EKO's lack of SQLite as grounds to remove the framework's valid optional SQLite capability.

## Inputs

- Root `AGENTS.md`, shared comprehensive-review README/REPORTING/TASKS card, and Codex README.
- Codex [B-DOC-01](B-DOC-01.md), [F-API-01](F-API-01.md), and completed application reports relevant to current operator claims: A-BOOT-01, A-CFG-01, A-MEM-01, A-INP-01, A-PLG-01, A-INT-01, A-TOOL-01, A-OUT-01, A-SRF-01 through A-SRF-04, and A-FE-01.
- Only committed source/docs from the two pinned repositories; no other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework public API, feature flags, examples, and split-crate/facade installation contracts belong to `echo-agent`; optional SQLite remains valid there. |
| EKO product policy | Canonical `~/.eko` paths, no SQLite, local-personal threat model, surface parity, startup commands, and GUI status belong to EKO documentation. |
| Adapter boundary | Operator docs may explain trigger/render differences, but must not invent an application CLI that has no adapter or hide capability gaps as product intent. |
| Duplicate search | Searched command/flag names, example targets/features, Markdown paths, data roots, SQLite, worker/Subagent vocabulary, current/stale status labels, EventBus/context names, and registered GUI commands. Definitions and manifest entries were distinguished from prose. |
| Migration deletion | Retain useful framework APIs and dated historical evidence. Remove or rewrite current-facing false commands/claims; replace duplicate current status documents with generated/canonical tables rather than another hand-maintained authority. |

## Current Path

```text
Framework consumer
  -> README quick start / docs indexes / examples README
  -> Cargo.toml dependency + feature + [[example]] contract
  -> facade/split-crate public API

EKO operator
  -> README / getting-started / configuration / gui-status
  -> Cargo aliases or Args parser
  -> main mode selection
  -> ~/.eko and project .eko discovery
  -> Tauri command registration + frontend invoke surface
```

Static source gives authoritative anchors where prose conflicts: framework `Cargo.toml:65-103,198-240`; EKO `src/cli/args.rs:7-68`, `src/main.rs:64-111,347-420`, `echo-agent-app-core/src/config_discovery.rs:8-17,184-280,396`, EKO manifests, and `src/tauri/mod.rs:248-284`.

## Findings

### Q-DOC-01-P2-01: EKO getting-started documents a command-line product that does not exist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/docs/getting-started.md:90`; `echo-agent-cli/docs/getting-started.md:125`; `echo-agent-cli/src/cli/args.rs:7`; `echo-agent-cli/src/main.rs:94`
- Reachability: this is the current getting-started path for CLI users; Clap parses one flat `Args` structure before selecting TUI or hidden internal modes.
- Expected invariant: published operator commands are accepted by the current binary and reach the described operation.
- Observed behavior: the guide prescribes `run`, `--headless`, and `sessions` subcommands, but current Args defines none of them and has no subcommand parser. There is no noninteractive EKO Agent/event contract, canonical under A-SRF-04-P1-06.
- Impact: first-run CLI and automation users cannot execute the documented basic interaction, JSON/headless, or session-management workflows.
- Root cause: framework headless and an older CLI shape were copied into EKO product docs without a command-to-parser reachability gate.
- Direction: remove unusable commands immediately; after A-SRF-04 defines EKO's noninteractive adapter, generate CLI docs from Clap and add end-to-end command examples. Do not present hidden legacy `--cli` as the missing typed automation contract.
- Regression validation: snapshot generated `--help`, then execute every published command with deterministic fake/provider fixtures and assert output/exit contracts.
- Validation reports: [V06](../validations/Q-DOC-01/V06-01.md), [V10](../validations/Q-DOC-01/V10-01.md)

### Q-DOC-01-P2-02: Framework feature and example documentation is not an executable contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/README.md:92`; `echo-agent/README.md:102`; `echo-agent/README.md:233`; `echo-agent/Cargo.toml:65`; `echo-agent/Cargo.toml:198`; `echo-agent/examples/README.md:11`
- Reachability: root README is package rustdoc and the normal install/run entry; examples/README declares which examples serve acceptance.
- Expected invariant: feature membership, default behavior, example commands, and example inventory agree with the manifest/tree.
- Observed behavior: README omits required features for three advertised commands, contradicts itself about whether `full` is default, misstates full membership, and the examples inventory names four absent files.
- Impact: consumers receive skipped/failed example targets and cannot trust the stated acceptance surface or dependency footprint.
- Root cause: feature tables, commands, and example classification are manually duplicated instead of derived from Cargo metadata/tree state.
- Direction: make the manifest authoritative; generate or statically compare README feature/example tables and delete absent inventory entries. Keep the prelude/API defects canonical in F-API-01 rather than expanding this finding.
- Regression validation: a docs gate parses Cargo metadata, validates local files, and executes each exact published command under its declared prerequisites.
- Validation reports: [V02](../validations/Q-DOC-01/V02-01.md), [V03](../validations/Q-DOC-01/V03-01.md), [V04](../validations/Q-DOC-01/V04-01.md), [V10](../validations/Q-DOC-01/V10-01.md)

### Q-DOC-01-P2-03: EKO setup and architecture guides point operators at obsolete roots and forbidden SQLite storage

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/README.md:99`; `echo-agent-cli/README.md:233`; `echo-agent-cli/README.md:527`; `echo-agent-cli/docs/configuration.md:3`; `echo-agent-cli/docs/architecture.md:157`; `echo-agent-cli/src/main.rs:64`; `echo-agent-cli/echo-agent-app-core/src/config_discovery.rs:8`; `echo-agent-cli/Cargo.toml:48`; `echo-agent-cli/echo-agent-app-core/Cargo.toml:9`
- Reachability: these are current setup/config/architecture documents used before startup; startup fixes the brand root before path resolution and manifests select application persistence features.
- Expected invariant: one canonical EKO root and storage matrix guides file placement; CLI documentation never claims SQLite is enabled.
- Observed behavior: docs repeatedly prescribe `~/.echo-agent` and `./.echo-agent` while current EKO selects/discovers `~/.eko` and `.eko`; README/architecture claim SQLite sessions/default memory and list `sqlite` as enabled even though neither EKO manifest enables it.
- Impact: users can put configuration, MCP credentials, memory, LSP files, and workspace data where current EKO does not canonically discover them, and maintainers are sent toward a prohibited CLI storage dependency.
- Root cause: framework defaults and pre-brand EKO paths were copied into application docs, while path and persistence selections changed independently.
- Direction: after A-CFG-01 precedence is resolved, publish one generated root/path matrix; describe EKO's file-backed stores explicitly. Preserve and clearly label SQLite only in framework docs for other consumers.
- Regression validation: table-driven config discovery using every published path plus a manifest gate asserting CLI dependency closure excludes `echo-agent/sqlite`.
- Validation reports: [V07](../validations/Q-DOC-01/V07-01.md), [V08](../validations/Q-DOC-01/V08-01.md)

### Q-DOC-01-P2-04: EKO's linked current GUI status contradicts its root README

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/README.md:213`; `echo-agent-cli/docs/gui-status.md:32`; `echo-agent-cli/src/tauri/mod.rs:248`; `echo-agent-cli/src/tauri/commands/research.rs:40`; `echo-agent-cli/web-frontend/src/api/endpoints.ts:339`
- Reachability: root README links gui-status as the detailed matrix; registered commands and frontend invocations are the production GUI definition/registration path.
- Expected invariant: linked current documents agree on whether a feature is absent, partial, or connected.
- Observed behavior: README says workflow execution is not wired and papers IPC is missing, while the same commit registers/implements both command families, frontend callers exist, and gui-status calls them wired/connected.
- Impact: users and reviewers cannot tell which GUI capabilities exist, causing duplicate work or incorrect release claims.
- Root cause: two manually maintained current capability summaries drift independently.
- Direction: retain one generated definition-registration-reachability surface matrix and make README link to it without copying status prose. Runtime correctness stays owned by A-SRF/A-FE tasks.
- Regression validation: compare every capability row with command registration and a frontend/TUI/CLI/channel consumer; distinguish connected from correct and from parity-complete.
- Validation reports: [V09](../validations/Q-DOC-01/V09-01.md)

### Q-DOC-01-P3-01: Current navigation contains repository-local dead links

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/README.md:1170`; `echo-agent/README.md:1173`; `echo-agent/docs/en/README.md:75`; `echo-agent/docs/zh/README.md:73`; `echo-agent-cli/README.md:6`; `echo-agent-cli/docs/getting-started.md:239`
- Reachability: these links are in current root/index/onboarding documents and are clicked before source exploration.
- Expected invariant: every local link resolves inside the repository that publishes the document.
- Observed behavior: framework indexes link six absent bilingual pages; EKO links an absent LICENSE and a sibling-repository path that cannot resolve in a standalone clone.
- Impact: public navigation and licensing/framework-learning handoffs fail.
- Root cause: no repository-relative link gate and local multi-repository workspace paths were treated as published URLs.
- Direction: remove or restore absent pages; use a stable external framework URL from EKO; add a scheme-aware local link check.
- Regression validation: parse every current Markdown local target and fragment, resolve relative to its document and committed tree, and fail on absent targets.
- Validation reports: [V05](../validations/Q-DOC-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Commit, dirty-state, dependency and isolation boundary | yes | passed | [V01](../validations/Q-DOC-01/V01-01.md) |
| V02 | Framework README example-command feature contract | yes | failed | [V02](../validations/Q-DOC-01/V02-01.md) |
| V03 | Framework feature/config matrix | yes | failed | [V03](../validations/Q-DOC-01/V03-01.md) |
| V04 | Framework example inventory paths | yes | failed | [V04](../validations/Q-DOC-01/V04-01.md) |
| V05 | Current local link/path checks | yes | failed | [V05](../validations/Q-DOC-01/V05-01.md) |
| V06 | EKO CLI command-to-Args reachability | yes | failed | [V06](../validations/Q-DOC-01/V06-01.md) |
| V07 | EKO config/data path matrix | yes | failed | [V07](../validations/Q-DOC-01/V07-01.md) |
| V08 | EKO SQLite/product storage boundary | yes | failed | [V08](../validations/Q-DOC-01/V08-01.md) |
| V09 | GUI status definition-registration-consumer matrix | yes | failed | [V09](../validations/Q-DOC-01/V09-01.md) |
| V10 | Command/example execution sampling | yes | not_run by explicit constraint | [V10](../validations/Q-DOC-01/V10-01.md) |
| V11 | Stale terminology/architecture and historical classification | yes | failed | [V11](../validations/Q-DOC-01/V11-01.md) |
| V12 | Exact ID/header/link/executor/isolation integrity | yes | passed | [V12](../validations/Q-DOC-01/V12-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [V30](../validations/Q-DOC-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-DOC-01 competing/current-document drift | current | [V11](../validations/Q-DOC-01/V11-01.md); current operator docs still conflict |
| F-API-01 removed EventBus/context APIs and split-crate imports | current | committed `git grep` reproduced the names/imports; canonical findings remain F-API-01-P2-01 through P2-03 |
| EKO README “workflow execute/papers absent” | stale | [V09](../validations/Q-DOC-01/V09-01.md) |
| EKO `~/.echo-agent` and SQLite architecture | stale | [V07](../validations/Q-DOC-01/V07-01.md), [V08](../validations/Q-DOC-01/V08-01.md) |
| Framework examples inventory as acceptance catalog | stale | [V04](../validations/Q-DOC-01/V04-01.md) |

## Coverage And Uncertainty

- No published command, code block, frontend command, external URL, or anchor was executed. V10 records this as `not_run`; per the explicit review constraint this remains a future regression gate and does not block the source-conclusive static review.
- Link sampling covered current root/onboarding/index families and all detected failures there, not every one of 154 Markdown files or every fragment.
- Date-prefixed EKO design documents were treated as historical by filename and were not exhaustively revalidated. Current-looking README/config/architecture/master/status documents were prioritized.
- Framework implementation defects remain owned by F-API-01/F-FEAT tasks; EKO runtime defects remain owned by A-CFG/A-SRF/A-OUT/etc. This task owns only false or irreconcilable documentation contracts.

## Handoff

- Primary reproduced the static link/manifest/Args comparisons in V30. Dynamic command/example sampling remains a roadmap validation before documentation fixes are accepted.
- Fix order: Q-DOC-01-P2-01 and P2-03 operator hazards; Q-DOC-01-P2-02 generated feature/example contract; Q-DOC-01-P2-04 single surface matrix; Q-DOC-01-P3-01 local links.
- Read [B-DOC-01](B-DOC-01.md), [F-API-01](F-API-01.md), A-CFG-01 and A-SRF-04 before implementing so documentation does not canonize known defective runtime behavior.
- This report becomes stale if either pinned HEAD changes any referenced docs, manifests, Args/config discovery, Tauri registration, or frontend capability callers.

## Primary Acceptance

The primary reviewer independently sampled the framework README-to-manifest
feature/example contract, absent example and local-link targets, EKO
getting-started commands against the flat Clap `Args`, canonical `.eko` path
discovery, no-SQLite manifests, and GUI command registration/frontend consumers.
All five findings and their ownership boundaries were accepted in V30.
