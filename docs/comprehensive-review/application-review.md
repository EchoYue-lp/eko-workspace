# EKO 应用层三方 Review 综合复核

> 状态：2026-08-14 实装修复后的当前权威结论
> 复核日期：2026-08-14
> 应用基线：`echo-agent-cli` `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> 框架基线：`echo-agent` `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> 工作树说明：两个仓库均有用户未提交改动；本轮在现有工作树上完成复查、修复与全量验证，未提交

本文件综合并复查以下三份独立报告：

- [Codex application review](codex/reports/synthesis/application-review.md)
- [ZCode-ds application review](zcode-ds/reports/synthesis/application-review.md)
- [ZCode-glm application review](zcode-glm/synthesis/application-review.md)

三模型逐项发现、交集、差异和赞同/驳回理由见
[application-model-comparison.md](application-model-comparison.md)。

三份原稿继续作为证据索引保留，但不再分别充当最终结论。后续规划和修复以本文件的分歧裁决、优先级和分层判定为准；具体实现前仍须回到原子任务报告及当前代码重新定位。

## 0. 2026-08-14 实装修复复核

本轮按定义、注册、真实调用路径和回归测试重新检查了下文全部风险簇。没有新增平行 Store、Task/Plan/Todo 模型或第二套调度循环。通用文件原子性、typed terminal、Tool 取消和 redaction 留在 framework；workspace generation、会话分支、surface composition、webhook policy 留在 EKO 应用；adapter 只做无损转换与产品 metadata 注入。

关键架构取舍继续采用既有 B-REF-01 对 Claude Code、Codex、Cursor、Devin 和 Temporal 的交叉调研：Plan 是可编辑 artifact 而非审批状态机；执行有 typed terminal 和稳定 identity；Subagent attempt 有界、可取消、可恢复；应用策略不污染通用框架。本轮没有出现需要推翻该结论的新证据。

### P0 复核结果

| 原风险 | 当前状态 | 当前证据 |
|---|---|---|
| 跨 workspace 配置覆盖 | 已闭合 | `AppState.config_path` 保存启动时不可变配置源；config/provider 写入不再按可变 cwd 重新解析 |
| 跨 workspace 草稿覆盖 | 已闭合 | 文件读取返回 `workspace_id + revision`；写回以两者为前置条件并通过原子 CAS 拒绝旧 workspace/外部修改 |
| 会话 stale writer 覆盖 | 已闭合 | runtime 是 canonical transcript 唯一 writer；GUI 只合并 UI metadata；edit/regenerate 通过后端 immutable branch 重建 canonical prefix |
| Memory 多实例 lost update | 已闭合 | `FileStore` 同路径实例共享一个进程内权威并使用文件锁/CAS；primary 与 pool 的 workspace store 一次性重绑 |
| 历史 artifact 被重跑删除 | 已闭合 | analysis 输出按不可变 run 目录保存；失败重跑不继承或删除历史 run artifact |
| enrichment 空值覆盖 | 已闭合 | provider 结果按字段维度 merge；失败维度保留已有非空证据 |
| rule promotion 错误读后覆盖 | 已闭合 | 读取错误 fail closed；规则、审计和 memory promotion 具备原子写与失败回滚 |
| webhook 原始参数/错误外发 | 已闭合 | payload 在序列化/签名前统一做嵌套 secret redaction、字符和数组上限 |
| 终端输入 preview 持久化 | 已闭合 | 终端输入事件仅记录字节数等 metadata，不持久化明文 preview；交互式终端仍不受 agent automation mode 阻断 |

### P1 风险簇复核结果

| 风险簇 | 当前状态 | 已完成 | 仍需行动 |
|---|---|---|---|
| Workspace generation | 部分闭合 | switch/exit 串行化；切换前准备 stores/目录并校验 root；拒绝 active chat、Running TaskRun 和 busy pool；primary/pool/store/memory/routing 完成后才发布 current；TUI/CLI 使用同一 `AppState` workspace 命令 | TaskRuntime/scheduler 的持久根仍是进程启动时的全局实例；plugin/LSP/MCP 尚未加入统一 generation receipt；后半段失败仍缺完整 rollback/degraded receipt |
| Turn terminal/取消/投递 | 主路径闭合 | `drive_chat -> TurnOutcome` 统一 completed/cancelled/failed；provider 中途失败保留 partial token；GUI terminal 不再以 stream transport success 推断；TUI/CLI/channel 共用 driver | channel/REPL 仍需 surface 级“当前 turn 可发现且可取消”的端到端矩阵，尤其验证 transport disconnect 与 next-message HITL 并发 |
| Task claim/恢复/artifact | 已闭合本报告的具体反例 | JSONL torn tail 可修复且中段损坏 fail closed；writer 不再错误启用 plan mode；claim/attempt/revision、worktree ownership、artifact terminal contract 已由现有 TaskRuntime/框架测试覆盖 | workspace generation 收敛前，不能宣称 TaskRuntime persistence 已按 workspace 隔离 |
| HITL 语义 | 已闭合 | GUI rule 与 framework `PermissionService` 同步；REPL EOF 拒绝；session scope 文案/语义一致；没有给终端/MCP 增加 `full-auto` 门控 | 保留多 surface contract fixture，防止未来出现文案与 scope 漂移 |
| 多模式能力对等 | 部分闭合 | TUI/CLI workspace、TaskRuntime、pool、conversation store、review integration 和 workspace-aware attachments 已接通；共享 `drive_chat` 提供 typed terminal | 继续补齐 channel cancel、browser event bridge、MCP persistence、plugin/LSP/MCP generation 和各 surface 的同 fixture 验收 |

结论：下文 2026-08-13 的 9 个 P0 已完成修复并通过回归；不能把整个应用误报为“全部完成”，因为 workspace generation 和少数 surface parity 项仍是明确 P1。后续只围绕上表剩余项收敛现有权威，不再新增平行运行时。

## 1. 综合结论

EKO 不缺少一套新的应用架构。共享 `drive_chat`、`PreparedUserTurn`、文件会话存储、revisioned Task graph、TaskRuntime 文件权威、AgentPool、生成 DTO 等核心构件已经存在。主要问题是这些权威没有贯穿完整生命周期：workspace/config/memory 的 generation、conversation turn 的 terminal、Task 的 claim/revision/attempt、artifact identity 和各交互面的投影在不同边界丢失或被重建。

因此，正确方向不是再造 Store、Task 模型、事件总线或 approval 状态机，而是让现有权威端到端收敛，并删除被替代的旁路。

本轮接受 **9 个 P0**：7 个属于可证明的数据覆盖、历史破坏或错误读后写链路，2 个属于可证明的敏感内容落盘/外发链路。ZCode-ds 和 ZCode-glm 的“0 P0”结论不成立：它们遗漏或低估了后来通过更细粒度静态数据流复核得到的破坏链路。这里的 P0 是按本目录 `REPORTING.md` 的定义判定，不代表套用公网多租户威胁模型。

P1 不采用三份报告中的任一原始总数作为规划工作量。`8`、`25`、`109` 分别来自不同的合并粒度和优先级口径，直接比较没有意义。综合稿以风险簇和独立验收场景管理 P1，避免把共享根因错误合并成一个测试，也避免把同一根因的每个表现都算成独立架构项目。

## 2. 2026-08-13 基线确认的 P0（历史证据）

| 风险 | 结论 | 当前代码复核 | 原始证据 |
|---|---|---|---|
| 跨 workspace 配置覆盖 | 保留 P0。保存目标由可变 workspace/CWD 重新解析，编辑期间切换 workspace 可把内容写到另一项目 | 当前 `switch_workspace` 仍分步修改 cwd、store 和 Agent，没有不可变 generation/保存前置条件 | [A-CFG-01-P0-01](codex/reports/tasks/A-CFG-01.md) |
| 跨 workspace 草稿覆盖 | 保留 P0。相对路径与内容哈希不足以证明目标仍是打开时的文件 | GUI 草稿/写回仍缺 workspace identity + revision precondition | [A-PROJ-01-P0-01](codex/reports/tasks/A-PROJ-01.md) |
| 会话 stale writer 覆盖 | 保留 P0。GUI autosave 与 Agent finalize 可用旧前缀覆盖完整 transcript | 会话显示态、Agent 历史和文件写入仍不是一个 generation commit | [A-STATE-01-P0-01](codex/reports/tasks/A-STATE-01.md) |
| Memory 多实例 lost update | 保留 P0。主 Agent 与 pooled Agents 可持有同一文件的独立 `FileStore` 快照 | workspace 切换仍逐个重绑；没有共享单实例或 revision 拒绝 | [A-MEM-01-P0-01](codex/reports/tasks/A-MEM-01.md) |
| 历史 artifact 被重跑删除 | 保留 P0。旧 run 记录继续引用被覆盖/删除的路径 | artifact identity 尚未统一为不可变 run/revision/attempt 路径 | [A-DOM-01-P0-01](codex/reports/tasks/A-DOM-01.md) |
| enrichment 空值覆盖 | 保留 P0。部分刷新失败可用空字段覆盖已有非空证据 | 当前缺少字段级 merge/保留旧证据约束 | [A-DOM-01-P0-02](codex/reports/tasks/A-DOM-01.md) |
| rule promotion 错误读后覆盖 | 保留 P0。读取失败可被当成空内容继续写回，且多步 mutation 不具备恢复性 | 尚无 last-known-good + 原子 mutation receipt | [A-EVO-01-P0-01](codex/reports/tasks/A-EVO-01.md) |
| webhook 原始参数/错误外发 | 保留 P0。用户配置 webhook 后，tool args 和 raw error 会直接序列化发送 | `WebhookTurnObserver` 当前只做字符截断，没有 secret redaction | [A-OBS-01-P0-01](codex/reports/tasks/A-OBS-01.md) |
| 终端输入 preview 持久化 | 保留 P0。交互式终端输入内容进入持久化事件预览 | 本地产品定位不要求禁止终端，但仍要求密钥不进入日志/持久化预览 | [A-SRF-02-P0-01](codex/reports/tasks/A-SRF-02.md) |

说明：本地桌面威胁模型意味着不能用 `permission_mode` 阻断用户主动打开终端、连接 MCP 或操作文件；它不豁免“防止无意数据丢失”和“不要把密钥写进日志”这两条本地同样成立的安全约束。

## 3. 2026-08-13 P1 权威风险簇（历史基线）

### 3.1 Workspace generation 不一致

`switch_workspace` 当前先发布 `workspace.current`，再依次改变进程 CWD、primary Agent、pool、persistence、conversation store、runtime state、memory、skills 和 routing。任一步失败都可能留下混合 generation；`exit_workspace` 还把 conversation store 重置到 `Persistence::base_dir()`，而启动使用 `infra::create_conversation_store()`，两者根目录不一致。

权威验收：一次 switch/reload 要么让 primary/pool、stores、config watcher、hooks、plugins、LSP/MCP 和 UI 全部确认同一 generation，要么回滚或返回明确 degraded receipt。禁止在失败后继续报告成功。

主要证据：[A-CFG-01](codex/reports/tasks/A-CFG-01.md)、[A-MEM-01](codex/reports/tasks/A-MEM-01.md)、[A-PLG-01](codex/reports/tasks/A-PLG-01.md)、[A-STATE-01](zcode-ds/reports/tasks/A-STATE-01.md)。

### 3.2 Turn terminal、取消与投递不一致

`drive_chat` 返回 `Result<(), String>`，但 envelope adapter 会把底层 stream error 规范化为 terminal payload，导致函数成功返回与事件终态脱钩。GUI、webhook、持久化和队列随后分别推断 terminal，出现 error 显示 completed、cancel 伪装成 agent error、partial answer 被清空、interrupt prompt 留下 ghost turn 等问题。REPL/channel 仍缺可达的每回合 cancel owner。

权威验收：一个 typed `TurnOutcome` 同时驱动 sink terminal、会话提交、队列释放、webhook、tool execution 和 shutdown；每个 turn 恰好一个单调 terminal，并保留失败/取消前的有效 partial output。

主要证据：[A-CHAT-01](codex/reports/tasks/A-CHAT-01.md)、[A-SRF-03](zcode-ds/reports/tasks/A-SRF-03.md)、[A-SRF-04](zcode-ds/reports/tasks/A-SRF-04.md)、[A-OBS-01](zcode-ds/reports/tasks/A-OBS-01.md)。

### 3.3 Task claim、暂停恢复与 artifact identity 不一致

当前代码仍可确认：`events.jsonl` 任意非空坏行会让 `read_events` 整体失败；writer Subagent 构建后仍调用 `set_plan_mode(true)`；pause/cancel、wave fault cleanup、run trace terminal、worktree/attempt/artifact identity 在 framework executor 与 EKO adapter 间没有贯穿。

这推翻 ZCode-glm “A-TSK-02..05 接近 clean、adapter conformance 已成立”的总括判断。正面结论应缩小为：framework revision service 和 EKO file authority 已存在，且值得保留；不能据此声称执行、恢复和投影已经无损。

权威验收：同一个不可变 `run_id + revision + task_id + claim/attempt` 穿过 dispatch、Subagent、worktree、verification、integration、artifact 和 UI；pause 后可恢复，fault 后无遗留 Running sibling；崩溃尾行可恢复且不会吞掉中间损坏。

主要证据：[A-TSK-01](zcode-ds/reports/tasks/A-TSK-01.md)、[A-TSK-03](zcode-ds/reports/tasks/A-TSK-03.md)、[A-TSK-04](zcode-ds/reports/tasks/A-TSK-04.md)、[A-TOOL-01](zcode-ds/reports/tasks/A-TOOL-01.md)。

### 3.4 HITL 语义与管理面失真

GUI permission rule 可以保存和列出，但没有进入工具调用判定；REPL EOF 被当成空输入批准；`SessionAllTools` 的 `*` 是整会话全工具批准，而部分 UI 文案容易让用户理解为“本工具本会话批准”。这些是本地交互正确性问题，不是要求新增 `full-auto` 门控。

权威验收：用户明确选择的 scope 与框架 rule 一一对应；EOF/transport loss fail closed；GUI/TUI/CLI/channel 使用同一语义，只保留渲染差异。

主要证据：[A-HITL-01](zcode-ds/reports/tasks/A-HITL-01.md)。

### 3.5 多模式能力不对等

已确认的主缺口包括：TUI/CLI workspace、REPL/channel turn cancellation、channels-only scheduler/background service、GUI browser event bridge、GUI MCP 配置持久化、Task writer 实际只读、TUI Subagent 详情与任务控制不完整。它们都是产品缺口，不能用“该模式不需要”解释。

权威验收：建立从 capability 定义、composition、trigger、typed event/snapshot、render、cancel 到 recovery 的可执行矩阵；GUI、TUI、CLI、channel、cron/background 共享能力事实，表面仅负责输入输出适配。

主要证据：[A-SRF-01](zcode-ds/reports/tasks/A-SRF-01.md)、[A-SRF-02](zcode-ds/reports/tasks/A-SRF-02.md)、[A-SRF-04](zcode-ds/reports/tasks/A-SRF-04.md)、[Q-E2E-01](zcode-ds/reports/tasks/Q-E2E-01.md)。

## 4. 分歧裁决

| 分歧 | 裁决 |
|---|---|
| 0 P0 vs 9 P0 | 接受 9 P0。后来报告给出了具体 source-to-sink / read-modify-write 链路；“本地应用”不降低数据覆盖和日志泄密的严重度 |
| P1 数量 8 / 25 / 109 | 不选任一数字作为 roadmap 数。8 明显漏项；25 是较合理的粗粒度集合；109 是原子验收表现数量。规划按五个权威风险簇，测试保留所有独立表现 |
| Task adapter clean vs 存在恢复/暂停/只读 writer 故障 | “clean”结论驳回。保留“已有单一 file authority 和 revision service”这一正面事实，但 adapter/executor 生命周期尚未收敛 |
| `gitignore::globstar_match` UTF-8 panic 是 P1 | 当前降为 P3/接线前阻断。panic 可复现且违反 Rust 规则，但 `should_ignore_path` 只位于未接入主路径的旧 `ProjectIndex`；定义存在不等于运行时可达。触及该模块时应直接修复或连同死代码删除 |
| MCP URL 私网/allowlist 拦截是否安全要求 | 按 AGENTS.md 判为错误的在线服务式过度门控。只保留明显错误输入与明文 HTTP 的轻量校验，不使用 `permission_mode` 阻断用户配置的本地 MCP |
| 本地 ToolExecutionRepository 保存完整 args 是否一律 P0 | 不一律。它是用户本机的功能数据，不能仅因 plaintext 判 P0；只有违反产品预期进入日志、跨边界外发、权限过宽或无保留策略造成具体暴露时才升级。终端 preview 和 webhook 已有这类具体链路 |
| 缺 API key 未在 bootstrap fail-fast 是否 P1 | 保留为 setup/错误体验问题，但不与数据完整性 P1 同批。应用可以启动进入设置页；真正验收是首次请求前给出可操作的 typed configuration error，而非强制整个应用拒绝启动 |
| `Retrying`/`Paused` 投影 lossy | 当前执行路径并不产生所有 framework 状态，不能据此声称实时丢失；文档应缩窄“lossless”承诺。未来接入对应状态时必须先扩展投影 |

## 5. 分层判定

| 层 | 应保留/实现的职责 | 不应拥有的职责 |
|---|---|---|
| Framework (`echo-agent`) | 通用 DAG/revision/claim、取消/暂停/重试原语、typed stream terminal、通用 redaction/atomic-write helper、Subagent 执行身份 | EKO workspace、UI projection、worktree 产品策略、reviewer 策略、具体保留期和 webhook 产品事件 |
| Application (`echo-agent-cli`) | workspace/config generation、文件存储布局、conversation/artifact 生命周期、交互式 HITL 文案与 scope、surface parity、webhook policy、worktree/GUI/TUI/CLI 产品流程 | 第二套 DAG 主循环、通用 retry/cancel/settlement、平行 Task/Plan/Todo store |
| Adapter | 无损类型转换、EKO metadata、policy/hook 注入、调用 framework service | ready frontier、DAG 校验、重试主循环、状态权威或 revision 推断 |

实现前重复性搜索结论：现有 `drive_chat`、`PreparedUserTurn`、Task revision service、TaskRuntime file store、AgentPool、FileConversationStore、generated DTO 和 webhook observer 均应扩展，不应新建平行实现。迁移阶段的 adapter 必须在同一阶段切换真实主路径并删除旧 owner。

## 6. 修复顺序

1. **P0 containment**：为跨 workspace 写入增加不可变 identity/revision；保护 stale writer；artifact 路径不可变；enrichment merge 保留旧非空值；所有日志/webhook sink 在序列化前 redaction。
2. **Workspace generation**：一次性协调 cwd、config watcher、hooks、plugins、primary/pool、conversation/memory/runtime stores、LSP/MCP 与 UI；失败回滚或返回 degraded receipt。
3. **Durable recovery**：区分 Missing/Valid/Corrupt；修复 JSONL torn tail；原子投影 receipt、幂等恢复、删除 tombstone/cascade；保持文件存储，不引入 SQLite。
4. **Turn 与 Task lifecycle**：typed terminal；唯一 cancel owner；pause/fault sibling cleanup；claim/attempt identity 贯穿 worktree、artifact、trace 和 UI；移除 writer 的错误 plan-mode 限制。
5. **Surface parity**：把 GUI-only 业务逻辑移入 app-core；用同一 fixture 驱动 GUI/TUI/CLI/channel/cron/background 的 capability matrix。
6. **性能、可访问性和死代码**：事件 identity 稳定后再规范化前端 store、lazy-load 大 artifact、补 modal/focus 语义，并删除 `ProjectIndex`、旧 persistence/search/output 等已确认的应用层死权威。

## 7. 当前验证状态

2026-08-14 在同一 dirty worktree 上完成全部适用门禁：

- `echo-agent`：format；两组 all-feature Clippy（含 forbidden panic APIs）；workspace all-target/all-feature tests；no-default library check；12 个独立 feature check，全部通过。
- `echo-agent-cli`：format；两组 all-feature Clippy；workspace all-feature tests（app-core 659 passed/2 个显式 live-provider ignored，integration 5，CLI lib 90，main 9）；app-core no-default；GUI binary check；GUI feature 51 tests，全部通过。
- frontend：Prettier；Vitest 105 tests；TypeScript/Vite production build，全部通过。
- 定向回归：partial stream failure、conversation branch canonical history、workspace Running TaskRun gate、research partial side effect、Git/worktree cancellation/rollback 和 bounded output 全部通过。

尚未执行真实桌面手工交互和真实第三方 provider smoke；这不影响上述确定性回归结论，但 surface parity 剩余项仍须以端到端 fixture 和真实 transport 验收后才能关闭。
