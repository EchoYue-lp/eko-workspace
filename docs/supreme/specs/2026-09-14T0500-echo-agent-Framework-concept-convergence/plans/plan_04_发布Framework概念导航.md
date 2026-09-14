---
schema_version: 3
supersedes: null
slug: echo-agent-Framework-concept-convergence/发布Framework概念导航
goal: 把全 workspace 已确认的架构、核心概念、状态权威和生命周期收敛为正式中英文导航层
ships: 发布中英文 Architecture、Core Concepts 与 Lifecycles 基础文档，以唯一状态权威串联现有领域章节和可执行 examples
verify: 六份基础文档与双语导航结构对等、链接可解析、概念/生命周期/边界完整且不扩大 open Finding 承诺；documentation
  contract、formatter、相关 Clippy/check、facade/example contracts、SDK 零漂移、semantic
  strict/change-evidence、92/92 Issue 对账和独立复审全部通过
design_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/design.md
delivery_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/plans/delivery-map.md#publish-framework-concept-navigation
todos:
  - id: establish-foundational-doc-contract
    files:
      - echo-agent/echo-agent-learning/tests/documentation_contract.rs
    summary: 扩展现有文档合同，覆盖六份基础文档、双语结构、root/docs 导航和本地链接
    verify: 合同在基础文档缺失时先 red，发布后对文件/导航缺失、结构不对等、断链和必需事实路由 fail closed
  - id: publish-architecture-authority
    files:
      - echo-agent/docs/en/architecture.md
      - echo-agent/docs/zh/architecture.md
      - echo-agent/docs/adr/0040-framework-concept-documentation-authority.md
    summary: 发布 11-package 分层架构、公共组合和文档权威 ADR
    verify: Architecture 准确区分 root facade、七个 split framework crates、SDK
      protocol/Host、learning consumer 与 embedding application，ADR 记录方案、取舍和影响
  - id: publish-concepts-and-lifecycles
    files:
      - echo-agent/docs/en/concepts.md
      - echo-agent/docs/zh/concepts.md
      - echo-agent/docs/en/lifecycles.md
      - echo-agent/docs/zh/lifecycles.md
    summary: 发布限定核心术语、唯一权威矩阵和七条生命周期主线
    verify: Concepts 覆盖 owner/非责任并拒绝通用 AgentRevision/全局 Run/第二执行角色；Lifecycles
      覆盖接纳、终态、取消、恢复、清理和 projection
  - id: route-readmes-indexes-and-examples
    files:
      - echo-agent/README.md
      - echo-agent/README.zh.md
      - echo-agent/docs/en/README.md
      - echo-agent/docs/zh/README.md
    summary: 从双语 README 与文档索引路由到基础页和现有可执行 learning contracts
    verify: Root Architecture 段成为简化分层入口，已验证 feature/topology/example 摘要保留；docs
      indexes 顺序对等，示例链接指向现有 target/contract
  - id: record-concept-convergence-evidence
    files:
      - echo-agent/.echo-semantic
    summary: 更新 framework docs Asset、Capability Maps 与验证证据，不批量关闭 open Finding
    verify: 新增 concept-navigation Evidence/Audit；92 个 Finding 保持 71 open/21
      resolved，所有 Issue 映射唯一且 remote state不因文档导航改变
artifact_id: plan:2b5bb6e3-ec42-4262-bc7c-543f87bbc60b
lifecycle: completed
design_revision: sha256:66b3925591ca1967fad6cbf3331cf7fb9a6ec73c9dfe3d6ca28e02f76b8eed7e
---
## Approach

- 在新的 `doc/Echoyue/framework-concept-navigation` 分支从当前 `7ba1f122` 基线执行。先按已确认 design 写 ADR0040 作为子仓库架构权威，再记录带 `--architecture-change`、design authority 与 ADR digest 的 semantic-preflight；任何测试/业务代码修改都发生在 preflight 之后。
- 扩展 `echo-agent-learning/tests/documentation_contract.rs`：先要求六份基础页和双语导航入口存在，取得缺失文件 red；发布文档后再验证 en/zh heading-level profile、表格/diagram 结构、root/index 链接与本地 link resolution。
- `architecture.md` 只讲 11-package DAG、分层、root facade、consumer/adapter/application boundary 与能力路由，不复制 feature/API 细节。
- `concepts.md` 以 owner / identity / persistence / non-responsibility 表定义 Agent、Session、Conversation、Invocation、Turn、Task、Plan、Subagent、Context、Checkpoint、Journal、Projection、Trace、Delivery 和限定 Revision。
- `lifecycles.md` 以 Agent Turn、Context/Persistence、Task/Subagent、Tool/Permission/Effect、Observation/Delivery、Extension、SDK 七条 flow 解释 accept、cancel、terminal、resume、cleanup、retention 与 projection。
- 六份基础页通过相对链接复用现有领域章节和已经进入 Cargo/example_contracts test chain 的 learning examples；不新增 example 源码。
- Root README 保留已验证的 feature/workspace/example 摘要，只将 Architecture 段收敛为短分层图和三份基础页入口；docs indexes 新增同序的 Start Here。
- 更新 `.echo-semantic` 的 framework docs Asset、workspace/相关 maps 与 concept navigation Evidence/Audit；不改变任何 Finding status 或 GitHub Issue state。

## Global Constraints

- 本 Plan 不修 runtime/API，也不批量修复 #53/#67/#101 或其它 71 个 open Finding；基础文档对未闭合行为只能收窄承诺。
- Cargo manifests 是 package/feature topology 权威；root facade/SDK inventory 是公共 surface 权威；source/tests/ADR/resolved Evidence 是行为依据；open Finding 决定可承诺上限。
- 不新增通用 AgentRevision、全局 Run/LifecycleState、第二套执行角色、Session registry、store、event bus、schema 或 generator。
- Plan 是可审阅 artifact，Task graph 拥有 revision/dependency/claim/settlement；Todo/Event/UI/Trace 是对应 projection/observation，不得写成状态权威。
- Context、transcript、runtime checkpoint、long-term memory、Journal、Trace、Delivery Ledger 与 Git/file checkpoint 必须明确分开。
- Framework 只记录 product-neutral 机制；EKO Workspace/Device/GUI/TUI/Product Backend policy 明确归 embedding application。
- Permission 是 runtime enforcement；prompt/rule/plan 不替代 decision。取消请求、EOF、最后一条文本和 UI state 不等于成功终态。
- 只使用 Subagent 术语；Revision/Run/Checkpoint 必须带 owner 限定。
- 中英文页面 heading hierarchy、表格对象、diagram、示例映射和导航顺序语义对等；不要求逐字翻译。
- 文档链接的 example 必须已存在且进入 Cargo example 或 test contract；不得新增未测试 snippet 或 wrapper。
- 不新增依赖或 SDK identity；SDK/contracts/sdks 路径零 diff。`echo-agent-cli` 与 `echo-website` 不修改。
- 现有 92/92 Finding-Issue 映射保持唯一；发现新独立问题立即先建 Issue/Finding，再决定是否进入当前 scope。

## Files

- Modify: `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — 基础文档存在性、双语结构、导航和链接合同。
- Create: `echo-agent/docs/en/architecture.md` — 11-package 分层架构与边界。
- Create: `echo-agent/docs/zh/architecture.md` — 中文对等架构文档。
- Create: `echo-agent/docs/en/concepts.md` — 核心术语、identity 与 owner。
- Create: `echo-agent/docs/zh/concepts.md` — 中文对等核心概念。
- Create: `echo-agent/docs/en/lifecycles.md` — 七条生命周期与终态/恢复/清理。
- Create: `echo-agent/docs/zh/lifecycles.md` — 中文对等生命周期。
- Create: `echo-agent/docs/adr/0040-framework-concept-documentation-authority.md` — 文档信息架构与事实源决策。
- Modify: `echo-agent/README.md` — 简化 Architecture 段并增加三份基础页入口。
- Modify: `echo-agent/README.zh.md` — 中文对等 README 入口。
- Modify: `echo-agent/docs/en/README.md` — 增加 Start Here 并路由基础页。
- Modify: `echo-agent/docs/zh/README.md` — 中文对等文档导航。
- Modify: `echo-agent/.echo-semantic` — framework docs/maps/source snapshot 与 concept navigation Evidence/Audit。

## Reuse

- `echo-agent/.echo-semantic/maps/*.md` — 11 张 Capability Map — 概念边界、状态权威、生命周期与 open limitation 的主要治理来源。
- `echo-agent/.echo-semantic/behaviors` 与 `rules` — 11 个 Behavior/11 条 Rule — 公共承诺与不变量。
- `echo-agent/docs/en|zh/39-framework-application-boundary.md`、`40-context-system.md`、`41-persistence-concepts.md`、`29-long-running-tasks.md` — 现有领域细节页。
- `echo-agent/docs/adr/0013-learning-examples-and-documentation-boundary.md`、`0014-framework-capability-placement.md`、`0031-sdk-identity-governance-scope.md`、`0032-sdk-contract-scope-classification.md` — 已确认边界决策。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — Markdown/link/Cargo metadata helpers — 扩展既有测试而不新增框架。
- `echo-agent/echo-agent-learning/examples` 与 `tests/example_contracts` — 已编译/测试的概念 consumer。
- `echo-agent/Cargo.toml` 与 root facade — package/feature/public composition 事实源。

## Todos

### establish-foundational-doc-contract

requirements:
- § 三层文档入口
- § 文档事实源
- § 异常与边界场景
- § 文档校验扩展现有 contract
- § 验收标准

interfaces:
- consumes: root/docs Markdown、现有 local-link/Cargo metadata helpers、六个预期基础路径与双语导航位置
- produces: foundation docs existence/navigation/structure/link fail-closed contract

steps:

1. 增加六份基础文档和 root/docs 双语入口的存在性/导航合同，并在文件尚不存在时取得有效 red。
   verify: red 只报告缺失的 architecture/concepts/lifecycles 页面或入口，不是编译/环境失败。
   expected: 文档发布有可回归的前置证据。

2. 在文档落地后扩展结构合同：比较 en/zh heading-level profile、表格与 code-fence/diagram 数量，并校验基础页、README、index 的本地链接可解析。
   verify: 任一双语页缺章节、表格/diagram、导航或相对链接目标时 fail closed；其它 40+ 领域页的历史文字不要求逐字一致。
   expected: 结构和路由可机器验证，内容正确性仍由 source/Evidence/review 判断。

### publish-architecture-authority

requirements:
- § 业界调研与取舍
- § 架构分层
- § 用户提出领域的去向
- § 系统边界
- § 文档事实源

interfaces:
- consumes: Cargo metadata DAG、root facade、SDK Host/protocol、learning consumer、framework/application ADR、confirmed industry references
- produces: 双语 architecture pages 与 ADR0040 documentation authority

steps:

1. 写 ADR0040，记录候选方案（重写所有领域页、单一巨型概念页、三层导航）、选择三层导航的理由、事实源优先级、open Finding 约束、兼容和回滚。
   verify: ADR 包含背景、候选、决策、取舍、影响范围和官方业界参考；不声明新 runtime/API。
   expected: preflight 可以绑定正式架构权威。

2. 发布双语 Architecture，展示 11-package DAG、七个 split framework crates、root facade、SDK protocol/Host、learning consumer、adapter/application boundary 与 capability 路由。
   verify: package/依赖角色与 Cargo metadata 一致；Product Backend、Frontend/Desktop、Device sync 明确不归 framework。
   expected: 读者可从架构页找到每个领域的 owner 文档而不把 SDK/EKO 混入 core。

### publish-concepts-and-lifecycles

requirements:
- § 核心概念与唯一权威
- § 生命周期主线
- § 用户提出领域的去向
- § 关键取舍

interfaces:
- consumes: Architecture 边界、11 张 Capability Map、11 个 Behavior/Rule、现有领域 docs/ADR、resolved Evidence 与 open limitations
- produces: 双语 Concepts glossary/authority matrix 与 Lifecycles flows/invariant tables

steps:

1. 发布双语 Concepts，逐项定义至少 14 个核心概念的 identity、owner、scope/persistence 与 non-responsibility，并解释限定 Revision/Run/Checkpoint。
   verify: Agent/Session/Conversation/Invocation/Turn/Task/Plan/Subagent/Context/Checkpoint/Journal/Projection/Trace/Delivery 全部可追溯；不出现未实现的聚合 authority。
   expected: 相邻同名概念可以由 owner 前缀消歧。

2. 发布双语 Lifecycles，覆盖 Agent Turn、Context/Persistence、Task/Subagent、Tool/Permission/Effect、Observation/Delivery、Extension、SDK 七条主线。
   verify: 每条包含 trigger/accept、state authority、events/effects、cancel/failure、terminal、resume/cleanup、projection；clear 操作按具体 authority 分开。
   expected: 文档不从 cancel request、EOF、trace/feed/UI 推断成功终态。

3. 在 Concepts/Lifecycles 添加到现有详细章节和已编译/测试 learning examples 的映射表。
   verify: 所有相对链接可解析，示例路径由既有 Cargo/documentation contracts 覆盖；无新 example 源码。
   expected: 概念读者可进入真实 API/consumer，不需要复制未测试示例。

### route-readmes-indexes-and-examples

requirements:
- § 三层文档入口
- § 新增导航层，不重写所有功能章节
- § 预期产物
- § 验收标准

interfaces:
- consumes: 六份基础页、现有 README Architecture 段、docs indexes、已验证 feature/workspace/example 摘要
- produces: root README 简化 Architecture 入口与双语 docs Start Here 导航

steps:

1. 将两份 root README 的 Architecture 段收敛为短分层图、三份基础页链接和 framework/application 说明，同时保留已验证 feature/workspace/example 摘要。
   verify: 现有 topology/feature/command contracts 保持 green；README 不复制完整 glossary/lifecycle。
   expected: README 首次读者先看到稳定入口，详细事实仍由 docs/Cargo 拥有。

2. 在 `docs/en/README.md` 与 `docs/zh/README.md` 顶部加入同序 Start Here，列出 Architecture、Core Concepts、Lifecycles、Framework/Application Boundary 与 Getting Started。
   verify: 双语导航顺序、链接和职责对等，现有 numbered domain index 保留。
   expected: 新旧文档形成导航层与领域层，不产生第二份 API 目录。

### record-concept-convergence-evidence

requirements:
- § 文档发布流
- § 正式文档不直接暴露内部 Finding 文本
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: 最终六页/README/index/ADR/test diff、工程日志、SDK zero-diff、92/92 Issue state 与独立 review
- produces: framework concept navigation Evidence/Audit、更新后的 docs Asset/Capability Map refs 与治理进度

steps:

1. 更新 framework docs Asset、workspace map 及七条 lifecycle 所属 maps，新增 concept-navigation verification Evidence；不改变任何 Finding status。
   verify: 新页面绑定 current source snapshot，历史 Evidence/Audit 固定到前一提交；open Finding 仍为71、resolved仍为21。
   expected: 正式文档可从语义地图追溯，同时不把 navigation 当 runtime repair。

2. 独立 review 后写 concept-navigation Audit 并执行最终语义、Issue、工程和文档门禁。
   verify: reviewer无 blocker；semantic strict/change-evidence、92/92 Issue映射、documentation/facade/example contracts和diff check通过；所有 Issue remote state不因本 outcome改变。
   expected: `publish-framework-concept-navigation` 可独立提交并进入治理最终验证。

## Decisions

- 三份基础页分别拥有 architecture、terminology/authority、lifecycle/navigation；现有 numbered docs 继续拥有 API/config 细节。
- README 保留已验证 feature/workspace/example 摘要，只压缩 Architecture 段；不删除刚闭合的三个合同。
- 双语语义正确性由 source/Evidence/review保证，机器合同验证结构、链接与路由，不做逐字翻译比较。
- ADR0040 在 preflight 前创建作为子仓库正式 authority；preflight 后才修改 Rust documentation contract。
- 本 outcome 不关联新 Finding/Issue，也不改变现有 Finding/Issue 状态。
