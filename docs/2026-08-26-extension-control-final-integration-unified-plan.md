# EKO Extension Control 与当前重构最终集成统一计划

> 状态：Milestone A/B completed and released
>
> 日期：2026-08-26，发布收口更新于 2026-08-27
>
> 类型：顶层跨仓库阶段性工程计划，不是 `echo-agent` 公共 API 或 EKO 产品行为的长期事实源
>
> 当前执行范围：Milestone A/B 已完成；下一执行入口为交互收敛 F0 characterization
>
> 硬停止线：F0 characterization 退出门通过前，不开始 F1-F4 生产迁移，也不删除 `InteractionMode`

## 0. 2026-08-27 发布收口覆盖

本节覆盖下文基于 `c6379aa`、`5b29431`、`04092ec` 和“未 push”前提写成的历史执行快照。
Milestone A/B 已按 child-first 顺序完成并发布：

| 对象 | 冻结提交 |
| --- | --- |
| framework | `echo-agent@9f8d723ecb8e27a67754afe86fd7d285307866d7` |
| application | `echo-agent-cli@e7d9e9082df09058b7542eeb69e4cf1d5d2df387` |
| website | `echo-website@c25c86d96d2d4958e0c4d9424b61b3c0a8271131` |
| superproject gitlinks | `b17578d` |
| 顶层文档归属规则 | `dd95a9a` |

三个 child 的适用本地门禁、精确相对依赖和远端引用已经核验。GitHub Actions 的 workflow-level
调度失败没有进入 job/runner/steps，按用户当前研发节奏延后到 F7 Final Integration Gate 统一修复；
不得把它描述为代码门禁全绿，也不再用它阻塞 F0-F6 功能开发。

原 10 分钟 soak 证据继续有效；10k/100k artifact/review history 全规模性能测试、10 分钟与 1 小时
协作 soak 统一在 F7 执行，若最终仍保留 2 小时总 soak，也属于 F7。日常阶段必须保留较小规模的
同语义正确性 fixture，不得删除性能预算或永久跳过最终门禁。

## 1. 计划结论

两个原计划的方向都成立，但不能直接串行照抄：

1. Extension worktree 的 surface 收口、specialist owner 复用和 workspace scope 方向正确；
2. 现有 SkillSync 实现仍是“先 runtime fanout、失败后内存 rollback、最后写 durable file”，与新的 durable-first settlement 合同相反；
3. Extension 分支基于 `277c174`，而 CLI main 已前进到 `5b29431`，新增了 ProductData 两阶段生命周期、TaskRuntime async/projection、channel incarnation 和 boot inbox recovery；
4. 交互收敛计划基于 `04092ec`，其中 AgentRouter tracked steer、drain 和 turn settlement 的核心生产路径已被 `5b29431` 实现，后续不得再新增第二套 receipt/runtime；
5. 当前窗口先把 Extension 适配到这些新权威并完成全项目集成；交互收敛只保留为冻结后的后续路线，不进入生产修改。

统一顺序如下：

```text
Extension dirty baseline 冻结
  -> 旧基线 checkpoint
  -> feature merge CLI main@5b29431
  -> durable-first Extension settlement
  -> 五 surface 对等 + 旧 owner 删除
  -> 双 reviewer + 全门禁
  -> squash merge CLI main
  -> 当前重构最终集成 + Rust 1.98 + 10 分钟 soak
  -> 冻结新基线
  -> STOP

后续窗口才允许：
  steer/follow-up -> Todo/Task -> Agent/Subagent tools -> InteractionMode 删除
```

## 2. 当前事实快照

### 2.1 仓库与 worktree

| 对象                   | 当前事实                                                                              |
| ---------------------- | ------------------------------------------------------------------------------------- |
| `echo-agent/main`      | `c6379aad32213ae7e806318d4fd8a28499274c39`，与 `origin/main` 一致，干净               |
| `echo-agent-cli/main`  | `5b294310cfff39572c231429e9d36856c6bb8caa`，与 `origin/main` 一致                     |
| CLI main 用户改动      | 仅 `docs/adr/0001-agent-collaboration.md`，diff SHA-256 `3e2acfec...`                 |
| Extension worktree     | `refactor/extension-control-service@277c174784624671e32272ab2a3deb586dd997f2`         |
| 分支关系               | `main...HEAD = 6/0`，merge-base 为 `277c174`                                          |
| Extension tracked diff | 37 个文件，SHA-256 `90d1c34ef2e8c07e0643424f848d4afe359cb79469e5bdc061e32bceae330a44` |
| Extension untracked    | 4 个：ADR、`extension_control.rs`、Tauri LSP command、LSP panel                       |
| 与 main 重叠           | 37 个 tracked 文件中 22 个也被 main 的 6 个提交修改                                   |
| Cargo 依赖             | 两个相对路径均 realpath 到 `echo-agent/main@c6379aa`                                  |
| 活跃进程               | 无指向 Extension worktree 的 Cargo、rustc、Node、npm 或 Git 长驻进程                  |
| 磁盘                   | 可用约 94 GiB；两个 target 合计不足 1 GiB，不触发 clean                               |
| 本机 Rust              | 当前 stable 为 1.97.1；Milestone B 必须显式使用 Rust 1.98.x                           |

顶层 superproject 当前另有用户拥有的 `AGENTS.md` 修改和未跟踪交互计划。任何实现、提交或清理不得覆盖、暂存或重写这些内容。

### 2.2 CLI main 新增的六个权威提交

| Commit    | 必须保留的权威                                                                           |
| --------- | ---------------------------------------------------------------------------------------- |
| `8a478a3` | ProductData workspace scope、两阶段 admission/join、caller drop 后继续 settlement        |
| `c7da089` | final framework lock contract                                                            |
| `de55914` | TaskRuntime async operation owner、typed IPC、blocking boundary                          |
| `27c86ce` | bounded history/checkpoint projection、committed-but-degraded query semantics            |
| `04092ec` | channel sender/incarnation、scoped lifecycle、outbound/tool renderer 拆分                |
| `5b29431` | boot owner、Agent inbox journal、tracked steer drained、turn settlement、orphan recovery |

Extension merge 冲突只能把 Extension adapter 接到这些 owner 上，不能恢复 `277c174` 的旧 state/runtime/channel 主循环。

## 3. 两个原计划的 review 修正

### 3.1 Extension 计划保留项

- `ExtensionControlService` 是唯一 EKO extension mutation admission；
- framework 保留通用 registry、protocol 和 manager；
- Plugin、MCP、Hook、LSP、Browser 由现有 specialist runtime 执行；
- workspace focus、durable enablement、配置、生命周期、surface receipt 留在应用层；
- GUI、TUI、CLI/JSONL、channel 只做参数转换和 receipt 渲染；
- ADR 使用 `0012-extension-control-authority.md`；
- 不增加 SQLite、permission-mode gate 或 `Worker` 术语。

### 3.2 Extension 计划必须补强项

1. **SkillSync 改为 durable-first**：先原子提交 `enabled-skills.json`，再发布 runtime generation；durable 成功后不得用内存 rollback 假装未提交。
2. **receipt 必须 typed**：手写 `serde_json::Value` 不能作为 GUI/Tauri 最终合同；Rust DTO 与 generated TypeScript 必须字段无损。
3. **Extension service 自己拥有非可取消 settlement**：当前 plain `Mutex<()>` 只能串行，caller future 被 drop 时会取消整个 async body；必须改为 accepted operation 的独立 owner/supervisor。
4. **repair debt 不新建第二个 store**：durable desired generation 是事实源；debt 由 desired generation 与 runtime applied generation 的差异推导，并在 restart、workspace load、下一 mutation 前重放。
5. **幂等与 ABA 显式化**：operation identity、content identity、desired generation、workspace generation 和 specialist generation 必须分别核验。
6. **旧 owner 删除进入同一里程碑**：不能只新增 service 后保留 surface 私有 mutation 或 `SkillsHub.loaded` authority。

### 3.3 交互收敛计划的当前代码修正

原计划不能再以 `echo-agent-cli/main@04092ec` 为实施基线。`5b29431` 已经具备：

- live Agent delivery 调用 `steer_input_tracked`；
- receipt 等待 framework `Drained`；
- foreground waiter 提供 turn typed terminal；
- cold delivery 与 live delivery 共用 AgentRouter durable journal；
- boot reconciliation 处理未结算 inbox attempt；
- ProductData、TaskRuntime query/mutation 和 channel scope 已有新的 owner。

因此后续交互 Iteration 1 应降级为“统一术语、DTO 和剩余边界审计”，不得再写一个 mailbox、drain tracker 或 turn settlement supervisor。

原计划还需修正：

- 长时 soak 移到 `InteractionMode` 等后续产品阶段全部完成后的最终总验收；Milestone B 的原
  10 分钟证据已完成，后续不在中间阶段重复运行；
- 当时没有 push 授权；该前提已被第 0 节的 2026-08-27 发布收口取代；
- `TodoStatus` 仍在 `PlanTask`、completion gate 和 store 路径中参与运行时判断，后续 Todo 收敛仍有效，但本窗口不改；
- `InteractionMode` 仍有约 288 个当前引用，本窗口只保证 Extension 接线不扩散、不删除；
- 当前 `docs/adr/0001-agent-collaboration.md` 是独立用户改动，只能作为单独 docs commit 处理。

## 4. 业界依据与 EKO 取舍

### 4.1 可核验共性

- OpenAI Codex app-server 将 Thread、Turn、Item 分层，turn 以 started/item events/completed 收敛；skills list、skill config、hooks list 和 plugin/MCP runtime state 是不同接口。当前官方文档页在本环境返回 403，因此只把固定 commit 的官方开源协议作为版本快照：
  - [Codex app-server protocol](https://github.com/openai/codex/blob/fde2156057c38c0227ce94c8514d04c7498df60d/codex-rs/app-server/README.md)
- Cursor Plan Mode 把 plan 做成可审阅、可编辑、可保存的 artifact，不要求业务 runtime 增加 plan-approval 状态：
  - [Cursor Plan Mode](https://cursor.com/docs/agent/plan-mode)
- Cursor Skills 区分文件发现、作用域、显式/自动调用和 session 使用；MCP 区分 project/global config、enablement 和 live server/tool state：
  - [Cursor Skills](https://cursor.com/docs/skills)
  - [Cursor MCP](https://cursor.com/docs/mcp)
- Cursor Subagents 使用独立 context、结构化返回和可选 worktree 隔离；这支持“specialist runtime 保持独立 owner，协调层只路由”的取舍：
  - [Cursor Subagents](https://cursor.com/docs/subagents)
- Claude Code 的当前在线文档在本环境超时；本计划只复用仓库内已核验的 [Claude Code 能力目录](../echo-agent-cli/docs/adr/0003-claude-code-capability-catalog.md)，不声称本次刷新了外部工具名称。

### 4.2 EKO 具体取舍

跨系统共性是“发现、durable enablement、live execution、surface projection 分离”，不是复制某个产品的工具名。

EKO 是本地单用户应用，因此：

- 不增加多租户权限门或 `full-auto` 门控；
- 只防止错误 workspace、旧 generation 覆盖、新旧 owner 并存、durable 状态被错误回滚、密钥进入日志；
- extension desired state 使用文件和内存，不引入 SQLite；
- framework 继续提供能力菜单，CLI 是否使用不是删除 framework API 的理由。

## 5. 分层与唯一 owner

### 5.1 三层判定

| 层              | 拥有                                                                                                                                                  | 禁止拥有                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| framework       | `SkillRegistry`、`HookRegistry`、`McpManager`、`LspManager`、plugin parser/integrator、通用 tool/runtime primitives                                   | EKO workspace focus、`enabled-skills.json`、UI receipt、产品 theme/output-style policy            |
| EKO application | `ExtensionControlService` admission、workspace/host scope、durable desired state、repair orchestration、config watcher lifecycle、surface-neutral DTO | 第二套 Skill/Hook/MCP/LSP registry、第二个 file store、第二个 watcher、第二个 specialist executor |
| surfaces        | 请求参数、operation id、展示 typed receipt                                                                                                            | 私有 mutation state machine、直接改 registry/config、从文本猜 settlement                          |

### 5.2 specialist owner 映射

| 领域               | durable/product owner                    | live specialist owner                                                     | Extension service 职责                                 |
| ------------------ | ---------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------ |
| Skill              | `enabled-skills.json` desired generation | framework `SkillRegistry`，由 `PluginRuntimeService` 原子发布到 AgentPool | commit desired、捕获 targets、协调 fanout、返回 debt   |
| Plugin             | plugin registry/state/preference file    | `PluginRuntimeService`                                                    | exact workspace admission、调用、聚合 receipt          |
| Hook               | EKO config + global/workspace hook files | framework `HookRegistry`                                                  | 解析 exact root/config、让 specialist 替换 user source |
| MCP                | `McpConfigRuntime` canonical `mcp.json`  | framework `McpManager`/Agent tool manager                                 | 只调用 durable commit + real reconcile                 |
| LSP                | global/workspace `.lsp.yaml`             | framework `LspManager`，由 plugin runtime 绑定                            | exact workspace rebind、watcher admission              |
| Browser            | EKO Browser config/lifecycle             | existing `BrowserRuntime`                                                 | scoped command routing和 receipt                       |
| Theme/output style | plugin preferences                       | `PluginRuntimeService` + `AgentPluginGeneration`                          | 确保 primary、existing pool、future pool 同 generation |

## 6. Milestone A：完成 Extension Control Authority

### A0. 冻结与归属核验

执行前再次记录：

- Extension worktree 活跃进程；
- `git status --porcelain=v2`；
- tracked diff SHA-256 和 4 个 untracked path；
- `main...HEAD`、merge-base、main/origin-main hash；
- Cargo 两个 path 的 `realpath` 与依赖 HEAD；
- CLI main ADR diff SHA-256；
- 磁盘与 target 大小。

建立“定义、注册、生产调用”三列调用图，至少覆盖：

- `SkillRegistry`/`SkillsHub`/enabled config；
- `PluginRuntimeService`/Plugin registry；
- `McpConfigRuntime`/MCP manager/health；
- `HookRegistry`/config watcher；
- `LspManager`/LSP watcher；
- `BrowserRuntime`；
- primary/existing/future AgentPool generation；
- GUI/TUI/CLI/JSONL/channel command registration。

退出门：形成精确文件归属与旧 owner 删除表，不修改 main，不创建新的 registry/store/watcher。

### A1. 旧基线 feature checkpoint

先在 `277c174` 基线上修复现有 worktree 的编译、格式、测试和前端问题。checkpoint 的语义是“保存现有 Extension authority 收口成果”，不是宣称新 SkillSync settlement 已完成。

适用门禁全部通过后提交：

```text
chore(refactor): checkpoint extension control authority
```

约束：

- 禁止 reset、stash、rebase 或覆盖 dirty worktree；
- 不把 CLI main 的 `0001-agent-collaboration.md` 带入 feature；
- checkpoint 前先把 Extension ADR 改为 `0012-extension-control-authority.md` 并更新旧基线索引，避免与 main 新增的 TaskRuntime `0008` 发生 add/add 文件冲突；merge main 后再次核验 `0008`-`0012` 索引；
- 所有 commit 使用 `git -c commit.gpgsign=false commit`。

### A2. merge `main@5b29431`

在 feature 分支执行 `git merge main`，不 rebase。

冲突解决原则不是逐行二选一，而是“以 main owner 为骨架，重新接入 Extension adapter”。

| 文件/区域                                     | merge 取舍                                                                                                                                                            |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `state.rs`                                    | 保留 main 的 ProductData、AgentRouter、boot/foreground/model/workspace settlement；只重加 scoped Extension control、target snapshot、crate-private specialist adapter |
| `runtime.rs`                                  | 保留 main 的 ApplicationLifecycleOwner 和 ProductData shutdown；Extension service 注册 admission/join，bootstrap reconcile 走 ProductData flow                        |
| `workspace/runtime.rs`                        | 保留 main host activity、publication、recovery 和 eviction；workspace load/rebind 在 host publication 中重放 desired generation                                       |
| `agent_pool.rs`                               | 保留 main pool admission/incarnation；在现有 `AgentPluginGeneration` 增加 output-style/skill generation，应用到 primary、existing、future Agent                       |
| `plugin_runtime.rs`                           | 保留 specialist mutation owner；适配 main AgentPool API，不复制 pool admission 或 workspace lifecycle                                                                 |
| `channels.rs`                                 | 保留 main sender/incarnation/outbound/tool-render 架构；只在新 command dispatch 上调用 app-core service，不恢复旧的大函数布局                                         |
| `tui/events.rs`                               | 保留 main TaskRuntime async/projection 和 turn path；只迁移 Extension slash handler 与 typed renderer                                                                 |
| `src/main.rs`/`src/cli/modes.rs`/`desktop.rs` | 只注入同一个 `Arc<ExtensionControlService>`；不得改变 `InteractionMode` 语义                                                                                          |
| docs/ADR                                      | 保留 `0008` TaskRuntime ADR、`0009`、`0010`、`0011`；Extension ADR 改为 `0012` 并更新索引                                                                             |

merge 后先跑 compile/fmt quick gate，并检查：

- `git diff --check`；
- `rg "worktrees|/Users/" */Cargo.toml` 零命中；
- Cargo path 仍为 `../echo-agent` 与 `../../echo-agent`；
- 不存在 main 新文件被 feature 旧树删除的 reverse diff。

### A3. Extension service 非可取消 settlement

把 `ExtensionControlService` 从 plain mutex 改为应用级 accepted-operation owner：

1. admission 在 service 内串行化所有 Plugin/Skill/MCP/Hook/LSP/Browser mutation；
2. accepted 后由 service-owned task 完成 settlement，caller drop 只丢失等待者，不能取消工作；
3. service shutdown 分两阶段：close admission，再 join 所有 accepted settlements；
4. config watcher 和 manual surface mutation 使用同一 admission；
5. operation result 保留 bounded recent receipt，供同 operation id 重试返回；
6. settlement task panic/join failure 转为 typed terminal，不得静默丢失。

优先复用 main 已有模式：

- ProductData `begin_owned_flow`/nested I/O/settle；
- `PluginRuntimeService::run_owned_mutation` 的 specialist 非可取消执行；
- model/workspace mutation 的 running/settled/closed lifecycle。

不得新建第二个通用 lifecycle framework 或第二个文件 store。

### A4. durable-first SkillSync

#### A4.1 durable desired schema

在现有 `enabled-skills.json` 中保存唯一 desired fact，允许在开发阶段直接升级 schema，无兼容负担。至少包含：

- schema version；
- monotonic desired generation；
- canonical content hash；
- enabled skill map；
- bounded recent operation identities，用于 duplicate/conflict 判定。

同一个文件完成 identity 和 desired state，不增加 repair-debt store。

#### A4.2 commit 顺序

```text
validate request + resolve skill catalog
  -> canonicalize desired content
  -> detect duplicate/conflicting operation id
  -> stage JSON in same directory
  -> sync file
  -> atomic rename/replace
  -> sync parent directory
  -> publish committed generation in memory
  -> fanout to global seed + loaded workspace runtimes
  -> return Settled or Degraded receipt
```

文件提交使用现有 `echo_agent::utils::fs::atomic_write` 或同等已验证 helper，并通过 ProductData owned flow 执行；不得继续使用裸 `std::fs::write`。

#### A4.3 typed receipt

receipt 必须显式区分：

- durable commit identity：operation id、content id/hash、desired generation；
- settlement：`Settled` 或 `Degraded`；
- target receipts：scope、workspace generation、specialist generation、applied/no-op/failure；
- repair debt：target、component、expected generation、observed generation、reason、retryable；
- authority scope 与 committed file path。

pre-commit validation/write 失败返回 error；durable 已提交后 fanout 失败只能返回 `Degraded`，不能返回“未发生”或执行内存 rollback。

Rust DTO 使用 serde + ts-rs，Tauri 直接返回 typed DTO；前端不得手写另一套字段。

#### A4.4 repair replay

- restart：bootstrap 从 durable desired generation 构建 global seed，再加载 workspace；
- workspace load：host publish 前 reconcile exact desired generation；
- next mutation：新 commit 前先重放或重新计算上一 generation 的 debt；
- same content：不重复增加 generation，但仍可修复未收敛 target；
- same operation id + same content：返回原/重建 receipt；
- same operation id + different content：typed conflict；
- stale generation：不得写入 newer AgentPool/plugin runtime；
- future pooled Agent：从 pool retained generation 创建，不依赖 mutation 当时的 Agent 列表。

repair debt 是 durable desired 与 live applied generation 的差异，不新增第二个持久化权威。

#### A4.5 fault matrix

必须覆盖：

- caller drop before durable writer returns；
- caller drop after durable commit、mid-fanout；
- atomic replace 前/后故障；
- durable commit 成功但 1/N targets 失败；
- restart 后 repair；
- workspace load repair；
- next mutation 先 repair；
- workspace A -> B -> A ABA；
- duplicate operation/content；
- old generation late completion；
- existing pooled Agent 与 future pooled Agent；
- disabled sibling 不被错误加载；
- install/sync 改变 skill content 后 content identity 更新。

### A5. Specialist runtime 闭环

#### Skill

- framework `SkillRegistry` 是 live authority；
- `SkillsHub` 只做 catalog discovery，不保存 `loaded` authority；
- install/uninstall/sync 先处理文件 artifact，再通过同一 desired mutation 发布；
- content changed 但 enablement 未变时仍产生新的 content identity/generation。

#### Plugin

- `PluginRuntimeService` 继续拥有 scan、dependency、lifecycle、MCP ownership、Hook/LSP wiring；
- Extension service 只 capture exact runtime、调用 specialist、聚合 receipt；
- plugin reload、skill publish、theme/style publish 共用 mutation sequence，防止 generation 互相覆盖。

#### MCP

- `McpConfigRuntime` 继续拥有 canonical config atomic commit 和 real reconcile；
- CLI/TUI/channel 的 connect/disconnect/toggle 必须修改 durable config 并实际 reconcile；
- health 绑定 workspace runtime identity，不使用 bootstrap Agent 的全局 map；
- user-owned name 与 plugin-owned name 继续由现有 ownership registry 仲裁。

#### Hook/LSP

- config root 使用 captured workspace project root，不能使用 process cwd；
- config watcher 每个 workspace 只有一个 target，reload 进入 Extension admission；
- malformed candidate 保留 last-known-good live generation；
- LSP 继续由现有 `LspManager` 执行 start/stop/status。

#### Browser/theme/output style

- Browser 命令跨 GUI/TUI/CLI/channel 进入同一 service；
- theme 是 surface presentation receipt；
- output style instructions 进入 `AgentPluginGeneration`，覆盖 primary、existing pool、future pool；
- 不把 TUI 视觉主题误当 Agent output-style runtime authority。

### A6. Surface parity 与旧 owner 删除

#### 必删 owner

1. `SkillsHub.loaded` 字段、`set_loaded_skills` 和 surface 创建私有 hub 后标 loaded 的路径；
2. CLI `/mcp connect|disconnect` 只打印文本的假实现；
3. GUI/TUI/CLI/channel 直接调用 plugin/skill/hook/LSP/MCP mutation 的路径；
4. `HookConfigLoader::load_merged_from_disk()` 依赖 process cwd 的 production 调用；
5. bootstrap `PluginState.mcp_health: HashMap<...>` 全局权威；
6. `reload_plugins_owned`、`replace_mcp_config_owned` 等绕过 Extension service 的公开 mutation adapter；
7. 只更新 primary Agent 的 theme/output-style publication；
8. touched production code 中“某 surface 不需要能力”的注释和对应 `None` 接线。

#### surface 合同

| Surface        | 入参适配                     | 输出                                      |
| -------------- | ---------------------------- | ----------------------------------------- |
| GUI/Tauri      | typed request + operation id | generated typed receipt                   |
| TUI            | slash parser                 | 同 receipt 的文本/状态渲染                |
| CLI REPL       | command parser               | 同 receipt 的文本/exit semantics          |
| JSONL/headless | structured command/event     | 完整 machine-readable receipt，不只字符串 |
| channel        | sender-scoped command parser | 同 receipt 的 bounded renderer            |

任何 surface 不得重新 resolve focus after service acceptance；receipt 携带 captured authority scope。

### A7. 文档、ADR、examples、website

- ADR 重命名为 `docs/adr/0012-extension-control-authority.md`；
- 更新 CLI `docs/README.md`、`architecture.md`、`configuration.md`、`features.md`、`skill-sync.md` 和 `MASTER-PLAN.md` 的当前事实；
- ADR 必须补充 durable-first、repair debt、caller drop、ABA 和 no-second-store 决策；
- `examples/lh6_product_soak.rs` 只适配新 lifecycle，不恢复旧 bootstrap；
- framework API 未改变时，`echo-agent` docs/examples 不应被修改；只记录已核验其 owner 边界；
- `echo-website` 当前是 framework 文档站，Extension Control 是 EKO 应用策略，默认不修改；最终交付必须明确写“不适用”的代码依据，并运行现有 website 检查。

现有 CLI main `docs/adr/0001-agent-collaboration.md` 不属于 Extension diff，不得混入 Extension feature commit。

### A8. 定向测试、双 reviewer 与门禁

先运行：

```bash
cargo test -p echo-agent-app-core extension_control --all-features --locked
cargo test -p echo-agent-app-core plugin_runtime --all-features --locked
cargo test -p echo-agent-app-core config_watcher --all-features --locked
```

新增/扩展测试必须让 `extension_control` filter 覆盖：

- non-abortable settlement；
- durable commit/degraded debt；
- restart/workspace load/next mutation repair；
- ABA、duplicate operation/content；
- existing/future pool；
- surface DTO round-trip。

实现冻结后并行进行两个只读 review：

- Reviewer A：durable settlement、repair debt、restart、ABA、generation、旧 owner 删除；
- Reviewer B：GUI/TUI/CLI/JSONL/channel 对等、typed DTO/API 字段无损、renderer 不猜状态。

任何 P0/P1/P2 都修复，并重新运行受影响的完整门禁。reviewer 不得直接修改冻结分支。

完整 CLI 门禁：

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo metadata --locked --format-version 1 --no-deps
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used \
  -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-features --locked
cargo check -p echo-agent-app-core --no-default-features --locked
cargo check --no-default-features --features gui --bin echo-agent-tauri --locked
cargo test --no-default-features --features gui --locked
```

前端门禁：

```bash
npx prettier --check "src/**/*.{ts,tsx}"
npm run lint
npm test -- --run
npm run build
```

所有 generator-producing tests 结束后再检查 generated diff；不得用 restore 覆盖真实 contract 变化，也不得保留无解释生成文件。

### A9. 提交、squash merge 与清理

feature 最终内容建议逻辑上分两组，但 squash 到 main 时 Extension 代码和其自身 ADR/docs 属于一个提交：

```text
feat(extensions): own EKO extension lifecycle
```

随后在 CLI main 单独提交现有用户 ADR：

```text
docs(architecture): update agent collaboration model
```

第二个提交 staged path 必须只有：

```text
docs/adr/0001-agent-collaboration.md
```

合并步骤：

1. feature 再次 `git merge main`，不 rebase；
2. 复验相对 Cargo path、Cargo.lock、generated DTO、ADR index；
3. 在 CLI main 使用 `git merge --squash refactor/extension-control-service`；
4. staged audit 后提交 Extension；
5. 单独 staged/提交 `0001` ADR；
6. 确认 main 含 squash 结果后删除 Extension worktree 和 feature branch；
7. 不 push；
8. 因无 push 授权，不提交顶层 superproject gitlink，顶层状态只如实报告本地 CLI pointer 变化。

## 7. Milestone B：当前重构最终集成

### B0. 冻结最终基线

记录：

- `echo-agent/main`、`echo-agent-cli/main`、`echo-website/main` HEAD；
- 三个 status；
- Extension squash commit、ADR commit；
- CLI Cargo.lock hash；
- generated DTO path 清单；
- Extension worktree/branch 已不存在；
- 顶层用户拥有改动未被 staged。

### B1. Rust 1.98 工具链门禁

本机当前 stable 为 1.97.1，最终门禁前必须安装/选择 Rust 1.98.x，并记录：

```bash
rustc --version
cargo --version
cargo clippy --version
```

所有最终 Rust 命令显式在同一个 1.98 toolchain 下运行。不得一部分用 1.97、一部分用 1.98。

### B2. framework 与 CLI 完整门禁

即使 Extension 只修改 CLI，也要对最终 dependency pair 运行：

#### `echo-agent`

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used \
  -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-targets --all-features --locked
cargo check --workspace --lib --no-default-features --locked
```

Extension 不修改 framework feature/Cargo API 时不跑逐 feature 条件矩阵；如果实际 diff 触及 framework manifest、feature 或公共 API，则补跑 AGENTS.md 指定的完整 feature matrix。

#### `echo-agent-cli`

重复 A8 的完整 CLI/GUI/frontend 门禁，确保 squash 后 main checkout 实际通过，而不是只依赖 feature worktree 结果。

#### `echo-website`

若无 Extension 文案变更，至少运行：

```bash
npm run docs:check:source
npm run format:check
npm run lint
npm test
npm run build
```

若实际修改 website，则运行完整 `npm run verify`（包括 shell/site/docs/e2e）。无论是否修改，交付说明都要记录 website 适用性；检查失败仍必须修复。

### B3. hygiene 与合同审计

必须为零或得到明确解释：

```bash
rg "worktrees|/Users/" */Cargo.toml
```

另行检查：

- CLI Cargo.toml/Cargo.lock/production Rust 中 SQLite dependency/feature 为零；
- touched files 新增 `Worker`/`worker` 产品术语为零；
- `InteractionMode` 未被删除、重命名或扩散为替身；
- framework public Skill/Hook/MCP/LSP/Plugin API 未因 CLI 无调用被删除；
- Extension mutation 绕过点为零；
- `SkillsHub.loaded` authority 为零；
- process cwd 作为 Hook/LSP workspace identity 的 production 路径为零；
- bootstrap global MCP health authority 为零；
- generated diff 全部可解释；
- `git diff --check` 全绿；
- Cargo.lock 在 `--locked` 下无隐式更新。

### B4. 10 分钟并发 soak

在干净、已提交的 CLI main 上运行现有并发 soak 600 秒，输出到新的唯一目录，不覆盖旧 ledger：

```bash
cargo run -p echo-agent-app-core --example lh6_concurrency_soak --locked -- \
  --duration-seconds 600 \
  --output-dir <new-output-dir>
```

如果 Extension fault path 需要真实 surface/provider 联动，另跑 bounded product probe，但必须把 probe 标记为 probe，不能冒充 2 小时 acceptance evidence。

Milestone B 不再追加长时 soak。后续仍有 steer/follow-up、Todo/Task/Subagent 和
`InteractionMode` 产品阶段；10 分钟、1 小时以及如仍保留的 2 小时总 soak 统一放到 F7。

### B5. 最终状态

Milestone B 结束时必须同时满足：

1. `echo-agent/main` HEAD/status 明确；
2. `echo-agent-cli/main` 包含 Extension squash commit 与独立 ADR commit，status 明确；
3. `echo-website/main` HEAD/status 明确；
4. Extension worktree 和 feature branch 已删除；
5. 所有命令有实际 exit/result，不写“预计通过”；
6. Rust 1.98、`--locked`、Cargo.lock 和 generated DTO 一致；
7. 10 分钟 soak ledger 为 completed/零 stranded owner；
8. 顶层用户改动与未提交计划未被覆盖；
9. child 与 superproject 已按第 0 节提交并 push；远端 CI 修复 deferred 到 F7；
10. 没有开始下一阶段生产修改。

## 8. Deferred：交互与任务模型收敛路线

本节只用于统一依赖顺序，不是当前执行授权。

### 8.1 进入条件

Milestone B 和新的冻结 commit 已按第 0 节发布，进入条件已经满足。F0 必须重新生成调用图和
引用计数，不得直接以旧计划的 `04092ec` 计数开工。

### 8.2 更新后的后续顺序

```text
F0 新冻结基线 characterization（`9f8d723` / `e7d9e90`）
  -> F1 receipt vocabulary/DTO cleanup（复用当前 tracked runtime，不新建 runtime）
  -> F2 Todo 收缩为 Task graph projection
  -> F3 Conversation Agent / Task Subagent 薄控制工具
  -> F4 删除 InteractionMode
  -> F5 长期 Agent 与 attempt-scoped Subagent 语义
  -> F6 cursor wait、restart、五入口对等
  -> F7 旧路径删除、完整门禁、10k/100k 性能门、10 分钟/1 小时 soak、deferred CI
```

### 8.3 F0 当前明确禁止

F0 characterization 中不得：

- 新增 `agent_followup`、`agent_wait` 等模型协作工具；
- 修改 AgentRouter message/follow-up 产品语义；
- 把 `TodoStatus` 改成新的 Task 状态模型；
- 新增或删除 Task/Subagent API；
- 删除 `InteractionMode`、GUI mode picker、CLI `/mode` 或 generated DTO；
- 删除 `create_complex_task/check_run_status/cancel_run`；
- 创建“临时”的 `ExecutionStyle`、`RouteMode` 等 mode 替身；
- 启动 10 分钟、1 小时或 2 小时长时 soak；
- 运行完整 10k/100k 性能门，F0 仅运行较小规模 characterization。

## 9. 文件所有权

| Owner                    | 独占范围                                                                    |
| ------------------------ | --------------------------------------------------------------------------- |
| Extension settlement     | `extension_control.rs`、enabled desired schema、receipt/repair tests        |
| Specialist plugin        | `plugin_runtime.rs`、Agent plugin generation candidate                      |
| Main runtime integration | `state.rs`、`runtime.rs`、`workspace/runtime.rs`，必须以 main 新 owner 为准 |
| Surface                  | Tauri/TUI/CLI/JSONL/channel adapters 与 renderer，不拥有 mutation semantics |
| Frontend contract        | generated DTO、endpoints、LSP panel/settings projection                     |
| Coordinator              | Cargo.lock、ADR numbering/index、merge、staged audit、worktree cleanup      |
| Reviewer                 | 只读，不直接提交到冻结实现分支                                              |

`state.rs`、`runtime.rs`、`workspace/runtime.rs`、`channels.rs`、`tui/events.rs` 不允许两个实现 owner 并行写。

## 10. 预计工作量

| 阶段                                                |       预计人日 |
| --------------------------------------------------- | -------------: |
| A0-A2 冻结、checkpoint、merge main                  |            2-3 |
| A3-A5 settlement、SkillSync、specialist integration |            4-7 |
| A6-A7 surfaces、删除、DTO、docs                     |            3-5 |
| A8-A9 reviewer、完整门禁、merge/cleanup             |            3-5 |
| B0-B5 Rust 1.98、三仓门禁、hygiene、10 分钟 soak    |            2-4 |
| **当前两个里程碑合计**                              | **14-24 人日** |

后续交互收敛不沿用 35-53 人日作为承诺值；`5b29431` 已完成原 Iteration 1 的重要部分，必须在 F0 重新估算。

## 11. 完成定义

Milestone A/B 的最终完成状态如下：

- Extension durable desired state、non-abortable settlement、degraded receipt 和 repair replay 真实可达；
- Plugin/MCP/Hook/LSP/Browser specialist owner 未被复制；
- GUI/TUI/CLI/JSONL/channel 同 service、同 DTO 语义；
- 旧 mutation owner 已删除；
- Extension ADR 为 `0012`；
- feature 已 squash 到 CLI main，worktree/branch 已清理；
- `0001-agent-collaboration.md` 独立提交；
- Rust 1.98 与所有适用门禁通过；
- 10 分钟 soak 通过；
- 三个 child main 已发布，superproject gitlink 已更新；
- 顶层 `AGENTS.md` 已独立提交，两份阶段计划继续单独提交；
- 远端 CI workflow-level 调度失败已明确 deferred 到 F7；
- steer/follow-up、Todo/Task/Subagent、`InteractionMode` 尚未进入生产修改。
