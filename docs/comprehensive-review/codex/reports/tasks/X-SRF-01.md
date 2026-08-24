# X-SRF-01: Surface feature parity

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework dirty and inspected through committed HEAD only; application clean; Codex reports only

## Question

Are GUI, TUI, CLI, channels, cron, and background modes complete Agents differing only in trigger and rendering policy?

## Scope

- Definition, registration, composition and real caller matrix for GUI, TUI, interactive/noninteractive CLI, QQ/Feishu channels, cron and background.
- Chat/Task/Auto, PlanTask edit/control, Subagent, Tool, HITL, attachment, artifact/chart, memory, Skill/plugin, MCP/Browser/LSP, workflow, automation and cancellation availability.
- Canonical Agent event, Tool artifact and TaskRuntime facts as they cross surface adapters.
- Trigger/render classifications, host service startup, durable replay and active-turn ownership.
- Existing parity/static tests inspected without execution.

## Out Of Scope

- Atomic channel group identity, cancel, attachment, service shutdown and noninteractive details owned by A-SRF-04.
- Tauri setup/terminal/workflow command details owned by A-SRF-02; TUI projection/edit details by A-SRF-01; GUI reducer/remount/focus details by A-SRF-03.
- Tool writer/allowlist/channel renderer defects owned by A-TOOL-01; plugin transactions by A-PLG-01; MCP/LSP/Browser lifecycle by A-INT-01.
- Shared chat producer terminal semantics, attachment preparation internals, export formats, source fixes and dynamic gates.
- All dirty framework worktree contents and all non-authorized reviewer reports.

## Inputs

- Root AGENTS.md; shared README/REPORTING/TASKS exact card; Codex README.
- Authorized complete Codex dependencies [A-SRF-01](A-SRF-01.md), [A-SRF-02](A-SRF-02.md), [A-SRF-03](A-SRF-03.md), [A-SRF-04](A-SRF-04.md), [A-TOOL-01](A-TOOL-01.md), [A-PLG-01](A-PLG-01.md), and [A-INT-01](A-INT-01.md).
- Current clean CLI source and committed framework HEAD anchors. No other reviewer directory or task report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Agent/Tool events, Task/Subagent identity, artifact references, cancellation primitives, framework channel/LSP/MCP/Browser/workflow mechanics remain in `echo-agent`. Framework APIs are not deleted because one EKO surface fails to use them. |
| EKO product policy | Capability enablement, long-lived host service startup, attended command availability, pool policy generation, surface replay/delivery and local interaction UX belong to `echo-agent-cli`. |
| Adapter boundary | Every adapter consumes one EKO capability manifest, one active-turn lifecycle and one lossless event/artifact projection. Trigger and presentation may vary; identity, terminal, error, artifact and durable state may not. |
| Duplicate search | Searched boot/entry modes, `drive_chat`, TaskRuntime launch, AgentPool, Tauri invoke handlers, TUI/CLI/channel commands, workflows, cron/background startup, cancellation, attachment/artifact/chart branches, plugin projection and parity tests across both repositories. |
| Migration deletion | Preserve shared `drive_chat`, TaskRuntime, AgentPool and framework contracts. After cutover delete Tauri-only workflow authority, hand-maintained prose matrix, duplicated surface registration lists, local cancellation-token ownership, and lossy Chart/Tool wildcard render branches. |

Direct user terminal, Browser, MCP, LSP and local extensions remain outside Agent automation permission gates. Cron/background do not need interactive widgets, but they do need equivalent durable facts and explicit interaction-required outcomes.

## Current Path

```text
shared application/runtime core
  AgentRuntime + TaskRuntimeStore + AgentPool + canonical task tools
    GUI -> TauriChatSink -> typed frontend stores/components
    TUI -> TuiChatSink -> local reducer + live Subagent bus + plan poller
    CLI -> ChannelChatSink -> blocking human renderer
    channel -> ChannelChatSink -> text/paragraph transport
    cron/background -> TaskRuntime launch -> durable events/artifacts/webhooks

surface-local composition
  GUI AppState starts task service + scheduler; Tauri registers GUI commands
  TUI/CLI start temporary headless services; own separate command catalogs
  channel-only starts neither scheduler nor background service
  plugin style mutates primary Agent only; pool Agents keep prior/default policy
```

### Capability Matrix

| Capability | GUI | TUI | Interactive CLI | Channel | Cron | Background | Classification |
|---|---|---|---|---|---|---|---|
| Chat/Task/Auto core | shared `drive_chat` | shared `drive_chat` | shared `drive_chat` | shared `drive_chat` | TaskRuntime trigger | TaskRuntime trigger | core parity positive |
| TaskRun/PlanTask execution | live | live | live | live via Agent/tools | live | live | core parity positive |
| Attended graph edit | canonical revision UI | missing | command/tool only, not equivalent UI control | Tool/natural language only | N/A | N/A | real attended gap |
| Durable Task/Subagent replay | full event/artifact hydration | plan poll + ephemeral bus | text/paths | text; Tool facts dropped | durable | durable | real projection gap |
| Tool success/error/artifact | typed cards/detail | bounded messages/artifact action | human renderer/path | ordinary lifecycle/artifact dropped | durable | durable | real projection gap |
| HITL | dialog provider | card provider | terminal provider | next-message provider | interaction-required policy | interaction-required policy | trigger/render difference; provider defects atomic |
| User attachments | picker | `/attach` | `/attach` | transport types exist but live receivers text-only | N/A | persisted refs only | channel capability missing |
| Chart/full structured output | full typed spec | 500-char Notice | 500-char preview | 500-char text | durable only when promoted | durable only when promoted | real loss |
| Memory/Skills/plugins | live | live commands | live commands | shared pooled tools | pooled Agent | pooled Agent | plugin output-style policy diverges |
| MCP/Browser/LSP Tools | live with atomic defects | live | live | pooled Tool surface | pooled Tool surface | pooled Tool surface | core reachable; lifecycle defects atomic |
| Workflow library CRUD/run | GUI-only Tauri authority | missing | missing | missing | can trigger separate background kind | can trigger separate background kind | real product capability gap |
| Scheduler/background service | started | started | started | not started in channel-only | self | self | host composition gap |
| Foreground cancel | GUI scoped command/map | active local control | token discarded while REPL blocks | token discarded; `/cancel` mostly HITL | run-store policy | run-store policy | lifecycle gap |
| Noninteractive typed Agent | no CLI role | no CLI role | absent | protocol adapter only | durable events | durable events | real CLI adapter gap |

Positive conclusions:

- All attended chat surfaces already converge on one `PreparedUserTurn`/`ChatResources`/`drive_chat` path; cron/background converge on TaskRuntime. A rewrite or second Agent engine is unnecessary.
- Core Tools, Task execution, memory, Skills, MCP/Browser/LSP and Subagents are broadly constructed for pooled/headless Agents. Current gaps are not valid evidence that a mode is a reduced product.
- Unattended cron/background correctly differ in trigger and live interaction policy; they should not receive GUI/TUI widgets or cloud-style permission gates.

## Findings

### X-SRF-01-P1-01: Surface-local capability composition creates real missing services and commands

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/main.rs:172-200,239-304,347-405`; `echo-agent-cli/src/cli/modes.rs:32-63,68-109,118-150`; `echo-agent-cli/src/tauri/desktop.rs:183-240`; `echo-agent-cli/src/tauri/commands/panels.rs:676-784`; `echo-agent-cli/src/tui/commands.rs:53-125`; `echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:10-139`.
- Reachability: each production host constructs the shared core, then separately starts services and registers commands. Channel-only bypasses headless services; Tauri owns workflow business logic; TUI omits graph edits/workflow/research export; no one-shot CLI branch exists.
- Expected invariant: one EKO capability manifest starts common host services exactly once and exposes every applicable attended capability through the same application service; only trigger/presentation differs.
- Observed behavior: handwritten composition roots and registries drift. Missing automation startup, workflow authority, attended Task edit and noninteractive CLI are not reasonable renderer differences.
- Impact: users must switch surfaces to use core features, channel-only hosting silently disables schedules/background recovery, and new capabilities require editing several unverified registries.
- Root cause: common runtime convergence stopped below application capability composition; adapters independently own service and command topology.
- Direction: add one application-owned capability/service manifest after runtime bootstrap, with typed supported/available state and shared handlers; derive Tauri/TUI/CLI/channel registration and process startup from it. Move workflow authority to app-core and delete adapter-private implementations/duplicate lists after cutover.
- Regression validation: production composition matrix proving one startup and one live caller for every capability/surface, including missing-service injection and exact N/A classifications.
- Validation reports: [V01](../validations/X-SRF-01/V01-01.md), [V02](../validations/X-SRF-01/V02-01.md), [V06](../validations/X-SRF-01/V06-01.md), [V09](../validations/X-SRF-01/V09-01.md)

### X-SRF-01-P1-02: Surface renderers do not preserve one lossless event and artifact contract

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/agent/mod.rs:143-270` at committed HEAD; `echo-agent-cli/src/tauri/commands/chat.rs:1525-1550`; `echo-agent-cli/src/tui/events.rs:2160-2205,4818-4874`; `echo-agent-cli/src/cli/repl.rs:700-729`; `echo-agent-cli/src/cli/channels.rs:500-625`; `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:190-255`.
- Reachability: every shared chat emits the same canonical events, but each sink/reducer chooses a separate projection; TaskRuntime-backed activity also reaches different live/durable consumers.
- Expected invariant: text renderers may summarize, but complete Chart/Tool/Task/Subagent/attachment/artifact facts remain available through typed state or a durable continuation reference and survive restart/lag.
- Observed behavior: GUI preserves full typed facts; TUI/CLI/channel irreversibly truncate Charts, channel drops ordinary Tool lifecycle/artifacts, TUI cannot durably hydrate detailed runtime evidence, and live channel transports do not populate attachment delivery.
- Impact: the same Agent run is auditable/recoverable on GUI but incomplete on text surfaces; users can lose errors, provenance, generated content and Subagent evidence by choosing a different interface.
- Root cause: canonical transport exists, but no application-level lossless projection/result contract constrains adapters; wildcard/preview rendering is treated as sufficient consumption.
- Direction: define one serializable EKO surface event/snapshot carrying canonical identities, terminal/failure and complete artifact refs. Make GUI/TUI/CLI/channel views consume it; delete lossy Chart previews and material wildcard drops. Replay TaskRuntime durable facts before overlaying live events.
- Regression validation: shared event fixture with Tool success/error/cancel, parallel Subagents, large Unicode Chart, attachment and artifact replayed through every renderer; compare canonical facts/hash rather than pixels/text.
- Validation reports: [V03](../validations/X-SRF-01/V03-01.md), [V06](../validations/X-SRF-01/V06-01.md), [V09](../validations/X-SRF-01/V09-01.md)

### X-SRF-01-P1-03: Foreground turn ownership is fragmented across adapter-local state

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/chat.rs:536-707`; `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:23-74,130-169`; `echo-agent-cli/src/tui/events.rs:555-577,4800-4874`; `echo-agent-cli/src/cli/repl.rs:215-235,500-544`; `echo-agent-cli/src/cli/channels.rs:190-262`.
- Reachability: every foreground turn creates cancellation/correlation state in its caller. GUI backend detaches execution from WebView refs; CLI blocks while holding only a local token; channel detaches a token inaccessible to commands; TUI reconstructs only part of durable runtime state.
- Expected invariant: one application lifecycle registry keyed by conversation/turn owns admission, scoped cancellation, replay/rebind, HITL association and one terminal independent of adapter lifetime.
- Observed behavior: remount/restart/lag can detach GUI/TUI views; CLI/channel cannot reach the active cancellation token; no common snapshot/cursor is available to all adapters.
- Impact: foreground work can continue invisibly, become uncancellable, retain stale state or lose its terminal/evidence depending on surface and disconnect timing.
- Root cause: `drive_chat` is shared but invocation ownership was left to each UI adapter instead of a shared application service.
- Direction: introduce one EKO active-turn service and durable/replayable surface cursor, pass surface sinks as subscribers, and map Ctrl+C/cancel/disconnect/remount to scoped lifecycle operations. Delete adapter-local anonymous cancellation ownership and ref-only correlation after cutover.
- Regression validation: token/Tool/Subagent/HITL phases with Ctrl+C, channel disconnect, WebView remount, TUI restart and lag; assert scoped cancel/rebind, ordered replay and exactly one terminal.
- Validation reports: [V04](../validations/X-SRF-01/V04-01.md), [V06](../validations/X-SRF-01/V06-01.md), [V09](../validations/X-SRF-01/V09-01.md)

### X-SRF-01-P1-04: Runtime prompt policy reaches the primary Agent but not pooled surface Agents

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:449-490,723-764`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:214-250,817-900`.
- Reachability: plugin output style can be activated from GUI/TUI/CLI, while GUI conversations, channels, TaskRuntime, cron and background acquire existing or future Agents from the pool.
- Expected invariant: active Agent response policy applies to primary plus all existing/future pooled Agents, or is truthfully scoped and not advertised globally.
- Observed behavior: style projection mutates only `agent_handle`; pool state/construction has no style generation. Shared Tool/Hook registries do propagate, proving this is a per-Agent projection gap rather than intentional mode policy.
- Impact: identical prompts can follow different active product style depending on surface, conversation age or unattended execution path.
- Root cause: one product-wide policy was stored as mutation of a single Agent identity.
- Direction: add an application runtime-capability generation consumed by primary, existing pool and future pool construction; keep generic system-context projection in the framework. Remove single-primary projection ownership after migration.
- Regression validation: activate/reload/remove style before and after creation of GUI/channel/task/cron/background Agents; verify one generation and equivalent prompt projection.
- Validation reports: [V05](../validations/X-SRF-01/V05-01.md), [V09](../validations/X-SRF-01/V09-01.md)

### X-SRF-01-P2-05: The parity gate validates prose rather than production reachability or facts

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:1-149`.
- Reachability: the test-only module is the named product parity contract and its matrix is compiled only for tests.
- Expected invariant: removing a production handler/service binding or dropping an event field makes the parity gate fail.
- Observed behavior: five surfaces omit background and noninteractive CLI; eleven rows contain manual prose; assertions check only count, fixed array length and non-empty text. They pass despite the source-conclusive gaps above.
- Impact: maintainers receive false green evidence for the project's strongest product invariant, allowing surface drift to accumulate.
- Root cause: architecture assertions were encoded as strings instead of executable registration/call/reducer fixtures.
- Direction: derive compile-visible capability mappings from the production manifest and replay shared request/event fixtures through each adapter. Delete the prose-only matrix once executable coverage owns the contract.
- Regression validation: mutation controls that remove one registration, startup binding, identity field, terminal or artifact ref and prove the parity suite fails.
- Validation reports: [V08](../validations/X-SRF-01/V08-01.md), [V09](../validations/X-SRF-01/V09-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and dirty-source isolation | yes | passed | [report](../validations/X-SRF-01/V00-01.md) |
| V01 | Shared core definition/reachability | yes | passed | [report](../validations/X-SRF-01/V01-01.md) |
| V02 | Six-surface capability registration/startup | yes | failed | [report](../validations/X-SRF-01/V02-01.md) |
| V03 | Event/Tool/attachment/artifact projection | yes | failed | [report](../validations/X-SRF-01/V03-01.md) |
| V04 | Active-turn cancel/replay/rebind lifecycle | yes | failed | [report](../validations/X-SRF-01/V04-01.md) |
| V05 | Primary/pool runtime policy projection | yes | failed | [report](../validations/X-SRF-01/V05-01.md) |
| V06 | Trigger/render difference classification | yes | passed | [report](../validations/X-SRF-01/V06-01.md) |
| V07 | Local desktop permission boundary | yes | passed | [report](../validations/X-SRF-01/V07-01.md) |
| V08 | Existing parity test coverage | yes | failed | [report](../validations/X-SRF-01/V08-01.md) |
| V09 | Atomic finding ownership/de-duplication | yes | passed | [report](../validations/X-SRF-01/V09-01.md) |
| V10 | Common dynamic surface replay matrix | future | not run per instruction | [report](../validations/X-SRF-01/V10-01.md) |
| V99 | Exact-ID/header/link/source-boundary integrity | yes | passed | [report](../validations/X-SRF-01/V99-01.md) |
| V30 | Primary capability-matrix sampling and acceptance | yes | passed | [report](../validations/X-SRF-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-SRF-01: TUI is a full Agent path but lacks attended graph edit, durable detail hydration and full Chart | current | V01, V02, V03, V04 |
| A-SRF-02: workflow authority is GUI/Tauri-only | current | V02; retained atomically under A-SRF-02 |
| A-SRF-03: GUI remount/focus/live reducer state is not durable lifecycle authority | current | V03, V04; details remain A-SRF-03 |
| A-SRF-04: shared CLI/channel runtime exists but channel automation, cancel, attachment and noninteractive parity are missing | current | V01-V04 |
| A-TOOL-01: channel drops material Tool events/artifacts | current | V03; canonical finding remains A-TOOL-01 |
| A-PLG-01: plugin style reaches primary but not pooled Agents | current | V05; cross-surface impact promoted here |
| A-INT-01: Browser/MCP/LSP are reachable without general permission gating but have lifecycle defects | current | V07; atomic lifecycle findings not duplicated |
| `surface_contract`: every product surface has evidence for every capability | regressed | prose remains non-empty while real registration/projection gaps exist; V08 |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, dynamic fixture, UI/channel launch or network operation ran. V10 is future evidence, not a pass.
- Static evidence is conclusive for missing registrations/startup, lossy match branches, missing shared cancellation ownership, primary-only style projection and prose-only tests. Exact timing and user-visible frequency remain dynamic.
- Framework dirty source was excluded. Only committed channel/event/scheduler anchors were sampled with `git show`/`git grep HEAD`.
- This report does not require cron/background to expose interactive selectors, pickers or dialogs. It requires durable equivalent facts, explicit terminal policy and common services appropriate to a long-lived host.
- Specific atomic defects remain in their owning reports; X-SRF findings describe only cross-surface roots and sequencing.
- Changes to entry composition, command registries, surface sinks/reducers, AgentPool projections, active-turn APIs or the parity harness stale this report.

## Handoff

- Preserve the existing shared Agent/TaskRuntime core. Build parity above it with one EKO capability manifest, one active-turn service and one lossless surface projection.
- First make identities/artifacts/lifecycle common, then derive GUI/TUI/CLI/channel renderers and automation hosts; do not implement six new execution paths.
- Keep unattended trigger/HITL differences explicit and preserve direct local user capabilities without cloud permission gates.
- X-EVT-01 should consume the lossless producer/consumer matrix; X-TOL-01 owns Tool schema/output field conformance; Q-E2E-01 owns dynamic scenario replay.
- Primary must independently reconstruct the five findings before changing `needs_evidence`.
