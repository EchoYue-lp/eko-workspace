# Sprint 11: `ExecutionMode::Team` + Checkpoint/Resume Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the `TeamAgent` (multi-agent ManagerWorker strategy) into the real dispatch path via a new `ExecutionMode::Team` variant, and add checkpoint/resume persistence so a team run that times out can restart by skipping already-completed plan/worker/synthesis phases — mirroring the DAG's skip-completed-on-retry pattern.

**Architecture:** Two-phase. **Phase A (wiring):** add `ExecutionMode::Team` + `SubagentDefinition.team: Option<TeamSpec>`; route in `SubagentExecutor::dispatch` to a new `dispatch_team` that builds a `TeamAgent` from name-resolved registered subagents (via an `ArcAgentBox` adapter since the builder wants `Box<dyn Agent>` but the registry returns `Arc<dyn Agent>`), runs it with `run_id` + `RuntimeStateStore` plumbed in. **Phase B (checkpoint):** `ManagerWorkerOrchestrator::run` reads prior `TaskNode`s at entry (skip Success, reset-and-rerun Running/Failed), writes 3 checkpoint nodes (plan/worker/synthesis), degrades to in-memory when `store=None`.

**Tech Stack:** Rust 2024. Framework crates `echo_core` (Agent trait, RuntimeStateStore, TaskNode), `echo_agent` (subagent executor, team module, state). App crate `echo-agent-app-core` (FileRuntimeStateStore injection, subagent_loader). No new deps.

**Spec:** `docs/superpowers/specs/2026-07-01-sprint-10b-and-11-design.md` §三.

**Risk:** HIGH — touches `SubagentExecutor::dispatch` (命脉门控层, AGENTS.md rule 5). Mitigation: only ADD a `Team` branch (never touch Sync/Fork/Teammate existing logic); checkpoint is optional (`store=None` degrades to today's behavior); all new types default-off (no team is declared unless a `.md`/registration explicitly creates one).

**Verified API facts (from code reconnaissance):**
- `Agent` trait (`echo-core/src/agent/mod.rs:331`): 4 required methods (`name`, `model_name`, `system_prompt`, `execute`); the rest have defaults → `ArcAgentBox` only needs those 4.
- `TeamAgent::execute(&self, task) -> Result<String, String>` (NOT echo_core Result; team is not an Agent) — `team/mod.rs:263`. `dispatch_team` calls it directly.
- `TeamAgentBuilder` (`team/mod.rs:390`): setters `manager(name, Box<dyn Agent>, SubagentDefinition)`, `worker(...)`, `strategy`, `timeout_secs`. NO `run_id`/`state_store` yet → we add them.
- `SubagentRegistry::get(name) -> Option<RegisteredSubagent{definition, has_instance}>` (async); `get_agent(name) -> Option<Arc<dyn Agent>>` (async).
- `RuntimeStateStore` (`state/mod.rs:154`): `save_node(conv_id, &TaskNode)`, `load_nodes(conv_id) -> Result<Vec<TaskNode>>`, `update_status(conv_id, node_id, TaskNodeStatus)`. All async `BoxFuture`.
- `TaskNode::new(id, name)` + `.with_status(...)`, `.with_outputs(Value)`. `Clone`. `TaskNodeStatus::is_terminal()` = Success|Failed.
- `ExternalRunContext { run_id: String, cancel, trace_sink }` (no Default; `run_id` mandatory).
- `SubagentResult` 8 fields, no Default; struct literal or `sync_result`/`fork_result` + mutate `.mode`.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `echo-agent/src/agent/subagent/team/agent_box.rs` | Create | `ArcAgentBox` newtype (Arc<dyn Agent> → impl Agent adapter) |
| `echo-agent/src/agent/subagent/team/mod.rs` | Modify | Add `run_id`/`state_store` to TeamAgent + builder; pass to orchestrator; re-export ArcAgentBox |
| `echo-agent/src/agent/subagent/team/manager_worker.rs` | Modify | Delete dead fields; checkpoint read/write/skip-on-resume in `run()` |
| `echo-agent/src/agent/subagent/types.rs` | Modify | Add `ExecutionMode::Team` + `TeamSpec` + `SubagentDefinition.team` field |
| `echo-agent/src/agent/subagent/executor.rs` | Modify | Add `dispatch_team` + router branch + `SubagentExecutorConfig.runtime_state_store` |
| `echo-agent/src/agent/subagent/builder.rs` | Modify | Add `.team(spec)` builder method on SubagentBuilder (for programmatic team registration) |
| `echo-agent-cli/echo-agent-app-core/src/infra.rs` | Modify | Inject `FileRuntimeStateStore` via builder (mirror worktree/workspace wiring) |
| `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs` | Modify | Parse `team_strategy`/`team_manager`/`team_workers` frontmatter → TeamSpec |
| `echo-agent-cli/echo-agent-app-core/src/subagents/coding/team-research.md` | Create (optional example) | Builtin team subagent example |

---

## Phase A — Wiring (Tasks 1-4)

### Task 1: `ArcAgentBox` adapter + delete dead `ManagerWorkerOrchestrator` fields

**Files:**
- Create: `echo-agent/src/agent/subagent/team/agent_box.rs`
- Modify: `echo-agent/src/agent/subagent/team/mod.rs` (module declaration + re-export)
- Modify: `echo-agent/src/agent/subagent/team/manager_worker.rs` (delete dead fields)

- [ ] **Step 1: Create `agent_box.rs`**

```rust
//! Adapter wrapping `Arc<dyn Agent>` as an `impl Agent`.
//!
//! `TeamAgentBuilder::manager`/`worker` consume `Box<dyn Agent>`, but
//! `SubagentRegistry::get_agent` returns `Arc<dyn Agent>` (a shared singleton
//! that may be used by multiple dispatch paths). The `Agent` trait is not
//! `Clone`, so a raw `Box::new(arc)` won't typecheck. This newtype transparently
//! delegates the 4 required trait methods to the inner `Arc`, letting a shared
//! agent be fed into the team builder.

use std::sync::Arc;
use echo_core::agent::Agent;
use echo_core::error::Result;
use echo_core::llm::types::Message;
use echo_core::tools::{ToolDefinition, ExternalRunContext};
use futures::future::BoxFuture;
use futures::stream::BoxStream;

pub struct ArcAgentBox(pub Arc<dyn Agent>);

impl Agent for ArcAgentBox {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn model_name(&self) -> &str {
        self.0.model_name()
    }
    fn system_prompt(&self) -> &str {
        self.0.system_prompt()
    }
    fn execute<'a>(&'a self, task: &'a str) -> BoxFuture<'a, Result<String>> {
        self.0.execute(task)
    }
    // All other trait methods (tool_names, execute_stream, set_working_dir,
    // etc.) use their default impls — we only need the 4 required ones.
}
```

NOTE: verify the exact import paths before finalizing — `ToolDefinition`/`Message`/`ExternalRunContext` may not be needed if no method signature references them in the required 4. Check `echo-core/src/agent/mod.rs` for the precise `execute` signature and trim unused imports (clippy will flag them). If `execute_stream` etc. are required (not default), implement them too — but per reconnaissance they have defaults.

- [ ] **Step 2: Declare the module + re-export in `team/mod.rs`**

Near the top of `echo-agent/src/agent/subagent/team/mod.rs`, after existing `mod` declarations:

```rust
pub mod agent_box;
pub use agent_box::ArcAgentBox;
```

- [ ] **Step 3: Delete dead fields from `manager_worker.rs`**

In `echo-agent/src/agent/subagent/team/manager_worker.rs`:
- Delete the `max_retries: u32` and `worker_timeout_secs: u64` fields from the struct (lines ~13-15).
- Delete the `impl Default` block (which only initializes those two fields).
- Change `ManagerWorkerOrchestrator::new()` to construct an empty struct: `pub fn new() -> Self { Self {} }`. (If the struct now has zero fields, `new()` returns `Self {}`.)
- Update the existing test `test_orchestrator_defaults` (lines ~192-198) — it asserts on the deleted fields, so DELETE that test (the fields no longer exist; AGENTS.md "看到就删").
- Update `test_team_strategy_default` if it references the orchestrator's fields (it doesn't, per reconnaissance — it only checks `TeamStrategy::default()`).

The struct becomes:
```rust
/// Orchestrates a team using the Manager-Worker pattern.
///
/// Stateless except for the checkpoint store passed into `run()`. Sprint 11
/// removed the dead `max_retries`/`worker_timeout_secs` fields (declared but
/// never read; timeouts come from the outer `TeamAgent::execute` wrapper and
/// `SubagentExecutor` dispatch timeout).
pub struct ManagerWorkerOrchestrator;

impl ManagerWorkerOrchestrator {
    pub fn new() -> Self {
        Self
    }
    // run() will be rewritten in Task 4.
}
```

(Rust allows unit-like structs with no fields written as `pub struct Foo;` or `pub struct Foo {}`. Use `pub struct ManagerWorkerOrchestrator;` and `Self`.)

- [ ] **Step 4: Compile + run team module tests**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_agent --lib agent::subagent::team
```
Expected: PASS (the remaining `test_team_strategy_default` still passes; `test_orchestrator_defaults` is deleted). If `run()` is now referenced but not yet rewritten, temporarily keep a stub `run()` returning `Err("not implemented".into())` — Task 4 rewrites it. Actually `run()` is called from `mod.rs:274`; to keep the crate compiling across tasks, leave the existing `run()` body intact for now (Task 4 rewrites it). Only delete the dead FIELDS + Default + the one test.

- [ ] **Step 5: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo fmt --all && cargo fmt --all -- --check
git add src/agent/subagent/team/agent_box.rs src/agent/subagent/team/mod.rs src/agent/subagent/team/manager_worker.rs
git -c commit.gpgsign=false commit -m "feat(team): ArcAgentBox 适配器 + 删 ManagerWorker 死字段

Sprint 11 Task 1: 框架接线地基。
- agent_box.rs: ArcAgentBox(Arc<dyn Agent>) newtype,impl Agent 透明委托 4 个
  required 方法(name/model_name/system_prompt/execute)。解 TeamAgentBuilder
  收 Box<dyn Agent> 但 registry 返回 Arc<dyn Agent> 的转换问题。
- manager_worker.rs: 删 max_retries/worker_timeout_secs 死字段(声明从不读,
  AGENTS.md 看到就删)。超时由 TeamAgent::execute 外层 timeout 统一管。
  Default impl + test_orchestrator_defaults 一并删。run() 签名 Task 4 重写。"
```

---

### Task 2: Add `ExecutionMode::Team` + `TeamSpec` + `SubagentDefinition.team`

**Files:**
- Modify: `echo-agent/src/agent/subagent/types.rs`

- [ ] **Step 1: Add `Team` variant to `ExecutionMode`**

In `echo-agent/src/agent/subagent/types.rs`, find the `ExecutionMode` enum (~line 15):

```rust
pub enum ExecutionMode {
    Sync,
    Fork,
    Teammate,
}
```

Add `Team`:
```rust
pub enum ExecutionMode {
    Sync,
    Fork,
    Teammate,
    /// Sprint 11: multi-agent team dispatch. Routes through `dispatch_team`
    /// which builds a `TeamAgent` from the `TeamSpec` on the definition.
    /// Unlike `Teammate` (single async agent poll), `Team` runs the full
    /// ManagerWorker plan→fan-out→synthesize pipeline.
    Team,
}
```

Update the `Display` impl (same file, ~line 26-33) to add:
```rust
            ExecutionMode::Team => write!(f, "team"),
```

- [ ] **Step 2: Add `TeamSpec` struct**

In the same file, add near `SubagentDefinition`:

```rust
/// Specification for a team-mode subagent (Sprint 11).
///
/// Carried on `SubagentDefinition.team`. The manager + workers are referenced
/// **by name** (late binding) — `dispatch_team` resolves them from the
/// `SubagentRegistry` at dispatch time. This decouples team topology from
/// instance lifetimes: each member is itself a normal registered subagent.
///
/// Only `TeamStrategy::ManagerWorker` is frontmatter-declarable (it's a unit
/// variant); `Pipeline`/`Debate`/`Swarm` carry inline agent-name data and are
/// programmatic-only.
#[derive(Debug, Clone)]
pub struct TeamSpec {
    /// Strategy (typically `ManagerWorker`; others are programmatic-only).
    pub strategy: crate::agent::subagent::team::strategy::TeamStrategy,
    /// Manager/leader subagent name (must be separately registered).
    pub manager: String,
    /// Worker subagent names (must each be separately registered).
    pub workers: Vec<String>,
    /// Team runtime config (concurrency, timeout, etc.). Reuse TeamConfig.
    pub config: crate::agent::subagent::team::TeamConfig,
}
```

NOTE: confirm `TeamStrategy` and `TeamConfig` are `pub` and `Clone` (reconnaissance says `TeamConfig` has public fields; `TeamStrategy` has variants — check `Clone` derive). If `TeamStrategy` is not `Clone`, either add `#[derive(Clone)]` to it (in `team/strategy.rs`) or wrap as `Arc`. Check before finalizing.

- [ ] **Step 3: Add `team` field to `SubagentDefinition`**

In `SubagentDefinition` (~line 64), after `isolate_workspace: bool` (~line 122):

```rust
    /// Sprint 11: team-mode specification. When `Some` AND
    /// `execution_mode == Team`, `dispatch_team` uses this to build the
    /// TeamAgent. `None` for normal Sync/Fork/Teammate subagents.
    pub team: Option<TeamSpec>,
```

- [ ] **Step 4: Update `SubagentDefinition::new`**

In the `new()` constructor (~line 131), add `team: None,` to the struct literal.

- [ ] **Step 5: Update ALL other `SubagentDefinition { ... }` literal constructions**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
grep -rn "SubagentDefinition {" src/ echo-core/ echo-execution/ echo-tools/ echo-state/ echo-orchestration/ echo-integration/ 2>/dev/null
```
Add `team: None,` to every struct literal that doesn't already have it (tests, builders, etc.). The compiler will flag each missing one.

- [ ] **Step 6: Compile + test**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo build -p echo_agent 2>&1 | grep "error\[" | head
```
Fix each "missing field `team`" by adding `team: None,`. Then:
```bash
cargo test -p echo_agent --lib
```
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo fmt --all && cargo fmt --all -- --check
git add src/agent/subagent/types.rs  # + any test files touched
git -c commit.gpgsign=false commit -m "feat(subagent): ExecutionMode::Team + TeamSpec + Definition.team 字段

Sprint 11 Task 2: 类型扩展(纯增量,零行为变更 — Team 模式无调用者前不激活)。
- ExecutionMode 加 Team 变体(纯 tag,不带数据;D-11-team-1)。
- TeamSpec { strategy, manager, workers(by name), config }(D-11-team-2 名称引用)。
- SubagentDefinition.team: Option<TeamSpec>(默认 None,不影响现有 subagent)。
- frontmatter 声明只支持 ManagerWorker 策略(unit 变体);其余编程构造。"
```

---

### Task 3: Add `run_id`/`state_store` to TeamAgent + builder; plumb into orchestrator

**Files:**
- Modify: `echo-agent/src/agent/subagent/team/mod.rs`

- [ ] **Step 1: Add fields to `TeamAgent` + new builder setters**

In `team/mod.rs`, the `TeamAgent` struct (~line 250):
```rust
pub struct TeamAgent {
    pub team: Team,
    pub strategy: strategy::TeamStrategy,
}
```
Add:
```rust
pub struct TeamAgent {
    pub team: Team,
    pub strategy: strategy::TeamStrategy,
    /// Sprint 11: stable run_id for keying checkpoints (NOT Team.id which
    /// regenerates per build). None → in-memory, no persistence.
    pub run_id: Option<String>,
    /// Sprint 11: optional state store for checkpoint/resume. None → degrade.
    pub state_store: Option<std::sync::Arc<dyn echo_core::state::RuntimeStateStore>>,
}
```

Update `TeamAgent::new` (~line 257) to initialize both to `None`.

Update `execute_inner` ManagerWorker arm (~line 272-275) to pass them:
```rust
            strategy::TeamStrategy::ManagerWorker => {
                let orch = manager_worker::ManagerWorkerOrchestrator::new();
                orch.run(&self.team, task, self.run_id.as_deref(), self.state_store.as_deref()).await
            }
```

In `TeamAgentBuilder` (~line 390), add fields + setters:
```rust
    run_id: Option<String>,
    state_store: Option<std::sync::Arc<dyn echo_core::state::RuntimeStateStore>>,
```
Initialize both to `None` in `new()`. Add setters:
```rust
    pub fn run_id(mut self, run_id: Option<String>) -> Self {
        self.run_id = run_id;
        self
    }
    pub fn state_store(mut self, store: Option<std::sync::Arc<dyn echo_core::state::RuntimeStateStore>>) -> Self {
        self.state_store = store;
        self
    }
```
In `build()` (~line 475-501), pass both into the constructed `TeamAgent`.

- [ ] **Step 2: Temporarily stub the new `run()` signature in manager_worker.rs**

In `manager_worker.rs`, change `run()` signature to accept the new args (Task 4 fills the body):
```rust
    pub async fn run(
        &self,
        team: &Team,
        task: &str,
        run_id: Option<&str>,
        store: Option<&dyn echo_core::state::RuntimeStateStore>,
    ) -> Result<String, String> {
        // Task 4 rewrites this with checkpoint logic. For now, ignore
        // run_id/store and run the existing single-pass logic.
        let _ = (run_id, store);
        // ... existing body unchanged ...
    }
```
Keep the existing body intact (Task 4 rewrites it).

- [ ] **Step 3: Compile + test**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_agent --lib agent::subagent::team
```
Expected: PASS (behavior unchanged — run_id/store ignored).

- [ ] **Step 4: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo fmt --all && cargo fmt --all -- --check
git add src/agent/subagent/team/mod.rs src/agent/subagent/team/manager_worker.rs
git -c commit.gpgsign=false commit -m "feat(team): TeamAgent + builder 增 run_id/state_store(透传给 orchestrator)

Sprint 11 Task 3: 透传链路(行为暂不变 — run() 仍跑原逻辑,Task 4 加 checkpoint)。
- TeamAgent 增 run_id: Option<String>(稳定 key,非 Team.id) + state_store。
- TeamAgentBuilder 增 run_id()/state_store() setter。
- execute_inner ManagerWorker 臂透传 run_id.as_deref() + state_store.as_deref()
  给 orchestrator.run()。
- run() 签名增两参(暂忽略,Task 4 实现 checkpoint 读写)。"
```

---

### Task 4: Checkpoint/resume in `ManagerWorkerOrchestrator::run`

**Files:**
- Modify: `echo-agent/src/agent/subagent/team/manager_worker.rs`

This is the core logic task. Implement skip-on-resume + 3 checkpoint writes, per spec §三 "检查点设计" + user review patches #1 (deterministic idx binding) + #3 (state reset defense).

- [ ] **Step 1: Rewrite `run()` with checkpoint logic**

Replace the `run()` body in `manager_worker.rs`:

```rust
    pub async fn run(
        &self,
        team: &Team,
        task: &str,
        run_id: Option<&str>,
        store: Option<&dyn echo_core::state::RuntimeStateStore>,
    ) -> Result<String, String> {
        let manager_name = team.leader_name().ok_or("No leader in team")?;
        let workers: Vec<&TeamMember> = team.workers().collect();
        if workers.is_empty() {
            return Err("No workers in team".into());
        }

        // ── Skip-on-resume: load prior checkpoint nodes (DAG pattern) ──
        // Only active when both run_id and store are present; else in-memory.
        let prior_nodes: std::collections::HashMap<String, echo_core::state::TaskNode> =
            if let (Some(rid), Some(st)) = (run_id, store) {
                st.load_nodes(rid).await.unwrap_or_default()
                    .into_iter()
                    .map(|n| (n.id.clone(), n))
                    .collect()
            } else {
                Default::default()
            };

        // Fast-path: if synthesis already Success, return stored answer.
        if let Some(rid) = run_id {
            let synth_id = synth_node_id(rid);
            if let Some(node) = prior_nodes.get(&synth_id) {
                if node.status == echo_core::state::TaskNodeStatus::Success {
                    if let Some(ans) = node.outputs.as_str() {
                        debug!("Team fast-path: returning stored synthesis (no agent calls)");
                        return Ok(ans.to_string());
                    }
                }
            }
        }

        info!(team = %team.name, manager = %manager_name, worker_count = workers.len(), "Starting Manager-Worker execution");

        // ── Phase 1: plan (skip if prior Success) ──
        let sub_tasks: Vec<String> = if let Some(rid) = run_id {
            let plan_id = plan_node_id(rid);
            if let Some(node) = prior_nodes.get(&plan_id) {
                if node.status == echo_core::state::TaskNodeStatus::Success {
                    // Reuse stored plan (deterministic idx binding, user patch #1).
                    if let Some(arr) = node.outputs.as_array() {
                        let reused: Vec<String> = arr.iter()
                            .filter_map(|v| v.get("task").and_then(|t| t.as_str()).map(String::from))
                            .collect();
                        if !reused.is_empty() {
                            debug!(count = reused.len(), "Reusing stored plan");
                            reused
                        } else {
                            self.plan_sub_tasks(team, manager_name, task).await?
                        }
                    } else {
                        self.plan_sub_tasks(team, manager_name, task).await?
                    }
                } else {
                    self.plan_sub_tasks(team, manager_name, task).await?
                }
            } else {
                self.plan_sub_tasks(team, manager_name, task).await?
            }
        } else {
            self.plan_sub_tasks(team, manager_name, task).await?
        };

        // Checkpoint: write plan node (ordered [{idx, task}] array).
        if let (Some(rid), Some(st)) = (run_id, store) {
            let plan_outputs = serde_json::Value::Array(
                sub_tasks.iter().enumerate()
                    .map(|(idx, t)| serde_json::json!({"idx": idx, "task": t}))
                    .collect()
            );
            let node = echo_core::state::TaskNode::new(plan_node_id(rid), "team_plan")
                .with_status(echo_core::state::TaskNodeStatus::Success)
                .with_outputs(plan_outputs);
            let _ = st.save_node(rid, &node).await;
        }

        debug!(sub_task_count = sub_tasks.len(), "Plan ready");

        // ── Phase 2: fan-out workers (skip prior Success; reset+rerun Running/Failed) ──
        let results = self.execute_sub_tasks(&sub_tasks, workers, run_id, store, &prior_nodes).await;

        // ── Phase 3: synthesize (always runs unless fast-pathed above) ──
        let synthesis = self.synthesize(team, manager_name, task, &results).await?;

        // Checkpoint: write synthesis node.
        if let (Some(rid), Some(st)) = (run_id, store) {
            let node = echo_core::state::TaskNode::new(synth_node_id(rid), "team_synthesis")
                .with_status(echo_core::state::TaskNodeStatus::Success)
                .with_outputs(serde_json::Value::String(synthesis.clone()));
            let _ = st.save_node(rid, &node).await;
        }

        Ok(synthesis)
    }
```

Add helper id functions at module level:
```rust
fn plan_node_id(run_id: &str) -> String { format!("team_{run_id}_plan") }
fn synth_node_id(run_id: &str) -> String { format!("team_{run_id}_synthesis") }
fn worker_node_id(run_id: &str, idx: usize) -> String { format!("team_{run_id}_worker_{idx}") }
```

- [ ] **Step 2: Update `execute_sub_tasks` for per-worker checkpoint + skip**

Change signature to accept the resume context and write per-worker nodes:

```rust
    async fn execute_sub_tasks(
        &self,
        sub_tasks: &[String],
        workers: Vec<&TeamMember>,
        run_id: Option<&str>,
        store: Option<&dyn echo_core::state::RuntimeStateStore>,
        prior_nodes: &std::collections::HashMap<String, echo_core::state::TaskNode>,
    ) -> Vec<(String, Result<String, String>)> {
        let worker_count = workers.len();
        let mut handles = Vec::new();

        for (i, sub_task) in sub_tasks.iter().enumerate() {
            let worker = &workers[i % worker_count];
            let worker_name = worker.name.clone();
            let agent = Arc::clone(&worker.agent);
            let task = sub_task.clone();

            // Skip-on-resume: if this worker_idx already Success, reuse its output.
            if let Some(rid) = run_id {
                let wid = worker_node_id(rid, i);
                if let Some(node) = prior_nodes.get(&wid) {
                    if node.status == echo_core::state::TaskNodeStatus::Success {
                        if let Some(out) = node.outputs.as_str() {
                            info!(worker = %worker_name, idx = i, "Reusing stored worker result");
                            // Push a "pre-completed" handle that yields the stored result.
                            handles.push(tokio::spawn(async move {
                                (worker_name, task, Ok(out.to_string()))
                            }));
                            continue;
                        }
                    } else {
                        // State reset defense (user patch #3): Running/Failed/Blocked
                        // → reset to Pending before re-running, overwriting stale state.
                        if let Some(st) = store {
                            let reset = echo_core::state::TaskNode::new(wid.clone(), format!("team_worker_{i}"))
                                .with_status(echo_core::state::TaskNodeStatus::Pending);
                            let _ = st.save_node(rid, &reset).await;
                        }
                    }
                }
            }

            handles.push(tokio::spawn(async move {
                let result = agent.execute(&task).await
                    .map_err(|e| format!("Worker {worker_name} failed: {e}"));
                (worker_name, task, result)
            }));
        }

        let mut results = Vec::new();
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok((name, task, result)) => {
                    match &result {
                        Ok(_) => info!(worker = %name, "Worker completed sub-task"),
                        Err(e) => warn!(worker = %name, error = %e, "Worker failed"),
                    }
                    // Checkpoint per-worker (Success or Failed).
                    if let (Some(rid), Some(st)) = (run_id, store) {
                        let status = match &result {
                            Ok(_) => echo_core::state::TaskNodeStatus::Success,
                            Err(e) => echo_core::state::TaskNodeStatus::Failed, // note: TaskNodeStatus::Failed has no payload
                        };
                        let outputs = match &result {
                            Ok(o) => serde_json::Value::String(o.clone()),
                            Err(_) => serde_json::Value::Null,
                        };
                        let node = echo_core::state::TaskNode::new(worker_node_id(rid, i), format!("team_worker_{i}"))
                            .with_status(status)
                            .with_outputs(outputs);
                        let _ = st.save_node(rid, &node).await;
                    }
                    results.push((task, result));
                }
                Err(e) => {
                    warn!("Worker spawned task panicked: {e}");
                }
            }
        }
        results
    }
```

NOTE: verify `TaskNodeStatus::Failed` is a unit variant (reconnaissance says `Failed` with no payload, unlike `Blocked{reason}`). Adjust if it carries data. Also the `i` from `enumerate` on `handles.into_iter()` may not align with the original sub_task index after `continue` — **this is a bug risk**. Fix: track the index alongside the spawned handle (capture `let idx = i;` inside the loop and move it into the closure, then use it for the node id). Revise to carry `idx` in the tuple `(worker_name, task, idx, result)`.

- [ ] **Step 3: Compile + run tests**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_agent --lib agent::subagent::team
```
Expected: PASS (existing tests call `run()` without store → in-memory path, behavior unchanged).

- [ ] **Step 4: Add checkpoint unit tests (the critical resume tests)**

In `manager_worker.rs` `#[cfg(test)] mod tests`, add tests using a stub `RuntimeStateStore` (in-memory `Mutex<HashMap<String, TaskNode>>` keyed by conversation_id). Tests:
1. `run_with_store_writes_three_checkpoints` — run with stub agents + store, assert plan/2-worker/synthesis nodes all `Success`.
2. `run_resumes_skipping_completed_worker` — pre-seed plan Success + worker_0 Success, assert only worker_1 spawned, synthesis merges stored + new.
3. `run_resets_running_or_failed_workers` (user patch #3) — pre-seed worker_0 Running + worker_1 Failed, assert both reset to Pending then re-run.
4. `run_fast_path_returns_stored_synthesis` (user patch #4) — pre-seed synthesis Success, assert zero agent calls.
5. `run_synthesis_missing_reruns_only_synthesis` (user patch #4) — pre-seed plan + all workers Success but no synthesis, assert only synthesize runs.

For stub agents, implement a tiny `CountingAgent` that records calls + returns a canned string. Use `Arc<AtomicU32>` call counters to assert "zero agent calls" / "only worker_1 called".

These tests are substantial — write each carefully with the stub store + stub agents. This is the highest-value part of the Sprint (proves the resume contract).

- [ ] **Step 5: Run all team tests + clippy + fmt + commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_agent --lib agent::subagent::team
cargo clippy -p echo_agent --all-targets -- -D warnings
cargo fmt --all && cargo fmt --all -- --check
git add src/agent/subagent/team/manager_worker.rs
git -c commit.gpgsign=false commit -m "feat(team): ManagerWorker checkpoint/resume(DAG skip-on-resume 模式)

Sprint 11 Task 4(核心):run() 三态 checkpoint + skip-on-resume。
- 入口 load_nodes(run_id) → 折 HashMap;synthesis Success → fast-path 返回。
- plan:若 Success 复用(有序 [{idx,task}] 数组,确定性绑定 patch #1);否则跑 + 写。
- workers:Success 复用;Running/Failed/Blocked 先 save_node 重置 Pending 再 spawn
  (状态重置防线 patch #3);每个完成写 Success/Failed 节点。
- synthesis:跑完写 Success 节点(outputs=答案)。
- store=None → 纯 in-memory 降级(向后兼容现有测试)。
- 删 plan/worker/synth id helper。

测试 × 5:三 checkpoint 全跑 / 复用 completed worker / 重置 Running-Failed /
fast-path synthesis / synthesis 缺失只重跑 synthesize(patch #4)。"
```

---

### Task 5: `dispatch_team` + router branch + config field

**Files:**
- Modify: `echo-agent/src/agent/subagent/executor.rs`

- [ ] **Step 1: Add `runtime_state_store` to `SubagentExecutorConfig`**

In `executor.rs`, `SubagentExecutorConfig` (~line 107):
```rust
    /// Sprint 11: optional state store for team-mode checkpoint/resume.
    /// None → teams run in-memory (no persistence). Injected by the app
    /// (FileRuntimeStateStore) to enable skip-on-resume.
    pub runtime_state_store: Option<std::sync::Arc<dyn echo_core::state::RuntimeStateStore>>,
```
Add `runtime_state_store: None,` to `Default` impl (~line 140).

- [ ] **Step 2: Add the `Team` branch to the dispatch router**

In `dispatch()` (~line 290-300), the `match mode`:
```rust
            let result = match mode {
                ExecutionMode::Sync => self.dispatch_sync(&req).await,
                ExecutionMode::Fork => self.dispatch_fork(&req).await,
                ExecutionMode::Teammate => {
                    match self.dispatch_teammate(req.clone()).await {
                        Ok(handle) => handle.join().await,
                        Err(e) => Err(e),
                    }
                }
                ExecutionMode::Team => self.dispatch_team(&req).await,
            };
```
**Do NOT touch the Sync/Fork/Teammate arms** (命脉 defense).

- [ ] **Step 3: Implement `dispatch_team`**

Add the method (per spec §三 updated code block — the corrected version with `ArcAgentBox`). Mirror near `dispatch_sync`/`dispatch_teammate`:

```rust
    /// Sprint 11: dispatch a team-mode subagent. Builds a TeamAgent from the
    /// definition's TeamSpec (manager + workers resolved by name from the
    /// registry), plumbs run_id + state_store, and runs it.
    async fn dispatch_team(&self, req: &DispatchRequest) -> Result<SubagentResult> {
        use crate::agent::subagent::team::ArcAgentBox;
        use crate::agent::subagent::team::TeamAgent;

        let registered = self.registry.get(&req.agent_name).await.ok_or_else(|| {
            ReactError::Other(format!("Subagent '{}' not found", req.agent_name))
        })?;
        let spec = registered.definition.team.as_ref().ok_or_else(|| {
            ReactError::Other("Team mode requested but definition has no TeamSpec".into())
        })?;

        // Resolve manager + workers by name (late binding, D-11-team-2).
        let manager_def = self.registry.get(&spec.manager).await
            .ok_or_else(|| ReactError::Other(format!("Team manager '{}' not registered", spec.manager)))?
            .definition.clone();
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
                .definition.clone();
            let w_agent = self.registry.get_agent(name).await
                .ok_or_else(|| ReactError::Other(format!("Cannot get worker agent '{}'", name)))?;
            builder = builder.worker(name, Box::new(ArcAgentBox(w_agent.clone())), w_def);
        }
        let team_agent = builder.build();

        let start = std::time::Instant::now();
        let result = team_agent.execute(&req.task).await
            .map_err(|e| ReactError::Other(format!("Team execution failed: {e}")))?;

        Ok(SubagentResult {
            agent_name: req.agent_name.clone(),
            output: result,
            duration: start.elapsed(),
            iterations: 1,
            tokens_used: None,
            was_truncated: false,
            mode: ExecutionMode::Team,
            usage: None,
        })
    }
```

NOTE: `TeamAgent::execute` already wraps in `tokio::time::timeout` (mod.rs:263) using `TeamConfig.default_timeout_secs` — do NOT add a second timeout. Verify `TeamAgent::builder()` exists (reconnaissance says yes, mod.rs:503).

- [ ] **Step 4: Update `agent_dispatch` tool enum**

In `echo-agent/src/tools/builtin/agent_dispatch.rs:165` + `:197-202`, add `"team"` to the schema enum + match arm:
```rust
                    "enum": ["sync", "fork", "teammate", "team"],
```
```rust
                    .and_then(|m| match m {
                        "sync" => Some(ExecutionMode::Sync),
                        "fork" => Some(ExecutionMode::Fork),
                        "teammate" => Some(ExecutionMode::Teammate),
                        "team" => Some(ExecutionMode::Team),
                        _ => None,
                    });
```

- [ ] **Step 5: Compile + test**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo build -p echo_agent
cargo test -p echo_agent --lib
```
Expected: PASS. Add a routing test asserting a Team-mode definition dispatches to `dispatch_team` (stub the registry with a team subagent + stub agents).

- [ ] **Step 6: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo fmt --all && cargo fmt --all -- --check
git add src/agent/subagent/executor.rs src/tools/builtin/agent_dispatch.rs
git -c commit.gpgsign=false commit -m "feat(subagent): dispatch_team + ExecutionMode::Team 路由

Sprint 11 Task 5(接线完成):TeamAgent 接入真实 dispatch 路径。
- SubagentExecutorConfig 加 runtime_state_store(Sprint 11 checkpoint 注入点)。
- dispatch router 加 Team 分支(**不改** Sync/Fork/Teammate,命脉 defense)。
- dispatch_team:解析 TeamSpec → 按名解析 manager/workers → ArcAgentBox 包箱 →
  TeamAgent::builder 注入 run_id + state_store → execute(自带 timeout,不二次包)。
- agent_dispatch 工具 schema 增 'team' enum 值。

Team 模式现可经 agent_dispatch(mode=team) 触发;无 TeamSpec 的 definition 报错。"
```

---

## Phase B — App wiring (Tasks 6-7)

### Task 6: Inject `FileRuntimeStateStore` + frontmatter parsing + optional builtin

**Files:**
- Modify: `echo-agent-cli/echo-agent-app-core/src/infra.rs`
- Modify: `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs`
- Create (optional): `echo-agent-cli/echo-agent-app-core/src/subagents/coding/team-research.md`

- [ ] **Step 1: Inject `FileRuntimeStateStore` into the executor config**

In `infra.rs`, where the main agent is built (near the worktree/workspace factory injection, ~line 344-347), add:
```rust
    // Sprint 11: inject FileRuntimeStateStore for team-mode checkpoint/resume.
    // Mirror worktree/workspace factory injection pattern.
    let runtime_state_store: Option<Arc<dyn echo_agent::state::RuntimeStateStore>> = {
        // Construct or reuse the FileRuntimeStateStore (app already has one
        // for runtime checkpoints — reuse the same instance if available).
        // ... locate the existing FileRuntimeStateStore ...
        // builder = builder.subagent_runtime_state_store(store);
    };
```
NOTE: the app already constructs a `FileRuntimeStateStore` for runtime checkpoints (`runtime_state_file.rs:19`). Reuse that instance (don't create a second one). Find where it's built (`infra.rs:739 create_runtime_state_store` per MASTER-PLAN) and pass the same `Arc` into the executor config. Add a `ReactAgentBuilder.subagent_runtime_state_store(...)` builder method (mirror `subagent_data_workspace_factory`).

- [ ] **Step 2: Parse team frontmatter in `subagent_loader.rs`**

In `WorkerFrontmatter` (~line 69), add:
```rust
    #[serde(default)]
    team_strategy: Option<String>,    // "manager-worker"
    #[serde(default)]
    team_manager: Option<String>,
    #[serde(default)]
    team_workers: Vec<String>,
```
In `WorkerDefinition`, add `pub team: Option<TeamSpec>` (re-export from framework). In the `resolve` function (~line 305), if `team_strategy == Some("manager-worker")` and `team_manager.is_some()` and `!team_workers.is_empty()`, build a `TeamSpec` and set `definition.execution_mode = ExecutionMode::Team` + `definition.team = Some(spec)`. Else leave as-is.

- [ ] **Step 3 (optional): Create `team-research.md` builtin example**

Create `echo-agent-cli/echo-agent-app-core/src/subagents/coding/team-research.md`:
```markdown
---
name: team-research
description: "团队研究 worker：manager 分解任务，explorer + summarizer 协同。"
team_strategy: manager-worker
team_manager: planner
team_workers: ["explorer", "summarizer"]
tags: ["team"]
---
团队研究模式：manager(planner) 分解研究任务 → explorer 探索 + summarizer 总结 → manager 综合。
```
Register via `include_str!` in the builtin fallback (mirror existing coding/*.md registration).

- [ ] **Step 4: Compile + test + commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
cargo check --workspace
cargo test --workspace
cargo fmt --all && cargo fmt --all -- --check
git add echo-agent-app-core/src/infra.rs echo-agent-app-core/src/subagent_loader.rs echo-agent-app-core/src/subagents/coding/team-research.md
git -c commit.gpgsign=false commit -m "feat(app): 注入 FileRuntimeStateStore + team frontmatter 解析

Sprint 11 Task 6: 应用层接线。
- infra.rs: 主 agent build 注入 FileRuntimeStateStore(复用 runtime checkpoint
  那份),经新增 builder.subagent_runtime_state_store(...) 透传。
- subagent_loader.rs: 解析 team_strategy/team_manager/team_workers frontmatter
  → TeamSpec + ExecutionMode::Team(只认 manager-worker 策略)。
- builtin team-research.md 示例(manager=planner, workers=[explorer, summarizer])。"
```

---

### Task 7: Full verification (both repos) + cargo clean + docs

- [ ] **Step 1: echo-agent full verification**
```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
./scripts/verify-all-crates.sh
```
All 8 crates + clippy + 12 features must pass.

- [ ] **Step 2: echo-agent-cli full verification**
```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo clippy --all-targets -- -D warnings
```

- [ ] **Step 3: Frontend**
```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend
npx tsc -b && npm run build
```

- [ ] **Step 4: cargo clean both**
```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent && cargo clean
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli && cargo clean
```

- [ ] **Step 5: Update MASTER-PLAN + deep-iteration-plan** — mark Sprint 11 ✅ with commit hashes; note Team mode now reachable, checkpoint/resume shipped.

---

## Self-Review Notes

**Spec coverage:** All spec §三 items mapped — ExecutionMode::Team (T2), TeamSpec (T2), SubagentDefinition.team (T2), config.runtime_state_store (T5), dispatch router (T5), dispatch_team (T5), dead-field deletion (T1), run() checkpoint (T4), TeamAgent run_id/state_store (T3), infra injection (T6), loader frontmatter (T6), all 5 resume tests (T4 Step 4 incl. user patches #1/#3/#4).

**Known unknowns to resolve during implementation:**
- Whether `TeamStrategy` is `Clone` (Task 2 — if not, add derive or Arc-wrap).
- Whether `TaskNodeStatus::Failed` carries data (Task 4 — reconnaissance says unit variant, verify).
- The `execute_sub_tasks` index-after-`continue` alignment (Task 4 Step 2 — carry idx in tuple).
- Exact `Agent` trait required-method set (Task 1 — 4 required per reconnaissance, verify no others).
- Where the existing `FileRuntimeStateStore` is constructed in the app (Task 6 — reuse, don't recreate).
- `ReactAgentBuilder.subagent_runtime_state_store` doesn't exist yet (Task 6 — add it, mirroring `subagent_data_workspace_factory`).

**Risk reminders:** dispatch router change is additive-only; checkpoint degrades gracefully (store=None); all new types default-off. The 5 resume tests are the primary safety net.
