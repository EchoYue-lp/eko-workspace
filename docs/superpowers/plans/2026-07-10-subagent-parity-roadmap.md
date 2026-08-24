# Subagent Parity Roadmap — Implementation Plan

> **Phase 0 status (2026-07-10):** ✅ **代码完成** on `feature/subagent-parity-phase0`（两仓）。
>
> **Phase 0.5 + Phase 1 status (2026-07-10):** ✅ **代码完成** — `agent_tool` GUI 卡片（runtime_context + bridge parent 语义）+ frontmatter `model`/`max_turns`/`is_background` + `general-purpose` + explorer `model:fast` + `SubagentResult.summary`/`artifacts`。TUI 对等仍 Deferred。
>
> **Phase 2 status (2026-07-10):** ✅ **代码完成** — `dispatch_background` + `agent_tool.background` + `DispatchStarted.background` + GUI 卡片/完成回灌主会话；`isolate_worktree` 无 factory 硬失败。延期：TUI background、`prior_summary` resume。
>
> 下一闸：Phase 3（按路线图）或 commit + 手动验收。

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 EKO subagent 对齐 Claude Code / Cursor / Codex 的共识模型——默认 fresh 上下文、主 agent 可即时委派、per-role 配置、结构化摘要回传、background 编排——同时保留 EKO 已有的 worktree/workspace 与 TaskRuntime DAG 优势。

**Architecture:**
- **框架**（`echo-agent`）只扩展通用原语：继承策略、dispatch 默认、`SubagentResult` 结构化字段、background handle。
- **应用**（`echo-agent-cli`）负责：主 agent 挂 `agent_tool`、`.md` frontmatter 扩展、builtin 角色、prompt 目录、TaskRuntime 默认 fresh、UI 回灌。
- **关键语义锁定（勿再混用）：**
  - **Fresh** = 不继承父 system / history / memory（对标 Claude/Cursor 默认）。
  - **Fork inheritance** = 显式继承父上下文切片（对标 Claude `fork`）。
  - **`ExecutionMode::Fork`** = 并发 + worktree/workspace 物理隔离路径（EKO 现状）；**不等于**「必须继承上下文」。
  - Phase 0 默认：TaskRuntime **仍走 `ExecutionMode::Fork`**（保住 implementer worktree / data workspace），但 `parent_context` 用 **fresh inheritance**；`mode: fork` / frontmatter `inherit_*` 才打开继承。

**Tech Stack:** Rust（echo-agent + echo-agent-app-core）、现有 `agent_tool` / `SubagentEvent` / TaskRuntime、前端 `subagentRunStore`（Phase 2）。

**Spec 来源:** 本轮审计（五问 + 对标）+ [eko-subagent-audit canvas](/Users/ls/.cursor/projects/Users-ls-MyWork-code-ylp-agent-learn-lp-agent/canvases/eko-subagent-audit.canvas.tsx)。

**Risk:** HIGH（命脉：dispatch / 主 agent 工具面 / 回传进父上下文）。缓解：Phase 0 只改默认继承与工具注册，不拆 worktree；每 Task 有单测；跨仓先合 `echo-agent` 再合 `echo-agent-cli`。

**验证基线（每个 Phase 结束必跑）:**
```bash
# echo-agent
cd echo-agent && cargo fmt --all && cargo fmt --all -- --check
./scripts/verify-all-crates.sh --quick   # 或全量 verify-all-crates.sh

# echo-agent-cli
cd echo-agent-cli && cargo fmt --all && cargo fmt --all -- --check
cargo test --workspace
cargo check --no-default-features --features gui --bin echo-agent-tauri
```

**提交规则:** 用户明确要求再 commit；`git -c commit.gpgsign=false commit`；提交前 `cargo clean`（AGENTS.md）。

---

## Global Constraints

- EKO 是本地个人助理：不为「防 XSS→RCE」给交互式委派加 `require_full_auto`。
- echo-agent-cli **不**引入 SQLite；对话/摘要用文件或内存。
- TUI / GUI 功能对等：主 agent `agent_tool`、background 回灌两边都要有。
- 框架删改判定：不能因「CLI 没用」删 pub API；新增优先放应用层，确认通用后再下沉。
- UTF-8 安全截断；禁止 `unwrap`/`expect`/字节切片 panic API。
- 过时代码可直接删（`run_isolated` / `ContextBuilder` 在 Phase 3）。

---

## File Structure

| File | Phase | Action | Responsibility |
|---|---|---|---|
| `echo-agent/src/agent/subagent/context.rs` | 0 | Modify | 文档化 fresh vs fork；可选 `fresh_default()` 别名 |
| `echo-agent/src/agent/react/mod.rs` | 0 | Modify | `delegate_to_agent_with_parent_context_*` 默认 fresh inheritance；可选 inherit 参数 |
| `echo-agent/src/tools/builtin/agent_dispatch.rs` | 0/1 | Modify | schema 暴露 roles；默认 mode=sync(fresh)；`fork`=继承 |
| `echo-agent-cli/.../infra.rs` | 0 | Modify | 主 agent `.register_agent_dispatch_tool()`；`TASK_MANAGEMENT_GUIDE` 角色目录 |
| `echo-agent-cli/.../subagent_loader.rs` | 0/1 | Modify | frontmatter：`inherit_history` / `model` / `max_turns` / `is_background` |
| `echo-agent-cli/docs/system-deep-dive/03-subagent.md` | 0 | Modify | 同步语义文档 |
| `echo-agent-cli/.../subagents/coding/general-purpose.md` | 1 | Create | 内置通用 worker |
| `echo-agent-cli/.../infra.rs` + loader | 1 | Modify | per-role model 注入；explorer 默认 fast |
| `echo-agent/.../types.rs` `SubagentResult` | 1 | Modify | `summary` / `artifacts` 字段 |
| `echo-agent-cli/.../task_runtime/executor.rs` | 1 | Modify | 解析结构化回传；父上下文只吃 summary |
| `echo-agent/.../executor.rs` + events | 2 | Modify | background dispatch + completion 事件 |
| `echo-agent-cli` UI stores/components | 2 | Modify | background 线程 + 结果回灌主会话 |
| `echo-agent/.../isolated.rs` + `context_builder.rs` | 3 | Delete | 死路径清理 |
| `echo-agent-cli/echo-agent-eval` 或新 eval | 3 | Create/Modify | token 节省 / 委派准确率评测 |

---

## Phase Gate 总览

| Phase | 交付物 | 可合并条件 |
|---|---|---|
| **0 语义对齐** | fresh 默认 + 主 agent `agent_tool` + prompt 角色表 | 单测绿；手动：Chat 里 `agent_tool` 派 explorer 不污染主上下文 |
| **1 能力对标** | frontmatter model/max_turns/background 标记；general-purpose；结构化 summary | 注册表可读 model；父 LLM 只见 summary |
| **2 编排体验** | 真 background + UI 回灌；多 implementer worktree 强制 | GUI/TUI 都能看后台线程结果 |
| **3 打磨** | 可选 Bash 角色；删死代码；评测基线 | eval 有数字；死路径零引用 |

---

# Phase 0 — 语义对齐（~1 周，Tasks 1–5）

### Task 1: 锁定「Fresh inheritance」API（框架）

**Files:**
- Modify: `echo-agent/src/agent/subagent/context.rs`
- Modify: `echo-agent/src/agent/react/mod.rs`（`delegate_to_agent_with_parent_context_and_cancel` / `_cancel_and_message` / `build_parent_context`）
- Test: `echo-agent/src/agent/subagent/context.rs`（已有 tests 模块）+ `executor.rs` enhance_task tests

**Interfaces:**
- Produces: `ContextInheritance::fresh_default()`（= 今日 `sync_default`）；`build_parent_context_with(inheritance: &ContextInheritance)`
- Consumes: 现有 `ContextInheritance::for_mode`

**设计（写进代码注释）:**
```text
TaskRuntime / 默认委派: ExecutionMode::Fork + ContextInheritance::fresh_default()
显式继承:            ExecutionMode::Fork + ContextInheritance::fork_default()
                      或 agent_tool mode="fork"
```

- [ ] **Step 1: 写失败测试 — fresh 不带 system/history/memory**

在 `context.rs` tests 追加：

```rust
#[test]
fn fresh_default_is_alias_of_sync_default() {
    let a = ContextInheritance::fresh_default();
    let b = ContextInheritance::sync_default();
    assert_eq!(a.inherit_system_prompt, b.inherit_system_prompt);
    assert_eq!(a.inherit_history, b.inherit_history);
    assert_eq!(a.inherit_memory, b.inherit_memory);
}

#[test]
fn from_parent_fresh_yields_empty_inheritable_content() {
    let tools = vec![/* one ToolDefinition "search" */];
    let msgs = vec![/* one user Message */];
    let ctx = SubagentContext::from_parent(
        "PARENT SYSTEM",
        &tools,
        &msgs,
        None,
        &ContextInheritance::fresh_default(),
    );
    assert!(ctx.system_prompt.is_empty());
    assert!(ctx.messages.is_empty());
    assert!(ctx.tool_definitions.is_empty());
    assert!(ctx.store.is_none());
    assert!(!ctx.has_content());
}
```

- [ ] **Step 2: 实现 `fresh_default`**

```rust
/// Claude/Cursor-aligned default: no parent conversation inheritance.
/// Prefer this name in new call sites; `sync_default` remains as the historical alias.
pub fn fresh_default() -> Self {
    Self::sync_default()
}
```

- [ ] **Step 3: 改 `delegate_to_agent_with_parent_context_and_cancel`**

把硬编码：

```rust
let mode = ExecutionMode::Fork;
parent_context: self.build_parent_context(&mode).await,
```

改为：

```rust
let mode = ExecutionMode::Fork; // keep FS isolation path
let inheritance = ContextInheritance::fresh_default();
parent_context: self.build_parent_context_with(&inheritance).await,
```

对 `_cancel_and_message` 做同样修改。

新增：

```rust
async fn build_parent_context_with(
    &self,
    inheritance: &ContextInheritance,
) -> Option<SubagentContext> { /* 现 build_parent_context 体，for_mode 换成传入 inheritance */ }

async fn build_parent_context(&self, mode: &ExecutionMode) -> Option<SubagentContext> {
    self.build_parent_context_with(&ContextInheritance::for_mode(mode)).await
}
```

- [ ] **Step 4: 单测 — enhance_task 在 fresh parent_context=None/empty 时不插入 Inherited System Context**

复用 `enhance_task_no_context_returns_task_unchanged`；再加：

```rust
#[test]
fn enhance_task_empty_fresh_context_leaves_task_alone() {
    let ctx = SubagentContext::empty();
    let out = SubagentExecutor::enhance_task("do thing", Some(&ctx), None);
    assert_eq!(out, "do thing");
}
```

- [ ] **Step 5: 跑框架测试**

```bash
cd echo-agent && cargo test -p echo_agent context::tests fresh_default -- --nocapture
cargo test -p echo_agent enhance_task -- --nocapture
```

Expected: PASS

- [ ] **Step 6:（用户要求时）commit echo-agent**

```bash
git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
fix(subagent): default TaskRuntime delegation to fresh inheritance

Keep ExecutionMode::Fork for worktree/workspace, but stop inheriting
parent system prompt/history/memory unless explicitly requested.
EOF
)"
```

---

### Task 2: `agent_tool` 默认 fresh；`mode=fork` 才继承（框架）

**Files:**
- Modify: `echo-agent/src/tools/builtin/agent_dispatch.rs`
- Test: 同文件或 `echo-agent/src/tools/builtin/` 下现有 agent_dispatch 测试

**Interfaces:**
- Consumes: Task 1 `fresh_default` / `fork_default`
- Produces: `mode` 语义文档化：`sync`|缺省 → fresh；`fork` → fork_default inheritance（仍可走 Fork 执行路径）

- [ ] **Step 1: 调整 execute 里 parent_context 构建**

今日：

```rust
let effective_mode = mode_override.clone().unwrap_or(ExecutionMode::Sync);
let ctx = f.build(&effective_mode).await;
```

改为：

```rust
let inheritance = match mode_override.as_ref() {
    Some(ExecutionMode::Fork) => ContextInheritance::fork_default(),
    _ => ContextInheritance::fresh_default(),
};
// 执行模式：writer 隔离仍需要 Fork 路径 —— 见 Step 2
let ctx = f.build_with_inheritance(&inheritance).await; // 给 ParentContextFactory 加方法
```

给 `ParentContextFactory` 加：

```rust
pub async fn build_with_inheritance(&self, inheritance: &ContextInheritance) -> SubagentContext {
    // 同 build，但 for_mode 换成传入 inheritance
}
pub async fn build(&self, mode: &ExecutionMode) -> SubagentContext {
    self.build_with_inheritance(&ContextInheritance::for_mode(mode)).await
}
```

- [ ] **Step 2: 执行 mode 与继承解耦（最小改动）**

在 `DispatchRequest` 组装处：

```rust
let exec_mode = mode_override.clone().unwrap_or(ExecutionMode::Sync);
// 若目标 definition.isolate_worktree|isolate_workspace == true，强制 ExecutionMode::Fork
// （在 dispatch 前查 registry；查不到则保持 exec_mode）
```

伪代码放在 `AgentDispatchTool::execute`：查 `executor.registry().get(agent_name)`，若 isolate_* 则 `mode_override = Some(Fork)`，但 `parent_context` 仍按用户选的 inheritance（缺省 fresh）。

- [ ] **Step 3: 更新 tool description / parameters.description**

明确写：

```text
mode: optional. Omit or "sync" = fresh context (recommended).
"fork" = inherit parent system prompt + recent messages (use when subagent needs shared background).
Worktree/workspace isolation is automatic for roles that declare it, independent of mode.
```

- [ ] **Step 4: 测试**

```bash
cd echo-agent && cargo test -p echo_agent agent_dispatch -- --nocapture
```

- [ ] **Step 5:（用户要求时）commit**

---

### Task 3: 主 agent 注册 `agent_tool`（应用）

**Files:**
- Modify: `echo-agent-cli/echo-agent-app-core/src/infra.rs`（`create_agent` 主 builder，约 L226–246）
- Test: `echo-agent-cli/echo-agent-app-core` 现有 infra/agent 测试；若无则新增

**Interfaces:**
- Consumes: 框架 `ReactAgentBuilder::register_agent_dispatch_tool`
- Produces: 主 agent 工具列表含 `agent_tool`；与 `task_execute` 并存

- [ ] **Step 1: 改主 builder**

```rust
let mut builder = ReactAgentBuilder::new()
    // ...existing...
    .enable_subagent()
    .register_agent_dispatch_tool()  // NEW — 即时委派
    .enable_human_in_loop()
    // ...
```

确认 `register_default_subagents` 在 builder.build 之后仍会更新 `agent_tool` catalog（已有 catalog_handle 路径）；若 catalog 在注册后才填充，grep `catalog_handle` / `set_catalog` 确保主 agent 也能看到 7 个角色。

- [ ] **Step 2: 写/改测试**

```rust
#[tokio::test]
async fn primary_agent_registers_agent_tool() {
    // 用最小 AppConfig / mock LLM 构建 create_agent
    // assert tool names contain "agent_tool"
    // assert tool names contain "task_execute" when route/task_runtime_store 提供
}
```

若 `create_agent` 难单测，改为测 builder 配置位：抽 `fn primary_builder_flags() -> ...` 或断言 `ReactAgent` 的 `has_tool("agent_tool")`（按现有 API）。

- [ ] **Step 3: 跑应用测试**

```bash
cd echo-agent-cli && cargo test -p echo-agent-app-core primary_agent_registers -- --nocapture
```

- [ ] **Step 4: 手动验收清单（写进 PR 描述）**

1. TUI/GUI 开 Chat，问「用 explorer 查一下 X」→ 应走 `agent_tool`，主会话不见中间 Grep 噪声。
2. Task 模式仍可用 `task_create` + `task_execute`。
3. implementer 任务仍进 worktree（`ObservedIsolation::Worktree`）。

---

### Task 4: 主 prompt 列出 roles + description（应用）

**Files:**
- Modify: `echo-agent-cli/echo-agent-app-core/src/infra.rs`（`TASK_MANAGEMENT_GUIDE` + `create_agent` 组装）
- Modify: `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs`（导出 `format_role_catalog`）

**Interfaces:**
- Produces: `fn format_subagent_catalog(defs: &[WorkerDefinition]) -> String`
- Consumes: `discover_subagents` 结果

- [ ] **Step 1: 实现 catalog 格式化**

```rust
pub fn format_subagent_catalog(defs: &[WorkerDefinition]) -> String {
    let mut out = String::from("\n## Available subagents (agent_tool)\n");
    out.push_str("Use agent_tool for noisy/bounded side work. Prefer task_execute for multi-step DAGs.\n");
    for d in defs {
        let flags = [
            d.readonly.then_some("readonly"),
            d.isolate_worktree.then_some("worktree"),
            d.isolate_workspace.then_some("workspace"),
            d.can_delegate.then_some("can_delegate"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(",");
        out.push_str(&format!(
            "- `{}`: {}{}\n",
            d.name,
            d.description,
            if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags)
            }
        ));
    }
    out
}
```

- [ ] **Step 2: 在 `create_agent` 里，`register_default_subagents` 之后或发现 defs 时，把 catalog append 进 system_prompt**

注意：system_prompt 在 build 前已固定。两种做法（选 A）：

**A（推荐）:** `discover_subagents` 提前调用 → `system_prompt.push_str(&format_subagent_catalog(&defs))` → build → `register_default_subagents` 用同一批 defs（避免发现两次不一致）。

**B:** build 后 `agent.set_system_prompt`（若 API 存在）。优先 A。

- [ ] **Step 3: 收紧 `TASK_MANAGEMENT_GUIDE` 派发段**

把「仅 task_execute 单步」改为：

```text
## 即时委派（agent_tool）
对高噪声、边界清晰的副作用任务，直接 agent_tool(agent_name, task)。
默认 fresh 上下文；需要共享对话背景时 mode=fork。
角色列表见下方 Available subagents。

## 多步编排（task_create + task_execute）
...保留原 DAG 说明...
```

- [ ] **Step 4: 单测 catalog 含 explorer/implementer 且含 description**

```rust
#[test]
fn catalog_lists_builtin_names_and_descriptions() {
    let defs = discover_subagents(None, None);
    let text = format_subagent_catalog(&defs);
    assert!(text.contains("`explorer`"));
    assert!(text.contains("`implementer`"));
    assert!(text.contains("只读") || text.contains("探索") || text.contains("Read"));
}
```

---

### Task 5: 文档对齐 + Phase 0 验收

**Files:**
- Modify: `echo-agent-cli/docs/system-deep-dive/03-subagent.md`
- Modify: `docs/MASTER-PLAN.md`（若存在状态表，记 Phase 0 done）

- [ ] **Step 1: 更新 03-subagent.md**

写清：

| 概念 | 含义 |
|---|---|
| Fresh | 不继承父上下文（默认） |
| Fork inheritance | `mode=fork` / frontmatter inherit |
| ExecutionMode::Fork | 并发 + worktree/workspace |
| agent_tool | 主 agent 即时委派 |
| task_execute | DAG 编排 |

- [ ] **Step 2: 全量验证（两仓）**

按文首验证基线跑通。

- [ ] **Step 3: Phase 0 Done 标准**

- [ ] 默认委派 task 文本 **无** `[Inherited System Context]`
- [ ] 主 agent 有 `agent_tool`
- [ ] system prompt 含角色表
- [ ] implementer 仍 `ObservedIsolation::Worktree`
- [ ] `task_execute` 仍可用

**⛔ 未达 Done 标准不得开 Phase 1。**

---

# Phase 1 — 能力对标（1–2 周，Tasks 6–10）

### Task 6: frontmatter `model` / `max_turns` / `is_background`

**Files:**
- Modify: `echo-agent-cli/.../subagent_loader.rs`（`WorkerFrontmatter` + `WorkerDefinition`）
- Modify: `echo-agent-cli/.../infra.rs`（`register_default_subagents` / `build_*_worker_agent`）
- Modify: `echo-agent/src/agent/subagent/types.rs`（若缺 `is_background` 字段则加；`model`/`max_iterations` 已有）

**Interfaces:**
- Produces:
  ```rust
  pub struct WorkerDefinition {
      // existing...
      pub model: Option<String>,       // None | "inherit" | concrete id
      pub max_turns: Option<usize>,  // → max_iterations
      pub is_background: bool,       // Phase 1 只解析+存；真正调度在 Phase 2
  }
  ```

- [ ] **Step 1: 解析 frontmatter**

```yaml
---
name: explorer
description: "..."
readonly: true
model: fast          # or inherit / concrete
max_turns: 30
is_background: false
---
```

`model: inherit` 或缺省 → `None`（用父模型）。
`model: fast` → 应用层解析为配置里的 fast 模型 id（见 Task 7）。

- [ ] **Step 2: 注册时写入 `SubagentDefinition`**

```rust
SubagentBuilder::new(name, description)
    .model(model_opt)           // if Some
    .max_iterations(max_turns)  // if Some
    // is_background → definition 新字段或 tags 含 "background"
```

- [ ] **Step 3: `build_readonly_worker_agent` / writer 使用 per-role model 字符串构建 `LlmConfig`**

今日全部 `model` 参数来自父。改为：

```rust
let worker_model = def.model.as_deref().unwrap_or(parent_model);
// 用 worker_model 调 build_llm_config / .model(worker_model)
```

- [ ] **Step 4: 测试**

- 解析带 `model`/`max_turns` 的临时 `.md`
- 断言 `WorkerDefinition.model == Some("fast".into())`
- 断言注册后 `SubagentDefinition.max_iterations == Some(30)`

---

### Task 7: 内置 `general-purpose` + explorer 默认 fast model

**Files:**
- Create: `echo-agent-cli/echo-agent-app-core/src/subagents/coding/general-purpose.md`
- Modify: `BUILTIN_WORKER_FILES` in `subagent_loader.rs`
- Modify: `explorer.md` frontmatter `model: fast`
- Modify: `infra.rs` — 解析 `fast` → 实际 model id（从 `AppConfig` / runtime model 的 fast 变体；若无独立 fast，则用当前 model 的 cheaper alias 或文档约定 env `EKO_FAST_MODEL`）

**general-purpose.md 初稿：**

```markdown
---
name: general-purpose
description: "通用多步任务：需同时探索与修改、或无法归入专精角色时使用。"
readonly: false
worktree: true
tags: ["general"]
---

你是 EKO 的通用 subagent。在独立上下文完成指派任务，返回简洁结论与证据路径。
不要修改全局 plan；需要后续工作请输出 suggested_tasks。
```

- [ ] **Step 1: 加入 builtin 列表并单测 `discover_subagents` 含 `general-purpose`**
- [ ] **Step 2: explorer.md 加 `model: fast`；测解析**
- [ ] **Step 3: `role_for_kind` / profiles 不强制改（general 由主 agent 经 agent_tool 选用）**
- [ ] **Step 4: 更新 03-subagent.md 角色表为 8 张**

---

### Task 8: 结构化回传 `summary` + `artifacts` + `suggested_tasks`

**Files:**
- Modify: `echo-agent/src/agent/subagent/types.rs` — `SubagentResult`
- Modify: `echo-agent/src/agent/subagent/executor.rs` — 从最终文本解析或由约定块填充
- Modify: `echo-agent-cli/.../task_runtime/executor.rs` — `build_task_prompt` 契约 + 父上下文只存 summary
- Modify: `echo-agent/src/tools/builtin/agent_dispatch.rs` — 工具返回 summary（非全文）

**Interfaces:**
- Produces:
  ```rust
  pub struct SubagentResult {
      pub output: String,           // 保留全文（UI/存储）
      pub summary: String,          // 给父 LLM；空则 UTF-8 安全截断 output
      pub artifacts: Vec<String>,   // 路径列表
      // existing fields...
  }
  ```

**回传约定（写入 worker prompt）：**

```text
## Return format
1) Write a short SUMMARY (≤ 1200 chars) under heading `## Summary`
2) Optionally `## Artifacts` as bullet paths
3) Optionally fenced JSON suggested_tasks (existing)
Everything else may be detailed notes; the parent only receives Summary (+ suggested_tasks).
```

- [ ] **Step 1: 框架解析 helper（UTF-8 安全）**

```rust
pub fn split_subagent_output(raw: &str) -> (String /*summary*/, Vec<String> /*artifacts*/) {
    // 找 ## Summary ... 到下一 ## 或 EOF
    // 找 ## Artifacts 列表
    // 若无 Summary：chars().take(1200).collect()
}
```

- [ ] **Step 2: `agent_tool` 成功时 `ToolResult::success(result.summary)`**（非 `result.output`）
- [ ] **Step 3: TaskRuntime `put_summary` 的 `completed_work` 用 summary；全文可另存字段或 debug 日志**
- [ ] **Step 4: 单测截断中文不 panic；有 Summary 标题时优先提取**

---

### Task 9: Phase 1 文档 + 验证门

- [ ] frontmatter 字段表写入 03-subagent.md
- [ ] 两仓 verify
- [ ] Done 标准：explorer 可用 fast；general-purpose 可派；父 LLM 侧字符串为 summary

**⛔ 未达不得开 Phase 2。**

---

# Phase 2 — 编排体验（~2 周，Tasks 10–13）

### Task 10: Background dispatch 原语（框架）

**Files:**
- Modify: `echo-agent/src/agent/subagent/types.rs` — `is_background` on definition（若 Task 6 已加则跳过）
- Modify: `echo-agent/src/agent/subagent/executor.rs` — `dispatch_background` → 立即返回 handle
- Modify: `echo-agent/src/agent/subagent/events.rs` — `DispatchCompleted` 已有；确保 background 也发

**Interfaces:**
```rust
pub struct BackgroundSubagentHandle {
    pub execution_id: String,
    pub agent_name: String,
    // oneshot/watch for completion optional
}

impl SubagentExecutor {
    pub async fn dispatch_background(&self, req: DispatchRequest) -> Result<BackgroundSubagentHandle>;
}
```

- [x] **Step 1: 单测 — dispatch_background 在 worker 跑完前返回 handle**
- [x] **Step 2: 完成后仍发 `SubagentEvent::DispatchCompleted`**
- [x] **Step 3: `agent_tool` 参数 `background: bool`（或读 definition.is_background）**

---

### Task 11: 结果回灌主会话（应用 + UI）

**Files:**
- Modify: `echo-agent-cli/src/tauri/mod.rs`（execution://event 已有）
- Modify: `web-frontend/src/stores/subagentRunStore.ts`
- Modify: chat 驱动 — background 完成后把 **summary** 注入主会话为 tool result 或 system note（对标 Cursor background）
- TUI：对等事件渲染

- [x] **Step 1: 协议 — `execution://event` 增加 `background: true` 字段（若尚无）**
- [x] **Step 2: GUI 右栏显示 Running → Completed；Completed 时主 chat 出现一条「subagent X finished: {summary}」**
- [ ] **Step 3: TUI 对等**（延期）
- [ ] **Step 4: 手动验收清单**（见下方提示词；需 GUI 实机）

---

### Task 12: 可选 resume/follow-up（最小可用）

**范围控制（YAGNI）：** 不做完整会话持久化编辑器。只做：

- 同一 `execution_id` 结束后，主 agent 可再 `agent_tool` 带 `resume_execution_id`（可选）把 **上一次 summary + 新 task** 拼成新 prompt 再派同角色（新 fresh 上下文，但带 prior summary）。

- [ ] **Step 1: schema 可选字段 `prior_summary`**
- [ ] **Step 2: 文档说明「非同一 ContextManager，而是摘要续聊」**
- [ ] **Step 3: 单测 prompt 含 prior_summary**

若两周不够：**降级为文档化延期**，不阻塞 Task 13。

---

### Task 13: 多 implementer 强制 worktree

**Files:**
- Modify: `echo-agent-cli/.../task_runtime/executor.rs` — 并行 writer wave
- Modify: `echo-agent/.../executor.rs` — 已有 isolate_worktree；补：并行两个 worktree writer 不得同 path

- [x] **Step 1: 断言 `implementer.md` 保持 `worktree: true`**
- [x] **Step 2: 并行两个 Implementation task 时，若任一缺少 worktree factory → fail 该 task（不静默共写主树）**
- [x] **Step 3: 集成测试或单元测试 mock factory 分配不同 path**

**Phase 2 Done 标准:** background 可跑；UI 可见；多 writer 不共写主 worktree。 ✅（TUI / 手动 GUI 验收除外）

---

# Phase 3 — 打磨（持续，Tasks 14–16）

### Task 14: 按需 Bash/Browser 噪声隔离角色

**仅当** 主会话实测被 shell/browser 输出污染时再做。

- Create: `subagents/coding/shell-runner.md`（readonly:false，强约束「只跑命令、回传摘要」）
- Browser：若已有 MCP browser，做 `browser-agent.md` 限制工具面
- 不提前实现「完整 Playwright agent」

### Task 15: 删死路径

**Files:**
- Delete or gut: `echo-agent/src/agent/subagent/isolated.rs`（`run_isolated` 无调用方）
- Delete: `context_builder.rs`（若确认仅单测）
- Remove: `SubagentKind::Plugin` 或实现最小 loader（二选一；默认删占位）
- Grep 全仓确认零引用；更新 docs

```bash
rg -n "run_isolated|ContextBuilder|SubagentKind::Plugin" echo-agent echo-agent-cli
```

### Task 16: 评测基线

**Files:**
- `echo-agent-cli/echo-agent-eval` 或 `docs/evals/subagent-parity.md` + 脚本

指标（最小）：

| 指标 | 测法 |
|---|---|
| 主上下文 token 节省 | 同任务：直做 Grep vs agent_tool explorer；比父上下文 message 字节/字符数 |
| 委派准确率 | 10 条固定 prompt，期望角色命中率 |
| 并行吞吐 | 3 个只读 task fan-out 墙钟时间 |

- [ ] 产出一张表写入 MASTER-PLAN / eval 报告
- [ ] 不把评测当 CI 红线（先人工跑）

---

## 跨仓合并顺序

```text
1) echo-agent: Task 1–2, 8(框架部分), 10, 15
2) echo-agent-cli: Task 3–7, 8(应用), 9, 11–14, 16
```

worktree 开发时临时 path 指向本地 echo-agent；**合 main 前改回相对 path**（AGENTS.md）。

---

## Self-Review（写作时已检）

| Spec 项 | 对应 Task |
|---|---|
| 默认 Sync/fresh | Task 1–2（语义=fresh；执行仍 Fork 保 worktree） |
| 主 agent agent_tool | Task 3 |
| prompt 角色表 | Task 4 |
| frontmatter model/max_turns/is_background | Task 6 |
| general-purpose + explorer fast | Task 7 |
| 结构化 summary/artifacts/suggested_tasks | Task 8 |
| background + 回灌 | Task 10–11 |
| resume/follow-up | Task 12（可降级） |
| 多 implementer worktree | Task 13 |
| Bash/Browser | Task 14（按需） |
| 删死路径 | Task 15 |
| 评测 | Task 16 |

**刻意修正相对原 Phase 0 表述:** 「默认隔离改为 Sync」落实为 **默认 Fresh inheritance + 保留 Fork 执行路径**，避免 implementer 丢失 worktree。

---

## 执行方式

Plan 已保存到 `docs/superpowers/plans/2026-07-10-subagent-parity-roadmap.md`。

**两种执行选项：**

1. **Subagent-Driven（推荐）** — 每 Task 派独立 subagent，Task 间人工/父 agent review
2. **Inline Execution** — 本会话按 Task 连续推进，Phase 门控处停顿验收

选哪个？建议先只跑 **Phase 0（Tasks 1–5）**，合入后再开 Phase 1。

---

## Deferred（不插入主线）

### TUI 对等优化（用户 2026-07-10 拍板延期）

对标 Claude Code / Codex + EKO GUI。已知缺口与 Approach A 设计已调研，**不在 Phase 0–3 主线插入**，避免打乱 subagent parity 节奏。

已知缺口：
1. 工具参数 `Value::to_string()` 双重转义不可读
2. AssistantTurn 默认折叠且无展开 handler，最终答案看不见
3. `worker_trace_sink=None`，无 subagent 摘要条

落地时最小文件集：`src/tui/events.rs`、`src/tui/mod.rs`、`src/tui/widgets/chat.rs`、新 `widgets/subagent.rs`。单独 milestone / Phase 闸后开窗执行。
