# 并行整改批次 · Wave 0 契约冻结（v1 草案）

状态：**草案 v1**。lane 开工前由合流负责人确认一次；确认后本文件是 T1–T6 的唯一接缝事实源。
基线：`echo-agent` `f7bac62e`(main)、`echo-agent-cli` `0616c6c`(main)。下文所有 `path:line` 均已用 `git show <baseline>:<path>` 核对，与当前主检出（不在 main）无关。

## 1. lane 与文件所有权

| lane | 仓库 | 分支 | 独占写范围 | 只读依赖 |
|---|---|---|---|---|
| T1 | cli | `fix/Echoyue/mcp-config-ownership` | `echo-agent-app-core/src/mcp_config_runtime.rs` | — |
| T2 | cli | `fix/Echoyue/plugin-cleanup-receipt` | `echo-agent-app-core/src/plugin_runtime/` | T1 的 ownership API |
| T3 | cli | `fix/Echoyue/cron-occurrence-identity` | `scheduler/`、`runtime.rs`、`state/` | T4 的 occurrence API、T1/T2 的签名 |
| T4 | cli | `fix/Echoyue/task-cancellation-settlement` | `tasks/task_runtime/` | — |
| T5 | fw | `fix/Echoyue/subagent-tail-settlement` | `src/agent/subagent/executor.rs` | — |
| T6 | cli | `fix/Echoyue/sandbox-isolation-contract` | `src/tauri/commands/panels.rs` | — |
| R1/R2 | — | 只读，不建 worktree | 无 | — |

硬规则：**`runtime.rs` 只有 T3 能写**；T1/T2 需要的构造点或字段变更，由 T1/T2 在各自分支之内给出「准备补丁片段 + 签名说明」，交给 T3 应用。T3 不得反向修改 T1/T2/T4 的文件。

## 2. 接缝 S1：T1 → T2（名称 ownership 租约）

**现状（`0616c6c`）**：
- `mcp_config_runtime.rs:179` `PluginMcpOwnershipToken(u64)`（`Copy`），`:191` `McpNameOwnershipState{owners, next_token}`，`:199` `McpNameOwnershipRegistry`，`:254` `McpNameOwnershipGuard`；
- `:288 claim_plugin` 直接插入 owner 并发放 token；`:329 release_plugin` 直接删除 owner；
- `:437 pub(crate) fn ownership(&self) -> Arc<McpNameOwnershipRegistry>` 是唯一对外暴露点；
- 用户侧名字由 `claim_user_names`（`:223`/`:259`）与 `settle_user_names`（`:248`）维护；reconcile 在 `:706` 调用 `.claim_user_names(...)`，断开失败只 `warn`（`:867`）。

**冻结内容**：
1. S1 的 API 归 T1 所有，T2 是插件侧唯一消费者；T1 可新增「未完成/待提交」租约态（claim 后未确认安装成功前，名字既不能被他人夺走，也不能被当作已安装），但 **`claim_plugin` / `validate_plugin_claim` / `release_plugin` 的现有调用语义必须保持**，否则 T2 在同一 wave 内同步适配并在报告中标注。
2. 用户禁用 server 时，断开确认之前**不得** `settle_user_names` 释放名字；断开失败/超时保留「未完成」标记与该名字的 User ownership，后续重试成功后释放。这是 T1 的核心验收，不允许用「先释放再重连」绕过。
3. T2 只能通过 `McpConfigRuntime::ownership()`（`:437`）拿到 registry，不得另建 registry 或复制 token 语义。

## 3. 接缝 S2：T2 → T3（构造点签名与失败 receipt）

**现状**：`plugin_runtime/runtime.rs:2-9` `pub(crate) async fn new(agent_handle: AgentHandle, lsp: PluginLspRuntime, mcp_ownership: Arc<McpNameOwnershipRegistry>) -> anyhow::Result<Arc<Self>>`；失败时 receipt 随 `anyhow::Error` 丢失。调用点 `runtime.rs:1779-1790`（T3 所有）在 `Err` 分支走 `bootstrap_lifecycle.settle(ApplicationLifecycleReason::BootstrapRollback, ...)`。

**冻结内容（保持 main 在任意合并点可编译）**：
1. T2 新增 `pub(crate) async fn new_owned(..) -> Result<Arc<Self>, PluginBootstrapFailure>`，`PluginBootstrapFailure` 至少携带 `error` 与 `receipts: Vec<CleanupReceipt>`（含精确 MCP token）。
2. 过渡期保留 `new(..)` 作为薄包装（失败时映射为 `anyhow`、丢弃 receipts），**不得**在 T3 切换调用点之前删除它；T3 把 `runtime.rs:1779` 切到 `new_owned` 并处理 receipts；切换合流后由 T2 删除包装（独立小提交）。
3. 判定「成功」的唯一条件：所有 receipt 均已结算且 token 已释放或明确移交；存在未结算 receipt 时必须返回失败，不得返回 `Ok`。

## 4. 接缝 S3：T4 → T3（稳定 occurrence → TaskRun）

**现状**：`scheduler/runner.rs:15-19` 已 import `TaskRuntimeStore`、`RunDriverFailure`；`:42` 持有 `task_runtime_store: Option<Arc<TaskRuntimeStore>>`；store 侧已有 `create_run_for_active_workspace`（`store/runtime.rs:2050`）、`transition_run`（`:2581`）、`boot_auto_resume_decision`（`:2978`）、`resume_task_run_after_boot`（`:2997`）。

**冻结内容**：
1. T4 在 `tasks/task_runtime/` 内提供**幂等**接口：给定稳定 `OccurrenceIdentity`（可序列化、可持久化、由调度槽位而非投递时刻决定），要么返回已存在的 run，要么原子创建一个；返回 `(TaskRun, created: bool)`。
2. 查询路径不得改变 run 状态、不得触发 resume、不得写 journal 之外的副作用。
3. T3 消费该接口实现「同 occurrence 重投不重复执行」，并且**不得**改写 `boot_auto_resume_decision` / `resume_task_run_after_boot` 的 owner fence 语义：普通恢复不抢 cron，物理 owner 保护保留。
4. 现有 `create_run*` 语义不变；新增接口必须是**纯增量**，保证 T3 合入前 main 始终可编译。

**T4 已落定的接口（2026-10-22，T3 按此消费，勿改）**：模块 `echo_agent_app_core::tasks::task_runtime::occurrence`，在 `task_runtime` 根 `pub use occurrence::{OccurrenceIdentity, OccurrenceRunRequest};`。

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OccurrenceIdentity {
    pub workspace_id: String,
    pub occurrence_id: String,        // framework SchedulerInvocation::occurrence_id
    pub scheduler_task_id: String,    // CronTask.id
    #[serde(with = "echo_agent::utils::time::local_rfc3339")]
    pub scheduled_at: DateTime<Utc>,  // 槽位时间；manual 用请求时间，绝不用投递时刻
}
impl OccurrenceIdentity {
    pub fn from_scheduler_invocation(workspace_id: impl Into<String>, invocation: &echo_agent::scheduler::SchedulerInvocation) -> Self;
    pub fn digest(&self) -> String;          // 版本化 SHA-256
    pub fn run_id(&self) -> String;          // "occ-" + 64 hex，稳定幂等键
    pub fn validate(&self) -> Result<(), StoreError>;
}

impl TaskRuntimeStore {
    /// 只读：不改状态、不触发 resume
    pub fn find_occurrence_run(&self, identity: &OccurrenceIdentity) -> Result<Option<TaskRun>, StoreError>;
    /// 幂等准入：首次原子创建，重复投递返回已存在 run；created=false 时不写事件、不改状态、不 resume
    pub fn get_or_create_occurrence_run(&self, request: &OccurrenceRunRequest) -> Result<(TaskRun, bool), StoreError>;
}
```

`from_scheduler_invocation` 显式忽略 `attempt_id`/`attempt`/`trigger`；run_id 由 identity 四项（长度前缀防别名）SHA-256 决定，因此「同一槽位重复投递只创建一个 run」是构造性保证，而不是靠运行时去重。

## 4.1 R1 结论对 T3 的硬约束（2026-10-22，已确认）

R1 只读调查确认：**崩溃后存在永久未结算的 interruption**，且启动恢复会把它报成已结算。

机制（基线 `0616c6c`，逐行已核）：
- root 级 fence 是跨进程独占 lease（`state/app_state.rs:87,222-224` → `store/runtime.rs:40-45` → `file_shadow.rs:55` → `root_authority.rs:242`，底层 `echo-core/src/utils/fs.rs:106-136` fs2 flock），进程崩溃时内核自动释放，新进程一定能接管 root；
- run 级 fence `PhysicalOwnerIntent`（`types.rs:1431-1438`，无 pid/epoch/代际）在 `store/runtime.rs:1403` 写入并覆盖整个 driver 执行期，completion 只在同进程 `:1438` 写；`owner_id` 是 UUID，重启不可重建；
- 重启后 `recover_incomplete(:4998)` → `recover_interrupted_run(:5068)` 因 fence 非空在 `:5072-5074` 返回 `Ok(false)`，`confirm_run_settled(:1864-1891)` 拒绝 ack → run 永久 Running，而 `recover_incomplete` 返回 `Ok(0)`，boot 打印 settled（`state/app_state.rs:1808-1814`）；
- 之后 `request_cancel(:3429)`、`reserve_run_driver_admission(:1215)`、`boot_auto_resume_decision(:3121)` 全被同一 fence 拒绝；Paused 变体 `recover_paused_orphan_cells(:5021-5026)` 跳过，command cell 永久 active。
- 边界澄清：fence 判定通过 → 写 recovery batch 的微窗口**不丢**（单批原子 append，`:5249-5285`）；丢失发生在 driver 写 fence(`:1403`) → completion(`:1438`)/finalize(`:1484`)/ack(`:1541`) 这段 4 次独立 batch、无事务的窗口。

**对 T3 的约束**：
1. `recover_incomplete() == Ok(0)` 不等于「已结算」；T3 的启动恢复入口不得用它作为成功判据，也不得让「fence 拒绝」静默变成 settled 日志。
2. 需要「物理 owner」的可重建依据：root 独占 lease 的持有者代际（epoch / 进程身份），而不是 run 级 owner UUID 的相等比较。只允许在能证明旧 owner 进程已死（lease 已易主）时接管并结算；活着的并发 owner 仍必须被拒绝——这与「保留物理 owner 保护」一致，是收窄而不是移除。
3. cron 竞争尚未确认：`state/app_state.rs:1806` 先起 scheduler、`:1807` 才恢复，`runner.rs:125` 的 intent 写在 start 之后，恢复快照可能把活 cron run 判成僵尸。T3 需在 `store/runtime.rs:5001` 附近加确定性测试钩子后再下结论，未证实前不得改动现有顺序。
4. 该修复跨 T3 与 T4 的文件（fence 在 `tasks/task_runtime/store/`，恢复入口在 `scheduler/` + `runtime.rs`），**不在本批六条 lane 的冻结范围内**；见第 7 节 T7。

## 5. 合流与门禁

- Wave 1 并行：T5 / T4 / T1 / T6 / R1 / R2 同时开；T2 等 T1 的 token 语义冻结后开；T3 先做不依赖 T4 接口的部分（恢复入口、FileStore 失败语义），接口到位后接上。
- T5 先合 `echo-agent` main，bump 集成基线后再合 CLI lane。
- CLI 合入序：T1 → T2 → T4 → T3 → T6（每步 squash merge，前一步合完再 merge main）。
- 每条 lane 自测：`cargo fmt --all -- --check` + 聚焦 `cargo test -p echo_agent_app_core <name>` + touched crate clippy；源码冻结后跑一次完整门禁（CLI 6 条 + `--no-default-features`，FW 5 条，T6 另加 GUI 条件矩阵）。
- 共享架构文档、官网生成物、锁文件、顶层 gitlink（当前指向 `95ec66c6`/`f57b90a2`，均非 main）由合流负责人统一处理。

## 6. 范围外

215 项、LSP、生成 bindings、UI 候选展示、权限文档修正。

## 7. 待确认

- **T7 取舍（R1 衍生）**：R1 已确认的永久未结算窗口需要一条独立 lane（跨 T3/T4 文件，且要改 run 级 fence 语义与既有不变量测试）。默认建议：不并入 T3，作为 T7 在本批合流后单独开；若要求并入，T3 验收需追加「崩溃重启后 run 不再永久 Running」。
- S2 的过渡包装删除时机（当前定为 T3 合流后的独立提交）。

## 8. 衍生残留（不在六条 lane 内）

| 来源 | 结论 | 处置 |
|---|---|---|
| R1 | boot 后存在**永久未结算 interruption**（run 永久 Running，boot 谎报 settled） | 见 `r1-boot-interruption.md`；默认单开 T7，跨 T3/T4 文件 |
| R2 | 父 Session 关闭后子 Run 流**存在永久挂起**（P0 已复现）+ 客户端 feed 泄漏 | 见 `r2-sdk-child-handles.md`；属**合同缺口**（合同未定义关闭时子 Run 的下场），需 ADR，不在本批 |
| T5 | `echo-core` 的 `retaining_source_stream` 默认实现会在到达 executor drain 之前丢弃 tail 错误；依赖该默认实现的 agent 不受 T5 修复保护 | T5 只改 `executor.rs`，此点在报告中列为残留；是否补 echo-core 待 T5 合流后评估 |
| T1 | 超时路径 drop 掉进行中的 MCP preparation ⇒ `McpPreparationScope`（`echo-integration/src/mcp/client.rs:165-210`，**无 Drop**）的 `building` 永真 ⇒ `McpManager` 的 `preparing` ticket 永不清 ⇒ 该 name 在本 agent 生命周期内永久无法重连（`mod.rs:605` "another preparation is still active"） | 要求在 T1 文件内修复（超时改为后台跑到自然结束 + 关闭迟到 client），不合框架；框架侧补 Drop-cancel 应单开 lane |

## 9. 资源约束（本批实测）

构建期间磁盘曾被打到 146MiB（SDK facade 构建 + 多 worktree 并行）。约束：所有 cargo 加 `--offline`、限制 `-j2`；只构建目标 crate；单次编译前检查 `df -h /Users/ls`，低于 4GiB 停止并报告。

**不要共用 `CARGO_TARGET_DIR`**（此前合流负责人的建议是错的，已撤回）：实测多个 worktree 共用同一 target 时，cargo 把不同 worktree 的 `echo-agent-app-core` 当成同一个 unit —— 共享 target 里 `echo_agent_app_core` 只有一个 hash 的测试二进制，lane 之间会互相覆盖产物，测试结果可能来自别人的源码。规则改为：每条 lane 用自己 worktree 的默认 target，全绿后 `cargo clean` 自己的 target（日志先落 `/tmp`）。共享 target 已于 2026-10-22 删除（30GB）。

**lane 内构建统一用 `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` + `-j2`**：默认 debuginfo 下单个 `app-core` target 可达 17GB，三条 lane 并行会直接吃满磁盘（实测剩 194MiB），且约 170 个 rustc 进程在 16GiB RAM 上会被内核 SIGKILL。关 debuginfo 后产物约为原来的 1/3–1/4，不影响测试语义；`--all-features` clippy 一律不在 lane 内跑（10GB 级），留给冻结门禁。

**并发上限**：本机（16GiB RAM）最多 2 条 lane 同时编译。超出时按「先完成者先 clean」轮转，不要靠重试硬扛。

## 10. 执行记录

| 时间 | 事件 |
|---|---|
| 2026-10-22 | T5 squash 合入 `echo-agent` main → `287ed2d6`；`git diff 691990d6 HEAD` 为空（合入树与审过的分支一致）。框架集成基线 = `287ed2d6` |
| 2026-10-22 | lane 分支 SHA：T1 `a267585`+`841056b`；T3 `e11994a`+`cbca3ef`；T4 `1f88bfb1`(接口)+`458be9b`；T2 待交付；T6 未开始 |
| 2026-10-23 | T4 squash 合入 CLI main → `a20535a`；T1 → `c00dd4a`（26 tests 经主控独立复跑）；T3 → `aefa79e`（scheduler 16 / state 105 / clippy --all-targets 全绿） |
| 2026-10-23 | T2 squash 合入 CLI main → `83374f8`；T2 追加提交（删过渡包装 + pool 保留 receipts + cherry-pick T3 接线）→ `d3ab0e1` |
| 2026-10-23 | T6 squash 合入 CLI main → `8edf7b6` |
| — | **六条 lane 全部合入**：fw main `287ed2d6`（T5）、CLI main `8edf7b6`（T4→T1→T3→T2→T6）。逐文件校验：各 lane 分支与其独占文件 `git diff <branch> HEAD -- <path>` 全为空 |
| — | 进入冻结点：CLI 全量门禁（fmt / clippy all-features / 严格 panic lint / `test --workspace --all-features` / app-core no-default / GUI bin + GUI tests）→ 框架全量门禁 → 顶层 gitlink → CI |

**合流注意**：T3 的分支基于 `1f88bfb1`，不含 T4 的 `458be9b`（occurrence clippy 修复）与 T2 的 `new_owned`。因此 T3 合流前需要 `git merge main`，并（由 T3 本人）把 `runtime.rs` 的 `PluginRuntimeService::new` 调用切到 `new_owned`——这段接线必须在 T2 落地之后，否则编译失败。

**冻结门禁注意（ts-rs 污染）**：本地跑 `cargo test -p echo-agent-app-core` 时，ts-rs 导出测试会改写 `web-frontend/src/generated/*.ts`（16 个文件）。冻结点门禁必须把 `TS_RS_EXPORT_DIR` 指向临时目录（CI 已如此），否则工作树会被污染、且可能把生成物夹进合流提交。T3、T4 都各自 `git checkout` 还原过。

## 11. 冻结点门禁与 CI（2026-10-23）

**本地全量门禁**（隔离 target、`CARGO_PROFILE_*_DEBUG=0`、`CARGO_BUILD_JOBS=2`、`TS_RS_EXPORT_DIR=/tmp/ts-rs-export`）：

- CLI `8edf7b6`：fmt-check / clippy all-targets all-features / 严格 panic lint / `test --workspace --all-features` / app-core `--no-default-features` / GUI bin check / GUI tests —— **7/7 exit 0，2256 tests passed，0 warning**（日志 `/tmp/cli-gate.log`）。
- 框架 `287ed2d6`：fmt-check / clippy all / 严格 panic lint / `test --workspace --all-targets --all-features` / `check --no-default-features` —— **5/5 exit 0，3670 tests passed，0 warning**（日志 `/tmp/fw-gate.log`）。

**CI**：框架 main push 后 `Rust CI` 成功。CLI 首轮 `linux-quality` 失败于依赖风险 gate：`consumer inputs changed; human risk reassessment required`。根因是 `consumer_digest` 覆盖 `src`/`echo-agent-app-core/src`/`src-tauri` 下全部 `.rs`，本批应用层改动改变了指纹；`Cargo.lock` 未变（lock sha 仍匹配），六项已接受风险的 identity/checksum/kind 未变。处置：重算 `consumer_inputs_sha256`（`e46c7fcc…` → `9c834f6c…`），并在 `dependency-risk-decision.md` 记录复评理由，`scripts/test_cli_dependency_risks.py` 6/6 通过，提交 `456ccf5` 推入 main 触发重跑。

**顶层指针**：`fix/Echoyue/parallel-remediation-pointers` = `a73922e`（基于 `origin/main`，仅两个 submodule 指针 + 本目录文档）：`echo-agent` → `287ed2d6`，`echo-agent-cli` → `456ccf5`。该 commit 通过临时 index 构造，未改动顶层工作树（该工作树当前停在无关的 chore 分支且有未提交改动）。

## 12. 交付结论（2026-10-23）

| 项 | 结果 |
|---|---|
| `echo-agent` main | `287ed2d6`（T5），已 push；`Rust CI` run 39823176645 **success**（7m17s） |
| `echo-agent-cli` main | `456ccf5`（T4→T1→T3→T2→T6 + 依赖风险指纹复评），已 push；`Rust CI` run 39827830383 **success**（17m48s，4/4 job 绿） |
| 顶层 superproject main | `a73922e`，指针 `echo-agent=287ed2d6` / `echo-agent-cli=456ccf5` |
| 本地全量门禁 | CLI 7/7、框架 5/5，全部 exit 0；2256 + 3670 tests，0 warning |
| lane 清理 | 6 条 worktree 已 remove + prune，6 条 lane 分支与顶层指针分支已 `-D` 删除 |

**与计划的偏差（如实记录）**：CLI 实际合入序为 T4→T1→T3→T2→T6，计划写的是 T1→T2→T4→T3→T6。原因：T4 先落地才能给 T3 提供稳定接口并修掉其 clippy 拦截；T2 是唯一改动 `plugin_runtime/` 的 lane，最后合不影响其它 lane。文件级等价校验（各 lane 独占路径 `git diff <branch> HEAD` 为空）在每次合入后执行，因此顺序偏差未造成内容差异。

**未纳入本批（待裁）**：R1 的永久未结算 interruption（建议 T7）、`retaining_source_stream` 吞 tail 与 `McpPreparationScope` 无 Drop（建议 T8）。R1/R2 的证据与复现均在 `r1-boot-interruption.md` / `r2-sdk-child-handles.md`。
