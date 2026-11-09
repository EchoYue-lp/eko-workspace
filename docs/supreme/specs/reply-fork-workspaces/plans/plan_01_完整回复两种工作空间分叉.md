---
schema_version: 4
slug: reply-fork-workspaces/complete-reply-fork
outcome:
  summary: 每轮完整回复末尾提供当前工作空间和从源HEAD创建的新Worktree两种分叉，复制同一可恢复历史并打开准确目标的新空闲会话，保留原会话、未提交代码及真实部分成功资源。
  acceptance:
    - 一轮只在最终完整回复末尾出现一个可访问的分叉菜单；两模式复制同一canonical前缀，原会话和SideConversation历史不变，原半轮/未结算准入保持。
    - 当前工作空间继续共用原目录；新Worktree使用固定源commit
      OID、不复制dirty/untracked/ignored文件，沿原Workspace绑定本地或已认证SSH同一目标，文件/命令/Git/终端/产物与目标一致。
    - 目标新conversation空闲，不克隆旧Goal/TaskRun/Subagent/control
      owner或源conversation专属model override；必要附件/长原文/工具记录引用在目标实际读取，关闭来源后仍可读。
    - 双击/重试/迟到/源切换和跨资源部分失败不重复创建、不抢新焦点、不删除用户数据；实际已创建资源可从原Worktree/Workspace/managed-import入口恢复。
    - 中文ADR/说明、合同/适用示例与官网来源同步，分开记录focused/GUI/真实Tauri/本地与SSH/完整门禁和mainCI，不用局部成功替代整体分叉验收。
out_of_scope:
  - 不复制源未提交/未跟踪/ignored代码，不替用户commit/stash/push或改源分支；不克隆活动Goal、任务图、Subagent、控制owner或源conversation
    model override。
  - 不新增任务CRUD/store/executor、全局Saga状态机、SideConversation专用IM协议或第三套Git
    Worktree算法；不扩大artifact reader根，不读取或持久保存密码/私钥。
  - 不修改独立Framework/SDK公共API、zstd456/SDK94候选、锁/风险合同、其它并行运行时切片、外部校验工具或保护refs；不执行push/merge/关闭Issue/发布/部署。
design_ref: docs/supreme/specs/reply-fork-workspaces/design.md
design_sections:
  - ref: design.md § 目标行为与范围
    digest: sha256:ba9af20f1a89c59240df852501c87b699329e95eff8e02af24f2c1970aaac87a
  - ref: design.md § 分层与复用依据
    digest: sha256:c6e5ce8374e4a98f6083ce29b3b3a4ccce1f5171d2c723b920e7e2f0e53def75
  - ref: design.md § 同一来源快照与新会话
    digest: sha256:fb73bc7cf2621fd7bf0c7421b2bdf64ebe13281843fd85ce00c88e52d7c090f8
  - ref: design.md § 项目、SSH 与引用绑定
    digest: sha256:99fc58f01105eac82289f8d76b36953eddd2e91c5275109731ffc8307f9b09ec
  - ref: design.md § 重复、失败与恢复
    digest: sha256:522a902a5324cf05fcac7f176d736061faae283c9e5db48eb6abeb9533410a83
  - ref: design.md § 验收标准
    digest: sha256:230f922b66c3ea05767cb35d0d2ab776390be44bced0a505dcfbd139cb6429d1
  - ref: design.md § 官方参考与取舍
    digest: sha256:693f05822a840fe7fccd012122398d16a04291dae4d0623a37599f8eae32c25e
delivery_ref: null
todos:
  - id: complete-reply-fork-entry
    summary: 将两种目的地接入每轮唯一完整回复入口，并保持准确请求与导航身份。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx
      - echo-agent-cli/web-frontend/src/components/chat/MessageBubble.fork.test.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx
      - echo-agent-cli/web-frontend/src/stores/conversationStore.ts
      - echo-agent-cli/web-frontend/src/stores/conversationStore.test.ts
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/types/api.ts
    acceptance:
      - 多assistant/工具/内部消息的同轮只有最后完整回复可分叉，streaming/半轮/未结算及独立tool记录不得拥有误入口；键盘/焦点/浅深/窄窗沿现组件，原编辑/重生和SideConversation历史保留。
      - 菜单明确两目的地和HEAD-only代码边界；一次提交固定准确source
        workspace/generation/conversation/前缀/目的地，重复点击复用请求，source切换或迟到结果不覆盖当前选择。
  - id: compose-existing-project-fork-owner
    summary: 由应用薄协调把同一canonical前缀导入原或新Worktree目标工作空间，不复制运行控制。
    files:
      - echo-agent-cli/src/tauri/commands/conversations.rs
      - echo-agent-cli/src/tauri/commands/panels.rs
      - echo-agent-cli/src/tauri/commands/workspace.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs
      - echo-agent-cli/echo-agent-app-core/src/workspace/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs
      - echo-agent-cli/echo-agent-app-core/src/managed_conversation.rs
      - echo-agent-cli/echo-agent-app-core/src/terminal/project_ssh.rs
    acceptance:
      - 后端保留真实source managed
        admission、内部过滤及canonical最终完整回合校验，两种形式共用原前缀/import，不只靠UI
        ordinal或executionCompleted。
      - 原create_worktree与新fork共用现项目Git/FileSystem/锁实现；如需提取，至少切换真实原入口并删被替代重复逻辑，不能另写第三套Git创建/清理。
      - 新Worktree固定源commit后用原registry/runtime登记目标，新conversation经managed
        import/CAS保持空闲；不会修改源整个workspace的project_location，不复制Goal/TaskRun/Subagent/control
        owner或源专属model override。
      - 本地真实夹具证明HEAD-only且原dirty/untracked/ignored未改；detached正控制、非git/unborn负控制与SSH同target/连接代际/断线不本地fallback分别验证。
      - SSH来源连接只执行源Git且不转移owner；目标先由原GUI/create_project_ssh_auth形成独立draft连接/PTY，仅该draft交原创建流程reassign。不得来源已绑定connection_id或同Arc双登记；双方并存、任一关闭另一方存活、目标创建失败不伤来源及错代际/错目标均有控制，人工认证资格独立记录。
  - id: retain-readable-history-and-partial-resources
    summary: 保全目标可读取的历史资源并使跨资源失败和重试沿原owner恢复。
    files:
      - echo-agent-cli/echo-agent-app-core/src/attachments.rs
      - echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs
      - echo-agent-cli/echo-agent-app-core/src/managed_conversation.rs
      - echo-agent-cli/echo-agent-app-core/src/workspace/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/src/tauri/commands/conversations.rs
      - echo-agent-cli/web-frontend/src/stores/conversationStore.ts
      - echo-agent-cli/web-frontend/src/stores/conversationStore.test.ts
    acceptance:
      - 目标内联/文件附件和可核归属的长原文/工具输出引用逐bytes/大小/SHA实际回读；只转换已知产品资源描述符，不字符串改写用户任意路径、不复制整个源产品数据或放宽目标reader根。来源关闭/独立清理后目标副本继续可读。
      - 旧UI execution/message引用不冒目标活动执行；canonical tool
        call/result保留，缺准确状态证据保持Unknown，不借源live owner/伪造新执行成功。
      - Git
        add成功后list失败、登记/迁移/import/CAS/切换失败、回复丢失/取消/重开均保真实已创建资源/错误，通过既有列表/回执/恢复入口继续；未知执行状态先重读，不重复创建或force删除dirty目录。
  - id: qualify-product-contract-and-native-target
    summary: 同步所属中文产品合同并取得真实目标、新版页面与适用工程资格。
    files:
      - echo-agent-cli/docs/zh/adr/0045-managed-conversation-lifecycle.md
      - echo-agent-cli/docs/zh/getting-started.md
      - echo-agent-cli/docs/zh/features.md
      - echo-agent-cli/examples
      - echo-website/src/docs/content/eko/zh/getting-started.md
      - echo-website/docs-sync-manifest.json
    acceptance:
      - CLI中文ADR/说明记录两模式、空闲新会话、HEAD-only与部分成功恢复；生成合同按真实变化同步，原例子纳适用编译链，不适用项写明原因；官网按已审可恢复精确源同步，不造英文副本或部署。
      - 源码稳定后统一前端/GUI/Tauri制品指纹与恢复材料，真实新窗口验证每轮入口/两目的地/焦点与目标I/O；SSH使用用户GUI认证并分别记录账号/路径/范围，不把mock或组件截图当原生/远端验收。
      - 所有验证针对冻结输入、记录真实选中/失败/忽略/出口；使用主根内fixture和独立target，按当前5focused/jobs≤5/debug0/heavy1及批后保全→无reader→准确clean规则执行，root唯一集成/提交，本Goal/mainCI/发布资格分别结算。
artifact_id: plan:2cd8e53e-602d-4e01-bc28-0e2350905b8a
lifecycle: ready
---
## Notes

本Plan仅交付同一完整回复分叉结果，两种目的地不能被拆成只完成一种后声称该结果完成。用户已明确HEAD-only；继承沿原普通fork的新空闲conversation装配，不再提出复制未提交代码或旧运行owner的新选项。当前正式 source事实是原branch同workspace、原create_worktree仅建目录，跨目标协调与资源引用仍待实现。Todo文件是相关路径而非并行写权；实施由root按当前source owner重新派发，尤其conversations.rs/ConversationStore/MessageBubble/prepared_turn可能与历史工具和原文恢复切片交集，必须在同一冻结窗口整合。没有运行Cargo、原生或真实SSH，不据本Plan ready状态宣称功能完成。
