# Framework Capability Placement Audit

Review date: 2026-08-30

## Scope and rule

This audit applies the framework placement rule from
[framework ADR 0014](../echo-agent/docs/adr/0014-framework-capability-placement.md).
An existing second consumer is not required. A candidate can enter
`echo-agent` when its semantics are product-neutral, its dependency direction
is framework-to-application, and it has an independently testable public
contract. A real consumer path must switch before an old authority is deleted.

The audit is a disposition record, not an implementation authorization. No Rust
module is moved by this document.

## Existing framework authorities

| Framework symbol | Path | Reused meaning |
| --- | --- | --- |
| `TurnSteerMailbox` | `echo-agent/src/agent/steer.rs` | tracked active-turn input, incarnation and drain lifecycle |
| `SubagentControlRegistry` | `echo-agent/src/agent/subagent/control.rs` | attempt-scoped Subagent guidance, interrupt and settlement |
| `SegmentedFileEventJournal` | `echo-agent/echo-state/src/journal/segmented.rs` | generic event sequence, durability, integrity, replay and pruning |
| `PreparedPluginSet` | `echo-agent/src/plugin/prepared.rs` | immutable plugin preparation and generation reuse |
| `ToolManager` | `echo-agent/echo-execution/src/tools.rs` | shared tool registration, execution and resource projection |
| `RuntimeTaskService` / `RuntimeDagController` | `echo-agent/echo-orchestration/src/tasks/` | revisioned Task graph, claims, retry, cancellation and generic execution |
| `KeyedExecutionAdmission` | `echo-agent/src/agent/admission.rs:90` | opaque-key lease counts, per-key process permits, retirement fences, close and idle waits |
| `DeliveryLedger` | `echo-agent/echo-state/src/delivery.rs:585` | product-neutral envelope, lifecycle facts, FIFO frontier, attempt identity, stale claims, bounded terminal projection and prepared reconciliation |

These existing authorities remain the first reuse points. A new app integration may
not recreate their generic lifecycle or status meaning.

## Candidate dispositions

| Candidate | Current definition | Generic kernel disposition | EKO boundary | Next action |
| --- | --- | --- | --- | --- |
| `AgentPool` | `echo-agent-cli/echo-agent-app-core/src/agent_pool/{pool,admission,generation}.rs` | **Keyed admission extracted**: framework `KeyedExecutionAdmission` now owns active/by-key counts, per-key process permits, retirement fences, close and idle waits. Agent cache and eviction remain EKO. | EKO config, capacity classes, Agent creation, workspace generation, tool visibility, model policy and plugin target publication remain in app-core. | Plan 02 is complete; future work may evaluate only additional product-neutral pool kernels. |
| `AgentRouter` | `echo-agent-cli/echo-agent-app-core/src/agent_router/{address,inbox,router,delivery,recovery,persistence}.rs` | **Typed direct integration complete**: framework `DeliveryLedger<..., AgentAddress, AgentMessage>` is the sole lifecycle/projection/retry authority; app-core `AgentInboxProjection`, `FoldedDelivery`, legacy wire and checkpoint codec are deleted. | EKO workspace identity, ConversationStore validation, file-backed inbox layout, groups, retirement policy, live/cold runtime, wake scheduling and surface projection remain in app-core. | Final integration may validate a fresh data root and retirement behavior; no legacy replay promise or source-named conversion layer remains. |
| `ChatEventLog` | `echo-agent-cli/echo-agent-app-core/src/chat_event_log/{event,journal,projection}.rs` | **Already converged at primitive level**: framework `SegmentedFileEventJournal` owns generic sequence, integrity and replay. | EKO event payload, conversation identity, retention pins and UI/channel projection remain in app-core. | Keep the EKO log as an application-owned projection; improve framework journal only for a standalone generic contract. |
| `PluginRuntimeService` | `echo-agent-cli/echo-agent-app-core/src/plugin_runtime/{types,runtime,publication}.rs` | **Already converged at primitive level**: framework `PreparedPluginSet` owns immutable preparation and validation. | EKO target publication, preferences, workspace generation and primary/pool fan-out remain in app-core. | Keep publication transaction in EKO; do not duplicate framework preparation. |
| `ExtensionControlService` | `echo-agent-cli/echo-agent-app-core/src/extension_control/{types,service,policy,skills}.rs` | **Keep EKO** except for individually proven extension protocol primitives. | EKO Skill, Hook, MCP, LSP, Browser and Plugin mutation policy and receipts. | Do not create a generic service around EKO mutation policy; audit a protocol primitive separately if needed. |
| Task DAG / retry / cancel / claim / revision | `echo-agent/echo-orchestration/src/tasks/` plus EKO product integration | **Framework owner**; no new extraction. | EKO file facts, review/worktree policy and surface projection. | Extend existing framework contracts only when a gap is product-neutral; remove any duplicate integration logic in the same change. |
| `AppState`, workspace registry, DomainProfile, research/analysis/browser policy | `echo-agent-cli/echo-agent-app-core/src/state/`, `workspace/`, `research/`, `analysis/`, `browser/` | **Keep EKO**: semantics depend on the local product. | Full current implementation and product lifecycle. | No framework migration. |

## Exact symbol and call-path ledger

The following ledger records the concrete definitions, construction or
registration points, and production consumers used for the dispositions above.
Tests and historical source scans are not treated as production reachability.

| Candidate | Exact symbols and definitions | Construction / registration | Production call paths | Current owner |
| --- | --- | --- | --- | --- |
| `AgentPool` | `AgentPool` at `echo-agent-cli/echo-agent-app-core/src/agent_pool/pool.rs:6`; `AgentPool::from_runtime` at `pool.rs:378`; `AgentPool::acquire` at `pool.rs:748`; `AgentPoolExecutionLease` at `admission.rs:451`; framework `KeyedExecutionAdmission` at `echo-agent/src/agent/admission.rs:90`. | `AgentRuntime::build_agent_pool` calls `AgentPool::from_runtime` at `echo-agent-cli/echo-agent-app-core/src/runtime.rs:1387`; workspace transitions bind the pool in `state/workspace.rs`. | Foreground chat acquires at `echo-agent-app-core/src/chat_driver.rs:458`; Task service at `tasks/service.rs:123`; CLI channel at `src/cli/channels.rs:1192`; run driver at `echo-agent-app-core/src/run_driver.rs:112`; scheduler at `echo-agent-app-core/src/scheduler/runner.rs:162`. | EKO `AgentPool` owns cache/policy; framework owns keyed admission. |
| `AgentRouter` | `AgentAddress` at `echo-agent-cli/echo-agent-app-core/src/agent_router/address.rs:41`; `AgentRouter` at `agent_router/inbox.rs:122`; framework `DeliveryLedger` and `prepare_*` lifecycle API at `echo-agent/echo-state/src/delivery.rs`; `AgentRouterRetirementGuard::purge` at `inbox.rs:103`. | `AppState::default` creates `AgentRouter::at_default_root` at `echo-agent-app-core/src/state/app_state.rs:259`; workspace runtime receives the router at `state/workspace.rs:589`. | `AppState::send_agent_message_owned` validates and enqueues through the facade at `state/app_state.rs:1967`; CLI/TUI/channel commands use `state.agent_router` at `src/cli/channels.rs:2215`; conversation deletion purges the router at `echo-agent-app-core/src/conversation_deletion.rs:1276`. EKO's physical append uses `DeliveryLedger::apply_prepared_with`; production code does not construct `DeliveryEvent` directly. | Framework `DeliveryLedger` owns lifecycle/projection/retry; EKO owns address, file layout, physical durability/reopen, runtime selection, wake and retirement. |
| `ChatEventLog` | `ChatEventLog` at `echo-agent-cli/echo-agent-app-core/src/chat_event_log/event.rs:168`; `StreamJournal = SegmentedFileEventJournal<PersistedChatEvent>` at `event.rs:113`; `ChatEventLog::open` at `chat_event_log/journal.rs:93`; `reconcile_conversation_inputs_at_boot` at `journal.rs:433`. | `AppState::default` opens the log at `echo-agent-app-core/src/state/app_state.rs:190`; CLI JSONL and Tauri use `ChatEventLog::open` at `src/cli/jsonl.rs:225` and `src/tauri/commands/chat.rs` call paths. | `AppState` stores the shared log in `state/app_state.rs:62`; `conversation_input.rs:342` adapts it for durable ingress; `manual_compression.rs:128` and Tauri task commands consume it. | EKO chat payload/identity/retention projection; framework `SegmentedFileEventJournal` owns generic journal semantics. |
| `PluginRuntimeService` | `PluginRuntimeService` at `echo-agent-cli/echo-agent-app-core/src/plugin_runtime/types.rs:294`; `PluginRuntimeService::new` at `plugin_runtime/runtime.rs:2`; `prepared_generation_identity` is called at `state/app_state.rs:3567` and defined at `plugin_runtime/runtime.rs:837`. | `AgentRuntime::bootstrap` constructs it at `echo-agent-app-core/src/runtime.rs:1307`; `ExtensionControlService` receives the shared instance at `extension_control/service.rs:134`; CLI runtime wiring is at `src/main.rs:325`. | Workspace runtime publishes it at `echo-agent-app-core/src/state/workspace.rs:415`; config watcher observes it at `echo-agent-app-core/src/config_watcher.rs:702`; TUI applies the active theme at `src/tui/mod.rs:2036`. | EKO target publication and preference transaction; framework `PreparedPluginSet` owns immutable preparation. |
| `ExtensionControlService` | `ExtensionControlService` at `echo-agent-cli/echo-agent-app-core/src/extension_control/types.rs:304`; `publish_curated_skill` at `extension_control/service.rs:591`; `reconcile_enabled_skills_on_load` at `service.rs:1071`. | `AppState::default` creates it at `echo-agent-app-core/src/state/app_state.rs:250`; runtime bootstrap binds it at `echo-agent-app-core/src/runtime.rs:616`. | `AppState::extension_control_for_runtime` is consumed by `extension_commands.rs:2540`; CLI evolution publishes skills at `src/cli/cmd_impls/evolution.rs:753`; Tauri panels publish at `src/tauri/commands/panels.rs:1414`; config watcher reconciles at `echo-agent-app-core/src/runtime.rs:789`. | EKO extension mutation and settlement policy; no generic service extraction is approved. |

## Evidence and call-path notes

- The app-core pool still combines Agent cache creation and eviction with
  EKO-specific `EkoConfig`, `WorkspaceKind`, tool-control generation and plugin
  publication. Plan 02 removes only its duplicate admission reducer; the whole
  `AgentPool` remains an EKO owner.
- Plan 06 froze the framework `DeliveryLedger` contract and the EKO field-level
  round-trip fixture.
- Plan 07/08 were superseded during development by the typed direct integration:
  the existing AppState cold-delivery path uses the framework ledger with
  `AgentAddress` and `AgentMessage` as its concrete types. Legacy event
  definitions, old projection, checkpoint codec and read-only conversion view
  were deleted together with the duplicate authority.
- The EKO `EkoConfig::to_agent_config` method was also removed after a complete
  call-site search found no production consumer; bootstrap continues to select
  provider-neutral fields for the framework configuration without exposing a
  dead application wrapper.
- Framework `SubagentEvent::DispatchCompleted`, `DispatchFailed`, and
  `DispatchCancelled` now expose the typed field as `outcome`; EKO durable
  `TaskExecutionSummary`, `SubagentRun`, and `SubagentReleased` projections use
  the same name, so the execution envelope (`SubagentResult`) is never confused
  with its terminal outcome.
- Framework `TaskSpec::with_extension` and `TaskSpec::extension_as` now provide
  the typed extension boundary. EKO uses them for complete `EkoTaskExtension`
  values; only partial `TaskPatch` updates retain dynamic JSON merge semantics.
- EKO task DTO crossings now use standard `TryFrom`/`TryInto` implementations;
  source-named `to_task_spec`, `from_task_spec`, `to_task_plan_patch`, and
  `to_task` methods were removed from the active API.
- The app-core router owns file-backed inboxes, workspace retirement, runtime
  selection and wake scheduling while framework owns the delivery lifecycle and
  projection. Any future router work must reuse those framework lifecycles
  instead of adding a second durable input authority or conversion layer.
- The app-core chat log and plugin runtime already call framework journal and
  prepared-generation primitives. Moving the EKO wrappers would not improve
  reuse and would couple framework APIs to EKO payloads.
- `ExtensionControlService` coordinates several EKO mutation policies in one
  product scope; its current composition is not a generic framework service.

## Required follow-up contract

Plan 02 satisfies the following contract for the keyed admission kernel:

1. a framework trait or service contract with no EKO types;
2. a framework unit/integration test and runnable example;
3. an EKO product integration and one production path switched to it;
4. wire, persistence, receipt and five-surface compatibility evidence; and
5. a deletion list for any displaced app-core authority.

The app-core candidate is no longer authoritative for keyed admission: its
`AgentPoolAdmission` is a composition layer over the framework owner. For any future
kernel candidate, the next plan must name exact symbols and provide the same
contract before changing the owner. No empty framework wrapper is sufficient.
