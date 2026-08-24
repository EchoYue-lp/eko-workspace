# F-EXT-01: Tool contract, registry, schema, and artifacts

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at handoff; review reports are outside the source repositories

## Question

Is the reusable Tool contract typed, collision-explicit, cancellable,
paginated, and capable of bounded model output plus complete artifacts through
its real execution path?

## Scope

- `echo-core/src/tools`: `Tool`, parameters/result/failure/stream types,
  `ToolContext`, pagination, and text artifact persistence.
- `echo-execution/src/tools.rs`: generic registry, schema catalog, normal and
  streaming execution, timeout/retry/concurrency, and telemetry.
- `echo-tools/src/registry.rs` and `files/artifact.rs`: feature registration and
  complete artifact retrieval; representative collection tools for pagination.
- Root `ReactAgent` construction, custom-tool registration, execution pipeline,
  hooks, output budgeting, events, and LLM tool-result projection.
- Both repositories for semantic duplicate and live-caller searches. EKO was
  inspected only to exclude a second application-owned Tool authority.

## Out Of Scope

- Source fixes or API changes.
- Individual shell/file/code/Git implementation correctness (`F-EXT-02`).
- MCP/A2A/channel adapters and specialized extension ecosystems (`F-EXT-03`).
- Provider-specific tool-call wire parsing (`F-LLM-02`/`F-LLM-03`).
- Security attack or exploit analysis.
- Cargo, rustc, tests, builds, or dynamic fixtures. The user explicitly
  prohibited executable validation during this review phase; V12 records the
  future matrix without claiming execution.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and `TASKS.md`; Codex
  reviewer protocol.
- Dependency [F-CORE-01](F-CORE-01.md), used for the accepted identity and
  `ToolFailure` boundary.
- [F-API-01](F-API-01.md), read only to avoid duplicating its facade/export
  findings; it intentionally deferred Tool runtime semantics here.
- Current source and scoped git history. No other reviewer directory or report
  was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Tool schema/argument validation, registration identity/collision policy, execution/stream lifecycle, cancellation, typed result/failure, model projection, pagination, output bounds, and complete artifact references are reusable framework mechanisms. |
| EKO product policy | Which tools are enabled, local artifact root/retention, approval UI, model-visible tool subset, and rendering belong to EKO. No EKO policy is needed to correct the findings below. |
| Adapter boundary | Domain tools and MCP/skill adapters supply a canonical name/schema and translate typed parameters/results. The adapter must not own a second validator, registry replacement rule, retry loop, pagination protocol, or artifact authority. |
| Duplicate search | Searched `Tool`, `ToolManager`, `ToolRegistrar`, parameter/schema/execute/validate methods, `ToolResult`, page/cursor/artifact types, registration and model-message call paths across both repositories. One structural authority exists at each intended layer; the root name-based validator is a duplicate behavioral authority and EKO has no second executor. |
| Migration deletion | Make the generic registry/executor authoritative. Delete `ParseValidateStage`'s tool-name table and disconnected validation helper after callers move; preserve one explicit replacement API and remove silent replacement from ordinary registration. Preserve the existing text artifact writer/reader rather than adding an application copy. |

## Current Path

```text
Tool implementation
  -> Tool::{name, description, parameters}
  -> register_all_tools / ReactAgent core registration / builder custom tools
  -> ToolManager DashMap<String, Box<dyn Tool>>
       get_openai_tools -> ToolDefinition -> model
       execute_tool[_stream]_with_context -> execute_with_context

model ToolCall.arguments
  -> JSON Value -> Object map or empty map
  -> ReAct pipeline
       intervention
       ParseValidateStage (name table, not Tool contract)
       visibility/plan/hooks/permission (may rewrite arguments)
       ExecuteStage -> ToolManager
       output guard -> text spill/truncation -> trace
  -> String only
  -> AgentEvent::ToolResult + Message::tool_result
```

The positive artifact path is substantial. `AgentConfig` defaults to a 1 MiB
temporary text-artifact threshold (`src/agent/config.rs:252`); central output
projection spills the complete UTF-8 string, records its SHA-256, returns a
500-character preview with exact `read_artifact` instructions, and bounds spill
failure with a fallback token budget (`src/agent/snapshot.rs:926`). The reader
confines paths to the configured root, validates snapshot identity/hash, reads
bounded UTF-8 pages, and puts its continuation cursor in visible output
(`echo-tools/src/files/artifact.rs:183`). Shared collection pagination is also
snapshot-bound and overflow-safe, but only attaches its cursor as metadata.

## Findings

### F-EXT-01-P1-01: Advertised schema and validation are disconnected from execution

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:744`,
  `echo-agent/echo-core/src/tools/mod.rs:880`,
  `echo-agent/echo-core/src/llm/types.rs:603`,
  `echo-agent/echo-execution/src/tools.rs:618`,
  `echo-agent/echo-execution/src/tools.rs:730`,
  `echo-agent/src/agent/react/run/pipeline.rs:129`,
  `echo-agent/src/agent/react/run/pipeline.rs:258`,
  `echo-agent/src/agent/react/run/pipeline.rs:340`,
  `echo-agent/src/agent/react/run/execution.rs:271`
- Reachability: every ReAct tool call enters the default pipeline, then normal
  or streaming ToolManager execution. `validate_tool_parameters_async` has no
  caller; both manager paths invoke the tool directly.
- Expected invariant: the schema shown to the model and the `Tool` validation
  contract are enforced immediately before the exact parameters execute,
  including hook/approval rewrites.
- Observed behavior: schema JSON is copied without registry validation;
  `Tool::validate_parameters` is never called by the manager or live pipeline.
  `ParseValidateStage` instead recognizes only `read_file`, four write tool
  names, and `shell`. Later hooks can replace parameters without revalidation,
  and raw non-object input becomes an empty map. Macro tools happen to
  deserialize inside execution while manual tools implement inconsistent local
  checks.
- Impact: a framework consumer cannot rely on advertised required/type/range/
  additional-property constraints. Hook-modified or malformed calls may execute
  with a parameter shape the model schema rejects, including side-effecting
  tools; failure timing/category varies by implementation.
- Root cause: schema publication, an optional validator method, a ReAct-specific
  name table, macro deserialization, and manual checks evolved as separate
  authorities.
- Direction: validate schemas at registration and invoke one canonical,
  schema-aware/typed validator after all allowed rewrites and before execution
  in ToolManager. Delete the name table and disconnected public helper once all
  paths use that authority; adapters should only deserialize to their typed
  input.
- Regression validation: manual/macro tools; missing/wrong/range/unknown fields;
  scalar/array/null input; invalid registered schema; hook and approval rewrites;
  normal/stream parity; prove execute was not called on failure.
- Validation reports: [V01](../validations/F-EXT-01/V01-01.md),
  [V03](../validations/F-EXT-01/V03-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P1-02: Ordinary registration silently replaces the selected tool

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:720`,
  `echo-agent/echo-execution/src/tools.rs:528`,
  `echo-agent/src/agent/react/mod.rs:393`,
  `echo-agent/src/agent/react/mod.rs:461`,
  `echo-agent/src/agent/react/builder.rs:975`,
  `echo-agent/src/agent/react/capabilities.rs:36`,
  `echo-agent/src/agent/react/capabilities.rs:68`
- Reachability: ReactAgent registers reserved/core and feature tools first, then
  the builder registers caller-supplied tools using ordinary `add_tool`.
- Expected invariant: a duplicate canonical name is rejected/reported, or the
  caller explicitly invokes a replacement operation and receives the displaced
  registration.
- Observed behavior: `ToolManager::register` calls `DashMap::insert` and ignores
  the returned old tool. `ToolRegistrar` cannot return a conflict. This silently
  changes schema, risk/permission, and implementation even though a distinct
  documented `replace_tool` API already exists.
- Impact: a plugin/custom/domain registration can unintentionally replace
  framework controls such as `final_answer` or `tool_search`; behavior depends
  on assembly order and the caller receives no diagnostic.
- Root cause: canonical identity is only a map key, with no namespace,
  ownership, reserved-name, or collision result in the registration contract.
- Direction: make ordinary registration return a typed duplicate error with
  origin facts; reserve explicit replacement for the existing replacement API.
  Validate batches atomically for internal and prior collisions. Do not solve
  this with an EKO-only allowlist.
- Regression validation: builtin/custom, within-batch, feature-feature, skill,
  MCP, removal/re-registration, and concurrent schema-read cases; assert
  definition and execution identity remain aligned.
- Validation reports: [V02](../validations/F-EXT-01/V02-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P1-03: The generic executor accepts but does not consume cancellation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:1024`,
  `echo-agent/echo-execution/src/tools.rs:618`,
  `echo-agent/echo-execution/src/tools.rs:662`,
  `echo-agent/echo-execution/src/tools.rs:759`,
  `echo-agent/src/agent/react/run/phases/tools.rs:176`,
  `echo-agent/src/agent/react/run/phases/tools.rs:303`,
  `echo-agent/echo-tools/src/code.rs:334`
- Reachability: ExecuteStage injects the run token into every ToolContext;
  ToolManager owns semaphore, retry, timeout, normal, and stream waits for all
  callers.
- Expected invariant: cancellation terminates every generic wait with a typed
  cancelled failure, while a tool may additionally propagate the token into a
  child process.
- Observed behavior: ToolManager never reads the token. Only searched
  `run_code` and `agent_tool` explicitly consume it. The ReAct wrapper notices a
  separate run token, grants five seconds, then abandons/drops the invocation;
  direct ToolManager consumers and underlying work are not covered by that
  orchestration behavior.
- Impact: independent framework consumers cannot reliably stop queued,
  retrying, streaming, or long-running tools. ReAct can report abandonment
  without a typed per-tool cancelled terminal or proof that child work stopped.
- Root cause: cancellation was added as optional context metadata and in outer
  orchestration/tool-specific patches rather than in the executor state
  machine.
- Direction: race the same token against permit acquisition, retry delay,
  normal/stream execution, and output backpressure; map one terminal to
  `Cancelled`, and define side-effect facts. Keep child-process cancellation in
  specialized tools but delete redundant outer polling once the executor owns
  the contract.
- Regression validation: cancel before start, queued, retry delay, normal
  future, stream receive/send, side-effect started, timeout race, and completion
  race; assert one terminal and no retry.
- Validation reports: [V04](../validations/F-EXT-01/V04-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P1-04: Shared pagination hides its continuation cursor from the model

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/pagination.rs:172`,
  `echo-agent/echo-tools/src/web/search.rs:99`,
  `echo-agent/echo-tools/src/web/search.rs:168`,
  `echo-agent/src/agent/snapshot.rs:1239`,
  `echo-agent/src/agent/react/run/phases/tools.rs:216`,
  `echo-agent/echo-core/src/llm/types.rs:420`
- Reachability: ten domain-tool sites apply shared `PageInfo`; real ReAct
  success projection turns only the processed output string into the model's
  tool message.
- Expected invariant: if a first page is truncated, the model receives the
  opaque continuation cursor required by the next call.
- Observed behavior: `PageInfo::apply_to` writes `page.next_cursor` only to
  `ToolResult.metadata`. Representative collection outputs contain items and
  counts but no cursor; the model/result event projection drops metadata.
  `web_search` even instructs the model to use `page.next_cursor`. The separate
  `read_artifact` implementation is a positive exception because it embeds its
  cursor in text.
- Impact: the model cannot request page two of ordinary paginated results, so
  the new pagination contract silently makes results beyond the first page
  unreachable during autonomous execution.
- Root cause: pagination was designed against the rich ToolResult/telemetry
  object, while the live LLM message boundary remained string-only.
- Direction: introduce one typed, model-visible bounded result envelope or a
  canonical text projection containing machine-readable page facts. Do not
  patch every tool separately; delete descriptions/tests that assume invisible
  metadata once the shared projection is authoritative.
- Regression validation: each paginated tool through the real Message sent on
  the next LLM request; multi-page completion, last page, cursor mismatch,
  Unicode, artifact cursor distinction, and provider round-trip.
- Validation reports: [V05](../validations/F-EXT-01/V05-01.md),
  [V09](../validations/F-EXT-01/V09-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P1-05: A public binary result becomes successful empty model output

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:286`,
  `echo-agent/echo-core/src/tools/mod.rs:390`,
  `echo-agent/src/agent/react/run/pipeline.rs:720`,
  `echo-agent/src/agent/snapshot.rs:926`,
  `echo-agent/src/agent/snapshot.rs:1239`,
  `echo-agent/src/agent/react/run/phases/tools.rs:216`
- Reachability: `ToolResult::binary` is a public framework constructor accepted
  by ToolManager and the same live ReAct success path as text results. No
  current builtin constructor was found, so impact is presently on independent
  framework consumers rather than a proven EKO builtin call.
- Expected invariant: every accepted result kind has a bounded visible
  projection and complete recoverable representation, or unsupported kinds are
  rejected by the contract.
- Observed behavior: `binary` sets `output` empty and stores the payload only in
  `bytes`. Output guard, spill/truncation, artifact writer, events, and model
  messages inspect only the string. The invocation therefore reports success
  with empty content and writes no binary artifact. Structured JSON avoids the
  bug only because its constructor duplicates JSON into output.
- Impact: a valid third-party Tool implementation can lose its complete
  successful payload at the framework's primary consumer boundary with no
  error or recovery reference.
- Root cause: ToolResult was widened into a tagged/rich container without an
  exhaustive canonical projection; the artifact system remains text-only.
- Direction: either implement exhaustive typed projection/artifact storage for
  every supported kind (including MIME/hash/size and a reader), or remove the
  unsupported bytes API. Preserve the current complete text path as one
  implementation, not the universal projection.
- Regression validation: bytes/image with empty/non-empty text, MIME, JSON,
  table/diff/file/command kinds, threshold/spill failure, event and model
  projection, complete hash-verified recovery.
- Validation reports: [V06](../validations/F-EXT-01/V06-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P1-06: Pre-execution validation and policy blocks are emitted as successes

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:17`,
  `echo-agent/src/agent/react/run/pipeline.rs:129`,
  `echo-agent/src/agent/react/run/pipeline.rs:958`,
  `echo-agent/src/agent/snapshot.rs:1229`,
  `echo-agent/src/agent/react/run/phases/tools.rs:215`,
  `echo-agent/src/agent/react/run/phases/tools.rs:241`
- Reachability: missing `path`/`command`, invocation visibility, plan mode,
  hooks, and permission decisions can all set `ctx.blocked` before ExecuteStage.
  The snapshot execution method is the live concurrent/serial ReAct entry.
- Expected invariant: invalid arguments produce an unsuccessful
  `InvalidArguments` terminal; blocked/denied/cancelled policies have distinct
  typed terminals. They cannot enter success metrics/events.
- Observed behavior: any `ctx.blocked` pipeline returns `Ok(reason)` with no
  ToolResult. `run_tools` consequently emits `AgentEvent::ToolResult`, adds a
  normal model tool result, and counts the call in the successful branch.
  Executed `success:false` results do retain ToolFailure, proving the model is
  bypassed only on the pre-execution path.
- Impact: observers and agents cannot distinguish a successful operation from
  rejected invalid input or policy denial; retry/recovery, metrics,
  checkpoints, and UI status can be incorrect.
- Root cause: pipeline flow control (`blocked`) doubles as a terminal semantic
  and returns text through `Result<String, _>` instead of one exhaustive typed
  outcome.
- Direction: replace boolean/text blocking with a typed terminal outcome that
  distinguishes invalid, unavailable/hidden, denied, and cancelled. Project it
  once to event/model text. Delete success handling for `ctx.blocked` and the
  parallel legacy execution outcome once migrated.
- Regression validation: each blocking stage plus executed success/failure;
  assert event kind, failure category, model feedback, metrics, checkpoint, and
  no tool invocation.
- Validation reports: [V07](../validations/F-EXT-01/V07-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

### F-EXT-01-P2-07: The public Tool trait permits mutually recursive execution defaults

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:738`,
  `echo-agent/echo-core/src/tools/mod.rs:748`,
  `echo-agent/echo-core/src/tools/mod.rs:764`,
  `echo-agent/echo-execution/src/tools.rs:618`
- Reachability: a manual external implementation needs only name, description,
  and parameters to satisfy the compiler; ToolManager always enters
  `execute_with_context`.
- Expected invariant: the trait requires one executable primitive, or a default
  returns a typed unsupported result.
- Observed behavior: default `execute` calls default `execute_with_context`,
  which calls `execute` again. A comment asserts implementations must override
  one method, but the type system does not enforce it. Current builtins/macros
  override at least one, so no current internal runtime instance was found.
- Impact: a valid independent consumer implementation can enter unbounded
  future recursion instead of receiving a compile error or typed failure.
- Root cause: backwards-compatible bidirectional delegation encoded an
  unenforceable semantic requirement.
- Direction: require one canonical context-aware execution method. If legacy
  context-free tools remain necessary, use an explicit adapter/secondary trait;
  delete one side of the recursive defaults.
- Regression validation: compile-time minimal implementations for canonical and
  legacy adapters, plus public-manager execution; an implementation omitting
  execution must not compile or must return a typed unsupported result.
- Validation reports: [V08](../validations/F-EXT-01/V08-01.md),
  [V10](../validations/F-EXT-01/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate, and layering search | yes | passed with duplicate validation authority | [V01](../validations/F-EXT-01/V01-01.md) |
| V02 | Registration and runtime reachability | yes | failed | [V02](../validations/F-EXT-01/V02-01.md) |
| V03 | Schema/execute contract and rewrites | yes | failed | [V03](../validations/F-EXT-01/V03-01.md) |
| V04 | Generic cancellation trace | yes | failed | [V04](../validations/F-EXT-01/V04-01.md) |
| V05 | Cursor to model projection | yes | failed | [V05](../validations/F-EXT-01/V05-01.md) |
| V06 | Bounded output and complete artifact by payload kind | yes | failed; text path passed | [V06](../validations/F-EXT-01/V06-01.md) |
| V07 | Invalid/block typed terminal | yes | failed | [V07](../validations/F-EXT-01/V07-01.md) |
| V08 | Public Tool execution default invariant | yes | failed | [V08](../validations/F-EXT-01/V08-01.md) |
| V09 | UTF-8, panic, and overflow static scan | yes | passed | [V09](../validations/F-EXT-01/V09-01.md) |
| V10 | Existing test/assertion coverage | yes | failed | [V10](../validations/F-EXT-01/V10-01.md) |
| V11 | Historical drift classification | yes | passed | [V11](../validations/F-EXT-01/V11-01.md) |
| V12 | Targeted executable fixtures | future | not run by explicit user constraint | [V12](../validations/F-EXT-01/V12-01.md) |
| V13 | Mechanical handoff integrity and source cleanliness | yes | passed | [V13](../validations/F-EXT-01/V13-01.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [V30](../validations/F-EXT-01/V30-01.md) |
| V31 | Primary acceptance integrity and source isolation | yes | passed | [V31](../validations/F-EXT-01/V31-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `55d7a25`: classify failures and gate retries | current but incomplete before execution | Executed unsuccessful ToolResults preserve retry/side-effect facts; pre-execution blocks bypass the type (P1-06). |
| `b5b2e2e`: persist complete oversized output artifacts | current for UTF-8 text | Default central spill and `read_artifact` are complete and hash-bound; binary result is outside that path (P1-05). |
| `97a1e90`: bound output and drain cancellation | current at outer ReAct/text path, partial generically | ReAct observes cancellation and text spill failure is bounded; ToolManager itself ignores `ToolContext.cancel` (P1-03). |
| `9fad29f`: defer schemas and recover artifacts | current | Tool visibility/search and text artifact recovery are live; this commit does not establish schema validation at execution. |
| `bbca516`: unify pagination and output telemetry | current metadata, incomplete model contract | Shared page metadata and counters are live, but next cursor is not model-visible (P1-04). |
| F-CORE-01: ToolFailure is the positive typed error exception | current | Executed failures retain it; this review narrows the pre-execution bypass rather than duplicating F-CORE's non-tool error finding. |

## Coverage And Uncertainty

- No executable validation was performed. V12 is intentionally `not_run`; all
  behavioral conclusions above are static call-graph proofs and the task remains
  `needs_evidence` pending Codex primary source-anchor sampling.
- The full field-by-field implementation of all 122 framework `parameters`
  methods and every individual domain tool belongs to F-EXT-02/F-EXT-03. This
  task inspected the generic authority, macro path, live pipeline, and
  representative manual/paginated/artifact tools.
- No current builtin uses `ToolResult::binary`; P1-05 is a public framework
  contract failure for reasonable external consumers, not a claim of current
  EKO binary loss.
- Dropping an async tool future may cancel cooperative work; it does not prove
  child-process/network termination. P1-03 therefore states that the manager
  lacks a cancellation guarantee, not that every cancellation leaks work.
- The scoped production panic/UTF-8/overflow scan passed. It does not cover all
  domain tool implementations or provider adapters.

## Handoff

- Primary review should sample V03 (validator caller absence and post-validation
  rewrites), V05 (metadata-to-string loss), V06 (binary constructor-to-empty
  output), and V07 (blocked-to-Ok result) before accepting.
- Downstream iteration should first define one exhaustive Tool invocation/result
  contract, then wire it into ToolManager and delete the root name table. Fixing
  individual tools before the authority is selected would preserve duplication.
- Preserve the existing text artifact writer/reader, snapshot-bound pagination,
  UTF-8 handling, checked arithmetic, classified executed-tool failures, stream
  terminal requirement, and deterministic schema ordering.
- F-EXT-02 should audit individual shell/file/code/Git schema and cancellation
  behavior without reopening the generic findings. F-EXT-03 should test MCP,
  skill, LSP, and dynamic registrations against the eventual collision and
  typed projection contract.
- Primary review independently sampled the Tool trait, ToolManager validation/
  registration/cancellation paths, pipeline blocked projection, pagination,
  and binary-result boundary. The seven findings and priorities were accepted;
  see V30.
- This report becomes stale if Tool/ToolRegistrar signatures, ToolManager
  register/execute paths, pipeline stage order, `Message::tool_result`,
  pagination metadata, or artifact/result projection changes.
