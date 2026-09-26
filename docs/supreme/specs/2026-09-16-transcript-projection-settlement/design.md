---
title: Transcript Projection Durable Settlement
artifact: design
carrier: markdown
---

# Transcript Projection Durable Settlement

## 问题与目标

`ContextManager` 只拥有模型可见窗口，`RuntimeStateStore` 保存 ReAct checkpoint，
`ConversationStore` 保存用户可见 transcript。当前 `save_transcript_projection` 把
ConversationStore 写入错误降为 warning，pre-compact 仍可能继续压缩并 realign cursor，finalize
仍可能发布 terminal。这样一次 backend failure、timeout 或 commit 后丢失 acknowledgment，会让
checkpoint、模型窗口和用户历史观察到不一致事实，且恢复、clear、delete 没有 durable debt 可结算。

目标是在通用 framework 中建立一个唯一 transcript persistence coordinator，使每次投影写入都能
得到可恢复、幂等、有界且可观测的 settlement；独立 SDK 只无损承接 public ConversationStore
合同，不建立第二套持久化或终态权威。

## 目标行为

- 每个 runtime generation 通过 RuntimeStateStore 的 durable revision/CAS 同时最多拥有一个 pending
  transcript projection；prepare、attempt、ack 与 clear 都比较预期 revision，不同 Agent 实例或进程不能
  覆盖、ack 或清除其它 writer 的 pending。
- 投影 batch 使用稳定 operation identity，并绑定 conversation epoch、generation、ordinal 与完整
  canonical payload digest。
- pre-compact 只有在完整 transcript 已确认提交并清除 pending 后才能压缩或 realign cursor。
- finalize 在 pending marker 无法持久化时不得发布 terminal；marker 已持久化时，暂时性投影失败可保留
  execution terminal，并留下下一次 admission/recovery 必须先结算的 durable debt。
- recovery 在提交 `Hydrated` 前结算 pending；warm admission 同样先结算，不能依赖进程重启。
- exact runtime clear 必须先结算 pending；product delete 通过 ConversationStore 的原子 delete
  epoch/tombstone fence 与 RuntimeStateStore 的 durable scope retirement 阻止 timeout 后的晚到 apply 或
  checkpoint 重建，退役 stable conversation 下所有 generation pending，并在 receipt 中报告 operation
  identities。
- backend timeout 被视为 outcome unknown，不能推断 effect 未发生；reconcile 使用同一 operation
  identity 查询或重试，重复提交返回 `AlreadyApplied`。
- terminal 前发布 framework-owned typed `TranscriptProjectionSettlement` 观察；调用方可以查询 durable
  pending 状态。观察结果区分 `Settled`、`Deferred`、`Blocked` 与 `Conflict`，包含 operation identity、
  generation、attempt 和错误分类，但不并入 `TurnDeliveryOutcome`，也不改变 execution terminal authority。
- 未配置 ConversationStore 时保持现有无 transcript projection 行为；只要配置 ConversationStore，就必须
  同时配置 RuntimeStateStore。缺少 durable pending owner 的组合在 Agent admission 阶段、任何模型调用或
  persistence effect 之前直接拒绝。

## 范围与非目标

范围包括 framework 的 ConversationStore 原子投影/delete-fence 合同、File/SQLite 实现、
RuntimeStateStore compare-and-save/generation-scope retirement 合同与 File/SQLite 实现、runtime checkpoint
的 versioned pending marker、
compact/finalize/recovery/clear/delete 路径、deadline 与故障注入测试；以及
独立 SDK 的 ConversationStore/RuntimeStateStore protocol、Host extension bridge、TypeScript/Python/Java
合同与 E2E。

非目标：不创建第二套 Outbox、DeliveryLedger、Task 状态机或后台队列；不把 transcript settlement
塞进 TurnDeliveryOutcome；不引入 EKO SQLite；不新增仅 GUI 可见的状态；不发布 registry 或预编译
SDK 制品；不顺带解决 checkpoint current_plan、长期 memory 或 Task DAG Finding。

## 系统边界

`echo_agent` 唯一拥有执行终态、runtime checkpoint、pending marker、generation/scope CAS retirement、
recovery 与 clear 顺序。`ConversationStore` 唯一拥有已提交 transcript fact、stable conversation epoch
与 delete tombstone。内部 persistence coordinator 只编排现有权威，
不拥有第二份 transcript。`echo-agent-sdk` protocol/Host/语言客户端仅传输 typed batch 与 receipt。
EKO 应用只选择 backend、timeout policy 和用户投影，不复制 apply/reconcile 状态机。

```text
ContextManager (active window)
        |
        v
AgentPersistenceCoordinator
  | prepare pending marker
  v
RuntimeStateStore revisioned checkpoint + pending / retirement tombstone
  |
  | apply/reconcile with stable operation_id
  v
ConversationStore epoch-fenced committed transcript
  |
  v
cursor ack + clear pending -> compact / hydrate / terminal / clear
```

## 核心结构与数据流

`TranscriptProjectionBatch` 携带 deterministic operation identity、conversation identity、runtime
generation、conversation epoch 和 generation-scoped ordered messages。Operation identity 由 schema
version、conversation、epoch、generation、ordinal range 与完整 canonical batch digest 计算，不使用随机
重试 UUID。Canonical batch 覆盖 message identity、role、content、metadata、ordinal 和首次 prepare 时
固化的 created_at；recovery 不重新生成任何 effect 字段。

`PendingTranscriptProjection` 保存完整 canonical batch、cursor before/after、prepare time、attempt
metadata、最后错误分类和 checkpoint revision。它是 effect 完成前的 durable intent，不是第二份已提交
transcript；只有 ConversationStore receipt 才能把它结算为 fact。它作为 `AgentCheckpoint` versioned
private payload 的 optional 字段持久化，因此旧 checkpoint 无 pending 时继续兼容恢复。

`RuntimeStateStore` 新增或扩展 public compare-and-save checkpoint 合同：请求携带 expected revision，
成功 receipt 返回 committed revision，过期写入返回 typed revision conflict；plain save 保留既有
unmanaged 初始化语义，但不能被 coordinator 使用，也不能覆盖已有 revisioned checkpoint、generation
tombstone 或 scope retirement。generation retire 与 scope retire 同样是 CAS 操作，写入
durable tombstone，使 stale writer 无法在 exact clear 或 product delete 后重建 checkpoint，新 generation
admission 也不能越过 active scope retirement。独立 SDK 必须无损承接这些 wire operation；本设计不新增
pending 专用的第二套 store。

Legacy mutator 采用明确的 managed boundary：plain checkpoint save/clear 只处理从未进入 CAS 管理的记录；
一旦 record 出现 revision、pending 或 retirement，旧 mutator 返回 `ManagedStateRequiresCas`，raw clear 不得
删除 tombstone。`save_messages` 的 replace/import 语义只适用于 unmanaged conversation；epoch 已建立后，
raw save/delete 返回 `ManagedConversationRequiresProjection`。确需迁移或 restore 时使用带 expected
revision/epoch 的显式 managed import 操作，不能借旧 API 重置 authority。

`ConversationStore::apply_transcript_projection` 是新的 public 原子操作：原子 ensure conversation 与
merge batch，并比较 batch 绑定的 conversation epoch；当前 epoch 相同且相同 operation/digest 返回
`AlreadyApplied`，相同 ordinal/不同 digest 返回 conflict，epoch 已删除或已推进返回 fenced receipt，
且保留其它 generation 的已提交消息。File 与 SQLite backend 实现相同合同；trait 默认实现只能明确
返回 Unsupported，不能用 `get + save_messages` 冒充原子性。

Coordinator 在 prepare pending 前调用原子的 `ensure_projection_epoch`：未存在的 conversation 从 epoch 1
开始，已存在的 managed conversation 返回当前 epoch，删除后重建使用 tombstone epoch 的 checked
increment，溢出返回 typed error。Admission 先读取 active runtime scope revision，再取得 conversation
epoch，最后用同一 expected scope revision CAS prepare pending。若 delete 在 epoch acquisition 与 prepare
之间抢先建立 scope retirement，prepare 必须失败；若 prepare 先提交，随后 begin-delete 的原子 manifest
必须包含它。epoch acquisition 及 receipt 属于 ConversationStore public/SDK wire contract。

内部 `AgentPersistenceCoordinator` 统一 prepare、apply、reconcile、ack 与 clear 顺序。每次 durable
transition 都携带 expected checkpoint revision；revision mismatch 必须 reload/reconcile，不能依赖单个
Agent 实例的 execution mutex 充当 generation-global lock。现有 mutex 只串行化同一实例的 execution。
若未来需要多条队列，必须复用现有 DeliveryLedger，而不是把 pending marker 扩张为新 Outbox。

Product delete 使用稳定 delete operation identity 与 expected conversation epoch。Coordinator 先以原子
CAS `begin_scope_retirement` 在 RuntimeStateStore 同时 fence scope，并把当时全部 generation、checkpoint
revision 与 pending operation identity 固化为 durable delete manifest；record 还保存每项 `Pending` /
`DroppedByDelete` 进度、expected epoch 与 settlement phase。随后 ConversationStore 原子推进 epoch 并写
tombstone，coordinator 按 manifest 退役 runtime lineage，最后把 stable completion receipt 写回 retirement
record。任何 generation authority 删除前，其 operation identity 已在 manifest 中；崩溃恢复从未完成项
继续，最终 receipt 始终报告完整集合。完成后仅删除 checkpoint payload，最小 retirement record 在
tombstone retention 内保留。任何在 fence 之后到达的旧 revision save 或旧 epoch apply 都返回
fenced/conflict receipt，不能复活 lineage 或 transcript。相同 delete identity 重试返回
原始 `AlreadyDeleted` receipt，即使 conversation 已有新 incarnation；不同 delete identity 携带旧
expected epoch 才返回 epoch conflict。判定顺序是先按 delete identity/payload 查 receipt，再比较 current
epoch。Receipt 只在声明的 tombstone retention 窗口内完整保留；窗口外旧 operation 返回带
retention floor 的 `ReceiptExpired`，不得重新执行删除。conversation 以新 epoch 重建后，任何旧 delete
请求都不得删除新 incarnation。

Settlement 状态由 coordinator 产生：成功 apply/AlreadyApplied 为 `Settled`；durable marker 保留且可重试
为 `Deferred`；marker 未持久化或依赖缺失为 `Blocked`；payload/ordinal/epoch 不一致为 `Conflict`。
每次 terminal 发布前必须先发 typed settlement observation，RuntimeStateStore 查询返回同一 durable debt
视图，日志与 trace 只投影该结果而不另建权威。

错误与 deadline 映射固定如下：

| Store/transport 结果 | Settlement | 是否可发布 execution terminal |
| --- | --- | --- |
| `Applied` / `AlreadyApplied` 且 ack 成功 | `Settled` | 是 |
| dispatch 后 timeout/disconnect、commit outcome unknown，或明确 transient/no-commit I/O | durable marker 存在时 `Deferred`，以同一 operation identity 重试 | 是，且必须先发布 settlement observation |
| revision conflict | reload/reconcile；预算耗尽且 marker 仍 durable 时 `Deferred` | 是 |
| Unsupported、缺 durable owner、prepare/serialization/validation 失败、corrupt state | `Blocked` | 否 |
| ordinal/digest/epoch conflict，或 reload 后发现不同 canonical payload | `Conflict` | 否 |

每次 prepare/apply/ack/reconcile 都受一个 framework settlement budget 约束；单次 backend/SDK 调用 deadline
取 run safe-point 剩余时间、settlement budget 剩余时间与 backend 配置上限的最小值。SDK registration、Host
dispatch 和 extension invocation 共用请求的 absolute deadline，只能向下传递剩余时间，不能在适配层重置
计时。重试使用有上限的 attempt count 与 capped backoff；本轮预算耗尽只在 durable marker 已存在时转为
`Deferred`，admission/recovery 可用新预算继续，同一个 backend future 不能越过 deadline 阻塞 safe point。

## 异常和边界场景

- prepare marker 写失败：状态不可恢复，pre-compact/finalize 均 fail closed。
- apply 返回 error 或 timeout：保留 pending，进入有界 reconcile；late commit 由 operation identity 去重。
- apply committed 但 ack checkpoint 失败：下一次 recovery 观察 `AlreadyApplied` 后补写 cursor/clear pending。
- 两个 Agent/进程同时 prepare、attempt 或 ack 同一 generation：只有 expected revision 命中的写入生效；
  失败方 reload 后复用同一 operation identity，不生成第二个 batch。
- pending 持续失败：禁止新模型调用、压缩和 exact clear；execution terminal 只在 marker durable 时保持。
- corrupt pending：恢复失败关闭，不猜测、丢弃或重放 effect。
- `Completed`、`NoResponse`、cancel、consumer disconnect、provider failure、tool failure、guard/intervention
  stop 与 max-iteration 都必须经过同一个 terminal persistence helper；cold hydration 与 warm admission 都先
  reconcile，再允许上下文或模型调用继续。
- Unsupported、corrupt/invalid payload 和 semantic conflict 是 fail-closed；timeout/disconnect 只在 durable
  marker 已存在时允许 Deferred，不能把 outcome unknown 当作 no-commit。
- 两个 generation 并发投影同一 stable conversation 时，原子 merge 不得发生 read-modify-replace lost update。
- exact clear 先 CAS retire generation，再删除 checkpoint；stale revision 不能重建已清除 generation。
- product delete 先持久化 scope retirement，再推进 epoch/tombstone，最后退役所有 generation pending 和
  runtime lineage；旧 revision/epoch 的 late write 被 fence。相同 delete identity 重试幂等，旧 expected
  epoch 不能删除后来重建的新 incarnation。
- operation timeout 与 reconcile timeout 分离，任何 backend future 都不能无限阻塞 safe point。

## 关键取舍

Claude Code 持续保存本地 transcript，并让 resume 恢复完整历史；`/clear` 不把旧 conversation 当作
模型窗口的一部分继续使用。这支持 active context 与 durable transcript 分离。Claude Agent SDK 的
SessionStore 要求 ordered append/load，并建议并发 summary 更新使用 transaction、CAS 或 lock；这支持
原子 merge，而不是 read-modify-replace。Temporal 指出 effect 已完成但 acknowledgment 丢失时会重试，
因此写操作必须有稳定 idempotency key 与 timeout，不能从 timeout 推断 effect 未发生。

参考：

- https://code.claude.com/docs/en/sessions
- https://code.claude.com/docs/en/agent-sdk/session-storage
- https://docs.temporal.io/activity-definition

EKO 是本地单用户产品，不需要分布式 workflow scheduler 或多租户队列；但进程崩溃、磁盘失败与
extension timeout 在本地仍成立。因此选择每 generation 单 pending marker，而不是引入 Temporal 式系统。

## 复用与实现约束

- 复用现有 transcript generation/ordinal/digest cursor、RuntimeStateStore checkpoint payload、execution
  mutex、File/SQLite 原子写工具和 DeliveryLedger 概念。
- execution mutex 不能替代跨实例 revision/CAS；所有 generation 级 pending transition 必须由 durable
  checkpoint revision 判定。
- ConversationStore delete 与 RuntimeStateStore retire 是一个可恢复 saga，不假装跨 backend 原子事务；
  durable scope retirement phase 是重启后继续完成 delete 的唯一协调状态。
- 保持 `save_messages` 对 unmanaged conversation 的 replace/import 语义；managed conversation 必须使用
  projection 或带 expected epoch 的显式 import，旧 mutator 不能静默绕过新 authority。
- framework 只提供产品无关机制；timeout 默认与 EKO UI 投影留给应用配置。
- `ConversationStore` + `RuntimeStateStore` 是启用 durable transcript projection 的不可拆分配置；不保留
  volatile store-only 模式，也不把 pending/receipt 下沉到 ConversationStore。现有只配置
  ConversationStore 的复用方必须补充 RuntimeStateStore 或关闭 transcript projection。
- adapter 必须字段级无损，不通过 schema-free JSON、默认值或字符串中转吞掉 operation identity、
  generation、ordinal、epoch、absolute deadline、error/retryability 或 settlement。
- TUI、GUI 与 CLI/channel 若未来展示 deferred 状态，必须消费同一 core status；本设计不新增 UI-only 状态。
- 所有文本与错误预览保持 UTF-8 安全；新增 Rust 代码遵守无 unwrap/expect/panic/unreachable 约束。

## 验收标准

- File 与 SQLite ConversationStore 对 Applied、AlreadyApplied、ordinal conflict 和双 generation 并发
  merge 有确定性测试，且不丢已提交消息。
- unmanaged/managed transition 测试证明 plain checkpoint/conversation save、clear、delete 无法覆盖 pending、
  retirement、epoch 或 tombstone；显式 managed import 必须比较 expected revision/epoch。
- 只配置 ConversationStore 的 Agent 在 admission 前返回 typed configuration error，且 context、LLM、trace、
  guard、checkpoint 与 transcript backend 均无副作用；两者均未配置时保持既有行为。
- 首次 epoch acquisition、delete 后 recreate、acquire-vs-retire 竞态与 epoch overflow 都返回确定 receipt；
  SDK wire 字段级无损。
- prepare/apply/ack 的每个 crash cut、commit-then-error、永不返回 backend 与 late commit 都可恢复结算。
- 并发 Agent/进程对同一 generation 的 prepare/ack 通过 revision conflict 收敛到一个 operation；delete
  fence 后旧 revision 的 checkpoint save 与旧 epoch 的 late apply 都不得复活 lineage 或 transcript。
- 未确认投影不会压缩 context、触发 PostCompact 或 realign cursor。
- `Settled`、`Deferred`、`Blocked` 与 `Conflict` 可通过 typed observation 和 durable query 区分；durable
  deferred 不改写 execution terminal，undurable failure 不发布 terminal。
- recovery、warm admission、exact clear、product delete，以及 Completed、NoResponse、cancel、consumer
  disconnect、provider/tool failure、guard/intervention stop、max-iteration 全部共享同一 coordinator。
- lost-ack delete 在 retention 窗口内使用相同 delete identity/payload 返回原始 AlreadyDeleted receipt，即使
  conversation 已以新 epoch 重建；不同 identity 的旧 expected epoch 返回 epoch conflict，窗口外返回
  ReceiptExpired/retention floor，三者都不修改新 transcript/runtime lineage。
- 多 generation delete 在每一个 crash cut 后都从 durable manifest 继续，并返回相同完整
  `DroppedByDelete` operation identity 集合。
- Unsupported、corruption、semantic conflict、transient I/O、disconnect 与 timeout 的状态映射固定；
  absolute deadline 经过 framework、Host 与 extension 只缩短不重置，所有等待均有界。
- SDK Host 能在 extension commit 后断连并于重试/重启得到 AlreadyApplied；三语言合同与 generator 零漂移。
- Framework 与 SDK 各自通过完整本地门禁、远端 CI、semantic strict/change-evidence 和独立复审。
- Issue #106 只在 framework、SDK 与最终跨仓语义证据全部进入远端 main 后关闭。
