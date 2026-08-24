# Sprint 10b + 11 设计 spec

> **范围**:两个独立 Sprint,本 spec 一并设计但**分阶段实施**(Sprint 10b 先,Sprint 11 后)。
> - **Sprint 10b**:`run_code` 工具(框架新工具,Python/R/JS 代码执行经 SandboxManager,working_dir 绑 worker 工作区)。低-中风险,greenfield。
> - **Sprint 11**:`ExecutionMode::Team` + Team checkpoint/resume(把 TeamAgent 接入真实 dispatch 路径 + 用 TaskNode 做 plan/worker/synthesis 三态持久化 + skip-on-resume)。中-高风险,命脉门控层。
>
> **最后更新**:2026-07-01(设计已与用户对齐确认,含 RCE 护栏策略 + R 原生支持 + TeamSpec 字段 + 名称引用 worker + 删死字段)。

---

## 一、背景与定位

### Sprint 10b 背景
Sprint 10(`8671582`+`bd4f1a3`)交付了数据 worker 的 per-worker tmpdir 工作区原语(`DataWorkspaceFactory` → `set_working_dir` → `ToolContext.working_dir`),但数据 worker 目前只能用 Polars 数据工具(只读 + export),**不能跑任意 Python/R 脚本**。Sprint 10b 补齐这块:新增 `run_code` 工具,让数据/科研 worker 能跑任意脚本,自动落在 worker 的隔离工作区。

### Sprint 11 背景(经调研纠偏,关键!)
**原 spec 假设已部分失效**,经代码核实纠正:

1. **Team 模式不是"round-based 辩论"** — `ManagerWorkerOrchestrator::run`(`team/manager_worker.rs:39`)是 **single-pass**:Phase 1 plan(LLM 分解子任务)→ Phase 2 fan-out(并发 worker)→ Phase 3 synthesize(单次合并)。无 rounds、无"轮间共识"。所以 checkpoint 的粒度是 **per-phase / per-worker**,不是 per-round。

2. **`ExecutionMode::Teammate`(dispatch 路径)≠ `TeamAgent`/`ManagerWorkerOrchestrator`(team 模块)** — 两个不相连的概念:
   - `dispatch_teammate`(`executor.rs:422`)是单 agent 异步 spawn+poll,**不调用 TeamAgent**。
   - `TeamAgent`(`team/mod.rs:250`,多 agent 策略 ManagerWorker/Pipeline/Debate/Swarm)**零生产调用者**,只在 `mod.rs:546-553` 测试中构造。
   - 用户拍板:**用 Option 4(新增 `ExecutionMode::Team` 变体)**把 TeamAgent 接入真实 dispatch 路径。

3. **`ManagerWorkerOrchestrator.max_retries` / `worker_timeout_secs` 是死字段**(声明但从不读),`TeamAgent::execute` 外层 `tokio::time::timeout`(`mod.rs:263`)才是真超时。按 AGENTS.md "看到就删"删之。

### 对标业界
- **Claude Code** `isolation: worktree` + 内置 code execution(子 agent 在隔离环境跑代码)。
- **Codex / Cursor** 并行 subagent + sandbox code execution。
- 共性:**LLM 生成的代码必须经沙箱跑 + 绑工作目录**,本地个人助理场景"开箱即用"优先于"零信任拒绝"。

---

## 二、Sprint 10b — `run_code` 工具

### 目标
新增框架工具 `run_code`,让数据/科研 worker subagent 能执行任意 Python/R/JS/... 代码片段,自动落在 worker 的隔离 tmpdir 工作区(Sprint 10 链路)。

### 范围(file:line,以核实为准)

**框架层(`echo-agent/`)**:
1. **修补 `Code` 后端 R 支持**(决策 b):
   - `echo-execution/src/sandbox/local.rs:132-145` `build_code_command` 的 interpreter match 增 `"r" => ("Rscript", "-e")`。
   - `echo-execution/src/sandbox/docker.rs:273-283` `build_inner_command` Code 分支增 R(对齐 image `rocker/r-base:latest`,见 `mod.rs:84`)。
2. **新工具 `echo-tools/src/code.rs`**(`RunCodeTool`):见下"组件"。

**应用层(`echo-agent-cli/echo-agent-app-core/src/`)**:
3. 注册:`echo-tools/src/registry.rs:197 register_all_tools` 加 `RunCodeTool::new()`。
4. Data worker 显式接线:`infra.rs` data-shaper/analyst worker 注册时 `agent.add_tool(Box::new(RunCodeTool::new().with_sandbox_manager(mgr)))`(data worker 走 readonly subset,`infra.rs:607`,需显式 add)。
5. 更新提示词:`echo-agent-app-core/src/subagents/data/{data-shaper.md,analyst.md}` 加一段告知 LLM `run_code` 工具可用 + 强调 working_dir 语义(见下"提示词")。

### 组件:`RunCodeTool`(`echo-tools/src/code.rs`)

**模型**:`RunSkillScriptsTool`(`echo-execution/src/skills/external/run_script_tool.rs:62-108`,持 sandbox 的 Tool)+ `ShellTool::execute_with_context`(`echo-tools/src/shell.rs:378-466`,`ctx.working_dir → SandboxCommand.with_working_dir` 模式)。

```rust
pub struct RunCodeTool {
    sandbox: Option<Arc<SandboxManager>>,
}

impl RunCodeTool {
    pub fn new() -> Self { Self { sandbox: None } }
    pub fn with_sandbox_manager(mut self, m: Arc<SandboxManager>) -> Self {
        self.sandbox = Some(m); self
    }
}

impl Tool for RunCodeTool {
    fn name(&self) -> &str { "run_code" }
    fn description(&self) -> &str { "执行一段代码(Python/R/JavaScript/...)。自动在当前任务工作目录运行。" }
    fn parameters(&self) -> serde_json::Value {
        // { language: enum[python,r,javascript,ruby,perl,php,bash], code: string }
    }
    fn permissions(&self) -> Vec<ToolPermission> { vec![ToolPermission::Execute] }
    fn risk_level(&self) -> ToolRiskLevel { ToolRiskLevel::Dangerous }

    fn execute_with_context<'a>(
        &'a self, params: ToolParameters, ctx: &'a ToolContext,
    ) -> BoxFuture<'a, Result<ToolResult>> {
        Box::pin(async move {
            // 1. 熔断:未知语言直接工具层返回错误,不下发沙箱(用户建议 #3)
            let language = params.get("language").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::MissingParameter("language".into()))?;
            validate_language(language)?; // 不在白名单 → Err
            let code = params.get("code").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::MissingParameter("code".into()))?;

            // 2. RCE 护栏(决策 a):有沙箱→用;无沙箱→warn 不拒(AGENTS.md 本地助理模型)
            let cmd = SandboxCommand::code(language, code);
            let cmd = if let Some(dir) = &ctx.working_dir {
                cmd.with_working_dir(dir)  // data worker tmpdir
            } else { cmd };

            if let Some(ref mgr) = self.sandbox {
                if !mgr.has_container_sandbox() {
                    tracing::warn!(
                        "Running unsandboxed inline code in local-only mode. \
                         Ensure you trust the generated code."
                    );
                }
                let res = mgr.execute(cmd).await?;
                Ok(ToolResult::success(format_output(&res)))
            } else {
                // 完全无 SandboxManager:仍 warn + 走 bare tokio::process 回退
                tracing::warn!("No SandboxManager configured for run_code; running bare.");
                run_bare(cmd).await
            }
        })
    }
}
```

### 关键决策

**(a) RCE 护栏 = "warn 不拒"**(`AGENTS.md` 本地个人助理模型)
- `SandboxManager::has_container_sandbox()`(`manager.rs:142-144`)为 false 时,`tracing::warn!` 醒目告警,但**仍执行**。
- 理由:EKO 是本地个人助理,用户让它跑 Python 处理本地 Excel,弹"无沙箱拒绝"会破坏开箱即用。`manager.rs` 现有 XSS→RCE 注释面向线上服务,不适用本地。
- 未来留 `security.allow_unsafe_local_code` 配置开关(本 Sprint 不实现,YAGNI)。

**(b) R 执行 = 修补 `Code` 后端**(原生一等公民,非临时文件补丁)
- `local.rs:132-145` 增 `"r" | "R" => ("Rscript", "-e")`。
- `docker.rs:273-283` Code 分支增 R(对齐 `rocker/r-base:latest`)。
- 备选(写临时文件 + `Program{Rscript}`)**不取** — R 是一等公民应原生支持;现有所有语言都走 `Code` 路径,R 对齐才一致。
- 防御性回退:若 docker 修补发现 R 镜像复杂度飙升,工具层回退 `Rscript` 文件路径;但 local 后端绝对原生。

**(c) 内联代码执行方式** — 现状 `Code` 后端用 **arg-based**(`-c`/`-e` flag,`local.rs:154-159`),所有语言一致。
- **本 Sprint 保持 arg-based**(与现有所有语言一致,增量最小)。
- **已知限制 / 未来工作**:arg-based 有 ARG_MAX 长度限制 + shell 转义风险(用户建议 #1 指出 stdin 模式更稳)。**全语言切 stdin 是 cross-cutting 优化,影响 Python/JS/...,不在本 Sprint 范围** — 留独立 follow-up(本 Sprint 只加 R,arg-based 一致)。

**(d) `validate_language` 熔断大小写不敏感**(用户 review 补丁 #1):
- 入口 `validate_language(language)?` 前先 `let language = language.to_lowercase();`,白名单全小写匹配。
- LLM 偶发输出 `Python`/`PYTHON`/`R`,统一 `.to_lowercase()` 规避。

**(e) Docker R 修补不试错**(用户 review 补丁 #2):
- `docker.rs` Code 分支增 R 时**直接盲信** `rocker/r-base:latest` 镜像存在,不在工具层做"探测+回退"。
- 镜像缺失让 Docker 引擎抛 `ImageNotFound`/`CommandFailed`,工具层统一捕获转 `ToolResult`,保持干脆。
- 即:决策 (b) 的"工具层回退 Rscript 文件路径"**不再实现**(那会引入条件编译/运行时试错复杂度);Docker R 走原生 Code 路径,失败由引擎报错。

### 提示词更新(用户建议 #2)
`data-shaper.md` / `analyst.md` body 加:
> 你现在可以用 `run_code` 工具运行任意 Python/R 脚本。**代码会自动在当前任务的临时隔离目录(`working_dir`)中执行** — 无需 `os.makedirs("/tmp/...")`,直接读写当前目录文件即可。

### 数据流
```
LLM tool call {language:"python", code:"..."}
  → RunCodeTool::execute_with_context(params, ctx)
    → SandboxCommand::code("python", code)
    → ctx.working_dir 是 data worker tmpdir → .with_working_dir(dir)
    → has_container_sandbox()? 用 : warn 不拒仍跑
    → SandboxManager.execute(cmd) → ExecutionResult
  → ToolResult {stdout/stderr/exit_code}
```

### 测试
- 单元:语言白名单熔断(未知语言 → 工具层 Err);默认语言 python;schema 校验。
- 集成(stub sandbox):传 `ctx.working_dir` → `SandboxCommand.with_working_dir` 被调用;R 走 `Rscript -e`;Python 走 `python3 -c`。
- 注册:`register_all_tools` 含 run_code;data worker agent 持有 run_code。
- 端到端(可选,本地有 R/python):Python 计算 + R 出图各一,产物落在 working_dir。

### 风险
- **低**。greenfield 工具,不动 dispatch;working_dir 链路 Sprint 10 已通。框架唯一改动是 `Code` 后端加 R 分支(arg-based 一致)。
- 跨仓库:`Code` 后端在 echo-agent 框架,工具在 echo-tools 框架 crate,注册在应用层 — 都在同一编译图内,无跨仓库合并顺序问题。

### 验收
- 数据 worker 能跑任意 Python/R 脚本;产物落在 worker tmpdir;无沙箱时 warn 但不阻塞。
- `grep run_code echo-tools/src/` 命中;`grep "Rscript" echo-execution/src/sandbox/local.rs` 命中 R 分支。

---

## 三、Sprint 11 — `ExecutionMode::Team` + Team checkpoint/resume

### 目标
1. **接线**(用户三步蓝图 Step 1):新增 `ExecutionMode::Team` 变体,`SubagentExecutor::dispatch` 路由到新 `dispatch_team` 方法;`dispatch_team` 把 `run_id` + `RuntimeStateStore` 灌进 `ManagerWorkerOrchestrator`。
2. **筑基**(Step 2):清死字段 + 用 `TaskNode` 在 3 个状态转换点(plan / per-worker / synthesis)做 checkpoint + skip-on-resume。
3. **验证**(Step 3):端到端中断-恢复测试。

### 范围(file:line,以核实为准)

**框架层(`echo-agent/`)**:
1. `src/agent/subagent/types.rs:15` `ExecutionMode` enum 增 `Team` 变体(无数据,纯 tag)。
2. `src/agent/subagent/types.rs:64` `SubagentDefinition` 增 `pub team: Option<TeamSpec>` 字段(决策 1)。
3. 新类型 `TeamSpec`(同 `types.rs` 或 team 模块):
   ```rust
   pub struct TeamSpec {
       pub strategy: TeamStrategy,         // 复用 team/strategy.rs:7
       pub manager: String,                 // 名称引用(决策 2)
       pub workers: Vec<String>,            // 名称引用其它已注册 subagent
       pub config: TeamConfig,              // 复用 team/mod.rs:41
   }
   ```
   **策略范围说明**:`TeamStrategy` 有 4 变体(ManagerWorker 是 unit,Pipeline/Debate/Swarm 携带内联 agent 名数据)。本 Sprint frontmatter 声明**只支持 `ManagerWorker`**(纯 unit 变体,可经 `team_strategy: manager-worker` 声明);其它 3 策略只能编程构造(它们仍无生产调用者,见"范围外")。`TeamSpec.strategy` 字段类型仍是完整 `TeamStrategy` enum(编程 API 完整),只是 loader 的 frontmatter 解析只认 `manager-worker`。
4. `src/agent/subagent/executor.rs:107 SubagentExecutorConfig` 增 `runtime_state_store: Option<Arc<dyn RuntimeStateStore>>` 字段(+ `Default` 初始化 None)。
5. `src/agent/subagent/executor.rs:290-300` dispatch router 增 `ExecutionMode::Team => self.dispatch_team(&req).await` 分支(决策:只加分支,**绝不改 Sync/Fork/Teammate 既有逻辑**)。
6. 新方法 `dispatch_team(&self, req: &DispatchRequest) -> Result<SubagentResult>`(同 executor.rs):解析 `TeamSpec`、按名称从 registry 解析 manager + workers、构造 `TeamAgent`、传 `run_id`(从 `req.runtime_context`) + store、执行、返回合成字符串封装为 `SubagentResult { mode: Team, .. }`。**超时**:依赖 `TeamAgent::execute` 自身的 `tokio::time::timeout` 包装(`mod.rs:263`,用 `TeamConfig.default_timeout_secs`)— `dispatch_team` **不再二次包超时**,避免双重 timeout。
7. `team/manager_worker.rs` 重构:
   - **删** `max_retries` / `worker_timeout_secs`(决策 3,AGENTS.md 看到就删)。
   - `run()` 签名增 `run_id: Option<&str>` + `store: Option<&dyn RuntimeStateStore>`(决策:store=None → 纯 in-memory 降级)。
   - 三 checkpoint 点 + skip-on-resume(见下"检查点设计")。
8. `team/mod.rs:250 TeamAgent` 增 `run_id: Option<String>` + `state_store: Option<Arc<dyn RuntimeStateStore>>` 字段(builder setter),`execute_inner` 透传给 orchestrator。

**应用层(`echo-agent-cli/echo-agent-app-core/src/`)**:
9. `infra.rs` 主 agent build 时 `builder.subagent_runtime_state_store(...)` 注入 `FileRuntimeStateStore`(镜像 Sprint 8/10 注入 worktree/workspace factory)。
10. `subagent_loader.rs` 增 frontmatter 解析:`team_strategy: manager-worker` + `team_workers: [a, b, c]` + `team_manager: name` → `TeamSpec`。
11. (可选)加 builtin team subagent `.md` 作示例(`src/subagents/coding/team-research.md`,manager=planner, workers=[explorer, summarizer])。

### 关键决策

**(1) `SubagentDefinition.team: Option<TeamSpec>` 字段**(非 enum 变体带数据)
- 保持 `ExecutionMode` 纯 C-like enum(`Display`/`Default`/serde 干净)。
- `TeamSpec` 放 `SubagentDefinition`,与 Sprint 6 的 `.md` 热加载一致(frontmatter 可声明 team 字段)。
- enum 变体带数据(`ExecutionMode::Team(TeamSpec)`)会污染 dispatch router 签名 + 前端配置类型胶水,**不取**。

**(2) Worker 经名称引用其它已注册 subagent**(非嵌套实例)
- `TeamSpec.workers: Vec<String>` 是名字,`dispatch_team` 按名从 `SubagentRegistry` 解析。
- 解耦拓扑:Manager 不强持有 Worker 实例;Worker 自己也是独立 subagent(可独立 store/context)。
- 符合"基于名称的后期绑定"分布式最佳实践。

**(3) 删 `ManagerWorkerOrchestrator.max_retries` / `worker_timeout_secs` 死字段**
- 两者声明从不读(`manager_worker.rs:13, 15`),纯认知噪音。
- 超时由 `TeamAgent::execute` 外层 `tokio::time::timeout`(`mod.rs:263`)+ `SubagentExecutor` dispatch 超时统一管。
- 未来若需 per-worker retry,应在 `TaskNode` 恢复策略统一做,不在 orchestrator 私有字段。

### 检查点设计(对齐 DAG `executor.rs:456-460` skip 模式)

**持久化原语**(已有,复用):
- `RuntimeStateStore` trait(`echo-agent/src/state/mod.rs:154`)+ `FileRuntimeStateStore`(`echo-agent-cli/echo-agent-app-core/src/runtime_state_file.rs:19`,JSON 文件 + atomic rename + Mutex 串行)。
- `TaskNode`(`state/mod.rs:57`):`id`/`name`/`status: TaskNodeStatus`/`outputs: serde_json::Value`。
- `TaskNodeStatus`(`state/mod.rs:26`):`Pending|Running|Success|Failed|Blocked{reason}|Hydrated`,`is_terminal()`=Success|Failed。
- **key = 稳定 `run_id`**(从 `ExternalRunContext.run_id`,`echo-core/src/tools/mod.rs:577`),**不用** `Team.id`(每次 build 重新生成 uuid,`mod.rs:484`)。

**3 个 checkpoint 节点**(决策:plan 带有序索引数组,用户建议 #1 确定性绑定):

| 节点 id | 写入时机 | status | outputs |
|---|---|---|---|
| `team_{run_id}_plan` | `plan_sub_tasks` 后(`manager_worker.rs:55`) | `Success` | `[{"idx":0,"task":"..."},{"idx":1,"task":"..."}]` **有序数组** |
| `team_{run_id}_worker_{idx}` | 每个 worker 完成后(`:133`) | `Success`/`Failed` | worker 结果字符串 |
| `team_{run_id}_synthesis` | `synthesize` 后(`:65`) | `Success` | 最终合成答案 |

**确定性绑定**(用户建议 #1):plan outputs 是 `[{idx, task}]` 有序数组,resume 时按 `idx` 精确匹配 `worker_{idx}` 节点 — 即便 worker 并发完成,结果也能稳定锚定。

**skip-on-resume**(`run()` 入口):
```
1. store.load_nodes(run_id) → 折成 completed: HashMap<&str, TaskNode>
2. if plan 节点 status.is_terminal() && Success → 跳过 plan_sub_tasks,从 outputs 反序列化子任务数组
3. for each (idx, sub_task):
     if worker_{idx} 存在 && status.is_terminal() && Success → 跳过,复用 outputs
     else(不存在 / Running / Failed / Blocked):
        **先 store.save_node 重置为 Pending**(用户 review 补丁 #3 — 覆盖断电/超时卡死的脏 Running/Failed 状态)
        → spawn 该 worker
4. if synthesis status.is_terminal() && Success → 直接返回存储答案(全 fast-path,零 agent 调用)
```

**状态重置防线**(用户 review 补丁 #3):
- 跳过条件**严格** = `node.status.is_terminal() && status == Success`。
- 若节点是 `Running`(断电/超时卡死非终结态)或 `Failed`(偶发网络问题)— **必须先 `save_node` 重置为 `Pending`** 再 spawn,覆盖陈旧脏状态,防进度条逻辑混乱。
- 即:`is_terminal() && !Success`(Failed)与非终结态(Running/Blocked)同样要重置 + 重跑。

**降级**(用户建议 #2):`store = None` → `run()` 跳过所有 checkpoint 读写,纯 in-memory 单次执行(向后兼容现有测试 + 无 store 场景)。

### dispatch_team 实现要点(经 API 核实修正)

**关键 API 现实**(经代码核实,修正了原伪代码):
- `TeamAgent` **不实现 `Agent` trait**,其 `execute(&self, task: &str) -> Result<String, String>`(注意 `String` 错误,非 echo_core 错误)。故 `dispatch_team` **直接调 `team_agent.execute(&req.task).await`**(不经 `execute_agent_streaming`),自己封装 `SubagentResult`。
- `TeamAgentBuilder` **无 `run_id`/`state_store` setter** — 本 Sprint 新增。已有 setter:`manager(name, Box<dyn Agent>, def)` / `worker(name, Box<dyn Agent>, def)` / `strategy` / `timeout_secs`。
- **Arc→Box 转换问题**:`TeamAgentBuilder.manager/worker` 收 `Box<dyn Agent>`,但 `SubagentRegistry::get_agent` 返回 `Arc<dyn Agent>`。`Agent` 非 `Clone`,不能直接 `Box::new(arc.clone())`(类型不符)。**解法**:新建 `ArcAgentBox(Arc<dyn Agent>)` newtype,`impl Agent for ArcAgentBox` 透明委托(所有方法转发给内部 arc)。这是已知的"把共享 agent 装进要 Box 的 API"标准模式。

```rust
async fn dispatch_team(&self, req: &DispatchRequest) -> Result<SubagentResult> {
    let registered = self.registry.get(&req.agent_name).await
        .ok_or_else(|| ReactError::Other(format!("Subagent '{}' not found", req.agent_name)))?;
    let spec = registered.definition.team.as_ref()
        .ok_or_else(|| ReactError::Other("Team mode requested but no TeamSpec on definition".into()))?;

    // 按名称解析 manager + workers(决策 2:名称晚期绑定)
    let manager_def = self.registry.get(&spec.manager).await
        .ok_or_else(|| ReactError::Other(format!("Team manager '{}' not registered", spec.manager)))?
        .definition;
    let manager_agent = self.registry.get_agent(&spec.manager).await
        .ok_or_else(|| ReactError::Other(format!("Cannot get manager agent '{}'", spec.manager)))?;

    let mut builder = TeamAgent::builder()
        .manager(&spec.manager, Box::new(ArcAgentBox(manager_agent.clone())), manager_def)
        .strategy(spec.strategy.clone())
        .run_id(req.runtime_context.as_ref().map(|c| c.run_id.clone()))
        .state_store(self.config.runtime_state_store.clone());
    for name in &spec.workers {
        let w_def = self.registry.get(name).await
            .ok_or_else(|| ReactError::Other(format!("Team worker '{}' not registered", name)))?
            .definition;
        let w_agent = self.registry.get_agent(name).await
            .ok_or_else(|| ReactError::Other(format!("Cannot get worker agent '{}'", name)))?;
        builder = builder.worker(name, Box::new(ArcAgentBox(w_agent.clone())), w_def);
    }
    let team_agent = builder.build();   // TeamAgent::execute 自带 tokio::time::timeout(mod.rs:263)

    let start = Instant::now();
    let result = team_agent.execute(&req.task).await
        .map_err(|e| ReactError::Other(format!("Team execution failed: {e}")))?;
    Ok(SubagentResult {
        agent_name: req.agent_name.clone(),
        output: result,
        duration: start.elapsed(),
        iterations: 1,
        tokens_used: None,      // team 不聚合 worker tokens(留 follow-up)
        was_truncated: false,
        mode: ExecutionMode::Team,
        usage: None,
    })
}
```

**`ArcAgentBox` newtype**(放 `team/` 模块或 subagent 模块,框架层):
```rust
/// 把 `Arc<dyn Agent>` 包成 `Box<dyn Agent>` 能消费的形态。
/// TeamAgentBuilder 的 manager/worker 收 Box<dyn Agent>,但 registry
/// 返回 Arc<dyn Agent>(共享 singleton);Agent 非 Clone,需 newtype 透明委托。
pub struct ArcAgentBox(pub Arc<dyn Agent>);
impl Agent for ArcAgentBox {
    // 所有 trait 方法转发给 self.0(...). 生成宏或手写。
}
```

### 数据流
```
LLM agent_dispatch(agent_name="team-research", task="...")
  → SubagentExecutor::dispatch → mode=Team(来自 definition.execution_mode)
  → dispatch_team(req)
    → 解析 TeamSpec(manager + workers by name)
    → 构造 TeamAgent(run_id + store 注入)
    → TeamAgent::execute(task)
      → tokio::time::timeout 包装 execute_inner
      → ManagerWorker::run(team, task, run_id, store)
        → load_nodes(run_id) → completed set
        → plan(若未完成)→ checkpoint plan 节点
        → fan-out workers(跳过已 Success)→ per-worker checkpoint
        → synthesize(若未完成)→ checkpoint synthesis
        → 返回合成答案
  → SubagentResult { output, mode: Team }
```

### 测试(Step 3 端到端验证)
- **注册 + 路由**:声明 `execution_mode: Team` + `team: Some(TeamSpec)` 的 SubagentDefinition → dispatch 进 `dispatch_team`(stubbed agents)。
- **三 checkpoint 全跑**:manager + 2 workers(stub),断言 plan / 2 worker / synthesis 节点全写入 `Success`。
- **中断-恢复(关键)**:预置 plan `Success` + worker_0 `Success` → 断言只 spawn worker_1,synthesis 合并存储的 + 新结果。
- **状态重置防线**(用户 review 补丁 #3):预置 worker_0 `Running`(模拟断电卡死)+ worker_1 `Failed` → 断言两者都被重置为 Pending 并重跑,不卡死。
- **synthesis 缺失特例**(用户 review 补丁 #4):预置 plan + 全 workers `Success` 但 synthesis **不存在**(模拟合并阶段 LLM 超时)→ 断言 plan/workers 全跳过,只重调一次 manager synthesize。
- **全 fast-path**:预置 synthesis `Success` → 断言零 agent 调用,直接返回存储答案。
- **降级**:store=None → 纯 in-memory,行为等同改造前(无 checkpoint)。
- **frontmatter 加载**:`team_strategy: manager-worker` + `team_workers: [a,b]` → 解析为 TeamSpec。
- **死字段删除回归**:`ManagerWorkerOrchestrator::new()` 不再有 max_retries/worker_timeout_secs 字段。

### 风险 + 缓解
- **中-高**。动 `SubagentExecutor::dispatch` 命脉门控层(AGENTS.md 规则 5:命脉层需 spec 先行 + 新鲜上下文 — 本 spec 即先行)。
- **缓解 1**:`dispatch` 内**只加 `Team` 分支**,绝不改 Sync/Fork/Teammate 既有逻辑(用户防线建议 #1)。
- **缓解 2**:**分阶段实施** — 先把"接线"(Step 1)跑通(声明 Team 的 subagent 能进 dispatch_team 并打印 run_id),再做 checkpoint(TaskNode 状态机)(用户防线建议 #2)。
- **缓解 3**:`ExecutionMode::Team` 是新增,默认配置不声明任何 team → 零行为变更;checkpoint 是 optional(store=None 降级)。
- **缓解 4**:跨仓库 — 框架(echo-agent)先合并,应用(echo-agent-cli)后(infra.rs 注入 store 依赖框架新字段)。

### 范围外(本 Sprint 不做)
- Debate/Swarm/Pipeline 策略仍无生产调用者(只 ManagerWorker 接 checkpoint)。
- per-worker 细粒度 retry(留 TaskNode 恢复策略未来统一做)。
- 富 team 组合 UI/编辑器。
- `security.allow_unsafe_local_code` 配置开关(那是 Sprint 10b 的,YAGNI)。

### 验收
- 声明 `execution_mode: Team` 的 subagent 经 `agent_dispatch` 触发 → 进 `dispatch_team` → 跑完 plan/worker/synthesis。
- 中断后重启同 run_id → 跳过已完成 phase/worker,从 checkpoint 续。
- `grep "max_retries\|worker_timeout_secs" team/manager_worker.rs` 零命中。
- Sync/Fork/Teammate 三模式既有测试全绿(回归)。

---

## 四、实施顺序(两 Sprint 独立,10b 先)

1. **Sprint 10b**(低风险,本窗口可全完成):框架 `Code` 后端加 R + `RunCodeTool` + 注册 + 提示词。验证全绿 → 提交。
2. **Sprint 11**(中-高风险,建议新窗口):**因动命脉门控层**,按 AGENTS.md 规则 5 应在**新鲜上下文**做(重读本 spec + MASTER-PLAN 恢复)。分两子阶段:Step 1 接线 → Step 2 checkpoint。

> **建议**:Sprint 10b 本窗口推进;Sprint 11 提交 10b 后开新窗口(本 spec 已落盘,新窗口读它接续)。

---

## 五、验证规范(两 Sprint 通用,AGENTS.md)

```bash
# echo-agent(根是 package 非 workspace,必须逐 crate)
cd echo-agent && ./scripts/verify-all-crates.sh   # fmt + 逐 crate test + clippy + feature 矩阵

# echo-agent-cli(真 workspace)
cd echo-agent-cli
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo clippy --all-targets -- -D warnings
cargo clean
git -c commit.gpgsign=false commit -m "..."

# 前端
cd echo-agent-cli/web-frontend && npx tsc -b && npm run build
```

跨仓库合并:echo-agent(框架)先,echo-agent-cli(应用)后(应用依赖框架新字段)。提交前两仓库各自 `cargo clean`。

---

## 六、决策记录(防回头路)

- **D-10b-RCE-1**:`run_code` 无沙箱时 warn 不拒(本地个人助理模型,开箱优先);未来 `security.allow_unsafe_local_code` 开关留底。
- **D-10b-R-1**:R 走 `Code` 后端原生支持(arg-based,与所有语言一致);临时文件路径不取。
- **D-10b-stdin-1**:全语言切 stdin 是 cross-cutting 优化,留独立 follow-up;本 Sprint 只加 R,arg-based 一致。
- **D-10b-case-1**:`validate_language` 入口 `.to_lowercase()`,白名单大小写不敏感(防 LLM 输出 `Python`/`PYTHON`)。
- **D-10b-docker-r-1**:Docker R 走原生 Code 路径盲信镜像,失败由引擎报错;不做工具层试错回退(避免条件编译复杂度)。
- **D-11-team-1**:`ExecutionMode` 保持纯 enum(无数据);team 配置放 `SubagentDefinition.team: Option<TeamSpec>`。
- **D-11-team-2**:Worker 经名称引用(晚期绑定),不嵌套实例。
- **D-11-team-3**:删 `ManagerWorkerOrchestrator` 死字段;超时统一外层管。
- **D-11-checkpoint-1**:用 `TaskNode` + `FileRuntimeStateStore`(已有原语),key=稳定 run_id(非 Team.id)。
- **D-11-skip-1**:plan outputs 是有序 `[{idx,task}]` 数组,worker_{idx} 按索引确定性绑定。
- **D-11-reset-1**:skip 条件严格 `is_terminal() && Success`;Running/Failed/Blocked 节点必须先 `save_node` 重置为 Pending 再 spawn(防脏状态)。
- **D-11-synthesis-edge-1**:plan+workers 全成但 synthesis 缺失 → 全跳过只重调 synthesize(必须测试覆盖)。
- **D-11-降级-1**:store=None → in-memory 单次执行,向后兼容。
- **D-11-命脉-1**:`dispatch` 只加 Team 分支,不改 Sync/Fork/Teammate;分阶段(接线先,checkpoint 后)。

---

## 七、参考实现(AGENTS.md "先调研再决策")

- **Claude Code**:`isolation: worktree` + 内置 code execution(子 agent 隔离跑代码)+ plan as artifact(prompt 驱动非状态机)— 对齐 Sprint 10b working_dir + Sprint 11 checkpoint-as-artifact。
- **Codex (OpenAI)**:`codex exec --json` 非交互事件流 + sandbox 权限模型 + task lifecycle skip-on-resume — 对齐 Sprint 11 skip 模式。
- **Cursor / Devin**:plan-then-execute + 文件所有权 + background agent approval gate — Sprint 7-9 已对齐,本 Sprint 不重复。
- **EKO 内部 DAG**(`executor.rs:456-460`):skip-completed-on-resume 是 Sprint 11 直接模板(同 store 模式、同 status-persists-in、同 re-read-at-entry)。

跨系统共性:**LLM 代码经沙箱跑 + 绑工作目录**(10b)+ **多 agent 协同状态用 artifact 持久化 + skip-on-resume**(11)。
