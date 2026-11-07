---
title: 执行报错治理：后台准入、回执所有权与有界保存
artifact: design
carrier: markdown
---

# 执行与保存可靠性设计

## 审查结论与证据

本设计针对截图中 `Tool 'shell' execution timed out` 与 `Runtime state deadline exceeded: runtime checkpoint CAS` 的整条生产路径。框架快照 `f2e44dc9be8972e286f9a84520ddb263f0012424`、应用快照 `5f5fd4912855cfda2326c54f294f11cb879257fa`，均含现有未提交增量。独立只读审查未运行 Cargo、App 或原任务；实现尚不通过，保存具体耗时占比尚未测量。

原数据根为应用 `.echo-semantic/local/persistent-GUI-owner-66760c3-preparation-20261031-r01/isolated-runtime/data`。原 Run 为 `runs/run_0e6c96fc-9b2c-408a-9357-462afe1ac266.jsonl`；原 TaskRun 事件为 `workspaces/Lp-agent/.eko/tasks/taskrun:df382922-de01-47a4-8219-25e7d01a0779/events.jsonl`，对应聊天 journal 的流目录为 `chat-events/sha256_c8cf992d9d37c3920b270d89945295407b3f6be81b0ed2963fb46bc502f17de3`。不将旧报告里“命令结果未知”的早期结论覆盖成当时已知；以下为读取后续原记录所得。

| 北京时间 | 原事实 |
|---|---|
| 16:08:47 | 最后模型请求成功；后台 shell 请求 `background=true, timeout=3600`，call `call_5034791ef36446a9a36f29d2`，cell `d4006718-e930-4473-9018-de8e80294ec6` 持久 prepared。prepared 时间不等于 OS 进程开始时间。 |
| 16:11:29.729 | 占用前一执行的 cell `e33bdfd2-c80d-4e81-9616-85b4d2aa59bc` 以 101 结束。 |
| 16:11:30 | 新启动调用经过约 163.247 秒返回工具超时，但已创建的后台 cell 并未随之结束。 |
| 16:11:44～45 | 工具批次之后保存报 CAS 入口 deadline 已过期；transcript 为 blocked，整轮失败。工具结果到保存观察相隔约 14.43 秒。 |
| 16:22:15.748 | 新 cell 最终以 101 退出；原测试 `shutdown_end_releases_the_coordinator_for_the_same_surface` 失败，报 `a successful shutdown did not release the exact incarnation`。 |

### 根本性 findings

| 编号 | 严重程度 | 源码事实与影响 | 处理 |
|---|---|---|---|
| EXEC-01 | Important | CLI `command_cells.rs:1754,2080` 在 prepared/Started 持久化后、返回 receipt 前等待外置 shell semaphore；`executor/limits.rs:129` 默认 shell=1。与 ShellTool 的后台立即回 cell 体验和外层 120 秒工具预算组合失配，普通 Chat 与 run-owned 均受影响。 | 全局资源等待进入原 cell 的 Queued runner，接纳回执不等待执行槽。 |
| EXEC-02 | Important | `echo-execution/src/tools.rs:103-135` 的 started timeout 取消后等待原 future，最终可能返回 timeout；`shell.rs:334-337` 的 lifetime cancel=None 使后台可继续，成功 launch receipt 可丢失。`command_cell.rs:633-641` 的 tracked-capacity 等待同样不能被当前启动调用取消。 | 区分接纳前取消与接纳后 cell 生命周期，准确保留 receipt，覆盖两类 capacity。 |
| EXEC-03 | Important | `snapshot.rs:1098` 从操作开始共享默认 10 秒；错误精确来自 `state/file.rs:1512` 的 CAS 入口检查，而非已证明 CAS 写了 10 秒。`reconcile_scope_owner_sync:1002` 扫描全记录，`runtime_records_sync:1201` 同记录再次读/解析，owner 验证和 coordinator 又多次恢复完整 payload。 | 先分段取证，再在同一原操作内消除重复工作，保留 CAS 和恢复权威。 |
| EXEC-04 | Important | 原检查点 9,526,506 字节、130 条消息，包含 8,028 个 raw delta blocks、69 个 request/block 身份；raw 与 approved 两组 JSON 各约 4.11 MB。增量原件包含身份、sequence 与发布证据，并非可随意删除的冗余。 | 不删 provenance、不改存储格式来伪造快检；验证读取/校验/序列化成本后优化。 |
| EXEC-05 | Important | 101 的实际测试失败与工具超时、保存失败是不同事实。 | 回到原微信恢复计划修复其业务断言；本设计不代替或扩大该业务任务。 |

`CommandCellRegistry` 合同明确允许有界等待容量（`echo-core/src/tools/cell.rs:335-337`），因此不能笼统宣称所有 framework launch 都违反立即返回。根因在 Eclip 外置准入与工具调用/后台生命周期的配合；实际哪个保存子阶段占主导仍需受控 profile，不能把当前风险判断写成已测根因。

## 目标与范围

后台启动在接纳边界前可取消且无执行，接纳后总能找回准确 cell 回执；排队、运行、取消、期限和退出由原 supervisor 持有。保存维持真实、有界和可恢复，不把未知写成成功。两者共同保证长工具与大上下文不会因错误的观察/所有权边界使整轮失真。

覆盖普通 Chat、TaskRun、后台 shell、原 CommandCell manager、ToolManager、FileRuntimeStateStore 与 coordinator 的直接路径。没有新队列、执行器、Outbox、SQLite、TaskRun 状态机或资源策略放宽。原微信业务测试修复、自动重跑旧命令、重发通知、删除历史及登录/原数据修改均不属于本计划。

## 分层与复用

| 层次 | 责任 |
|---|---|
| 框架通用机制 | 原 CommandCell manager 中的准入上下文、取消/期限、Queued 执行等待、资源保有；原 Store 中的 CAS/恢复和操作内验证复用。 |
| Eclip 产品策略 | 全进程 shell=1 等资源上限、TaskRun/普通 Chat 接线、原 Started/Finished 事实及现有 UI 观察。 |
| 薄适配边界 | 提供通用异步资源准入回调及原资源 guard；引用原 manager/TaskTracker，不拥有第二个 ready frontier 或执行循环。 |

复用 `Prepared/Queued/Running`、`CommandCellReservation`、`start_prepared/abort_prepared`、原 watcher、TaskTracker、同一 journal 和 `PersistenceCallContext`。现有 `resource_guards` 只能保有已取得资源，同步 ToolAdmissionFn 也不能替代异步等待；需要时在原通用 API 增加最小明确接缝，并核对全部合理公共消费者。

## 目标行为

### 接纳之前：调用方可以撤回

启动调用携独立准入取消和有界调用预算，与后台 cell 的生命周期 token/deadline 分开。tracked-capacity、manager 准入锁、prepared 产生及应用 Started 持久化均受准入上下文监督；不能仅取消外层 future 而让容量释放后继续创建新任务。

接纳前取消/超时必须证明没有 OS/远端执行；若已生成 prepared reservation 但尚未转移所有权，由原 reservation 的中止路径结算。持久 Started 失败不启动命令，原失败真实保留。取消/关闭/接纳竞争只产生一次明确结果，不制造第二个 cell。

### 接纳之后：回执与 owner 一起转移

prepared 的原 cell、必要资源 guard、原持久 Started 及 supervisor 接纳形成唯一交接点。完成交接后返回准确 `cell_id/accepted_at/deadline`，不等待全局 shell 槽空闲。外层观察预算随后到期，不能把已接纳的成功回执降为“未创建”；调用取消与回执传输竞争时，按原 invocation/call 身份仍可查询原回执，不为重试自动创建副本。

这只要求原作用域内回执可恢复，不自动授予再次执行同命令的幂等复用：AI 有意重复运行命令仍是新调用。相同提交身份的传输重试与新的业务调用必须区分。

### 原 Queued runner 管理容量与生命周期

选择把 Eclip 的全局容量准入注入现有 `BackgroundCommandManager` 原 runner：start 被接受后进入已有 Queued 并确认接纳，再由同一 runner 等待产品容量和内部执行许可。准入只提供异步获取与持有 guard，不另起应用调度队列。

排队使用原 cell 绝对 deadline；本设计保留当前从 accepted_at 起计算、包括排队时间的 timeout 语义，不悄悄改成进程启动后才计时。排队取消/到期/关闭不启动进程；开始执行后显式 stop_cell、run cancel 或 shutdown 沿原 cleanup 等真实物理结算。普通前台 shell 的既有限时不放宽。后台脱离前台 Turn 的政策保持，不能简单把 Turn token 当 lifetime token。

真实 Started/Queued/Running/Finished 不混称；prepared 记录和日志时间不能冒充进程执行时间。全局 shell=1 不改变，所有 workspace 仍共享原上限。任何路径都恰一次获得/释放许可；原观察和 terminal 持久化失败保留原 owner 债务，不提前释放后伪造结束。

### 保存的分段证据与优化

同一次保存记录脱敏 scope/runtime/operation、共享 deadline 和各阶段进入/离开：epoch、scope/generation 读取、payload 校验、冻结上下文、投影/摘要、序列化、CAS 前/后、transcript apply、ack 与错误清理。使用原运行身份和单调耗时，不记录正文/密钥。先使用隔离复制的 9.5 MB 样本和合成多 scope 数据，原用户数据仅只读。

首段优化保持文件格式、raw/approved 内容与原物理写入路径不变：一次扫描每个 owner 只读/解码一次；同一次原操作复用已解析 payload 和冻结 messages；已有 runtime load 返回的 scope 不重复查询。CAS 的预期 version/epoch 与真实提交校验不省略，I/O 期间不持有 Context 锁，不从缓存承诺提交。

`_scope_index` 是可重建投影，不是 authority，不能直接信任它跳过崩溃恢复。cold reopen、部分 CAS、scope revision 冲突、退休 tombstone、pending transcript 和外部取消继续核对原权威事实。

若分段实测仍不能满足原预算，维持未通过状态并进入同一 FileRuntimeStateAuthority 的缓存/失效补充设计评审；不自动上第二数据库、不删增量、不改格式、不全局放宽 10 秒。受控验收须解释每个耗时阶段和仍有的慢路径，而不是用“加大 timeout 后绿”代替修复。

### 失败与恢复

真实保存失败仍阻止无依据的最终发布，前端展示原 blocked/unknown，不能吞掉异常继续宣称已保存。调用方超时与实际效果未知要分别记录；已经准入的 blocking write 由原 Store 等待/结算和原恢复路径接管，不能因删除 Future 生成第二次无身份写。

恢复只对原 pending operation 做幂等核对；旧确认不覆盖后续内容，回执丢失不触发工具重放。晚到成功、失败、取消均不能改写其它 execution 或释放错误 owner。

## 参考实现与候选取舍

[Tokio Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown) 区分通知取消、由原任务清理、等待 TaskTracker 中任务结束；[Tokio Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) 的异步准入排队和取消规则支持把接纳前撤回与已接纳执行分开。官方正文已通过只读 HTTPS 核对；web 搜索工具 404 不当作浏览成功。

| 候选 | 结论 |
|---|---|
| 提高工具/保存 timeout，或 ShellTool 全局 manages_own_timeout | 不解决回执丢失和准入取消，还会放宽前台合同，不采用。 |
| 应用另起延迟启动队列与 owner | 可暂时回避等待，但复制 supervisor 和取消责任，维护成本高，不采用。 |
| 原 cell runner 加中立异步准入及独立 admission context | 采用；框架拥有机制，应用提供资源政策，复用已有 Queued。 |
| 用 `_scope_index`/新数据库/删除 raw deltas 加速 | 破坏恢复或 provenance，未获设计资格，不采用。 |
| 同操作消除重复读取/解析/冻结 | 采用；先测量、保持格式和权威，仍不达标时不能宣称完成。 |

## 验收标准

| 场景 | 必须成立 |
|---|---|
| shell 槽被长任务占用，连续后台提交 | 后续请求快速取得准确 cell 回执并处于原 Queued；无额外进程、无第二队列。 |
| tracked-capacity 已满，调用方超时/取消 | 容量随后释放也不创建或运行该未接纳任务；无 guard、tracked slot、future 泄漏。 |
| prepared/Started/接纳点逐切点竞争 | 持久失败无执行；接纳前可中止；接纳后原 receipt 可找回，相同传输身份不重复执行。 |
| 排队到期、stop_cell、run cancel、shutdown | 原期限/取消生效，物理退出后释放原许可，terminal 和输出完整，普通 Chat/run-owned 都覆盖。 |
| 前台 shell 与其它工具 | 原限时和权限不放宽，不借本修复统一免除 timeout。 |
| 当前 9.5 MB 与多 scope 保存样本 | 分段证据完整，原 10 秒内在声明硬件/负载条件下正常保存；未满足即保持失败，不改预算作绿。 |
| CAS/epoch 竞争、部分写、ack 丢失、cold reopen | 原成功/冲突/未知分类、精确 operation、恢复幂等和退休围栏保持；不信任不完整投影。 |
| 注入保存慢/失败与工具已执行 | 不重跑工具，不虚报保存成功，不丢已接纳副作用的 owner。 |
| 原业务测试 101 | 与新受控执行回归分列；原微信计划未修复/未验收，不将其计为本计划通过。 |

完整交付区分源码 review、定向回归、真实调用链、App、main CI。当前保存瓶颈仍需实测；本设计不把此缺口写成完成事实。

## 协作、文档与交付边界

长期通用合同与 ADR 归框架；资源政策、普通 Chat/TaskRun owner 接线、配置与用户说明归应用。修改公共 Rust 接缝前盘点整个框架合理消费者，现有 examples 加入编译；SDK pin 和已接受跨语言合同独立，不自动续签依赖风险或升级。官网只同步已验证的行为说明。

本主题以“已接纳后台命令及保存结果始终可追踪”为一个交付结果；计划中的观察、准入、保存和恢复验收都不能省略。推荐先执行本计划，再接缓存和 UI 最终集成；共享源码单一 writer、独立资源窗口，保留所有原日志和自然退出。本轮只输出设计/计划，不执行上述改动、测试、App、提交或发布。
