---
slug: eko-gui-product-convergence/gui-home-navigation
outcome:
  summary: GUI
    首页明确区分工作空间与任务，提供新建任务、最近任务和工作空间导航，输入区以左下工作空间/权限、右下模型的布局复用真实操作，并通过目标隔离和真实窗口视觉验收。
  acceptance:
    - 首页和侧栏以 EKO 品牌区分工作空间/工程与其普通任务会话：新建任务复用原 Conversation
      草稿且不新建目录；最近任务、工作空间与原运行中/后台/HITL/IM/cron/终端入口可发现。
    - 主输入左下工作空间/权限、右下模型及工作空间搜索/三来源创建菜单均调用既有真实入口；跨项目点击和所有晚到结果保留原目标，不影响新焦点、草稿或已运行任务。
    - 在浅/深色与1400×900、900×600下完成真实新源码浏览器和 Tauri
      的空态、会话态、菜单、窄屏与键盘/焦点审阅，分别保存截图/源码身份，不用浏览器 mock 代替原生验收。
    - 相关前端行为、类型、lint/格式/构建和独立代码/视觉审查通过；EKO 正式双语说明及官网/示例同步需求按归属对账。本结果不冒充
      Steer、个人微信或整个远程项目已交付。
out_of_scope:
  - 不改 Rust、manifest/lock、依赖、共享
    endpoints/生成类型、Workspace/Conversation/TaskRuntime store
    或其持久化、权限和运行时语义，不新增组件框架、任务系统或远程设备 registry。
  - 不实施 Steer 持久化确认/安全点业务、个人微信 adapter/扫码认证或远程 SSH/unknown
    结算；这些独立结果不阻塞本首页结果，也不借布局变化宣称已修复。
  - 不复制 WorkBuddy 品牌、吉祥物、无功能推荐卡或未被要求的办公/设计模式，不修改 AGENTS、Root 私有调度记录或历史视觉 Plan
    的任务身份。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 1. 文档定位与已确认目标
    digest: sha256:1fd53dab610685fe4efdd18d746769d34d71defe89f09020d4724cd28637703f
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 首页与工作空间导航
    digest: sha256:d3885ae131866409bf68ec9f0573af0f57bec76beab6a3af087ba073008c4198
  - ref: design.md § 统一视觉和信息层级
    digest: sha256:ae3294ad6b1fdac64513ac849bf6f49a48364a90ce2d945221959a39ff3b5862
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#gui-home-navigation
todos:
  - id: separate-task-and-workspace-navigation
    summary: 纠正新建任务、最近任务与工作空间导航，复用原会话/工作空间权威并固定跨项目目标。
    files:
      - echo-agent-cli/web-frontend/src/App.tsx
      - echo-agent-cli/web-frontend/src/components/layout/LeftSidebar.tsx
      - echo-agent-cli/web-frontend/src/stores/uiStore.ts
    acceptance:
      - 新建任务、命令面板与快捷入口调用既有 startNew 后进入聊天，保持 Workspace 和原合法全局会话，不打开 Workspace
        创建弹窗或取消旧后台执行。
      - 最近任务按有限 Workspace 集合只读聚合现有 conversationApi.list(workspaceId)，每行保留
        workspaceId；先 await switchTo 再 loadConversation，失败可见，跨项目/同名/同 ID 换
        generation 与迟到返回不串目标。
  - id: compose-clear-home-and-real-controls
    summary: 让首页和会话输入区形成清晰留白与稳定布局，重排原工作空间、权限、模型和发送控件。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx
      - echo-agent-cli/web-frontend/src/components/chat/WelcomeScreen.tsx
      - echo-agent-cli/web-frontend/src/components/chat/AgentComposerFrame.tsx
      - echo-agent-cli/web-frontend/src/components/chat/AgentPane.tsx
    acceptance:
      - EKO 首页使用浅灰侧栏、留白、宽大圆角输入和轻边框；主 composer
        左下工作空间/权限，右下模型，思考深度、附件、上下文与发送/停止仍可达，现有权限/模型 API 与配置作用域不变。
      - 共享 composer 通过显式主输入 variant 或局部 wrapper 应用首页样式，Subagent
        输入不放大；聊天保持原挂载/滚动/运行状态/HITL，布局变化不实现或掩盖已列 Steer 缺口。
  - id: reuse-three-source-workspace-menu
    summary: 在 composer 与侧栏提供可搜索、可键盘操作的工作空间选择和三来源创建入口。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx
      - echo-agent-cli/web-frontend/src/components/workspace/NewTaskDialog.tsx
      - echo-agent-cli/web-frontend/src/components/workspace/ProjectSourceFields.tsx
      - echo-agent-cli/web-frontend/src/stores/uiStore.ts
    acceptance:
      - 选择已有 Workspace 与新建工作空间、打开已有本地目录、远程 SSH 设备/目录入口复用原 store/API
        和唯一来源表单；明确三来源属于 Workspace 创建，项目类型与位置来源正交。
      - 菜单显示原真实项目身份和位置，远程位置不显示为本机产品数据
        root；搜索/选择、Escape、外部点击关闭和焦点恢复可用，窄屏模型/权限入口不被遮挡；目录选择/终端不增加 full-auto 门控。
  - id: qualify-navigation-visuals-and-document
    summary: 验证任务目标、菜单可访问性和真实窗口视觉，同步正式 EKO 说明并交独立审查。
    files:
      - echo-agent-cli/web-frontend/src/components/layout/
      - echo-agent-cli/web-frontend/src/components/chat/
      - echo-agent-cli/web-frontend/src/components/workspace/
      - echo-agent-cli/web-frontend/src/stores/uiStore.ts
      - echo-agent-cli/docs/en/architecture/overview.md
      - echo-agent-cli/docs/zh/architecture/overview.md
    acceptance:
      - 按实际改动验证新建任务不创建 Workspace、跨 Workspace 点击顺序/迟到 fence、主/Subagent composer
        隔离、菜单键盘与窄屏控件；保留原 ChatInput thinking publication、Workspace
        dialogs、Conversation/Workspace store 和主聊天常驻相关覆盖。
      - 浏览器与真实 Tauri
        以新源码分别验证首页/已有会话/运行中/后台/HITL/IM/cron入口、工作空间菜单和两主题/两尺寸，保存截图与实际限制；正式双语说明和官网/示例需求有明确归属，源码与视觉独审通过后按所属仓门禁交付。
schema_version: 4
artifact_id: plan:ddad4877-b86d-482c-aea0-8a2ba7214a4d
lifecycle: ready
---
## Notes

本计划是独立的首页导航结果，历史 GUI 视觉基础的 artifact、Outcome、Todo 和 lifecycle 均保留。原三来源 Workspace API 已固定，可在其上并行实施；远程项目整体验收或 SSH unknown 收尾不成为本前端结果的等待条件。ChatInput 与后续 Steer 实现的交集仍由协调者安排唯一写入者。
