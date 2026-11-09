---
schema_version: 4
slug: eko-gui-product-convergence/gui-visual-foundation
outcome:
  summary: 交付统一主题、排版、布局、导航、Modal 和状态呈现的 GUI 基础，并完成支持窗口与主题下的真实桌面视觉审阅。
  acceptance:
    - GUI使用现有语义token、theme/plugin-theme和common组件形成一致的字体、密度、表面、边框、遮罩、对比度与焦点样式；导航、主工作区、上下文与设置具有清晰层级，不新增主题store或第二套Modal行为。
    - 保留现有导航/会话挂载、context target、resize、插件主题覆盖及Modal
      portal/嵌套/焦点/键盘语义；样式调整不改变Task/HITL身份、执行状态或工具权限。
    - 在浅/深主题和1400×900、900×600窗口完成真实Tauri交互与截图审阅，覆盖导航、主工作区、上下文、设置及弹窗焦点；浏览器或jsdom证据单独标注，不能代替真实原生验收。验证使用明确隔离的开发数据和正确Tauri入口。
    - 完成相关前端行为/格式/类型/构建验证、必要原生验证与独立视觉/代码复审；正式EKO说明与需同步的官网/示例有明确处置。每个Cargo验证批次保存原件后清理准确自有target并记录磁盘，主线交付仍遵循适用门禁。
out_of_scope:
  - 不实现IM/cron新管理接线、独立入口或A2A/LSP物理退役、HITL/任务列表业务重构、SSH或工具/插件执行语义。
  - 不更换组件框架、主题或任务状态权威；不把浏览器mock加入产品流程，不使用用户真实会话/渠道/调度数据冒充验证fixture。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 1. 文档定位与已确认目标
    digest: sha256:1fd53dab610685fe4efdd18d746769d34d71defe89f09020d4724cd28637703f
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 统一视觉和信息层级
    digest: sha256:ae3294ad6b1fdac64513ac849bf6f49a48364a90ce2d945221959a39ff3b5862
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#gui-visual-foundation
todos:
  - id: unify-existing-visual-primitives
    summary: 归一现有主题token、控件密度与common表面/弹窗的视觉配方。
    files:
      - echo-agent-cli/web-frontend/src/index.css
      - echo-agent-cli/web-frontend/src/components/common/
    acceptance:
      - 浅/深色和插件覆盖继续消费同一套变量，文字/图形与颜色共同表达状态；复用Card/Modal/StatusBadge/Toggle，保留全部已存在的可访问性与嵌套行为，修复所触及的无效样式。
  - id: align-workbench-and-navigation
    summary: 把视觉配方接入主布局、导航和上下文，保持现有布局与状态行为一致。
    files:
      - echo-agent-cli/web-frontend/src/components/layout/AppLayout.tsx
      - echo-agent-cli/web-frontend/src/components/layout/ContextPane.tsx
      - echo-agent-cli/web-frontend/src/components/layout/LeftSidebar.tsx
      - echo-agent-cli/web-frontend/src/components/layout/SettingsDialog.tsx
      - echo-agent-cli/web-frontend/src/components/chat/AgentPane.tsx
      - echo-agent-cli/web-frontend/src/components/providers/ProviderPanel.tsx
      - echo-agent-cli/web-frontend/src/stores/contextPaneStore.ts
      - echo-agent-cli/web-frontend/.env.tauri
    acceptance:
      - 缩小窗口时主区、右栏和overlay不遮挡关键操作，导航字号/间距/状态清晰；若改264px左栏或上下文尺寸则成对更新计算和消费；ChatPanel切换工作台仍保持挂载。
      - 仅在现有Tauri检测入口上修正已确认的环境变量名称漂移，不引入新的宿主探测/状态层；SettingsDialog和LeftSidebar接缝由唯一写入者集成。
      - 原生审阅中发现的Provider添加按钮沿用现有text-on-accent变量修正深色文字对比度，保持原操作和面板行为。
  - id: verify-native-visual-result
    summary: 完成真实桌面视觉场景、回归与正式说明同步，形成可审阅交付。
    files:
      - echo-agent-cli/web-frontend/src/components/common/
      - echo-agent-cli/web-frontend/src/components/layout/
      - echo-agent-cli/web-frontend/src/stores/contextPaneStore.test.ts
      - echo-agent-cli/docs/en/architecture/overview.md
      - echo-agent-cli/docs/zh/architecture/overview.md
    acceptance:
      - 保存改前/改后的真实窗口截图、主题/尺寸/场景和源码身份，逐项确认布局、键盘焦点、设置与context交互；受限平台或缺少原生截图时明确未完成项。
      - 执行与实际改动匹配的既有回归和构建，零未处理失败；原生进程停止且日志/必要制品保存到target外后及时cargo
        clean。文档、examples与网站同步需求逐项说明，独审通过后再进入最终门禁和按仓交付。
artifact_id: plan:89342a2a-833b-4f72-acfa-63d2a650e331
lifecycle: ready
---
## Notes

在本机已审合同提交上实施：顶层 e0e8fdde15a25a84dc6b8fa57ccf442da87c8c25、framework c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa、EKO 0de6960a8610002b471e99034ca39c3dad903ec7、SDK 961da0e0d17b815653d80cb4bf4973248f543770、官网 a157575354baea1648894de4b6ed9f0dd836f9ad。合同内容已经交付协调者并通过独立审查，main/CI 交付仍须按原门禁收口，不把本地贡献误写为主线完成。

视觉组独占 index.css、common、AppLayout/ContextPane、AgentPane 的视觉修改和必要尺寸计算；不触及 chatStore/useTauriChat、TaskRuntime store或终端生命周期。LeftSidebar、SettingsDialog是导航/后续退役接缝，由协调者在视觉组冻结后独占集成，禁止双writer。终端调色消费留给SSH结果，不提前改终端行为。

现有可执行证据为Modal、PrimaryWorkspace、ContextPane、uiStore与WorkspaceDialogs可访问性测试；未发现已纳入版本管理的浏览器视觉harness。可以使用独立验证夹具与临时驱动，但真实Tauri验收仍必须单独完成。
