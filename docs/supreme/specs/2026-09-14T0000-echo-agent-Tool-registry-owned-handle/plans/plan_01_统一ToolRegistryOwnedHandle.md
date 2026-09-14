---
schema_version: 3
supersedes: null
slug: echo-agent-Tool-registry-owned-handle/统一ToolRegistryOwnedHandle
goal: 让ToolManager动态registry以owned Arc handle执行Tool，彻底消除DashMap guard跨await的阻塞生命周期
ships: ToolManager执行路径不再持有registry
  guard跨await，动态replace/unregister可在current-thread runtime完成，并保留Tool
  generation与result cache freshness fence
verify: current-thread确定性交错证明active
  Tool期间replace/unregister均不阻塞；ToolManager全量定向测试、动态Tool example、Rust public
  inventory与SDK合同生成检查、适用public API feature矩阵、ADR/Finding/Issue
  reconciliation、Clippy/check及semantic gates全部通过
design_ref: docs/supreme/specs/2026-09-14T0000-echo-agent-Tool-registry-owned-handle/design.md
delivery_ref: null
todos:
  - id: replace-borrowed-registry-ref-with-owned-handle
    files:
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/src/agent/react/run/phases/tools.rs
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent/src/agent/snapshot.rs
    summary: 以单一Arc Tool generation替换跨await DashMap Ref并保持cache epoch fence
    verify: lookup在同步临界区clone Arc并释放guard；current-thread active
      Read期间replace与unregister立即返回；旧generation不能回填新cache
  - id: synchronize-rust-contract-and-example
    files:
      - echo-agent/src/agent/react/capabilities.rs
      - echo-agent/echo-agent-learning/examples/demo35_dynamic_tools.rs
      - echo-agent/contracts/sdk
      - echo-agent/sdks/shared
      - echo-agent/docs/adr/0035-owned-tool-registry-handles.md
    summary: 同步owned Tool返回合同、动态工具示例、ADR与Host/Rust-only inventory
    verify: Box注册入口与Agent bool
      remove保持；Arc返回迁移清楚；canonical/scope计数不变，只有预期signature与digest生成差异
  - id: close-tool-registry-mutation-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: "写入repair、verification和独立复审并关闭Issue #115对应Finding"
    verify: Finding resolved具备三类关闭refs；92个Finding与Issue保持一对一；其它79个open
      Finding状态不变；semantic gates通过
artifact_id: plan:43800b01-2c43-4990-bb60-2c46500ee87a
lifecycle: completed
design_revision: sha256:56e24ab3177c4f863f5c129720de52797810c3a387ed3c063fc0cb24f7d59577
---
## Approach

- 将ToolManager唯一registry value从Box转换为Arc；注册继续接收Box并只在边界执行一次Arc转换。
- get_tool在DashMap同步临界区克隆Arc后返回owned handle；所有异步执行和framework消费者只持有Arc，不暴露或携带DashMap Ref。
- replace/unregister返回旧Arc generation；已领取旧generation的执行允许完成，catalog mutation不承担强制取消语义。
- 保留Plan 12的epoch-before-get与mutation invalidation。Concurrent lookup可取得完整old或new generation，但old generation在epoch变化后不能store result cache。
- 通过旧实现会挂起的current-thread active-call交错建立red，再实现Arc storage；持久测试同时覆盖replace、unregister和cache fence。
- Rust public identity仍属host_or_rust_only：运行唯一generator刷新signature/digest，不新增三语言facade；同步正式ADR与demo35。
- 独立review通过后关闭finding.tool-registry-mutation-active-call-deadlock；Issue #115保持open直到远端main交付。

## Global Constraints

- ToolManager是唯一Tool catalog与result cache owner；不得增加shadow registry、包装trait或application-side generation authority。
- Registry guard不得跨任何await；Arc只表达generation lifetime，不把Drop描述为async cleanup settlement。
- Register/try_register/ToolRegistrar继续接收Box<dyn Tool>；ToolManager get/replace/unregister与ReactAgent固有返回改为Arc<dyn Tool>；Agent::remove_tool -> bool保持不变。
- Replacement/unregister不取消已领取old generation；旧调用是否取消继续由ToolContext与caller lifecycle决定。
- Plan 12的context-scoped cache key、write-lifetime epoch、TTL/capacity和stream/non-stream parity不得回退。
- Public inventory与generated contract只由export_schema --update产生，不手改manifest或digest；canonical 9683与五类scope计数保持不变。
- 不新增依赖、wire/schema或TypeScript/Python/Java facade；不修改echo-agent-cli与echo-website。
- 不使用unwrap/expect/panic/unreachable/todo、直接不安全索引或Worker术语。
- Issue #115已先于修复建立；仅关闭该Finding，其它open Finding不顺带处理。

## Files

- Modify: `echo-agent/echo-execution/src/tools.rs` — Arc registry value、owned lookup、mutation返回、cache fence与交错测试。
- Modify: `echo-agent/src/agent/react/run/phases/tools.rs` — 消除DashMap Ref value访问。
- Modify: `echo-agent/src/agent/react/run/pipeline.rs` — 消除DashMap Ref value访问。
- Modify: `echo-agent/src/agent/snapshot.rs` — 消除DashMap Ref value访问。
- Modify: `echo-agent/src/agent/react/capabilities.rs` — ReactAgent动态工具返回Arc。
- Modify: `echo-agent/echo-agent-learning/examples/demo35_dynamic_tools.rs` — 展示owned old-generation handle合同。
- Modify: `echo-agent/contracts/sdk` — 由唯一generator刷新Rust public signature与source contract。
- Modify: `echo-agent/sdks/shared` — 由唯一generator刷新共享digest。
- Create: `echo-agent/docs/adr/0035-owned-tool-registry-handles.md` — 记录并发、public API、兼容与回滚。
- Modify: `echo-agent/.echo-semantic` — 当前snapshot、repair/verification/rereview及Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新repair进度、计数与frontier。

## Reuse

- `ToolManager::tools`、`registration_lock`、`definitions_version`与`result_cache_epoch` — 原地替换value ownership，不建第二authority。
- 标准库`Arc<dyn Tool>`与Tool的Send+Sync合同 — 提供跨await owned lifetime，无新依赖。
- `result_cache_key`、`store_cached_result(expected_epoch)`与registry invalidation — 保留Plan 12 freshness fence。
- `demo35_dynamic_tools`、ToolManager execute tests与public facade inventory — 既有动态Tool和合同验收入口。
- Tokio shared-state指导、DashMap locking contract与OpenAI Codex Arc runtime registry — ADR的外部设计依据。
- `finding.tool-registry-mutation-active-call-deadlock`与Issue #115 — 唯一关闭目标。

## Todos

### replace-borrowed-registry-ref-with-owned-handle

requirements:
- § 目标行为
- § 系统边界
- § 异常与边界场景
- § 复用与实现约束

interfaces:
- consumes: ToolManager唯一DashMap registry、Box<dyn Tool>注册输入、definitions version与result cache epoch。
- produces: Arc<dyn Tool> registry generation、owned lookup和不跨await的执行生命周期。

steps:

1. 在旧实现上加入active Read挂起时从current-thread同步replace/unregister的交错red，并保留Plan 12 old-generation cache断言。
   verify: 旧实现的受控命令在deadline内不能完成mutation，证据明确落在DashMap Ref lifetime而非Tool逻辑失败。
   expected: Issue #115有确定性失败证据，测试本身不以永久挂起污染常规suite。

2. 将registry value和lookup/mutation返回迁移为Arc，逐个消除async调用链中的DashMap Ref。
   verify: lookup临界区结束后只保留Arc；replace/unregister在old Tool future释放前返回；old result因epoch变化不store。
   expected: 一个catalog、一个generation publication、一个cache epoch，current-thread不再阻塞。

3. 运行ToolManager registration/cache/stream/retry/cancel/timeout定向回归。
   verify: active-call replace/unregister、completed replacement、Read/Write overlap与其它ToolManager tests全部通过。
   expected: Plan 12 freshness与既有Tool执行合同无回归。

### synchronize-rust-contract-and-example

requirements:
- § 公共合同
- § 关键取舍与业界依据
- § 验收标准

interfaces:
- consumes: Arc registry public shape与现有export_schema生成链。
- produces: 更新后的Rust/Host-only signature inventory、shared digest、ADR 0035与可执行dynamic Tool示例。

steps:

1. 更新framework调用点、ReactAgent固有返回与demo35，写Accepted ADR 0035。
   verify: 所有get_tool consumer直接使用Arc；Agent::remove_tool bool与Box注册入口保持；example编译并观察old handle is_some。
   expected: Rust迁移影响与old-generation lifetime清楚，CLI/website无需同步。

2. 通过唯一generator刷新contracts/sdk和sdks/shared并执行第二次check。
   verify: get_tool/replace/unregister及ReactAgent对应signature按Arc更新；canonical 9683和scope计数不变；wire/schema与三语言facade无新增。
   expected: 生成结果byte-stable，手工差异为零。

3. 执行公共API条件矩阵与SDK合同门禁。
   verify: 仓库规定的逐feature check、SDK contract/language checks、demo35编译及相关crate Clippy/check全部exit 0。
   expected: default/no-default/独立feature和Host/Rust inventory均接受新合同。

### close-tool-registry-mutation-finding

requirements:
- § 验收标准
- 用户要求语义修复严格执行一Finding一Issue。
- Echo Semantic resolved Finding关闭合同。

interfaces:
- consumes: red/green日志、ADR 0035、generated inventory、最终diff、Issue #115与独立review。
- produces: resolved Finding、repair/verification Evidence、rereview Audit和更新后的MASTER-PLAN。

steps:

1. 刷新Tool Map/Behavior/Rule/Asset与source snapshot，写repair/verification Evidence。
   verify: before绑定745a3f87，after绑定current source digest；Issue URL保持唯一；其它Finding状态不变。
   expected: authority、并发顺序、public contract、验证与回滚可从Finding追踪。

2. 独立review后写rereview Audit并执行最终语义与Issue门禁。
   verify: reviewer无blocker；strict snapshot、change evidence、Issue reconciliation和git diff check通过。
   expected: Finding在本地标记resolved，Issue #115仍open等待远端main交付。

## Decisions

- 使用标准Arc owned handle；不引入Box代理、ArcSwap、actor registry或第二catalog。
- Dynamic mutation线性化点仍是registration lock内的DashMap insert/remove；Arc只解耦old generation lifetime。
- Public Rust返回类型允许从Box/Ref迁移到Arc，因为原合同会在合法current-thread调用中deadlock，且identity已分类host_or_rust_only。
- Tool取消与Tool析构不是registry mutation的职责；remove只撤销后续可发现性。
- 该Plan只关闭Issue #115，不重新打开Plan 12的cache Finding。