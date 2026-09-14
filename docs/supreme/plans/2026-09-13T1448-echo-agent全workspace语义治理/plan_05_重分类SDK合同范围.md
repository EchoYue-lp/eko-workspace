---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/重分类SDK合同范围
goal: 把Rust public facade inventory与真正跨语言SDK合同分层，停止用intrinsic
  route或identity完成率替代外部能力范围
ships: 将 SDK inventory 重分类为真实对外 contract、Host 或 Rust-only、language intrinsic、内部
  helper 与 deferred backlog，并保留漂移监控而非完成度门禁
verify: 生成后的9682个canonical identity分别为external_contract 5606、host_or_rust_only
  1765、language_intrinsic 780、internal_helper 90、deferred 1441；alias继承canonical
  scope，external_contract三语言全部done；parity schema升级而extension protocol/operation
  catalog保持不变；SDK合同、三语言catalog、语义快照与change
  evidence全部通过，业务runtime、examples和website无差异
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#sdk-contract-scope-reset
todos:
  - id: define-sdk-scope-authority
    files:
      - echo-agent/echo-sdk-protocol/src/inventory.rs
      - echo-agent/docs/adr/0032-sdk-contract-scope-classification.md
      - echo-agent/docs/sdk/README.md
      - echo-agent/docs/en/README.md
      - echo-agent/docs/zh/README.md
    summary: 定义identity级SdkScope、确定性规则和正式治理合同
    verify: scope与route/status正交；规则按固定优先级可重算，551个已完成intrinsic保持external
      contract，1441个deferred只按capability推进
  - id: generate-scope-contracts
    files:
      - echo-agent/contracts/sdk/parity-manifest.schema.json
      - echo-agent/contracts/sdk/parity-manifest.json
      - echo-agent/contracts/sdk/source-contract.json
      - echo-agent/sdks/shared/contract-digests.json
    summary: 通过唯一生成器刷新manifest schema、scope字段和source digests
    verify: 只通过export_schema --update生成32MB manifest；schema version升级，extension
      protocol和facade-operation-catalog内容不变，生成结果byte-stable
  - id: update-scope-consumer-gates
    files:
      - echo-agent/echo-sdk-protocol/tests/facade_inventory.rs
      - echo-agent/scripts/check-language-sdks.sh
      - echo-agent/sdks/typescript/test/catalog.test.js
      - echo-agent/sdks/python/tests/test_catalog.py
      - echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java
    summary: 让Rust与三语言门禁按sdk_scope验证外部合同而非按intrinsic route猜测
    verify: external_contract必须三语言done；其它scope保留显式disposition与统计；代表identity、alias继承、deferred不伪装完成均有合同测试
  - id: sync-semantic-contract
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 同步SDK语义对象、正式进度和本次公共合同变更证据
    verify: semantic-diff解释schema/manifest/digest与文档变化，SDK gap Finding不被错误关闭；strict
      snapshot、semantic-verify与独立review通过
artifact_id: plan:0f51d326-400f-467f-9426-b177a65026a3
lifecycle: completed
design_revision: null
---
## Context

- ADR 0031 已确认9,682个Rust public identity是SDK漂移inventory，不是项目完成度；当前5,606个三语言done，4,076个not-done全部落在intrinsic route。
- 现有`classification`描述wire/shape类型，`route.surface`描述适配路径，`languages.*.status`描述实现证据；三者都不能单独回答“是否属于外部SDK合同”。
- 当前未完成集合混合了2,346个callable method、880个language intrinsic、process-local handles/traits/callbacks和真正待评估capability，继续逐identity推进会重复原来的范围错位。
- Plan 04 的高风险审计已在`f1e90272`提交；本Plan只重分类SDK治理合同，不修复其它runtime Finding。

## Approach

- 在`ManifestEntry`新增identity级`SdkScope`，取值固定为`external_contract`、`host_or_rust_only`、`language_intrinsic`、`internal_helper`、`deferred`。Scope不进入`RouteObligation`，因为一个route可以聚合多个scope。
- `languages.*.status`继续表示“是否已有语言实现与contract test”，`sdk_scope`表示“当前消费合同归属”；二者不能合并。已完成identity一律属于external contract，不能因route为intrinsic降级。
- 固定规则优先级：三语言全done → external；未完成且`echo_agent::testing::*` → internal helper；未完成且reason为Rust language/trait/callback → language intrinsic；未完成且reason为process-local/host-owned/language-local opaque resource → host-or-Rust-only；其余 → deferred。
- Alias继承canonical source identity的scope，不独立计数。Deferred只能按外部有价值的capability做决策；未来Host/native authority、route和行为证据建立后再由规则晋级，禁止手改scope绕过合同。
- Schema新增必填字段并将ParityManifest schema version从1升2；wire extension protocol和operation catalog不变。32MB manifest只由`export_schema --update`生成，不人工编辑。
- 业界依据：
  - [Smithy 2.0 service types](https://smithy.io/2.0/spec/service-types.html)以service closure、operation input/output/error作为API模型，并允许生成代码做语言惯用转换。
  - [gRPC core concepts](https://grpc.io/docs/what-is-grpc/core-concepts/)以IDL service、remote methods和payload messages定义跨语言合同。
  - [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)明确区分语言无关接口描述与各语言client/server生成物。
- EKO取舍：以ACP/extension operation/value/handle行为作为外部合同，以Rust facade inventory做漂移监控；语言固有语法、Host进程资源、测试helper和deferred capability保留显式分类但不制造伪parity任务。

## Global Constraints

- 不人工编辑`contracts/sdk/parity-manifest.json`；只修改生成器并运行唯一update入口。
- `sdk_scope`是identity属性，不改变route、wire method、Host handler或operation catalog。
- 预期canonical计数必须精确为5606/1765/780/90/1441并合计9682；统计变化必须由规则或源码事实解释，不能为迎合计数硬编码identity清单。
- 551个三语言done且intrinsic route的identity继续是external_contract。
- `external_contract`必须三语言`done`且有具名contract test；其它scope不得被总量完成率解释为缺失外部功能。
- ParityManifest schema version升2；`EXTENSION_PROTOCOL_VERSION`、facade operation catalog schema/content保持不变。
- Framework合理public API不因EKO未使用而删除；本Plan不删除Rust API、不改变runtime行为、不映射新的language facade。
- `echo-website`不更新：本次只改变SDK治理metadata和正式framework文档，没有新增对外runtime capability。
- 任何公共contract/source digest变化都调用semantic-diff，完成前调用semantic-verify；Rust/脚本/三语言门禁仍独立执行。

## Files

- Modify: `echo-agent/echo-sdk-protocol/src/inventory.rs` — 新增SdkScope、确定性分类函数和ManifestEntry字段。
- Modify: `echo-agent/echo-sdk-protocol/tests/facade_inventory.rs` — 增加scope总量、规则、alias、代表identity和deferred合同。
- Modify: `echo-agent/scripts/check-language-sdks.sh` — external contract按scope执行三语言完成门禁并输出分类统计。
- Modify: `echo-agent/sdks/typescript/test/catalog.test.js` — TypeScript catalog按scope验证。
- Modify: `echo-agent/sdks/python/tests/test_catalog.py` — Python catalog按scope验证。
- Modify: `echo-agent/sdks/java/src/test/java/com/echoagent/sdk/FacadeParityTest.java` — Java catalog按scope验证。
- Modify: `echo-agent/contracts/sdk/parity-manifest.schema.json` — 生成SdkScope schema和必填字段。
- Modify: `echo-agent/contracts/sdk/parity-manifest.json` — 为全部entry生成sdk_scope。
- Modify: `echo-agent/contracts/sdk/source-contract.json` — 刷新manifest/aggregate digest。
- Modify: `echo-agent/sdks/shared/contract-digests.json` — 刷新共享source-contract digest。
- Create: `echo-agent/docs/adr/0032-sdk-contract-scope-classification.md` — 记录背景、业界方案、决策、取舍和影响。
- Modify: `echo-agent/docs/sdk/README.md` — 用scope ladder替代intrinsic完成门禁叙述。
- Modify: `echo-agent/docs/en/README.md` — 登记ADR 0032。
- Modify: `echo-agent/docs/zh/README.md` — 登记ADR 0032。
- Modify: `echo-agent/.echo-semantic` — 刷新SDK map/behavior/rule/evidence/snapshot与change evidence。
- Modify: `docs/MASTER-PLAN.md` — 更新本阶段真实状态和下一repair frontier。

## Reuse

- `echo-sdk-protocol/src/inventory.rs` — 复用现有SemanticClass、AcpRelationship、route reason、语言status与唯一ManifestEntry装配点。
- `echo-sdk-protocol/src/facade.rs` — 复用process-local/host-owned/intrinsic reason，不建立第二route resolver。
- `echo-sdk-protocol/src/bin/export_schema.rs` — 唯一manifest/schema/catalog/source-contract生成入口。
- `contracts/sdk/facade-operation-catalog.json` — 保持语言中立operation contract与字节内容不变。
- `scripts/check-sdk-contracts.sh`与`scripts/check-language-sdks.sh` — 继续作为Rust contract与三语言source gate。
- `docs/adr/0031-sdk-identity-governance-scope.md` — 继续作为inventory不等于项目完成度的上游决策。
- `evidence.high-risk-audit-frontier` — 保留Plan04 Finding分类；SDK scope变更不关闭runtime/protocol Finding。

## Todos

### define-sdk-scope-authority

requirements:
- 用户确认：SDK inventory继续用于漂移监控，但必须区分真实外部contract、Host/Rust-only、language intrinsic、internal helper和deferred。
- AGENTS.md：关键公共合同决策先调研成熟实现，并明确本项目取舍。

interfaces:
- consumes: `ManifestEntry`、SemanticClass、route reason、canonical/alias identity与languages status。
- produces: `SdkScope` enum、`sdk_scope_for`唯一规则和ADR 0032正式语义。

steps:

1. 在ADR 0032记录Smithy/gRPC/OpenAPI的语言中立contract模式，并定义五类scope、优先级、升级条件和兼容影响。
   verify: ADR含背景、候选方案、最终决策、取舍、影响；明确scope不等于route/status且不授权删除Rust API。
   expected: Reviewer可以从单一规则解释每类identity为何进入当前scope。

2. 在inventory generator定义`SdkScope`和纯分类函数，并由唯一ManifestEntry装配点赋值；alias继承canonical scope。
   verify: 不新增第二route resolver或手工9682项清单；函数输入只使用当前可重算source/status事实。
   expected: 当前规则精确生成5606/1765/780/90/1441，551个done intrinsic保持external。

3. 更新SDK README和双语ADR索引，删除“所有intrinsic未实现才不Runnable/Parity complete”的过重口径。
   verify: 文档只按external capability/contract声明可用性；Host/Rust-only/intrinsic/helper/deferred均有清晰定义。
   expected: 使用者能区分wire operation、Host资源、语言本地值和待评估capability。

### generate-scope-contracts

requirements:
- 用户确认：保留完整inventory和drift gate，但不逐identity手工推进。
- 生成合同：manifest/schema/source digest只能从同一export入口产生。

interfaces:
- consumes: 更新后的inventory generator与固定nightly rustdoc input。
- produces: schema v2 parity manifest、source-contract digest和shared digest；operation catalog保持原内容。

steps:

1. 运行唯一update入口生成manifest/schema/source-contract，并刷新shared contract digest。
   verify: 生成前后operation catalog哈希一致；manifest每个entry有合法sdk_scope且canonical统计匹配。
   expected: 大文件diff只包含确定性字段/schema/digest变化，没有手工排序或格式漂移。

2. 再次以只读模式生成并比较。
   verify: 第二次生成零diff，source-contract聚合digest与shared digest一致。
   expected: 合同输出byte-stable，CI可重复计算。

### update-scope-consumer-gates

requirements:
- 用户确认：外部SDK完成度只以真正external contract衡量。
- 现有三语言source clients和contract tests继续保留，不删除已实现intrinsic能力。

interfaces:
- consumes: schema v2 ManifestEntry.sdk_scope与现有languages status/contract_test。
- produces: Rust/TypeScript/Python/Java一致的scope gates和统计。

steps:

1. 更新Rust facade inventory tests，验证五类全覆盖、alias继承、规则优先级、计数与代表identity。
   verify: external_contract三语言全done；deferred/host/intrinsic/helper不被伪装为done或强制实现。
   expected: 规则漂移会在Rust合同测试直接失败。

2. 更新language gate和三语言catalog tests，不再用`route.surface != intrinsic`推断外部合同。
   verify: 四套consumer对同一sdk_scope和计数达成一致；wrong/missing scope fail closed。
   expected: 后续新增Rust identity必须得到确定scope，但不会自动生成逐identity语言任务。

3. 执行focused Rust、SDK contract与三语言source门禁。
   verify: `cargo fmt --all -- --check`、facade inventory test、`./scripts/check-sdk-contracts.sh`、`./scripts/check-language-sdks.sh`全部exit 0。
   expected: 生成器、schema、Host digest和三语言consumer在同一source revision闭合。

### sync-semantic-contract

requirements:
- Echo Semantic：公共contract变更在首个diff后执行semantic-diff，完成前执行semantic-verify。
- 文档同步：架构决策、SDK正式文档和顶层MASTER-PLAN必须反映真实行为。

interfaces:
- consumes: 最终scope规则、生成artifacts、focused gate结果与Plan04 frontier。
- produces: 当前source digest的SDK语义对象/change evidence、MASTER-PLAN状态与独立review结论。

steps:

1. 刷新SDK map/behavior/rule/evidence和baseline source digest，记录scope变化不改变wire/runtime。
   verify: semantic-diff只报告identity治理schema/digest/文档变化；operation catalog与runtime authority义务保持preserved。
   expected: sdk-gap-generation等开放Finding不被scope重分类错误关闭。

2. 执行semantic-verify、独立review和diff scope检查。
   verify: strict snapshot/change evidence、review、git diff check通过；业务runtime、examples和website零差异。
   expected: SDK scope reset可独立提交并停止，下一步按capability或Finding而非identity数量推进。

## Decisions

- 新字段名为`sdk_scope`，属于ManifestEntry而非RouteObligation。
- 采用“已交付外部contract优先”的规则；551个done intrinsic不得降级。
- 1441个deferred只作为capability backlog，不作为1,441个独立任务。
- Minimal方案不修改facade operation catalog；未来若需consumer展示scope，另行设计多值聚合字段。
- ParityManifest schema version升2，extension protocol与operation catalog版本保持不变。
- 官方Smithy/gRPC/OpenAPI模式共同支持“语言中立操作/消息合同 + 语言投影”，本项目不再以Rust语法表面作为跨语言合同。