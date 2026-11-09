# Todo 01 历史执行记录：关闭成功后的唯一会话结束通知

## 当前交付指针（2026-11-08 归档）

原 Plan `plan:ff675784-0933-485c-8d05-03413ce26f4c` 保持 `ready`。Frame IM `1f35f0d2…`、CLI 后端/IPC `70ac06a5…`、Accepted 观察 `bd2a083e…` 已分别本地交付；R08 293 个唯一相关测试、R09 相邻五项、R13 观察/相邻五项各有独立源码票，不能跨切点累加。全哈希、原件和边界见[阶段报告当前归档指针](../../reports/weixin-channel-recovery-2026-11-07.md)。

同源 phone 准备仅获离线 Final C0/I0/M0；新 `a1ea/2b296/48ee` 密度包（Mach-O `c9d1…`）仅 GUI 密度交付，未并入未提交 SRE，不继承真实 app.log、手机或最终组合门禁资格。最终 source-risk/gates 及原账号手机仍待，本次不自动 GUI。

## 初次实施原件（历史）

下文完整保留 2026-11-07 初次实施记录，其“本轮未提交”“尚未执行”“后续”与失败均是当时态，不作为归档时的当前状态或重跑指令。

执行时间：2026-11-07 17:02–17:30（Asia/Shanghai）。开发根：`/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`。
本轮未提交、未推送、未启动或停止 GUI、未执行手机验收。

## 1. 实施前状态核对

计划取证切点（15:50）与实施时（17:05）之间，另一会话已在同一根目录补入 RED 测试，两个关键文件字节已变化，不能沿用计划表中的旧哈希：

| 文件 | 计划表 15:50 哈希 | 实施前实际哈希 |
|---|---|---|
| `echo-agent/echo-integration/src/channels/session.rs` | `5d6aa856…` | `f05dc194…`（16:04，仅新增 `Shutdown` 变体与测试） |
| `echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs` | `71b6f2de…` | `db0d8722…`（16:08，仅新增协调器会话结束测试） |

其余 6 个关键文件哈希与计划表一致。两仓 HEAD 仍为 `f2e44dc9…` / `5f5fd491…`；分支均为 `feature/Echoyue/ssh-session-reuse`。未发现活动 cargo/rustc 进程或新日志，判断为上一轮 RED 后中断，本轮在原文件上继续。

## 2. 生产改动（仅一处）

`echo-integration/src/channels/session.rs` 的 `SessionHandler::close`：仅在 `handler.close()` 成功且 `remove_if` 实际删除同一 map 条目后，克隆该 session 的 `instance`/`generation`，先 `drop(guard)` 释放 mutex/map 引用，再同步调用既有 `settle_session_end(..., SessionEndReason::Shutdown)`。删除与通知之间没有 `await`；失败或取消不删除、不通知；沿用 `settle_session_end` → `defer_end` 的既有延迟结算与 panic containment。

未新增类型、回调或状态机；`record_session_end` 的既有 reason 无关逻辑直接接纳 `Shutdown`，应用侧 `register`/retirement gate 未改。

实施后哈希：`8cd6b130…`（含新增测试）；按仓库 edition 2024 执行 `rustfmt` 后为 `05c67457f62a7d7b`，格式化后 `rustfmt --check` 通过（未改动行为）。

## 3. RED（修改前真实失败）

临时移除 `settle_session_end` 调用、保留其余删除逻辑后运行：

```
cargo test -p echo_integration --features channels session_close
test session_close_publishes_one_shutdown_end_per_retained_session ... FAILED
  Error: "shutdown published 0 ends for 2 sessions"
test session_close_keeps_failed_agent_owner_for_retry ... FAILED
  Error: "successful close published [] instead of one shutdown end"
```

原始日志：`echo-agent/.echo-semantic/local/weixin-recovery-20261107/test-fw-session-close-RED-*.log`。

## 4. GREEN

- `cargo test -p echo_integration --features channels channels::session`：36 passed，0 failed（含 reset、timeout、失败/取消 close 重试、stream settlement、callback panic containment）。
- `cargo test -p echo-agent-app-core --features channels channel_runtime::`（`EKO_DATA_DIR` 指向本任务隔离目录）：170 passed，2 failed；两项失败均为 `terminal_stream_*` 的 `openpty: Operation not permitted` 沙箱环境错误，与本次改动无关，如实保留。
- 协调器回归 `channel_runtime::tests::shutdown_end_releases_the_coordinator_for_the_same_surface`：ok（A 首次准入 → 提前无前驱替换被拒 → close A → current=None 且 pending=A → 同 coordinator SessionHandler B 准入，current=B，incarnation 不复用）。
- 未设置 `EKO_DATA_DIR` 时同一模块有 33 项因 `shadow I/O: TaskRuntime root /Users/ls/.eko/tasks …` 和“认证回归必须由调度器指定 EKO_DATA_DIR”失败；这是测试运行环境要求，不是本轮回归。

日志：`test-fw-session-close-GREEN-*.log`、`test-app-shutdown-end-170816.log`、`test-app-channel-runtime-env-*.log`。

## 5. 尚未执行 / 后续

- 并发 close 的专属回归（当前由重复 close 与 map 身份检查覆盖，未单列并发用例）。
- fmt/clippy 与仓库合并门禁按计划在最终同一源码快照执行（Todo 04）。
- 文档/ADR 的 `Shutdown` 合同说明归 Todo 04。