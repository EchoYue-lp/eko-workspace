# EKO 应用层三模型 Review 对照

> 日期：2026-08-13
> 对照对象：Codex (GPT)、ZCode-ds (DeepSeek)、ZCode-glm (GLM)
> 权威结论：[application-review.md](application-review.md)
> 实施方案：[application-fix-plan.md](application-fix-plan.md)

## 1. 先回答三个问题

### 三个模型都找到了哪些优化方向？

三者在**问题域**上高度一致，都认为应用层需要优化：

1. workspace/config 切换与热更新；
2. conversation、memory 和文件持久化；
3. chat terminal、cancel、queue 和 sink；
4. TaskRuntime、Subagent、worktree、artifact 与恢复；
5. GUI/TUI/CLI/channel/cron/background 功能对等；
6. HITL、MCP、LSP、browser、terminal 管理；
7. frontend wire contract、状态管理、性能和可访问性；
8. observability、webhook、tool execution 和敏感字段；
9. evolution、research、analysis 和 output；
10. 重复 authority、死代码、测试与文档漂移。

### 三者找到的具体问题统一吗？

**不统一。** 它们共享同一审查目录和 29 个 A 类任务框架，所以目录级覆盖天然相似；但具体 source-to-sink 链路、运行时可达性和优先级差异很大：

- Codex：9 P0、109 P1、32 P2，共 150 个原子表现；粒度最细，后果导向。
- DeepSeek：0 P0、合并后 25 个 P1，并保留大量 P2；主路径和模式矩阵最清楚。
- GLM：0 P0、8 P1、57 P2、74 P3，共 139 项；维护性最全，但严重级别偏低。

这里的数量不能直接比较。Codex 将同一根因在不同入口、故障点和验收场景拆开；DeepSeek 会合并为 canonical P1；GLM 又把大量 test/doc/dead-code 单列为 P3。

### 我赞同吗？

我赞同三者共同的架构方向，也接受大部分有明确代码链路的 finding；但不接受任一模型的完整原始优先级：

- 接受 Codex 后续确认的 9 个 P0，因为它们满足共享协议中的数据损坏/敏感暴露定义。
- 接受 DeepSeek 对 Task 主路径、surface parity、writer read-only、double setup、MCP persistence 的复核和执行优先级。
- 接受 GLM 的 P2/P3、前端、死代码和重复实现地图，作为清理 backlog。
- 驳回或降级证据不足、不可达或套用错误威胁模型的条目，见第 6 节。

## 2. 模型侧重点

| 模型 | 最有价值的部分 | 主要不足 | 适合怎样使用 |
|---|---|---|---|
| Codex (GPT) | 跨写者数据竞争、generation、artifact lineage、敏感 sink、原子验收场景 | 拆分过细；109 个 P1 不能直接当 109 个迭代；部分“本地 plaintext”需要区分功能数据与日志泄漏 | 作为严重级别与故障测试基线 |
| DeepSeek | 25 个 canonical P1、端到端场景、六类 surface parity、Task/恢复主路径 | 早期基线漏掉后续发现的 P0；部分“adapter clean”正面结论过度 | 作为实施主干和能力矩阵 |
| GLM | 139 项广覆盖，前端/死代码/依赖/文档/重复 helper 详细 | P0/P1 明显低估；定义存在与运行时可达偶有混淆；部分在线服务安全思维不适合 EKO | 作为 P2/P3 清理和维护性 backlog |

## 3. 高优先级发现逐项对照

符号含义：`直接` = 同一具体失效链路；`同族` = 同一问题域但不是同一链路；`未提出` = 综合稿没有形成等价 finding。

| 问题 | Codex | DeepSeek | GLM | 我的裁决 |
|---|---|---|---|---|
| config 保存跨 workspace 覆盖 | 直接，P0 | 同族：switch 后 watcher/config scope 错乱，P1 | 同族：watcher、reload、cwd 漂移，P1/P2 | 接受 Codex P0；DS/GLM 发现的是同一可变 scope 根因的其他表现 |
| GUI draft 写入另一 workspace 同路径 | 直接，P0 | 同族：workspace switch/stub、diff/path authority | 同族：file/diff/project authority | 接受 P0；需要 workspace identity + expected revision |
| autosave/finalize stale transcript 覆盖 | 直接，P0 | 同族：exit store root、恢复与持久化问题 | 直接/接近：`save_conversation` lost-update，但仅 P3 | 接受 P0；GLM 找到窗口但严重低估 |
| primary/pool 多个 FileStore 快照 lost update | 直接，P0 | 同族：hot memory 投影不刷新、pool divergence，P1 | 同族：同样的 refresh 错误，P1/P2 | 接受 P0；投影 stale 与底层 lost update 要分别验收 |
| 旧 analysis run artifact 被重跑删除 | 直接，P0 | 同族：domain artifact destruction/Task cleanup，P2 | 未形成同等级链路 | 接受 P0；历史记录引用的 bytes 不可被重跑删除 |
| enrichment 失败用空值覆盖旧证据 | 直接，P0 | 同族：auto-ingest failure、artifact destruction | 直接/接近：失败仍盖 `enriched_at`、export 丢 source，P3 | 接受 P0；GLM 找到症状但没有跟到证据覆盖后果 |
| rule promotion 读失败后覆盖文件 | 直接，P0 | 同族：REPL reflection 越过 review gate，P1 | 同族：pre-compaction auto-write 绕过 review，P2 | 接受 P0；三者发现了 evolution 写入的不同入口 |
| webhook 原始 tool args/error 外发 | 直接，P0 | 直接，P1 | 直接，P1 | 三者强共识；接受 P0，按共享报告协议而非公网威胁模型裁决 |
| terminal input preview 持久化 | 直接，P0 | 同族：terminal lifecycle/cleanup | 未提出等价 P0 | 接受 P0，仅修持久化和日志；绝不能门控用户终端 |
| `events.jsonl` 坏尾行使 run 不可恢复 | 直接，P1 | 直接，P1 | 直接，P2 | 三者强共识；接受 P1，只容忍最后 torn tail，中间损坏 fail closed |
| pause 在 active wave 中变成 cancel | 直接，P1 | 直接，P1 | 仅列恢复/取消 hardening，P3 | 接受 P1；GLM 低估主路径恢复影响 |
| mid-wave fault 留下 Running sibling | 直接，P1 | 直接，P1 | 仅列 drain/recovery race，P3 | 接受 P1 |
| writer Subagent 被 `plan_mode` 变成只读 | 直接，P1 | 直接，P1 | 有 tool exposure/plan-mode 相关项但未升为核心 P1 | 接受 P1；旗舰 Task 能力跨所有 surface 失效 |
| chat error/cancel 被显示为 completed | 直接，P1 cluster | 直接，多个 P1 | 同族：sink owner、dead Err branch、cancel fixture，P2/P3 | 接受 P1；需要 typed TurnOutcome，不能只修前端文案 |
| interrupt prompt 留下 ghost turn | 直接，P1 | 直接，P1 | 同族：interrupt variant/ordering | 接受 P1 |
| REPL/channel 没有可达 cancel owner | 直接，P1 | 直接，P1 | 直接/接近，P2 | 接受 P1，属于模式对等和生命周期缺失 |
| `exit_workspace` 使用错误 conversation root | 直接，P1 | 直接，P1 | 同族：global/workspace store layout 不对称 | 接受 P1；boot/exit 必须调用同一 helper |
| workspace switch 留下 config/hooks/plugins/LSP 旧 scope | 直接，多 P1 | 直接，多 P1 | 直接，多 P1/P2 | 三者强共识；合并为一个 application generation 计划，保留各故障点测试 |
| hot memory mutation 刷新错 projection | 直接，P1 | 直接，P1 | 直接，P1 | 三者强共识 |
| channels-only 不启动 scheduler/background | 直接，P1 | 直接，P1 | 直接，P2 | 接受 P1；AGENTS.md 明确模式必须对等 |
| GUI browser bridge 因两个 `.setup()` 失效 | 直接，P1 | 直接，P1 | 未明确形成同一 finding | 接受 P1；是确定的 builder 组合错误 |
| GUI MCP 配置显示保存但重启丢失 | 直接，P1 | 直接，P1 | 主要发现 MCP over-gating 与 lifecycle，未形成同一持久化 finding | 接受 P1 |
| GUI permission rules 可管理但不生效 | 直接，P1 | 直接，P1 | 同族：HITL dispatcher 绕过/旧 gate dead code | 接受 P1；必须接现有 PermissionService 或删除假管理面 |
| REPL EOF 自动批准 | 直接，P1 | 直接，P1 | 同族：HITL provider 语义 | 接受 P1；EOF 与用户按 Enter 必须区分 |
| `SessionAllTools` 文案/scope 混淆 | 直接，P1 | 直接，P1 | 同族：permission alias 漂移 | 接受；scope 必须明确，不引入 full-auto 门控 |

## 4. 三者共同认可的中优先级优化

以下不是每一行都由三者使用同一个 ID，但三者都在相同 subsystem 报告了实质问题。

### Boot / Config

- config watcher 只在启动时绑定 cwd；switch 后不重建。
- AppConfig、env override、hook/plugin 配置存在多个快照或 reload 路径。
- headless/GUI composition 不一致，服务启动和 shutdown 顺序漂移。
- model/provider 更新在 primary Agent、pool 和 UI 间传播不完整。

我的判断：赞同。应由一个应用 generation/saga 解决，不能逐处加刷新调用形成更多旁路。

### Chat / Input / State

- `TauriChatSink` 承担 tool execution 持久化，sink 不再是纯渲染。
- TUI steer 绕过 `PreparedUserTurn`，staged paste 生命周期可能丢失。
- conversation edit/regenerate 只改 UI transcript，不 rewind Agent history。
- delete conversation 没有统一级联到 tool executions、Task runs、attachments/artifacts。
- restore empty/failure 与 pooled Agent 复用存在 stale context。

我的判断：赞同。统一到 driver observer、revisioned conversation commit 和 deletion tombstone；不要新建第二个 conversation store。

### Task / Worktree / Artifact

- TaskRuntime projection、review、artifact、frontend current selection 丢失 revision/attempt identity。
- recovery、pause/cancel、claim monotonicity 和 stale worktree 需要故障测试。
- runtime artifact projection/retention 字段存在写入但不消费的死表面。
- application adapter 有逐渐接管通用 retry/settlement 的风险。

我的判断：大部分赞同。通用 claim/retry/cancel 留 framework；EKO 只拥有 worktree、review、文件权威和 UI policy。

### Surface Parity

- TUI/CLI workspace 管理缺失或是假成功。
- Channel 缺 Task 管理、MCP/browser 管理、cancel/steer、scheduler/background service。
- TUI Subagent 详情被压成计数，多个事件未投影。
- GUI terminal/MCP/LSP 缺 awaited shutdown。
- research、evolution、reviewer verdict 等能力只在部分 surface 可达。

我的判断：赞同，而且这不是可选 UX backlog。AGENTS.md 已明确 GUI/TUI/CLI/channel 是同一 Agent 完全体；优先级按能力是否不可用决定。

### Frontend

- 手写 DTO 与 Rust wire/generated TS 漂移。
- `execution://event` 缺类型，存在 `as unknown as`。
- chat store/conversation store 循环依赖和双重 terminal authority。
- token streaming 导致大范围重渲染；artifact/output 传输过重。
- modal、textarea、mobile sidebar 缺少键盘和语义可访问性。
- ESLint/contract test 没有真正进入门禁。

我的判断：赞同。DTO/identity/terminal 契约先修，性能优化后做，避免优化错误的数据模型。

### Tools / Plugins / Subagents

- plugin Subagent 只更新 primary，不更新 pool；prompt 中 catalog 与 live registry 分裂。
- plugin/config 文件缺 reload 或一个坏组件使整批失败。
- MCP/LSP/browser disconnect/restart/shutdown 语义不完整。
- permission alias、deadline、sender correlation 在 surface 间漂移。

我的判断：赞同。修复方式是共享 registry/generation 和薄 surface adapter，不是引入另一套 plugin/Subagent 管理器。

### Output / Observability / Domain

- conversation export 丢 tool calls/results/attachments/reasoning。
- tool execution、webhook、run trace 有多个记录 owner 和关联 identity 丢失。
- research auto-ingest failure 被吞，export/audit 对 missing source 处理不一致。
- BibTeX key、escaping、enrichment status、library scan 有正确性/效率问题。

我的判断：赞同。先解决数据真实性、lineage、redaction，再做格式和性能。

### Dead Code / Duplication

- `ProjectIndex`、FileChangeTracker/CodingLoop 第二 authority。
- 旧 Persistence/SessionSearchEngine。
- dead output/LaTeX format、IpcAuth/旧 gate、TUI empty scaffold。
- 多份 atomic-write、parse format、permission alias、diff engine。
- frontend orphan generated files、dead event variant、无效 ESLint 配置。

我的判断：原则上赞同，但必须逐项证明应用内无真实调用后删除。不能用“CLI 未调用”删除 framework 的合理 public API。

## 5. 单模型发现的价值

### Codex 独有或明显更深入

- 9 个 P0 的完整破坏链路，而不只停留在“非原子写/可能 stale”。
- workspace identity 在 config 保存和 GUI draft 保存中的跨项目覆盖。
- conversation autosave/finalize 的 stale-prefix 覆盖。
- primary/pool FileStore 同路径多快照 lost update。
- immutable historical run 引用的 artifact 被重跑删除。
- enrichment 部分失败覆盖旧非空证据。
- evolution 读失败变空 authority 后写回。
- terminal input preview 进入持久化 sink。
- generation、terminal、claim/attempt、artifact identity 端到端一致性。

我的判断：大部分接受。这些是三方综合后优先级上升的主要原因。

### DeepSeek 独有或表达最清楚

- 25 个 canonical P1 的去重和重评级记录。
- pause -> cancel、mid-wave fault、ghost turn、writer read-only 等端到端主路径。
- 六类 entry surface 的能力矩阵和 Q-E2E 场景结果。
- double `.setup()`、MCP config 假保存、channels-only service 缺失。
- 明确区分“共享 core 已存在”与“management/control adapter 缺失”。

我的判断：接受其场景证据；不接受“0 P0”和“Task adapter 整体 clean”的扩大结论。

### GLM 独有或表达最完整

- 139 项 P1/P2/P3 的维护性目录。
- frontend DTO、重渲染、循环 import、a11y、HMR listener、ESLint 等细节。
- output/LaTeX、BibTeX、research scan、generated orphan 等长尾问题。
- dead code 数量、超大文件、依赖重复、atomic-write helper 重复地图。
- 多处文档、命名、测试 fixture 和状态字段漂移。

我的判断：大部分进入 P2/P3 backlog；不把所有项目都当独立里程碑，随权威路径切换顺手删除/修复。

## 6. 我不完全赞同或明确驳回的条目

| 原结论 | 我的判断 | 原因 |
|---|---|---|
| DeepSeek/GLM：应用层 0 P0 | 驳回 | 与共享 `REPORTING.md` 对数据损坏/secret exposure 的定义不一致；Codex 后续给出具体链路 |
| Codex：所有本地 plaintext tool args 都天然 P0 | 限定 | EKO 本地功能数据不因 plaintext 自动成为漏洞；进入日志、跨边界外发、权限过宽或无保留策略才按具体后果评级 |
| GLM：`gitignore::globstar_match` UTF-8 panic 为当前 P1 | 降 P3/接线前阻断 | panic 真实且必须修，但当前只在未接主路径的旧 ProjectIndex 内；定义存在不等于生产可达 |
| GLM/早期综合：Task adapter 已 clean | 驳回扩大表述 | 单一 file authority/revision service 是正面事实，但 pause/recovery/writer/terminal/identity 仍有主路径故障 |
| 缺 API key 必须让整个应用 bootstrap 失败 | 调整 | 应用可以进入设置 UI；必须保证首次请求前返回 typed、可操作配置错误，而非强制进程不能启动 |
| MCP private URL/allowlist 是安全要求 | 驳回 | EKO 是用户本机扩展；这是线上 SSRF 威胁模型。只保留命令名错误和明文 HTTP 等轻量校验 |
| 用 `permission_mode` 门控 terminal/MCP/browser | 驳回 | 直接用户交互不属于 agent 自动执行权限；违反项目产品边界 |
| TUI 没终端、research、evolution 可以视为产品选择 | 驳回 | AGENTS.md 明确要求功能对等；只能调整交互形式，不能取消能力 |
| 因 echo-agent-cli 未调用而删除 framework public API | 驳回 | framework 面向所有合理复用方；必须证明框架内过时或已被完整替代 |
| 所有 `#[allow(dead_code)]`、大文件、重复依赖都是 P2 | 降级逐项判断 | 它们是信号而不是自动 defect；需要具体维护/编译/行为影响，通常随触及模块清理 |

## 7. 最终统一视图

三者的共同答案不是“有 8/25/109 个 P1”，而是下面六个必须收敛的权威问题：

1. **写入身份**：任何配置、草稿、会话、memory、artifact、evidence 写入都绑定不可变 workspace/run/revision/generation，并拒绝 stale writer。
2. **持久化恢复**：Missing、Valid、Corrupt 分开；旧 bytes 可恢复；JSONL 只修 torn tail；不引入 SQLite。
3. **执行终态**：turn/Task/Subagent 每次只有一个 typed terminal、一个 cancel owner、一个 claim/attempt identity。
4. **配置 generation**：workspace switch 同时收敛 config/hooks/plugins/stores/primary/pool/LSP/MCP/UI，失败回滚或明确 degraded。
5. **模式对等**：GUI/TUI/CLI/channel/cron/background 共享 capability facts，只允许输入输出形式不同。
6. **权威删除**：新路径接管真实调用后，同批删除旧 store、sink owner、retry loop、projection、DTO 和 dead surface。

这六项使用 Codex 的破坏性后果决定优先级，使用 DeepSeek 的主路径与 surface 场景定义验收，使用 GLM 的维护性目录补齐清理和测试。

## 8. 原始报告索引

- [Codex application synthesis](codex/reports/synthesis/application-review.md)
- [DeepSeek application synthesis](zcode-ds/reports/synthesis/application-review.md)
- [GLM application synthesis](zcode-glm/synthesis/application-review.md)

原始报告仍用于查看完整 ID、文件锚点和 validation。实施时以当前代码重新验证，不直接复制旧行号。
