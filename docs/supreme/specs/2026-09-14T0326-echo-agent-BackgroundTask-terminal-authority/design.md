---
title: echo-agent BackgroundTask terminal authority 设计
artifact: design
carrier: markdown
---

# echo-agent BackgroundTask terminal authority 设计

## 问题与目标

`TaskSpawner`是framework内process-local、pollable、cancellable future handle，不是revisioned Task DAG。当前`BackgroundTask<T>`却存在同一生命周期内的多套弱权威：Tokio RwLock status、Mutex result、Notify、terminal bool与未消费JoinHandle。它们的发布顺序和观察方式不闭合：wait先读result后创建Notified，`notify_waiters`可丢wake；第一个waiter take结果后其它waiter永久等待；user future panic绕过status/result；semaphore admission不监听cancel或deadline；`max_concurrent=0`可永久Pending。Type-erased fallback还可能把真实Failed/Cancelled临时伪装成Completed。

源码与双语正式文档同时声明BackgroundTask handle可克隆，但类型没有Clone实现。目标是在不引入durable Task状态机、不改变`wait -> Result<T>`签名的前提下，建立一个process-local terminal authority：status和result原子发布，多个handle/waiter不会挂起，admission/execute共用cancel/deadline，panic/cancel/timeout都结算，SDK只记录真实Rust intrinsic变化。

## 目标行为

- `BackgroundTask<T>`实现无`T: Clone`约束的Clone；克隆共享同一task identity、state、notification和cancellation scope。
- 一个内部`BackgroundTaskHandleState<T>`原子保存`BackgroundTaskStatus`和可选`Result<T>`；它是handle wait/status与type-erased list/prune的唯一状态源。
- TaskSpawner的type-erased entry通过内部只读status source访问同一state；删除平行terminal bool和锁忙时伪造Completed/Running的fallback。
- Terminal publication在一个state临界区内同时写result和status，释放锁后才`notify_waiters`；所有terminal只发布一次。
- `wait(timeout)`先创建并`enable()` Notified，再读state。若有result，第一个waiter取得原值；若result已被另一个waiter消费但status已terminal，当前waiter立即得到status-derived错误，不再次睡眠。
- Failed和Cancelled后续waiter得到保留原错误/取消语义；Completed后续waiter得到明确“结果已由另一waiter消费”的错误。任一后续wait都不会把Completed伪装成新结果。
- 单次wait的timeout是从调用开始计算的绝对deadline；任何通知/竞争重试都不重置预算，timeout不消费result，调用方仍可重试。
- `default_timeout_secs`从spawn接纳时开始，覆盖semaphore admission和child task execution。排队、执行和清理共享一个绝对deadline。
- Admission同时监听cancel、deadline和semaphore close；取消在开始前结算Cancelled，deadline/closed semaphore结算Failed。
- `max_concurrent=0`保持`TaskSpawner::new -> Self`兼容，但spawn出的handle立即结算为Failed配置错误，不永久Pending，也不静默提升为1。
- 用户future由内层Tokio execution task执行，外层supervisor持有并await JoinHandle。Execution task panic由JoinError转为Failed；cancel/timeout会abort并await child task后再发布terminal，避免detached async effect。
- `is_panicked`从同一terminal status判断：Running/Pending为None，caught execution-task panic为Some(true)，其它terminal为Some(false)。

## 范围与非目标

范围包括`echo-orchestration` BackgroundTask/TaskSpawner内部状态、wait/admission/execution-task监督、Clone合同、type-erased status、retention/cancel tests、双语long-running-task文档、ADR、SDK inventory/digest、语义对象和Issue #39。

以下不在本设计内：

- 不替换`TaskRevisionService`、`RuntimeTaskService`或任何durable TaskRun graph。
- 不让多个waiter都获得同一个`T`；这需要`T: Clone`或改变返回类型，留给未来独立API设计。
- 不增加BackgroundTaskStatus变体、wire字段、持久store、retry、resume、checkpoint或跨进程恢复。
- 不增加TaskSpawner shutdown/close API；runtime teardown和owner drop由独立lifecycle工作处理。
- 不修CommandCell retention/cancel、BackgroundReview handle、scheduler或其它spawn路径。
- 不修改echo-agent-cli或echo-website；website继续按当前冻结策略延后。

## 系统边界

`BackgroundTaskHandleState<T>`只拥有一个进程内future的status与单消费者result。`TaskSpawner`拥有admission、execution-task supervision、type-erased registry和bounded terminal history。CancellationToken只表达请求；supervisor负责把请求转成child-task abort/join与terminal。Durable dependencies、claims、retries和restart recovery继续属于revisioned Task runtime。

```text
TaskSpawner.spawn
  |
  +-- shared state(Pending, no result) -- cloneable BackgroundTask<T>
  +-- supervisor
        |
        +-- cancel --------------------------> settle Cancelled
        +-- absolute deadline ---------------> settle Failed(timeout)
        +-- max_concurrent=0 ----------------> settle Failed(config)
        +-- acquire permit -> state Running
                               |
                               +-- spawn execution-task JoinHandle<Result<T>>
                               +-- cancel/timeout -> abort + await -> terminal
                               +-- join ok -> Completed/Failed
                               +-- join panic -> Failed(panic)

waiter: create Notified -> enable -> inspect same state -> take result OR terminal error OR await
```

## 核心结构与数据流

共享state使用短临界区的标准同步Mutex；其中只做clone status、take result和terminal写入，不跨await，也不执行用户代码。一个内部type-erased status source trait由generic shared state实现，使`AnyBackgroundTask`继续保持现有public API，同时list、status_text、status_snapshot、prune都读取同一真实status。

Supervisor在spawn时计算可选Tokio absolute deadline。Admission通过biased select按cancel、deadline、permit顺序结算；成功取得permit后发布Running并spawn execution task。Execution同样按cancel、deadline、child-task completion顺序结算；cancel/timeout分支abort child task并await析构完成后才写terminal。Tokio保证await JoinHandle时execution task析构已完成，且`&mut JoinHandle`在select中cancel-safe。

Terminal helper在state锁内检查非terminal、写入result和status；重复settle不覆盖首个terminal。锁释放后调用`notify_waiters`，所以被唤醒者看到完整terminal。Wait使用Tokio文档推荐的`Notified::enable`顺序，避免多并发receiver在检查与注册之间丢wake。

Clone手工复制id/name与Arc/CancellationToken，不对`T`增加Clone约束。`Result<T>`仍只被take一次；status永久保留，以便其它handle做精确terminal观察。

## 异常和边界场景

- Task在waiter检查前完成：enabled Notified或直接state读取都能观察terminal。
- 两个waiter并发：都在notify list；一个take result，另一个重读terminal并立即返回已消费/失败/取消错误。
- Waiter自身timeout：不改task state，不消费result；后续wait从同一absolute call-local新deadline继续。
- 多次通知或竞争循环：当前wait deadline不延长。
- Spawn后立即cancel且尚未取得permit：Cancelled，execution task不启动。
- 排队超过default timeout：Failed timeout，execution task不启动。
- `max_concurrent=0`且timeout禁用：仍立即Failed配置错误。
- Semaphore关闭：Failed且result/status/list一致。
- Execution task返回Ok：Completed与原始T一起原子发布。
- Execution task返回Err：Failed保留错误文本；首个wait得到原始ReactError，后续wait得到status-derived错误。
- Execution task panic：JoinError转为Failed并可由`is_panicked`识别；supervisor本身正常结束。
- Cancel/timeout与child-task completion同时ready：biased优先级使持久cancel优先，其次deadline，最后execution completion；不会因select随机性产生两种terminal。
- Type-erased list在terminal写入期间：标准state锁等待极短临界区，随后返回真实状态，不伪造Completed。
- Retention：只按同一state的terminal判断，Pending/Running仍不删除。

## 关键取舍与业界依据

Tokio Notify官方文档明确要求多并发receiver在检查共享容器前调用`Notified::enable()`，否则多个send只保存一个permit，另一个receiver可永久睡眠。Tokio JoinHandle文档说明它是task的owned join permission、drop会detach、`&mut JoinHandle`在select中cancel-safe，并保证await观察完成前task析构已结束；panic通过JoinError返回。OpenAI Codex的后台TUI请求把异步结果发送回单线程AppEvent reducer，而不是让多个消费者竞争取走同一个future返回值，支持“结果一次提交、状态多观察者读取”的边界。

- Tokio Notify source and multi-receiver `enable`: <https://github.com/tokio-rs/tokio/blob/master/tokio/src/sync/notify.rs>
- Tokio JoinHandle contract: <https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html>
- OpenAI Codex background requests: <https://github.com/openai/codex/blob/main/codex-rs/tui/src/app/background_requests.rs>

把`wait`改成`Arc<T>`或要求`T: Clone`被排除，因为会破坏所有现有Rust调用和真实SDK contract；当前注释已经预期首个consumer后使用status reporting。保留Tokio RwLock+result Mutex+terminal bool被排除，因为三套发布点无法形成一个原子terminal，fallback还会改变Failed/Cancelled含义。单纯把`notify_waiters`换成`notify_one`被排除，因为多个waiter仍竞争一个permit且不能解决result/status。给零并发静默加到1被排除，因为调用方配置错误会被隐藏。

## 公共合同与SDK影响

`TaskSpawner::new/spawn`、`BackgroundTask::wait/status/cancel/is_panicked`、BackgroundTaskStatus与TaskSpawnerConfig签名不变。新增`Clone for BackgroundTask<T>`兑现现有双语公开承诺；该trait impl在SDK inventory中属于Rust `language_intrinsic`，不要求TypeScript、Python或Java facade实现。

SDK生成链必须刷新Rust public API inventory、parity manifest与共享digest，并证明新增identity只进入language-intrinsic scope。不得借机映射其它BackgroundTask deferred/Host-only项。

## 复用与实现约束

- 复用CancellationToken、Semaphore、Notify/Notified::enable、Tokio absolute deadline与JoinHandle，不新增依赖。
- 复用CommandCell“先注册waiter再读snapshot”的本仓模式，但不复制CommandCell store、cursor、artifact或lease模型。
- 复用BackgroundTaskStatus和现有public方法；只增加Clone trait impl，不新增状态或方法。
- Result<T>是单消费者，status是可重复多观察者；文档必须明确两者差异。
- Terminal helper是status/result唯一写入口；所有admission、cancel、timeout、execution error/panic分支复用。
- Supervisor必须await aborted child task后再发布terminal；不得drop JoinHandle形成detached user future。
- Deadline从spawn接纳时计算且不重置；wait deadline从每次wait调用时计算且不重置。
- SDK只更新生成物与intrinsic分类；三语言状态不改。
- 不使用unwrap、expect、panic、unreachable、todo、UTF-8字节切片或禁用术语；Tokio内部执行单元统一使用`execution_task`或`child_task`命名。

## 验收标准

- 有效red证明两个并发waiter中旧实现有一个永久等待；修复后一个取得T、另一个立即得到Completed已消费错误，两个handle状态一致。
- Deterministic admission tests证明queued cancel、queued deadline和max_concurrent=0都结算且用户future未启动。
- User future panic通过受监督JoinHandle形成Failed，`wait`返回错误、`status`为Failed、`is_panicked=Some(true)`；测试不得在本项目源码中新增panic API，可通过已存在的panic test helper或受控aborted JoinError覆盖可测试分支，真实panic路径由代码审查与Tokio合同证明。
- Cancel/timeout在execution task仍活跃时abort并await，terminal返回后可观察析构完成；同时ready时terminal优先级确定。
- Retry wait timeout仍不消费最终result；通知竞争不延长wait deadline。
- Type-erased list精确显示Completed/Failed/Cancelled，retention不删除Pending/Running。
- 双语文档明确process-local边界、Clone、单消费者result、多观察者status和deadline范围；documentation contract通过。
- SDK gate确认新增Clone identity为language_intrinsic，三语言artifacts不新增BackgroundTask facade。
- BackgroundTask/TaskSpawner、workspace相关回归、Clippy/panic-policy、feature、docs、SDK、semantic strict/change-evidence、Issue reconciliation和独立review全部通过。
