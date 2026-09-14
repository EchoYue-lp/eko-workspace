---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/修复TaskPatch覆盖Claim竞态并同步SDK
goal: 让revisioned Task relation patch以读取时execution snapshot做条件提交，并同步这一public
  adapter contract的SDK漂移资产
ships: Task relation patch 以读取时 execution snapshot 做条件提交并拒绝覆盖 runtime 变化，同时把新增的
  public commit precondition 同步到 SDK inventory、operation catalog 和三语言合同
verify: 确定性red/green交错覆盖claim/retry/settlement与正常patch；TaskGraphCommit
  public字段进入唯一Rust/SDK inventory并将canonical总量更新为9683、external
  contract为5607，其他scope保持1765/780/90/1441；生成artifact、operation catalog/source
  digest、三语言contract与真实Host门禁、ADR/语义Finding、Clippy/feature check和独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-task-patch-claim-cas-sdk-contract
todos:
  - id: implement-execution-precondition
    files:
      - echo-agent/echo-orchestration/src/tasks/revisioned.rs
      - echo-agent/docs/adr/0008-canonical-runtime-task-authority.md
    summary: 实现canonical relation patch execution CAS并覆盖确定性交错
    verify: TaskRevisionService patch总是携带完整expected executions；Store exact
      compare后Conflict；claim/retry/settlement不被覆盖，正常patch不回归
  - id: synchronize-public-sdk-contract
    files:
      - echo-agent/contracts/sdk
      - echo-agent/sdks/shared
      - echo-agent/echo-sdk-protocol/tests/facade_inventory.rs
      - echo-agent/scripts/check-language-sdks.sh
      - echo-agent/sdks/typescript/test/catalog.test.js
      - echo-agent/sdks/python/tests/test_catalog.py
      - echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java
      - echo-agent/docs/adr/0032-sdk-contract-scope-classification.md
      - echo-agent/docs/sdk/README.md
    summary: 通过唯一生成器同步TaskGraphCommit新字段及SDK scope计数
    verify: 不手改generated artifacts；新字段是value:task external
      contract，canonical总量+1且external+1；extension protocol不变，SDK
      contract/language gates全绿
  - id: close-semantic-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入repair、verification、rereview证据并只关闭task-patch-claim-race
    verify: Finding resolved具备repair/verification/rereview refs；其它Task/SDK
      Findings保持open；semantic-diff/verify和独立review通过
artifact_id: plan:33b785e3-9b52-4839-a8e6-ed07f3468779
lifecycle: completed
design_revision: null
---
## Context

- Plan 06已取得稳定失败证据：旧实现接受`load Pending -> runtime claim -> stale Skip commit`并覆盖live claim。
- `TaskGraphCommit`是root facade的public `value:task`合同；Plan 06禁止SDK contract差异，已标记superseded。本Plan绑定包含SDK同步的替代outcome并接续当前WIP。
- ADR 0008、RFC 9110 If-Match和Kubernetes resourceVersion共同要求条件写拒绝读取后的并发状态变化。

## Approach

- `TaskGraphCommit.expected_executions`保存patch读取时完整`TaskId -> TaskExecution`；create/legacy direct commit可None，canonical patch总是Some。
- Store先比较graph revision，再exact比较execution map；claim/retry/settlement/status/error/timestamp任一变化都返回现有Conflict。Patch修改目标不豁免。
- graph revision仍只表示relation/spec commit，runtime mutation不递增；不新增store、validator、executor或状态机。
- 保留red日志并覆盖claim、SetStatus、spec Update、settlement、retry交错。
- 新public字段通过唯一generator同步public-api、manifest/schema、operation catalog、source/shared digest；预计canonical 9683、external 5607，其余scope不变。
- 三语言复用既有value:task route，不建立第二Task authority。最终只关闭`finding.task-patch-claim-race`。

## Global Constraints

- TaskRun、PlanTask、SubagentRun、TaskStatus与ready frontier不变；只使用Subagent术语。
- Execution precondition是conditional input，不是第二持久状态/UI projection；canonical patch不得None。
- External RevisionedTaskStore必须尊重precondition；None兼容分支不描述为安全patch。
- 不新增unwrap/expect/index/panic；generated SDK文件只由既有脚本产生，extension protocol不变。
- echo-agent-cli与echo-website不修改；它们不构造该低层commit，交付说明记录检查结果。
- Finding resolved必须有red、green、独立review与semantic证据；相邻Finding保持open。

## Files

- Modify: `echo-agent/echo-orchestration/src/tasks/revisioned.rs` — precondition、CAS与测试。
- Modify: `echo-agent/docs/adr/0008-canonical-runtime-task-authority.md` — 条件提交合同。
- Modify: `echo-agent/contracts/sdk` — 生成public-api、manifest/schema、catalog和source digest。
- Modify: `echo-agent/sdks/shared` — 生成shared catalog/digest。
- Modify: `echo-agent/echo-sdk-protocol/tests/facade_inventory.rs` — 更新scope snapshot与新字段合同。
- Modify: `echo-agent/scripts/check-language-sdks.sh` — 更新scope计数。
- Modify: `echo-agent/sdks/typescript/test/catalog.test.js` — 更新TypeScript计数。
- Modify: `echo-agent/sdks/python/tests/test_catalog.py` — 更新Python计数。
- Modify: `echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java` — 更新Java计数。
- Modify: `echo-agent/docs/adr/0032-sdk-contract-scope-classification.md` — 更新统计。
- Modify: `echo-agent/docs/sdk/README.md` — 更新SDK统计。
- Modify: `echo-agent/.echo-semantic` — repair/verification/rereview与current digest。
- Modify: `docs/MASTER-PLAN.md` — repair进度。

## Reuse

- `TaskRevisionService::apply_patch_to_loaded`、`TaskGraphCommit`、`RevisionedTaskStore::compare_and_commit` — 唯一conditional commit。
- `TaskExecution`、`TaskClaim`、`RevisionedTaskStoreError::Conflict` — typed execution与conflict。
- ADR 0008、`audit.task-subagent-workflow.state-authority`、`finding.task-patch-claim-race` — authority与关闭目标。
- `export_schema`、`export-language-sdk-catalog.sh`、`SdkScope`与四套catalog gates — 唯一SDK链。

## Todos

### implement-execution-precondition

requirements:
- 用户要求同一Task authority只有一个实现并按Finding修复。
- ADR 0008与条件写业界模式要求stale relation不得覆盖runtime事实。

interfaces:
- consumes: loaded graph、TaskGraphCommit、TaskExecution与RevisionedTaskStore。
- produces: optional expected execution map与canonical exact CAS。

steps:

1. 保留red测试并实现precondition捕获、序列化与Store compare。
   verify: red日志是行为失败；canonical patch Some、create None、runtime drift Conflict。
   expected: claim/retry/settlement在relation revision不变时仍被保护。

2. 运行claim/SetStatus/spec Update/settlement/retry与正常revisioned测试。
   verify: 确定性交错Conflict且原claim可settle，manual progress/idempotent running通过。
   expected: 状态机与relation revision不变。

### synchronize-public-sdk-contract

requirements:
- 公共API变更必须同步contracts、SDK与正式文档。
- ADR 0032要求scope与status独立。

interfaces:
- consumes: 最终TaskGraphCommit public shape与Plan05生成器。
- produces: public-api/parity/catalog/source/shared artifacts和四套count gates。

steps:

1. 运行唯一generator和shared export。
   verify: 新字段是external value:task identity；canonical/external各+1，其他scope不变，extension protocol为1。
   expected: 第二次check零diff。

2. 更新四套scope snapshot并执行完整门禁。
   verify: `cargo fmt --all -- --check`、`cargo clippy -p echo_orchestration --all-targets --locked -- -D warnings`、`cargo test -p echo_orchestration tasks::revisioned::tests --locked`、`cargo check -p echo_agent --no-default-features --features subagent --locked`、`./scripts/check-sdk-contracts.sh`、`./scripts/check-language-sdks.sh`全部exit 0。
   expected: Task、facade、Host与三语言在同一revision闭合。

### close-semantic-finding

requirements:
- Echo Semantic要求首个diff后semantic-diff，完成前semantic-verify。
- Resolved Finding必须有repair、verification、rereview证据。

interfaces:
- consumes: red/green日志、diff、ADR、generated artifacts与独立review。
- produces: resolved Finding、Evidence、rereview Audit与MASTER-PLAN状态。

steps:

1. 刷新Task/SDK对象并写repair/verification evidence。
   verify: before绑定6d55fae9，after绑定current source digest；其它Finding不变。
   expected: 证据可追到故障、CAS、SDK drift、命令和回滚。

2. 执行review、semantic-verify与scope检查。
   verify: reviewer无blocker；strict/change evidence/diff check通过；CLI/website零差异。
   expected: repair可独立提交和停止。

## Decisions

- Plan 07接续superseded Plan 06，绑定含SDK同步的替代outcome。
- 使用完整typed TaskExecution map，不使用字符串hash。
- Public字段属于既有value:task external contract并同步生成资产。
- runtime mutation不递增relation revision；None不用于canonical patch。
- 本次只关闭task-patch-claim-race。