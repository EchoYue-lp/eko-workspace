# EKO 应用层修复实施计划

> 状态：2026-08-13 历史实施快照，不是当前行动入口
> 日期：2026-08-13
> 输入：[应用层三方综合复核](application-review.md)
> 模型对照：[Codex / DeepSeek / GLM 发现与裁决](application-model-comparison.md)
> 当前入口：[跨层与质量重验证](cross-quality-remediation.md) 和
> [75 项原子 finding 台账](cross-quality-finding-ledger.md)。旧 task 身份不再作为跨上下文恢复链接。
> 基线：`echo-agent` `3aa7929`，`echo-agent-cli` `b3b2e81`

## 1. 决策摘要

**可以同步开展应用层修复，但不能在当前 checkout 直接启动“应用层全量修复”。**

当前框架任务已在 `echo-agent/main` 工作树修改约 90 个文件，并因公共 API 变化同步修改了 `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs`、`src/tauri/commands/providers.rs`、`src/tui/events.rs` 和 `Cargo.lock`。它还在继续处理 FileStore、ConversationStore、ReAct terminal、HITL、Subagent、scheduler 等应用依赖。

推荐采用三轨并行：

1. **轨道 F（现有框架任务）**：独占 `echo-agent` 及跨仓库公共 API 接线。
2. **轨道 A0（可立即启动的应用 P0）**：独立 `echo-agent-cli` worktree，只改不依赖框架新 API 的应用文件。
3. **轨道 A1（前端独立修复）**：可与 A0 同一应用 task 分批完成，或使用第二个独立 worktree；只改 `web-frontend` 与稳定 Tauri 文件契约。

框架任务完成并提交后，再启动轨道 A2（workspace generation、turn/Task lifecycle、全模式 parity）。A2 必须先 merge 最新 framework main 并把 CLI 相对路径依赖指向合并后的 `../echo-agent`。

## 2. 当前并行风险

### 2.1 已确认的重叠面

| 框架任务正在修改 | 应用计划中的依赖 | 并行结论 |
|---|---|---|
| `echo-state::FileStore` / `FileConversationStore` | A-MEM-01、A-STATE-01、workspace store rebinding | 等待框架提交；否则应用会围绕过时构造/共享语义实现 |
| ReAct stream、terminal、cancel、snapshot approval | `drive_chat` typed outcome、HITL、GUI/TUI terminal reducers | 等待框架 terminal/approval contract 稳定 |
| Subagent registry/executor/types | writer Subagent、TaskRuntime dispatch、TUI Subagent detail | 等待框架 API 稳定 |
| scheduler、human-loop、MCP/LSP/provider adapters | channels service、HITL、MCP/LSP 管理面 | 应用产品策略可先设计，最终接线等待 |
| `echo-core` fs/redaction/retention utilities | 应用 atomic write、redaction、retention | 等待并复用，不在 CLI 再造 helper |
| CLI `agent_pool.rs`、providers、TUI events | workspace/memory/model/parity 修复 | 当前 checkout 禁止并发编辑这些文件 |

### 2.2 为什么不能共享当前 checkout

- 两个任务会读取和覆盖同一未提交文件，无法可靠区分作者和回滚范围。
- CLI 编译使用本地 `../echo-agent`，框架每次 API 变化都会令应用快检结果漂移。
- 框架任务的提交门禁包含 all-features；应用未完成代码会污染其验证。
- `Cargo.lock`、`agent_pool.rs`、TUI events 已经发生跨任务重叠，后续手工摘取极易丢改动。

### 2.3 安全并行的必要条件

- 应用修复使用 `echo-agent-cli/.worktrees/<feature>` 独立 worktree 和独立分支。
- A0 不修改 `echo-agent`，也不临时把 Cargo path 指向框架任务的脏工作树。
- A0 禁止修改当前重叠文件：`agent_pool.rs`、`src/tauri/commands/providers.rs`、`src/tui/events.rs`、`Cargo.lock`，除非框架任务先明确释放所有权。
- 每个 batch 有固定文件清单；发现需要越界时停止该项，转入 A2 backlog。
- 合并顺序始终是 framework 先、CLI 后；CLI 分支合并前先 merge CLI main，并基于已提交 framework main 跑完整门禁。

## 3. 实施门禁

每个 batch 开始前必须留下四类证据：

1. **当前性**：将相关 finding 标记为 `current/fixed/stale/regressed`，重新定位行号。
2. **重复性**：全仓库搜索相同类型、helper、store、事件和真实调用；扩展已有权威，不新增平行实现。
3. **分层**：明确 generic mechanism、EKO product policy、adapter boundary；拿不准留应用。
4. **并行所有权**：记录本 batch 文件集，并与框架任务 `git diff --name-only` 做交集检查。

任何涉及 framework 公共 API、状态机、Task 调度、terminal contract 的设计，在动手前须基于已有 B-REF 外部参考验证或重新查 Claude Code/Codex/Cursor 官方资料；纯 bug 修复不重复调研。

## 4. 轨道 A0：现在即可开始

### Batch A0.1：Domain artifact 与 enrichment 数据保护

**目标**：关闭两个不依赖框架 API 的 P0。

**范围**：

- `echo-agent-app-core/src/analysis.rs`
- `echo-agent-app-core/src/research_connectors.rs`
- 对应 domain tests

**改动**：

- artifact 使用不可变 `run/revision/attempt` 目录或 content-addressed 路径；重跑不得覆盖旧 run 引用的 bytes。
- Europe PMC enrichment 采用字段级 merge：请求失败/缺失不得用空值覆盖旧非空证据。
- 记录 enrichment source、attempt、成功字段和失败字段；只有成功字段进入新 generation。

**验收**：

- 两次同输入重跑后，旧 run 的 artifact 仍可按原引用读取且哈希不变。
- 预置完整 enrichment，令部分子请求失败，旧非空字段保持不变。
- 崩溃/写失败不会发布指向不存在内容的新 artifact record。

**删除目标**：overwrite-in-place 与“先删旧目录再生成”的路径。

**并行性**：绿色。当前框架 diff 不触及这些文件。

### Batch A0.2：Evolution rule/evidence 写入保护

**目标**：关闭错误读后覆盖 P0，并为后续 generation 收敛准备 receipt。

**范围**：

- `echo-agent-app-core/src/evolution/rule_promoter.rs`
- `echo-agent-app-core/src/evolution/evidence.rs`
- `echo-agent-app-core/src/instruction_provider.rs`（仅文件写入部分）

**改动**：

- 读取错误必须保留为 `Corrupt/ReadFailed`，不得转换为空规则集合。
- mutation 先写唯一 temp + fsync + rename，再追加 operation receipt；失败保持旧 bytes。
- accept/undo 使用 operation id 和 revision，重复执行幂等。
- 优先等待并复用框架任务新增的 `echo_core::utils::fs::atomic_write`；A0 分支可先写调用适配，但不得复制第二个 helper。

**验收**：读失败、temp write 失败、rename 失败、receipt append 失败分别注入；旧规则可恢复，操作状态不伪装为成功。

**并行性**：黄色。业务逻辑可做；公共 atomic-write helper 的最终接线等待 framework commit。

### Batch A0.3：Webhook 与终端持久化 redaction

**目标**：关闭两个敏感内容 P0，不影响用户直接使用终端/MCP。

**范围**：

- `echo-agent-app-core/src/chat_driver.rs` 的 webhook observer
- `echo-agent-app-core/src/webhook/*`
- Tauri terminal event/log repository 的 preview 写入点

**改动**：

- 在序列化 choke point redaction，而不是依赖每个 producer 自觉处理。
- tool args、raw error、headers/env/terminal input 使用结构化敏感字段屏蔽与长度上限。
- 终端 UI 仍显示用户当前交互内容；持久化/日志只保存 redacted metadata，不能用 permission gate 禁用终端。
- 复用框架 redaction helper；若 helper 签名尚未稳定，只先完成测试 fixture 和应用 wrapper，最后接线延后。

**验收**：API key、Bearer、password、cookie、常见环境变量值在 JSONL、日志、webhook body 中零明文；普通非敏感参数仍可诊断。

**并行性**：黄色。产品 sink 可改；共享 helper 接线等待 framework commit。

### Batch A0.4：GUI 文件写入 workspace identity

**目标**：关闭跨 workspace 草稿覆盖 P0。

**范围**：

- `web-frontend/src/stores/workspaceStore.ts`
- `web-frontend/src/stores/fileStore.ts`
- `src/tauri/commands/files.rs`
- 文件浏览/编辑器相关测试

**改动**：

- 打开文件时返回 `workspace_id + canonical relative path + revision/hash`。
- 保存必须携带同一 identity 和 expected revision；后端在目标 workspace 根内重新解析并做 compare-and-write。
- workspace switch 后旧 draft 进入 detached/stale 状态，只能显式另存或重新加载，不能直接覆盖。

**验收**：A/B workspace 有同相对路径同内容文件；A 打开后切 B 再保存，后端拒绝 stale identity，两个文件均不被误写。

**删除目标**：仅以 relative path + content hash 决定目标的保存路径。

**并行性**：绿色。避免触及 providers/TUI/agent pool。

## 5. 轨道 A1：可与 A0 并行的前端工作

这些工作不应抢在身份契约之前做大规模 store 重构，但可先完成局部、低耦合修复：

| Batch | 内容 | 前置 | 验收 |
|---|---|---|---|
| A1.1 | `ToolInfo` 等 DTO 改用生成类型，补 Rust serialize -> TS consume contract fixture | 无 | wire 字段逐项一致，移除 `as unknown as` |
| A1.2 | Modal focus trap、Escape、ARIA、移动侧栏键盘关闭 | 无 | Playwright 键盘路径通过，无焦点逃逸 |
| A1.3 | ESLint + jsx-a11y 接入 test/CI | 无 | lint 零警告；不与 Prettier/build 重复 |
| A1.4 | 删除确认无引用的 generated orphan/dead variant | A1.1 后 | `rg` 无调用，build/test 通过 |

`MessageBubble`/chat store 性能、TaskRuntimePanel identity 和 reviewer verdict 渲染延后到 A2，因为 terminal/revision identity 仍会改变 store shape。

## 6. 轨道 A2：框架提交后实施

### Batch A2.1：Workspace/config/memory 单 generation

**前置**：framework FileStore/FileConversationStore 和共享 fs helper 已提交；CLI merge framework API 适配完成。

**权威 owner**：应用层 `WorkspaceTransitionService`（名称可按现有风格调整，不新增第二套 store）。

**阶段**：

1. Prepare：解析 workspace、config、hooks、plugins、stores、LSP/MCP targets，不发布。
2. Validate：路径、配置、依赖和 generation token 全部有效。
3. Apply：按固定顺序替换 primary/pool 与服务句柄。
4. Publish：最后更新 `workspace.current` 和 UI snapshot。
5. Failure：回滚到 last-known-good，或发布明确 degraded receipt；不得 silent partial success。

**必须覆盖**：cwd、AppConfig、watcher、hooks、plugin runtime、primary/pool、conversation/memory/runtime stores、review integration、skills、LSP/MCP、tool artifact root、task store、UI。

**额外修复**：`exit_workspace` 使用与 boot 相同的 `create_conversation_store()`；删除基于 `Persistence::base_dir()` 的错误重建路径。

**验收**：在每个 apply 边界注入失败；任何观察者只看到旧 generation 或完整新 generation。

### Batch A2.2：Conversation 单写者与恢复

**前置**：A2.1；framework conversation store contract 稳定。

**改动**：

- conversation commit 使用 revision/CAS；GUI autosave 与 Agent finalize 不再各自覆盖全记录。
- edit/regenerate 同时更新显示 transcript 和 Agent context，或明确创建新 branch/revision。
- pooled Agent restore 前先清空旧 history；restore failure 不展示为可继续会话。
- delete 使用 tombstone，协调 active turn、pool owner、tool executions、Task runs、attachments 和 artifacts。

**验收**：autosave/finalize 并发、edit/regenerate、空会话复用、删除中 active turn、崩溃重启矩阵。

### Batch A2.3：Typed TurnOutcome 与唯一 lifecycle owner

**前置**：framework typed terminal/cancel 已提交。

**改动**：

- `drive_chat` 返回 typed outcome，而非用 `Result<(), String>` 与事件 payload 双重表达。
- 一个 supervisor 拥有 cancel token、join handle、queued turn、shutdown 和 durable cursor。
- sink 只渲染；tool execution persistence、webhook 和 conversation settlement 作为 driver observer。
- GUI/TUI/CLI/channel reducers 使用同一 terminal fixture。

**验收**：success/error/cancel/timeout/disconnect/remount/restart 每种恰好一个 terminal；partial output 保留；队列必释放。

**删除目标**：surface-local terminal inference、fabricated cancel error、Tauri sink 持久化 owner、未托管 detached sends。

### Batch A2.4：Task claim/attempt 与恢复

**前置**：framework Task executor、claim、pause/cancel、Subagent invocation API 稳定。

**改动**：

- 移除 writer Subagent 的错误 `set_plan_mode(true)`；保持 readonly Subagent 的明确限制。
- `run_id + revision + task_id + claim/attempt` 贯穿 dispatch、worktree、verification、integration、artifact、trace、frontend selection。
- `events.jsonl` 只修复最后一个 torn tail；中间损坏 fail closed 并保留 evidence。
- pause/fault 后清理 Running sibling；同进程 resume 不轮询卡死。
- app adapter 删除通用 retry/settlement 主循环，只提供 EKO worktree/review policy。

**验收**：writer 实际修改隔离 worktree；pause/resume；mid-wave store fault；stale claim；crash tail；revision change；artifact lineage。

### Batch A2.5：HITL 与 MCP/LSP 产品接线

**前置**：framework approval scope、effective arguments、MCP/LSP lifecycle 稳定。

**改动**：

- permission rules 真正进入 shared PermissionService；EOF/transport loss reject。
- UI 明确区分“本工具本会话”与“全部工具本会话”。
- GUI MCP 配置原子持久化并从同一文件启动；移除私网/allowlist 过度门控，保留命令名与明文 HTTP 轻校验。
- LSP restart、MCP/LSP shutdown 暴露给 GUI/TUI/CLI。

**验收**：相同 approval fixture 通过四种交互面；本地 stdio/private HTTPS MCP 可用；窗口关闭有 awaited cleanup。

### Batch A2.6：Surface parity 收敛

建立一张可执行矩阵，而不是文案清单：

| 能力 | GUI | TUI | CLI | Channel | Cron/BG |
|---|---|---|---|---|---|
| Chat/Auto/Task | trigger + render + cancel + restore | 同 | 同 | 同 | 合法子集 + typed terminal |
| Workspace | switch/exit/snapshot | 同 | 同 | 当前 scope 可观察 | generation 固定 |
| Task/Subagent | create/update/list/execute/pause/resume/cancel/review | 同 | 同 | 同 | execute/recover |
| HITL | scope/timeout/reconnect | 同 | 同 | sender correlation | reject/typed blocked |
| Memory/evolution | review/accept/undo/refresh | 同 | 同 | 同能力的命令映射 | scheduled receipt |
| MCP/LSP/browser/terminal | 管理 + lifecycle | 同 | 同 | 适用管理命令 | health/shutdown |

每个单元必须有 definition -> composition -> trigger -> event/snapshot -> reducer/render -> cancel/recovery 证据。缺任一环即不算功能可用。

## 7. 清理批次

清理不单独拖成永久尾项；在权威路径切换的同一 batch 删除被替代实现：

- `ProjectIndex`、`FileChangeTracker`/空 CodingLoop authority；
- 旧 `Persistence`/`SessionSearchEngine` 会话权威；
- dead output/LaTeX 格式路径；
- `IpcAuth`/旧 permission gate helper；
- TUI `parallel_tasks` 空 scaffold（若 A2.6 不复用）；
- frontend dead DTO、orphan exports、dead event variant；
- Tauri/CLI 重复 parse/normalization helper。

删除 framework public API 不属于应用批次；必须由框架任务按“所有合理复用方”标准处理。

## 8. 分支与合并方案

### 推荐分支

- 框架现有任务：继续在当前 `echo-agent/main` 完成并提交，期间不让应用任务改 `echo-agent`。
- 应用 A0：`echo-agent-cli/.worktrees/application-p0`，分支 `fix/application-p0`。
- 前端 A1：若另开并行 task，`echo-agent-cli/.worktrees/frontend-contracts`，分支 `fix/frontend-contracts`。
- A2：框架合并后从最新 CLI main 新建 `fix/application-convergence`。

### 合并顺序

1. framework 每个稳定 batch 完整门禁并提交。
2. A0/A1 分支先 merge 各自 CLI main，解决彼此冲突后分别 squash merge。
3. 更新 CLI 的 framework 依赖到已提交 main，禁止指向脏 worktree/绝对路径。
4. 创建 A2 分支，完成跨层接线。
5. 最后跑两个仓库所有适用提交门禁和条件矩阵。

### 所有权协议

每个任务开工时在自己的进度说明列出文件清单。以下文件在框架任务结束前由框架任务独占：

- `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs`
- `echo-agent-cli/src/tauri/commands/providers.rs`
- `echo-agent-cli/src/tui/events.rs`
- `echo-agent-cli/Cargo.lock`

若框架任务还要修改新的 CLI 文件，应先把路径加入独占清单；A0 发现交集后不自行合并，转入 A2。

## 9. 验证与完成定义

### 每个应用 batch

- 最小相关 Rust/前端测试。
- 新增 fault/concurrency/round-trip fixture，不只测试 happy path。
- `rg` 证明旧 owner/命名/调用已删除。
- 记录 reviewed framework/CLI commit 和 dirty-state。

### 应用合并门禁

按根 `AGENTS.md` 执行：

```bash
cd echo-agent-cli
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used \
  -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-features --locked
cargo check -p echo-agent-app-core --no-default-features --locked
```

触及 Tauri/GUI 时补 GUI 矩阵；触及 `web-frontend` 时运行 Prettier、全部测试和 build。任何失败都必须修复，不能归类为“已有失败”后跳过。

### 全计划完成定义

- 9 个 P0 均有 fault regression 且旧 bytes/secret invariant 可验证。
- workspace/config/memory/conversation 只有一个 generation authority。
- turn 和 Task 每次执行只有一个 typed terminal/claim identity。
- GUI/TUI/CLI/channel/cron/background capability matrix 全部有可执行证据。
- adapter 不拥有第二套 DAG/retry/terminal/store。
- 旧权威和过时术语已删除。
- 两仓库适用门禁全绿；Cargo path 无绝对 worktree 路径。

## 10. 建议的下一步

现在最合理的动作是：**让框架任务继续独占当前 checkout，同时新建一个独立应用 worktree，只执行 A0.1 和 A0.4。** 这两个 batch 完全位于应用仓库、与当前框架修改无文件交集，也不依赖正在变化的 framework API。A0.2/A0.3 可以同步准备测试，但公共 atomic-write/redaction helper 的最终接线等框架第一个稳定提交。A2 全部等待框架 terminal/store/Subagent API 稳定后再开始。

## 11. 可直接调度的并行计划

### 11.1 并发上限与泳道

建议同时运行 **4 个 Codex 会话**，其中 1 个是已经在运行的框架会话，新增 3 个应用会话。不要再增加第 5 个实现会话；当前主要瓶颈会从编码变成公共契约漂移、CLI 门禁和合并冲突。

| 泳道 | 仓库/worktree | 当前状态 | 独占范围 | 禁止范围 |
|---|---|---|---|---|
| F | 当前 `echo-agent` + 当前 CLI checkout | 已运行 | 全部 framework；框架 API 所需的 CLI adapter | 不能顺手实现下列 A/B/C 产品功能 |
| A | `echo-agent-cli/.worktrees/app-domain-integrity` | 立即新建 | analysis/research/evolution domain 数据保护 | state、AgentPool、TaskRuntime、plugin、scheduler、browser、TUI |
| B | `echo-agent-cli/.worktrees/app-file-identity` | 立即新建 | GUI 文件打开/保存 identity 与 revision | providers、state、TaskRuntime、plugin、所有 frontend package metadata |
| C | `echo-agent-cli/.worktrees/app-frontend-contracts` | 立即新建 | generated DTO contract、a11y、lint | fileStore/workspaceStore/files command、chat/Task store identity 重构 |

### 11.2 当前框架泳道独占文件

截至本次调度检查，框架任务已经修改以下 CLI 文件。在它提交并释放所有权前，A/B/C 不得编辑：

```text
Cargo.lock
echo-agent-app-core/src/agent_pool.rs
echo-agent-app-core/src/browser/mod.rs
echo-agent-app-core/src/plugin_runtime.rs
echo-agent-app-core/src/scheduler/runner.rs
echo-agent-app-core/src/state.rs
echo-agent-app-core/src/tasks/task_runtime/executor.rs
echo-agent-app-core/src/tasks/task_runtime/store.rs
src/cli/modes.rs
src/main.rs
src/tauri/commands/providers.rs
src/tauri/desktop.rs
src/tui/events.rs
```

这是动态清单。每个应用会话开始一轮编辑前都运行：

```bash
git -C /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli diff --name-only
```

若框架任务新增 CLI 文件与本泳道交集，该文件立即冻结；应用会话保留测试/设计，把实现移到下一 wave。

### 11.3 Wave 1：立即并行

#### 泳道 F：继续框架修复

**任务**：保持现有目标，优先形成三个可消费的稳定 checkpoint：

1. `F-CP1`：`echo_core::utils::fs`、redaction、FileStore/FileConversationStore 稳定。
2. `F-CP2`：typed terminal/cancel/approval/effective arguments 稳定。
3. `F-CP3`：Task claim/pause/retry、Subagent invocation、plugin/scheduler lifecycle 稳定。

**交付要求**：每个 checkpoint 在 `echo-agent` 独立提交；若包含 CLI adapter，CLI 也必须形成独立提交。不能让应用会话长期依赖框架脏工作树。

#### 泳道 A：Domain 数据完整性

**分支**：`fix/app-domain-integrity`

**固定文件集**：

```text
echo-agent-app-core/src/analysis.rs
echo-agent-app-core/src/research_connectors.rs
echo-agent-app-core/src/research.rs（仅必要的 artifact record）
对应 tests/fixtures
```

**任务**：

- historical run artifact 不可变；重跑使用新 attempt 路径。
- enrichment 字段级 merge；失败不覆盖旧非空证据。
- auto-ingest persistence failure 返回真实 partial/failure，而非 Tool success。
- export/audit 对 missing source 使用同一语义。

**第一提交**：artifact lineage + 重跑回归测试。

**第二提交**：enrichment/ingest merge + fault tests。

**完成屏障**：不得为了 atomic write 自建 helper；若需要，等待 `F-CP1` 后追加第三个接线提交。

#### 泳道 B：GUI 文件写入身份

**分支**：`fix/app-file-identity`

**固定文件集**：

```text
src/tauri/commands/files.rs
web-frontend/src/stores/fileStore.ts
web-frontend/src/stores/workspaceStore.ts
web-frontend/src/components/file-browser/*（仅保存交互）
对应 Rust/TS tests
```

**任务**：

- open 返回 `workspace_id + relative_path + revision/hash`。
- save 要求 expected identity/revision，后端根内解析并 CAS。
- switch workspace 后旧 draft 标 stale，禁止误写同相对路径文件。
- Git/diff command failure 不得显示为 clean/equal。

**第一提交**：Rust command contract + A/B workspace collision tests。

**第二提交**：frontend stale-draft UX + contract tests。

**完成屏障**：B 不修改 `package.json`/lock；所需测试依赖由 C 统一管理。

#### 泳道 C：Frontend contract 与 a11y

**分支**：`fix/app-frontend-contracts`

**固定文件集**：

```text
web-frontend/src/types/*
web-frontend/src/generated/*
web-frontend/src/components/tools/*
web-frontend/src/components/common/modal 或现有 modal 文件
web-frontend/src/components/layout/*
web-frontend/package.json
web-frontend/package-lock.json
web-frontend/eslint.config.js
对应 tests
```

**任务**：

- 手写 `ToolInfo`/MCP/Skill DTO 收敛到 generated types。
- Rust serialize -> TS consume fixture，覆盖 null/undefined 和 event variants。
- modal focus/Escape/ARIA、textarea label、mobile sidebar 键盘关闭。
- ESLint + jsx-a11y 接入门禁。
- 在证明零引用后删除 orphan generated export/dead event variant。

**第一提交**：DTO contract fixture 与 wire 修复。

**第二提交**：a11y + lint。

**完成屏障**：不重构 chatStore、conversationStore、TaskRuntimePanel；它们等待 typed terminal/revision contract。

### 11.4 Wave 1 的验证分工

为避免三个应用会话同时跑全 workspace、互相争用编译缓存：

| 泳道 | 迭代快检 | 分支完成门禁 |
|---|---|---|
| A | `cargo test -p echo-agent-app-core <analysis/research test>` | `cargo fmt --all -- --check` + app-core tests |
| B | files command 定向 Rust test + 指定 Vitest | GUI feature check + frontend test/build |
| C | 指定 Vitest/ESLint/Prettier | frontend Prettier + 全测试 + build |
| F | framework 定向 crate/test | framework 完整门禁和适用 feature matrix |

CLI 全 workspace Clippy/test 只在 Wave 1 集成分支运行一次，不要求 A/B/C 各自重复三遍。各分支自己的所有已执行命令仍必须零失败。

### 11.5 Wave 1 合并屏障

Wave 1 不是“谁先完成谁直接 merge main”。使用以下顺序：

1. F 提交 `echo-agent` 的 `F-CP1`；若有 CLI adapter，也先提交到 CLI main。
2. C merge 最新 CLI main，解决 package/generated contract 后先合并 C。
3. B merge 更新后的 CLI main，适配 C 的 generated types，再合并 B。
4. A merge 更新后的 CLI main，接入 `F-CP1` atomic-write/redaction（若需要），再合并 A。
5. 在 CLI main 执行完整提交门禁；任何失败由引入该失败的最后一个 batch 修复。

理由：C 先建立 wire/test 基础；B 消费该 contract；A 与 frontend 几乎无交集且可最后接 framework helper。

### 11.6 Wave 2：按 framework checkpoint 解锁并行

Wave 2 最多仍保持 4 条泳道，但重新分配应用范围。

| 泳道 | 解锁条件 | 工作包 | 主要文件 owner |
|---|---|---|---|
| F | Wave 1 后继续 | `F-CP2/F-CP3` | framework + 必要 adapter |
| D | `F-CP1` | workspace/config/memory generation prepare/validate/apply/publish | `state.rs`、config watcher、instruction/memory、AgentPool |
| E | `F-CP1` | conversation revision/CAS、restore、delete tombstone/cascade | conversations commands、conversation stores、chat/conversation frontend store |
| G | `F-CP1` | webhook/terminal/tool execution redaction、retention、single recorder准备 | chat driver、webhook、tool_execution、terminal persistence |

注意：D 的 `state.rs/agent_pool.rs` 当前由 F 占用，只有 F 明确完成相应 CLI adapter 提交后才能建立 worktree。E/G 不碰 terminal 类型，只完成 revision/redaction 的稳定部分。

**Wave 2 合并顺序**：F-CP1 adapter -> D -> E -> G。每个分支合并前 merge 最新 CLI main。

### 11.7 Wave 3：跨层生命周期收敛

`F-CP2` 和 `F-CP3` 全部提交后启动。此阶段不建议四个实现分支同时改同一 event/type graph，而采用“一个契约 owner + 两个 surface consumer”：

| 泳道 | 工作包 | 独占文件 |
|---|---|---|
| H（契约 owner） | `drive_chat -> TurnOutcome`、lifecycle supervisor、observer settlement | app-core chat driver/types/resources |
| I（Task owner） | claim/attempt、pause/fault cleanup、writer plan-mode、JSONL recovery | app-core TaskRuntime/infra/worktree |
| J（surface owner） | GUI/TUI/CLI/channel reducers、cancel/steer、typed terminal fixtures | Tauri chat、TUI、CLI/channel、frontend chat store |

H 必须先提交 typed application contract；J 只在该提交上实现 surface adapters。I 与 H 可以并行，但不能自行定义第二套 terminal/event identity。

**Wave 3 合并顺序**：H -> I -> J。

### 11.8 Wave 4：模式对等与清理

最后按相互独立的 capability 并行：

- K：workspace/research/evolution 在 TUI/CLI/channel 的能力对等。
- L：Task manage/review/artifact/reviewer verdict 的全 surface 对等。
- M：MCP/LSP/browser/terminal 管理与 awaited shutdown。
- N：dead authority/DTO/output/scaffold 删除与性能优化。

每个 capability 必须通过统一矩阵 fixture；不能用“有 command 定义”代替 runtime reachability。K/L/M 合并后再做 N，防止提前删除仍被迁移使用的 adapter。

### 11.9 会话间协调协议

每个并行会话必须在首次编辑前声明：

```text
Branch/worktree:
Owned files/globs:
Forbidden files/globs:
Framework checkpoint consumed:
Expected commits:
Validation commands:
```

每天或每个提交后只交换以下信息，不共享大段对话：

- commit hash；
- 新增/修改公共类型或 wire 字段；
- 文件所有权变化；
- failed gate 与实际原因；
- 下一个会阻塞其它泳道的 checkpoint。

禁止两个会话口头约定“各改一半”同一个大文件。大文件只能有一个 owner，其他会话通过新测试、fixture 或等待提交协作。

### 11.10 立即执行清单

1. 保持现有框架任务继续运行，不向其工作树叠加应用编辑。
2. 要求框架任务以 `F-CP1/F-CP2/F-CP3` 形成可消费提交，而非最后一次性提交。
3. 新建 A、B、C 三个 CLI worktree/分支。
4. 将本节的 owned/forbidden 文件直接放入三个新会话 prompt。
5. Wave 1 完成前不启动 workspace、TaskRuntime、plugin、scheduler、browser 或 chat terminal 重构。
6. 按 C -> B -> A 的顺序集成应用分支，再执行一次 CLI 完整门禁。
