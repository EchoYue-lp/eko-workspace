# EKO 深水区迭代计划

> **历史快照**：本文固定记录 2026-07-01 的 Sprint 设计与实施记录，不是当前展开 spec，也不是新窗口的任务入口。
> 当前权威只在 [`MASTER-PLAN.md`](MASTER-PLAN.md) 与仓库根 `AGENTS.md`；本文中的“目标”“待做”“下一步”和旧路径仅用于追溯，不得直接恢复实施。
> **最后更新**:2026-07-01(Sprint 5/6/6b/7/8/9/10 完成 `5d3262a`/`e9fee8e`/`e124eea`/`80738b6`/`08fdd07`+`e8fe8bf`/`420f062`/`8671582`+`bd4f1a3`;Phase 1 全部完成 + Phase 2 并行写主干 + 数据隔离通)
> **定位**:EKO = 通用 Agent 底座 + Coding/数据/科研垂直优化。不照搬纯 Coding Agent(Claude Code/Devin),用多领域特性做壁垒。
> **对标**:Claude Code `isolation: worktree` + Cursor 文件所有权 + Codex 并行 subagent(2025-2026 业界收敛模式)。

---

## 一、背景与定位

Sprint 1-10 + Sprint 6b + Subagent 超时统一完成了当时的基础补缺、并行写主干与数据隔离，包括 Subagent `.md` 热加载、`inherit_history` 切片、文件所有权分析、Fork worktree 和数据 Subagent 独立 tmpdir。

四条主线:
1. **Subagent 种类**:行为型(explorer/reviewer/...)→ 领域专家型(refactorer/data-shaper/analyst/lit-miner/bio-validator)。
2. **提示词**:历史硬编码定义 → `.md` 热加载 + 降噪继承。
3. **并行写**:安全串行(`max_concurrent_writes:1`)→ worktree/沙箱隔离 + 语义合并。
4. **超时/架构**:统一总线(已做)→ Team 断点存盘。

---

## 二、现状基线(动手前先查,防重复造轮子)

| 能力 | 现状 | 位置 |
|---|---|---|
| 4 个行为 Subagent | explorer/reviewer/planner/summarizer,当时硬编码中文 prompt,Fork **只读**(`readonly_tools()`) | 历史 `echo-agent-app-core/src/infra.rs`；当前路径须从源码重查 |
| SubagentKind::Custom | 框架**支持**从 `.md` 加载,但 EKO 未接目录扫描 | `echo-agent/src/agent/subagent/types.rs:46` |
| SubagentBuilder | `.system_prompt()`/`.fork_mode()`/`.inherit_history()`/`.tags()` | `echo-agent/src/agent/subagent/builder.rs:109/90/133` |
| inherit_history | 框架**已有**字段,Fork 默认 `Some(10)` | `echo-agent/src/agent/subagent/types.rs:84` |
| enhance_task | Fork 派发时把父 system_prompt + 历史拼到 task 前 | `echo-agent/src/agent/subagent/executor.rs:505` |
| 超时统一(三模式) | `SubagentExecutorConfig.default_timeout_secs`=600,接 `AgentConfig.subagent_timeout_secs` | `executor.rs:111/124` + `react/mod.rs:354` |
| **未接超时** | 历史 Team 编排存在独立超时 | 历史快照；当前实现须从源码重查 |
| 并行写信号量 | `max_concurrent_writes:1`(串行) + Subagent/shell 并发限制 | 历史 `echo-agent-app-core/src/tasks/task_runtime/executor.rs` |
| worktree 基础设施 | U1c stage-2 `RunWorktree::create/diff_summary/keep/remove` + `UnattendedWriteMode` 3 态 + Tauri list/merge/discard。**仅用于无人值守 run**(cron/IM),交互式 Fork 未接 | `echo-agent-app-core/src/tasks/task_runtime/worktree.rs` + `executor.rs:launch_unattended_run` |
| DAG 失败处理 | 失败 task→Failed,兄弟跑完,run Failed,下游 Blocked,重试跳过已完成 | `echo-agent-app-core/src/tasks/task_runtime/executor.rs:29-30/325-331/454-455/487-497` |
| SandboxManager | hooks 命令走它(docker.rs 30s TTL 缓存)。**待验**能否复用给 data subagent | `echo-execution/src/sandbox/` |
| skills 热加载 | SKILL.md frontmatter + `SkillLoader::discover`(DiscoveryScope: Project/User/Custom)+ 三级披露。**subagent .md 加载可镜像此 pattern** | `echo-execution/src/skills/external/loader.rs:43` |

---

## 三、Phase 1 — 短期(低风险,可本/新窗口)

### Sprint 5:team 超时接 `subagent_timeout_secs`(收尾)✅ 完成 `5d3262a`

**目标**:把 team 层 3 个独立超时默认接到统一 config,消除"一套配置"的最后盲区。

> **完成笔记**(2026-06-30):3 个默认值(300/120/300)→600,全部对齐 `AgentConfig.subagent_timeout_secs`。`TeamAgentBuilder` 增 `.timeout_secs(secs)` 透传(让持有 config 的调用方能注入;此前 build() 写死 default)。关键判断:team 模块零外部调用者,但属 pub 框架能力菜单(多 agent 编排),保留并对齐默认值**不删**(与死代码 `isolated.rs` 性质不同;isolated 单独清理)。3 测试守护。`verify-all-crates.sh` 全绿(8 crate + clippy + 12 feature)。

**范围**:
- 历史 Team manager Subagent timeout 默认值 → 从 config 读。
- `echo-agent/src/agent/subagent/team/runner.rs:27` `timeout_secs: 120`(默认)→ 从 config 读。
- `echo-agent/src/agent/subagent/team/mod.rs:46/59` `default_timeout_secs: 300`(TeamConfig 默认)→ 600 + 接 config。

**方案**:
- Team manager Subagent、runner 与 config 从 `SubagentExecutorConfig.default_timeout_secs` 或 `AgentConfig.subagent_timeout_secs` 读超时,不写死。
- 检查 team 层是否能访问 `SubagentExecutorConfig`(Team 模式由 SubagentExecutor.dispatch_teammate 派发,executor 有 config)——能则透传,否则经 `AgentConfig` 路径。
- 默认值统一 600(对齐)。

**依赖**:无(独立)。
**风险**:低。Team 层构造签名可能要加参数,调用点少。
**验证**:`verify-all-crates.sh`(echo-agent)+ clippy + fmt。加测试断言 team 超时从 config 读。
**验收**:Team 层超时不再硬编码，所有 Subagent 超时由同一配置派生。

### Sprint 6:subagent `.md` 热加载 ✅ 完成 `e9fee8e`

**目标**:把 4 个 Subagent 的 prompt 从 `infra.rs` 硬编码剥到 `.echo-agent/subagents/*.md`,无需重编译即可调提示词;为 Phase 3 领域专家 Subagent 铺路。

> **完成笔记**(2026-06-30):加载器放**应用层**(`echo-agent-app-core/src/subagent_loader.rs`),框架 `SubagentKind::Custom` 保持惰性占位。应用直接产出 `SubagentDefinition`，并按项目 > 用户 > 内置的优先级发现 `.md`；只读标记在 builder 边界强制。旧硬编码定义随后删除。

**范围**:
- 新建 `.echo-agent/subagents/coding/{explorer,reviewer,planner,summarizer}.md`(或按领域分目录)。
- 历史 `echo-agent-app-core/src/infra.rs`:删旧硬编码常量 + `register_default_subagents` 改为**扫描目录**加载 .md 定义(镜像 skills 的 `SkillLoader::discover` pattern)。
- 框架层(`echo-agent/src/agent/subagent/`):若 `SubagentKind::Custom` 的 .md 加载逻辑不完整,补全 frontmatter 解析(name/description/system_prompt/mode/inherit_history/tags)。

**方案**:
- .md frontmatter 字段对齐 `SubagentDefinition`:`name`/`description`/`system_prompt`(body)/`execution_mode`/`inherit_history`/`tool_filter`/`tags`/`readonly`。
- 加载器:启动时扫 `.echo-agent/subagents/**/*.md`(用户级)+ 项目级 `./.agents/subagents/`(对标 skills 的 Project/User scope),解析 frontmatter → `SubagentDefinition`(Custom kind)→ 注册。
- 保留 `readonly_tools()` 物理强制(在只读 Subagent builder 层,不在 .md)——`readonly` frontmatter flag 决定是否走只读 builder。
- 向后兼容:若目录不存在,回退到内置默认(或要求初始化时生成默认 .md)。

**依赖**:无。
**风险**:中低。.md frontmatter 解析 + scope 优先级(Project 覆盖 User)需仔细;`register_default_subagents` 重构动 app-core 启动路径。
**验证**:`verify-all-crates.sh` + workspace test + GUI target。加测试:从 temp dir 加载 .md → 注册成功 + prompt 正确。
**验收**:改 .md prompt 不需重编译;历史硬编码定义零命中;4 个 Subagent 行为不变(回归)。

### Sprint 6b(可选):per-subagent `inherit_history` 降噪 ✅ 完成 `e124eea`

**目标**:用框架已有字段减少 Fork 继承的上下文噪声(替代"动态实体抽取"的务实版)。

> **完成笔记**(2026-06-30):核实发现一个 bug——`enhance_task` 此前忽略 `inherit_history`。修复后 Fork 默认历史数降为 2，聚焦型 Subagent 不再被长历史前缀稀释；per-Subagent 配置仍可覆盖。

**范围**:
- `echo-agent/src/agent/subagent/executor.rs:505 enhance_task`:从"父 system_prompt + 最近 N 条全吞"改为可配置切片(默认只取最后 1 条 user + 相关 tool result,而非 10 轮)。
- .md frontmatter 暴露 `inherit_history` 字段(Sprint 6 已加)。

**方案**:
- `enhance_task` 按 `definition.inherit_history` 决定取多少(0=不继承,1=最后 1 条,N=最后 N 条)。
- 不做实体抽取(留 Phase 3)——只做"少继承 + 切最后"的简单降噪。
- 默认 Fork inherit_history 从 10 降到 2-3(减少 bloat),.md 可覆盖。

**依赖**:Sprint 6(.md 字段)。
**风险**:低。改 `enhance_task` 切片逻辑 + 默认值。
**验证**:echo-agent test + 加测试:inherit_history=0 → task 不拼父 context;=2 → 只拼最后 2 条。
**验收**:Fork subagent 的 task 前缀变短;`inherit_history` 字段生效。

---

## 四、Phase 2 — 中期(高风险,需 spec 先行 + 新窗口)

> Phase 2 是并行写的核心,动 TaskRuntime DAG + 交互式 Fork + 写信号量,命脉层。**每个 Sprint 需独立 spec + 新鲜上下文**(AGENTS.md 规则 5)。

### Sprint 7:plan-time 文件所有权分析器(地基)✅ 完成 `80738b6`(纯分析器范围)

**目标**:DAG plan 阶段分析各 PlanTask 的文件交集,为"并行写 vs 串行"决策提供依据。这是 Phase 2 能成立的前提(无它则 worktree merge 一堆冲突)。

> **完成笔记**(2026-06-30):**范围经核实+用户拍板收窄为"纯分析器地基"**。规格原验收("run_dag 按交集图调度+worktree 并行")与本 Sprint"地基、依赖:无"定位冲突;且核实:`PlanTask.files` 字段**已存在**(无需新增 touched_files)、写信号量=1 已串行所有 writer、runtime 已有 advisory 文件冲突告警(executor.rs:982)。故取纯函数 `analyze_file_ownership(&[PlanTask]) -> OwnershipReport`(writer 全对文件交集 → `FileOverlapPair`,reader 忽略、空 files 跳过、路径归一化纯字符串)+ `insert_task`/`update_plan_task` 非阻塞 plan-time `tracing::warn` 告警。**不动 run_dag 调度/不接 worktree**——run_dag 行为变更属命脉层,且写信号量=1 时与现状等价(冗余),真生效需 Sprint 9。8 个测试(disjoint/相交/reader 忽略/空 files/路径归一/lex 序/空 plan/helper)。验证全绿(fmt + clippy + test --workspace 353 + GUI target + 前端)。下一步 Sprint 8(交互式 Fork worktree)消费本分析器报告。

**范围**:
- `echo-agent-app-core/src/tasks/task_runtime/planner.rs`(plan 生成)+ `executor.rs:run_dag`(调度)。
- 新增 `FileOwnershipAnalyzer`:每个 PlanTask 声明/推断它要写的文件集合,分析器输出"任务文件交集图"。

**方案**:
- PlanTask 增 `touched_files: Vec<String>`(planner 生成时填,或从 task 描述 LLM 抽取)。
- 分析器:两两任务文件交集 → 标记 `can_parallel`(不相交)vs `must_serialize`(相交)。
- run_dag 调度:`can_parallel` 组分到不同 worktree 并行;`must_serialize` 组串行。
- 对标 Cursor "file-level ownership is the foundation"。

**依赖**:无(独立地基)。
**风险**:中。文件交集推断可能不准(LLM 抽取 touched_files 有误差)——先接受不精确,用 worktree 隔离兜底(即便相交,worktree 也物理安全,只是 merge 时要解冲突)。
**验证**:workspace test + 加测试:相交任务→must_serialize;不相交→can_parallel。
**验收**:planner 输出 touched_files;run_dag 按交集图调度。

### Sprint 8:交互式 Fork worktree 隔离 ✅ 完成 `08fdd07`(框架)+ `e8fe8bf`(应用)

**目标**:把 U1c `RunWorktree::create + agent.set_working_dir` 从"无人值守 run"上提到"交互式 Fork dispatch",每个 Fork writer 拿自己的 worktree。

> **完成笔记**(2026-06-30):**范围经核实后收窄为"框架地基注入机制"**。当时不存在交互式 Fork writer，DAG writer 任务在主 Agent 原地执行；要满足 worktree 隔离需另做 DAG writer 路由。故先完成纯注入机制:
> - **框架**(`08fdd07`):`WorktreeFactory` trait(`create→WorktreeHandle{path,finalize}`,框架不接 git,应用注入)+ `SubagentExecutorConfig.worktree_factory` + `AgentConfig.subagent_worktree_factory`(cfg subagent)+ `ReactAgentBuilder.subagent_worktree_factory()` 透传 + `SubagentDefinition.isolate_worktree` + builder `.isolate_worktree()` + `Agent` trait `set_working_dir`/`clear_working_dir`。`dispatch_fork`:isolate+factory→create(**失败→Fork Err 不静默**,防写落主区)+ set_working_dir + 执行 + finalize(diff 追加 output)+ set_working_dir(None) 还原;factory=None+isolate→warn 不隔离。**默认 isolate_worktree=false → 零行为变更**。
> - **应用**(`e8fe8bf`):`EkoWorktreeFactory`(impl 框架 trait,`eko-fork-` 前缀,keep 不自动 merge 对齐 U1c/Claude Code Q1)+ `RunWorktree::create_fork`(`eko-fork-` 区分 unattended)+ 主 agent build 注入(git_repo_root 解析)+ frontmatter `worktree:true`(writer-only,readonly 声明被忽略)。
> - **不动 run_dag/DAG writer 路由**——那是 Sprint 9(写信号量 gated 释放 + writer→Fork worktree 路由,命脉调度层)。本轮 dispatch_fork 隔离机制就位,但当前无 writer Fork 路径激活它。
> - 调研依据:Claude Code `isolation:worktree`(官方 docs)+ Codex/Cursor per-agent worktree + johannesjo/parallel-code。测试:框架 8(dispatch_fork × 4:isolate+factory/isolate 无 factory/factory 失败/readonly)+ 应用 4(factory 非 git 失败 + frontmatter × 3)。验证全绿。

**范围**:
- 历史 `dispatch_fork`:Fork 派发时,若 Subagent 是 writer 角色(非 readonly),懒创建 worktree + `set_working_dir(worktree_path)` + 跑完 `diff_summary` 落 artifact。
- 复用 `echo-agent-app-core/src/tasks/task_runtime/worktree.rs` 的 `RunWorktree` 生命周期。
- 安全回退:worktree 创建失败 → Fork Failed(不静默继续,防无隔离写落主区)——对齐 U1c。

**方案**:
- Fork dispatch 增 `write_mode`(Disabled/Worktree/InPlace,复用 U1c `UnattendedWriteMode`)。
- write_mode=Worktree 时:懒创建 worktree(基于 plan 的文件所有权——Sprint 7)+ set_working_dir + 跑完 diff_summary。
- readonly Subagent 不建 worktree(沿用当时现状)。
- 跨仓库:worktree.rs 在 app-core;dispatch_fork 在 echo-agent 框架。需决定 worktree 创建逻辑放框架还是应用层(AGENTS.md 框架vs应用:worktree 是 EKO 产品形态依赖→应用层;但 Fork dispatch 在框架。可能需要应用层注入 worktree factory)。

**依赖**:Sprint 7(文件所有权决定哪些 Fork 需要 worktree)。
**风险**:高。跨框架/应用层 + worktree 生命周期 + 与现有 delegate_to_agent* 路径整合。需 spec 先行。
**验证**:verify-all-crates + workspace + GUI。加测试:writer Fork → worktree 创建 + set_working_dir + diff_summary;readonly Fork → 无 worktree。
**验收**:交互式 Fork writer 写到 worktree 不落主区;创建失败 → Failed。

### Sprint 9:DAG writer→Fork worktree 路由 ✅ 完成 `420f062`(隔离+路由,不放开并行)

**目标**(原规格):放开 `max_concurrent_writes` 1→4(仅 worktree 隔离生效时),无隔离则仍串行;复用 Tauri list/merge/discard 做 review。

> **完成笔记**(2026-07-01):**范围经核实+用户拍板收窄为"隔离+路由,不放开并行"**。核实发现 writer 此前在主 agent 原地跑(`run_main_agent_task`,持 execution_mutex),根本无 Fork writer 路径;且"按所有权 gated 并行释放"是命脉门控(错放开=Cursor 同文件 merge conflict 坑)。故本 Sprint 只交付:
> - **路由**:Implementation/Debugging 从"主 Agent 原地"→"Fork Subagent + 隔离 worktree"，消费 Sprint 8 的隔离机制。
> - **不放开并行**:max_concurrent_writes 仍 1,writer 仍串行(write_sem),但各自跑自己 worktree(不污染主区)。
> - 改动(纯应用层):builtin `implementer.md`(readonly:false/worktree:true)+ writer Subagent builder(省 `.readonly_tools`→完整写能力)+ register 分支 readonly/writer + Implementation/Debugging→`implementer` 路由 + 附件感知 delegate。
> - **留未来**:按所有权 gated 并行(相交串行/不相交并行)——命脉门控,本 Sprint 不碰。复用 Tauri list/merge/discard 做 Fork-worktree review 也留后续(Sprint 8 已铺 eko-fork- 前缀,list 命令扩展即可)。
> - 验证全绿(workspace 360 passed + GUI + 前端);role_routing × 3 测试 + builtin/nonexistent_scope 更新。

**范围**:
- `echo-agent-app-core/src/tasks/task_runtime/executor.rs:47-64 ConcurrencyLimits` + run_dag 写信号量获取逻辑。
- `echo-agent-cli/src/tauri/commands/panels.rs`(list/merge/discard_unattended_worktrees)——复用给交互式 worktree。

**方案**:
- 写信号量获取条件:仅当 dispatch **未获 worktree 隔离**时获取(串行);已获 worktree 隔离则跳过(并行,因不同 worktree 不冲突)。
- `max_concurrent_writes` 可配(YAML),默认仍 1(保守),用户可调 4-6。
- worktree 跑完 → 用户经 Tauri 命令 review/merge/discard 每个 Fork 的 worktree。

**依赖**:Sprint 8(交互式 Fork worktree)。
**风险**:高。信号量门控逻辑 + 误放开(无隔离时放开=Cursor 同文件冲突坑)。**必须**门控正确。
**验证**:workspace test + 加测试:无隔离→串行(信号量获取);有隔离→并行(跳过信号量)。
**验收**:并行写仅在 worktree 隔离时发生;无隔离仍串行;merge/review 流可用。

### Sprint 10:Data 沙箱 + 不相交输出文件 ✅ 完成 `8671582`(框架)+ `bd4f1a3`(应用)

**目标**:数据/科研场景的并行(不需 git,需 Python/R 沙箱)。

> **完成笔记**(2026-07-01):**调研先行**。结论:SandboxManager 是 stateless execute-and-done(无持久 per-Subagent 环境,不适合作数据 Subagent 工作区);worktree 与纯数据不匹配。故取**tmpdir 工作区原语**:
> - **框架**(`8671582`):`DataWorkspaceFactory` trait(create→`DataWorkspaceHandle{path,finalize}`,finalize 列产出文件)+ `SubagentExecutorConfig.data_workspace_factory` + `SubagentDefinition.isolate_workspace` + `dispatch_fork` workspace 分支(与 worktree 互斥,worktree 优先)。复用 Sprint 8 set_working_dir 机制。
> - **应用**(`bd4f1a3`):`EkoDataWorkspaceFactory`(tempfile `eko-data-<label>-`,**keep 不自动清理**——analyst 要读片段)+ 主 agent **总是注入**(无 git 依赖)+ frontmatter `workspace:true` + builtin `data-shaper.md`/`analyst.md`。
> - **Sprint 10b(run_code 工具,Python/R 经 SandboxManager 护栏)留后续**——当时数据 Subagent 用 Polars 数据工具。
> - 验收:数据 Subagent 并行跑不互相污染(各自 tmpdir)+ 输出不相交(隔离工作区)+ analyst 综合读各片段。7 个内置 Subagent。验证全绿。

**范围**:
- 先**验证** `echo-execution/src/sandbox/SandboxManager`(docker.rs)能否复用给 subagent 的 Python/R 运行(支持子进程级 FS 隔离?还是仅 docker?)。
- 若可复用:data-shaper/analyst Subagent 在各自沙箱(独立目录/内存)并行跑;输出到**不相交文件**(`run_001_clean.parquet` / `run_002_*.parquet`)。
- 合并:collector 只 **concat**(不 mid-stream merge),analyst 二次综合读所有片段→出 report。

**方案**:
- 不做 csv/parquet mid-stream merge(难度大,易 schema 冲突)——降维成"不相交输出 + concat + synthesize"。
- 沙箱生命周期:per-Fork 沙箱(类似 worktree 但无 git)。

**依赖**:Sprint 8(隔离机制)+ SandboxManager 验证。
**风险**:中高。SandboxManager 能力待验;若不支持子进程 FS 隔离,需另建轻量沙箱(tmpdir + 进程隔离)。
**验证**:先出调研结论(SandboxManager 能力),再决定方案。
**验收**:data Subagent 并行跑不互相污染;输出文件不相交;analyst 能综合。

### Sprint 11:Team 断点存盘

**目标**:历史 Team manager-Subagent 模式加持久化 + checkpoint,超时重启时读取已达成共识,不从头辩论。

**范围**:
- 历史 Team manager-Subagent 实现:加阶段性 Synthesis 缓存(写 store)+ 重启重试时读上次共识。
- 对标 DAG 的"skip completed on retry"(`executor.rs:454`)。

**方案**:
- manager Subagent 每轮 synthesis 后写 checkpoint(store 或文件)。
- 重试时 manager 读上次 checkpoint,从共识处继续,不重跑已完成的 Subagent。
- 需要 Team 模式的状态持久化(目前 Team 无 TaskRuntimeStore 级持久化)。

**依赖**:无强依赖(可独立于 Phase 2 其他项),但与 Team 超时(Sprint 5)配套。
**风险**:中。Team 状态持久化 + resume 协议是新机制。
**验证**:echo-agent test + 加测试:manager 中断后重启 → 读 checkpoint 跳过已完成 Subagent。
**验收**:Team 长会诊超时后重试不丢已达成共识。

---

## 五、Phase 3 — 长期(高,依赖外部数据源)

### Sprint 12:领域专家 Subagent prompt 矩阵

**目标**:在 Sprint 6 的 .md 加载基础上,补齐领域专家 Subagent 的 .md 定义。

**范围**:`.echo-agent/subagents/{coding,data,medical}/*.md`:
- Coding:`refactorer`(writer,带 write/shell 工具 + 绑 worktree)。
- Data:`data-shaper`(ETL/Schema,只读/沙箱)、`analyst`(跑统计/出图,runner)。
- Medical:`lit-miner`(文献检索)、`bio-validator`(指南比对)。

**方案**:每个 .md 定义角色 prompt + 工具需求(`tool_filter`/`readonly` flag)+ 继承策略。refactorer/analyst 标 writer→走 Sprint 8 worktree;data Subagent 标沙箱→走 Sprint 10。

**依赖**:Sprint 6(.md 加载)+ Sprint 8/10(隔离)。
**风险**:中(prompt 层)。工具层见 Sprint 13。
**验收**:6+ 领域专家 Subagent 可加载 + 派发。

### Sprint 13:工具层接入(lit-miner / bio-validator)

**目标**:补齐领域 Subagent 依赖的外部数据源工具。

**范围**:
- `lit-miner`:接 PubMed/ArXiv 检索——通过 MCP server(built-in 或外部)或内置 HTTP 工具。
- `bio-validator`:接临床指南(NCCN/UpToDate)——RAG 知识库或 MCP。

**方案**:优先用 MCP(用户自配 MCP server,AGENTS.md 用户自扩展定位);内置最小检索工具作 fallback。

**依赖**:外部数据源可用性(MCP server / API key)。
**风险**:高。依赖外部服务 + 数据质量。
**验收**:lit-miner 能查 PubMed 返回结果;bio-validator 能比对指南。

### Sprint 14:完整实体降噪继承(研究项)

**目标**:"动态实体抽取"继承——Coding 只继承类名/函数签名/error log;Medical 只继承患者表型/药物/靶点。

**方案**:研究项——实体抽取(启发式 or LLM 抽取)+ 按领域模板过滤。替代 Sprint 6b 的简单切片。

**依赖**:Phase 1-2 完成。
**风险**:高(研究问题,实体抽取可靠性)。
**验收**:同领域长对话下,subagent 继承的上下文精准度显著优于"最近 N 条全吞"。

---

## 六、跨阶段约束

### 框架 vs 应用(AGENTS.md)
- `.md` 加载器、worktree 生命周期、sandbox → **应用层**(EKO 产品形态依赖)。
- `SubagentExecutor` dispatch/timeout/`SubagentDefinition` → **框架层**(通用)。
- worktree factory 注入框架 dispatch(应用层构造,框架调用)——避免框架反向依赖应用。

### 验证规范(AGENTS.md)
- echo-agent:`./scripts/verify-all-crates.sh`(fmt + 逐 crate test + clippy + 12 feature 矩阵)。
- echo-agent-cli:`cargo fmt --all -- --check` + `check --workspace` + `test --workspace` + GUI target(`--features gui --bin echo-agent-tauri`)+ clippy + `cargo clean`。
- 跨仓库合并:echo-agent 先,echo-agent-cli 后。

### 上下文管理(AGENTS.md 规则 5/6)
- Phase 1(Sprint 5/6/6b)低风险,可本/新窗口。
- **Phase 2(Sprint 7-11)命脉层高风险,每个 Sprint 独立 spec + 新鲜上下文**。
- 每个 Sprint 完成 + 提交后,更新 MASTER-PLAN §二/§三/§四/§五 + 本文状态。

### 决策记录(防回头路)
- **D-并行写-1**:并行写必须 gated on 隔离(worktree/沙箱)生效;无隔离则串行。否则 = 同文件 merge conflict(Cursor 坑)。
- **D-合并-1**:Data/Research 不做 csv/parquet mid-stream merge;用不相交输出文件 + concat + 二次综合。
- **D-降噪-1**:近 Term 用 `inherit_history` 字段 + 切片(Sprint 6b);完整实体抽取留 Phase 3 研究,不投机。
- **D-领域Subagent-1**:domain Subagent 的 prompt 层(Phase 1-2)与工具层(Phase 3,依赖外部 MCP/RAG)分开;不把工具缺口当 prompt 问题。
- **D-超时-1**:三模式 + team 层统一从 `subagent_timeout_secs` 读(0=无超时);per-subagent `SubagentDefinition.timeout_secs` 仍可覆盖。

---

## 七、状态总表

| Sprint | Phase | 状态 | 复杂度 | 仓库 |
|---|---|---|---|---|
| 5 team 超时收尾 | 1 | ✅ 完成(`5d3262a`) | 低 | echo-agent |
| 6 subagent .md 热加载 | 1 | ✅ 完成(`e9fee8e`) | 中低 | echo-agent-cli |
| 6b inherit_history 降噪 | 1 | ✅ 完成(`e124eea`) | 低 | echo-agent |
| 7 文件所有权分析器 | 2 | ✅ 完成(`80738b6`,纯分析器地基) | 中 | echo-agent-cli |
| 8 交互式 Fork worktree | 2 | ✅ 完成(`08fdd07`+`e8fe8bf`,框架地基注入机制) | 高(取低范围) | 两仓库 |
| 9 DAG writer→Fork worktree 路由 | 2 | ✅ 完成(`420f062`,隔离+路由不放开并行) | 高(取中范围) | echo-agent-cli |
| 10 Data 沙箱(tmpdir 工作区) | 2 | ✅ 完成(`8671582`+`bd4f1a3`,tmpdir 原语无 git) | 中高(取中) | 两仓库 |
| 10b run_code 工具 | 2 | ✅ 完成(`273638c`+`d751209`+`dc699c5`+`ff286f0`+`349080b`,RCE warn-not-deny + R 原生 Code 后端) | 中(取低) | 两仓库 |
| 11 Team 断点存盘 + ExecutionMode::Team 接线 | 2 | ✅ 完成(`96b1717`+`14e4020`+`dbc4e68`+`a487774`+`08e8f9b`+`0f2c853`,TeamAgent 接 dispatch_team + manager Subagent checkpoint/resume) | 中-高(动命脉 dispatch,取中) | 两仓库 |
| 12 领域专家 Subagent prompt | 3 | 历史未执行项，不是当前任务 | 中 | .md |
| 13 工具层(lit-miner/bio-validator) | 3 | ⏳ 待做(依赖外部) | 高 | 两仓库 |
| 14 完整实体降噪 | 3 | ⏳ 研究项 | 高 | echo-agent |

> 已完成:Sprint 1-11 + 6b + 10b + subagent 超时统一(详见 MASTER-PLAN §三)。Phase 1 全部完成;Phase 2 并行写主干 + 数据隔离 + run_code + Team 接线与断点存盘全通(Sprint 7+8+9+10+10b+11);剩按所有权 gated 并行、领域专家(Phase 3)。
