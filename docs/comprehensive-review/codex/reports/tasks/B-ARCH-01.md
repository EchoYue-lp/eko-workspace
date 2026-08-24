# B-ARCH-01: Framework crate architecture

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d3`
> `echo-agent-cli` commit: `b3b2e81f2b2d`
> Worktree state: both source repositories clean

## Question

Are the eight `echo-agent` workspace members layered coherently, without
reverse dependencies or facade leakage?

## Scope

- All eight package manifests and crate roots.
- Root facade `src/lib.rs` and thin adapter modules for core, state, execution,
  integration, orchestration, and domain tools.
- `echo_execution` built-in skill dependency on `echo_tools`.
- Procedural macro crate-path resolution.
- Root and seven split-crate READMEs as architecture contracts.

## Out Of Scope

- Full public API quality and doctest execution (`F-API-01`).
- Feature/cfg compile matrix (`F-FEAT-01`, `Q-FW-02`).
- Internal correctness of ReAct, state, tasks, tools, and providers (their
  subsystem tasks).
- EKO adapter composition (`B-PATH-01`, cross-repository tasks).

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and `B-ARCH-01` card.
- Codex index `codex/README.md`.
- Corrected dependency report
  [B-BASE-01](B-BASE-01.md), especially attempts V01-02 through V03-02.
- No other reviewer report or historical audit was read.

## Layering Decision

- Generic mechanism: core contracts, implementations, provider integrations,
  state, orchestration, domain tools, macros, and the independent facade all
  belong to the reusable framework.
- EKO product policy: none is reviewed or moved in this task.
- Adapter boundary: root modules such as `memory`, `sandbox`, `workflow`, and
  `tools` are acceptable facade adapters when they remain thin; root-owned
  ReAct/evolution/DSL behavior is explicitly not treated as a re-export.
- Duplicate search: both repositories' manifests were covered by the baseline;
  within the framework, workspace crate names, macro-generated facade paths,
  public re-exports, duplicate Task/Plan/Workflow/Store/Tool identities, and all
  `echo_agent::workspace::*` call paths were searched. No new authority is
  proposed here.

## Current Path

The manifest DAG is:

```text
echo_core                 echo_macros
   ^                         ^
   |\________________________|___
   | \                       |   \
   |  echo_integration       |  echo_tools
   |  echo_state             |      ^
   |  echo_orchestration     |      |
   |_________________________| echo_execution
                 \              /
                  \            /
                    echo_agent
```

More precisely, facade `echo_agent` depends on all seven packages; execution
depends on core and tools; tools depends on core and macros; integration, state,
and orchestration depend on core. Cargo reports no cycle
([V01](../validations/B-ARCH-01/V01-01.md)).

Owner-to-facade mapping is mostly explicit: state-backed adapters at
`src/audit.rs:31`, `src/compression.rs:29`, and `src/memory.rs:44`; execution at
`src/sandbox.rs:27`, `src/skills/mod.rs:12`, and `src/tools/mod.rs:104`;
orchestration at `src/human_loop.rs:8`, `src/tasks.rs:8`, and
`src/workflow/mod.rs:61`; integration at `src/llm.rs:65`, `src/mcp.rs:8`, and
`src/channels.rs:61`; core contracts throughout root agent/error/retry/tokenizer
adapters. The facade also directly registers domain tools in the live ReAct
builder path (`src/agent/react/mod.rs:740`).

## Findings

### B-ARCH-01-P1-01: Attribute macros secretly require the facade package

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-macros/src/lib.rs:43`, `echo-macros/src/lib.rs:204`,
  `echo-macros/README.md:12`, `echo-macros/src/derive_tool.rs:34`
- Reachability: any external consumer following the split-crate README invokes
  `#[tool]`; expansion calls `echo_agent_crate_path()`, which requires a direct
  `echo_agent` dependency and aborts before generated code. The same resolver is
  used by callback, guard, handler, compressor, permission-policy, and audit
  attribute macros. Only derive(Tool) has a core-first fallback.
- Expected invariant: a published split macro crate either works with its
  documented `echo_macros` + `echo_core` dependencies or explicitly requires
  the facade.
- Observed behavior: the minimal valid consumer fails with “Could not find
  `echo_agent` in dependencies”.
- Impact: independent framework consumers cannot use seven advertised macros
  without adding the whole facade, defeating crate separation and causing
  compile-time facade leakage that Cargo's DAG cannot reveal.
- Root cause: attribute macros use a facade-only resolver while derive(Tool)
  independently evolved a correct core-first resolver.
- Direction: centralize crate-path resolution per generated contract. Generate
  against `echo_core` when traits live there and use orchestration only where
  required; retain a facade fallback for facade-only consumers. Delete the
  duplicate facade-only resolver.
- Regression validation: compile each macro from (1) facade-only, (2)
  core+macros, and where applicable (3) orchestration+core+macros probes, with
  renamed dependency cases.
- Validation reports: [V03-02](../validations/B-ARCH-01/V03-02.md)

### B-ARCH-01-P2-02: The documented workspace escape hatch omits two members

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `src/lib.rs:119`, `src/lib.rs:124`, `Cargo.toml:105`
- Reachability: `echo_agent::workspace` is public and used by
  `examples/demo70_scheduler.rs:8`; consumers seeking the documented direct
  split-crate path find five re-exports but no tools or macros.
- Expected invariant: a general “direct access to split workspace crates” API
  consistently exposes all direct split members, or documents an intentional
  subset.
- Observed behavior: `echo_tools` and `echo_macros` are omitted. A minimal
  `workspace::tools` import fails E0432.
- Impact: migration/import guidance is unpredictable, forcing consumers to mix
  facade adapters and direct dependencies based on undocumented exceptions.
- Root cause: the escape hatch was added as a partial migration list and did not
  track the complete workspace membership.
- Direction: decide whether this is stable API. If retained, expose all members
  with feature-appropriate names and tests; if migration-only, remove it after
  converting `demo70_scheduler` and document direct crate dependencies instead.
- Regression validation: compile one import per intended workspace member from
  a facade-only external probe.
- Validation reports: [V02](../validations/B-ARCH-01/V02-01.md),
  [V05](../validations/B-ARCH-01/V05-01.md)

### B-ARCH-01-P2-03: Minimal execution builds still depend on all base domain-tool code

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-execution/Cargo.toml:21`,
  `echo-execution/Cargo.toml:28`,
  `echo-execution/src/skills/builtin/filesystem.rs:5`,
  `echo-execution/src/skills/builtin/shell.rs:4`
- Reachability: every `echo_execution` consumer resolves non-optional
  `echo_tools`; the `files` and `shell` cfgs only hide built-in skill modules,
  not the package edge.
- Expected invariant: a no-default execution layer provides sandbox/registry
  mechanisms without pulling unrelated domain implementations.
- Observed behavior: `cargo tree -p echo_execution --no-default-features`
  includes `echo_tools`, which includes unconditional network/schema/tooling
  dependencies and `echo_macros`.
- Impact: independent execution consumers pay compile/dependency cost and the
  conceptual execution layer points outward to a domain leaf, making future
  tool reuse and feature isolation harder.
- Root cause: FileSystemSkill and ShellSkill wrappers are owned by execution
  while their implementations are owned by tools; the dependency was made
  unconditional even though both wrappers are feature-gated.
- Direction: first decide owner semantics in `F-SKL-01`. The smallest correction
  is an optional `echo_tools` dependency enabled by files/shell; a cleaner
  boundary may move domain-specific built-in skill wrappers beside their tools
  and leave execution with generic skill mechanisms. Delete the displaced
  wrappers when authority moves.
- Regression validation: assert `cargo tree -p echo_execution
  --no-default-features` contains no `echo_tools`, then compile files and shell
  features independently.
- Validation reports: [V01](../validations/B-ARCH-01/V01-01.md),
  [V06](../validations/B-ARCH-01/V06-01.md)

### B-ARCH-01-P2-04: Architecture READMEs are not a trustworthy public contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `README.md:104`, `README.md:121`, `README.md:281`,
  `README.md:388`, `echo-core/README.md:19`,
  `echo-tools/README.md:14`, `echo-integration/README.md:17`,
  `echo-macros/README.md:12`
- Reachability: these files are package `readme` metadata and the first consumer
  guidance shown by crates.io/repository users.
- Expected invariant: package READMEs name existing crates/features/imports and
  match Cargo metadata.
- Observed behavior: root docs list nonexistent `echo-agents`,
  `plan-execute`, and `self-reflection`, report conflicting 64/66 example counts
  versus 68 targets, and promise `full` enables everything despite six omitted
  flags. Split READMEs contain nonexistent imports and missing feature gates;
  the macros quickstart fails V03-02.
- Impact: consumers choose invalid dependencies/imports and cannot reliably
  infer the intended architecture from published documentation.
- Root cause: docs were maintained as feature marketing snapshots rather than
  generated/validated projections of the manifests and public API.
- Direction: after API decisions, regenerate the architecture/feature/target
  tables from metadata and make package quickstarts compile-tested examples.
  Remove nonexistent packages and features rather than preserving historical
  labels.
- Regression validation: compile every package README quickstart and add a
  metadata-driven doc drift check for member, feature, and example counts.
- Validation reports: [V04](../validations/B-ARCH-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Crate dependency graph | yes | passed | [V01](../validations/B-ARCH-01/V01-01.md) |
| V02 | Public facade mapping | yes | passed | [V02](../validations/B-ARCH-01/V02-01.md) |
| V03 | Attribute-macro compile-time reverse coupling | yes | failed after one inconclusive environmental attempt | [V03-01](../validations/B-ARCH-01/V03-01.md), [V03-02](../validations/B-ARCH-01/V03-02.md) |
| V04 | Current documentation comparison | yes | failed | [V04](../validations/B-ARCH-01/V04-01.md) |
| V05 | Workspace escape-hatch import probe | additional | failed | [V05](../validations/B-ARCH-01/V05-01.md) |
| V06 | Execution no-default dependency isolation | additional | failed | [V06](../validations/B-ARCH-01/V06-01.md) |
| V05 | Primary source/probe/finding acceptance | yes | passed | [V05](../validations/B-ARCH-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root `AGENTS.md`: root package plus seven subcrates | current | [V01](../validations/B-ARCH-01/V01-01.md) |
| Root README: 8 crates, 1 import | current only at package count; public facade is incomplete for direct workspace access | [V02](../validations/B-ARCH-01/V02-01.md) |
| Root README: `full` enables everything | stale | [V04](../validations/B-ARCH-01/V04-01.md) |
| Split README quickstarts describe current public API | regressed/stale | [V03-02](../validations/B-ARCH-01/V03-02.md), [V04](../validations/B-ARCH-01/V04-01.md) |

## Coverage And Uncertainty

The review mapped crate roots and facade adapters but did not enumerate every
public symbol. Duplicate Task/Plan/Workflow identities were searched only to
identify ownership boundaries; semantic duplicate-authority review belongs to
task/orchestration atomic tasks. Macro probes covered `#[tool]` as the shared
resolver representative, not each generated trait implementation. Documentation
sampling was architecture-focused.

## Handoff

- `F-API-01` should consume the facade map and validate stable public imports.
- `F-MAC-01` owns the complete macro matrix and should treat V03-02 as a
  confirmed external-consumer regression.
- `F-FEAT-01` should quantify execution/tools feature isolation and facade
  `full` drift.
- `F-SKL-01` must decide ownership before changing execution-to-tools layering.
- `B-DOC-01` should index, not blindly copy, the documentation drift anchors.
- This report becomes stale when workspace manifests, any crate root, macro
  path resolution, facade adapter modules, or architecture READMEs change.
