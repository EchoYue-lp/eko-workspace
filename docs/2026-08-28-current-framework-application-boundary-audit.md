# R0：当前 framework / application 边界审计

> 审计日期：2026-08-28
> 审计类型：只读边界 inventory；不实施 R1 迁移，不修改 framework API、examples 或 website。
> framework 基线：`echo-agent@302453b174086c3795dc026d16eeb668ecc66bed`（F2）
> application 基线：`echo-agent-cli@4462b8aee9a4409fead54d7607d7df34990c0aad`（F4）

## 结论先行

当前边界总体方向是正确的：`echo-agent` 持有通用 Agent/Turn、Task DAG、Subagent、Tool、Store、Scheduler、HITL、MCP/LSP/channel、Workflow 和插件原语；EKO `echo-agent-app-core` 持有 workspace、DomainProfile、产品配置、文件/工件/评审、surface 投影、pool、应用生命周期和本地持久化策略。F4 已让 TUI、CLI、JSONL、channel 共享一套 headless service，并让 GUI/TUI/CLI/channel 的 foreground turn 进入 app-core 的 `foreground_turn`/`chat_driver`。

仍需在 R1 处理的边界风险不是“把 EKO 全部下沉到 framework”，而是三类适配器变厚或重复装配：

1. `src/main.rs`、`src/tauri/desktop.rs`、`src/cli/modes.rs` 都是 composition root；GUI 与 headless 仍分别组装 `AppState`/scheduler/task service，退出和启动回滚必须继续收敛到一个应用 owner。
2. `chat_driver.rs`、`tool_execution.rs`、`tasks/task_runtime/executor.rs` 保留了 EKO 侧事件、Tool terminal/artifact、Task 资源和结算策略；这些是应用策略，但其中 generic terminal/retry/settlement 语义必须以 framework 合同为准，不能再出现第二条主循环。
3. app-core 仍有少量明确的 conditional/test-only 或 legacy 入口（`surface_contract.rs`、`long_horizon_contracts.rs`、`workspace/migration.rs`、兼容别名）；本审计只标记退出门，不在 R0 删除。

这份清单覆盖 F4 中实际存在的 **151 个 `echo-agent-app-core/src/*.rs` 文件**（另有 2 个 examples、2 个 integration tests，共 155 个 tracked Rust 文件）。用户输入中的“149 个”对应 F4 前一条 `7bc9174` 祖先：F4 之后新增 `agent_control.rs` 与 `conversation_input.rs`；为避免遗漏，本审计以 F4 的 151 个 source 文件为权威，并单独记录该计数差异。

## 1. 作用域、证据与判定规则

### 1.1 只读输入

为控制磁盘，本 R0 worktree 未保留初始化后的 submodule。证据来自现有 integration checkout 与 source checkout 的 Git 对象：

| 输入 | 绝对路径 / 读取方式 | 用途 |
|---|---|---|
| framework F2 | `/Users/ls/.codex/worktrees/f24e/integration/echo-agent`，`git show 302453b:<path>` | facade、split crates、通用 Task/Subagent/Tool/Store 原语 |
| CLI F4 | `/Users/ls/.codex/worktrees/f24e/integration/echo-agent-cli`，`git show 4462b8a:<path>` / `git grep 4462b8a -- <path>` | 151 个 app-core 文件、composition roots、Tauri/TUI/CLI/channel 调用路径 |
| superproject | 当前 R0 worktree | 只新增本文件 |

没有把源 checkout 的 dirty working tree 当作事实；所有基线代码判断均以提交对象为准。

### 1.2 搜索方法

- 先用 `git ls-tree` 统计并枚举所有 app-core Rust 文件，再用 `git grep` 同时搜索定义、父模块 `mod` 注册、导出、构造点、工具注册点和 composition-root 调用点。
- 对主路径从 `main` / `run_desktop_entry` / `run_tui` / `run_repl` / `run_channels_mode` 反向追到 `AgentRuntime::bootstrap`、`AppState::from_shared`、pool、TaskRuntime、`drive_foreground_chat` 和 sinks。
- 对 framework facade 逐项检查 root `src/lib.rs` 的 `pub mod`、`prelude`、`advanced`、`workspace` re-export，并核对 app-core/CLI 的真实调用。
- “定义存在”不等于“生产可达”：`#[cfg(test)]`、test module、只在 examples 使用、仅兼容别名和未从 composition root 进入的路径分别记录。

### 1.3 处置代码

| 代码 | 含义 |
|---|---|
| `K` / keep | 当前 owner 正确；保留并继续由该层维护 |
| `M` / migrate | 保留产品语义，但未来把 generic 语义切回唯一 framework authority，或合并重复装配 |
| `C` / conditional | feature/test/平台/兼容路径；在退出门满足前保留 |
| `D` / delete | 已确认只剩被替代或不可达路径；R0 只登记，R1 按退出门删除 |

退出门编号：`G-BOOT`（单一 composition/lifecycle）、`G-TURN`（事件与 turn terminal）、`G-TASK`（唯一 revisioned DAG/attempt settlement）、`G-TOOL`（requested/effective invocation + typed result/artifact）、`G-SUB`（唯一 Subagent lifecycle）、`G-GEN`（workspace/plugin/memory generation）、`G-SURF`（surface capability parity）、`G-LEGACY`（legacy/test-only 删除前的调用与文档证明）、`G-FW-PUB`（framework 公共 API 的合理复用方审查）。

## 2. Composition roots 与主路径

| 定义 / 注册点 | 当前可达路径与注册动作 | 合理复用方 | 处置 | owner / 退出门 |
|---|---|---|---|---|
| `echo-agent-cli/src/main.rs:41` `main`；`:76` `run_tui_or_cli_entry` | `#[tokio::main]`；GUI-only 分支转 `run_desktop_entry`，TUI/default 与 hidden CLI/JSONL/channel 进入同一 headless bootstrap；`:180` bootstrap、`:203` 注册 Task tools、`:210` pool、`:318` TUI service、`:437` hidden service、`:619` shutdown | 所有 EKO surface | `K`（根仍属应用；service 装配需继续合并） | CLI application；`G-BOOT`, `G-SURF` |
| `echo-agent-cli/src-tauri/src/main.rs:3` `main` | 专用 Tauri binary 只调用 `tauri::desktop::run_desktop_entry` | GUI 包装器 | `K` | CLI/Tauri adapter；`G-BOOT` |
| `echo-agent-cli/src/tauri/desktop.rs:68` `run_desktop_entry` | 配置 data root、panic/crash log、调用 `run_desktop` 并传播错误 | macOS/desktop launcher | `K` | Tauri adapter；`G-BOOT` |
| `echo-agent-cli/src/tauri/desktop.rs:124` `run_desktop` | GUI 自己完成 config/env、`AgentRuntime::bootstrap`、watcher、`AppState::from_shared`、Task tools、pool、scheduler/task service、plugin bind、MCP health、Dreaming、Tauri builder | 仅 GUI 但可复用 app-core service | `M`（与 `start_headless_services` 的 composition 继续收敛） | CLI application；`G-BOOT` |
| `echo-agent-cli/src/tauri/mod.rs:104` `build_tauri_app` | `tauri::Builder` 注册 native IPC 与全部 `commands::*`；managed `TauriState` | Tauri 仅做 transport/DTO | `K`（命令不得重新拥有领域 authority） | Tauri adapter；`G-SURF` |
| `echo-agent-cli/src/cli/modes.rs:670` `start_headless_services` | `AppState::from_shared`、scheduler store、`start_scheduler_and_task_service`、agent-control tools、extension reconciliation、delivery recovery；TUI 和 hidden CLI/JSONL/channel 共用 | 非 GUI 长驻入口 | `M`（目标是唯一应用 service owner，不是第二框架 runtime） | CLI application；`G-BOOT`, `G-GEN` |
| `echo-agent-cli/src/cli/modes.rs:790` `run_cli_mode` | 把 shared `HeadlessServices` 投影进 `ReplConfig`，调用 `run_repl` | REPL surface | `K` | CLI adapter；`G-SURF`, `G-TURN` |
| `echo-agent-cli/src/cli/modes.rs:354` `run_jsonl_mode` | JSONL one-shot/extension command 使用 shared foreground resources 与 typed outcome | automation/JSONL consumer | `K`（conditional by invocation） | CLI adapter；`G-TURN` |
| `echo-agent-cli/src/cli/modes.rs:869` `run_channels_mode` | framework `ChannelManager` 注册 QQ/Feishu；每 channel 建 `SessionHandler`，产出 app `AppChannelMessageHandler`；`:987` start_all、`:1032` wait cancel、`:1044` stop_all | framework channel transport + EKO per-conversation state | `K`（channel policy 留应用） | CLI channel adapter；`G-SURF`, `G-TURN` |
| `echo-agent-cli/src/cli/repl.rs:1136` `run_repl` | Reedline 输入、slash commands、HITL session、prepared turn、foreground admission；`:2698`/`:2788`/`:2813` 驱动 app foreground helpers | CLI interactive user | `K` | CLI surface；`G-TURN`, `G-SURF` |
| `echo-agent-cli/src/tui/mod.rs:2007` `run_tui` | terminal RAII、TuiApp 初始化、conversation restore、plugin/theme/model projection、调用 `events::run_event_loop` | TUI renderer | `K`（renderer 不得成为状态权威） | TUI surface；`G-SURF` |
| `echo-agent-cli/src/tui/events.rs:597` `run_event_loop` / `:1963` `dispatch_turn` | TUI input/approval/steer、`AppState::current_control_runtime`、foreground lease、prepared turn、`drive_foreground_chat*`、`TurnSettled` 投影 | TUI full Agent surface | `M`（只保留渲染/输入，generic terminal 不在此推断） | TUI adapter；`G-TURN`, `G-SURF` |
| `echo-agent-app-core/src/runtime.rs` `AgentRuntime::bootstrap` / `init_pool` | `main` 与 `desktop` 共用；创建 framework Agent、MCP/HITL/model/plugin/browser/runtime handles；pool 初始化通过 app `AgentPool` | 所有 EKO roots | `K` | app-core bootstrap owner；`G-BOOT`, `G-GEN` |
| `echo-agent-app-core/src/state.rs` `AppState::from_shared` / `start_scheduler_and_task_service` | app aggregate、stores、workspace scope、scheduler/task service、plugin/extension/browser/watchers；GUI 与 headless 都调用 | 所有 EKO service | `M`（重复初始化/退出 owner 仍需单一化） | app-core state owner；`G-BOOT`, `G-GEN` |
| `echo-agent-app-core/src/infra.rs` `AgentCreateParams`、store/env/browser/MCP helpers | 被 `main`/desktop/runtime 调用；负责 EKO config → framework builder 的适配 | framework builder/provider | `K`（只做转换与产品 hook） | app-core adapter；`G-BOOT` |
| `echo-agent-app-core/src/agent_pool.rs` `AgentPool::from_runtime` / pool leases | `main`/desktop 创建；TUI/CLI/channel/TaskRun 从 pool acquire execution receipt | 多 conversation / Subagent | `K`（EKO product concurrency；不要下沉为 framework policy） | app-core；`G-SUB`, `G-GEN` |
| `echo-agent-app-core/src/tasks/task_runtime/register.rs` | `main`/desktop 在 pool 前后注册 Task tools，并 bind `task_execute` 到 pool | framework Task tools | `K` / thin adapter | app-core adapter；`G-TASK` |
| `echo-agent-app-core/src/foreground_turn.rs` | GUI `src/tauri/commands/chat.rs:879`、TUI events、CLI REPL、channel 调用 admission/lease/settlement helpers | 所有 attended foreground surface | `K`（app active-turn policy） | app-core；`G-TURN`, `G-SURF` |
| `echo-agent-app-core/src/chat_driver.rs` | `drive_chat*` 被 foreground helpers、JSONL、tests 调用；绑定 `ChatResources`、framework `AgentTurnDriver`、event sink、TaskRun projection | 所有 chat-like surface | `M`（保留 EKO orchestration；generic event/terminal 必须 lossless） | app-core；`G-TURN` |
| `echo-agent-app-core/src/run_driver.rs` + `tasks/task_runtime/executor.rs` | background/cron/TaskRun 取得 pool agent，驱动 framework runtime DAG；`BackgroundTaskService`、scheduler fire fn、Task tools 是入口 | unattended TaskRun/cron/background | `M`（应用资源/评审保留，generic retry/cancel/settlement 归 framework 合同） | app-core + framework split；`G-TASK`, `G-SUB` |
| `echo-agent-app-core/src/plugin_runtime.rs` | `runtime` 创建；GUI/headless bind scheduler；pool 使用 `plugin_components` 产出 plugin agents/themes/styles | EKO plugin/skill/hook lifecycle | `K` / `M` generation refresh | app-core；`G-GEN`, `G-SURF` |
| `echo-agent-app-core/src/scheduler/runner.rs` | type alias `FrameworkSchedulerRunner` + EKO `build_fire_fn` callback；`scheduler/mod.rs` re-export framework types | framework SchedulerRunner reuse | `K`（conditional channels/cron） | app-core thin adapter；`G-BOOT`, `G-FW-PUB` |

### 2.1 F4 surface reachability矩阵

| Surface | 入口 | shared app service | foreground driver | framework reuse | 当前边界结论 |
|---|---|---|---|---|---|
| GUI/Tauri | `src-tauri/src/main.rs` → `run_desktop_entry` | `run_desktop` 自建 `AppState`、pool、scheduler/task service | Tauri chat → `drive_foreground_chat_with_ingress` | Agent/Runtime/Task/Tool/Store/HITL/MCP | 可达；与 headless 装配存在两个 root，标记 `M` |
| TUI | `src/main.rs` → `run_tui_or_cli_entry` → `run_tui` | `start_headless_services` | `dispatch_turn` → foreground helpers | 同上 + TUI provider | 可达且是完整 Agent；不得以“终端轻量”删能力 |
| CLI REPL | hidden branch → `run_cli_mode` → `run_repl` | 同一 `HeadlessServices` | REPL foreground helpers | prelude、Agent runtime、Task/Tool/HITL | 可达；renderer 只负责 REPL |
| JSONL | hidden branch → `run_jsonl_mode` | 同一 `HeadlessServices` | one-shot foreground helper | framework event/turn/Tool | 可达；按 invocation conditional |
| channel-only | hidden branch → `run_channels_mode` | 同一 `HeadlessServices` | channel `AppChannelMessageHandler` | framework ChannelManager/SessionHandler | F4 已不再绕过 headless service |
| CLI + channel | hidden branch并行启动 channel + REPL | 同一 `HeadlessServices` | 两个 surface 各自 lease | framework channel transport | 可达；必须保持 companion shutdown 与 exact identity |
| cron/background | scheduler fire fn / BackgroundTaskService | `TaskRuntimeStore` + `run_driver` | unattended，不等待 surface | framework scheduler + RuntimeTaskService | 可达；不得新增第二 DAG loop |

## 3. Framework facade 与合理复用候选

以下“keep”结论基于 framework 自身和合理外部复用方，而不是“EKO 当前是否调用”。EKO 使用 `echo_agent` facade；app-core 不直接依赖 split crate manifest（除了 facade `workspace::*` 的公开路径），符合框架独立性。`echo_agent::workspace` 在 app-core 的 profile/memory 测试和 CLI evolution 中有真实引用，不能按“仅迁移 escape hatch”删除。

| Framework 定义 / facade | F4 真实复用与可达性 | 合理复用方 | 处置 | owner / 退出门 |
|---|---|---|---|---|
| `src/agent/*`；facade `lib.rs:33`、`prelude:143-150` | app-core 65 个文件命中 `echo_agent::agent`；AgentHandle、ReactAgent、EventEnvelope、Steer、Invocation 全部从 bootstrap/turn/pool 进入 | 任意 Agent 产品 | `K` | framework；`G-FW-PUB` |
| `src/runtime.rs`；`prelude` `Turn*` 与 `AgentTurnDriver` | `chat_driver.rs`、`conversation_input.rs`、Task executor 使用 `AgentTurnDriver`/`TurnInputReceipt`/`TurnOutcome` | framework consumers needing one turn lifecycle | `K`；app adapter `M` | framework generic / app projection；`G-TURN` |
| `src/tasks.rs` / `echo-orchestration::tasks::revisioned`；`lib.rs:98`、`advanced:330-333` | app `revisioned_adapter.rs` 直接实现 `RevisionedTaskStore`，使用 `TaskRevisionService`、`TaskDraft`、`TaskGraphCommit`; app 仍投影 `RuntimeTaskEvent` | framework task authoring/execution consumers | `K` framework；app duplicate semantics `M` | framework Task authority；`G-TASK`, `G-FW-PUB` |
| framework `PlanValidator` / `RuntimeTaskService` / `RuntimeDagExecutor` | app planner 明确把 generic dependency/cycle/depth/retry validation交给 framework；executor 文档声明 traversal/cancel/stall在 framework | 任意 DAG agent | `K`；禁止第二 ready frontier | framework；`G-TASK` |
| `src/agent/subagent/*`；Subagent registry/builder/context/control/events/executor | app pool、Task executor、subagent loader/prompt、Tauri projection复用；F4 内部术语均为 Subagent | framework multi-agent products | `K` framework；app policy `K` | framework lifecycle + app role policy；`G-SUB` |
| framework `Tool`/`ToolManager`/`ToolPack`/`ToolCapabilities`/`CommandPolicy`；`prelude:169-180` | app infra、analysis、browser、agent_control、tool projection 直接使用；Tauri仅注册命令 | 任意 tool host | `K` | framework generic / app exposure policy；`G-TOOL` |
| framework `ToolResult`/failure taxonomy/artifact writer-reader | app `tool_execution.rs` 与 `tool_execution_projection.rs` 持久化 GUI detail；shell/files tools产生 canonical artifact | 任意 surface needing typed tool output | `K`；app reader/projection `M` | framework result/artifact facts；`G-TOOL` |
| `memory` Store/ConversationStore/FileStore/InMemoryStore；facade `prelude:206-220` | app pool/state/chat/event/deletion/workspace使用；EKO manifest不启用 sqlite | framework products needing file/sqlite/in-memory variants | `K`（SQLite 不能因 EKO 不用而删） | framework Store menu；`G-FW-PUB` |
| `state` RuntimeStateStore/journal；facade `prelude` + `workspace::state` | app runtime/state/task store依赖 journal/runtime state；CLI evolution使用 workspace state types | framework durable state consumers | `K` | framework; app file policy `K` |
| compression/tokenizer/context | app config、chat diagnostics、prompt contract、manual compression 使用 facade types | framework context products | `K` | framework generic; `G-FW-PUB` |
| `human_loop` / PermissionService；facade `advanced:293-300` | app HITL dispatcher/provider adapters，AgentPool permission service，TUI/REPL/channel registrations | framework attended/unattended hosts | `K`; EKO direct-user policy `K` | framework automated policy / app interaction routing；`G-SURF` |
| `mcp` / `lsp` / `channels` optional modules；facade `lib.rs:79-96`、`advanced:301-310` | app config/runtime、Tauri commands、channel mode直接调用；features由 app manifest选择 | framework integrations and EKO adapters | `K` / `C` by feature | framework transport; app enablement；`G-FW-PUB` |
| framework `scheduler::SchedulerRunner` / `CronTask*` | app `scheduler/mod.rs` re-export、`runner.rs` alias/fire adapter；GUI/headless state持有 runner | any framework host with cron | `K` framework；app callback `K` | framework runner / app fire policy；`G-BOOT` |
| framework `workflow::Graph`/DSL/loader/runtime | app `workflow_service.rs`负责本地 catalog/serialization，执行调用 framework `WorkflowDefinition`/`SharedState`；Tauri panels仍是 surface adapter | framework workflow products | `K` framework；app catalog `K`, Tauri algorithm `M` | framework execution / app persistence；`G-SURF` |
| framework sandbox / security / ScriptExecutionProfile | analysis/analysis_runtime/browser/tool paths reuse；本地 terminal/MCP不加 full-auto门控 | framework execution consumers | `K` | framework primitives; app local policy；`G-FW-PUB` |
| framework plugin/skills/hooks/hooks_bridge | app hook loader/plugin runtime/skills hub使用 framework HookDefinition、Skill types、scheduler callbacks | framework extension hosts | `K` framework；app generation `M` | framework lifecycle primitive / app source precedence；`G-GEN` |
| framework `evolution::MemoryLayerManager`/typed memory | app unified memory/evidence/review integration/pool bridge使用；产品 inbox/Domain policy留app | framework memory/evolution consumers | `K` | framework generic / EKO review policy；`G-GEN` |
| framework `trace`/RunStore/diagnostics | app observability DTO、chat/task event log桥接；framework trace fields不能被 app projection丢弃 | framework operators | `K`；projection `M` | framework trace facts / app retention；`G-TURN` |
| framework `a2a`/`topology`/`eval`/`improve` optional | F4 app 无 production主路径或仅测试/文档间接命中；framework manifest 仍为合理 feature menu | external framework consumers | `C`，不因 EKO 未选而删 | framework；`G-FW-PUB` |
| root `workspace` re-export `lib.rs:121-134` | app `state.rs`、`memory_bridge.rs`、CLI evolution 使用 `echo_agent::workspace::*`；可见 migration path且有真实调用 | split-crate adopters and EKO adapters | `K` | framework facade；`G-FW-PUB` |
| `prelude` / `advanced` re-export surfaces | F4 大量 `prelude::*`；`advanced` 主要是公开 convenience surface，不能从 EKO absence 推 dead | external consumers | `K`；文档/compile probe另审 | framework facade；`G-FW-PUB` |
| framework `testing` mocks | app tests 与 examples 使用 `MockLlmClient`/`MockAgent`；仅 test/feature reachable | framework downstream tests | `C` | framework testing feature；`G-FW-PUB` |

### 3.1 不应跨边界移动的产品概念

以下概念在 F4 明确属于 EKO，不得为了“统一”塞进 `echo-agent`：`WorkspaceId`/workspace generation、`DomainProfile`、review/acceptance/worktree、EKO file retention、Tauri/TUI/CLI/channel renderer、local webhook labels、surface capability manifest、AgentPool 的产品资源 ceiling、EKO 的 prepared user input/artifact path、extension source precedence，以及 direct-user terminal/file/MCP interaction policy。Framework 只提供通用 handle、Store、Tool、HITL、scheduler、cancellation/deadline、DAG、Subagent、artifact 和 hook 原语。

## 4. App-core 151-file inventory（逐文件）

说明：`D/R` 给出定义与注册锚点；`Reach` 给出 production/test/feature 可达性；`Reuse` 给出合理复用方；`Disp` 为 R0 处置。除明确标为 `T` 的文件外，所有 `pub mod` 文件均已从 `lib.rs` 或父 `mod.rs` 注册；“live”表示静态调用路径已找到，不等于本轮执行过动态运行。

### 4.1 root（56）

| 文件 | D/R | Reach | Reuse | Disp | owner / exit gate |
|---|---|---|---|---|---|
| `src/lib.rs` | module registry `:1-74` | live | app facade | K | app-core; G-BOOT |
| `src/agent_control.rs` | `lib.rs:1` | live：state/root register | framework AgentHandle/Tool + EKO control | K | app-core; G-SURF |
| `src/agent_handle.rs` | `lib.rs:2` re-export | live | framework `AgentHandle` | K | thin adapter; G-FW-PUB |
| `src/agent_pool.rs` | `lib.rs:3` | live：main/desktop/pool acquire | framework Agent/Store/Tool/HITL | K | app-core product pool; G-SUB |
| `src/agent_router.rs` | `lib.rs:4` | live：CLI/Tauri router commands | framework Agent/turn primitives | K | app-core cross-workspace policy; G-GEN |
| `src/analysis.rs` | `lib.rs:5` | live：Tauri/CLI analysis | framework Tool/Sandbox/Store | K | app domain; G-SURF |
| `src/analysis_runtime.rs` | `lib.rs:6` | feature/surface live；tests | framework ScriptExecutionProfile/Sandbox/Tools | K | app analysis runtime; G-FW-PUB |
| `src/attachments.rs` | `lib.rs:7` | live：GUI/TUI/CLI/channel ingress | framework Message/ContentPart + app artifact | K | app ingress; G-TURN |
| `src/chat_driver.rs` | `lib.rs:10` | live：all chat surfaces | framework AgentTurnDriver/EventEnvelope/Tool | M | app driver; G-TURN |
| `src/chat_event_log.rs` | `lib.rs:11` | live：surface sinks/replay | framework journal/EventEnvelope | M | app durable projection; G-TURN |
| `src/chat_resources.rs` | `lib.rs:12` | live：driver task-local | framework Cancellation/Memory/Tool | K | app adapter; G-TURN |
| `src/config.rs` | `lib.rs:13` | live：all roots | framework config/LLM | K | app config policy; G-BOOT |
| `src/config_discovery.rs` | `lib.rs:14` | live：config commands/bootstrap | framework paths/LLM | K | app config discovery; G-BOOT |
| `src/config_watcher.rs` | `lib.rs:15` | live：main/desktop watcher | framework Hook/Plugin types | K | app reload owner; G-GEN |
| `src/context_window.rs` | `lib.rs:16` | live：Tauri/CLI diagnostics | framework tokenizer/context | K | app projection; G-TURN |
| `src/conversation_deletion.rs` | `lib.rs:17` | live：state/conversation commands | framework ConversationStore | K | app deletion policy; G-GEN |
| `src/conversation_input.rs` | `lib.rs:18` | live：GUI/TUI/CLI/channel tracked ingress | framework `TurnInputReceipt`/steer state | K | app ingress adapter; G-TURN |
| `src/conversation_projection.rs` | `lib.rs:19` | live：state/restore/UI | framework ConversationStore/Message | K | app projection; G-SURF |
| `src/data_root.rs` | `lib.rs:20` | live before stores | framework path/fs helpers | K | app local root; G-BOOT |
| `src/developer_commands.rs` | `lib.rs:21` | live：CLI/TUI developer commands | framework Tool/Agent | K | app developer surface; G-SURF |
| `src/diff.rs` | `lib.rs:22` | live：CLI/Tauri file diff | framework utils/Git-adjacent primitives | M | app diff authority; G-SURF |
| `src/error.rs` | `lib.rs:23` | live throughout app | framework error conversion | K | app error boundary; G-TURN |
| `src/extension_commands.rs` | `lib.rs:26` | live：Tauri/CLI extension commands | framework skills/hooks | K | app command policy; G-GEN |
| `src/extension_control.rs` | `lib.rs:27` | live：state/root reconciliation | framework Skill/Hook registry | M | app generation owner; G-GEN |
| `src/foreground_turn.rs` | `lib.rs:28` | live：GUI/TUI/CLI/channel | framework turn/cancel | K | app active-turn owner; G-TURN |
| `src/hook_config_loader.rs` | `lib.rs:30` | live：watcher/plugin reload | framework HookDefinition | K | app source precedence; G-GEN |
| `src/infra.rs` | `lib.rs:31` | live：all roots | framework Agent/Store/MCP/LLM | M | app bootstrap adapter; G-BOOT |
| `src/instruction_provider.rs` | `lib.rs:32` | live：bootstrap/prompt | framework Agent config | K | app prompt policy; G-GEN |
| `src/manual_compression.rs` | `lib.rs:33` | live：Tauri/CLI command | framework compressor/Context | K | app UX policy; G-SURF |
| `src/mcp_config_runtime.rs` | `lib.rs:34` | live：main/desktop config resolution | framework MCP config | K | app precedence; G-BOOT |
| `src/model_config.rs` | `lib.rs:35` | live：bootstrap/TUI/GUI providers | framework LLM capabilities | K | app catalog; G-BOOT |
| `src/permission.rs` | `lib.rs:38` | live：state/TUI/JSONL | framework PermissionMode/ToolPermission | K | app policy adapter; G-SURF |
| `src/plugin_components.rs` | private `lib.rs:39`; used by pool/runtime | live internal | framework Skill/Hook/Tool | K | app plugin component bridge; G-GEN |
| `src/plugin_runtime.rs` | `lib.rs:40` | live：root/pool/scheduler | framework plugin/skills/scheduler | M | app generation owner; G-GEN |
| `src/prepared_turn.rs` | `lib.rs:41` | live：all user input surfaces | framework Message/ContentPart | K | app ingress; G-TURN |
| `src/product_data_io.rs` | `lib.rs:42` | live：TaskRun/domain tools | framework Tool/Store primitives | K | app product-data policy; G-SURF |
| `src/prompt_contract.rs` | `lib.rs:45` | live：prompt validation/tests | framework tokenizer | K | app prompt contract; G-TURN |
| `src/research.rs` | `lib.rs:46` | live：Tauri/CLI research | framework HTTP/Tool/Store | K | app domain; G-SURF |
| `src/research_connectors.rs` | `lib.rs:47` | live：research commands | framework HTTP/LLM/Tool | K | app domain; G-SURF |
| `src/research_tool.rs` | `lib.rs:48` | live：registered research tools | framework Tool | K | app tool adapter; G-TOOL |
| `src/run_driver.rs` | `lib.rs:49` | live：background/cron/TaskRun | framework turn + app TaskRuntime | M | app unattended driver; G-TASK |
| `src/runtime.rs` | `lib.rs:72` | live：bootstrap/lifecycle | framework Agent runtime primitives | K | app lifecycle owner; G-BOOT |
| `src/structured_extraction.rs` | `lib.rs:53` | live：analysis/research extraction | framework Tool/schema | K | app domain adapter; G-SURF |
| `src/subagent_loader.rs` | `lib.rs:54` | live：AgentRuntime/pool source loading | framework Subagent registry/types | K | app role catalog adapter; G-SUB |
| `src/subagent_prompt.rs` | `lib.rs:55` | live：prompt assembly/TaskRun | framework Subagent prompt compiler | K | app role/prompt policy; G-SUB |
| `src/surface_contract.rs` | `lib.rs:56-57 cfg(test)` | test-only parity contract | app surface tests | C | app tests; G-LEGACY, G-SURF |
| `src/terminal.rs` | `lib.rs:59` | live：Tauri/TUI direct terminal | framework shell/PTY primitives | K | app direct-user interaction; G-SURF |
| `src/tool_execution.rs` | `lib.rs:60` | live：Tauri detail/repository | framework ToolResult/artifact | M | app durable projection; G-TOOL |
| `src/tool_execution_projection.rs` | `lib.rs:61` | live：event/task → detail | framework AgentEvent/ToolResult | M | app lossless adapter; G-TOOL |
| `src/tool_exposure.rs` | `lib.rs:62` crate-private; registered by infra/state | live：Agent setup | framework ToolManager/ToolPack | M | app exposure policy; G-TOOL |
| `src/turn_context.rs` | `lib.rs:63` | live：turn-local registration | framework runtime context | K | app turn scope; G-TURN |
| `src/unified_memory.rs` | `lib.rs:65` | live：state/memory commands | framework Store/MemoryLayerManager | K | app memory policy; G-GEN |
| `src/utils.rs` | `lib.rs:66` | live helper calls | framework fs/time/canonical JSON | K | app utility boundary; G-BOOT |
| `src/workflow_service.rs` | `lib.rs:68` | live：Tauri/CLI workflow commands | framework Workflow Graph/Definition | M | app catalog; G-SURF |
| `src/workspace_routing.rs` | `lib.rs:70` | live：agent/workspace prompts | framework Agent/Message | K | app workspace policy; G-GEN |

### 4.2 auto_memory / browser / evolution / export / hitl / observability / output / profiles / project

| 文件 | D/R | Reach | Reuse | Disp | owner / exit gate |
|---|---|---|---|---|---|
| `src/auto_memory/mod.rs` | `auto_memory/mod.rs` + `lib.rs:8` | live：review/inbox | framework typed memory | K | app memory inbox; G-GEN |
| `src/auto_memory/policy.rs` | parent mod | live + tests | framework Store metadata | K | app observation policy; G-GEN |
| `src/browser/config.rs` | `browser/mod.rs` + `lib.rs:9` | live/feature | framework MCP config | K | app browser config; G-SURF |
| `src/browser/error.rs` | parent mod | live | framework error | K | app adapter; G-SURF |
| `src/browser/event.rs` | parent mod | live：Tauri/browser sink | framework event primitives | K | app projection; G-SURF |
| `src/browser/mod.rs` | `lib.rs:9` | live：runtime/bootstrap/Tauri | framework MCP/Tool/HITL | K | app BrowserRuntime; G-BOOT |
| `src/browser/risk.rs` | parent mod | live：browser approval | framework ToolParameters/HITL | K | app local risk policy; G-SURF |
| `src/browser/session.rs` | parent mod | live：session manager | framework MCP client | K | app browser session; G-GEN |
| `src/browser/sidecar.rs` | parent mod | conditional browser sidecar | framework MCP transport | C | app platform sidecar; G-BOOT |
| `src/evolution/dashboard.rs` | `evolution/mod.rs` + `lib.rs:24` | live：Tauri/CLI dashboard | framework memory/trace | K | app observability UX; G-SURF |
| `src/evolution/evidence.rs` | parent mod | live：review inbox | framework typed memory | K | app evidence policy; G-GEN |
| `src/evolution/hook_fire.rs` | parent mod | live：hook observer | framework HookRegistry | K | app integration; G-GEN |
| `src/evolution/mod.rs` | `lib.rs:24` | live registry | framework evolution primitives | K | app product evolution; G-GEN |
| `src/evolution/review_integration.rs` | parent mod | live：bootstrap/Dreaming/TaskRun | framework memory/Store | K | app review generation; G-GEN |
| `src/evolution/rule_promoter.rs` | parent mod | live：CLI/Tauri evolution | framework memory/skills | K | app explicit promotion; G-GEN |
| `src/export/latex.rs` | `export/mod.rs` + `lib.rs:25` | live：export command | framework artifact/fs | K | app export format; G-SURF |
| `src/export/mod.rs` | `lib.rs:25` | live | app export facade | K | app; G-SURF |
| `src/hitl/channel_provider.rs` | `hitl/mod.rs` + `lib.rs:29` | live channel HITL | framework HumanLoopProvider | K | app channel transport; G-SURF |
| `src/hitl/dispatcher.rs` | parent mod | live all surface registrations | framework HumanLoop types | K | app provider routing; G-SURF |
| `src/hitl/mod.rs` | `lib.rs:29` | live | framework HITL | K | app adapter; G-SURF |
| `src/hitl/repl_provider.rs` | parent mod | live CLI | framework HumanLoopProvider | K | app REPL provider; G-SURF |
| `src/hitl/tui_provider.rs` | parent mod | live TUI | framework HumanLoopProvider | K | app TUI provider; G-SURF |
| `src/observability/diagnostics.rs` | `observability/mod.rs` + `lib.rs:36` | live Tauri/CLI | framework Trace/RunStore | M | app diagnostic projection; G-TURN |
| `src/observability/mod.rs` | `lib.rs:36` | live | framework trace facade | K | app; G-TURN |
| `src/observability/types.rs` | parent mod | live DTO/TS | framework trace fields | K | app DTO; G-TURN |
| `src/output/format.rs` | `output/mod.rs` + `lib.rs:37` | CLI/TUI live | app renderer | K | app UI; G-SURF |
| `src/output/markdown.rs` | parent mod | CLI/TUI live | app renderer | K | app UI; G-SURF |
| `src/output/mod.rs` | `lib.rs:37` | live facade (`allow(dead_code)` present) | app renderer | C | app UI; G-LEGACY |
| `src/output/spinner.rs` | parent mod | CLI/TUI conditional | app renderer | C | app UI; G-LEGACY |
| `src/output/syntax.rs` | parent mod | CLI/TUI live | app renderer | K | app UI; G-SURF |
| `src/output/table.rs` | parent mod | CLI/TUI live | app renderer | K | app UI; G-SURF |
| `src/output/theme.rs` | parent mod | TUI/CLI live | app theme | K | app UI; G-SURF |
| `src/profiles/manager.rs` | `profiles/mod.rs` + `lib.rs:43` | live CLI/Tauri | app config/files | K | app profile policy; G-GEN |
| `src/profiles/mod.rs` | `lib.rs:43` | live | app profile facade | K | app; G-GEN |
| `src/profiles/types.rs` | parent mod | live DTO/config | app product policy | K | app; G-GEN |
| `src/project/coding_loop.rs` | `project/mod.rs` + `lib.rs:44` | CLI/TUI live | framework Agent/Tool | K | app coding policy; G-SURF |
| `src/project/context.rs` | parent mod | live | framework Agent context | K | app coding context; G-GEN |
| `src/project/detector.rs` | parent mod | live | fs/Git primitives | K | app workspace detection; G-GEN |
| `src/project/gitignore.rs` | parent mod | live | fs primitives | K | app file policy; G-SURF |
| `src/project/mod.rs` | `lib.rs:44` | live | app project facade | K | app; G-GEN |
| `src/project/prompt.rs` | parent mod | live bootstrap/TaskRun | framework prompt/Agent | K | app prompt assembly; G-TURN |
| `src/project/test_runner.rs` | parent mod | CLI/Tauri conditional | process/sandbox primitives | K | app developer tool; G-SURF |

### 4.3 scheduler / skills_hub / state / tasks

| 文件 | D/R | Reach | Reuse | Disp | owner / exit gate |
|---|---|---|---|---|---|
| `src/scheduler/mod.rs` | `lib.rs:50` | live | framework SchedulerRunner/CronTask | K | thin app facade; G-FW-PUB |
| `src/scheduler/runner.rs` | parent mod | live GUI/headless | framework SchedulerRunner + app fire fn | K | app callback adapter; G-BOOT |
| `src/scheduler/task.rs` | parent mod | live alias/CLI compatibility | framework CronTaskStore | C | app compatibility; G-LEGACY |
| `src/skills_hub/enabled_skills.rs` | `skills_hub/mod.rs` + `lib.rs:51` | live | framework Skill/Hook | K | app skills marketplace; G-GEN |
| `src/skills_hub/install.rs` | parent mod | live CLI/Tauri | fs + framework skill loader | K | app install policy; G-GEN |
| `src/skills_hub/mod.rs` | `lib.rs:51` | live | app skills facade | K | app; G-GEN |
| `src/skills_hub/registry.rs` | parent mod | live | framework SkillDescriptor | K | app catalog; G-GEN |
| `src/state.rs` | `lib.rs:52` | live aggregate + tests | framework Store/RuntimeState/HITL/Agent | M | app state authority; G-BOOT, G-GEN |
| `src/state/reliability_contracts.rs` | `state.rs:5931 mod` | test-only | app state tests | C | app tests; G-LEGACY |
| `src/tasks/background.rs` | `tasks/mod.rs` + `lib.rs:58` | live background task API | framework task/Tool | M | app compatibility projection; G-TASK |
| `src/tasks/mod.rs` | `lib.rs:58` | live | app Task facade | K | app; G-TASK |
| `src/tasks/service.rs` | parent mod | live scheduler/background service | framework TaskStatus/Store | M | app background policy; G-TASK |

### 4.4 tasks/task_runtime（30）

| 文件 | D/R | Reach | Reuse | Disp | owner / exit gate |
|---|---|---|---|---|---|
| `src/tasks/task_runtime/boot_reconciler.rs` | `task_runtime/mod.rs:31` | live boot recovery | framework Task/Store | K | app recovery projection; G-TASK |
| `src/tasks/task_runtime/command_cells.rs` | `mod.rs:32` | live TaskRun shell/awaiter | framework BackgroundCommandManager/Tool | K | app execution resource; G-TASK |
| `src/tasks/task_runtime/compact_context.rs` | `mod.rs:33` | live continuation/context | framework TaskStatus/Context | K | app TaskRun context; G-TURN |
| `src/tasks/task_runtime/completion_gate.rs` | `mod.rs:34` | live task completion/review | framework TaskExecution/Tool results | K | app acceptance policy; G-TASK |
| `src/tasks/task_runtime/continuation.rs` | `mod.rs:35` | live pause/resume/continuation | framework turn/cancel/Task | M | app continuation policy; G-TASK |
| `src/tasks/task_runtime/event_rebuild.rs` | `mod.rs:36` | live projection rebuild | framework TaskStatus/journal | K | app event fold; G-TASK |
| `src/tasks/task_runtime/execution_target.rs` | `mod.rs:37` | live target resolution | framework Agent/Subagent | K | app target policy; G-SUB |
| `src/tasks/task_runtime/executor.rs` | `mod.rs:38` | live TaskRun executor | framework RuntimeTaskService/turn/Subagent | M | split generic vs EKO resource policy; G-TASK, G-SUB |
| `src/tasks/task_runtime/file_shadow.rs` | `mod.rs:40` private | live internal file safety | fs/worktree primitives | K | app file shadow; G-SURF |
| `src/tasks/task_runtime/file_store.rs` | `mod.rs:40` private | live internal blocking I/O | app store/fs | K | app persistence helper; G-TASK |
| `src/tasks/task_runtime/history_projection.rs` | `mod.rs:41` private | live event history projection | app TaskRun DTO | K | app projection; G-TASK |
| `src/tasks/task_runtime/hook_event_dispatcher.rs` | `mod.rs:42` | live hooks | framework HookRegistry | K | app hook adapter; G-GEN |
| `src/tasks/task_runtime/ledger.rs` | `mod.rs:43` | live progress export | app TaskRun files | K | app progress projection; G-SURF |
| `src/tasks/task_runtime/long_horizon_contracts.rs` | `mod.rs:61 cfg(test)` | test-only | app contract tests | C | app tests; G-LEGACY |
| `src/tasks/task_runtime/memory_bridge.rs` | `mod.rs:44` | live TaskRun → memory | framework MemoryLayerManager/Store | K | app memory policy; G-GEN |
| `src/tasks/task_runtime/mod.rs` | `lib.rs:58` parent registry | live | app TaskRuntime facade | K | app; G-TASK |
| `src/tasks/task_runtime/planner.rs` | `mod.rs:45` | live plan file ownership | framework PlanValidator | K | app file ownership policy; G-TASK |
| `src/tasks/task_runtime/profiles.rs` | `mod.rs:46` | live DomainProfile templates | framework Subagent prompt | K | app product policy; G-SUB |
| `src/tasks/task_runtime/register.rs` | `mod.rs:47` | live root registration | framework Task tools/TaskRevisionService | K | thin adapter; G-TASK |
| `src/tasks/task_runtime/review.rs` | `mod.rs:48` | live review/retry policy | framework Task/Tool | K | app review policy; G-TASK |
| `src/tasks/task_runtime/revisioned_adapter.rs` | `mod.rs:49` | live `RevisionedTaskStore` impl | framework TaskRevisionService | K | thin lossless adapter; G-TASK |
| `src/tasks/task_runtime/root_authority.rs` | `mod.rs:50` private | live internal root ownership | app TaskRun store | K | app authority; G-TASK |
| `src/tasks/task_runtime/run_authority.rs` | `mod.rs:51` private | live journal/run ownership | framework journal + app files | K | app authority; G-TASK |
| `src/tasks/task_runtime/store.rs` | `mod.rs:52` | live canonical file store | framework journal/RuntimeTaskMutation | M | app durable store; generic recovery facts must remain typed; G-TASK |
| `src/tasks/task_runtime/subagent_control.rs` | `mod.rs:53` | live GUI/TUI/CLI control | framework SubagentControlRegistry | K | app command adapter; G-SUB |
| `src/tasks/task_runtime/task_execute_tool.rs` | `mod.rs:54` | live registered Task tool | framework Tool/TaskRevisionService | K | app Task tool adapter; G-TASK |
| `src/tasks/task_runtime/task_tools.rs` | `mod.rs:55` | live task_create/update/list | framework Task tools + app metadata | K | app public Task API adapter; G-TASK |
| `src/tasks/task_runtime/turn_lifecycle.rs` | `mod.rs:56` private | live internal Task turn | framework TurnOutcome | M | app lifecycle bridge; G-TURN |
| `src/tasks/task_runtime/types.rs` | `mod.rs:57` | live DTO/persistence schema | framework Task/Tool/Subagent facts | K | app product schema; G-TASK |
| `src/tasks/task_runtime/worktree.rs` | `mod.rs:58` | live worktree/review cleanup | Git/fs primitives | K | app worktree policy; G-SURF |

### 4.5 types / unified webhooks / workspace

| 文件 | D/R | Reach | Reuse | Disp | owner / exit gate |
|---|---|---|---|---|---|
| `src/types/mod.rs` | `lib.rs:64` | live DTO facade | serde/ts-rs | K | app API contract; G-SURF |
| `src/types/request.rs` | parent mod | live Tauri/CLI requests | app policy + framework inputs | K | app DTO; G-SURF |
| `src/types/response.rs` | parent mod | live Tauri/CLI responses | framework result facts | K | app DTO; G-SURF |
| `src/webhook/emitter.rs` | `webhook/mod.rs` + `lib.rs:67` | live event observer | framework event/Store | K | app delivery policy; G-TURN |
| `src/webhook/events.rs` | parent mod | live DTO | framework event facts | K | app webhook contract; G-TURN |
| `src/webhook/mod.rs` | `lib.rs:67` | live | app webhook facade | K | app; G-TURN |
| `src/workspace/layout.rs` | `workspace/mod.rs` + `lib.rs:69` | live path/layout | framework fs/path | K | app workspace root; G-GEN |
| `src/workspace/migration.rs` | parent mod | conditional legacy migration + CLI/Tauri commands | framework Store/fs | C | app legacy bridge; G-LEGACY |
| `src/workspace/mod.rs` | `lib.rs:69` | live WorkspaceId/scope | framework invocation cwd only | K | app workspace authority; G-GEN |
| `src/workspace/registry.rs` | parent mod | live workspace CRUD/switch | fs/Store | K | app workspace registry; G-GEN |
| `src/workspace/runtime.rs` | `mod.rs` private | live workspace runtime files | framework Store/State | K | app workspace runtime; G-GEN |
| `src/workspace/templates.rs` | parent mod | live workspace creation | app templates/fs | K | app workspace UX; G-SURF |

### 4.6 Inventory 计数与特殊状态

| 分类 | 数量 | 证据 |
|---|---:|---|
| `echo-agent-app-core/src/*.rs`（F4 exact） | 151 | `git -C .../echo-agent-cli ls-tree -r --name-only 4462b8a -- echo-agent-app-core/src \| rg '\\.rs$' \| wc -l` |
| app-core examples | 2 | `echo-agent-app-core/examples/lh6_concurrency_soak.rs`, `task_runtime_soak.rs` |
| app-core integration tests | 2 | `echo-agent-app-core/tests/f0_agent_control_characterization.rs`, `runtime_state_e2e.rs` |
| app-core tracked Rust total | 155 | exact F4 tree |
| predecessor `7bc9174` source count | 149 | same `ls-tree` command at predecessor |
| F4 additions relative to predecessor | 2 | `agent_control.rs`, `conversation_input.rs` |
| inventory disposition `K` | 124 | 当前 owner 正确，保留 |
| inventory disposition `M` | 19 | 未来收敛 generic 语义或 composition authority |
| inventory disposition `C` | 8 | feature/test/兼容路径，等待退出门 |
| inventory disposition `D` | 0 | R0 不直接删除 source |

`surface_contract.rs` 是 `lib.rs:56-57` 的 `#[cfg(test)]` 模块；`state/reliability_contracts.rs` 与 `tasks/task_runtime/long_horizon_contracts.rs` 也只由 test module 注册。它们不是生产 composition root，也不能被报告为 production capability。`plugin_components.rs` 虽然不是 `pub`，但被 `agent_pool.rs` 与 `plugin_runtime.rs` 真实调用，属于活的内部实现，不是删除候选。

## 5. 重复 authority、迁移候选与退出门

| 主题 | 当前唯一/重复事实 | R0 判定 | R1 退出门 |
|---|---|---|---|
| Agent/turn | framework `AgentTurnDriver` + app `foreground_turn`/`chat_driver`；surface 只应持有 lease/sink | app policy 正确；driver 中 generic terminal 不能再分叉 | `G-TURN`：envelope identity/sequence/terminal 在 GUI/TUI/CLI/channel/replay 字段级 round-trip |
| Task graph | framework `TaskRevisionService`/`PlanValidator`/`RuntimeTaskService`；app `TaskRuntimeStore` 是 EKO 文件 projection | F4 主路径已复用 framework DAG；app executor/store 仍厚，标 `M` | `G-TASK`：一次 revision/attempt/claim/settlement；删除第二 ready/retry/terminal loop |
| Subagent | framework registry/control/executor；app pool/DomainProfile/worktree/review | app pool/role policy 正确；不得让 Team/Handoff 或 pool另建 lifecycle | `G-SUB`：每个 dispatch 带 canonical invocation/cancel/deadline/outcome |
| Tool/artifact | framework ToolResult/artifact facts；app detail repository/projection | app 负责持久化/UX；不得丢 requested/effective/typed terminal/digest | `G-TOOL`：完整 artifact descriptor 与 typed terminal 穿过 Tauri/TUI/CLI/channel/replay |
| Store/recovery | framework Store/journal primitives；app file-backed Task/Conversation/Workspace generations | SQLite 等 framework option 保留；EKO 不启用 SQLite | `G-GEN`：corrupt/partial state quarantine、generation/tombstone、跨 store atomicity |
| Plugin/memory | framework Skill/Hook/Memory primitives；app source precedence/pool generation/review inbox | placement 正确；reload/rollback 需统一 | `G-GEN`：primary/current/future agent 同一 source generation |
| Scheduler | framework runner/store；app `build_fire_fn` / EKO TaskRun launch | `scheduler/runner.rs` 是薄 adapter；不新增 app scheduler loop | `G-BOOT`：GUI/headless/channel 长驻服务启动一次、可取消、可 join |
| Workflow/diff | framework Graph/DSL；app workflow catalog 与 GUI/CLI projections，Tauri panels仍是 adapter | app catalog可保留；GUI算法不可成为第二 domain authority | `G-SURF`：所有 surface 走同一 app service，Tauri 只做 IPC |
| Permissions | framework automated Tool permission；app direct-user local terminal/file/MCP policy | 禁止把 `full-auto/default` 套到用户主动交互 | `G-SURF`：automation 与 direct-user 路径分离且无 secrets/log 泄漏 |

## 6. 明确的 keep / migrate / delete / conditional ledger

### Keep（当前无需迁移）

- framework 的 `SqliteStore`、`SqliteConversationStore`、各种 Compressor、MCP/LSP/A2A/channel、Workflow、Tool domain、testing mocks：它们有合理外部复用方；EKO 不启用 SQLite 不是删除理由。
- EKO 的 `WorkspaceId`、`WorkspaceExecutionScope`、DomainProfile、worktree/review/acceptance、surface sinks、Tauri/TUI/CLI/channel composition、AgentPool、prepared-turn ingress、local webhook and artifact retention：这些是产品策略。
- framework facade 的 `prelude`、`advanced`、`workspace` 以及 split-crate path：F4 有真实 app/CLI 调用和外部迁移价值。

### Migrate / converge（只登记，不在 R0 实施）

- `chat_driver.rs` / `chat_event_log.rs`：保留 app orchestration 和 durable UI projection；把 generic event identity/order/terminal 继续以 framework `EventEnvelope`/`TurnOutcome` 为唯一事实。
- `tool_execution.rs` / `tool_execution_projection.rs`：保留 app repository、retention、surface detail；移除任何 raw-path reader、stream-over-final 或 parent-status inference，完整读取 framework artifact descriptor。
- `tasks/task_runtime/executor.rs` / `run_driver.rs`：保留 EKO write/shell/LLM limits、worktree/review/acceptance；generic DAG traversal、retry/cancel/join/settlement 只能调用 framework runtime service。
- `state.rs` / `runtime.rs` / `src/tauri/desktop.rs` / `src/cli/modes.rs`：形成一个应用 lifecycle/composition owner，GUI 与 headless 只提供 surface-specific providers/sinks。
- `plugin_runtime.rs` / `extension_control.rs` / memory bridges：统一 source generation、partial failure receipt、rollback/reconcile 后再删除 additive refresh/frozen bootstrap remnants。
- `diff.rs` / `workflow_service.rs` 与 Tauri panels：保留 app service，删除 command 层重复算法；Tauri 命令只 deserialize → call → DTO。

### Delete（已确认但需 R1 退出门）

- `surface_contract.rs`、`state/reliability_contracts.rs`、`tasks/task_runtime/long_horizon_contracts.rs`：只有在同语义 executable parity/contract fixture 已进入维护测试入口后，才删除重复 test-only scaffolding；当前 `C` 而非立即删除。
- `scheduler/task.rs` 的 compatibility alias、workspace legacy migration 旧分支、Task tools 的明确 legacy wrapper：先证明无生产/外部调用并同步删文档；`G-LEGACY` 未满足前保留。
- 任何 framework public Task/Store/Compressor/Integration API：只有 framework 内部与合理外部复用方均不需要，或被覆盖全部能力的新 authority 取代，才可 `D`；EKO 未调用不构成退出理由。

### Conditional

- `browser/sidecar.rs`、`analysis_runtime.rs` 的 feature/平台路径、`output/*` 的 renderer-only helpers、framework optional modules、test fixtures/examples：保持 conditional，分别由 feature/test gate 证明，不把不可达条件路径误报成 production capability。

## 7. 风险与后续验收要求

### 当前风险（静态审计已确认的边界风险）

1. **两个 app composition families**：GUI `run_desktop` 与 headless `start_headless_services` 都创建/绑定 state、scheduler/task service；未来改动若只改一个 root，会重新产生 surface drift。
2. **厚适配器风险**：Task executor、chat driver、Tool projection 都同时处理 framework facts 与 EKO policy；任何把 typed outcome 压成 bool/string 的改动都会造成跨 surface 不一致。
3. **generation/recovery 风险**：workspace、plugin、memory、TaskRun 和 conversation 都有持久化/恢复语义；必须先有 generation/receipt/quarantine 验收，再清理旧路径。
4. **计数口径风险**：文档/计划若继续写“149”而没有说明 predecessor，会漏掉 F4 新增的 `agent_control.rs` 与 `conversation_input.rs`；以后审计应使用 exact `ls-tree` 计数。
5. **动态可达性边界**：本轮是静态 inventory；没有把 cargo test、Tauri launch、真实 QQ/Feishu、MCP child、browser sidecar 或 full feature matrix 的未执行状态写成 release evidence。

### R1 必须先过的退出门

- `G-BOOT`：GUI/TUI/CLI/JSONL/channel/cron/background 的 bootstrap、rollback、cancel、join 都由一个 app owner 管理；没有 branch-local service owner。
- `G-TURN`：framework envelope 与 app `ChatDriverEvent`/`TurnOutcome` 字段级 round-trip；每个 turn 恰有一个 terminal，失败/取消/断开不被重标为成功。
- `G-TASK`：Task spec → revision adapter → persistence → execution → recovery round-trip；单一 DAG、attempt、claim、retry、settlement 权威。
- `G-SUB`：Subagent catalog/source generation/attempt/cancel/deadline/outcome 在 pool、TaskRun、TUI、Tauri、CLI、channel 一致；禁止新增 worker 术语或第二 lifecycle。
- `G-TOOL`：requested/effective invocation、typed failure、stream observation、complete artifact descriptor、digest/retention/cursor 在所有 surface 与 replay 一致。
- `G-GEN`：workspace/conversation/run/plugin/memory generation 的 commit、quarantine、tombstone、rollback 与 late-write fencing 有可执行测试。
- `G-SURF`：由 capability manifest/typed contract 驱动 surface parity；删除一个真实 binding 或事件字段时测试失败。
- `G-LEGACY` / `G-FW-PUB`：删除前完成全仓库调用搜索、reasonable external consumer judgment、examples/docs/feature 矩阵；不以 CLI absence 作为 framework deletion 依据。

## 8. 审计命令与验证状态

本次已执行的只读/文档验证命令（均针对 integration 路径或当前 R0 worktree）：

```text
git -C /Users/ls/.codex/worktrees/f24e/integration/echo-agent-cli ls-tree -r --name-only 4462b8a -- echo-agent-app-core/src | rg '\\.rs$' | wc -l
git -C /Users/ls/.codex/worktrees/f24e/integration/echo-agent-cli ls-tree -r --name-only 4462b8a -- echo-agent-app-core | rg '\\.rs$' | wc -l
git -C /Users/ls/.codex/worktrees/f24e/integration/echo-agent-cli grep -n <pattern> 4462b8a -- <path>
git -C /Users/ls/.codex/worktrees/f24e/integration/echo-agent show 302453b:<path>
git -C /Users/ls/.codex/worktrees/f24e/integration/echo-agent-cli diff --name-status 7bc9174 4462b8a -- echo-agent-app-core/src
git diff --check -- docs/2026-08-28-current-framework-application-boundary-audit.md
git diff --stat -- docs/2026-08-28-current-framework-application-boundary-audit.md
```

未执行 cargo/test/Tauri/browser/channel 动态验证：本 R0 任务只要求边界 inventory，且明确不得在该 worktree 初始化全部 submodule；动态验证应由后续 R1/质量门禁在完整 integration 环境执行。未将这些未执行命令当作通过证据。

## 9. 变更边界

本提交只新增本文件。没有修改：

- `echo-agent` framework API、Cargo manifests、features、examples 或 tests；
- `echo-agent-cli` 源码、Cargo manifests、generated TypeScript、examples 或 website；
- 其他 worktree 或源 checkout。

Submodule 初始化曾短暂开始，随后在本 worktree 立即执行 `git submodule deinit -f --all` 清理；最终审计读取 integration 的绝对路径和 Git 对象，避免继续占用磁盘。
