# echo-agent Harness 引擎迭代路线

> **状态**:M0-M4 已完成并提交；下一步 M5 / 4.C EventEnvelope
> **日期**:2026-07-12
> **范围**:`echo-agent` 通用 Agent 框架；涉及 EKO 的装配决策单独注明
> **目标**:强化 echo-agent 作为可靠、可组合、可嵌入、可验证的 Agent Harness，而不是追平某个框架的功能表
> **外部参考**:Claude Code、Codex、DeepAgents、AgentScope。外部实现只用于验证设计共识，不决定 echo-agent 的产品形态

---

## 0. 文档定位

本文件是本轮 echo-agent Harness 引擎迭代的设计与排期依据，负责记录：

1. 当前引擎的真实缺口和已有能力。
2. 主执行路径需要收敛的稳定契约。
3. 每个阶段的框架/应用边界、验收条件与依赖关系。
4. 暂缓项及重新立项条件，防止重复造轮子或过早建设产品壳。

本文件不替代具体 implementation plan。每个 Phase 开始前，必须先重新 grep 整个 `echo-agent` 仓库，确认字段、类型、工具和运行路径是否已经存在，并为该 Phase 单独拆任务清单。

完成一个 Phase 并提交后，必须同步更新 `docs/MASTER-PLAN.md`。跨上下文恢复以 MASTER-PLAN 为总入口，本文件保留该专项的详细设计。

---

## 1. 总体判断

echo-agent 当前不缺横向“大功能”：ReAct、13 阶段工具 pipeline、HITL、subagent/team、上下文压缩、checkpoint、MCP、workflow、trace/eval、evolution 都已经存在。

下一阶段价值最高的工作不是继续增加平行模块，而是收敛主执行路径的契约：

```text
AgentInvocationContext
  -> immutable effective run policy
  -> instruction/context assembly
  -> model call
  -> typed tool execution result
  -> budget/control decision
  -> versioned event stream
  -> checkpoint/finalize
```

### 1.1 跨实现共性

从 Claude Code、Codex、DeepAgents、AgentScope 可提炼出以下共性：

- 指令、工具可见性、权限和执行沙箱是不同策略层，不能混为一个开关。
- plan 是 prompt/artifact/行为约束，不应扩张成复杂运行时审批状态机。
- 每次 invocation 应有明确、稳定、可追踪的运行策略。
- 工具完整结果、模型可见摘要和 UI/trace 元数据应分离。
- 长任务需要稳定事件流、明确终态和可验证的 resume 语义。
- 预算耗尽应改变下一步允许的动作，而不只是记录数字或突然报错。
- 模型差异应通过能力 profile 消化，不能把 provider、模型能力、产品交互模式揉成一个 ModeEngine。

### 1.2 框架与应用边界

| 层 | 应包含 | 不应包含 |
|----|--------|----------|
| `echo-core` | 通用数据契约、trait、纯策略类型 | 临时文件、HTTP 路由、EKO UI 字段、Redis 产品装配 |
| `echo-execution` | 工具执行、artifact/spill 实现、沙箱相关执行能力 | EKO 会话生命周期、GUI 投影 |
| 根 `echo-agent` | ReAct 主循环、run snapshot、策略解析、事件和 checkpoint 接线 | 多租户 SaaS 控制面、EKO 专属模式 |
| `echo-agent-cli` | EKO 默认装配、TUI/GUI/channel 渲染、本地产品策略 | 反向决定框架 API 是否存在 |

拿不准时默认留在应用层。只有已经证明与具体产品无关、存在合理复用方、能够通过 feature 或依赖边界裁剪的能力，才下沉框架。

---

## 2. 已有能力：不重复建设

下列能力已经存在，不作为本轮新模块立项：

| 能力 | 现有锚点 | 本轮处理 |
|------|----------|----------|
| 结构化摘要和增量合并 | `echo-core/src/compression.rs` `StructuredSummary` | 保留 |
| tool_call/tool_result 断链修复 | `echo-state/src/compression/mod.rs` | 加入 resume 回归测试，不重写 |
| Critic/Verifier | `src/agent/react/run/phases/verify.rs`、任务 verifier | 保留 |
| 动态工具面 | skills、`disabled_tools`、readonly registry | 收敛策略叠加，不造第二套 registry |
| 沙箱执行 | Local/Docker/K8s `SandboxExecutor` | 不做 Workspace 大一统 |
| Plan mode | `PlanModeStage` + permission | 不加 plan 审批状态机 |
| Subagent/Team | Sync/Fork/Teammate/Team | 加入事件和恢复契约，不重写编排器 |
| MCP client/server | `echo-integration/src/mcp` | 保持框架能力 |
| Workflow/DAG | `echo-orchestration` | 保持 |
| trace/eval/replay/evolution | `src/trace`、`src/eval`、`src/evolution` | 扩展为 harness 回归基础 |
| RuntimeStateStore/ConversationStore | 现有 state/memory trait | 不新增平行 SessionStore |

### 2.1 明确不做

- 不换成 LangGraph 或把 Python middleware 模型照搬进 Rust pipeline。
- 不新增 `ModeEngine` / `LocalizedModeEngine` 大系统。
- 不用透明 ToolOffload 替换显式后台任务。
- 不默认注册 general-purpose subagent；由消费方装配。
- 不把 EKO InteractionMode 放进框架模型 profile。
- 不因 EKO 不使用 SQLite 而删除 echo-agent 框架对合理复用方提供的 SQLite 实现。

---

## 3. Phase 0：主路径正确性

Phase 0 是当前最高优先级。三个任务按顺序实施，不并行改主循环。

### 3.A P0：统一工具输出与 Artifact/Spill 主路径

#### 现状

主路径 `TruncationStage` 调用 `AgentRunSnapshot::truncate_tool_output`，只做 head/tail 截断。旧 `ReactAgent::truncate_tool_output` 虽包含 spill，但存在两个问题：

1. 它不在 13 阶段 pipeline 的权威路径上。
2. `>2000` 字符的摘要分支位于 `>1MiB` spill 之前，并有必返回的结构化 fallback，因此超大输出在旧路径上也基本到不了 spill。

结果是 shell/test/MCP 等大结果会丢失中间内容，模型无法可靠回读全文；`snapshot` 与 `execution` 还维护两套不同算法。

#### 目标

工具执行结果在内部至少区分三部分：

```rust
ToolOutput {
    model_view: String,
    full_content: Inline | ArtifactRef,
    metadata: ToolOutputMetadata,
}
```

第一期可以保持公开 `ToolResult` 兼容，先建立内部类型或等价结构，不要求一次性改完所有外部 Tool trait。

固定策略顺序：

1. 判断是否需要 artifact/spill；需要时保存完整结果。
2. 生成模型可见 preview，包含稳定的回读指引。
3. 未 spill 但超过 token 预算时，执行 UTF-8 安全 head/tail。
4. LLM summary 只作为显式可选策略，不遮蔽完整 artifact。
5. 写盘失败时降级为安全 truncate，并在 metadata/trace 标记失败原因。

#### 分层

- `ArtifactRef`、纯 metadata 契约：满足复用条件时放 `echo-core`。
- spill、清理、文件落地和 backend 可访问性：`echo-execution` 或根 agent 执行层。
- pipeline：只调用权威策略，不直接实现临时文件细节。
- 不新建与 `ToolResult` 平行的公开 API，除非 implementation plan 证明内部扩展无法覆盖。

#### 必须解决的设计点

- 默认 spill 阈值独立于 `max_tool_output_tokens`，不能因未配置 token 限制而失效。
- working directory 下使用受管 `.echo-agent/spill`，以满足 `read_file` 路径约束；文档明确该目录是运行时 artifact 目录，项目可将其加入 ignore。
- artifact 路径/句柄必须能被当前执行 backend 的回读工具实际访问；不能只验证宿主机文件存在。
- Local/Docker/K8s backend 语义需要分别验证或明确 capability 限制。
- 所有字符 preview/truncate 使用 `.chars().take()` 等 UTF-8 安全实现，禁止字节切片。
- 清理策略不能在热路径无限扫描；失败为 best-effort，不影响当前工具结果返回。

#### 验收

- [x] Pipeline 主路径的大输出产生可回读 artifact。
- [x] `snapshot` 与 `execution` 不再维护两套输出预算算法。
- [x] >1MiB、token 超限、写盘失败、中文和 emoji 均有测试；空输出由既有无预算路径保持原样。
- [x] working-directory artifact 通过真实 `ReadFileTool` 回读测试。
- [x] `ToolResult.truncated` 与 metadata 区分 spilled/truncated/spill_failed_truncated；trace 在预算阶段之后记录正确 truncated 状态。
- [x] 英文/中文 streaming 与 safety 文档说明 artifact 生命周期、默认目录和失败语义。

#### 完成记录

- 提交：`echo-agent 528359b`（`fix(agent): preserve oversized tool outputs as artifacts`）
- 删除：`execution.rs` 中旧 LLM summary/spill/truncate 分叉及辅助函数。
- 权威实现：`AgentRunSnapshot::process_tool_output`。
- 验证：`./scripts/verify-all-crates.sh` 全绿；8 crate 测试分别为 247/0/220/64/91/129/274/448，逐 crate clippy 零警告，独立 feature 矩阵全过。
- 清理：`cargo clean` 释放 8.4GiB。

### 3.B P0：固化 Invocation 级 EffectiveRunPolicy

#### 问题

现有 `AgentRunSnapshot` 已经是正确承载点，但部分运行策略仍通过共享可变状态读取，例如 `disabled_tools: Arc<RwLock<Option<HashSet<_>>>>`。Model profile、plan mode、skill、应用 run 级工具隐藏若直接调用同一个 setter，会出现覆盖和并发 invocation 相互影响。

#### 目标

扩展现有 `AgentRunSnapshot`，在 run 开始时解析不可变的有效策略，不新建第二套 run snapshot：

```text
EffectiveRunPolicy
  working_dir
  visible/disabled tools
  permission policy reference
  iteration/token/tool/time budgets
  model capability profile
  resolved project instructions
  response format
```

有效工具隐藏集合至少遵循：

```text
harness/model exclusions
  union agent config exclusions
  union plan/skill restrictions
  union invocation exclusions
```

#### 动态例外

以下状态可以在运行中变化，但必须有明确通道和 trace：

- cancellation
- steer mailbox
- HITL decision/resume
- 工具调用产生的新消息

工具面、working directory、模型能力和项目规则默认不应在同一 invocation 中悄悄变化。

#### 验收

- [x] invocation 工具策略按值冻结，两个 invocation 不再通过共享 setter 覆盖彼此。
- [x] agent/harness exclusion、skill allowlist、plan mode、应用 invocation exclusion 的组合有矩阵测试；当前框架没有独立 model exclusion 通道，模型装配产生的默认排除归入 agent/harness exclusion。
- [x] snapshot 创建后修改 agent 级配置不影响当前 run。
- [x] LLM 可见工具面与执行 pipeline 使用同一冻结策略，隐藏工具即使被 provider 强行返回也无法执行。
- [x] 不新增全局锁或第二个工具 registry。

#### 完成记录

- 框架提交：`echo-agent 2266d0f`（`fix(agent): freeze invocation tool policy`）。
- 应用提交：`echo-agent-cli 664e80a`（`fix(chat): scope tool exclusions to invocation`）。
- 框架边界：`AgentInvocationContext.disabled_tools` 是通用 invocation exclusion；`ToolRuntime` 在 snapshot 创建时合并 agent 默认与 invocation 排除，并统一过滤 skill/plan/tool exclusions。
- 应用边界：Chat/Task/Auto 的产品工具面仍由 EKO 决定，Chat exclusion 只通过 invocation 传值，不再修改 pooled agent。
- 回归：同一个 pooled agent 连续执行 Chat 和 Auto，Chat 中 `create_complex_task` 被阻止，Auto 中可执行，证明策略不泄漏。
- 验证：框架 `./scripts/verify-all-crates.sh` 全绿，8 crate 测试分别为 247/0/220/64/91/129/274/451；CLI workspace 480+4+41+9 测试、GUI 41 测试、all-features clippy 与 channels/tui/eval/improve/gui-devtools feature 组合全过。
- 清理：框架 `cargo clean` 释放 8.1GiB，CLI `cargo clean` 释放 36.8GiB。

### 3.C P0/P1：统一 RunBudgetPolicy

#### 纠偏

AgentScope `ReplyBudgetControlMiddleware` 统计的是 reply 内加权 input/output token；达到阈值后注入收束提示并强制 `tool_choice=none`。它不是“接近 max_iterations 时提醒”。

echo-agent 需要同时支持两类不同能力：

- `IterationWindDown`：接近迭代上限时停止开新分支并收束。
- `ReplyTokenBudget`：累计模型 token 达到预算后禁止继续使用工具并要求 final。

两者共享控制器，但不混用名称。

#### 目标模型

```text
RunBudgetPolicy
  max_iterations
  max_model_tokens
  max_tool_calls
  max_wall_time

BudgetDecision
  Continue
  WindDown
  FinalOnly
  HardStop
```

第一期最小范围：

1. 保留现有 `max_iterations` 硬停止义。
2. 增加 iteration wind-down，一次性注入短提示。
3. 累计 provider 返回的 input/output token；无 usage 时记录 unknown，不伪造精确值。
4. `FinalOnly` 时禁止新工具调用；优先使用 provider/tool-choice 能力，能力不支持时通过工具面为空和 prompt 双重约束。
5. budget 决策进入 trace/event，便于 GUI/TUI/评测观察。

#### 默认值

- 新的 wind-down 默认关闭或保持行为兼容；先由消费方显式开启。
- 不默认用 `soft_remaining=2` 改变所有现有 agent 行为。
- `max_iterations == 0` 继续表示无限迭代，不触发 iteration wind-down。

#### 验收

- [x] iteration wind-down 与 token budget 分别有独立测试和配置名。
- [x] WindDown 只触发一次。
- [x] FinalOnly 后工具不可调用；请求同时隐藏工具并发送 `tool_choice=none`。
- [x] 模型正常 final 时成功结束；仍耗尽 `max_iterations` 时沿用明确失败终态。
- [x] HITL pause/resume 继续使用同一 invocation-local loop state，预算跨 await 不重置并有回归测试。
- [x] provider 未返回 usage 时保持 unknown，不使用估算 token 触发预算。

#### 第一期完成记录

- 框架提交：`echo-agent bd8c8f4`（`feat(agent): add invocation run budgets`）。
- 应用提交：`echo-agent-cli 665713e`（`feat(trace): display run budget decisions`）。
- 通用契约：`RunBudgetPolicy` 支持 `iteration_wind_down_remaining` 与 `max_model_tokens`，既可作为 agent 默认值，也可由 `AgentInvocationContext` 覆盖；snapshot 创建时冻结。
- 控制路径：预算计数只存在于单一 `run_core_loop` 的 `LoopState`；wind-down 注入一次提示，token 阈值进入 `FinalOnly`，不新增运行状态机。
- 可观测性：`AgentEvent::BudgetDecision` 与 `RunEvent::BudgetDecision` 同时记录 decision、reason、iteration、reported tokens 和 usage 完整性；CLI `/trace` 可查看。
- 回归：覆盖一次性 wind-down、token 阈值阻止 provider 强制工具调用、下一请求 `tool_choice=none`、usage 缺失不误触发、invocation 覆盖冻结、异步 pause/resume 计数保持。
- 验证：框架 `verify-all-crates.sh` 全绿，8 crate 测试分别为 247/0/220/64/91/129/274/456；CLI workspace 480+4+41+9、GUI 41 测试、all-features clippy 与有效 feature 组合全过。
- 清理：框架 `cargo clean` 释放 10.0GiB，CLI `cargo clean` 释放 27.1GiB。
- 范围说明：目标模型中的 `max_tool_calls`、`max_wall_time` 与 `HardStop` 仍是后续扩展，不属于本节明确列出的第一期最小范围；有真实消费方需求后再补，不用未落地字段污染当前 API。

---

## 4. Phase 1：Harness 组装契约

Phase 1 依赖 Phase 0 的 immutable run policy，避免在共享状态上叠加更多临时开关。

### 4.A P1：ModelCapabilityProfile

#### 目标

用模型能力 profile 表达“agent 应如何运行”，而不是只用模型名追加 prompt：

```text
ModelCapabilityProfile
  provider/model selector
  supports_parallel_tools
  supports_tool_choice_none
  supports_structured_output
  context_window
  reasoning mode/capabilities
  excluded_tools
  prompt suffix
```

#### 边界

- `ProviderAdapter` 继续处理协议、请求构造、thinking/cache 等 provider 细节。
- `ModelCapabilityProfile` 处理模型能力对 harness 行为的影响。
- EKO Chat/Task/Auto 不进入该 profile。
- 工具排除通过 EffectiveRunPolicy 计算，不 unregister 共享 registry。
- selector 使用规范化 `provider:model`，不以模糊 `gpt-*` 作为唯一匹配依据。

#### 首期范围

- API + resolver + 1 至 2 个 example。
- 不内置大规模、易过期的模型表。
- 消费方可注册覆盖项。
- 文档明确 DeepAgents HarnessProfile 仍是 beta 参考，不复制 middleware 替换机制。

#### 验收

- [x] provider:model 精确项优先于 provider 默认项。
- [x] profile exclusions 与 invocation restrictions 统一并入冻结工具策略。
- [x] prompt suffix 进入 canonical system context，压缩后仍保留。
- [x] capability 不支持 `tool_choice=none` 时，预算 FinalOnly 使用空工具面 + prompt 回退。

#### 完成记录

- 提交：`echo-agent 339fad4`（`feat(llm): resolve harness model profiles`）。
- 纠偏：仓库已有 `ProviderCapabilities + ModelProfile`，因此没有新建平行的 `ModelCapabilityProfile`。
- 新增：`ModelProfileResolver` 与 `ModelProfileOverride`，provider 默认项先应用，规范化 `provider:model` 精确项后应用。
- Harness 接入：profile exclusions 合并进 EffectiveRunPolicy；prompt suffix 进入 canonical system prompt；FinalOnly 根据 capability 决定显式 `tool_choice=none` 或空工具面回退。
- 文档：双语 factory 文档删除不存在的 ModeEngine/LocalizedModeEngine/AgentMode API，只保留真实 Factory、ModelProfile、PromptTemplate 与应用层装配边界；净删除 493 行。
- 验证：8 crate 测试分别为 249/0/220/64/91/129/274/459，逐 crate clippy 与 feature 矩阵全绿；EKO workspace 与 GUI target 编译通过。
- 清理：框架/CLI `cargo clean` 分别释放 8.8GiB/3.7GiB。

### 4.B P1：InstructionResolver

#### 问题

现有 `load_project_rules` 只识别向上目录中的 `.echo-agent/AGENT.md|RULES.md|rules.md`。直接追加 `AGENTS.md` 和 `CLAUDE.md` 会显著增加误加载父工作区或 home 目录文件的概率，不能视为“极低风险”。

#### 目标

```text
ResolvedInstructions
  content
  sources
  project_root
  precedence
```

第一期策略：

1. 显式 project root 优先；否则寻找最近 Git/worktree root。
2. 在 project root 范围内解析规则，不默认越过项目边界扫描到文件系统根。
3. 支持框架原生 `.echo-agent/AGENT.md|RULES.md|rules.md`。
4. 兼容 `AGENTS.md` 和 `CLAUDE.md`。
5. 记录每个来源及优先级，注入 trace，便于解释“规则从哪里来”。
6. 第一阶段不做每轮热重载。

层级文件是覆盖还是合并，必须在 implementation plan 中先对照 Claude Code/Codex 的当前官方语义，再冻结规则；不能让 `load_project_rules -> Option<(PathBuf, String)>` 的旧返回值限制未来能力。

#### 验收

- [x] Git root/worktree root 边界测试。
- [x] 不加载项目边界之外的常见 `AGENTS.md`/`CLAUDE.md`。
- [x] `.echo-agent` 原生规则与兼容规则的优先级有测试。
- [x] symlink、无 Git 仓库、空文件、非法 UTF-8/读取失败均安全降级。
- [x] `ResolvedInstructions.sources` 显示来源、类型和 precedence，annotated canonical block 保留路径诊断。

#### 完成记录

- 提交：`echo-agent 7b4dd0b`（`feat(context): resolve bounded project instructions`）。
- 官方依据：Codex 从 project root 到 cwd 逐级扫描、每层最多一份、根到叶合并、空文件跳过；近层规则后置覆盖。Claude 官方网页因检索工具解码失败未取得，因此 `CLAUDE.md` 仅作为兼容 fallback，不复制未核实的 Claude memory 语义。
- 新契约：`InstructionResolver -> ResolvedInstructions { content, sources, project_root }`；`InstructionSource` 记录 path/kind/precedence。
- 边界：显式 `project_root` 优先，否则最近 `.git` 目录或 worktree `.git` 文件；无 root 时只扫描 cwd。解析后指向 root 外部的 symlink 被跳过。
- 优先级：每层只取一个文件，`.echo-agent/AGENT.md|RULES.md|rules.md` 优先，其次 `AGENTS.override.md|AGENTS.md|CLAUDE.md`；多层按 root 到 cwd 合并。
- 接入：`AgentConfig/ReactAgentBuilder::project_root` 控制 canonical system context 的发现边界；旧 `load_project_rules/rules_injection/inject_rules` 保留为框架便利 API 并委托新 resolver。
- 验证：8 crate 测试分别为 255/0/220/64/91/129/274/459，逐 crate clippy、feature 矩阵、`project-rules` 398 测试和 75 doctests 全绿；EKO workspace/GUI 编译通过。
- 清理：echo-core、框架、CLI `cargo clean` 分别释放 389.1MiB、11.1GiB、3.0GiB。

### 4.C P1：Versioned EventEnvelope

#### 问题

`AgentEvent` 已覆盖大量流式行为，但 GUI/TUI/CLI/channel、未来 NDJSON/SSE、checkpoint resume 都需要稳定的身份、顺序和终态契约。继续直接扩充 payload enum 会让 adapter 各自补字段并产生漂移。

#### 目标

在不复制 EKO UI projection 的前提下，为框架事件增加统一 envelope：

```text
EventEnvelope
  schema_version
  event_id
  sequence
  conversation_id
  run_id
  turn_id
  execution_id
  parent_event_id
  timestamp
  payload: AgentEvent
```

#### 不变量

- 同一 execution 内 `sequence` 单调递增。
- terminal event 恰好一次。
- tool start/result、subagent parent/child、HITL deferred/resume 可以关联。
- `run_id` 继续只在正式 task run 存在；普通 chat 不伪造 run。
- resume 不重复发射已经持久化的副作用完成事件。
- NDJSON、SSE、GUI bridge 只是 adapter，不重新定义服务事件。

#### 分层

- 通用 envelope 和 payload 契约：框架。
- GUI/TUI 渲染字段、前端 store projection：echo-agent-cli。
- HTTP/SSE 路由和 replay retention：消费方或未来可选 service，不在本 Phase 建设。

#### 验收

- [ ] 正常成功、模型失败、工具失败、取消、HITL、max budget 均恰好一个 terminal event。
- [ ] sequence/identity 在主 agent、subagent、background task 中一致。
- [ ] 旧 `AgentEvent` 消费方有明确迁移方式；项目不需要长期双协议兼容，迁移完成后删除旧分叉。
- [ ] TUI/GUI/CLI/channel 能投影同一框架事件，不以“某模式不需要”为由缺失能力。

### 4.D P1：清理 ModeEngine 文档漂移

`docs/en/38-factory-modes.md` 和 `docs/zh/38-factory-modes.md` 中不存在的 `ModeEngine` / `LocalizedModeEngine` API 必须删除或明确标记历史设计。

文档应只描述真实存在的：

- AgentFactory / PromptTemplate
- Plan mode
- EffectiveRunPolicy
- ModelCapabilityProfile
- 消费方装配模式

验收：文档示例可编译，`rg "ModeEngine|LocalizedModeEngine" echo-agent/docs` 仅允许出现在历史说明中。

---

## 5. Phase 2：恢复与验证

### 5.A P1/P2：Resume Conformance Suite

优先加强既有 RuntimeStateStore/checkpoint 的正确性，不新增平行 SessionStore。

#### 故障注入点

- prepare 前后
- compact 前后
- model stream 中断
- tool start 后、result 持久化前后
- verifier 前后
- HITL deferred/resume
- finalize hook/checkpoint
- channel receiver 被关闭
- cancel 与 background completion 竞争

#### 核心不变量

- 恢复后消息仍满足 tool_call/tool_result pairing。
- 已确认完成的危险/写工具不能因 resume 被重复执行。
- Deferred 恢复到同一 conversation/turn/run/execution。
- terminal hook、checkpoint、conversation projection 不重复执行。
- Running/Failed/Success checkpoint 节点的重跑规则明确。
- 没有 state store 时行为与当前内存模式一致。

#### 验收

- [ ] 每个阶段至少一个中断/恢复测试。
- [ ] 所有测试使用 scripted/mock LLM，不依赖真实模型。
- [ ] trace 能指出从哪个 checkpoint 恢复以及跳过了哪些已完成动作。

### 5.B P2：Harness Trajectory Regression Suite

在现有 `src/eval` / `TrajectoryReplay` 上增加模型无关场景，断言最终结果之外的完整轨迹：

- 大工具输出产生 artifact 并回读。
- 无限工具循环被 detector/budget 收束。
- token budget 进入 FinalOnly。
- verifier 失败后自修正。
- compression 后继续合法工具调用。
- HITL 暂停和恢复。
- subagent 失败传播和 parent/child 事件关联。
- steer 中途改变目标。
- UTF-8/中文/emoji 全路径。
- feature 组合下核心事件轨迹一致。

建议形成稳定的 harness contract fixtures：输入脚本、模型响应脚本、预期事件序列、checkpoint 不变量和最终结果。

---

## 6. 暂缓：Hosted Agent Service

托管服务不是错误方向，但当前没有足够证据把以下产品级能力直接承诺为 echo-agent Phase 2：

- 新 `SessionStore`
- `TenantId` 平行身份模型
- Redis 实现
- `WakeupDispatcher`
- 固定 `/sessions/{id}/chat` HTTP API
- SSE replay retention

### 6.1 暂缓理由

1. 现有框架已有 `session_id`、`conversation_id`、`ConversationStore`、`RuntimeStateStore`、A2A、channels、HITL provider；必须先证明缺的是通用原语而不是组装方式。
2. Session 生命周期、tenant isolation、HTTP 路由和 wakeup worker 带有明显产品部署决策。
3. 当前只有 AgentScope 一条主要参考，尚不足以证明跨框架共识。
4. EventEnvelope 和 resume contract 尚未稳定，先建服务层会把不稳定内部事件固化成外部 API。

### 6.2 重新立项条件

满足以下条件后再开独立 G0 设计评审：

- 至少一个 EKO 之外的真实消费场景。
- 明确现有 ConversationStore/RuntimeStateStore 无法覆盖的通用需求。
- EventEnvelope 与 resume conformance 已完成。
- 对 Claude/Codex/AgentScope 之外至少一个成熟托管 agent 实现完成调研。
- 能清楚证明 MessageBus/Wakeup 放框架优于放消费方应用。

若届时只证明 `MessageBus` 是通用原语，可仅提取 trait + InMemory 实现；HTTP/Redis 仍可留外部 crate 或参考应用。

---

## 7. 分阶段路线与门禁

```text
Phase 0：主路径正确性
  3.A Tool output artifact/spill
    -> 3.B EffectiveRunPolicy
      -> 3.C RunBudgetPolicy

Phase 1：Harness 组装契约
  4.A ModelCapabilityProfile
  4.B InstructionResolver
  4.C EventEnvelope
  4.D ModeEngine 文档清理

Phase 2：恢复与验证
  5.A Resume Conformance
  5.B Trajectory Regression

Future：真实消费方驱动
  Hosted service / MessageBus / Redis
```

### 7.1 建议拆分

| Milestone | 范围 | 风险 | 预计 |
|-----------|------|------|------|
| M0 | 3.A 工具输出权威路径 | 中 | 2–4 天 |
| M1 | 3.B EffectiveRunPolicy | 高，涉及 run 主路径 | 2–4 天 |
| M2 | 3.C 预算策略 | 中高 | 2–4 天 |
| M3 | 4.A + 4.D profile 与文档 | 中 | 2–3 天 |
| M4 | 4.B InstructionResolver | 中 | 1–2 天 |
| M5 | 4.C EventEnvelope | 高，跨消费方 | 3–5 天 |
| M6 | 5.A/5.B conformance 与回归 | 中高 | 1–2 周 |

M1、M5 属于核心数据流高风险步骤，必须在新鲜上下文中执行，并先写独立 implementation plan。

### 7.2 每个 Milestone 的强制流程

1. Grep 整个 echo-agent 仓库，确认现有定义和活路径。
2. 写独立 implementation plan，列出删除的旧分叉。
3. 实施并补充与风险相称的测试。
4. 执行全量验证，而不是只跑相关 crate：

```bash
cd echo-agent
cargo fmt --all
cargo fmt --all -- --check
./scripts/verify-all-crates.sh
cargo clean
```

5. 检查所有 feature 组合、Cargo 路径和 `.worktrees/` 规则。
6. 使用 `git -c commit.gpgsign=false commit` 提交。
7. 更新 `docs/MASTER-PLAN.md` 的状态、决策、提交号和下一步。

---

## 8. 成功标准

本轮完成后，echo-agent 应满足：

1. 任意工具大输出都有确定的完整结果保存和模型回读路径，不再静默丢中间内容。
2. 每次 invocation 的有效工具面、规则、模型能力和预算在 run 开始时可解释、可测试、并发隔离。
3. iteration、token、工具数和时间预算可以统一决策 Continue/WindDown/FinalOnly/HardStop。
4. 模型差异通过能力 profile 处理，不需要虚构 ModeEngine 或把 EKO 模式塞进框架。
5. 项目规则有明确 root、优先级和来源，不会从无关祖先目录意外加载常见规则文件。
6. 所有交互模式消费同一版本化事件契约，身份、顺序、parent-child 和终态稳定。
7. 任一主循环阶段中断后，恢复行为可通过 conformance suite 验证，写工具不会无意重复执行。
8. hosted service 保持消费方驱动，不因单一对标实现而提前污染核心框架。

---

## 9. 关键源码锚点

### echo-agent

- `src/agent/react/run/pipeline.rs`：权威工具 pipeline
- `src/agent/snapshot.rs`：AgentRunSnapshot 与主路径 truncate
- `src/agent/react/run/execution.rs`：旧 summary/spill/truncate 分叉
- `src/agent/react/run/stream_channel.rs`：ReAct 循环、iteration 与 finalize
- `src/agent/react/run/phases/finalize.rs`：硬停止与 terminal 行为
- `src/agent/react/subsystems/tool_exec.rs`：共享 `disabled_tools`
- `echo-core/src/agent/mod.rs`：AgentInvocationContext / AgentEvent
- `echo-core/src/tools/mod.rs`：ExternalRunContext / ToolContext / ToolResult
- `echo-core/src/project_rules.rs`：当前规则发现
- `echo-core/src/compression.rs`：CanonicalContext / StructuredSummary
- `echo-state/src/compression/mod.rs`：tool pairing 与 canonical 重注入
- `src/trace`、`src/eval`：事件回放与回归基础
- `docs/en/38-factory-modes.md`、`docs/zh/38-factory-modes.md`：ModeEngine 漂移

### 外部参考

- AgentScope `middleware/_budget.py`：token reply budget + final-only
- AgentScope `app/`：托管服务是参考，不直接推导框架 API
- DeepAgents `profiles/harness/harness_profiles.py`：provider construction 与 harness behavior 分层
- Codex `exec --json`：状态变化事件流和 resume 思路
- Claude Code / Codex：持久项目规则、工具面与权限策略分离

---

## 10. 当前执行入口

M0-M5 已完成。M5 / 4.C 的落地结果：

1. `echo-core` 新增可序列化 `EventEnvelope/EventIdentity`，固定 schema version、conversation/run/turn/execution、parent、timestamp 和 payload。
2. sequence 在包装流内单调递增；缺失终态和底层 stream error 统一收敛为一个 terminal Error，重复终态在第一个终态后截断。
3. tool result/progress 通过 `parent_event_id` 关联对应 ToolCall；普通 chat 保持 `run_id=None`。
4. event_id 由 schema + identity + sequence 确定性生成；`envelope_event_stream_after` 可从已持久化 sequence 继续，为 resume 幂等去重提供通用原语。
5. GUI、TUI、CLI REPL、IM channel、正式/无人值守 TaskRuntime、framework subagent 和 A2A adapter 均消费同一 envelope；UI projection 仍留在应用层。

M0-M6 已完成。M6 / 5.A + 5.B 的落地结果：

1. `AgentCheckpoint::restore_messages` 在恢复前校验工具调用 ID、名称、唯一配对和完整性，损坏或半写入历史不会进入模型上下文。
2. 工具批次在全部结果进入消息历史后的第一个安全边界立即保存 checkpoint；恢复 trace 记录 checkpoint 时间和已完成 tool call IDs，为写副作用去重提供可审计依据。
3. `validate_event_trajectory` 对 schema、连续 sequence、稳定 identity、parent-child、tool lifecycle 和 terminal exactly-once 做模型无关验证。
4. `TrajectoryReplay::contract_violations` 将工具、phase iteration 和 subagent outcome 的生命周期约束纳入 eval；canonical fixture 覆盖 success、model error、cancel/HITL notice、budget FinalOnly、subagent 和 UTF-8。
5. 沿用既有 `RuntimeStateStore`，没有新增 SessionStore；通用恢复/轨迹契约留在框架，EKO 的审批策略和 UI projection 留在应用层。

专项路线至此完成。Hosted Agent Service 仍属于 Future，只有出现真实消费方需求时才重新立项。
