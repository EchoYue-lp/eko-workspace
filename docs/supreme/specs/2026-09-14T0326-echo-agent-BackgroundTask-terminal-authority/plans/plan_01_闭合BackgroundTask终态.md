---
schema_version: 3
supersedes: null
slug: echo-agent-BackgroundTask-terminal-authority/闭合BackgroundTask终态
goal: 统一BackgroundTask的process-local终态发布、等待、admission、取消、deadline与panic结算
ships: BackgroundTask以单一共享state原子发布status与单消费者result，可克隆handle的多waiter不会挂起，admission/执行共享取消与绝对deadline，execution
  task panic、零并发和提前取消都结算
verify: 并发多waiter、timeout重试、queued cancel/deadline、零并发、execution
  cancel/timeout/panic和type-erased status/retention均有确定性red/green；双语文档、SDK
  intrinsic生成物、orchestration tests、Clippy、semantic/Issue和独立review全部通过
design_ref: docs/supreme/specs/2026-09-14T0326-echo-agent-BackgroundTask-terminal-authority/design.md
delivery_ref: null
todos:
  - id: unify-background-task-state
    files:
      - echo-agent/echo-orchestration/src/tasks/background_task.rs
    summary: 以单一共享state闭合Clone handle、terminal publication和多waiter等待
    verify: 一个waiter取得T，其它waiter立即获得精确terminal反馈；lost
      wakeup不可复现，timeout可重试，type-erased状态不伪造
  - id: supervise-admission-and-execution
    files:
      - echo-agent/echo-orchestration/src/tasks/background_task.rs
    summary: 让admission与execution task共享取消和绝对deadline并监督JoinHandle结算
    verify: queued cancel/deadline、max_concurrent=0、execution
      cancel/timeout/panic全部到达唯一terminal，终态后child task已析构
  - id: document-and-sync-sdk-contract
    files:
      - echo-agent/docs/en/29-long-running-tasks.md
      - echo-agent/docs/zh/29-long-running-tasks.md
      - echo-agent/docs/adr/0039-background-task-terminal-authority.md
      - echo-agent/contracts/sdk
      - echo-agent/sdks/shared
      - echo-agent/echo-sdk-protocol/tests/facade_inventory.rs
      - echo-agent/scripts/check-language-sdks.sh
      - echo-agent/sdks/typescript/test/catalog.test.js
      - echo-agent/sdks/python/tests/test_catalog.py
      - echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java
    summary: 同步BackgroundTask双语合同、ADR与Clone language-intrinsic SDK inventory
    verify: 文档明确Clone与单消费者result、多观察者status和deadline；SDK只增加Rust intrinsic
      identity且三语言facade不扩张
  - id: close-background-task-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: "写入repair、verification和独立复审并只关闭Issue #39对应Finding"
    verify: 目标Finding具备三类关闭refs；92个Finding与Issue一对一；其它74个open Finding不变；semantic
      gates通过
lifecycle: superseded
artifact_id: plan:200471c3-c48f-4848-bc3a-8f6990b44c21
design_revision: sha256:699d382cd0fe819c2ae321418af4e5ab9f20cc37170497063ae1186b205077bf
---
## Approach

- 用内部`BackgroundTaskState<T>`和type-erased只读status source替换分离的status/result/terminal flag；所有handle、list、prune只读同一state。
- 手工实现无`T: Clone`约束的`Clone for BackgroundTask<T>`；result仍只take一次，后续waiter从永久terminal status立即返回明确错误。
- `wait`按Tokio官方模式先创建并enable Notified再读state，每次调用只计算一次absolute timeout deadline，通知循环不延长预算。
- `TaskSpawner::spawn`从接纳时建立可选absolute deadline；admission和execution使用biased cancel/deadline/normal优先级，零并发直接Failed。
- 用户future在受监督execution task中运行；supervisor通过JoinHandle取得Ok/Err/panic，cancel/timeout先abort并await child task，再原子发布terminal。
- 同步双语long-running-task文档与ADR0039；刷新SDK inventory/manifest/shared digest，只接受新增Clone trait impl的language-intrinsic变化。
- 独立review后只关闭`finding.background-task-wait`；Issue #39保持open直到本地提交进入远端main。

## Global Constraints

- TaskSpawner/BackgroundTask只管理process-local future，不得复制RevisionedTaskStore、RuntimeTaskService或Task DAG状态机。
- BackgroundTaskState是status/result唯一权威；type-erased registry不得保留terminal bool或合成Completed fallback。
- Terminal在单一state临界区写result和status，锁释放后notify；重复settle不覆盖首个terminal。
- BackgroundTask Clone不得要求`T: Clone`；T仍是单消费者，status是多观察者，后续wait不得挂起或伪造T。
- Wait必须先enable notification再inspect state；单次wait timeout使用一次absolute deadline且不消费result。
- Spawn deadline覆盖admission与execution；cancel优先于deadline，deadline优先于同时ready的normal completion。
- Cancel/timeout必须abort并await child task后才发布terminal；JoinError panic必须映射Failed并可由is_panicked观察。
- `max_concurrent=0`不得panic、永久Pending或静默改成1，必须形成可观察Failed handle。
- 不新增public方法、状态variant、依赖、持久schema或Task关系；只新增既有文档承诺的Clone trait impl。
- SDK新增identity必须是Rust language_intrinsic；不得顺带映射BackgroundTask deferred或Host-only项，三语言facade保持不变。
- docs/en与docs/zh保持对等；echo-agent-cli与echo-website不修改。
- 不使用unwrap/expect/panic/unreachable/todo、UTF-8字节切片或禁用术语；内部执行单元命名为`execution_task`或`child_task`。
- Issue #39已存在且唯一；本Plan只关闭其本地Finding，远端Issue等待main交付。

## Files

- Modify: `echo-agent/echo-orchestration/src/tasks/background_task.rs` — shared state、Clone、wait、admission、execution supervision、terminal与测试。
- Modify: `echo-agent/docs/en/29-long-running-tasks.md` — process-local handle、single-consumer result、多观察者status与deadline。
- Modify: `echo-agent/docs/zh/29-long-running-tasks.md` — 同步中文合同。
- Create: `echo-agent/docs/adr/0039-background-task-terminal-authority.md` — 状态权威、Notify/JoinHandle、兼容与回滚。
- Modify: `echo-agent/contracts/sdk` — 刷新Rust public inventory与parity manifest。
- Modify: `echo-agent/echo-sdk-protocol/tests/facade_inventory.rs` — 人工复核后刷新intrinsic membership冻结摘要。
- Modify: `echo-agent/sdks/shared` — 刷新共享contract digest/catalog生成物。
- Modify: `echo-agent/scripts/check-language-sdks.sh` — 刷新经人工复核的canonical scope计数。
- Modify: `echo-agent/sdks/typescript/test/catalog.test.js` — 同步language-intrinsic计数。
- Modify: `echo-agent/sdks/python/tests/test_catalog.py` — 同步language-intrinsic计数。
- Modify: `echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java` — 同步language-intrinsic计数。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit和Finding状态。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一frontier。

## Reuse

- `echo-agent/echo-orchestration/src/tasks/background_task.rs` — 现有public status、spawn/wait/cancel、retention与Issue #39反例。
- `echo-agent/echo-orchestration/src/tasks/command_cell.rs` — 先注册waiter再读snapshot和absolute admission deadline的可工作模式；不复制cell资源模型。
- Tokio `Notify::Notified::enable` — 多receiver防lost wakeup。
- Tokio `JoinHandle` — panic JoinError、cancel-safe mutable await与析构完成边界。
- `echo-agent/scripts/check-sdk-contracts.sh`和`export-language-sdk-catalog.sh` — Rust inventory、scope与shared digest唯一生成链。
- `echo-agent/echo-sdk-protocol/tests/facade_inventory.rs` — intrinsic route membership冻结门禁，防止未复核的Rust-only API漂移。
- `echo-agent/scripts/check-language-sdks.sh`与三语言catalog tests — canonical scope总量冻结consumer，只同步language-intrinsic +1。
- `audit.task-subagent-workflow.time-lifecycle`、Issue #39与双语第29章 — 故障证据和公共合同。

## Todos

### unify-background-task-state

requirements:
- § 目标行为
- § 系统边界
- § 核心结构与数据流
- § 异常和边界场景

interfaces:
- consumes: BackgroundTaskStatus、现有BackgroundTask public方法、Notify和type-erased AnyBackgroundTask registry。
- produces: BackgroundTaskState<T>、shared status source、Clone handle、一次result消费与可重复terminal观察。

steps:

1. 添加并发双waiter red，证明旧实现一个take T后另一个永久等待；补type-erased Failed/Cancelled精确状态反例。
   verify: red来自真实wait/status路径和bounded外层timeout，不依赖sleep或手工修改私有状态。
   expected: lost wakeup与多观察者缺口有直接失败证据。

2. 用一个共享state替换分离status/result/terminal flag，并实现无T Clone约束的handle Clone。
   verify: 所有status/list/prune/wait从同一state读取；terminal result与status原子提交且只发布一次。
   expected: 一个process-local terminal authority，不再有合成Completed或平行flag。

3. 按enable-before-inspect和absolute wait deadline重写wait。
   verify: 并发waiter一个取得T、另一个立即terminal error；timeout后重试取得T；通知竞争不延长预算。
   expected: 任意完成时序都不会丢wake或永久等待。

### supervise-admission-and-execution

requirements:
- § 目标行为
- § 核心结构与数据流
- § 异常和边界场景
- § 复用与实现约束

interfaces:
- consumes: unified shared state、CancellationToken、Semaphore、absolute deadline与JoinHandle。
- produces: admission/execute共用supervisor、确定性terminal优先级与child-task settlement。

steps:

1. 添加queued cancel、queued deadline和zero-concurrency red，记录用户future是否启动。
   verify: 旧实现cancel/deadline不结算或零并发永久Pending；测试以condition signal和bounded timeout观察，不靠固定sleep。
   expected: admission缺口有可重复非零证据。

2. 建立spawn-time deadline与cancel-aware admission，所有early terminal走唯一settle helper。
   verify: queued cancel为Cancelled，queued deadline/zero config为Failed，execution task未启动，list与handle status一致。
   expected: Pending不再成为无界状态。

3. 通过受监督execution task闭合返回、取消、timeout与panic。
   verify: Ok/Err/panic映射Completed/Failed；cancel/timeout abort+await后才terminal；is_panicked与status一致；同时ready优先级确定。
   expected: user future不会因supervisor select丢handle而detached，所有路径exactly-one terminal。

### document-and-sync-sdk-contract

requirements:
- § 关键取舍与业界依据
- § 公共合同与SDK影响
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: 最终BackgroundTask行为、Clone trait impl和SDK scope分类。
- produces: 双语文档、Accepted ADR0039、更新后的Rust inventory/parity manifest与shared digests。

steps:

1. 更新双语第29章并创建ADR0039。
   verify: Clone、single-consumer result、多观察者status、deadline/cancel/panic、process-local边界和回滚在en/zh一致，documentation contract通过。
   expected: 文档不再隐含所有clone都能取得同一个T，也不把TaskSpawner误述为durable Task。

2. 用唯一生成链刷新SDK artifacts并检查scope。
   verify: 新增BackgroundTask Clone identity分类为language_intrinsic并刷新经人工复核的intrinsic membership摘要；canonical/external/Host/deferred既有项无非预期迁移；三语言catalog不新增BackgroundTask facade。
   expected: 真实公共Rust变化可追踪，但不恢复逐identity主线。

3. 运行BackgroundTask/TaskSpawner、workspace相关、Clippy、panic-policy、feature、docs和SDK门禁。
   verify: 所有适用命令exit 0；无禁用API、术语、warning或artifact drift。
   expected: process-local修复不回归Task DAG、CommandCell或SDK contract。

### close-background-task-finding

requirements:
- § 验收标准
- § 复用与实现约束
- 用户要求语义修复严格执行一Finding一Issue。

interfaces:
- consumes: red/green日志、ADR0039、SDK diff、最终实现、Issue #39与独立review。
- produces: resolved BackgroundTask Finding、repair/verification Evidence、rereview Audit和MASTER-PLAN状态。

steps:

1. 刷新Task/Subagent map、behavior/rule/assets与source snapshot，写repair/verification Evidence。
   verify: before绑定`81e2756c`，after绑定current source digest；Issue #39 URL唯一；其它Finding状态不变。
   expected: Pending/Running/terminal、result、wait、admission和execution settlement可追踪。

2. 独立review后写rereview Audit并执行最终语义与Issue门禁。
   verify: reviewer无blocker；Finding具备repair/verification/rereview refs；92/92 Issue映射、strict snapshot、change evidence和git diff check通过。
   expected: Finding本地resolved，Issue #39仍open等待远端main交付。

## Decisions

- Clone兑现既有公共文档，但T保持单消费者；后续handle只做terminal观察。
- Shared generic state是唯一terminal authority；type-erased registry只持有同一state的status view。
- Deadline覆盖admission和execution且不重置；cancel在同时ready时优先。
- 用户future由supervisor持有JoinHandle，取消/超时先abort+await再terminal。
- Clone identity只进入Rust language intrinsic SDK scope，不扩三语言facade。
- 本Plan只关闭Issue #39。
