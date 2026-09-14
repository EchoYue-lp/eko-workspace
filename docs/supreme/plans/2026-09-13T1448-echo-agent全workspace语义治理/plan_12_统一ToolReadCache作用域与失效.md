---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/统一ToolReadCache作用域与失效
goal: 让ToolManager read cache只在同一effective workspace与invocation
  lineage复用，并阻止in-flight Read跨Write复活旧结果
ships: ToolManager read cache 以 effective workspace 与 invocation lineage
  定义作用域，并用 write-lifetime epoch CAS 阻止 in-flight Read 在状态变更后重新发布旧结果
verify: 跨workspace/run与Read-Write交错的旧实现red、新scoped key/epoch/guard
  green闭合；ToolManager cache/stream tests、echo_execution Clippy/check、SDK
  artifact零diff、ADR/两个Finding/独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-tool-read-cache-authority
todos:
  - id: unify-cache-scope-and-epoch
    files:
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/docs/adr/0034-context-scoped-tool-result-cache.md
    summary: 以context-scoped key和write-lifetime epoch CAS统一read cache freshness
    verify: 跨scope不命中，同scope保留TTL复用，任何Write生命周期后in-flight旧Read不能store
  - id: close-cache-findings
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入ADR、repair、verification和独立复审并关闭两个Tool cache Finding
    verify: 两个Finding resolved具备三类关闭refs；其它Finding不变；semantic gates通过
artifact_id: plan:50c9d8d3-6f77-4f3b-a924-7ba090aa61b7
lifecycle: completed
design_revision: null
---
## Context

- 两个High Finding共同描述同一ToolManager read cache authority：key缺workspace/invocation identity；write clear后旧in-flight Read可无条件写回。
- Plan 11已在`50890faa`统一stream/non-stream validation，cache逻辑位于其后且可独立修复。
- RFC 9111把cache key定义为选择可复用表示所需的上下文，并要求状态变更失效已存响应；本地Tool cache采用同一原则但不声称HTTP语义。

## Approach

- 先添加两个确定性交错red：相同tool/params在不同working_dir与run lineage不得互相命中；Read捕获旧值后，Write完成再释放Read，后续Read不得命中被复活旧值。
- 用私有`ToolResultCacheKey`组合tool name、canonical parameter JSON、effective working directory及conversation/run/turn/message/execution identity。`working_dir=None`时读取当前进程cwd；无法取得effective cwd时禁用该次cache。
- 增加`result_cache_epoch: AtomicU64`。Read在lookup前捕获epoch，成功后只有在持cache write lock且epoch未变时才能store。
- 非Read调用创建write-lifetime invalidation guard：构造时在cache lock内bump epoch并clear，Drop在所有成功、失败、取消、timeout与caller-drop路径再次bump/clear。这样Write期间完成的Read无论先后都不会在Write之后留下旧值。
- 同一scope、同一epoch的Read继续使用现有TTL与capacity；stream/non-stream共用相同key/epoch逻辑。
- 不把call_id放入key，以保留同一execution内重复逻辑调用的复用；message/execution identity隔离跨消息与Subagent attempt。

## Global Constraints

- ToolManager继续是唯一result cache owner；不得新增per-tool/global第二cache或把cache状态放入ToolContext。
- Cache key必须包含effective cwd与稳定invocation lineage；任何无法确定cwd的调用宁可不缓存。
- 所有非Read调用即使失败、取消或部分执行也在开始和结束失效cache；不等待success文本。
- Epoch compare与cache insert必须在同一cache write临界区，避免check/store再竞态。
- 不使用unwrap/expect/panic/直接索引，不新增依赖、public API、wire/schema或SDK identity。
- 只关闭`finding.tool-read-cache-scope`与`finding.tool-read-cache-inflight-invalidation-race`；permission/sandbox及其它78个非目标open Finding保持状态。
- 正式ADR 0034记录scope、epoch、失败/cancel、兼容与回滚；docs/examples/CLI/website不改，公开ToolManager调用形状不变。

## Files

- Modify: `echo-agent/echo-execution/src/tools.rs` — scoped key、epoch guard、CAS store与确定性交错测试。
- Create: `echo-agent/docs/adr/0034-context-scoped-tool-result-cache.md` — cache authority与业界依据。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit及两个Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一Frontier。

## Reuse

- `ToolContext`既有working_dir和conversation/run/turn/message/execution identity — 直接组成scope，不新增context字段。
- `result_cache`、TTL、capacity、`cached_result`与`store_cached_result` — 原地扩展，不建第二cache。
- parking_lot RwLock与AtomicU64 — 现有依赖和definitions cache版本模式。
- `ReadCountingTool`与ToolManager并发测试模式 — 构造workspace隔离和Read/Write交错。
- RFC 9111 sections 2/4.4、两个Finding与Tool semantic对象 — 设计依据与关闭目标。

## Todos

### unify-cache-scope-and-epoch

requirements:
- 用户要求状态和数据有唯一权威，同一语义不得平行实现。
- workspace scope与in-flight invalidation缺一不可，必须同一次替换现有key/store逻辑。

interfaces:
- consumes: `ToolContext`稳定identity、ToolRiskLevel、现有result cache/TTL/capacity。
- produces: scoped key、monotonic epoch、write-lifetime invalidation guard与conditional store。

steps:

1. 添加workspace/run隔离与Read-before-Write-after交错red。
   verify: 旧实现跨scope只调用一次并返回错误workspace；旧Read在Write后store导致第三次Read命中旧值。
   expected: 两类Finding都有独立非零行为证据。

2. 实现context key、epoch CAS与write guard，接入stream/non-stream。
   verify: 相同scope重复Read命中；不同scope隔离；Write前/中/后Read均不能在结束后留下旧值；失败/cancel Drop同样bump。
   expected: 一个cache owner、一个epoch，无check/store窗口。

3. 运行ToolManager cache/stream focused tests、echo_execution Clippy/check与SDK artifact零diff。
   verify: 全部exit 0，TTL/capacity/retry/validation不回归。
   expected: public shape与生成合同不变。

### close-cache-findings

requirements:
- 架构级状态权威变化必须有Accepted ADR；resolved Finding必须有repair、verification与独立rereview证据。

interfaces:
- consumes: red/green日志、ADR 0034、最终diff与独立review。
- produces: 两个resolved cache Finding与current Tool semantic evidence。

steps:

1. 写ADR并刷新Tool Map/Behavior/Rule/Asset及repair/verification Evidence。
   verify: before绑定`50890faa`，after绑定current source digest；permission/sandbox Finding不变。
   expected: key、epoch、guard、兼容、限制与回滚可追踪。

2. 独立review后写rereview Audit并关闭两个目标Finding。
   verify: reviewer无blocker；两个Finding具备三类关闭refs；semantic strict/change-evidence通过。
   expected: repair可独立提交和停止。

## Decisions

- working_dir加conversation/run/turn/message/execution组成cache scope；call_id不参与，以保留同一execution的重复Read复用。
- Write在开始和Drop各失效一次，覆盖排队、执行、错误、取消与部分side effect；不只在success后clear。
- Epoch与insert在同一lock内比较，避免旧Read跨Write复活。
- 不与permission、sandbox或stream validation合并；它们不拥有result cache freshness。