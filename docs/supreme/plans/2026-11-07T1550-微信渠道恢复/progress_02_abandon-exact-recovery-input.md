# Todo 02 历史执行记录（当时进行中）：放弃单条待恢复输入

## 当前交付指针（2026-11-08 归档）

原 Plan `plan:ff675784-0933-485c-8d05-03413ce26f4c` 保持 `ready`。Frame IM `1f35f0d2…`、CLI 后端/IPC `70ac06a5…`、Accepted 观察 `bd2a083e…` 已分别本地交付；R08 的 293 唯一、R09 长回复/相邻五项、R13 观察/相邻五项均限各自源码切点，不跨切点累加。后继已取得的并发、工作空间、幂等及观测资格不由下方初始“待补”描述覆盖；准确原件见[阶段报告当前归档指针](../../reports/weixin-channel-recovery-2026-11-07.md)。

同源 phone 准备仅离线 Final C0/I0/M0，不是真实 app.log 或手机收件。新 `a1ea/2b296/48ee` 密度候选（Mach-O `c9d1…`）仅 GUI 密度交付，未并入未提交 SRE；完整 Plan 的最终 source-risk/gates、真实日志及原账号手机仍待，本次不自动 GUI。

## 初次实施原件（历史）

下文保留 2026-11-07 初次实施的改动、当时编译阻塞、失败及“待补”原文，不改历史断言或将旧失败覆写为通过，也不将其视为归档时仍未取得的局部资格。

执行时间：2026-11-07 17:30–17:55（Asia/Shanghai）。未提交、未推送、未启动 GUI、未执行手机验收。

## 1. 已实现的改动

### 原日志权威（app-core）

- `chat_event_log/journal.rs`
  - `cancel_conversation_input` 保持原公开行为，内部委托新增的 `cancel_conversation_input_checked(identity, expected_queue_revision, required_phase)`。
  - 新增 `cancel_recovery_conversation_input(identity, expected_queue_revision)`（`cfg(any(test, feature = "channels"))`）：在同一 `with_conversation_input_authority` 锁内依次核对身份、队列 revision、终态幂等、`phase == RecoveryRequired`，再追加既有 `Cancelled` 事实（保留原 attempt、reason、drained）。
  - `mark_channel_input_uncertain_checked`：当前已是 `Cancelled` 时直接返回，不追加事实；即使 `expected_queue_revision = None` 的原路径也不能复活已放弃项。
- `chat_event_log/projection.rs`：`RecoveryUncertain` 折叠对 `Cancelled` 项不再覆盖 phase/reason（迟到写入的双重防线）。
- `conversation_input.rs`：新增 `ConversationInputService::cancel_pending_recovery`，薄委托上述权威方法。

### 原渠道 owner（app-core）

- `channel_runtime.rs`
  - 新增 `ChannelPumpWiring`（原 handler 的 pump 依赖句柄集合；`coordinator` 与 `app_state` 使用 Weak，避免协调器注册表形成类型递归）与 `ChannelSessionWake`（原 durable handler 的成功唤醒能力）。
  - `ChannelSessionCoordinator` 新增 `session_wakes` 注册表与 `retain_session_wake` / `drop_session_wake` / `find_session_wake`；`record_session_end` 命中 current 时移除，`reopen_input_pump_admission` 清空（不触碰 lifecycle/pending obligation）。
  - handler 绑定 durable ingress 成功时以同一 service token 注册 wake；`start_input_pump` 与原 wake 共用同一 `start_channel_input_pump_with`（单份 adapter 构造，不生成第二个 owner）。
  - wake 的地址、agent conversation 与 cache id 由保留的原 `session_instance`（含 incarnation）推导，不读取当前 GUI 项目或最近账号。
- `channel_runtime/management.rs`：新增 `ChannelRuntime::abandon_recovery_input(identity, expected_queue_revision)`：
  - 从原日志不可变 binding 校验 `channel_id`、binding 数据与 digest；
  - 早退分类：非 RecoveryRequired、队列版本已变；
  - 按 product conversation 哈希找到原协调器上的 wake，并核对原工作空间、账户/凭据代次、工作空间代次；不一致明确拒绝，不重绑；
  - 仅在以上核验通过后调用 `cancel_pending_recovery`（phase/revision 由原 authority 锁内再核）；
  - 成功后调用原 slot 的 kick/start_input_pump，返回 `{receipt, queue_revision, pump}`。

### IPC 与界面

- `src/tauri/commands/channels.rs`：新增薄命令 `abandon_channel_recovery_input(identity, expected_queue_revision)`，委托 ChannelRuntime。
- `src/tauri/mod.rs`：注册该命令。
- `web-frontend/src/hooks/useTauriChat.ts`：新增 `abandonChannelRecoveryInput`，仅对 `channelOwned && phase === 'recovery_required'` 发送调用；成功提示“队列项已放弃；原执行状态与副作用结论保持不变”。
- `web-frontend/src/components/chat/ChatPanel.tsx`：仅对恢复中的渠道项显示“放弃这条待恢复消息”，并注明“仅解除队列阻塞；不撤销已发生的执行、发送或 UNKNOWN 结论”；渠道项的移除/编辑/执行仍禁用。
- `ChatPanel.controls.test.tsx`：新增 2 个用例（恢复项显示且可点击、普通渠道项无此入口），运行 46/46 通过；prettier 通过。
- `web-frontend/src/hooks/useTauriChat.test.tsx`：新增用例证明只有 `recovery_required` 渠道项才发出 IPC，且参数为原 identity 与观察到的 `queue_revision`；该文件 109/109 通过。

## 2. 尚未验证 / 阻塞

另一会话在 17:50 前后开始并发修改 `echo-agent/echo-core`、`echo-state`、`echo-execution`（新增 cache/layout、diagnostic 等，当前工作树存在未完成的类型与逗号错误）。`cargo check -p echo-agent-app-core --features channels` 因此无法编译本轮的 app-core 改动：

```
error[E0308] echo-core/src/llm/cache/diagnostic.rs:179
error[E0277] echo-core/src/llm/cache/diagnostic.rs:233
error: could not compile `echo_core` (lib)
```

上一轮（17:26）同一 target 的 app-core 测试构建是可编译的，故这不是本轮引入的失败，但必须在框架恢复可编译后重跑：

- `cargo test -p echo-agent-app-core --features channels abandon_recovery`
- `cargo test -p echo-agent-app-core --features channels channel_runtime::`
- `cargo check --no-default-features --features gui --bin echo-agent-tauri`
- 新增日志单元测试的实现也尚未编译（见 `conversation_input.rs` 的 `abandon_recovery_requires_exact_revision_and_blocks_late_uncertainty`）。

## 3. 本轮文件哈希（截断 16 位）

| 文件 | SHA256 前缀 |
|---|---|
| `echo-agent/echo-integration/src/channels/session.rs` | `05c67457f62a7d7b` |
| `echo-agent-app-core/src/channel_runtime.rs` | `3f9754e5d98ffca7` |
| `echo-agent-app-core/src/channel_runtime/management.rs` | `5d1b78ec7dc49c21` |
| `echo-agent-app-core/src/chat_event_log/journal.rs` | `4ad2a76e4a6ee34d` |
| `echo-agent-app-core/src/chat_event_log/projection.rs` | `02bf43ed23d79ac4` |
| `echo-agent-app-core/src/conversation_input.rs` | `5393a02a1d45483b` |
| `src/tauri/mod.rs` | `c76dca99302daf74` |
| `src/tauri/commands/channels.rs` | `d0afb7f0ca44215f` |
| `web-frontend/src/components/chat/ChatPanel.tsx` | `e0b1f37b0969ba05` |
| `web-frontend/src/hooks/useTauriChat.ts` | `0367a0608779b473` |
| `web-frontend/src/components/chat/ChatPanel.controls.test.tsx` | `c570186a3d265c6b` |
| `web-frontend/src/hooks/useTauriChat.test.tsx` | 见本轮 prettier 后文件 |

## 4. 待补

- 后端编译与单元/集成回归（等待框架工作树恢复）。
- ~~`useTauriChat` 的调用参数与拒绝条件测试~~（已完成，109/109）。
- 以上 Rust 文件已按 edition 2024 运行 `rustfmt`，逐个 `rustfmt --check` 通过；journal/projection/tauri 命令文件本就通过。fmt 只改格式，未重跑框架 session 测试（格式化前 36/36 已通过，格式化后待随下一批编译一并复核）。
- 并发放弃/恢复/claim、陈旧版本、正在执行、重复放弃的集成回归；同地址下一消息恰好执行一次、不同 workspace 隔离。
- 平台重复投递去重与 tombstone 行为核对。