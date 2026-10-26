---
schema_version: 4
slug: eko-gui-product-convergence/a2a-lsp-consumer-decoupling
outcome:
  summary: 在仍保留 A2A/LSP API 的框架上完成 EKO/SDK 消费者去依赖，精确退役相应工具、feature
    请求与已接受合同，保留其余能力可构建和可用。
  acceptance:
    - 在仍保留A2A/LSP实现的framework上，EKO与SDK不再请求、注册、广告、调用或投影退役能力；两消费者均完成各自适用本地完整资格并冻结交接后，可向a2a-lsp-retirement提供隔离分支实施与候选验证依据，不以一仓完成代替两侧资格。此本地交接不表示本Outcome已正式交付；两消费者先main及适用精确CI全绿，仍是框架删除main晋升的前置。
    - 先在框架既有prepare/resolver与通用diagnostics内交付无LSP专属API的冻结声明诊断，再切换EKO并证明不再调用lsp_document等专属元数据入口；框架诊断描述构建能力边界，EKO解释产品退役，保持单一prepared
      identity/generation，不新增公共API或状态权威。
    - SDK正式accepted身份按source_paths[0]+kind逐项撤销原266（A2A160，含47个intrinsic；LSP106）及本次明确授权新增3，共269；基线5621到5352，新增接受0、额外身份退役0。catalog精确撤销两家族、两方法、39个closed
      operations和23条route，454到431；其它家族/身份保留，必要聚合值shape和digest变化逐字段归因，不借全量Rust
      inventory扩删。
    - 旧digest/要求旧feature的扩展准入、旧family method/operation、绕过语言catalog的旧source
      invoke及三项插件旧值/字段均沿既有边界明确失败或不再暴露；不得返回伪空成功。匹配新合同的ACP/MCP/Task/Subagent/Host
      stdio、流/取消/恢复、R2父子close及三语言examples继续通过。
    - EKO旧配置/混合插件请求LSP时有明确组件诊断；合法MCP/Skill/任务/IM/cron/终端与原publication、rollback、cleanup
      receipt owner保持。运行时、feature、IPC、UI/设置、投影和消费者测试共同对账，未宣告仅隐藏菜单即退役。
    - SDK/EKO正式文档、examples可执行入口及官网影响有明确结论；官网源镜像由协调者随实际交付批次更新，不为未交付能力提前宣告。最终交付记录精确source/framework
      pin/lock、生成差集、实际退出与平台/真实或mock范围；各仓适用门禁和精确main CI按原交付规则验收，源码合流不等于已发布。
out_of_scope:
  - 本Plan只交付a2a-lsp-consumer-decoupling；不删除framework
    A2A/LSP生产API/feature，不更新SDK的f7bac62eee04a963f92a3e4308b6ea2336e91cc0
    pin，不把SDK仍反射到旧pin的Rust inventory当作新增接受或删除理由。
  - 不退役EKO独立CLI/TUI/JSONL/channel产品入口，不连带删除src/cli/channels共享IM逻辑；只去除其中LSP消费。console-surface-retirement、工具一般收敛、领域插件与性能优化分别交后续Outcome。
  - 不新增兼容执行器、平行Store/状态机、假空LSP元数据或新的通用失败协议；不改变ACP/MCP/Task/Subagent/R2原owner，不按名称整包删除SDK
    shared bridge/integrations。
  - Exa4343f0b736beb5c8606e31146528a2fe8be80510及保留tag/stash/必要历史refs不动；不把此计划创建授权解释为当前生产实现、push/merge
    main、包发布或官网部署授权。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 5. 入口收敛与 A2A/LSP 退役
    digest: sha256:c9c8f28566cfe1936dad26af7d33c6ec5a8fa0a0b9af1215b1ae3d8e7cffc09d
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:6f59f80f3c5fe48e08bac490651377bf47f90e5dbde6560e47c4e5d16d9fcde5
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#a2a-lsp-consumer-decoupling
todos:
  - id: freeze-exact-consumer-dispositions
    summary: 固化269项精确SDK处置与框架冻结诊断接缝，建立可先行交付的消费者去依赖依据。
    files:
      - echo-agent/src/plugin/prepared.rs
      - echo-agent/echo-core/src/plugin/registry.rs
      - echo-agent/docs/adr/0090-a2a-lsp-retirement-contract.md
      - echo-agent/docs/en/32-plugin-system.md
      - echo-agent/docs/zh/32-plugin-system.md
      - echo-agent-sdk/docs/adr/0042-sdk-a2a-lsp-retirement-boundary.md
      - echo-agent-sdk/docs/sdk/governance/a2a-lsp-retirement-boundary.md
      - echo-agent-sdk/contracts/sdk/accepted-external-contract.json
      - echo-agent-sdk/contracts/sdk/accepted-facade-operation-catalog.json
      - echo-agent-sdk/echo-sdk-protocol/src/bin/export_schema.rs
      - echo-agent-cli/docs/en/adr/0050-gui-product-convergence.md
      - echo-agent-cli/docs/zh/adr/0050-gui-product-convergence.md
    acceptance:
      - 复用现有MCP无feature诊断模式，在framework原PluginIntegrator::prepare/resolver
        owner内记录未启用lsp时的声明输入；合法文件冻结成功产生既有Warning并携带组件、插件身份与来源路径，InvalidUTF8或读取失败保留Error。目录、metadata失败与broken
        symlink等可识别旧声明也须在原owner内形成带组件/路径的明确诊断，不因旧optional-path字符串或NotFound分支静默丢失；不新建decoder/执行器或让EKO在prepare后重读文件。
      - 诊断随原PreparedPluginSet
        identity/generation及缓存冻结，合法MCP/Skill等保留组件继续独立处理；cache-hit返回同一冻结诊断集合，重复读取不向已有集合追加重复项，原Error不被Warning覆盖。启用lsp的原框架行为和其它组件不受中间桥接改变，框架文案只说明能力未启用，不包含EKO产品策略。完成相关默认/无lsp/有lsp正常对照与异常输入回归，先交付准确框架桥接，再供EKO消费；SDK原f7bac精确pin不因此更新。
      - 从正式accepted
        seed按原selector重算266及唯一source_paths[0]+kind键，另显式加入且仅加入三键：variant
        echo_core::plugin::capability::PluginCapability::LspServer；struct_field
        echo_agent::plugin::prepared::PluginWiringResult::lsp_discovered；struct_field
        echo_core::plugin::registry::ResolvedComponents::lsp_config_file。校验三键原已接受、与266无交集、并集269；不按canonical展示名或裸数量代替精确集合。
      - "SDK正式ADR补齐可机读的Status accepted/approved与每项独立Action:
        retire、Kind、Source、Reason、Compatibility、Replacement（明确unsupported）、Roll\
        back；沿用export_schema现有--retire-source <kind> <source> <repo
        ADR#section>，保存269键与ADR一一对应、源定义/调用检索和原seed摘要，不创建第二套accepted真理源。"
      - 实施前冻结F
        c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa、S961da0e0d17b815653d80cb4bf4973248f543770、E0de6960a8610002b471e99034ca39c3dad903ec7与GUI
        b038f6b97d6fc150efd2b0504e5bf13706d07162的实际合流输入；SDK仍使用原f7bac
        pin，EKO编译使用明确的保留能力framework。现有GUI差异逐路径保留，不能整文件回灌。
  - id: retire-sdk-consumption-and-regenerate
    summary: 在原framework pin上撤销SDK的A2A/LSP执行、接受与语言表面，生成精确的新合同。
    files:
      - echo-agent-sdk/echo-sdk-host/Cargo.toml
      - echo-agent-sdk/Cargo.lock
      - echo-agent-sdk/echo-sdk-host/src/features.rs
      - echo-agent-sdk/echo-sdk-host/src/core_profile/facade/
      - echo-agent-sdk/echo-sdk-protocol/src/facade.rs
      - echo-agent-sdk/echo-sdk-protocol/src/catalog.rs
      - echo-agent-sdk/echo-sdk-protocol/src/inventory.rs
      - echo-agent-sdk/echo-sdk-protocol/src/bin/export_schema.rs
      - echo-agent-sdk/echo-sdk-protocol/src/methods.rs
      - echo-agent-sdk/echo-sdk-protocol/src/capability.rs
      - echo-agent-sdk/contracts/sdk/
      - echo-agent-sdk/sdks/shared/
      - echo-agent-sdk/sdks/typescript/src/
      - echo-agent-sdk/sdks/python/src/echo_agent_sdk/
      - echo-agent-sdk/sdks/java/src/main/java/com/echoagent/sdk/
      - echo-agent-sdk/scripts/export-language-sdk-catalog.sh
    acceptance:
      - 移除framework-a2a/framework-lsp
        passthrough与sdk-facade-all成员、feature广告、A2A/LSP资源/stream/cleanup/adapter/handler；保留同文件MCP/topology及shared
        handle/bridge。FACADE_FAMILIES、METHOD_CATALOG、分类和source
        route生成一并撤销两家族/_echo_agent/a2a/op与_echo_agent/lsp/op及39操作，防止仅retire
        seed后被自动补回。
      - 删除三项插件身份的语言/值投影与实际消费：不再输出lsp_config_file或lsp_discovered，不接受LspServer
        variant，不以None/0/空值假保留；源Rust类型因旧framework
        pin仍存在只作为非阻断inventory事实，不恢复其accepted或可调用表面。原有其它plugin字段和合法组件保持。
      - 用仓库唯一export_schema完整--update并为269项逐一传入--retire-source，不使用--wire-candidate-update替代身份退役、不手改生成JSON；随后复用scripts/export-language-sdk-catalog.sh更新共享catalog/digests与既有语言投影。检查生成前后source-kind精确差集为269且加集空，route差集为原23且加集空，旧家族/方法/39操作均无产品残留；完整Rust
        inventory仍只按既有遥测政策报告。
      - TypeScript/Python/Java的A2A值/导出/JWT等仅按具名accepted集合移除，LSP通用Session
        invoke和catalog同步；共同模块只去退役成员。其余5352身份集合不增退，聚合shape/签名/digest的必要后继逐字段解释；Host
        pin及非本切片依赖不为整齐或重解锁改变。
  - id: remove-eko-runtime-and-plugin-dependencies
    summary: 让EKO初始化、插件生命周期和扩展控制脱离LSP，同时保留其它共享执行与清理权威。
    files:
      - echo-agent-cli/Cargo.toml
      - echo-agent-cli/echo-agent-app-core/Cargo.toml
      - echo-agent-cli/Cargo.lock
      - echo-agent-cli/echo-agent-app-core/src/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/plugin_components.rs
      - echo-agent-cli/echo-agent-app-core/src/plugin_runtime/
      - echo-agent-cli/echo-agent-app-core/src/extension_commands.rs
      - echo-agent-cli/src/cli/extension_surface.rs
      - echo-agent-cli/src/tui/commands.rs
      - echo-agent-cli/src/tui/events.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_pool/pool.rs
      - echo-agent-cli/echo-agent-app-core/src/config_watcher.rs
      - echo-agent-cli/echo-agent-app-core/src/config_discovery.rs
      - echo-agent-cli/echo-agent-app-core/src/extension_control/service.rs
      - echo-agent-cli/echo-agent-app-core/src/developer_commands.rs
    acceptance:
      - 移除manifest的LSP
        feature请求、runtime::register_lsp_tools及五个工具注册/服务器启动、PluginLspRuntime/binding/lsp_configs和prepare/reload/rebind/shutdown专用分支；按实际source
        paths检索所有调用点/配置投影，保留AgentPool、TaskRuntime、MCP、研究、IM、cron及恢复的共同初始化。
        AgentPool新Agent构造也去除register_lsp_tools/owner注入；config
        watcher不再触发reload_plugin_lsp，config discovery和developer/extension
        control不再宣告、路由或重载LSP，保留其它配置热重载与developer命令。
      - 旧用户配置和插件声明LSP时给明确退休诊断，不删除用户文件；含合法MCP/Skill/monitor与退役声明的混合输入只退出具名LSP表面，保留其它组件既有行为和可解释诊断/receipt，不返回整插件空成功。
      - plugin publication/rebind/rollback/shutdown继续由原generation与typed cleanup
        receipt结算；组件移除不释放别人的MCP token、不丢未结算原Agent/registry/scheduler
        owner；失败、取消、重试与恢复控制用原相关反例加保留能力正常控制证明。
        同文件既有inline/模块测试及自动重载/新Agent构造路径必须随消费者变化核对，不能只测primary启动。
      - EKO退役诊断仅消费上述框架通用冻结diagnostics中的组件、路径与严重程度，并映射产品退役文案；移除lsp_document及其它LSP专属类型/字段/方法依赖。声明在prepare后变化不得改写已准备代的诊断；混合插件保留合法组件，只有旧LSP声明时也不得假报语言组件已加载。不得把这条必要消费者切换延期给框架物理删除结果。
      - CLI/TUI/extension中仅移除LSP命令、类型和分支，仍保留独立入口与共享channel运行。runtime.rs、app_state.rs、manifests/lock、plugin_runtime由应用集成owner串行合流，不能与其它Outcome并行改同一生命周期owner。
  - id: close-eko-ipc-and-gui-surfaces
    summary: 由唯一GUI接缝owner撤销LSP IPC、菜单与类型投影，保留最新视觉和lazy行为。
    files:
      - echo-agent-cli/src/tauri/commands/lsp.rs
      - echo-agent-cli/src/tauri/mod.rs
      - echo-agent-cli/web-frontend/src/components/lsp/LspPanel.tsx
      - echo-agent-cli/web-frontend/src/components/layout/SettingsDialog.tsx
      - echo-agent-cli/web-frontend/src/components/layout/LeftSidebar.tsx
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/stores/uiStore.ts
      - echo-agent-cli/web-frontend/src/generated/
      - echo-agent-cli/src/tauri/commands/mod.rs
    acceptance:
      - 协调者唯一写Settings/导航、invoke注册和共享endpoints接缝；LSP面板、菜单键、IPC方法、事件和生成类型随真实后端一起退出，直接旧invoke/扩展请求有既有unsupported/method-not-found等明确失败，不能只隐藏菜单而继续宣告能力。
        同步移除commands/mod.rs的pub mod lsp，不留下悬空模块入口。
      - 以GUI
        b038贡献及届时受保护增量为基础保留token/Modal/状态组件与Sidebar/Settings视觉、lazy、错误边界和测试，不把旧版本整文件覆盖回来；切换设置页、空态和未知旧lsp键不会触发已删除loader或影响其它设置。
      - 按现有类型导出入口更新前端契约并核对跨crate
        import，不手工伪造空LSP响应；类型导出隔离于本次target外临时目录，只有已确认生成差异由唯一owner合流。
  - id: qualify-consumers-and-handoff-framework-retirement
    summary: 完成两消费者的反例、保留能力与正式生成验收，交付可供框架后置删除使用的精确证据。
    files:
      - echo-agent-sdk/echo-sdk-protocol/tests/
      - echo-agent-sdk/echo-sdk-host/tests/
      - echo-agent-sdk/sdks/typescript/test/
      - echo-agent-sdk/sdks/python/tests/
      - echo-agent-sdk/sdks/java/src/test/
      - echo-agent-sdk/sdks/typescript/examples/
      - echo-agent-sdk/sdks/python/examples/
      - echo-agent-sdk/sdks/java/src/main/java/com/echoagent/sdk/Example.java
      - echo-agent-sdk/docs/sdk/
      - echo-agent-sdk/README.md
      - echo-agent-sdk/README.zh.md
      - echo-agent-cli/echo-agent-app-core/src/plugin_runtime/tests.rs
      - echo-agent-cli/web-frontend/src/
      - echo-agent-cli/docs/en/
      - echo-agent-cli/docs/zh/
      - echo-website/docs-sync-manifest.json
      - echo-website/src/docs/content/
    acceptance:
      - SDK反例矩阵分别覆盖旧source digest/required a2a或lsp
        feature不能获得扩展准入、两旧family方法、39旧operation、绕过client的旧source
        invoke和三项旧插件值/字段失败；标准ACP模式可继续工作但不得把退役请求包装成功。新匹配合同、默认/core/剩余full-facade
        Host和所有保留家族有正常控制；保留其余capability/feature_surfaces。
      - R2父子close沿Host
        close_session_owned和三语session-parent-close/stdio-close原入口验证：成功父关闭收束本地子订阅，失败可重试，晚到start不复活，取消观察者不取消真实RPC，另一Session不受影响；不为退役两能力整包删除既有生命周期、流/取消/恢复测试。
      - 生成后按既有scripts/check-sdk-contracts.sh、scripts/export-language-sdk-catalog.sh
        --check、scripts/check-language-sdks.sh与各仓适用完整门禁完成验收；三语原examples/quickstarts真实执行且可执行链保留，EKO
        GUI条件矩阵与前端格式/lint/test/build覆盖实际改动。所有失败修复后同入口通过，不抬阈值、忽略错误或将旧结果冒充新快照。
      - SDK与EKO在不同repo可并行，但共用应用owner及GUI接缝串行；每任务独立target、jobs<=2、dev/test
        debug=0，全量/GUI/语言重矩阵同时只一套。批次前后记录df/du，低于5GiB不启新重任务；批次实际结束且无cargo/rustc/Host读取后，把Host/报告/原始退出先存target外再cargo
        clean，只清自有产物。
      - 正式SDK
        ADR/协议/语言README、EKO双语架构/ADR与examples按新消费者事实同步并解释不适用项；官网由协调者绑定已交付精确source/pin/manifest批次镜像，框架此阶段仍保留A2A/LSP的事实不能提前写成物理删除。两消费者分别冻结源码、锁、269键差集/生成结果、独审和适用本地完整运行资格后，可交框架删除Outcome进行隔离本地实现；本Plan的正式完成及框架删除main晋升仍须先完成两消费者适用精确main
        CI，保持消费者先main、框架后撤销的顺序。任何未授权push/merge/发布或网站部署不执行。
artifact_id: plan:b6ef1ac6-0745-438d-a3ff-75bb8cdbbcc5
lifecycle: ready
---
## Notes

以下沿用旧计划建立时的输入说明，仅解释接缝与授权沿革；执行依据以当前 YAML 与实际冻结交接为准。

本Plan的唯一结果是消费者解除依赖；对应 map 的前置工具事实清单已在顶层 `2fc9ad238b0549820e6d70e53c06a46e37a48279` 由协调者验收并合入协调分支。本轮仅创建计划，不实施上述生产变更，也不更新Design/map/旧Plan lifecycle。

输入复用 `audits/tool-inventory.md` 与 SDK readiness 聊天 `01a16cbf-c2ec-7bc2-8953-f9a4053ee965` 的只读结论。原266不是269的旧笔误：额外3个LSP插件accepted身份由协调者本轮明确具名授权退休，必须在SDK正式ADR与生成验收中保留这条范围证据。它们当前走共享value/plugin投影，仍保留的聚合类型不能通过伪空字段或LspServer空variant继续宣告它们；旧pin中的Rust定义与SDK产品表面是两条权威。

SDK生成接缝已核对 `echo-sdk-protocol/src/bin/export_schema.rs` 的full-update/ADR parser、`inventory.rs::apply_accepted_source_scope`、`facade.rs`、`catalog.rs` 与 `source_operations.rs::resolved_plugin_components_value`。只有退役accepted seed不足以阻止FACADE_FAMILIES重新补家族路由；不手改117项产物，也不把历史非阻断inventory漂移升格为本次接受或自动删除。

GUI贡献精确SHA为 `b038f6b97d6fc150efd2b0504e5bf13706d07162`。本轮只读时其集成树除Sidebar/Settings还有ErrorBoundary及新增SettingsDialog测试，均为其它任务的受保护增量；计划不采用“两文件之外均可覆盖”的假设。LSP菜单及共享invoke/endpoints只由协调者集成，实施者在开始写入时重新核对当前差异和归属。

两消费者可独立准备和验证，但本Outcome的完成必须同时满足两侧。框架物理撤销仍后置，本Plan补入的通用加法诊断先行，不改变SDK pin或接受范围。framework c3b的生产代码仍等于517264基线，SDK仍pin f7bac；待两消费者去依赖验收后，下一Outcome才物理删除framework生产能力。官网此阶段精确区分“消费者不再支持”与“框架已删除”，不把本计划或源码提交当发布回执。

实施发现EKO仍使用framework的`lsp_document()`专属方法读取冻结声明，原c3b没有通用document-presence入口。该依赖属于本Outcome必要收敛，故补入现有通用diagnostics的最小桥接，不另建公共API。先框架加法诊断可用，再EKO切换并验证，最后物理删除LSP；后置删除继续保留明确的旧输入诊断，不能保留旧执行能力，也不能使已迁移EKO重新依赖专属字段或方法。
