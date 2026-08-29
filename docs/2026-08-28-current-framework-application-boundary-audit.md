# R1 Framework / Application Boundary Closure

> Initial inventory: 2026-08-28 at framework `302453b`, CLI `4462b8a`.
> Final code baseline: framework `1446cae`, CLI `df88546`.
> Final CLI documentation baseline: `0417443`.
> This is a cross-repository closure ledger, not a second framework or EKO product specification.

## Conclusion

The original audit covered all 151 Rust files under `echo-agent-app-core/src` and classified 124 as
`Keep`, 19 as `Migrate/converge`, 8 as `Conditional`, and 0 as immediately deletable. R1 is now
complete:

- all 19 migration candidates have either converged on a framework producer, been reduced to a thin
  EKO adapter, or been explicitly retained as EKO product policy;
- all 8 conditional candidates have an explicit final disposition: three retained, three deleted,
  and two replaced by executable contracts before deletion;
- there is no remaining duplicate turn, Task graph, timeout, artifact, bootstrap, diff, plugin,
  hot-memory, tool-control, background dependency, workflow, or surface-test authority.

An application file remaining after R1 does not imply duplicated authority. Workspace identity,
DomainProfile, review/worktree policy, file-backed product projections, pool policy, UI rendering and
local extension source precedence are intentionally EKO-owned.

## Delivered Dependency Order

| Capability | Framework producer | EKO consumer / deletion |
| --- | --- | --- |
| canonical turn receipt | `fb92562` | `3c2a322` |
| typed Task timeout settlement | `98b3011` | `6e7c2c6` |
| typed Tool artifact reference | `1e69f6f` | `f78bfdf` |
| immutable prepared plugin generation | `24c5c2d`, hygiene `1446cae` | `c61ecc1` |
| hot-memory deletion/publication events | `c4be2a9` | `df88546` |
| application service composition | existing lifecycle primitives | `df52535` |
| workspace diff authority | existing file/Git primitives | `c1c5109` |
| interactive Tool control | existing ToolManager/Agent generation APIs | `1f22960` |
| background Task dependency authority | existing revisioned Task graph | `c73cd21` |
| static contract retirement | existing typed/executable owners | `29dadb9` |
| dead output/scheduler shims | n/a | `eb65ed9` |
| legacy workspace importer | n/a | `f909b3b` |
| unreachable output surfaces | n/a | `8ae3bad` |

## Final Disposition Of 19 Migrate / Converge Rows

| Original candidate | Final disposition | Framework generic mechanism | EKO policy / thin adapter | Exact deletion or retained boundary | Delivered commit(s) |
| --- | --- | --- | --- | --- | --- |
| `chat_driver.rs` | Converged | `AgentTurnDriver` + canonical `TurnReceipt` | resource/HITL/webhook projection only | deleted local usage, compaction and final-answer accumulation | `fb92562`, `3c2a322` |
| `chat_event_log.rs` | Keep-app, converged | segmented `EventJournal` owns sequencing/integrity/pruning | stream identity, retention pins and surface replay | no second journal algorithm; executable F6 owners replace string matrices | `3c2a322`, `29dadb9` |
| `diff.rs` | Converged app service | framework file/Git primitives remain reusable | one EKO workspace/ref-aware diff service | deleted Tauri DTO, `TextDiff` algorithm and root GUI `similar` dependency | `c1c5109` |
| `extension_control.rs` | Converged app owner | immutable `PreparedPluginSet` | source precedence, workspace targets and settlement receipt | deleted follower re-enumeration/independent reload paths | `24c5c2d`, `c61ecc1` |
| `infra.rs` | Keep-app, converged | Agent/Store/MCP/model generation primitives | EKO config/bootstrap and local capability policy | composition moved behind `ApplicationServices`; tool control uses canonical generation | `df52535`, `1f22960`, `df88546` |
| `plugin_runtime.rs` | Converged app transaction | framework prepares and wires one immutable generation | primary/pool/LSP/monitor/theme publication | deleted implicit constructor reload, duplicate parsing and additive follower reload | `24c5c2d`, `c61ecc1`, `1446cae` |
| `run_driver.rs` | Keep-app, converged | revisioned Task graph + canonical turn/memory facts | unattended EKO launch and review policy | no parallel dependency graph or memory refresh | `c73cd21`, `df88546` |
| `tool_execution.rs` | Keep-app projection | typed `ToolResult.artifact` | verified storage, retention and detail cursor | deleted metadata-key parsing and path-existence inference | `1e69f6f`, `f78bfdf` |
| `tool_execution_projection.rs` | Thin adapter | typed Tool invocation/result/artifact facts | lossless event-to-detail projection | no second artifact descriptor or terminal inference | `1e69f6f`, `f78bfdf` |
| `tool_exposure.rs` | Converged product policy | ToolManager disabled-tool generation | direct-user versus automated invocation policy | deleted phantom Tauri `tool_states`; all surfaces use one receipt | `1f22960` |
| `workflow_service.rs` | Keep-app; migration unnecessary | framework Graph/DSL execution | EKO catalog, persistence and command formatting | Tauri/CLI/TUI/channel already call the same service; no duplicate algorithm | verified at `df88546` |
| `observability/diagnostics.rs` | Keep-app, converged | provider usage and canonical `TurnReceipt`/RunStore facts | bounded diagnostic grouping and UI DTO | deleted parallel turn accounting inputs | `fb92562`, `3c2a322` |
| `state.rs` | Keep-app aggregate, converged | typed generation/publication primitives | workspace registry and EKO service aggregate | deleted branch-local composition, plugin fanout, hot refresh and phantom tool state | `df52535`, `c61ecc1`, `df88546`, `1f22960` |
| `tasks/background.rs` | Keep-app compatibility projection | canonical revisioned TaskRun dependency graph | background command/API projection | no background-only dependency authority | `c73cd21` |
| `tasks/service.rs` | Converged app scheduler adapter | canonical Task graph/status | scheduler/background admission policy | deleted second dependency waits and state mutation path | `c73cd21` |
| `task_runtime/continuation.rs` | Converged adapter | typed timeout/turn outcome | EKO budget and review continuation policy | deleted post-framework timeout repair and hot-memory refresh | `98b3011`, `6e7c2c6`, `df88546` |
| `task_runtime/executor.rs` | Converged adapter | RuntimeTaskService, typed timeout, Tool artifact and Task graph | worktree/resource/review policy | deleted terminal metadata inference, parallel dependency and refresh logic | `98b3011`, `6e7c2c6`, `1e69f6f`, `f78bfdf`, `c73cd21`, `df88546` |
| `task_runtime/store.rs` | Keep-app file authority, converged | journal/runtime mutation and typed timeout facts | EKO file layout and current projections | deleted post-framework terminal rewrite; executable contracts replace source scans | `98b3011`, `6e7c2c6`, `c73cd21`, `29dadb9` |
| `task_runtime/turn_lifecycle.rs` | Thin adapter | canonical `TurnReceipt` + typed timeout settlement | persist one EKO RunTurn and apply product continuation policy | no second stream/terminal or timeout authority | `fb92562`, `3c2a322`, `98b3011`, `6e7c2c6` |

## Final Disposition Of 8 Conditional Rows

| Original candidate | Final disposition | Reachability / reason | Exact deletion or retained boundary | Delivered commit |
| --- | --- | --- | --- | --- |
| `surface_contract.rs` | Deleted after replacement | test-only string matrix | wire tests moved to owning modules; five-surface executable fixture retained in F6 | `29dadb9` |
| `browser/sidecar.rs` | Keep-app | production Playwright MCP prepare/connect/restart path | Node/npm/profile packaging is EKO platform policy | verified at `df88546` |
| `output/mod.rs` | Keep-app, pruned | REPL still uses the renderer | removed unreachable format/markdown/syntax/table/theme surfaces and false TUI facade claim | `8ae3bad` |
| `output/spinner.rs` | Deleted | no caller outside its dead factory | removed `start_spinner`, module and `indicatif` dependency | `eb65ed9` |
| `scheduler/task.rs` | Deleted | no caller of compatibility `TaskStore` alias | framework `CronTaskStore` re-export remains | `eb65ed9` |
| `state/reliability_contracts.rs` | Keep-app test authority | executable restart/ABA/cold-delivery tests | retained; not counted as production capability or static scaffolding | verified by `29dadb9` gates |
| `task_runtime/long_horizon_contracts.rs` | Deleted after replacement | test-only source-string and magic-count scans | invariants moved to executable/compile-fail owner tests | `29dadb9` |
| `workspace/migration.rs` | Deleted | only imported obsolete `~/.echo-agent` layout | removed CLI/TUI/Tauri commands, state transition and module | `f909b3b` |

## Generation Closure

Plugin and memory generation are not parallel runtimes:

- Framework `PreparedPluginSet` freezes one dependency/variable/component view; EKO publishes the
  same generation to primary, existing pool, future pool, LSP and monitor targets with typed target
  settlement (`24c5c2d`, `c61ecc1`).
- `ReviewIntegration` owns a strict hot-memory snapshot and one generation-bound
  `Arc<MemoryLayerManager>`; primary, existing and future agents receive the same projection.
  Surface-local rereads, the unused `PROJECT.md` reflect silo and old rebind/apply-memory paths were
  removed (`c4be2a9`, `df88546`).

## Bootstrap And Surface Closure

`ApplicationServices` is the single EKO composition owner for GUI, TUI, CLI/JSONL, channel and soak
paths. It constructs AppState/pool/TaskRuntime once, calls `AppState::set_pool`, binds the selected
config save path, starts scheduler/task service, MCP health and Dreaming, performs extension/delivery
reconciliation, and hands all cancellable/joinable owners to `ApplicationLifecycleOwner`. The old
headless/desktop/soak bootstrap families and `HeadlessDreamingOwner` were deleted (`df52535`).

Tauri is now transport only for workflow, diff, Tool control and artifact detail. The frontend wire
shape remains stable, but domain algorithms and authority live in app-core/framework owners.

## Validation Status

Each slice ran focused tests plus its applicable framework/CLI gates before commit. The final R1
source set passed both strict clippy gates, workspace all-feature tests, app-core `1507 passed / 0
failed / 9 ignored`, CLI suites and compile-fail doctests. These are implementation gates, not the
project Final Integration/Release evidence: performance, long soak, manual GUI, remote CI, website
sync and child-first release remain pending.

No source, public API or product document is owned by this top-level ledger. Formal framework docs
remain in `echo-agent`; formal EKO docs remain in `echo-agent-cli`.
