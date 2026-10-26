---
schema_version: 4
slug: eko-gui-product-convergence/a2a-lsp-retirement
outcome:
  summary: 在框架、SDK、EKO 和官网完成 A2A/LSP 实现、feature、注册、配置、插件组件、accepted 合同、示例和文档的精确退役。
  acceptance:
    - 框架源码、公开出口、feature、注册、配置及插件表面真正移除具名 A2A/LSP；不留成功空壳。MCP、ACP、channels、框架合理存储
      API、Task/Subagent 与 ProjectRoot 通用能力继续可用。
    - 旧声明仍由原 prepare/resolver/cache owner 冻结完整来源、身份与诊断；非法输入、读取和非普通文件错误不被退役
      Warning 覆盖，健康同级组件与既有 publication/rollback/shutdown 义务保持。
    - SDK 采用实际物理删除的精确框架提交及同源 Host/lock；相对 SDK1eac 的 retained5352 identities/431
      routes 与 accepted/source/shared 合同保持全字节，旧0930基线的269身份/23路由/39操作撤销证明保留。EKO
      在新框架上保持 GUI、IM/cron、插件和共享任务能力。
    - 所属仓正式文档、示例和验证矩阵与真实删除一致；官网镜像、导航、宣传与 discovery 绑定已审源，区分消费者去依赖、框架源码删除和正式发布；恢复
      tag/分支与历史证据保持。
    - 精确 source/lock 通过适用完整门禁、fresh依赖策略、独审和正常 TaskSemantic；构建结束按原规则及时 cargo
      clean。隔离本地实施使用合格消费者交接；main promotion 仍须消费者先 main/精确 CI 绿，再推进框架删除及后继采用，任何
      push/merge/Issue关闭/发布部署均另依授权执行，未获授权前只交可审阅本地候选而不报最终完成。
out_of_scope:
  - 不附带 TUI/CLI/JSONL 入口退役、IM/cron 产品接线、工具治理/领域插件或 GUI Subagent
    深度策略；不改变它们已确认的权威与合同。
  - 不因 EKO 未调用而删除其它合理框架 API，不删除 ACP/MCP/channels/sqlite、SDK
    Host、ProjectRoot/Instruction/scope 通用能力。
  - 不重复撤销已退役的269项，不改旧0930差量基线；不提前选择仍保留能力的e4e作后继pin，不允许保留合同漂移或新依赖风险自动接受。
  - Exa4343保持本机独立提交且不推送/合main；不重写基线/tag/stash/历史
    refs，不删除用户数据和失败原件；本Plan不增加外部写入或发布授权。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 5. 入口收敛与 A2A/LSP 退役
    digest: sha256:c9c8f28566cfe1936dad26af7d33c6ec5a8fa0a0b9af1215b1ae3d8e7cffc09d
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:6f59f80f3c5fe48e08bac490651377bf47f90e5dbde6560e47c4e5d16d9fcde5
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#a2a-lsp-retirement
todos:
  - id: freeze-qualified-consumer-handoff
    summary: 冻结已合格消费者与框架交接，在隔离分支开始具名退役，并明确后续 main 顺序。
    files:
      - echo-agent/docs/adr/0090-a2a-lsp-retirement-contract.md
      - echo-agent-sdk/docs/adr/0042-sdk-a2a-lsp-retirement-boundary.md
      - echo-agent-sdk/docs/sdk/governance/a2a-lsp-retirement-boundary.md
      - echo-agent-cli/docs/en/adr/0050-gui-product-convergence.md
      - echo-agent-cli/docs/zh/adr/0050-gui-product-convergence.md
    acceptance:
      - 先核 SDK1eac 与 EKO4ab 各自适用 full、独审、TaskSemantic、依赖策略和实际清理原件，证明均已在保留
        A2A/LSP 的框架上去依赖；不得借 Plan 文件、focused、旧Host或报告数量代替。
      - 新任务源从完成合流资格并本地提交的 framework GUI-runtime 候选交接（634a+Hook91d+hardW
        e713/7f+ZIP439），冻结其真实
        source/lock/父系/owner与正常验证原件。复用旧工具报告及SDK窄pin接缝，仅核实际增量，不整文件回灌旧e4e/2b库存。
      - 隔离本地代码和候选资格不依赖外部 main 操作；对外/main promotion 严格保持消费者去依赖先合入其 main
        且精确CI全绿，框架删除后推进及SDK实际新pin采用。若 main/source
        在交接时有新变化，重新核本地资格与promote顺序，不把旧原件改标新资格。
      - 以具名 symbol/path 清单判定 A2A/LSP 删除；保留
        PluginRegistry.project_root、Project/Local
        scope、variables.project_dir、ResolvedInstructions/InstructionResolver
        及SDK对应桥。Design1add/map6dae、Plan04同一artifact_id的269项范围、原恢复分支/tag及旧合同差量事实保持；当前执行核同一Plan04的分阶段门禁与真实后继交接，不把旧快照SHA视为已晋升main的证明。
  - id: remove-framework-capability-surfaces
    summary: 物理删除框架A2A/LSP及其注册与公开投影，保留原owner的冻结旧声明诊断。
    files:
      - echo-agent/Cargo.toml
      - echo-agent/Cargo.lock
      - echo-agent/src/a2a/
      - echo-agent/src/lsp.rs
      - echo-agent/src/lib.rs
      - echo-agent/src/tools/lsp.rs
      - echo-agent/src/tools/mod.rs
      - echo-agent/echo-core/Cargo.toml
      - echo-agent/echo-core/src/lsp/
      - echo-agent/echo-core/src/lib.rs
      - echo-agent/echo-core/src/plugin/capability.rs
      - echo-agent/echo-core/src/plugin/registry.rs
      - echo-agent/echo-integration/Cargo.toml
      - echo-agent/echo-integration/src/lsp/
      - echo-agent/echo-integration/src/lib.rs
      - echo-agent/src/plugin/prepared.rs
    acceptance:
      - 删除具名模块、root/prelude/export、a2a/lsp
        leaf与full成员、跨crate转发、工具注册、LspServer值及ResolvedComponents.lsp_config_file/PreparedPlugin.lsp_document/PluginWiringResult.lsp_discovered等专属表面；消除悬空cfg/import/positive
        tests，不保空pub
        shim或默认值成功。其它依赖仅在全framework检索确认最后使用消失后移除；不能顺带删ACP/共享axum/通用source API。
      - 移除执行API后，原prepare仍在cold
        owner内识别旧lsp.yaml并冻结文本/输入identity与全部通用diagnostics；取消feature-on/off分叉但保既有非LSP正常行为。重用现有PreparedPluginDocument/hash/diagnostics内部机制，无新public元数据、平行reader/decoder/store；EKO/SDK只消费完整诊断，不再lsp_document或读取文件补消息。
      - 旧声明矩阵覆盖合法UTF-8文本Warning、非法UTF-8/读失败Error、目录/FIFO/设备等非普通文件、metadata
        denied、已解析源消失、broken/looping/有效symlink、真正缺席；非普通文件先分类不阻塞读。component/path/plugin/severity与原错误不被Warning替代；混合健康MCP/Skill/Hook继续处理。旧capability/feature/字段输入沿各自原入口明确unsupported/validation，不假空成功，也不擅自把致命清单错误改成可加载。
      - 缓存命中返回同一冻结集合且无重复诊断；prepare后改文本/删源/换链接不改已有generation，显式invalidation后才反映新输入并保持identity/hash可解释。publication/rollback/shutdown沿原typed
        receipts与cleanup owner，不能用组件删除丢弃未结算义务。
      - 证明删除是实际源码/feature/API撤销，剩余LSP/A2A字符串只允许旧输入诊断/负向夹具/真实历史文档等具名用途；历史证据不参与“零字符串”清理。保留source控制对照，旧F
        feature-on成功原件不重标成新F支持。
  - id: adopt-real-deletion-source-in-consumers
    summary: 让SDK精确采用真实删除提交并完成EKO后继兼容，保持全部保留合同与历史撤销证明。
    files:
      - echo-agent-sdk/echo-sdk-host/Cargo.toml
      - echo-agent-sdk/Cargo.lock
      - echo-agent-sdk/echo-sdk-host/src/core_profile/facade/source_operations.rs
      - echo-agent-sdk/scripts/check-sdk-contracts.sh
      - echo-agent-sdk/scripts/check-a2a-lsp-retirement.py
      - echo-agent-sdk/echo-sdk-protocol/src/bin/export_schema.rs
      - echo-agent-sdk/echo-sdk-protocol/src/inventory.rs
      - echo-agent-sdk/contracts/sdk/
      - echo-agent-sdk/sdks/shared/
      - echo-agent-sdk/scripts/export-language-sdk-catalog.sh
      - echo-agent-sdk/scripts/build-telemetry-test-collector.sh
      - echo-agent-cli/Cargo.toml
      - echo-agent-cli/echo-agent-app-core/Cargo.toml
      - echo-agent-cli/Cargo.lock
      - echo-agent-cli/echo-agent-app-core/src/plugin_runtime/
      - echo-agent-cli/echo-agent-app-core/src/config_discovery.rs
    acceptance:
      - 只有实际A2A/LSP物理删除commit才可成为后继SDK pin；canonical HTTPS manifest rev、唯一lock
        git source、Host resolved metadata与实构制品、check-sdk-contracts.sh expected
        source一致。check-a2a-lsp-retirement.py的build-pin断言受审迁移，0930/pre-retirement
        tag历史差量断言不改。e4e不是删除提交，不现在选pin。
      - 按既有SDK接缝处理两处旧lsp_config_file依赖：source_operations生产adapter与原projection测试构造/断言；消费新F通用完整diagnostics，不重建发现器，不恢复伪空旧字段/LspServer
        variant。保ProjectRoot/instruction/scope桥和其它合法组件，EKO在实际新F上仍不依赖删除API，保shared
        lifecycle/IM/cron/TaskRuntime。
      - 采用唯一full generator，以已提交5352 seed更新新源provenance/全Rust
        inventory遥测；不再次retire269。5352完整entry dict与accepted JSON、431
        routes/catalog及保留closed
        ops、source-contract、shared语言catalog/digests/extension
        schema相对1eac须保持全字节不变；若实际出现retained drift、缺失accepted
        source或生成失败，停止归因/报审，不能盲update、手改JSON回填或借遥测非阻断忽略。
      - 旧0930→current的269 identities（原266+3）、23 routes、39
        operations、19个实际source-routed负向地址、47
        intrinsic归属与原旧digest/签名夹具保持。新匹配合同和ACP/MCP/Task/Subagent/R2父子close/取消观察/恢复、三语原examples/Host正常控制继续实际通过；数量不代替精确集合/字节证明，19地址不称269项逐项运行。
      - 新pin/profile矩阵来自真实Cargo解析与原generator，不按猜测手删写死profiles；旧f7bac
        Host/collector/qualification不验证新source。需要本地candidate资格时只用明确获授权的隔离canonical
        Git cache方案，禁止正式path/file源、未审patch/global
        insteadOf；最终squash/源SHA改变须重验新epoch，不能重标旧raw或假称公开可拉。
  - id: synchronize-source-docs-examples-and-site
    summary: 同步正式源文档、示例和官网的实际物理退役事实，保持源码归属与恢复历史。
    files:
      - echo-agent/README.md
      - echo-agent/README.zh.md
      - echo-agent/docs/en/
      - echo-agent/docs/zh/
      - echo-agent/docs/internal/knowledge/
      - echo-agent/docs/adr/0090-a2a-lsp-retirement-contract.md
      - echo-agent/echo-agent-learning/Cargo.toml
      - echo-agent/echo-agent-learning/examples/demo23_a2a.rs
      - echo-agent/echo-agent-learning/tests/example_contracts.rs
      - echo-agent/echo-agent-learning/tests/example_contracts/demo55_lsp_tools.rs
      - echo-agent/.github/workflows/
      - echo-agent-sdk/README.md
      - echo-agent-sdk/README.zh.md
      - echo-agent-sdk/docs/sdk/
      - echo-agent-sdk/docs/adr/0042-sdk-a2a-lsp-retirement-boundary.md
      - echo-agent-cli/docs/en/
      - echo-agent-cli/docs/zh/
      - echo-website/docs-sync-manifest.json
      - echo-website/scripts/sync-docs.mjs
      - echo-website/scripts/generate-discovery.mjs
      - echo-website/src/docs/registry.ts
      - echo-website/src/docs/framework-adrs.generated.ts
      - echo-website/src/components/HomePage.tsx
      - echo-website/src/docs/content/
      - echo-website/public/llms.txt
      - echo-website/public/llms-full.txt
    acceptance:
      - 正式framework能力/配置/feature/API说明与示例注册、learning Cargo入口、LSP/A2A专用example
        contracts和CI
        feature项一起物理对账；删除专用可执行示例并清编译/测试链引用，保其余examples可执行验证。历史ADR/旧源恢复说明可保留且明确历史身份，不改成假当前支持。
      - 官网先从已审真实framework删除源同步双语正式镜像与manifest
        source/hash，再更新LSP/A2A导航与Home当前能力宣传、discovery/llms派生产物；不能只改首页或用SDK本地去依赖标框架已删。历史ADR/退休说明入口按真实历史保留，无悬空导航/恢复链接，不伪造未推送commit的公共URL。
      - EKO应用投影继续绑定其实际已审source
        revision与源摘要，保GUI/IM/cron/终端/共享恢复；不得把本Outcome写成CLI/TUI已退役。SDK文档分别说明当前build
        pin与原269撤销历史epoch，Rust telemetry不升级为accepted。
      - 采用既有docs:sync/docs:check:source、discovery和官网verify完整入口，源码镜像检查对准显式干净source而非auto误取primary旧checkout；跨仓source变化由原唯一docs-sync
        owner串行合流，不覆盖其它GUI/设置/插件/网站贡献。此阶段不执行网站部署或包发布。
  - id: qualify-new-locks-and-final-cross-repo-source
    summary: 对最终删除源与新锁完成fresh依赖独审、最小规则同步及全范围交付资格。
    files:
      - AGENTS.md
      - echo-agent/AGENTS.md
      - echo-agent/.cargo/cargo-deny-duplicate-baseline.json
      - echo-agent/scripts/check-cargo-deny-baseline.py
      - echo-agent-sdk/AGENTS.md
      - echo-agent-sdk/.cargo/cargo-deny-duplicate-baseline.json
      - echo-agent-sdk/docs/sdk/governance/dependency-warning-policy-proposal.md
      - echo-agent-cli/AGENTS.md
      - echo-agent-cli/.cargo/accepted-dependency-risks.json
      - echo-agent-cli/scripts/check_cli_dependency_risks.py
      - echo-agent-cli/docs/supreme/specs/cancellation-observation-consumer/dependency-risk-decision.md
      - echo-agent/.github/workflows/
      - echo-agent-sdk/.github/workflows/
      - echo-agent-cli/.github/workflows/
      - echo-website/package.json
    acceptance:
      - 本次删除触及manifest/features/public
        API，必须从删后实际manifest枚举剩余独立feature并完成所属workspace
        full/no-default/Clippy/fmt/适用矩阵；移除且仅移除不存在a2a/lsp的正向矩阵项，ACP/MCP/channels/sqlite等继续覆盖。SDK/Host/三语/collector与EKO
        GUI/前端/官网按最终改动完成原完整门禁，所有失败保raw并修复同入口，不仅挑focused子集。
      - F/S
        lock图、pin、feature/API、检查scope/工具版本/策略变化均触发fresh全scope原audit及cargo-deny四类raw+自然退出码、精确crate/version/source
        group差集与独立复审；不按49或50数量、原baseline摘要或“lock只缩小”自动通过。SDK
        AGENTS当前50字样与机器baseline49的epoch差异要归因，保历史接受，只最小同步当前准确合同。无新增ignore/allow、风险接受或警告豁免。
      - EKO依赖风险按其独立actual-lock/canonical-origin两次检查与六项既有工具测试、精锁/consumer/source/advisory身份执行fresh复评；不继承F/SDK
        duplicate例外、不将原六项风险改称resolved、不自动续期新锁。保完整scanner自然出口和独立gate真实结果，不能把已接受raw1写成零warning；其它扫描/配置/编译失败仍阻断。
      - 最小更新所属机器baseline/规则/ADR仅发生在新source/lock的fresh
        raw与独审闭合后，根与相关子仓AGENTS同步feature条件/当前精确合同，历史raw与旧裁决保持。不为跑通删除精确guard/扩大baseline/添加skip或新的风险接受；未能闭合则本Outcome不完成。
      - 最终交付冻结四仓实际source/pin/lock、same-byte
        retained合同、正式文档/镜像、负向与保留控制、平台及mock/真实范围、独审和精确main
        CI。framework候选与最终squash/main来源、SDK新pin和官网manifest分别对账；最终源SHA变化触发新epoch资格，不把源码合流或旧Host/原始退出结果冒充当前交付。
artifact_id: plan:2ee00bb3-1b06-4fde-a5e1-520792617316
lifecycle: ready
---
