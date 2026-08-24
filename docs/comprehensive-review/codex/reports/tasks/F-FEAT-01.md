# F-FEAT-01: Feature topology and isolation

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean

## Question

Does each framework feature enable exactly its required code and dependencies,
including no-default and standalone use?

## Scope

- All eight framework manifests and normalized Cargo feature metadata.
- Every Rust `cfg(feature = "...")` occurrence in the eight packages.
- Facade public module/re-export boundaries associated with empty features.
- Facade optional dependencies and split-crate ownership.
- Every non-meta facade feature compiled independently with no defaults, plus
  `full`, all seven split-crate no-default libraries, and three `full` example
  selection probes.

## Out Of Scope

- Feature-specific runtime semantics and tests, owned by subsystem tasks.
- Public API naming/coherence beyond feature reachability (`F-API-01`).
- Dependency advisories, versions and licenses (`Q-DEP-01`).
- All-target/all-feature quality gate execution (`Q-FW-01`, `Q-FW-02`).
- EKO feature selection; this task reviews the reusable framework only.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and the
  `F-FEAT-01` task card.
- Codex review track rules in `codex/README.md`.
- Accepted Codex dependency report [B-BASE-01](B-BASE-01.md).
- Accepted Codex architecture report [B-ARCH-01](B-ARCH-01.md), used only to
  identify the known execution/tools edge for independent revalidation here.
- No other reviewer report was read.

## Layering Decision

- Generic mechanism: Cargo feature gates and dependency forwarding are public
  framework contracts. Owner crates should own implementation dependencies.
- EKO product policy: none; EKO-specific feature selection remains application
  scope.
- Adapter boundary: the root facade may forward owner features and re-export
  APIs, but should not duplicate implementation dependencies without direct
  facade code.
- Duplicate search: all eight manifests, all Rust cfgs, every root optional
  dependency name, split-crate feature forwarding, and example
  `required-features` were searched. Definitions, cfg effects, dependency
  effects, public reachability and compilation were distinguished.

## Current Path

Cargo metadata declares 65 package-feature entries: 35 on the facade, seven on
`echo_core`, three on `echo_execution`, 13 on `echo_tools`, four on
`echo_integration`, one on `echo_state`, two on `echo_orchestration`, and
none on `echo_macros`. Every Rust cfg name is declared in its owning manifest
([V01](../validations/F-FEAT-01/V01-01.md)).

Capability forwarding generally follows ownership: MCP/LSP/channels to
integration, SQLite to state, WebSocket human-loop to orchestration, and domain
tools to tools. The facade is not actually minimal, however: its unconditional
`echo_execution` dependency uses that crate's defaults, so facade
`--no-default-features` still enables execution `files` and `shell` and all
eight tree-sitter parser dependencies. At runtime, the React builder calls
`echo_tools::register_all_tools` at `src/agent/react/mod.rs:740-746`;
capability availability is therefore determined by the resolved tools features,
not merely the facade's displayed default list.

Six facade compatibility flags have no dependency or cfg effect even though ten
examples require them. Conversely, the APIs they name are unconditional:
`sandbox` and `workflow` modules at `src/lib.rs:54,65`, macros at
`src/lib.rs:111-117`, provider factory at `src/llm.rs:105`, and memory/
multimodal types through unconditional public adapters. The `full` list at
`Cargo.toml:67` omits these six plus `testing`, making 17 example targets
unselectable under `full` ([V02](../validations/F-FEAT-01/V02-01.md)).

All 33 non-meta facade features compile independently, and all seven split
libraries compile with no defaults. Compilation therefore finds no standalone
type/cfg break, but it does not validate the semantic topology defects below.

## Findings

### F-FEAT-01-P2-01: Facade no-default silently enables file and shell domains

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `Cargo.toml:66`, `Cargo.toml:108`;
  `echo-execution/Cargo.toml:21-28`; `src/skills/mod.rs:12-16`;
  `src/tools/mod.rs:57-80`
- Reachability: every facade build selects the non-optional `echo_execution`
  dependency with its defaults; execution defaults enable `files` and `shell`,
  which forward to `echo_tools`. The facade then unconditionally re-exports
  FileSystemSkill, ShellSkill, files and shell APIs.
- Expected invariant: `echo_agent = { default-features = false }` provides the
  documented minimal ReAct engine and excludes optional file/shell domains.
- Observed behavior: the no-default dependency tree includes execution
  `files`/`shell`, encoding, and seven tree-sitter packages. Enabling facade
  `files` or `shell` does not transition these capabilities from absent to
  present.
- Impact: consumers cannot obtain the advertised small facade build, feature
  audits misstate file/shell availability, and compile/dependency cost is paid
  even when those capabilities were intentionally disabled.
- Root cause: the facade did not set `default-features = false` on its split
  execution dependency or explicitly forward its own files/shell flags.
- Direction: disable execution defaults at the facade edge and forward facade
  files/shell to both execution built-in skills and tools ownership as needed.
  Coordinate with `F-SKL-01`; delete any duplicate wrappers if ownership moves.
- Regression validation: assert the no-default tree contains neither execution
  files/shell features nor tree-sitter dependencies; compile and import the
  intended files and shell APIs only after their individual flags are enabled.
- Validation reports: [V04](../validations/F-FEAT-01/V04-01.md),
  [V19](../validations/F-FEAT-01/V19-01.md),
  [V25](../validations/F-FEAT-01/V25-01.md),
  [V35](../validations/F-FEAT-01/V35-01.md)

### F-FEAT-01-P2-02: Six public feature flags are no-op example selectors

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `Cargo.toml:98-103`, `Cargo.toml:179-236`,
  `Cargo.toml:299-302`; `src/lib.rs:54-65`, `src/lib.rs:111-117`;
  `src/llm.rs:99-105`; `src/memory.rs:42-47`
- Reachability: `workflow`, `sandbox`, `semantic-memory`, `macros`,
  `provider-factory`, and `multimodal` have empty feature definitions, no
  matching Rust cfgs, and identical dependency trees. Their APIs are already
  public in no-default builds, while ten examples are hidden until callers name
  the no-op flags.
- Expected invariant: enabling a public feature changes compiled code or
  dependencies, and an example gate represents an actual capability
  prerequisite.
- Observed behavior: these flags change only Cargo target selection.
- Impact: consumers believe they can exclude costly or sensitive capabilities
  but cannot, while valid examples appear unavailable under otherwise
  capability-equivalent builds. The feature API is misleading and increases
  unsupported combinations with no isolation benefit.
- Root cause: historical feature names survived after capability code became
  unconditional, while example metadata was not migrated.
- Direction: decide per capability whether it is truly core. Delete no-op flags
  and their artificial example gates for core APIs; for genuinely optional
  capabilities, add real module/dependency gates and compile-import tests. Do
  not retain compatibility aliases because this project requires no backward
  compatibility.
- Regression validation: for each retained feature, prove an API fails to
  import without it and compiles with it; metadata must contain no empty
  non-marker feature whose only consumer is `required-features`.
- Validation reports: [V01](../validations/F-FEAT-01/V01-01.md),
  [V02](../validations/F-FEAT-01/V02-01.md),
  [V28](../validations/F-FEAT-01/V28-01.md),
  [V29](../validations/F-FEAT-01/V29-01.md),
  [V31](../validations/F-FEAT-01/V31-01.md),
  [V33](../validations/F-FEAT-01/V33-01.md),
  [V34](../validations/F-FEAT-01/V34-01.md),
  [V41](../validations/F-FEAT-01/V41-01.md),
  [V48](../validations/F-FEAT-01/V48-01.md)

### F-FEAT-01-P2-03: The full meta-feature cannot select 17 official examples

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `Cargo.toml:67`; `Cargo.toml:163-302`; `README.md:104-115`
- Reachability: Cargo resolves `full` successfully for the library, then rejects
  example targets whose required set contains one of the seven omitted flags.
  Metadata computes 17 affected examples. V18, V47 and V48 independently
  reproduce semantic-memory, testing, and macros classes.
- Expected invariant: a feature documented as “get everything” selects every
  shipped capability/example, or its exclusions are explicit and principled.
- Observed behavior: `full` omits `macros`, `multimodal`,
  `provider-factory`, `sandbox`, `semantic-memory`, `testing`, and
  `workflow`. README also contradicts the manifest about default/full and
  individual inclusion.
- Impact: consumers following the main installation path cannot compile 25% of
  the 68 official examples without diagnosing hidden extra flags, and CI using
  all-features can conceal the broken `full` contract.
- Root cause: `full`, feature documentation, and target metadata are manually
  maintained independent lists.
- Direction: first resolve P2-02 and classify test-only examples. Then derive or
  validate `full` against the intentional feature inventory and add a Cargo
  metadata invariant that every non-test official example's required features
  are a subset of `full`.
- Regression validation: run the metadata subset assertion and
  `cargo check -p echo_agent --examples --no-default-features --features full`.
- Validation reports: [V02](../validations/F-FEAT-01/V02-01.md),
  [V17](../validations/F-FEAT-01/V17-01.md),
  [V18](../validations/F-FEAT-01/V18-01.md),
  [V47](../validations/F-FEAT-01/V47-01.md),
  [V48](../validations/F-FEAT-01/V48-01.md)

### F-FEAT-01-P2-04: Facade duplicates ten optional dependencies owned by tools

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `Cargo.toml:81-89`, `Cargo.toml:151-161`;
  `echo-tools/Cargo.toml:18-37`, `echo-tools/Cargo.toml:56-76`;
  `src/tools/mod.rs:82-98`
- Reachability: enabling web, media, data, statistics or database activates both
  root optional dependencies and the corresponding `echo_tools` feature. All
  implementation uses are in `echo_tools`; facade Rust targets contain no
  direct references to the ten crates.
- Expected invariant: the implementation owner declares optional third-party
  dependencies; the facade only forwards the owner's feature unless it has its
  own direct implementation.
- Observed behavior: root duplicates scraper/html2text/url,
  pdf-extract/lopdf/calamine/docx-rs/encoding_rs, polars and sqlx.
- Impact: dependency ownership is ambiguous, feature/version updates require
  synchronized duplicate manifest edits, and facade features can accidentally
  diverge from owner features. Cargo unification may avoid duplicate artifacts
  today but does not remove the maintenance coupling.
- Root cause: optional dependencies remained in the facade after domain
  implementations moved to the tools crate.
- Direction: remove the ten facade dependency declarations and `dep:` edges,
  retaining only `echo_tools/<feature>`. Keep direct root dependencies such as
  rusqlite where facade-owned code actually uses them.
- Regression validation: standalone feature checks V12/V15/V16/V23/V36 still
  pass, and a metadata assertion confirms no root optional dependency lacks a
  direct facade target reference.
- Validation reports: [V03](../validations/F-FEAT-01/V03-01.md),
  [V12](../validations/F-FEAT-01/V12-01.md),
  [V15](../validations/F-FEAT-01/V15-01.md),
  [V16](../validations/F-FEAT-01/V16-01.md),
  [V23](../validations/F-FEAT-01/V23-01.md),
  [V36](../validations/F-FEAT-01/V36-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Declared features versus all source cfgs | yes | failed | [V01](../validations/F-FEAT-01/V01-01.md) |
| V02 | No-op reachability, examples and `full` subset | yes | failed | [V02](../validations/F-FEAT-01/V02-01.md) |
| V03 | Optional dependency ownership/leakage | yes | failed | [V03](../validations/F-FEAT-01/V03-01.md) |
| V04 | Facade no-default library check | yes | passed | [V04](../validations/F-FEAT-01/V04-01.md) |
| V05-V16 | Mandatory 12-feature standalone matrix | yes | passed | [V05](../validations/F-FEAT-01/V05-01.md), [V06](../validations/F-FEAT-01/V06-01.md), [V07](../validations/F-FEAT-01/V07-01.md), [V08](../validations/F-FEAT-01/V08-01.md), [V09](../validations/F-FEAT-01/V09-01.md), [V10](../validations/F-FEAT-01/V10-01.md), [V11](../validations/F-FEAT-01/V11-01.md), [V12](../validations/F-FEAT-01/V12-01.md), [V13](../validations/F-FEAT-01/V13-01.md), [V14](../validations/F-FEAT-01/V14-01.md), [V15](../validations/F-FEAT-01/V15-01.md), [V16](../validations/F-FEAT-01/V16-01.md) |
| V17 | Facade `full` library check | yes | passed | [V17](../validations/F-FEAT-01/V17-01.md) |
| V18 | `full` semantic-memory example check | yes | failed | [V18](../validations/F-FEAT-01/V18-01.md) |
| V19-V20 | Execution/tools no-default checks | yes | passed | [V19](../validations/F-FEAT-01/V19-01.md), [V20](../validations/F-FEAT-01/V20-01.md) |
| V21-V41 | Remaining facade standalone feature checks | yes | passed | [V21](../validations/F-FEAT-01/V21-01.md), [V22](../validations/F-FEAT-01/V22-01.md), [V23](../validations/F-FEAT-01/V23-01.md), [V24](../validations/F-FEAT-01/V24-01.md), [V25](../validations/F-FEAT-01/V25-01.md), [V26](../validations/F-FEAT-01/V26-01.md), [V27](../validations/F-FEAT-01/V27-01.md), [V28](../validations/F-FEAT-01/V28-01.md), [V29](../validations/F-FEAT-01/V29-01.md), [V30](../validations/F-FEAT-01/V30-01.md), [V31](../validations/F-FEAT-01/V31-01.md), [V32](../validations/F-FEAT-01/V32-01.md), [V33](../validations/F-FEAT-01/V33-01.md), [V34](../validations/F-FEAT-01/V34-01.md), [V35](../validations/F-FEAT-01/V35-01.md), [V36](../validations/F-FEAT-01/V36-01.md), [V37](../validations/F-FEAT-01/V37-01.md), [V38](../validations/F-FEAT-01/V38-01.md), [V39](../validations/F-FEAT-01/V39-01.md), [V40](../validations/F-FEAT-01/V40-01.md), [V41](../validations/F-FEAT-01/V41-01.md) |
| V42-V46 | Remaining owner-crate no-default checks | yes | passed | [V42](../validations/F-FEAT-01/V42-01.md), [V43](../validations/F-FEAT-01/V43-01.md), [V44](../validations/F-FEAT-01/V44-01.md), [V45](../validations/F-FEAT-01/V45-01.md), [V46](../validations/F-FEAT-01/V46-01.md) |
| V47 | `full` testing-gated example check | yes | failed | [V47](../validations/F-FEAT-01/V47-01.md) |
| V48 | `full` no-op macros example check | yes | failed | [V48](../validations/F-FEAT-01/V48-01.md) |
| V49-V72 | Every non-meta split-crate feature standalone check | yes | passed | [V49](../validations/F-FEAT-01/V49-01.md), [V50](../validations/F-FEAT-01/V50-01.md), [V51](../validations/F-FEAT-01/V51-01.md), [V52](../validations/F-FEAT-01/V52-01.md), [V53](../validations/F-FEAT-01/V53-01.md), [V54](../validations/F-FEAT-01/V54-01.md), [V55](../validations/F-FEAT-01/V55-01.md), [V56](../validations/F-FEAT-01/V56-01.md), [V57](../validations/F-FEAT-01/V57-01.md), [V58](../validations/F-FEAT-01/V58-01.md), [V59](../validations/F-FEAT-01/V59-01.md), [V60](../validations/F-FEAT-01/V60-01.md), [V61](../validations/F-FEAT-01/V61-01.md), [V62](../validations/F-FEAT-01/V62-01.md), [V63](../validations/F-FEAT-01/V63-01.md), [V64](../validations/F-FEAT-01/V64-01.md), [V65](../validations/F-FEAT-01/V65-01.md), [V66](../validations/F-FEAT-01/V66-01.md), [V67](../validations/F-FEAT-01/V67-01.md), [V68](../validations/F-FEAT-01/V68-01.md), [V69](../validations/F-FEAT-01/V69-01.md), [V70](../validations/F-FEAT-01/V70-01.md), [V71](../validations/F-FEAT-01/V71-01.md), [V72](../validations/F-FEAT-01/V72-01.md) |
| V73 | Handoff Cargo-session/build-lock probe | yes | passed | [V73](../validations/F-FEAT-01/V73-01.md) |
| V74 | Primary metadata count | yes | passed after failed/inconclusive attempts | [V74 attempt 01](../validations/F-FEAT-01/V74-01.md), [attempt 02](../validations/F-FEAT-01/V74-02.md), [attempt 03](../validations/F-FEAT-01/V74-03.md), [attempt 04](../validations/F-FEAT-01/V74-04.md), [attempt 05](../validations/F-FEAT-01/V74-05.md), [attempt 06](../validations/F-FEAT-01/V74-06.md) |
| V75 | Primary `full`-to-example subset computation | yes | failed invariant | [V75](../validations/F-FEAT-01/V75-01.md) |
| V76 | Primary no-default dependency-tree trace | yes | failed invariant | [V76](../validations/F-FEAT-01/V76-01.md) |
| V77 | Primary facade dependency-ownership search | yes | passed after noisy attempt | [V77 attempt 01](../validations/F-FEAT-01/V77-01.md), [attempt 02](../validations/F-FEAT-01/V77-02.md), [attempt 03](../validations/F-FEAT-01/V77-03.md), [attempt 04](../validations/F-FEAT-01/V77-04.md) |
| V78 | Primary feature/source gate sampling | yes | passed | [V78](../validations/F-FEAT-01/V78-01.md) |
| V79 | Primary facade no-default compile | yes | passed | [V79](../validations/F-FEAT-01/V79-01.md) |
| V80 | Primary `full` example selection probe | yes | failed invariant | [V80](../validations/F-FEAT-01/V80-01.md) |
| V81 | Final report links/executor/source-isolation gate | yes | passed after failed attempt | [V81 attempt 01](../validations/F-FEAT-01/V81-01.md), [attempt 02](../validations/F-FEAT-01/V81-02.md) |

Every command in a range has its own immutable report; the compact range rows do
not combine executions.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AGENTS.md`: standalone matrix features must compile | current | V05-V16 all pass |
| Root README: zero defaults give minimal footprint | regressed | [V04](../validations/F-FEAT-01/V04-01.md), P2-01 |
| Root README: `full` gets everything | regressed | [V02](../validations/F-FEAT-01/V02-01.md), [V18](../validations/F-FEAT-01/V18-01.md) |
| B-ARCH-01: execution no-default still pulls tools | current | [V19](../validations/F-FEAT-01/V19-01.md), P2-01 |
| B-ARCH-01: README full omissions | current and expanded | P2-03 quantifies seven omitted flags and 17 examples |

## Coverage And Uncertainty

All 33 non-meta facade features and all 24 non-meta split-crate features received
independent no-default library compilation; all seven split libraries also
received pure no-default compilation. Example checks sampled three distinct
omission classes rather than compiling all 17 expected Cargo selection failures.
Runtime feature semantics remain owned by subsystem tasks. No source was
modified.

The task remains `needs_evidence` until primary review independently samples
feature counts, no-op classification, and the four findings.

## Handoff

- `F-API-01` should treat no-op features as misleading public API rather than
  evidence that the underlying APIs are optional.
- `F-SKL-01` should resolve files/shell built-in ownership before P2-01 is
  implemented.
- `Q-FW-02` should add metadata assertions for cfg/feature closure, `full`
  example coverage, and no-default dependency exclusion.
- `B-DOC-01` can rely on the README contradictions but should independently
  review broader documentation.
- This report becomes stale when any framework manifest, source cfg, example
  required-feature list, facade public module/re-export, or reviewed commit
  changes.
