# A-OBS-01: Diagnostics, webhooks, and operational visibility

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI clean; externally dirty framework paths excluded and
> required framework contracts read only from committed HEAD
> Accepted by: Codex primary reviewer after independent source-anchor,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Are EKO diagnostics, run context, webhook events, and logs wired to live
lifecycle facts without globals, accidental secret/content leakage, or
misleading success?

## Scope

- `echo-agent-app-core/src/webhook/{events,emitter}.rs`, live chat observer,
  scheduler adapter, `AppState`, and config watcher.
- `echo-agent-app-core/src/observability/{types,diagnostics}.rs`.
- GUI bootstrap, Tauri diagnostic commands, frontend endpoint and
  `ObservabilityPanel`; TUI/CLI/channel registration only where needed to prove
  parity and reachability.
- Committed framework `EventIdentity`, `Run`, `RunSummary`, `RunStore`, and
  configuration loader contracts needed to evaluate the thin EKO adapters.
- Definition/duplicate search, runtime reachability, config identity, lifecycle
  correlation, outbound redaction/bounds, retry/shutdown, diagnostic grouping,
  prompt provenance, panic/UTF-8/overflow, and existing tests.

## Out Of Scope

- Source changes and Cargo/rustc/test/build/fixture/network execution.
- Framework trace/audit persistence redaction, retention, concurrency,
  filesystem identity, and generic scheduler lifecycle owned by F-OPS-01.
- Canonical chat terminal/surface behavior owned by A-CHAT-01; this task covers
  only the webhook projection of those facts.
- Durable TaskRun lifecycle-hook replay owned by A-TSK-04; HTTP webhook delivery
  is a separate application adapter.
- Generic secret/sandbox policy owned by F-SEC-01 and broad GUI/Tauri surface
  integration owned by A-SRF tasks; these non-dependency reports were not read.
- Public-service threat assumptions. User-configured webhook endpoints are local
  assistant extensions; findings require no permission gate and address only
  accidental disclosure, incorrect identity, and application lifecycle defects.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex track
  rules and report templates.
- Exact Codex dependencies A-CHAT-01, A-TSK-04, and F-OPS-01.
- Clean CLI source at the pinned commit and committed framework blobs at the
  pinned commit. No other reviewer report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Event identity, Run/RunStore facts, typed terminal status, redaction primitives, and bounded delivery primitives belong in the reusable framework when provider-neutral. Existing framework APIs remain authoritative. |
| EKO product policy | Webhook event selection, configured destinations, GUI diagnostic presentation, current workspace/config display, and whether delivery is best-effort or durable are EKO application policy. |
| Adapter boundary | EKO should project canonical EventEnvelope/Run facts and attach product identity without inventing another run or terminal state machine. HTTP delivery may queue/retry but must not own chat/TaskRun settlement. |
| Duplicate search | Searched both repositories for WebhookEvent/Payload/Emitter/Endpoint, RunDiagnostics/Summary, list/load diagnostics, EventIdentity, RunStore, prompt assembly, state constructors, emitters, commands, frontend calls, tests, and behaviorally equivalent retry/redaction paths. |
| Migration deletion | Correct the existing application observer/emitter/diagnostic projection. Delete the Boolean `completed` terminal inference and unowned per-endpoint spawn path when typed outcome/bounded dispatch lands; do not add a second event registry or RunStore. |

## Current Path

```text
AppConfig.webhooks
  -> main or desktop creates one WebhookEmitter
  -> same instance -> config watcher + TUI/CLI/channel resources
                   -> GUI AppState -> chat resources + scheduler adapter

drive_chat EventEnvelope{conversation/run/turn/execution/...}
  -> observer receives only AgentEvent payload
  -> raw tool args/errors or Agent error -> WebhookEvent without identity
  -> emit() -> unbounded detached endpoint task -> HTTP + one fixed retry

cron framework FireFn
  -> EKO launch_cron_run -> read TaskRun terminal
  -> Completed only -> CronTaskCompleted -> same emitter

framework RunStore
  -> list_diagnostic_runs groups root and children by parent/business ID
  -> load_run_diagnostics loads children, root only when no child exists
  -> attach process-global bootstrap PromptAssembly
  -> Tauri IPC -> ObservabilityPanel
  -> CLI/channel /trace formatter
```

The positive boundaries to retain are: no global webhook singleton remains;
GUI replaces its constructor-local emitter with the watcher-owned instance before
publication; TUI/CLI/channel receive the shared instance; cron verifies the
TaskRun terminal before emitting completion; token and duration accumulation is
saturating; argument preview and Run input preview use UTF-8-safe character
iteration; HMAC signing does not log the secret directly.

## Findings

### A-OBS-01-P0-01: Webhooks send unredacted tool arguments and unbounded raw errors

- Priority: P0; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:119-156`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/events.rs:9-32`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:138-165`.
- Reachability: every live chat mode injects the configured emitter; ToolCall,
  ToolError, Agent Error, and setup error reach observer/emitter serialization
  and every matching user-configured endpoint.
- Expected invariant: operational summaries redact known credential/token forms
  and impose UTF-8-safe field/record bounds immediately before outbound
  serialization, independent of producer behavior.
- Observed behavior: tool arguments are raw JSON truncated to 240 characters;
  tool and Agent errors are raw and unbounded. No secret scanner or recursive
  redaction runs before serialization. HMAC authenticates bytes but does not
  limit their content.
- Impact: an API key or credential embedded in tool input/error may be copied to
  an endpoint the user configured only for operational notifications; an
  arbitrarily large error can also inflate every delivery body/task.
- Root cause: payload safety is optional producer formatting rather than one
  outbound adapter invariant.
- Direction: apply one recursive secret-redaction and bounded-summary policy at
  webhook serialization, preserve only typed non-sensitive identity, and refer
  to complete content by an authorized durable artifact when needed. No endpoint
  permission gate is warranted for this local-assistant threat model.
- Regression validation: nested JSON secrets, authorization headers, URLs,
  tool/Agent errors, Unicode, boundary lengths, and assert raw values never
  appear in serialized bytes.
- Validation reports: [V04](../validations/A-OBS-01/V04-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

### A-OBS-01-P1-02: Webhook events discard canonical correlation and can emit contradictory chat terminals

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:93-175`,
  `:483-503`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/events.rs:9-54`.
- Reachability: the shared driver creates a complete EventIdentity for every
  turn, then calls the observer with only `event.payload` on the live loop.
- Expected invariant: each event carries stable configuration/workspace,
  conversation, run, turn, execution, tool-call, event/delivery and attempt
  identity, and one terminal matching the canonical driver/TaskRun outcome.
- Observed behavior: the schema carries none of those correlation fields and no
  tool call ID or cancellation variant. Any FinalAnswer sets a Boolean that
  causes ChatCompleted at loop exit even when Error/Cancelled follows; AgentError
  can therefore coexist with uncorrelated completion. Cron's TaskRun terminal
  check is correct and is not part of this defect.
- Impact: receivers cannot join retries, tools, turns, task runs, workspaces, or
  configuration revisions, deduplicate events, or reliably determine whether a
  turn completed, failed, or was cancelled.
- Root cause: the observer consumes payloads instead of the canonical envelope
  and independently infers semantic success from one intermediate event.
- Direction: project an application webhook envelope from EventIdentity plus
  stable config/workspace and delivery IDs; consume the typed chat outcome that
  resolves A-CHAT-01-P1-01. Delete `WebhookTurnObserver.completed` and do not add
  another terminal state machine.
- Regression validation: FinalAnswer/Error/Cancelled/setup error/sink close,
  repeated tool names/call IDs, retries, parallel turns/workspaces, and exact
  agreement with TaskRun and UI terminal.
- Validation reports: [V03](../validations/A-OBS-01/V03-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

### A-OBS-01-P1-03: Detached webhook delivery has no bounded or observable terminal lifecycle

- Priority: P1; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:125-175`,
  `:179-208`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:260-268`.
- Reachability: every emitted chat or cron event uses this sole HTTP delivery
  path; normal shutdown has no handle for its spawned tasks.
- Expected invariant: accepted delivery work is bounded; the owner can observe
  delivered/failed/cancelled, correlate retries, and drain or cancel on shutdown.
- Observed behavior: `emit` returns `()`, spawns an outer task and an unbounded
  task per endpoint, then logs failures and performs one fixed-delay retry. It
  provides no queue bound, delivery ID/result, cancellation, join/drain, or
  persisted retry. Process exit may discard accepted events; successful delivery
  is invisible and terminal failure is only an uncorrelated URL warning.
- Impact: bursts can grow task/body memory; shutdown loses notifications; users
  cannot distinguish an endpoint with successful delivery from one silently
  failing after a transient or process exit.
- Root cause: best-effort network side effects have no application lifecycle
  owner or explicit delivery contract.
- Direction: define best-effort versus durable semantics and use one bounded
  dispatcher with stable delivery identity, typed outcomes, backoff, cancellation
  and shutdown drain. Reuse TaskRun facts; do not create a lifecycle ledger for
  chat/task execution.
- Regression validation: slow/failing endpoints, bursts, retry correlation,
  endpoint reload, cancellation and shutdown with bounded pending work.
- Validation reports: [V05](../validations/A-OBS-01/V05-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

### A-OBS-01-P1-04: A malformed watched config silently replaces live webhook endpoints with defaults

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:249-277`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/config.rs:678-685`,
  `:724-756`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:82-100`.
- Reachability: GUI and non-GUI bootstraps pass their live shared emitter to the
  watcher; any watched create/modify/remove event calls this path.
- Expected invariant: a temporary parse failure retains all live domains from
  the last accepted config and reports rejection; one successful snapshot
  updates them atomically.
- Observed behavior: watcher uses infallible `load_config`; an invalid explicit
  file becomes `AppConfig::default`, then endpoints are unconditionally replaced
  with the default empty list. Hook reload separately keeps last-known-good,
  creating mixed configuration identity.
- Impact: a routine partial save/invalid edit disables webhook notifications
  until a later valid watcher event, while other live config domains remain old;
  failure appears only in logs.
- Root cause: a bootstrap fallback API is reused for live reload despite the
  committed config module providing a fallible exact-file API for reloaders.
- Direction: parse the resolved watched path with `load_config_file`, validate
  all live domains, then apply one accepted snapshot; retain prior state on any
  error and expose a typed reload result.
- Regression validation: partial YAML, missing file, atomic replace, rapid
  invalid-valid edits, endpoint/hook identity, and recovery without restart.
- Validation reports: [V06](../validations/A-OBS-01/V06-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

### A-OBS-01-P1-05: Diagnostic details omit the root trace whenever child traces exist

- Priority: P1; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/observability/diagnostics.rs:16-63`,
  `:122-161`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/trace/mod.rs:125-141`,
  `:563-584`.
- Reachability: GUI, CLI, and channels all list then load through these shared
  functions against the primary agent's production RunStore.
- Expected invariant: a diagnostic group and its detail contain the same root
  plus child member set with deterministic deduplication.
- Observed behavior: list groups root R and its children under R. Detail loads
  only children and loads R itself only if no child exists. Root status, usage,
  input, events and timing are therefore present in the summary but absent from
  detail whenever a child exists.
- Impact: the observability panel can show a group status/token total that its
  detail cannot explain, hide the primary failure/completion, and misdirect run
  diagnosis.
- Root cause: the fallback-to-root branch treats root and child retrieval as
  alternatives rather than a union.
- Direction: load root independently, merge child IDs once, preserve explicit
  provenance and deterministic order, and surface partial-load errors instead
  of silently changing membership.
- Regression validation: root-only, child-only business ID, root plus multiple
  children, duplicate IDs, missing/corrupt member, mixed terminal status and
  summary/detail equality.
- Validation reports: [V07](../validations/A-OBS-01/V07-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

### A-OBS-01-P2-06: Historical run diagnostics are labeled with the current process prompt assembly

- Priority: P2; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/runtime.rs:102-108`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/state.rs:585-598`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/panels.rs:1162-1173`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/observability/types.rs:18-31`.
- Reachability: every GUI detail request clones the one bootstrap PromptAssembly
  and attaches it to the selected durable diagnostic; CLI `/trace` does the same.
- Expected invariant: historical operational data is run-keyed, or explicitly
  unavailable when exact configuration/workspace provenance was not captured.
- Observed behavior: the bootstrap snapshot has module hashes/counts but no run,
  config, workspace or model identity. It is attached unchanged to old runs and
  pooled-agent runs; channels omit it, so surfaces also disagree.
- Impact: users can diagnose a cache/prompt issue using module data from a
  different process restart, workspace, model or configuration and reach the
  wrong conclusion.
- Root cause: current process metadata is passed as a display convenience into a
  historical run projection without provenance validation.
- Direction: persist a bounded prompt manifest/fingerprint on the exact Run or
  correlate through one canonical run-keyed record; otherwise omit it and label
  unavailable. Do not introduce another diagnostic/run store.
- Regression validation: runs before/after restart, workspace/model/config
  changes, pooled agents, missing manifests, and GUI/CLI/channel parity.
- Validation reports: [V08](../validations/A-OBS-01/V08-01.md),
  [V09](../validations/A-OBS-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact-input isolation and dirty-state disclosure | yes | passed | [V00](../validations/A-OBS-01/V00-01.md) |
| V01 | Definition and duplicate authority search | yes | passed | [V01](../validations/A-OBS-01/V01-01.md) |
| V02 | Registration and live runtime reachability | yes | passed | [V02](../validations/A-OBS-01/V02-01.md) |
| V03 | Event/config/run/turn/tool/terminal identity | yes | failed | [V03](../validations/A-OBS-01/V03-01.md) |
| V04 | Secret and content redaction/bounds | yes | failed | [V04](../validations/A-OBS-01/V04-01.md) |
| V05 | Retry/failure/shutdown delivery lifecycle | yes | failed | [V05](../validations/A-OBS-01/V05-01.md) |
| V06 | Config identity and last-known-good reload | yes | failed | [V06](../validations/A-OBS-01/V06-01.md) |
| V07 | Root/child diagnostic membership | yes | failed | [V07](../validations/A-OBS-01/V07-01.md) |
| V08 | Historical prompt provenance | yes | failed | [V08](../validations/A-OBS-01/V08-01.md) |
| V09 | Existing test and edge-case inventory | yes | failed | [V09](../validations/A-OBS-01/V09-01.md) |
| V10 | Targeted executable scenarios | policy-deferred | not_run | [V10](../validations/A-OBS-01/V10-01.md) |
| V11 | Report/link/executor/source integrity gate | yes | passed | [V11](../validations/A-OBS-01/V11-01.md) |
| V30 | Primary acceptance sampling | yes | passed | [V30](../validations/A-OBS-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-CHAT-01-P1-01: driver collapses Agent error/cancel into transport success | current; canonical in dependency | [V03](../validations/A-OBS-01/V03-01.md) |
| A-TSK-04-P2-01: durable task lifecycle hook has no replay cursor | current; distinct dependency boundary | [V05](../validations/A-OBS-01/V05-01.md) |
| F-OPS-01-P0-02: framework trace/audit stores persist sensitive unbounded content | current; canonical framework owner, not duplicated here | [V04](../validations/A-OBS-01/V04-01.md) |
| F-OPS-01-P1-03/P1-04: RunStore interleavings and retention can corrupt/expand operational data | current; upstream residual risk | [V07](../validations/A-OBS-01/V07-01.md) |
| Webhook emitter comment: one process normally has one injected emitter | current after bootstrap trace | [V01](../validations/A-OBS-01/V01-01.md), [V02](../validations/A-OBS-01/V02-01.md) |

## Coverage And Uncertainty

- Pure static review only. No Cargo, rustc, test, build, dynamic fixture, or
  network process ran. V10 is future validation and not a review blocker.
- Framework current worktree was externally dirty, including trace/config-related
  areas. Every framework anchor was reconstructed from committed HEAD; dirty
  content/diffs were neither used nor changed. CLI stayed clean while source was
  inspected.
- The exact frequency of FinalAnswer followed by another terminal and detached
  delivery loss was not measured. The inability to preserve/correlate those
  states is established by type and control flow.
- F-SEC/A-SRF reports were intentionally not read because they are not task
  dependencies. Their catalog ownership was used only to avoid broad security
  and surface findings; synthesis should merge any title overlap under the
  narrower canonical owner.
- F-OPS-01 upstream RunStore races/corruption can independently distort all
  application diagnostics even after A-OBS findings are fixed.

## Handoff

- Fix order: outbound redaction/bounds; canonical webhook identity/terminal;
  fallible last-known-good config reload; bounded delivery lifecycle; root/child
  detail equality; run-keyed prompt provenance.
- Reuse A-CHAT-01's future typed chat outcome and framework EventIdentity/RunStore.
  Keep one webhook schema/emitter and one diagnostics projection; delete Boolean
  terminal inference and detached spawn authority rather than layering another
  state machine beside them.
- Primary reviewer should statically sample V03-V08 and resolve any overlap with
  F-SEC/A-SRF without importing their findings into this task. Dynamic tests stay
  deferred under the user's review-stage policy.
- This report becomes stale if webhook schema/emitter/observer, config reload,
  diagnostic grouping/types, prompt assembly storage, chat terminal contract,
  or framework Run/EventIdentity contracts change.
