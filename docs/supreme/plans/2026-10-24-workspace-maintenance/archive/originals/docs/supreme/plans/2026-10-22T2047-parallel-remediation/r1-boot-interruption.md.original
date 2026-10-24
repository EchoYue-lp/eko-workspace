# R1 调查报告：boot owner fence 之后的未结算 interruption

结论：**确认存在**（永久未结算）。
证据基线：`echo-agent-cli@0616c6c`。引用的 6 个文件在 `0616c6c..0d1bbd3` 之间 diff 为空；调查全程只读，未修改、未提交、未建分支。**未执行测试**（构建与并发 cargo 争用，已终止），因此不声称测试通过。

## 1. 两套 fence 的错配

- **root 级 fence = 跨进程独占 lease**：`state/app_state.rs:87,222-224` → `store/runtime.rs:40-45` → `file_shadow.rs:55` → `root_authority.rs:221-267`（关键 `:242 try_exclusive_file_lease`），底层 `echo-core/src/utils/fs.rs:106-136`（`:127` `fs2::try_lock_exclusive` 锁 `<root>/.<name>.lease`）。进程死亡时内核释放 ⇒ 新进程一定能接管 root。
- **run 级 fence = `PhysicalOwnerIntent`**：`types.rs:1431-1438`，字段只有 `run_id/owner_id/execution_id/workspace_id/physical_terminal_observed`，**无 pid、无 epoch、无 lease 代际、无过期**；`owner_id` 是每次新生成的 UUID（`:1402`、`:848`）。

两者没有联动：root 能接管，但 run 级 intent 仍是上一个进程的 UUID。

## 2. 永久性证据链

```
P2 已过 root lease（root 归我）→ store/runtime.rs:1403 写 intent(observed=false)
✗ 崩溃（:1403 返回 Ok 之后、:1438 之前 = 整个 driver 执行期）
P3: root lease 再次获得
  └ recover_incomplete(:4998) → recover_interrupted_run(:5068)
      ├ :5072-5074 fence 非空 → Ok(false)
      ├ :4843 confirm_run_settled: Running+active_turn → Err(:1876-1891) → 不 ack
      └ :4844-4849 只 ack observed=true → 该 intent 永不清
  └ :5009-5014 不计数 → recover_incomplete = Ok(0)
  └ boot 打印 "TaskRun boot reconciliation settled"（state/app_state.rs:1808-1814）
```

之后 `request_cancel(:3429-3435)`、`reserve_run_driver_admission(:1215)`、`boot_auto_resume_decision(:3121)`、workspace transition(`:1938-1947`) 全部 `Err(RecoveryBlocked)`。

唯一能写 completion 的生产路径是同进程原 driver（`:1438`）；ack 强制 `observed=true`（`:4924-4931`）；重启后 UUID 不可重建 ⇒ **无生产路径可清**。旁路只有测试专用 `complete_joined_owner_for_test`（`:705`）和 `remove_conversation`（`:4715`，删整个会话）。

## 3. 精确边界

| 窗口 | 判定 |
|---|---|
| fence 判定通过 → 写 recovery batch（`:5249-5285` 单批原子 append，失败可重试 `:604` 钩子，测试 `store/tests.rs:6193-6238`、`boot_reconciler.rs:346-363`） | **不存在**永久丢失 |
| driver 写 fence(`:1403`) → completion(`:1438`)/finalize(`:1484`)/ack(`:1541`)，4 次独立 batch、无事务，进程内 debts(`:972-1121`) 随进程消失 | **存在**永久未结算 |
| 恢复入口 vs cron 竞争（`app_state.rs:1806` 先起 scheduler、`:1807` 才恢复；`runner.rs:125` intent 写在 start 之后） | **未确认**，缺扫描↔判定之间的确定性 barrier（建议在 `:5001` 加 `park_next_recovery_scan_for_test`） |

变体：崩溃在写 completion 后、finalize 前（`observed=true` 但 `confirm_run_settled` 仍 Err）同样永久；Paused + intent(false) 会被 `recover_paused_orphan_cells(:5017-5026)` 直接跳过，command cell 永久 active。

## 4. 反例（未执行）

`store/tests.rs` 草图：临时 shadow root 建 run → Running → `begin_physical_owner_intent(observed=false)` → drop（模拟崩溃）→ 重开 store → `recover_once() == Ok(0)`、run 仍 Running、decision/cancel 均 `Err(RecoveryBlocked)` → **再重开一次仍 Running**。基线 `store/tests.rs:170-343` 已固化约 90%，只缺「永久」与「取消被拒」断言。

## 5. 修复方向（未实施）

1. 让 intent 绑定 root lease 的 **generation/epoch**：新进程拿到 lease 即证明旧 owner 进程已死，可接管并记账；活着的并发 owner 仍拒绝。
2. 被 fence 跳过的 run 必须计入 `blocked` / `failed_scopes`，禁止 boot 谎报 settled。
3. cron 竞争先补确定性测试钩子再改顺序。

## 6. 归属

修复跨 `tasks/task_runtime/store/`（T4 文件）与 `scheduler/` + `runtime.rs`（T3 文件），**不在本批六条 lane 的冻结范围**。默认作为 T7 在本批合流后单开；若要并入 T3，T3 验收需追加「崩溃重启后 run 不再永久 Running」。
