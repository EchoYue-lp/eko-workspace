---
schema_version: 3
lifecycle: completed
supersedes: null
slug: echo-agent全workspace语义治理/修复SubagentFactory取消与发布竞态
goal: 让SubagentRegistry lazy factory在取消、并发等待和重新注册下只发布当前registration revision的单一实例
ships: Subagent lazy factory 以 registration revision scoped single-flight
  原子完成同代构造与发布，取消后可恢复、旧代结果不发布，并关闭 cancellation 与 publication race 两个 Finding
verify: 旧实现的取消恢复与publication gap确定性red、新实现的single-flight/retry/stale revision
  green、既有registry/executor回归、public SDK
  inventory零变化、ADR/两个Finding/独立review闭合；cargo fmt、focused Clippy/test、subagent
  feature check和semantic strict/change-evidence全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-subagent-factory-singleflight
todos:
  - id: replace-factory-singleflight
    files:
      - echo-agent/src/agent/subagent/registry.rs
      - echo-agent/docs/adr/0033-subagent-factory-singleflight-publication.md
    summary: 用revision-scoped OnceCell统一factory构造与发布并覆盖取消和并发交错
    verify: cancel/error后可重试，同revision无双创建，旧revision结果不发布，既有registration与executor行为不回归
  - id: close-factory-findings
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入ADR关联证据与独立复审并只关闭两个Subagent factory Finding
    verify: 两个Finding resolved具备repair/verification/rereview
      refs；其它Finding保持原状态；semantic gates通过
artifact_id: plan:5fc92431-d8a9-401d-a90f-604fdf3e3136
design_revision: null
---
## Context

- Plan 04在Task/Subagent状态权威审计中确认两个High Finding：factory await被取消会永久残留全局instantiating标记；成功路径先删标记再发布实例，形成同revision双创建窗口。
- Plan 07已在`78b9f06b`闭合Task patch CAS；本切片继续处理同一边界中下一个无需产品裁决的framework lifecycle repair。
- Tokio 1.53.1官方`OnceCell::get_or_try_init`明确保证并发初始化合并，错误、取消或panic不发布值并允许等待者重试，适合作为既有依赖内的single-flight原语。

## Approach

- 每个`RegistryEntry`持有registration revision scoped `Arc<tokio::sync::OnceCell<Arc<dyn Agent>>>`；prebuilt注册使用已初始化cell，definition/factory注册使用空cell。
- `get_agent`先捕获当前entry的revision、factory与cell，再用`get_or_try_init`执行同代唯一构造。成功值先原子写入cell，之后复核registry中revision与cell identity；重注册后的旧代结果丢弃且不得成为当前实例。
- 删除registry-wide `HashSet + Notify + 50ms polling`协调链。初始化future被取消时，Tokio permit随future drop释放；同代后续resolve可重新尝试。factory error不缓存，等待者按OnceCell合同取得下一次串行尝试。
- 移除registry内部硬编码30秒waiter timeout；通用framework不替调用方决定factory构造deadline，Runtime/caller继续通过自身cancel/timeout控制等待。该行为在ADR 0033明确记录。
- `create_fresh_agent`继续表示显式fresh构造，不参与cached single-flight。
- 先在旧实现上记录两个确定性red：取消后第二次resolve可恢复；持有state read lock扩大publication gap时并发resolve不得触发第二次factory。再实施并补failure retry、重注册旧代隔离与既有回归。

## Global Constraints

- 只使用Subagent术语，不新增Worker命名。
- `SubagentRegistry`继续是唯一definition/factory/instance publication authority；OnceCell是entry内部协调原语，不是第二registry或状态机。
- 不新增依赖、public类型/字段、wire/schema或SDK identity；若generator显示公共合同差异，停止并修订Plan，不静默扩大SDK范围。
- 不使用unwrap/expect/panic/直接不安全索引；确定性测试不得依赖长sleep或概率调度。
- 只关闭`finding.subagent-factory-cancellation`与`finding.subagent-factory-publication-race`；TaskClaim/Attempt、definition catalog及其它Finding保持open。
- `echo-agent-cli`、`echo-website`和examples不修改；本次调用形状不变，交付说明记录不适用原因。

## Files

- Modify: `echo-agent/src/agent/subagent/registry.rs` — revision-scoped OnceCell single-flight、旧协调链删除与确定性交错测试。
- Create: `echo-agent/docs/adr/0033-subagent-factory-singleflight-publication.md` — 记录状态权威、取消、错误、重注册、deadline和替代方案。
- Modify: `echo-agent/.echo-semantic` — 当前snapshot、repair/verification/rereview Evidence/Audit及两个Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一Frontier。

## Reuse

- `src/agent/subagent/registry.rs` — `RegistryEntry.revision`、`SubagentRegistry::get_agent`、`register_*`与`publish_catalog`继续作为唯一authority。
- `tokio::sync::OnceCell::get_or_try_init` — 既有Tokio 1.53.1提供cancel-safe async single initialization，不自建guard或waiter状态机。
- `audit.task-subagent-workflow.state-authority`、两个factory Finding与`evidence.task-subagent-workflow` — 故障基线和关闭对象。
- `docs/adr/0024-unified-subagent-prompt-compilation.md`、`docs/adr/0030-versioned-subagent-event-envelope.md` — 保持Subagent能力发布与事件边界不变。

## Todos

### replace-factory-singleflight

requirements:
- 用户已确认按Finding驱动统一全workspace语义，并要求同一状态只有一个authority。
- 两个High Finding共享registry-wide instantiating机制，必须原子替换，不能保留临时双协调链。

interfaces:
- consumes: `RegistryEntry.revision`、`AgentFactory::create`、`SubagentRegistry::get_agent`与Tokio OnceCell。
- produces: revision-scoped cancellation-safe cached agent publication。

steps:

1. 在旧实现上加入取消恢复和publication gap确定性交错。
   verify: 第一次factory被abort后第二次resolve在短界限内重新进入factory；state read lock阻塞publication时第二resolve不得产生第二次create。
   expected: 旧实现两条测试分别暴露永久标记和双创建窗口，形成非零red证据。

2. 以per-entry OnceCell替换HashSet/Notify/polling，并保留registration revision复核。
   verify: cancel释放初始化权，成功值在同一cell原子发布；re-register替换cell，旧代结果不进入当前entry。
   expected: 同revision最多一次成功构造，取消或失败后可重试，旧revision无法覆盖新registration。

3. 覆盖failure retry、prebuilt/definition/factory同步与异步注册、并发waiter、remove/re-register及existing executor consumer。
   verify: registry focused tests与相关executor测试通过，public signature和SDK inventory无差异。
   expected: 不新增第二生命周期owner，create_fresh_agent语义不变。

### close-factory-findings

requirements:
- Echo Semantic resolved Finding必须有repair、verification和独立rereview证据。
- 架构级状态权威变化必须进入正式ADR。

interfaces:
- consumes: red/green日志、最终diff、ADR 0033与独立review。
- produces: 两个resolved Finding、current Task/Subagent map与可重算semantic evidence。

steps:

1. 写ADR 0033并刷新Task/Subagent语义对象。
   verify: ADR为Accepted，记录Tokio官方依据、选项、deadline、失败重试、revision隔离、兼容影响和回滚。
   expected: 状态权威与生命周期承诺可从代码、ADR、Rule和Evidence互相追踪。

2. 执行独立review并只关闭两个factory Finding。
   verify: reviewer无blocker；Finding具备repair/verification/rereview refs；其余85个Finding中的非目标项保持原状态。
   expected: semantic strict snapshot与change evidence通过，修复可独立提交和停止。

## Decisions

- cancellation与publication race合并为一个Plan，因为二者由同一全局instantiating authority造成，最小正确修复是一次性替换该机制。
- 选择Tokio OnceCell而非自建RAII async cleanup或per-name Mutex；它是已安装依赖的官方cancel-safe原语，并把构造与成功发布合成一个操作。
- 移除registry内部固定30秒waiter timeout；deadline属于调用方，避免同一factory因内部轮询与外部runtime timeout形成两套策略。
- factory error不缓存；后续等待者可按OnceCell合同发起下一次串行初始化。