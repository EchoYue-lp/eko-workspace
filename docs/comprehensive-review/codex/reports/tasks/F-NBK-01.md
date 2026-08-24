# F-NBK-01: Notebook and structured working artifacts

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex review reports added outside the source repositories

## Question

Is Notebook a coherent, reachable framework API with stable cell/artifact semantics rather than an isolated or aspirational path?

## Scope

- `echo-agent/src/notebook/mod.rs`: public cell/tracker, ordering, synchronization, summaries, lifecycle, and Markdown/JSON exports.
- Root public export and `AgentConfig::enable_notebook` from definition through production readers.
- Canonical Tool call/result/error events and text artifact identity, only for field-by-field mapping and duplicate-authority control.
- Existing tests, examples, rustdoc, scoped history, and the prior Notebook poison-lock audit claim.
- Bounded EKO source/doc search only to distinguish its artifact label and file-backed product workbench from the independent framework API.

## Out Of Scope

- Source fixes, public API migration, report-index changes, or generated files.
- Cargo, rustc, tests, builds, dynamic fixtures, benchmarks, or network research.
- Tool execution defects owned by `F-EXT-01`, ReAct loop/batch semantics, EKO workbench behavior, and final artifact UX.
- `structured.rs` final-response JSON parsing: it is a typed final-output wrapper, not a Notebook authority.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex `README.md`; task and validation templates.
- Completed dependency reports [F-API-01](F-API-01.md) and [F-EXT-01](F-EXT-01.md).
- Completed [F-RCT-01](F-RCT-01.md), read only to de-duplicate its accepted no-op `enable_notebook` finding.
- Current source, bounded application references, and scoped Notebook history. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | A reusable, versioned projection from canonical tool executions to inspectable cells, including identity/outcome/artifact references and bounded retention/sink semantics, reasonably serves unrelated framework consumers and belongs in `echo-agent`. |
| EKO product policy | Notebook/workbench UI, file layout, lineage/staleness policy, report rendering, and TaskRuntime `ArtifactKind` remain EKO application concerns. EKO's current non-use is not a reason to delete a reasonable public framework tracker. |
| Adapter boundary | An EKO adapter may persist/render the framework projection and inject product metadata. It must not own a second tool-call identity, execution order, terminal status, or artifact writer. |
| Duplicate search | Searched both repositories for Notebook cell/tracker/config/record/export symbols; Tool call/result/error identity; artifact refs; persistence/import/retention; examples/tests/docs. One Notebook tracker exists. Canonical execution facts already exist in `RunEvent` and `ToolOutputArtifactRef`; EKO only has a product artifact-kind label. |
| Migration deletion | Do not add a parallel recorder. Project canonical Tool events/artifacts into Notebook cells; delete/demote the free-form Agent-internal recording path if replaced. Either wire `enable_notebook` to that owner or delete only the false Agent option/docs while retaining the independently useful public tracker. |

## Current Path

```text
public standalone path
  echo_agent::notebook
    -> NotebookTracker::new
    -> record_cell(free-form strings, duration)
    -> Vec<NotebookCell> under RwLock
    -> cells clone | Markdown String | JSON String

advertised Agent path
  AgentConfig::enable_notebook(bool)
    -> stored private field
    -> no constructor/runtime reader
    -> no tracker owner, record call, accessor, or persistence

canonical live Tool path
  ExecuteStage
    -> stable call_id + ToolCall(args)
    -> ToolManager execution/output guard/artifact
    -> ToolResult(success, sizes, handling, artifact) [+ ToolError]
    -> trace/model/event consumers
  (no projection into Notebook)
```

The standalone API is publicly reachable through `src/lib.rs:50`; it is not dead merely because EKO does not construct it. The Agent option's lack of a reader is already owned by F-RCT-01-P2-05 and is not re-numbered here.

Positive robustness facts: `record_cell` truncates input/output with Unicode-safe `chars().take(200)`, assigns contiguous indexes while holding the write lock, and recovers poisoned locks without panic (`src/notebook/mod.rs:51-60`). The old poison-panic audit claim is fixed.

## Findings

### F-NBK-01-P2-01: Notebook cells cannot correlate or reproduce canonical tool executions

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/notebook/mod.rs:13`, `echo-agent/src/notebook/mod.rs:43`, `echo-agent/src/trace/mod.rs:302`, `echo-agent/src/agent/react/run/pipeline.rs:475`, `echo-agent/src/agent/react/run/pipeline.rs:826`, `echo-agent/echo-core/src/tools/artifact.rs:69`
- Reachability: independent framework consumers can construct and record the public tracker directly. The live Agent tool path instead emits canonical `ToolCall`/`ToolResult`/`ToolError` events and artifact descriptors; there is no mapping between them.
- Expected invariant: an exported reproducibility cell identifies one canonical execution, pairs start with exactly one terminal outcome, states success/failure/truncation, preserves meaningful order, and points to complete output when its preview is truncated.
- Observed behavior: a cell contains only an append index, tool name, caller-supplied 200-character summaries, duration, and recording timestamp. It lacks conversation/run/turn/message/call identity, success/error/failure, start identity, truncation/size/handling, artifact reference/hash, and schema version. Repeated or concurrent calls cannot be correlated to canonical events; lock-acquisition append order does not establish invocation order.
- Impact: exported JSON/Markdown cannot replay, audit, or reliably join a cell to the actual tool result despite the public reproducibility claim. Naively wiring this tracker would create a lossy second record beside the canonical Tool trace.
- Root cause: Notebook was designed as a free-form convenience log before canonical Tool identity, terminal events, and artifacts became authoritative.
- Direction: define a versioned Notebook cell as a projection/view of canonical `RunEvent` ToolCall/Result/Error plus `ToolOutputArtifactRef`; preserve explicit call/run identity, terminal outcome and artifact linkage. Do not introduce a second execution recorder. Delete or demote the free-form internal recording route once the canonical projector replaces it.
- Regression validation: repeated same-name calls, parallel calls completing out of order, typed failure, timeout/cancel, inline/truncated/spilled output, export/reload, and exact call-ID joins.
- Validation reports: [V01](../validations/F-NBK-01/V01-01.md), [V03](../validations/F-NBK-01/V03-01.md), [V07](../validations/F-NBK-01/V07-01.md)

### F-NBK-01-P2-02: Tracker lifetime and exports are unbounded

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/notebook/mod.rs:31`, `echo-agent/src/notebook/mod.rs:44`, `echo-agent/src/notebook/mod.rs:64`, `echo-agent/src/notebook/mod.rs:70`, `echo-agent/src/notebook/mod.rs:85`
- Reachability: any unrelated framework consumer can call the public constructor and retain cloned tracker handles across a long-lived session.
- Expected invariant: a session recorder has caller-visible lifecycle/retention or a durable append sink, and reading/exporting does not clone or lock an unbounded history wholesale.
- Observed behavior: cells append forever to one `Vec`; there is no capacity, retention, clear/drain, persistence sink, or pagination. `tool_name` is unbounded, `cells()` clones the entire history, and both exports retain the read lock for the complete O(n) render/serialization.
- Impact: long-running consumers can grow memory without a framework bound, double it during snapshots, and block recording during large exports. Wiring the advertised automatic per-tool path would amplify this on exactly the long sessions Notebook claims to preserve.
- Root cause: the type models an in-memory demo collection rather than an explicit recorder lifecycle/storage contract.
- Direction: choose a configurable bounded buffer or append-only sink with explicit retention and paged/snapshot export; bound all cell fields and avoid holding the write-blocking read lock while formatting. EKO selects file layout/retention policy but should not reimplement the generic lifecycle.
- Regression validation: sustained append beyond capacity, large tool names, concurrent record/export, bounded allocation, retention/drain semantics, and durable reload if a sink is selected.
- Validation reports: [V04](../validations/F-NBK-01/V04-01.md), [V06](../validations/F-NBK-01/V06-01.md), [V08](../validations/F-NBK-01/V08-01.md)

### F-NBK-01-P3-03: Exports can change Markdown structure and hide JSON serialization failure

- Priority: P3
- Confidence: high for Markdown; medium for current JSON impact
- Layer: framework
- Evidence: `echo-agent/src/notebook/mod.rs:70`, `echo-agent/src/notebook/mod.rs:76`, `echo-agent/src/notebook/mod.rs:85`, `echo-agent/src/notebook/mod.rs:89`
- Reachability: every standalone consumer exporting a tracker executes these methods; tool names and summaries are caller/tool-controlled strings.
- Expected invariant: export preserves cell boundaries for arbitrary Unicode/text and returns serialization errors distinctly from valid content.
- Observed behavior: Markdown interpolates tool/input/output directly, so newlines, headings and backticks reshape the document. JSON export catches any serde error and returns `[]`, indistinguishable from an empty Notebook. Current fields serialize reliably to JSON, making the latter primarily a brittle future/error-contract defect.
- Impact: shared Markdown can present incorrect step structure, while a future fallible cell field can silently erase the visible export instead of reporting failure.
- Root cause: display rendering and fallible serialization were exposed as infallible string helpers without escaping or typed errors.
- Direction: return `Result` for fallible export, include a schema version, and render untrusted text through escaped/fenced structural sections. Remove the `[]` fallback.
- Regression validation: CJK/emoji boundaries, embedded headings/newlines/backticks, empty versus failed serialization, and round-trip version/error cases.
- Validation reports: [V05](../validations/F-NBK-01/V05-01.md), [V06](../validations/F-NBK-01/V06-01.md), [V08](../validations/F-NBK-01/V08-01.md)

## De-duplicated Dependency Finding

F-RCT-01-P2-05 remains the sole finding for `enable_notebook` being a stored but unread Agent construction option. [V02](../validations/F-NBK-01/V02-01.md) and [V07](../validations/F-NBK-01/V07-01.md) add Notebook-specific evidence but create no duplicate ID. A fix must not treat CLI non-use as evidence to delete the standalone public framework API.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and source-clean snapshot | yes | passed | [report](../validations/F-NBK-01/V00-01.md) |
| V01 | Definition/export/duplicate search across both repositories | yes | passed | [report](../validations/F-NBK-01/V01-01.md) |
| V02 | Agent registration and runtime reachability | yes | failed; dependency finding | [report](../validations/F-NBK-01/V02-01.md) |
| V03 | Cell identity/order/terminal/artifact mapping | yes | failed | [report](../validations/F-NBK-01/V03-01.md) |
| V04 | Persistence, retention, and bounded export | yes | failed | [report](../validations/F-NBK-01/V04-01.md) |
| V05 | Malformed text, UTF-8, panic/overflow, export fidelity | yes | failed; positive subchecks passed | [report](../validations/F-NBK-01/V05-01.md) |
| V06 | Existing test/example coverage | yes | failed; no matches | [report](../validations/F-NBK-01/V06-01.md) |
| V07 | Documentation and historical drift | yes | failed; old poison issue fixed | [report](../validations/F-NBK-01/V07-01.md) |
| V08 | Targeted executable fixtures | no per review instruction | not run; future matrix defined | [report](../validations/F-NBK-01/V08-01.md) |
| V90 | Early invalid-path search disclosure | evidence integrity | inconclusive; not adopted | [report](../validations/F-NBK-01/V90-01.md) |
| V99 | Report/link/executor/source-clean integrity | yes | attempt 01 inconclusive; attempt 02 passed | [01](../validations/F-NBK-01/V99-01.md), [02](../validations/F-NBK-01/V99-02.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [report](../validations/F-NBK-01/V30-01.md) |

Primary static acceptance is recorded in V30. The deliberately deferred
executable matrix remains implementation-phase regression work rather than a
blocker for these source-conclusive findings.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AUDIT_REPORT.md`: poisoned Notebook RwLock causes cascading panic | fixed | All five lock sites recover with `unwrap_or_else(|e| e.into_inner())`; [V05](../validations/F-NBK-01/V05-01.md), [V07](../validations/F-NBK-01/V07-01.md) |
| Agent config: enabling Notebook records every tool invocation | stale/aspirational | No config reader, tracker owner, record call or accessor; canonical finding F-RCT-01-P2-05; [V02](../validations/F-NBK-01/V02-01.md) |
| Module rustdoc: exported summaries form a full reproducible session | current only as a display export; stale as reproducibility | Cells omit canonical identity/outcome/artifact and truncate both payloads; [V03](../validations/F-NBK-01/V03-01.md) |
| EKO file-backed workbench: application owns lineage/UI/files and avoids a second kernel | current and compatible | It does not make the independent framework tracker dead; [V01](../validations/F-NBK-01/V01-01.md), [V07](../validations/F-NBK-01/V07-01.md) |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, dynamic fixture, allocation benchmark, or network lookup was run. V08 defines the future regression matrix.
- No Notebook import API exists, so malformed persisted-cell rejection could only be assessed as a missing contract; arbitrary deserialization was not executed.
- Current JSON fields appear serialization-infallible, so P3-03's JSON branch has medium present impact; the Markdown branch is direct and high confidence.
- The review does not decide whether every framework consumer wants Notebook enabled. It only establishes that the public standalone option is reasonable framework capability and that the Agent option currently promises behavior it does not own.
- Status remains `needs_evidence` pending Codex primary source reconstruction and acceptance, not because a dynamic review command is required.

## Handoff

- Primary should independently sample `src/notebook/mod.rs:13-104`, `src/agent/config.rs:171-174,807-818`, `src/trace/mod.rs:302-359`, and pipeline ToolCall/terminal emission before acceptance.
- Preserve F-RCT-01-P2-05 as the sole no-op construction finding; merge only backlinks, not IDs.
- F-EXT-01 remains canonical for Tool execution/artifact defects. Notebook should consume/project those facts, never become a second executor or artifact writer.
- Application synthesis may use the EKO `ArtifactKind::Notebook` and file-backed workbench as policy inputs, but must not infer framework API death from current CLI non-use.
- This report becomes stale if Notebook gains an Agent owner, its cell schema changes, Tool event/artifact identity changes, or a persistence/retention API is added.
