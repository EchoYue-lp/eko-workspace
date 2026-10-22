# R2 调查报告：Session 成功关闭后子 Run 句柄的命运

结论：**存在挂起与泄漏**（P0 已用真 TS SDK 复现），根因是**合同缺口**而非单纯实现 bug。
调查全程只读，未修改被跟踪的源文件；为取证临时构建了 host 与 TS/Java 产物，结束已删除。复现脚本在 `/tmp/eko-r2/`（不在仓库内）。

检出事实更正：`echo-agent-sdk` HEAD = `a9f267f`，分支是 **`main`**；任务描述里的 `evidence/sdk-governance` 分支在本地不存在。

## 1. 三语言 × 三问题

| | TypeScript | Java | Host (Rust) |
|---|---|---|---|
| Q1 父关闭后的事件等待 | 正常路径 EOF（终止事件先于 close 响应，`client.ts:441-442`）；**live delivery 处于 ACK 暂停时有永久挂起**；`wait()/replay()` 立即 `closed_handle` | 同构；**额外确定性挂起**：关闭后首次 `updates()/events()` 会 `computeIfAbsent` 新建永不关闭的 publisher（`EchoAgentClient.java:1274-1284,1365-1374`） | 关闭时对每个子 Run `wait_receipt` → `persist_settled_receipt` → `remove_delivery_for_run`（`handler.rs:1128-1148`、`state.rs:317-319,396`），**`StreamDelivery` 无 Drop、不补发 terminal** |
| Q2 句柄失效语义 | 未本地 close 时全部 Host `closed_handle`；`close()` 先 `get()` → 永远失败且不 `endEvents`（`client.ts:1423-1433`） | 同路径 `closed_handle`；本地已 close 后返回 **`stale_handle`**（`RunHandle.java:135-138`），与 TS/Host 不一致 | tombstone 后 get/wait/cancel/steer/replay 一律 `closed_handle`+`retryable:never`；重复 close `released:false`；无 `run/close` 方法 |
| Q3 并发 close | 无 double-free；**feed/publisher Map 永不删除**（`client.ts:993-999,316-317`）→ 每 Run 常驻 | 同（`EchoAgentClient.java:227-228,1391-1399`） | 单锁 + tombstone，语义安全；**但** `persist_settled_receipt` 失败会中断循环且 `closing` 永不复位（`handler.rs:1102-1148`） |

## 2. 已复现的关键事实

- Host normal：`terminalEventsAtClose:[{type:"cancelled",seq:"3",at:444ms}]`，`session/close` 响应 at 454ms → 终止事件确实先于 close 响应。
- Host paused（window=1）：`terminalEventsAtClose:[]`、`terminalEventsAfterClose:[]`、`gapNotifications:1`，close 仍 `released:true`；之后 replay → `closed_handle`。**终止事件被静默丢弃**（`core_profile/events.rs:122-124,135-168`；`on_committed_event` 不检查 publish 返回值，`:419-424`）。
- TS 场景 B（`max_outstanding_live_events=1` 且不 ACK，关闭后订阅）：`postCloseFeed.status="HANG"`，drain 到 `["think_start","gap"]` 后 2s 超时；`run.replay` 与 `run.close()` 均 `closed_handle` → **无逃生通道**。
- TS 场景 A/C（正常窗口）：关闭后订阅仍能拿到 `cancelled` 并 `ENDED`，说明挂起只发生在投递已被暂停/丢弃时。
- Java：`mvn -o -q test -Dtest=LifecycleTest` → 17/17 通过，但只证明「被 close 的 publisher 会 onComplete」，不覆盖「永不 close」。

## 3. 严重度

1. **P0 永久挂起（已复现）**：关闭 Session 时子 Run 的 live delivery 处于暂停态 → 终止事件丢弃 + tombstone → 迭代器永不结束，replay/close 均失效。触发条件：`max_outstanding_live_events` 被未 ACK 事件填满后关闭。
2. **P1 `RunHandle.close()` 跨父生命周期永久不可完成**（TS/Java 各一处：先 `get()`/`cancelInternal()` 再决定本地 closed）。
3. **P1 Java 专属**：关闭后新建 publisher 永不完成。
4. **P2 客户端句柄/订阅泄漏**：feed/publisher Map 无删除路径。
5. **P2 Session close 可无界阻塞**（代码事实，未复现）：`wait_receipt()`、`wait_until_idle`、SDK `close()` 三处都无超时，且持 `lifecycle.write()`。
6. **P3**：`closing` 无复位；tombstone FIFO 淘汰后错误码从 `closed_handle` 漂移为 `invalid_value`；`closed_handle` 的 `operation` 字段统一为 `"_echo_agent/run"`。

## 4. 合同 vs 实现（关键判断）

- 合同明确：`run/*` 只有 start/get/wait/cancel/steer，**无 run/close**；`RunHandle.close()` 不 retire Host Run；released id → `closed_handle`；pause→gap→replay 是**必须**的恢复路径（`docs/sdk/protocol.md:240-251`、`docs/sdk/sdk-core-profile.md:99-114`）。
- 合同**没有**规定：父 Session 关闭时子 Run 句柄/流的下场，是否补发 terminal/stream-close，关闭后 replay 是否可用。ADR0028 只说 Session close 取消执行并丢弃 family resources，未提 Run。
- 因此 Host 的「tombstone 一切」不违反合同，但让合同**要求**的 replay 恢复路径在父关闭后失效 —— 这是**合同缺口**。P0/P1 落在缺口内；`stale_handle` vs `closed_handle` 是合同外的语言差异。

## 5. 未确认

- `status()/usage()/outcomeStatus()` 在父关闭后的真实返回（仅代码推断；需要 facade 版 host，`sdk-facade-all` 因磁盘满构建失败）。
- Java 端到端（需 facade host，未跑）。
- Session close 无界阻塞（未构造出忽略取消的 Run）。
- `persist_settled_receipt` 失败导致的半关闭（无故障注入）。
- 真实慢消费者在 window=256 下能否在 close 前填满窗口。

## 6. 测试覆盖缺口

三语言都没有「close session 时子 Run 仍 active，之后再使用/再订阅子 Run」的用例：TS `protocol-host.test.js:44-63`、`host.test.js:112-149`；Java `ProtocolHostTest.java:67-99`；Host Rust e2e `core_profile_e2e.rs:704,1092,7450…` 都是先等 Run 完成。P0/P1 目前无回归保护。

## 7. 归属

R2 是证据收集，**不扩展 accepted contract**。P0/P1 涉及协议层「关闭时补发终结信号」或放宽 tombstone，属于需要 ADR 的合同决策，不在本批六条 lane 内；记录为后续候选（与 R1 衍生的 T7 分开评估）。
