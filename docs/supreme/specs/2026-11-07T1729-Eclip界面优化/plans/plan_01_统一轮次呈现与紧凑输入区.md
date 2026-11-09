---
schema_version: 4
slug: 2026-11-07T1729-Eclip界面优化/plan_01
outcome:
  summary: Eclip 在实时与冷历史中以可信轮次统一呈现正文和可折叠过程，提供紧凑输入区及来源明确、可恢复的运行统计。
  acceptance:
    - 同一原journal的实时、重开及分页视图得到相同正文/过程顺序；两段正文之间的连续过程只有一行摘要，最终发布后整轮过程可反复展开，最终答案独立可见。
    - 保存不确定、失败、HITL、缺详情、Fork来源及操作资格全部保持；候选或EOF不触发最终折叠；统计不重复、不假全量、速度缺证据显示--。
    - 最终冻结前端执行 npx prettier --check "src/**/*.{ts,tsx}"、npm test、npm run build
      全部自然退出0；涉及的Rust公开事件/历史合同及examples按所属AGENTS完成验证，失败不得跳过。
    - 同一准确源码和资产构建的本地App经用户明确安排的真实点击、流式更新和cold reopen验收；组件通过、App通过、main CI与发布分别记录。
out_of_scope:
  - 不自动采用上轮未验证候选；不在未经独立授权时操作App、提交、推送或发布。候选须先按本设计重新核对。
  - 不重写canonical历史、TaskRun/Goal/最终发布状态，不新增消息或统计权威store、SQLite和协作管理控制台。
  - 不以UI计时、字符数或工具prepared时间推导生成速度、实际进程开始或执行完成；不自动升级SDK pin。
design_ref: docs/supreme/specs/2026-11-07T1729-Eclip界面优化/design.md
design_sections:
  - ref: design.md § 审查结论与证据
    digest: sha256:25eef7d965cd1b99297065368b78a32c86d7fc65143843db9a650deb8bf8ccb2
  - ref: design.md § 目标与范围
    digest: sha256:277a53c21a8be8e37e61327d83d21e27191da4c506cbc3ea2ee7b2c3aec0b001
  - ref: design.md § 分层与复用
    digest: sha256:c41b1786f22df3715e5a66b0b18b2018dc912481609f96e6b658402517af9c73
  - ref: design.md § 目标行为
    digest: sha256:fefb99a4907607fcdac723d435f992e17319f66ef03728d8c3703e661166b86f
  - ref: design.md § 核心数据流与合同
    digest: sha256:1c923c4226160d3a3b75ae144e8951d70eef4e09d9d754f5e35db69697a277e8
  - ref: design.md § 参考实现与取舍
    digest: sha256:530ca2851d8286402c32f40155821ace590ce8bf99924f7df9a250e9915e13cc
  - ref: design.md § 验收标准
    digest: sha256:f60f94c24b100126d2549b63d9bd124f3c5bc3803160922cc4383b5c1278ab4e
  - ref: design.md § 协作、文档与交付边界
    digest: sha256:c37e7cec3985176cc62b7c9d31be5321deb09f9d69abd7a0801b4368ea167fa5
delivery_ref: null
todos:
  - id: restore-trusted-turn-identity
    summary: 沿原已核验publication为实时与历史补齐可信轮次坐标，并处理上轮候选与正式目标的差异。
    files:
      - echo-agent-cli/echo-agent-app-core/src/conversation_projection.rs
      - echo-agent-cli/src/tauri/commands/conversations.rs
      - echo-agent-cli/web-frontend/src/types/api.ts
      - echo-agent-cli/web-frontend/src/stores/conversationStore.ts
      - echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
    acceptance:
      - 先核对应用 .echo-semantic/local/chat-process-folding-20261107-r01/start.json
        与当前5个候选文件字节、staged/unstaged增量；仅由对应writer保留、修订或撤回本会话候选，不覆盖其它dirty。
      - 使用已有history/publication与消息测试复现同一轮冷恢复分裂；只投影原workspace/conversation/root_turn/turn/execution，缺失或冲突保持未关联。
      - NativeHistoryImport的源坐标与目标容器分离；相同文本、同名block和相邻时间不产生归属，Fork/编辑/重新生成仍沿原后端资格。
  - id: measure-and-restore-usage
    summary: 使运行统计有真实请求计时和可重建累计，冷历史或回放缺口不冒充全量。
    files:
      - echo-agent/echo-core/src/agent/mod.rs
      - echo-agent/src/agent/react/run/phases/think.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/
      - echo-agent-cli/src/tauri/commands/conversations.rs
      - echo-agent-cli/web-frontend/src/types/api.ts
      - echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts
      - echo-agent-cli/web-frontend/src/stores/chatStore.ts
    acceptance:
      - 在原模型消费循环取得首/末有效输出和同口径usage，新增可选观测沿原envelope无损传递；计数覆盖不明、单包、无usage或非正时长不计算tok/s，请求总耗时不冒充decode。
      - 主行只计目标会话实际主链请求；重试尝试费用保留、逻辑步骤不重复；Fork继承源usage不加入目标账目，同会话编辑/回退不冲销已发生费用。用带精确输入/输出/命中数的重放夹具分别核算，Subagent/辅助调用明细单列。
      - 复用原journal
        identity/through-sequence检查点与后缀恢复累计，覆盖超过4096事件、重复回放、失败已收费请求、压缩、切换会话和Fork；无可信前缀显示不完整，不能新建统计store。
      - 盘点新增公共事件字段的框架构造/匹配/序列化消费者，保持provider原厂ID和SDK独立pin，不让缺字段变成零值证明。
      - 用100输出/1000ms和400输出/2000ms两次合格尝试验证主行最近值200tok/s、明细加权500/3tok/s；新请求进行中标上次。另覆盖重试、单包和不匹配口径，不平均100与200的速率。
  - id: render-one-turn-timeline
    summary: 用一套只读轮次投影统一正文和单行过程，并按原最终发布事实整体折叠。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx
      - echo-agent-cli/web-frontend/src/components/chat/NativeMessageTimeline.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ExecutionProcessGroup.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ToolExecutionGroup.tsx
      - echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.tsx
    acceptance:
      - 20个连续thinking/tool条目夹在两段正文之间时只有一个摘要行；展开完整保留原序和公开summary，签名/加密/redacted原件不展示。
      - 原轮次身份稳定；流式追加和状态变化不重挂载。冷页从同一过程段中间开始再补前缀时，用partial段别名迁移最近显式展开选择、焦点和原事件滚动锚点，不因段首变化丢失交互状态。
      - 合格最终发布后进度正文与过程整体默认收起；最终答案、错误/保存警告/HITL和产物保留可见；裸候选、complete、EOF、失败取消负控不误触发。
      - 既有多个Reasoning折叠测试按新合同改为一个入口且完整原序展开，不删内容断言；缺summary/transcript的工具有原引用占位。删除被替代的平行分组。
  - id: compact-composer-and-statistics
    summary: 把选择控件与发送停止收进输入框，在外侧用一行显示可解释统计。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx
      - echo-agent-cli/web-frontend/src/components/chat/AgentComposerFrame.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ExecutionStatus.tsx
      - echo-agent-cli/web-frontend/src/stores/chatStore.ts
    acceptance:
      - 同时调整frame/textarea最小高度、padding、框内工具行和框外统计，去除重复停止入口；菜单不被裁切，草稿、附件、队列、幂等提交和输入回执保持。
      - 360px与桌面宽度验证无横向溢出；长模型/workspace可截断且有完整可访问名称，窄屏保留权限/停止，键盘、触摸可操作。
      - 主行和明细严格按设计口径显示轮次/步骤、速度、累计用量、缓存、当前上下文；低缓存率不隐藏，prepared/accepted时间不称进程运行时间。
  - id: validate-and-document-ui
    summary: 在同一冻结快照证明实时、历史、异常和布局一致，并同步所属说明与验收证据。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/
      - echo-agent-cli/web-frontend/src/hooks/chatEventHandler.test.ts
      - echo-agent-cli/web-frontend/src/stores/
      - echo-agent-cli/echo-agent-app-core/src/conversation_projection.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/tests.rs
      - echo-agent-cli/docs/zh/guides/gui-task-runtime-panel.md
      - echo-agent-cli/docs/zh/adr/0057-native-messages-and-progress-policy.md
      - echo-agent/docs/adr/
      - echo-agent/echo-agent-learning/examples/
      - echo-website/
    acceptance:
      - 现有component/store/history测试覆盖完整目标矩阵，Rust
        DTO/事件round-trip与真实原journal冷恢复分别保留结果；新增计时合同记录所属仓ADR。
      - 检查examples与官网：无新UI模型工具无需新增示例调用；公共事件构造变更时现有示例纳入编译，官网只同步真实可交付行为，不把未验收预览当成已上线。
      - 原生验收先安排独立资源和用户操作窗口，记录准确App/资产/源码绑定；无原生证据保留待验，不自动启动或重开原数据App。
artifact_id: plan:67249d90-c9a2-4352-a5fc-a96fb91c243c
lifecycle: ready
---
