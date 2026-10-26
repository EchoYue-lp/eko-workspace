---
schema_version: 4
slug: eko-gui-product-convergence/hook-ask-semantics
outcome:
  summary: 框架保留 hook Ask/RequireApproval 的交互要求，并在允许规则、缓存、拒绝、无 provider、超时和取消场景下给出一致结果。
  acceptance:
    - PreToolUse 与 PermissionRequest 的明确 Ask/RequireApproval
      在同一工具调用中保留强制交互意图及建议；普通 allow、审批缓存和 full-auto 均不能签发未经过本次真实交互的允许结果，任何已适用的
      Deny 先于批准结算。
    - 复用唯一 PermissionService、既有 handler/provider 和调用级
      context/receipt：批准前不执行工具，拒绝、无人可答及等待失败不执行工具；普通未请求强制交互的调用保留现有权限模式、规则与缓存行为。
    - 新增通用 Rust public 方法落实方案 A，不给 PermissionInvocationContext 加字段、不新增公共状态实体或
      EKO 专属策略；身份、建议、有效输入及批准范围经真实主路径消费，改参后仍复检受保护路径并绑定最终输入 receipt。
    - 相关反例与正常控制、既有取消结算、公开
      API/feature/示例及适用消费者检查有新鲜证据并通过独立复审；ADR、双语正式说明和网站同步接缝完整，SDK accepted 合同不因
      Rust API 新增自动扩入。实现/本地验收与后续精确 main/CI 交付分别记录。
out_of_scope:
  - 不新增 HITL 队列、permission store、运行审批状态机或协议，不将 EKO 菜单、弹窗或连接管理策略下沉框架；不修改 GUI
    呈现或插件主动输入/选择合同。
  - 不统一 Other 到 Cancelled 的错误类型、不新增审批默认 deadline、不重写 provider 生命周期或 GUI pending
    队列清理；只证明和保持现有取消、拒绝与 Timeout/Deferred 结算。
  - 不重做已 resolved 的 hook deny 跨来源聚合修复，不扩大为普通非 hook 权限模式/缓存政策重构，也不顺带退役 A2A/LSP
    或独立终端入口。
  - 不自动更改 SDK pin、accepted JSON、语言 API、路由或生成物，不将完整 Rust public inventory
    当作新增三语言接受合同；真实合同差异交回 SDK owner 核对。
  - 不以候选或本地通过声明跨仓主线完成，不执行未获授权的 push、merge、PR、发布、全项目资源清理或上游 lifecycle 变更。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 1. 文档定位与已确认目标
    digest: sha256:0f17d2cbbec5131fac3d3c4b955710ddcf09a84c2872177492af6425a4a7ac4f
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § HITL 弹窗
    digest: sha256:6d1ace366e78550f0a0eda37546aab3abd4c7597fa872e4a6ddd1ae9e7febb49
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#hook-ask-semantics
todos:
  - id: route-explicit-hook-approval-through-shared-service
    summary: 沿现有 pipeline/snapshot 把 hook 的强制交互要求接入 PermissionService 显式入口，复用同一个
      handler 与批准后处理。
    files:
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/echo-orchestration/src/human_loop/service.rs
    acceptance:
      - 实现前在 framework 全仓及直接 EKO/SDK 消费者复核重复性和可达性，确认 HookRegistry deny
        归约、PermissionRequest/HumanLoopRequest、PermissionInvocationContext、SessionApprovalCache
        和 ToolApprovalReceipt 继续拥有原语义；新增的是通用显式交互入口，不复制另一套 handler/缓存/receipt 处理。
      - PermissionStage 不再丢弃 Ask/RequireApproval：PreToolUse 的交互要求经
        PermissionRequest hook 后仍存在，后者 Allow 不能清空前者的 Ask，任一 hook Deny
        先拒绝；建议原文和有效输入随调用继续传递，不能把任意建议字符串猜成会话或永久批准动作。
      - snapshot 沿已有 request/session/agent/scope context 和 cancellation select
        传达强制交互要求；service 在适用拒绝检查之后进入真实 handler，绕过本次调用的自动 Allow/cache/full-auto
        放行。已有公开 check 签名及 PermissionInvocationContext 字段保持不变；强制调用无
        service、human-loop 关闭或 NullHandler 无真实 provider 时明确失败，普通调用保持原行为。
      - 批准后的范围缓存写入、规则更新、用户改参后的受保护路径复检和最终有效输入 receipt 仍由 service 既有后处理执行；强制 Ask
        再次出现时仍须新交互，未要求强制交互的后续普通调用继续按既有 scope 缓存规则运行。
  - id: prove-approval-and-wait-settlement-boundaries
    summary: 用现有 inline 测试与 provider/handler 夹具证明强制交互反例、拒绝优先及现有等待结算，没有真实批准时无工具副作用。
    files:
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/echo-orchestration/src/human_loop/service.rs
      - echo-agent/echo-orchestration/src/human_loop/permission.rs
    acceptance:
      - 分别覆盖两类 hook 和 Ask/RequireApproval；针对 allow rule、同 scope 审批缓存、full-auto
        以及跨事件 Ask 后 Allow 构造当前版本的失败证据，再证明 provider
        被调用且批准前工具计数为零。默认只读自动允许作为正常对照，不能只检查 trace 中出现 ask。
      - 覆盖 hook Deny 与其他决定竞争、受保护路径拒绝、无 service、禁用 human-loop、NullHandler
        等情况；Deny/无人可答均不执行工具，不通过伪造 Allow receipt、默认 provider 或新 store 冒充批准。
      - 检查 request/session/agent/scope、context 和建议可被真实 handler/provider
        观察；Approved/带 scope 批准及 ModifiedArgs 正常执行且 receipt
        匹配最终参数；改参命中受保护路径仍拒绝；Denied、Timeout、Deferred、NeedMoreInfo 未批准结果不执行工具。
      - 复用
        permission_wait_stops_when_invocation_is_cancelled、PendingApprovalProvider、CountingAllowHandler
        等现有入口验证实际取消等待，先可观察地确认provider已进入待答状态再取消，不得用future首次poll前cancel代替；只断言现有取消终止与无执行，不偷偷统一外层
        Other/Cancelled 类型或引入默认 timeout。保留既有 deny 聚合、缓存 scope、provider 替换和 shell
        receipt 回归，不把老 trace 测试改名或改期待后当作新增交互验收。
  - id: document-and-qualify-framework-approval-contract
    summary: 记录最小 public 方法及权威边界，完成框架正式说明、可执行示例和受支持消费者的适用验收。
    files:
      - echo-agent/docs/adr/
      - echo-agent/docs/en/05-human-loop.md
      - echo-agent/docs/zh/05-human-loop.md
      - echo-agent/docs/en/23-hooks.md
      - echo-agent/docs/zh/23-hooks.md
      - echo-agent/echo-agent-learning/examples/demo03_approval.rs
    acceptance:
      - 新增或按所属约定更新 ADR，先查编号；记录方案 A 的跨 crate public
        方法理由、候选比较、官方来源和查询日期、影响与恢复范围。通用机制由 framework 独立拥有，不以没有第二消费者否定能力，也不引入 EKO
        策略；不改写历史 resolved Finding 或旧交付事实。
      - 双语 HumanLoop/Hook 文档说明明确 Ask 的强制语义、Deny 优先、无人可答失败、普通调用不变及取消/timeout
        的实际范围；复用 demo03_approval 展示真实调用而非 API 空壳，示例继续进入既有可执行验证链；官网正式源同步交由协调者，不在
        framework writer 中改网站。
      - 对照 SDK 已接受的 PermissionService/context/check 方法、Host InvocationContextDto
        和路由，证明旧公开签名/值与原 pin 消费不受破坏；新增 Rust 方法只作为 framework API/public inventory
        变化，不自动进入 SDK accepted 合同或语言包装。确有消费者合同差异时先交回 owner，不静默刷新生成物。
      - Build 按协调者当前资源租约按实际改动完成相关 focused、human-loop 启用/关闭、跨 crate public API
        条件矩阵、examples 与最终仓库门禁，独立复审后再做最终直接验证；每批 Cargo 验证保存 target
        外证据/必要制品并确认无活动读取后及时 cargo clean 自有准确 target，遵守两条
        focused/一套完整矩阵/jobs<=2/debug0/低于5GiB停新重构建。新增 API 先交付
        framework，消费者其后采用；本地、精确 main/CI 与原生验收分别报告。
artifact_id: plan:2e0b8f47-7ef8-4117-89bb-f1a2bc36cca8
lifecycle: ready
---
## Notes

执行基线为 framework `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`，生产源码与 `51726454985fd6f064b21a7e9c539e22d494c987` 相同。前置产品边界合同已经本机独立审查并交付协调者，精确 main/CI 尚待合流，不把本地贡献当作主线完成。

官方资料查询日期为 2026-10-24：[Claude Code PreToolUse](https://code.claude.com/docs/en/hooks#pretooluse-decision-control)、[PermissionRequest](https://code.claude.com/docs/en/hooks#permissionrequest-decision-control)、[Agent SDK 权限顺序](https://code.claude.com/docs/en/agent-sdk/permissions#how-permissions-are-evaluated)。它们用于解释显式交互与自动放行的分层，不替换本项目已批准合同，也不用未公开的缓存内部实现作依据。

方案 A 使用显式 service 方法共享既有 handler/批准后处理，不修改已被 SDK 接受的 PermissionInvocationContext 结构。GUI 只呈现既有请求，框架不接收 EKO 产品策略；正式实现先查重复和 ADR 编号。
