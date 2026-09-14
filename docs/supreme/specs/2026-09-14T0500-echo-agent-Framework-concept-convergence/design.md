---
title: echo-agent Framework Concept Convergence
artifact: design
carrier: markdown
---

# echo-agent Framework Concept Convergence

## 问题与目标

`echo-agent` 已经完成全 workspace discovery、高风险 Audit、SDK contract scope reset 和十一个 repair slice，当前有 11 张 Capability Map、11 个 Behavior、11 条 Rule、26 个 Asset、39 份 Audit、31 份 Evidence 和 92 个已映射 GitHub Issue 的 Finding。但长期正式文档仍主要按功能编排，缺少一条跨模块的概念主线：

- README 架构图主要是组件罗列，没有表达公共 facade、11-package DAG、Host/SDK consumer 和 framework/application 边界。
- Agent、Session、Conversation、Invocation、Turn、Task、Subagent、Workflow 与 Run 在多篇文档分别出现，但没有一份可以防止错误聚合的限定术语表。
- Context、transcript、runtime checkpoint、long-term memory、Journal、Projection、Trace 和 Delivery Ledger 的事实权威虽已在语义材料中分开，公开文档的跨章路由仍不足。
- 功能章节描述“有什么”，却未统一说明接纳、执行、取消、终态、恢复、清理和外部副作用的责任人。
- Product Backend、Frontend/Desktop 与 Device sync 是 EKO 产品问题，不能因为概念表完整就塞进通用 framework。

目标是建立一套可从概念导航到真实源码、详细章节、可执行 example 和语义证据的正式中英文文档体系。该体系必须表达已经确认的唯一权威和生命周期，不得把 74 个 open Finding 改写成“已实现保证”。

## 业界调研与取舍

### 参考的成熟实现

- [Claude Code: How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works) 以 gather context → take action → verify results 的 agent loop 为主线，再分开 session、context window、checkpoint 和 permission。它明确 session 历史、context window 与可持久 memory 不是同一个对象。
- [Claude Code: Subagents](https://code.claude.com/docs/en/sub-agents) 以独立 context、限定 tool/permission 和结果回传描述 Subagent，而不是另建第二套执行角色领域。[Permissions](https://code.claude.com/docs/en/permissions) 还明确 prompt 影响模型行为，runtime policy 才执行权限边界。
- OpenAI 官方 Codex 源码把 [Thread/Turn/Item JSONL events](https://github.com/openai/codex/blob/main/codex-rs/exec/src/exec_events.rs) 明确分层，并把 [session-scoped state](https://github.com/openai/codex/blob/main/codex-rs/core/src/state/session.rs) 与 [active turn state/cancellation](https://github.com/openai/codex/blob/main/codex-rs/core/src/state/turn.rs) 分开。这说明 surface event 可以类型化，但不应把 Thread、Turn 和 Item 压成一个运行态。
- [Cursor Agent](https://cursor.com/docs/agent/overview) 将 instructions、tools 和 model 视为 agent 的基本组合，并把 checkpoint 明确定位为与 Git 分离的文件回滚点。[Plan Mode](https://cursor.com/docs/agent/plan-mode) 把 plan 作为可审阅、可编辑 artifact，而非一组额外 run state；[Subagents](https://cursor.com/docs/subagents) 同样以独立 context 和返回结果建模。
- [LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) 区分 thread-scoped checkpointer 与 cross-thread store，用于分开短期执行恢复与长期知识。
- [Devin Session Tools](https://docs.devin.ai/work-with-devin/devin-session-tools) 将 Shell、IDE 和 Browser 的操作记录收敛到可观察 progress view，支持人类接管；这是观察与交互 surface，不代替执行与持久化权威。

### 跨系统共性

1. Agent 是指令、模型、上下文和可用能力的组合者，不是所有运行数据的总表。
2. Session/Thread 是多 Turn 作用域；Turn/Invocation 是一次可接纳、可取消、有终态的执行；stream item/event 是观察投影。
3. Plan 是可审阅 artifact，Task graph 才拥有 revision、dependency、claim 和 settlement。
4. Subagent 以独立 context、能力边界、attempt identity 和 outcome 表达，不另设第二套执行角色概念。
5. Checkpoint、transcript、long-term memory、trace 和文件回滚的持久性、恢复用途和retention 不同。
6. Permission policy 是runtime enforcement；prompt/rule/plan 可引导行为，但不能伪装成已执行的权限决定。
7. UI/feed/progress/trace 提供观察和接管，不从 EOF、最后一条文案或渲染状态反推业务终态。

### echo-agent 的取舍

- 采用上述概念分层和文档组织方式，但只记录仓库已存在的行为。
- 不引入通用 `AgentRevision`。Revision 必须被 owner 限定，例如 Task revision、Plugin generation、runtime-state incarnation 或 SDK schema revision。
- 不为“概念统一”新建全局 Session/Run/State 聚合，不改变现有 runtime 生命周期。
- 不把 EKO 的 Workspace、Device sync、GUI/TUI reducer、DomainProfile、review/worktree policy 或本地产品权限放进 framework。
- 不以 SDK canonical identity 数量表示项目语义完成度。

## 目标行为

### 三层文档入口

`docs/en/` 与 `docs/zh/` 各新增三份同构的基础文档：

1. `architecture.md`：解释 package DAG、分层、公共 facade、framework/application 边界、Host/SDK 和 learning/examples consumer。
2. `concepts.md`：定义 Agent、Session、Conversation、Invocation、Turn、Task、Plan artifact、SubagentAttempt、Workflow、Context、Checkpoint、Journal、Projection、Trace、Delivery 与 Effect，并列出不存在的聚合概念。
3. `lifecycles.md`：以入口、权威、事件、副作用、终态、恢复和清理为主线，串联 Agent Turn、Context、Task/Subagent、Observation/Persistence、Tool/Permission、Extension 和 SDK 生命周期。

Root `README.md`/`README.zh.md` 继续承载产品定位、快速上手以及已由 Cargo metadata 合同验证的 feature、workspace 和 example 摘要；其 Architecture 段收窄为简化分层图与上述三份入口链接。`docs/*/README.md` 是完整文档导航权威。现有编号功能章节继续拥有领域细节，基础文档不复制 API 列表和配置表。

### 架构分层

```text
Embedding application / protocol surface
        |
        v
echo_agent root facade and public composition
        |
        +--> echo_core          contracts and domain primitives
        +--> echo_execution     execution mechanisms
        +--> echo_state         persistence, memory, compression
        +--> echo_orchestration turn, task, subagent, workflow
        +--> echo_integration   external protocol implementations
        +--> echo_tools         reusable tool implementations
        +--> echo_macros        compile-time adapters

Independent consumers / boundaries:
  echo-sdk-protocol -> echo_core
  echo-sdk-host     -> echo_agent + echo-sdk-protocol
  echo-agent-learning -> echo_agent
```

`Cargo.toml` 依赖图是 package topology 事实源；root facade 是稳定公共路径权威；crate 内部模块不因文档图改变 owner。

### 核心概念与唯一权威

| 概念 | 精确含义 | 权威 / 来源 | 不是什么 |
| --- | --- | --- | --- |
| Agent | 实现模型调用、context 准备、tool loop 与 raw execute/chat 的能力对象 | `Agent`/`ReactAgent`、AgentConfig、factory | 不是 Session/Task/Trace 总表 |
| Session | protocol/channel 入口的多 Turn 作用域与资源 owner | ACP/channel session 实现 | 不是 ConversationStore 或 model context |
| Conversation | 稳定的对话/历史作用域 | ConversationStore 及其 stable scope | 不是一次 Turn |
| Invocation | 对一次 Agent 调用的配置、identity 与 resource guard | InvocationContext / EventIdentity | 不自动等同 product Run 或 trace Run |
| Turn | 一次可接纳、可取消且有 receipt 的 driven execution | AgentTurnDriver / TurnReceipt | 不是 Task graph node 或 stream item |
| Task | 带 revision、dependency、claim 和 settlement 的持久执行单元 | TaskRevisionService / RuntimeTaskService | 不是 Todo projection 或 Plan 文本 |
| Plan | 可编辑、可审阅的 Task 规格 artifact | Task revision 内的 specification | 不拥有独立 run state machine/store |
| Subagent | 在独立 context 和限定 capability 中执行的 Agent attempt | Subagent registry/executor/control/outcome | 不存在第二套执行角色层 |
| Context | 当前模型可见的 bounded messages/resources | ContextManager 及 invocation policy | 不是 transcript、checkpoint 或 long-term memory |
| Checkpoint | 特定 owner 的恢复快照 | RuntimeStateStore / workflow checkpointer | 不是 Journal fact log 或 Git checkpoint |
| Journal | 特定持久领域的 ordered fact commit | Journal implementation | 不是所有 event 的全局总线 |
| Projection | 由 fact/status 导出的查询或界面视图 | 领域 reducer / consumer | 不反向决定事实 |
| Trace | 对执行的诊断 observation | RunStore / trace Run | 默认不是业务 commit 或 Turn terminal |
| Delivery | 有 route/payload/attempt/settlement 的副作用交付记录 | Delivery Ledger | 不宣称任意外部 effect exactly-once |
| Revision | 被具体 owner 限定的不可变版本 | Task/Plugin/schema/runtime scope 各自 API | 没有通用 AgentRevision |

### 生命周期主线

#### Agent Turn

```text
surface request
  -> resolve/create Agent and Session resources
  -> assemble InvocationContext
  -> admit/accept Turn
  -> raw Agent loop (LLM <-> Tool/Effect)
  -> producer settlement
  -> TurnReceipt terminal
  -> trace/history/UI projections
  -> close owned resources
```

文档必须明确：取消请求不是终态，EOF 不是成功，stream 最后一条文本不是 receipt。Raw Rust `Agent` API 作为合理低层 contract 保留；已知 channel/direct route 覆盖缺口继续由 Finding #107/#36 追踪，不宣称已全入口统一。

#### Context and persistence

```text
stable conversation scope
  -> runtime incarnation hydrate
  -> context select/budget/assemble
  -> optional compression
  -> execute Turn
  -> transcript projection + runtime checkpoint
  -> clear one named authority, never "clear everything" implicitly
```

`reset active context`、`clear runtime incarnation`、`delete transcript`、`delete long-term memory` 和 `prune trace/checkpoint` 必须分开说明。压缩只改变模型可见窗口，不重写 transcript 或 fact history。

#### Task and Subagent

```text
Task revision commit
  -> ready frontier
  -> claim PlanTask
  -> dispatch SubagentAttempt
  -> progress/effect observations
  -> Subagent outcome
  -> Task settle/retry/pause/cancel
  -> Todo/UI projections
```

Task DAG 是关系权威；Workflow 是相邻通用能力，不为文档简化而合并。TaskClaim 与 SubagentAttempt 当前的关联缺口继续由 Finding #99 追踪。

#### Tool, Permission and Effect

```text
resolve tool
  -> schema/custom validation
  -> effective permission decision
  -> admission/concurrency control
  -> execute external effect
  -> typed result + trace/audit observation
  -> owned cleanup/settlement
```

Framework 只描述通用的自动 Agent effect policy 和 sandbox 原语。EKO 本地用户直接操作终端/文件选择器/MCP 的产品权限取舍属于 `echo-agent-cli`，不应出现在 framework 规则中。

#### Observation, persistence and delivery

```text
domain event/fact
  -> named authority commits or publishes
  -> projection/feed/history/trace consumers
  -> retention or checkpoint
  -> generation/replay fence
```

Journal 只在明确 journal-backed 领域中是 fact authority；EventEnvelope 表达 identity/order，Trace 表达诊断，Feed 表达消费视图，Delivery Ledger 表达副作用交付。任何一个都不是全 workspace 的全局事实库。

#### Extensions

```text
discover -> parse -> prepare -> validate -> publish generation
        -> activate/use -> replace/reload -> withdraw -> close
```

MCP、Hook、Skill、Plugin 和 LSP 保留各自 registry/resource owner；Plugin generation 可组合发布，但不抹掉 child resource 的 unwind/close 责任。现有 extension lifecycle Finding 必须在公开文档中以收窄承诺处理，不得写成已实现的 atomic hot reload。

#### SDK and contracts

```text
Rust public facade
  -> deterministic inventory/classification
  -> protocol and Host operations
  -> TypeScript/Python/Java source SDKs
  -> contract and connection tests
```

SDK 文档必须分开 external contract、Host/Rust-only、language intrinsic、internal helper 和 deferred backlog。`9684` 是漂移 inventory，`5607` 才是当前 external contract scope；数量可由生成物展示，但不手写成无校验承诺。

### 用户提出领域的去向

| 用户领域 | Framework 文档处置 |
| --- | --- |
| Agent 生命周期 | 记录 Agent/Session/Invocation/Turn；不新建 AgentRevision |
| Session 与 Context | 记录 context 装配、窗口、压缩、checkpoint 和限定 clear |
| Run 执行 | 以 driven Turn 和特定领域 Run 表达；不建全局 Run 聚合 |
| Observation | 区分 event/fact/projection/feed/history/trace/retention/generation |
| Workspace 与 Device | 只记录 invocation workspace reference、file/process/sandbox；Device sync 不存在 |
| Permission 与 Tools | 记录 validation、policy composition、decision、effect、cleanup |
| MCP、Hook、Library | 记录 extension lifecycle、outbox/delivery/resource transfer；Library 是 package 复用而非 runtime registry |
| SDK 与 Contracts | 记录 Rust authority、protocol、Host、source SDK 和 scope classification |
| Product Backend | 明确属于 embedding application / `echo-agent-cli` |
| Frontend 与 Desktop | 明确属于 surface projection / `echo-agent-cli` |
| Delivery 与 Observability | 记录 framework delivery primitive、trace/metrics/telemetry 与诊断边界；部署策略属于 consumer |

## 范围

### 在范围内

- 新增中英文 `architecture.md`、`concepts.md`、`lifecycles.md`。
- 更新 root README 和中英文文档索引的导航、简化架构图和事实源链接。
- 将新概念入口链接到现有领域章节，但不复制详细 API 说明。
- 把每个核心概念映射到已有、可编译或可测试的 learning example/contract；确有空缺时才新增 example。
- 扩展现有 documentation contract，覆盖 root README、`docs/en`、`docs/zh` 的存在性、本地链接、中英文结构对等和 source-derived 事实。
- 新增 framework 子仓 ADR 0040，固化“概念入口 + 领域章节 + 语义证据”的文档权威和生成/校验边界。
- 更新 `.echo-semantic` 中 framework docs Asset、workspace map、相关边界 map 与 verification Evidence。

### 独立 Finding 前置

以下已有 Finding 与 Issue 必须保持独立 delivery outcome，不在概念文档 Plan 中批量关闭：

- #114 `finding.workspace-topology-doc-drift`：README 补齐 SDK protocol/Host packages 与 11-package 事实。
- #79 `finding.public-feature-table-drift`：README 移除不存在的 `tasks` feature，从 Cargo manifest 校验公共 feature 表。
- #80 `finding.readme-example-target-drift`：README 命令指向真实 Cargo example/test target，documentation contract 覆盖 root README 命令。

这三个 outcome 都会修改 README，应顺序小步交付并各自保留 repair/verification/rereview 证据。其他已知细节文档 Finding（如 #53 Evolution namespace、#67 MCP version、#101 Tool pipeline example）不被本设计偷偷修复；基础文档不复制它们当前有争议的值。

### 非目标

- 不修改 Agent、Task、Subagent、Context、Tool、Permission、MCP、Hook、Plugin、SDK 的 runtime 行为或公共 Rust API。
- 不新增平行 Store、Event bus、Run state machine、Task validator、Session registry 或 schema。
- 不将 92 个 Finding 全部修复，也不将 open 转换为resolved。
- 不修改 `echo-agent-cli` 或 `echo-website`；它们只在 framework 外部合同稳定后再分别同步。
- 不将 EKO 本地个人助理的安全模型写成所有 framework consumer 的默认。
- 不把 ADR、`.echo-semantic` 或顶层阶段计划放到公开 quickstart 中代替用户文档。

## 系统边界

### 通用 framework

`echo-agent/` 内的根 facade、core contracts、execution/state/orchestration/integration/tools/macros、SDK protocol/Host 和 learning consumer 是本设计的事实范围。文档可描述通用原语和已实现默认，不得描述某一 embedding application 的产品策略。

### Embedding application

Product Backend、Runtime Scope 组合、用户账号、本地 Workspace、GUI/TUI/CLI/channel 对等、Desktop/Electron/Tauri 与 Device sync 属于 `echo-agent-cli` 或其它 consumer。Framework 文档只标注接入点和责任边界。

### 适配边界

ACP、A2A、Channels、Headless 和 SDK Host 是 Agent/Turn/Task 向外部协议的投影与适配。该层可翻译 identity、event 和 error，不得重新拥有 framework 内已定义的终态、permission 或 Task graph。Open Finding 对应的现有例外必须以“当前限制”呈现，不得从文档中消失。

## 核心结构与数据流

### 文档事实源

| 事实 | 权威 | 文档处理 |
| --- | --- | --- |
| Package 与 feature topology | Cargo manifests / `cargo metadata` | 表格由清单校验，不允许无来源手写计数 |
| Public Rust surface | root facade + SDK inventory | 细节链接 API docs/inventory，不把所有 identity 复制入手写页 |
| Runtime behavior | source + tests + ADR + resolved Evidence | 只声明已可验证的 behavior/rule |
| Known limitation | open Finding + Issue | 收窄公开承诺，不把 Finding 原文当用户文档 |
| Example | Cargo metadata + learning manifest + contract test | 文档命令必须指向真实 target |
| SDK scope | parity manifest/source contract | 区分 external、Host/Rust-only、intrinsic、helper、deferred |

### 文档发布流

```text
source/manifests/tests
      + resolved semantic Evidence
      + open Finding limitations
          -> bilingual foundational docs
          -> domain chapters and example links
          -> README/index navigation
          -> documentation contract
          -> independent review
```

`.echo-semantic` 是工程治理权威，不是终端用户文档生成器。正式文档由人类可读的概念结构表达，contract 只校验可确定的存在性、对等性、链接和 source-derived 事实。

## 异常与边界场景

1. **中英文结构漂移**：一侧新增/重命名章节而另一侧缺失时，documentation contract 失败；不以文字完全相等作为翻译校验。
2. **源码与概念冲突**：如 Cargo manifest、public API 或 resolved Evidence 与文档不一致，文档不能决定 runtime；必须打开/复用 Finding 并保持门禁失败。
3. **Open Finding 与新文档冲突**：基础文档只能降低承诺或标注当前边界，不能用文案关闭 Finding。
4. **编译功能不存在**：README/feature/example 命令受 Cargo metadata 和实际 target 校验；复制即失败的命令不得发布。
5. **概念同名不同 owner**：AgentFactory、Run、Checkpoint、Revision 等必须附加模块/领域限定；文档不为名字相同而合并 API。
6. **功能受 feature 控制**：概念页可描述能力，但必须链接 authoritative feature table，不声明默认编译已包含所有实现。
7. **跨语言 SDK 不完全对等**：文档必须按 sdk scope 说明，不从 Rust public 可见性推导三语言必须有 facade。
8. **链接目标移动**：本地链接、锚点和 example path 改动必须在同一提交通过 contract，不把 404 留给 website 或 crates.io 发布后发现。
9. **产品边界混入**：出现 EKO-only GUI/TUI/Device/Workspace policy 时停止并路由到 `echo-agent-cli`；不用“可配置”包装后留在 framework。
10. **新问题**：实施中若发现新的独立语义问题，先建唯一 GitHub Issue 和 Finding，再决定是否进入当前 delivery outcome。

## 关键取舍

### 新增导航层，不重写所有功能章节

新的三份基础文档解决概念关系和路由问题，现有 40+ 篇功能章节继续提供 API/config/example 细节。这避免同一行为在 README、概念页和领域章节三处分岔。

### 概念统一不等于状态统一

术语表可以统一读者心智模型，但 Session、Task、Trace、Plugin 和 Delivery 必须保留各自 authority。不建立一个抽象 `LifecycleState` 或全局 `Run`来追求表面一致。

### 正式文档不直接暴露内部 Finding 文本

用户需要的是真实承诺和限制，不是内部审计日志。Open Finding 决定文档能声明的上限，详细 Finding/Issue 仍留在 `.echo-semantic` 与 GitHub。

### Example 优先复用而不增加数量

概念页优先链接已有且进入 test chain 的 demo04、demo30/31、demo34/37/39、demo50/51/53/54/55、demo64/65/66/67 等。只有某个核心数据流没有任何可执行 consumer 时，才新增最小 contract example。

### 文档校验扩展现有 contract

复用 `echo-agent-learning/tests/documentation_contract.rs` 的 Markdown 收集、本地链接解析和 Cargo target 盘点方式，扩展到 framework root。不引入新的文档生成器或第二套 manifest。

## 复用与实现约束

1. **已有实现**：11 张 Capability Map、11 个 Behavior、11 条 Rule、26 个 Asset、ADR 0001-0039、双语领域章节和 learning contracts 已经覆盖内容；新文档只引用和结构化。
2. **标准库**：Rust `std::fs`/`Path` 足以扩展现有 link/parity contract；不需要新的 Markdown parser 依赖。
3. **平台原生能力**：Cargo metadata 是 package/feature/example target 权威，Git 提供 revision，不自建 package graph store。
4. **已安装依赖**：现有 `regex`、documentation contract、facade inventory、SDK scripts 和 semantic verifier 已覆盖机器门禁。
5. **最小自定义实现**：如现有 helper 不足，只在 documentation contract 内增加对称路径/标题和 Cargo target 检查，不新建通用文档框架。
6. **分层结论**：通用 Agent/Turn/Task/Context/Persistence/Tool/Extension/SDK 概念属于 framework；EKO surface、Workspace/Device 策略属于 application；ACP/A2A/Channel/SDK Host 是适配边界。
7. **重复性结论**：仓库已有领域文档与语义图，不新建平行行为定义；三份基础页是跨领域导航与关系层。
8. **术语约束**：只使用 Subagent，不引入第二套执行角色术语；必须使用限定 Revision/Run/Checkpoint 名称。
9. **路径约束**：正式 framework 文档和 ADR 只进 `echo-agent/`；本 design/delivery map/Plan 留在顶层 `lp-agent/docs/supreme/`。
10. **Issue 约束**：每个语义 Finding 唯一映射 GitHub Issue；新问题立即建 Issue；Issue 只在修复进入远程 main 后关闭。

## 预期产物

| 产物 | 归属 | 作用 |
| --- | --- | --- |
| 中英文 Architecture | `echo-agent/docs/en|zh/architecture.md` | package DAG、分层、facade、consumer 与 framework/application boundary |
| 中英文 Core Concepts | `echo-agent/docs/en|zh/concepts.md` | 限定术语、identity 关系和“不是什么” |
| 中英文 Lifecycles | `echo-agent/docs/en|zh/lifecycles.md` | 六条主生命周期、权威、终态、恢复和 cleanup |
| Root/index 导航 | README + `docs/*/README.md` | 从快速上手导向概念和领域细节 |
| ADR 0040 | `echo-agent/docs/adr/` | 固化文档权威、source-derived 事实和重复避免 |
| Documentation contract | 现有 learning test | 验证 root/docs 链接、双语结构、Cargo-derived 事实和 example target |
| Example mapping | 基础文档 + learning manifest | 将概念路由到已编译/测试 consumer |
| Semantic Evidence | `.echo-semantic` | 记录 framework docs Asset、map refs、verification 与当前 open Finding |
| 独立 README repairs | #114/#79/#80 对应 outcome | 先消除 topology、feature 和 target 直接冲突 |

## 验收标准

1. `architecture.md`、`concepts.md`、`lifecycles.md` 在 en/zh 同时存在，章节结构、表格行和 diagram 语义对等，且只有 Subagent 执行角色术语。
2. Architecture 以 Cargo metadata 的 11-package DAG 为事实，准确表达 root facade、SDK protocol/Host 和 learning consumer。
3. Concepts 至少覆盖本设计列出的 14 个核心概念，每个都指定 owner 与非责任；明确没有通用 AgentRevision、全局 Run 或第二套执行角色概念。
4. Lifecycles 至少覆盖 Agent Turn、Context/persistence、Task/Subagent、Tool/Permission/Effect、Observation/Delivery、Extension、SDK 七条流，每条都说明接纳、权威、terminal、cancel/failure、recovery/cleanup 和 projection。
5. Root README 保留已验证的 feature/workspace/example 摘要，Architecture 段收窄为简化概览并链接三份基础页；`docs/en/README.md` 与 `docs/zh/README.md` 提供相同导航顺序。
6. #114、#79、#80 分别在独立 delivery outcome 中拥有 repair、verification、rereview；在进入远程 main 前 Issue 仍保持 open。
7. 新概念页不复制 #53/#67/#101 所指的有争议数值/流程，这些 Finding 状态不因本交付改变。
8. 每个示例链接都指向 Cargo 真实 example 或 test contract；新增/修改示例时必须编译或测试。
9. Documentation contract 会在英文/中文页缺失、本地链接断开、README target 不存在、package/feature 事实漂移时 fail closed。
10. 没有新 public Rust API、runtime state/schema、SDK canonical identity 或三语言 facade 变化；SDK 生成和 contract check 零 diff。
11. `.echo-semantic` strict snapshot、change-evidence、Issue reconciliation 和独立review全部通过；未关闭 Finding 保持 open 且链接唯一 Issue。
12. `cargo fmt --all -- --check`、受影响 documentation tests、facade smoke、examples contract 和适用的 Clippy/check 全绿。
13. 提交说明显式记录：`echo-agent-cli` 与 `echo-website` 未修改，因为本阶段只收敛 framework 正式概念合同，对外同步在合同稳定后单独执行。
