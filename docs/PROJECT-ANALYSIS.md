# EKO 项目深度分析报告

> **历史快照**：本文固定描述 2026-07-01 的源码，不是当前架构、实现计划或任务入口。
> 当前权威只在 [`MASTER-PLAN.md`](MASTER-PLAN.md) 与仓库根 `AGENTS.md`；文中的路径、类型和状态不得直接作为新实现依据。
>
> 三仓库:`echo-agent`(通用框架)+ `echo-agent-cli`(EKO 应用,含 Tauri 桌面壳 + React 前端)+ `echo-website`(官网,与运行时无关)。定位:**本地个人超级智能助理**,无多用户/公网场景——这是所有安全/架构取舍的出发点。
>
> 本报告基于 2026-07-01 源码核实,所有结论附 file:line 引用。生成方式:并行派发 10 个窄主题 subagent 各聚焦一个源码范围,证据取自源码而非文档转述。

---

## 维度 1 · 交互与入口

### 1.1 四模式入口统一驱动(核心架构亮点)

EKO 的关键设计是 **`drive_chat` 单一驱动点**——GUI / TUI / IM channel 三种交互模式共用同一个驱动函数,差异只在 `ChatSink`(渲染适配器)实现。

| 模式 | 入口 | ChatSink 实现 | 调用 drive_chat 处 |
|---|---|---|---|
| **GUI(Tauri)** | `src-tauri/main.rs` → `tauri/desktop.rs:120 run_desktop` | `TauriChatSink`(`tauri/commands/chat.rs:781`)经 Tauri event 发前端 | `chat.rs:599` |
| **TUI(ratatui)** | `src/main.rs:50` → `tui/mod.rs:1117 run_tui` | `TuiChatSink`(`tui/events.rs:806`)映射到本地 event 喂 mpsc | `tui/events.rs:888`(send_to_agent) |
| **IM channel** | `cli/modes.rs:58 run_channels_mode` | `ChannelChatSink`(`chat_driver.rs:120`)转发原始 event | `cli/channels.rs:156` |
| **cron** | `state.rs:597 start_scheduler_with_store` | (无人值守,无 Sink) | `executor.rs:2162 launch_cron_run` |

- **`drive_chat`**(`chat_driver.rs:60`)签名:`drive_chat(agent, message, multimodal, res: Arc<ChatResources>)`。它把 `res.mode_hint` 包进 agent,在 `with_chat_resources`(`:86`)作用域内把 `Arc<ChatResources>` scope 进 task_local `CURRENT_CHAT_RESOURCES`(`chat_resources.rs:50`)。
- **`ChatSink` trait**(`:31`):核心方法仅 `on_agent_event(event) -> bool`(返 false 中断),其余运行状态/Subagent trace/trace sink 回调可选。三模式各实现一个 Sink 即完成渲染层解耦。
- **`ChatResources`**(`chat_resources.rs:23`):共享上下文载体,含 `pool`/`store`/`conv_id`/`cancel: CancellationToken`/`mode_hint`/`layer_manager: MemoryLayerManager`/`attachments`。运行时 agent 工具(如 `create_complex_task`)经同步函数 `current_chat_resources()` 读 task_local,无需经工具参数传递——这是"控制权移交大模型"的关键管线。
- **TUI/GUI 功能对等**:两者是功能完全相同的 Agent 完全体(对标 Claude Code),禁止以"某模式不需要"为由拒绝接入。

### 1.2 cron 调度(独立子系统)

cron **不走 drive_chat**(无人交互),走 `launch_cron_run` 自主 run:

```
tick(30s) → fire_task → build_fire_fn 闭包
  → pool.acquire("__cron__:{task_id}:{fire_id}")   # per-run 独占 agent
  → launch_cron_run(source="cron", route=ParallelReadonlyDelegation, WriteMode=Worktree)
  → run 结束 → pool.release
```

- **`SchedulerRunner`**(`echo-orchestration/scheduler/runner.rs:52`)`tokio::select!` + `sleep(30s)`,取 `[now-30s, now]` 窗口,对 `next_run` 落窗且 `Enabled` 的任务 fire,用 `last_fired` map 防重发。
- **pool-per-run 隔离**:key 前缀 `__cron__:{task}:{fire}` 让每次 fire 获唯一 agent,绝不复用;worktree working_dir 绑 per-run,避免并发覆盖。

### 1.3 Dreaming 定时(独立,勿与 cron 混)

`infra.rs:669 spawn_dreaming_task` 用 `tokio::interval(86400s)` 每日跑离线反思,**仅桌面长驻模式**;CLI/TUI/channels 靠 session-end review。

---

## 维度 2 · 执行与调度

### 2.1 Subagent 三种模式(Sync/Fork/Teammate)

`ExecutionMode` enum(`subagent/types.rs:15`)由 `SubagentExecutor.dispatch`(`executor.rs:195`)路由:

| 维度 | Sync | Fork | Teammate |
|---|---|---|---|
| 语义 | 委托/阻塞 | 分叉独立 | 并行协作 |
| 执行 | 内联 await,无 spawn | `tokio::spawn` + semaphore 限流(默认 5) | `tokio::spawn` 返回 `TeammateHandle` |
| 上下文 | 部分(enhance_task) | 完整继承父上下文 | 独立,经 Mailbox 通信 |
| 隔离 | 共享父 | 独立 timeout/cancel | 独立 task + Mailbox |
| 返回 | 直接 Result | join Result | `TeammateHandle` → join |

- **`AgentDispatchTool`**(`tools/builtin/agent_dispatch.rs:50`,tool 名 `agent_tool`):主 agent 经此工具调起 subagent,参数 `mode` ∈ {sync/fork/teammate} 映射 `ExecutionMode`(`:197-202`),cancel token 经 `child_token()` 派生实现级联取消。
- **Teammate = Team 编排**:`subagent/team/` 目录,历史 `TeamRole`(Leader/Subagent/Reviewer)、`Mailbox`、`TeamConfig.cross_talk` 控制队友互通；这些名称只描述当时快照，当前实现须重新从源码核验。
- **Fork 默认继承历史 2 条**(`enhance_task` 的 `inherit_history`,Sprint 6b 修复——此前设了不生效)。

### 2.2 任务生成与 DAG 派发(应用层核心)

```
PlanTask 列表(planner 产出)
  └─ run_dag()  executor.rs:435        ← DAG 调度核心
       ├─ by_id 索引 + completed/failed_set 状态机
       ├─ skip-completed-on-retry(executor.rs:456,重跑预填 completed)
       ├─ 按 depends_on 拓扑分 wave,tokio::spawn 并发派发 ready
       │     Subagent/write/shell/llm 并发许可限流
       └─ 历史任务 Subagent adapter → 三路路由:
            • read-only         → 只读 Subagent
            • Implementation/Debugging → writer Subagent(Fork + worktree)
            • Verification      → run_main_agent_task(主 agent 原地)
```

- **DAG 调度**:依赖全完成才 ready;上游失败则下游 `Blocked`;全死 `Failed`,部分死 `Paused`(等决策);stall 检测 ready 空且未全完成 → `Failed: DAG stalled`。
- **review 修复重排**:review 产出 `tasks_with_fixes`,下 wave 优先用 fix 变体。
- **writer 失败回退**:writer 派发失败 → `warn` 后回退 `run_main_agent_task`,保证 DAG 不中断。

### 2.3 读写并行控制(当前串行,地基已就位)

- **写信号量** `ConcurrencyLimits.max_concurrent_writes`(默认 **1**)→ writer 全局串行(`executor.rs:471`)。
- **未来防护** `file_write_locks: HashSet<String>`(`:478`)跟踪正在写的文件,为放开信号量后的文件级互斥做准备(当前仅 warn 不死锁)。
- **plan-time 文件所有权分析器** `analyze_file_ownership`(`planner.rs:168`):O(n²) all-pairs 求 writer 任务文件交集,输出 `FileOverlapPair{task_a,task_b,shared}`,**纯告警非阻塞**(write_sem 已全局串行)。对标 Cursor "file-level ownership is the foundation"。
- **worktree 隔离**(Sprint 8/9):Fork writer 经 `WorktreeFactory` 创建 `eko-fork-<label>` git worktree + `set_working_dir` 把 Subagent 约束在隔离目录,writer 不落主区。
- **数据/科研 Subagent tmpdir**(Sprint 10):`DataWorkspaceFactory`(无 git 耦合),各 Subagent 跑独立 tmpdir 产出不相交文件。

### 2.4 LLM 网关与路由(静态配置,无动态路由)

- **trait 两层**:核心 `LlmClient`(`echo-core/src/llm/mod.rs:53`,`chat`/`chat_stream`/`model_name`/`capabilities`,流为 `'static` 支持 trait 对象);Provider 差异 `ProviderAdapter`(`traits.rs:22`,区分 thinking 协议)。
- **路由 = 静态配置 + 工厂**:`LlmProvider` 枚举仅 `OpenAi`(折叠 deepseek/dashscope/qwen/zhipu/moonshot 等 OpenAI 兼容)/`Anthropic` 两值。`LlmConfig::build_client()`(`config.rs:302`)按 YAML `provider` 字段静态 match。**无运行时按模型名/任务类型的动态 router**。
- 实际配置:`echo-agent.yaml` 默认 `qwen3.6-plus` → `provider: dashscope`。Agent 启动一次性构造 `Arc<dyn LlmClient>`,运行时不切换 provider。

### 2.5 代码沙箱(三层架构 + RCE 防护)

`echo-execution/src/sandbox/`,统一 trait `SandboxExecutor`,`IsolationLevel = None/Process/OsSandbox/Container/Orchestrated`:

- **`SandboxManager`**(`manager.rs:21`)持 `local`(AlwaysSandbox)/`docker`/`k8s` 三层 + `SandboxPolicy`,`auto_detect()` 探测可用层。`execute()` → `policy.evaluate()` 算最低隔离级 → `select_executor()` 选满足要求的最轻量层,不满足可降级或 `SandboxError`。
- **RCE 防护**(`SandboxPolicy` `policy.rs:26`):正则 `DANGEROUS_PATTERNS`(`curl/wget/nc/eval/exec/rm -rf/dd/| bash/$()`)强制升级 Container;只读命令→None,解释器→OsSandbox;默认 `SecurityLevel::Strict`(=Docker)。`has_container_sandbox()` 标注无容器时退化为宿主直跑的风险面。`SENSITIVE_MOUNT_PATHS` 禁挂 `/etc /proc /sys`。
- **working_dir 绑定**:`SandboxCommand::with_working_dir`,Local 用 `current_dir`,Docker 用 `-w`。Hook 命令经 `with_sandbox_manager` 注入,执行时绑 `source_dir`。

### 2.6 人机协同(HITL / Approval Gate)

`echo-core/src/tools/permission.rs` + `echo-orchestration/src/human_loop/`:

- **`PermissionMode`(8 值,permission.rs:59)**:`Default`(危险操作确认,默认)/`Plan`(只读)/`AcceptEdits`/`BypassPermissions`(可被 admin 关)/`Auto`(AI 分类器)/`Bubble`(子 agent 上抛)/`DontAsk`(CI/无人值守)/`StrictConfirm`。
- **决策链**(`PermissionService::check` service.rs:~490):受保护路径(.git/.ssh/.env)无条件 Deny → Bypass → Plan → `RuleRegistry`(deny-first:deny>ask>allow)→ approval_cache → DenialTracker(连续拒绝过多升级)→ 模式分发。需确认时 `PermissionRequestHandler::handle()`。
- **审批影响**:`AllowForSession`/`AllowAlways` 写规则后续直接放行;`Deny` 记 DenialTracker。
- **`HumanGate` 已删**:生产代码无残留,被统一 `HumanLoopProvider`(Selection/approval kind)取代。**注意:AGENTS.md 强调这些 approval gate 只管 agent 自动决策路径,不管用户交互式工具(终端/文件选择器)——后者不该被卡。**

---

## 维度 3 · 存储与上下文

### 3.1 记忆系统(分层,CLI 用 FileStore,不用 SQLite)

trait/类型在 **echo-core**,实现在 **echo-state**,应用层在 **app-core**。

| 层 | 实现 | CLI 后端 | 路径/命名空间 |
|---|---|---|---|
| **Hot** | MEMORY.md(YAML frontmatter + md body) | 直接文件 | `.echo-agent/MEMORY.md`,常驻 context(预算 2000 token) |
| **Warm** | `TypedMemoryStore` | `FileStore`(非 sqlite) | Store KV `["agent","memories"]` |
| Cold(可选) | `TypedMemoryStore` | FileStore | `["agent","cold_memories"]`(默认未用) |
| 会话历史 | `ConversationStore` | `FileConversationStore` | `~/.echo-agent/conversations/<id>.json` + `_meta.json` |
| runtime checkpoint | `RuntimeStateStore` | `FileRuntimeStateStore` | `~/.echo-agent/runtime_state/` |
| 语义层 | `EmbeddingStore`+`HttpEmbedder` | 条件启用(有 EMBEDDING_API_KEY) | `store.vecs.json` |

- **`MemoryType`**(10 变体):UserPreference / ProjectFact / ArchitectureDecision / DebuggingLesson / ErrorResolution / CommandPattern / ToolUsage / WorkflowPattern / SkillCandidate / DeprecatedNote。`MemoryMeta` 含 confidence/stability/recall_weight/risk/status/source 等。
- **`Store` trait**(`store.rs:182`):`put/get/search/search_with(Semantic|Hybrid)/delete/list/prune_expired/dedup_by_content`,namespace `&[&str]`。
- **召回复合分**(`recall.rs:77`):**`S = 0.5·sim + 0.3·0.5^(age_days/30) + 0.2·recall_weight`**;Superseded 过滤,recall_count 异步 +1。
- **读写时机**:recall 每轮 turn 入口(`context.rs:241 recall_long_term_memories`,硬编码 top_k=5,包 `<protected_memory>` 防压);写入 5 路径——(a)`remember` 工具、(b)`detect_and_write_memory_triggers`(user_correction/error_resolution/repeated_workflow)、(c)`pre_compaction_flush`(压缩前 LLM 抽 durable fact)、(d)transcript 持久化、(e)runtime checkpoint。
- **Hot/Warm 晋升**(`consider_promotion` layer.rs:507):`is_hot_eligible`(conf≥0.85 && stab≥0.70 && Active && risk≠High)且信任级达标才自动晋升;超预算按 `demotion_score` 降级。

### 3.2 LLM Prompt KV Cache(体系完整)

**4 段切分**(`cache/layout.rs:62`):system / canonical / history / runtime_context(零拷贝只读切片)。

- **稳定前缀哈希**(`diagnostic.rs:11`):SHA-256 只 hash system+canonical+tools+history(排除 runtime_context),16 位 hex;tool schema 走 canonical JSON(BTreeMap sorted keys)保跨进程稳定。
- **Anthropic 断点**(`anthropic_cache.rs:42`,最多 4):① SystemLastBlock ② ToolsLastTool ③ HistoryIndex(len≥4 取 75% 深度)④ HistoryLastStable;`truncate(4)` 硬卡上限;runtime_context 始终排除。
- **OpenAI/DeepSeek**:自动前缀缓存(`openai_cache.rs`)。
- **运行时**:`react_loop.rs:39 call_llm_with_retry` 建 `CacheHints`;`phases/think.rs:308`。Sprint 2 修了主 think 路径空 breakpoints 的回退 bug(此前主路径零 cache_control)。
- **应用层无通用 LRU/KvStore**:仅 approval_cache/docker 可用性/飞书去重等零散 ad-hoc 缓存。MASTER-PLAN 明确"不补应用层 LRU"——记忆负责长期、cache 负责 provider 侧短期。

### 3.3 上下文压缩(6 压缩器,CLI 用 SlidingWindow)

`ContextCompressor` trait(`compression.rs:434`):`compress(CompressionInput{messages, token_limit, current_query, focus}) → CompressionOutput{messages, evicted, checkpoint}`。`evicted` 供 L3 长期记忆提取(防纯丢)。

| 压缩器 | 原理 | 调LLM | CLI |
|---|---|---|---|
| **SlidingWindow** | 保留最近 N 条非 system,更早全 evicted | 否 | **默认** |
| Summary | keep_recent 之前调 LLM 出结构化摘要,失败回退自然语言→SlidingWindow | 是 | strategy=summary |
| Hybrid | pipeline(默认 Summary→SlidingWindow)+ short_circuit 省 LLM | 是 | strategy=hybrid(可选) |
| Adaptive | L1 Snip/Fold→L2 Micro→L3 Collapse→L4 auto-compact→L5 Reactive | L4 可选 | strategy=adaptive |
| VisibilityHorizon | 按 tool-group 摘要活跃窗口外的旧 tool 调用 | 否 | prepare 内独立 pre-pass |
| IncrementalSummary | 增量更新上次摘要(状态有记忆) | 是 | 暴露未接 |

- **`prepare` 编排**(`mod.rs:959`):快照基线 → VisibilityHorizon pre-pass(evicted 交 `MemoryPromoter.promote()`)→ 估 token + budget-aware `effective_limit` → `split_protected`(分离 `<skill_content>` 等)→ `compressor.compress` → `merge_protected` → L3 提取 → sanitize tool 配对 → verifier 兜底 → CanonicalContext 重注入(插 `sys_end` 保 prompt cache 前缀)。
- 触发条件:`estimated_tokens > token_limit`(默认 `usize::MAX` 不压)。

### 3.4 状态快照与长程事务回滚

- **Runtime checkpoint**(`snapshot.rs:312`):序列化完整 Message 列表 + `current_plan` + `active_skills` + `working_dir` + timestamp → `~/.echo-agent/runtime_state/`,供跨进程 resume。无 store/conversation_id 时静默 no-op。
- **Transcript projection**(`snapshot.rs:382`):`filter_user_visible_transcript` 后写 ConversationStore,GUI/TUI history 同形。
- **内存级快照回滚**(`react/mod.rs:1340`):`snapshot()`/`rollback(steps_back)`/`rollback_to(id)`,经 Tauri 暴露。
- **Saga 长程事务回滚:不存在。** grep `saga|compensat` 零命中;现有 `rollback` 仅限会话快照/evolution 审计/prompt 版本三处单一资源事务,**无跨服务编排式补偿事务**——符合本地单机定位。

---

## 维度 4 · 安全与认知

### 4.1 自进化闭环(最完整子系统)

引擎 `echo-agent/src/evolution/` + `src/improve/`,应用层 `app-core/src/evolution/`:

```
[触发] TriggerDetector(triggers.rs:8)
  └─→ [记忆分层] MemoryLayerManager(layer.rs:176)  ──过 EvolutionSecurityGuard(security.rs)
        └─→ [召回] MemoryRecaller(recall.rs:14,复合分)
              └─→ [候选] SkillCandidateDetector(candidate.rs:8)
                    └─→ [草稿] SkillDraftGenerator(draft.rs:8,确定性模板不调LLM)
                          └─→ [策展状态机] Curator(curator.rs:5)
                                ├─→ [Dreaming](dreaming.rs:92,离线反思,高召回晋升Hot/低召回降Archived)
                                └─→ [规则晋升] RulePromoter(rule_promoter.rs:14,高置信记忆→AGENTS.md)
```

- **Curator 状态机**(`curator.rs:30`):`Candidate → Draft → Active → Stale → Deprecated → Archived`。**只归档不删除**(全文无 `.delete()`),Archived 为终态,供参考。自动流转按空闲天数(Active≥30d→Stale→60d→Deprecated→90d→Archived),pinned 豁免。
- **Dreaming**(`dreaming.rs:92`):扫 Warm 层;高召回晋升 Hot,Archived 先 `revive_archived` 复活;Superseded 墓碑永跳过。
- **RulePromoter**(`rule_promoter.rs`):门槛 `min_confidence=0.95`/`min_age_days=7`/仅 3 类(ProjectFact/WorkflowPattern/UserPreference);读 AGENTS.md → 追加 `## Rules` → 写回,原记忆打 `<!-- PROMOTED_TO_RULE -->` 防重复。
- **缺口**:evolution hook 事件 8 个已定义但后 6 个(SkillPatchApplied 等)零 fire + 零消费者(deferred,事件暂无消费者不投机)。

### 4.2 护栏(Guardrails,四向 + 内容)

- **`Guard` trait**(`guard/mod.rs:62`)+ `GuardManager`(`:79`)并发执行遇 Block 短路;`GuardResult` = Pass/Block/Warn。
- **四向**(`:18`):Input / Output / ToolInput / ToolOutput。
- **两类 Guard**:`RuleGuard`(regex/长度/模式)+ `LlmGuard`(AI 分类);另有 `ContentGuard`(PII 检测/脱敏)。
- **演进安全闸**(`security.rs`):21 种密钥正则(AWS/GitHub/OpenAI/JWT/私钥)+ 21 条中英文提示注入信号;速率限制(50 写/会话、5 补丁/天、10 晋升/会话);规则晋升只允许 `InputTrustLevel::Trusted`。

### 4.3 权限控制(刻意轻量,无 RBAC)

- **无 RBAC/角色/能力模型。** 全仓 grep `rbac`/`role`/`capability` 仅命中无关项(subagent team 角色分工)。**无任何 Role/Permission/Capability 枚举或 ACL。**
- 仅有 **`PermissionMode`(应用层 policy.rs:16,四态)**:`Default / AutoEdit / FullAuto / Strict`——本质是"审批策略"(控制工具是否需用户确认),非访问控制。
- **设计依据**(AGENTS.md):本地个人助理,"不套用线上 Web 服务威胁模型",多用户权限隔离属过度设计;安全靠"进化安全闸 + Hook Permission 决策 + 护栏"分层兜底。

### 4.4 技能 + Hooks(自扩展)

- **Skills 双层**:代码技能(`agent.add_skill`)+ 文件技能 SKILL.md 三级渐进披露(catalog→activate→resources)。`skills/` 19 目录 50+ SKILL.md;`skills_hub/` 本地市场支持 Git 安装。
- **Hooks**(`skills/hooks.rs:130`):7 种 `HookAction`(Command/Prompt/Permission{allow|deny|ask}/Http/McpTool/Agent/ActivateSkill)+ 全生命周期事件点。命令走 sandbox、HTTP 走 SSRF-safe、不能升 permission_mode。

---

## 维度 5 · 运维与底层

### 5.1 可观测性/Tracing(自建 Run 模型,非 OTel)

- **不是 tracing crate 的 span 模型**,而是自建 `Run` 结构(`trace/mod.rs:34`:run_id/parent_run_id/session_id/status/events/token_usage/timings),经 `RunStore` trait 持久化。
- **两个实现**:`InMemoryRunStore`(测试用)+ `JsonlRunStore`(`mod.rs:404`,生产用,每 run 写 `{dir}/{run_id}.jsonl`)。
- 日志层才用 `tracing` crate 宏(如 `react_loop.rs:71/92` 的 LLM 请求/响应日志)。
- **可观测面板**(`observability/diagnostics.rs`):`compute_cache_diagnostics()` 聚焦"为何 cache 命中率低"——overall_read_rate、fingerprint 变更计数(system prompt/tools schema/cwd/Subagent prompt/provider)与建议修复。**非通用 token 面板**。
- `trajectory.rs` 存 ShareGPT JSONL,供离线 eval/改进。

### 5.2 Token 计费(精确统计,但无金额换算)

- **三层统计**:① `Usage`(`types.rs:655`)prompt/completion/total + 四家 cache 字段归一 + `cache_hit_rate()`;② `TokenUsageTracker`(`tokenizer.rs:222` AtomicU64 累加器,`cumulative_cache_hit_rate`);③ `TokenUsage`(`trace/mod.rs:231` Run 级饱和累加)。
- **事件流**:`AgentEvent::LlmUsage` 在 `stream_channel.rs:331`/`phases/think.rs:185` 发出。
- **计费能力:完全缺失。** grep `cost/billing/price/usd/计费` 代码零命中(仅文档命中)。Token 数量精确统计但**从未换算金额**——符合本地个人助理(无多租户计费需求)。

### 5.3 熔断/限流/重试(有熔断+指数退避,无限流)

- **`CircuitBreaker`**(`circuit_breaker.rs`)三态 Closed/Open/HalfOpen,`CircuitBreakerConfig`(failure_threshold/recovery/timeout),HalfOpen 限 1 并发探针防惊群;经 `ReactAgentBuilder::set_circuit_breaker` 注入 `GuardSubsystem`。
- **`RetryPolicy`**(`retry.rs:35`)指数退避 `delay_for`(exp 封顶 10)+ jitter + max_delay 60s;`retry_llm_call`(`react_loop.rs:22`)结合熔断+重试,`is_retryable` 判 network/429/5xx。
- **MCP 传输自带退避**:SSE(base 2000ms×2 封顶 30s)、HTTP(BASE_DELAY×2^n)。
- **无限流/throttle**:429 仅被识别后重试,无主动令牌桶限速。

---

## 综合评价(优劣与缺口)

| 维度 | 成熟度 | 亮点 | 主要缺口 |
|---|---|---|---|
| 交互入口 | ★★★★★ | `drive_chat` 统一驱动 + task_local ChatResources,三模式解耦干净 | 飞书 webhook 仅处理 text(多模态待补) |
| Subagent/调度 | ★★★★☆ | Sync/Fork/Teammate 三模式 + DAG + worktree/tmpdir 隔离地基完整 | writer 仍全局串行(max_concurrent_writes=1),gated 并行未释放;run_code 工具未做 |
| LLM 网关 | ★★★☆☆ | provider 抽象 + 四家 cache 归一完整 | **无动态路由**(静态配置驱动);无计费 |
| 记忆/压缩 | ★★★★★ | 6 层记忆 + 复合分召回 + 6 压缩器 + pre_compaction L3 提取,防纯丢 | Cold 层默认未用 |
| 沙箱/HITL | ★★★★☆ | 三层沙箱 + RCE 正则防护 + 8 态 permission 决策链 | 无容器时退化为宿主直跑(RCE 风险面,本地场景可接受) |
| 自进化 | ★★★★★ | 记忆→技能→规则闭环 + Dreaming + 只归档不删 + 安全闸 + 审计 | evolution hook 后 6 事件零 fire(deferred) |
| 安全/RBAC | ★★★☆☆ | 护栏四向 + 内容脱敏 | **刻意无 RBAC**(本地定位,非缺陷) |
| 运维 | ★★★★☆ | 自建 Run 模型 + 熔断 + 退避重试 + cache 诊断面板 | 无 OTel/分布式追踪;无限流;无计费 |

**当时结论**:EKO 在 **Agent 编排(多模式入口统一、Subagent 三模式、DAG + worktree 隔离)、记忆/压缩/缓存三层上下文管理、自进化闭环**三方面已有较深实现。后续状态已经变化，当前结论必须回到源码与 `MASTER-PLAN.md` 重新核验。
