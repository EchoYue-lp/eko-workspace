---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/修复TaskPatch覆盖Claim竞态
goal: 让revisioned Task relation patch在提交时拒绝任何自其读取快照后发生的runtime execution变化
ships: Task relation patch commit 同时校验其读取时的 execution snapshot，runtime
  claim、retry 或 settlement 的并发变化返回 typed conflict 且不覆盖 live claim
verify: 确定性交错测试先复现load Pending -> runtime claim -> patch
  commit覆盖问题，再证明Skip/SetStatus/Update等patch返回RevisionConflict且live
  claim仍可settle；正常patch、runtime
  claim/retry/settle与现有Task行为不回归，ADR/语义Finding同步，focused Rust、subagent
  feature、Clippy、semantic-diff/verify与独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-task-patch-claim-cas
todos:
  - id: add-execution-precondition
    files:
      - echo-agent/echo-orchestration/src/tasks/revisioned.rs
      - echo-agent/docs/adr/0008-canonical-runtime-task-authority.md
    summary: 为TaskGraphCommit增加读取时execution precondition并明确Store CAS合同
    verify: TaskRevisionService对relation
      patch携带完整TaskId->TaskExecution期望；create不需要precondition；Store在revision相同但execution漂移时返回typed
      Conflict
  - id: prove-race-and-compatibility
    files:
      - echo-agent/echo-orchestration/src/tasks/revisioned.rs
    summary: 用确定性交错测试覆盖claim/retry/settlement与正常patch
    verify: 旧实现上的load->claim->commit测试先失败；修复后并发patch不覆盖claim，正常manual
      progress与idempotent running行为保持
  - id: close-semantic-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 记录repair/verification/rereview证据并关闭task-patch-claim-race
    verify: Finding仅在实现、focused
      tests、独立review和semantic-verify通过后resolved；其它Task/Subagent/Workflow
      Findings保持open
artifact_id: plan:0e5c6e44-158c-42fd-97fd-8079b36f5988
lifecycle: superseded
design_revision: null
---
## Context

- Plan 04在`audit.task-subagent-workflow.state-authority`确认：Task patch读取Pending revision N后，runtime可在不递增graph revision的情况下claim为Running；SetStatus/Skip等patch又被execution drift检查豁免，最终可覆盖live claim。
- 当前graph revision只拥有relation/spec版本，runtime claim/retry/settle拥有同一snapshot内的execution状态。让每次claim递增graph revision会把两种authority重新耦合并扰乱runtime safe point，因此不采用。
- ADR 0008已确认`TaskRevisionService`与`RuntimeTaskService`共享同一graph但职责不重叠；本修复扩展现有commit CAS，不新增store、validator或executor。
- [RFC 9110 If-Match](https://www.rfc-editor.org/rfc/rfc9110.html#name-if-match)要求状态变更在validator不匹配时拒绝，以防lost update；[Kubernetes API resourceVersion](https://kubernetes.io/docs/reference/using-api/api-concepts/#resource-versions)同样要求客户端用读取时版本做条件更新并处理409 conflict。

## Approach

- 给`TaskGraphCommit`增加可序列化的optional execution precondition，内容是patch读取时全部现有`TaskId -> TaskExecution`。Create commit没有既有graph，保持None。
- `TaskRevisionService::apply_patch_to_loaded`在应用operation前从`current.snapshot.tasks`捕获precondition；候选仍可按patch修改execution。
- `RevisionedTaskStore::compare_and_commit`先检查graph revision，再对当前execution map与precondition做exact equality。任何claim、retry、settlement或其它runtime mutation都会冲突；Store不得以“patch也修改该task”为由跳过。
- 保留旧commit无precondition的兼容分支和现有drift检查，避免一次修复强迫外部store立即重写所有非patch调用；canonical TaskRevisionService路径必须始终提供precondition，ADR记录外部store实现责任。
- 不把precondition塞进TaskPatchEffects，不把runtime状态混入relation revision，不合并TaskRevisionService与RuntimeTaskService。
- 测试直接控制store顺序形成`load -> claim -> commit`，不依赖sleep；同时覆盖正常patch和live claim仍可settle。
- 先让新回归测试在旧Store逻辑上失败，再实现CAS并通过，符合高风险并发修复门禁。

## Global Constraints

- TaskRun、PlanTask、SubagentRun关系和TaskStatus状态机不改变；不新增Worker术语。
- graph revision继续只表示relation/spec commit；runtime claim/retry/settle不递增它。
- execution precondition只用于conditional commit，不成为第二持久状态或UI projection。
- 所有TaskId查找使用安全迭代/get，不引入unwrap/expect/index/panic。
- External`RevisionedTaskStore`实现必须能观察并尊重precondition；兼容分支只服务没有relation patch precondition的旧/direct commit。
- 不修改echo-agent-cli、echo-website、SDK contract或examples；公共contract变化更新ADR 0008和语义材料。
- 失败测试、实现、focused验证、独立review和semantic-verify缺一不可；Finding关闭不代表其它Task runtime Finding关闭。

## Files

- Modify: `echo-agent/echo-orchestration/src/tasks/revisioned.rs` — 增加execution precondition、Store校验与确定性交错测试。
- Modify: `echo-agent/docs/adr/0008-canonical-runtime-task-authority.md` — 记录relation revision与runtime execution双precondition合同及业界依据。
- Modify: `echo-agent/.echo-semantic` — 更新Task map/behavior/rule/evidence、repair/verification/rereview证据和Finding状态。
- Modify: `docs/MASTER-PLAN.md` — 更新首个Finding repair的真实状态与下一frontier。

## Reuse

- `TaskRevisionService::apply_patch_to_loaded` — 唯一relation patch候选与commit入口。
- `TaskGraphCommit`与`RevisionedTaskStore::compare_and_commit` — 现有conditional commit边界，不创建第二CAS API。
- `TaskExecution`与`TaskClaim` — 复用完整typed execution事实，不发明近义revision。
- `RevisionedTaskStoreError::Conflict`与`TaskRevisionError::RevisionConflict` — 复用现有typed conflict。
- `audit.task-subagent-workflow.state-authority`与`finding.task-patch-claim-race` — 当前故障假设、证据和关闭目标。
- ADR 0008 — 继续作为canonical runtime Task authority，不新建平行架构决策。

## Todos

### add-execution-precondition

requirements:
- 用户要求：同一种Task DAG、revision、retry/cancel authority只能有一个实现，修复以Finding驱动。
- ADR 0008：relation commit与runtime execution共享graph但必须拒绝stale/superseded状态。
- RFC 9110/Kubernetes：状态变更必须携带读取时validator并在不匹配时返回conflict。

interfaces:
- consumes: `RevisionedTaskGraph` loaded snapshot、`TaskGraphCommit`、`TaskExecution`、existing store conflict。
- produces: optional execution precondition与canonical patch commit CAS合同。

steps:

1. 在不改状态机的前提下给commit加入读取时execution map，并由apply_patch_to_loaded完整填充。
   verify: create commit为None；所有relation patch为Some且包含加载时全部现有TaskId/execution。
   expected: candidate可以表达“我要改什么”和“我基于什么execution事实”两套不混淆的信息。

2. Store在revision匹配后先校验execution precondition，再应用现有relation drift/next revision规则。
   verify: 任一Task execution的status/claim/retry/error/timestamps变化都会得到现有typed Conflict。
   expected: runtime变化无法被Skip/SetStatus/reset豁免覆盖，graph revision语义不变。

### prove-race-and-compatibility

requirements:
- Audit已确认load->claim->commit精确交错未被现有测试覆盖。
- Rust硬约束：异常路径返回Result，不使用panic API。

interfaces:
- consumes: execution precondition commit合同与InMemoryRevisionedTaskStore。
- produces: 确定性race回归和正常路径兼容证据。

steps:

1. 先增加直接控制store的regression：加载Pending，构造stale patch candidate，runtime claim，再提交candidate。
   verify: 在修复实现前测试观察旧逻辑接受commit并覆盖claim，因此失败；保存失败日志。
   expected: 故障无需sleep或概率调度即可稳定复现。

2. 实现后验证Skip、SetStatus以及spec Update遇到runtime claim/retry/settlement都返回RevisionConflict，且原claim仍可正常settle。
   verify: targeted tests逐一断言Conflict、current claim和最终settlement；正常manual progress/idempotent running tests继续通过。
   expected: relation patch与runtime mutation线性化，不改变合法单线程行为。

3. 执行focused工程验证。
   verify: `cargo fmt --all -- --check`、`cargo clippy -p echo_orchestration --all-targets --locked -- -D warnings`、`cargo test -p echo_orchestration revisioned --locked`、`cargo check -p echo_agent --no-default-features --features subagent --locked`全部exit 0。
   expected: crate、root facade和subagent feature消费者编译/测试通过。

### close-semantic-finding

requirements:
- Echo Semantic：代码首个diff后执行semantic-diff，完成前执行semantic-verify。
- Finding resolved必须有repair、verification和rereview证据，不得只改status。

interfaces:
- consumes: 最终代码diff、失败/通过日志、ADR 0008与独立review。
- produces: 当前source digest的Task语义对象、repair/verification Evidence、rereview Audit和resolved Finding。

steps:

1. 刷新Task boundary对象并写入repair/verification evidence，保留其它open Finding。
   verify: semantic-diff只关闭task-patch-claim-race，TaskClaim->SubagentAttempt、factory、workflow等Finding仍open。
   expected: resolved Finding能追到before/after行为、命令结果和回滚点。

2. 执行独立review与semantic-verify。
   verify: reviewer无blocker，strict snapshot/change evidence与git diff check通过；代码差异仅在计划路径。
   expected: repair可独立提交和停止，MASTER-PLAN准确显示1个Finding关闭而非整个runtime完成。

## Decisions

- 使用execution precondition，不让runtime mutation递增graph revision。
- Precondition存储完整typed`TaskExecution`而非脆弱字符串hash，便于Store实现精确比较和测试。
- Canonical TaskRevisionService必须提供precondition；None兼容分支不被描述为安全patch路径。
- 复用现有Conflict错误，不新增第二错误/状态机。
- 本次只关闭task-patch-claim-race；相邻attempt link与factory Findings不顺带修复。