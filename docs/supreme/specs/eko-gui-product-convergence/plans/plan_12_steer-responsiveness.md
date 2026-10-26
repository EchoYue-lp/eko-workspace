---
schema_version: 4
slug: eko-gui-product-convergence/steer-responsiveness
outcome:
  summary: 运行中追加输入复用原持久化入口与 tracked
    回执，及时确认且不重复提交、不串会话，并在未启动工具的安全边界优先考虑纠正，保留已启动操作结算；同时把 TaskRun 的 Steer/Goal
    控制收敛到 composer 上方：Steer 支持多条持久化输入的编辑、拖拽排序、删除和安全点立即执行，Goal 在同一 TaskRun
    内保持唯一，修改按 revision/CAS 立即刷新权威投影并使旧计划/证据失效，删除以结束当前目标运行实现。
  acceptance:
    - 运行中的模型或物理工具尚未结束时，原输入 Persisted 回执即可让 GUI 显示已收到并清除对应未被替换的草稿；重复
      Enter、并发提交和同次重试只产生一份持久输入，持久化前失败不丢文本或附件。
    - 后续等待、进入上下文、明确拒绝或未知由原 tracked 回执呈现；切换 Workspace/Conversation、同 ID 重开、开始新
      turn 或编辑新草稿后，迟到结果不清新稿、不改新会话，原输入事实和观察仍归原 owner。
    - 已接受的纠正输入在原工具批及尚未启动下一波工具的安全边界优先进入原上下文并让模型重新考虑工作；已发布 ToolCall
      均有匹配结果或真实未执行事实，已启动调用保留实际结果、资源和物理结算，Stop 仍走原取消入口。
    - composer 上方多条 Steer 的编辑、拖拽排序、删除和立即执行均使用原持久输入身份及 input/queue
      revision；冲突、渠道归属保护、失败草稿、迟到回执和重启恢复有反例与正常控制，不用删除重提或前端临时数组代替权威更新。
    - 同一 TaskRun 的唯一 Goal 修改经过原暂停/安全点和 expected_goal_revision CAS，成功立即刷新
      canonical projection 并使旧
      Plan/GoalRequirement/受影响证据失效；删除结束当前目标运行，拒绝或未结算时保留真实状态，不写空 Goal、不伪报物理停止。
    - 真实调用链的反例、正常控制、前端行为/类型/格式、框架定向验证、独立审查及真实 Tauri
      运行中追加输入验收通过；正式文档、示例与官网同步按所属仓对账，精确快照和模拟/原生范围分列。
out_of_scope:
  - 不重复实现 S1 已完成的 view identity fence，不把它的 41 项 Hook 测试或本计划落盘当作 S2/S3、合流
    GUI、真实服务或 main 的验收。
  - 不重做 Plan11 首页导航或工作空间布局，不扩个人微信、IM/cron、SSH、领域插件和入口退役；ChatInput/ChatPanel
    的真实交集按唯一作者串行接线。
  - 不新增 queue/store/dispatcher、第二套 TaskRun/Goal 状态机或审批流程；复用已有 Goal/Plan revision
    与原取消/安全点，不以清稿提前伪造 MailboxAccepted/Drained；不广播全部 Subagent，不把 Steer 当作 HITL
    回答或停止。
  - 不抢占已启动模型或物理工具，不丢 future、伪造结果或自动重放未知副作用；不修外部 echo-semantic
    工具，不新增依赖/权限门控/SQLite，也不执行发布、部署或未授权 main/push。
  - 完整有效要求的独立分母、长期约束与替换投影、压缩恢复和缺失关键上下文门禁由 effective-requirements 独立结果承接；本结果不能凭
    Goal CAS 或输入控件的定向通过关闭那组义务。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 运行中的追加输入（Steer）
    digest: sha256:3c805cc87fb2cb74ed14e0e3cf44cd8d64c51e8349d378aaaf3ffc0801a5c590
  - ref: design.md § 唯一 Goal 与当前有效要求
    digest: sha256:2d810ec9aca486a6a1ef7d20ceb15490a9dfa04ef58be81941fa955cfe83ad12
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:6f59f80f3c5fe48e08bac490651377bf47f90e5dbde6560e47c4e5d16d9fcde5
  - ref: design.md § 12. 复用结论与完成标准
    digest: sha256:1e7084072aa642175303e9e30b436fb4f9b2aa0e460d654bb4e74abb3ad51815
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#steer-responsiveness
todos:
  - id: acknowledge-one-durable-draft
    summary: 让一次草稿提交在原持久回执后及时确认，防止重复提交并保护后续草稿。
    files:
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.test.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatInput.thinking.test.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
    acceptance:
      - 复用 onSend → sendMessage → queue_chat_input →
        ConversationInputService.submit 的真实路径，通过既有调用的薄返回/回调传递原 Persisted
        回执及对应提交身份；确认和清稿不再等待 refreshFrontier/steer drain，已收到不等于已执行纠正。
      - 一次草稿的附件准备、入队中和同次重试固定原 externalId/input_id、目标和负载身份，重复
        Enter/点击不创建新身份；响应丢失后的同身份重试沿原去重/冲突合同，不重放不确定的执行，不新增第二队列或持久提交 store。
      - 只清除本次提交仍匹配的文本、附件和预览资源；持久化前失败保留输入，编辑后新稿、跨会话/工作空间/代际和卸载后的迟到确认不得被清除，原已持久输入仍可在原
        frontier 恢复。
      - 原本地 slash command、非运行中普通发送、附件失败、模型/思考配置及 S1 晚回 fence 行为保留；原 boolean
        不能单独冒充持久回执，后续 tracked 观察继续由原后端 owner 结算且无未处理拒绝。
  - id: manage-durable-steer-at-composer
    summary: 在 composer 上方按原输入和队列 CAS 管理多条 Steer，并如实呈现安全点立即执行结果。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.test.tsx
      - echo-agent-cli/src/tauri/commands/chat.rs
      - echo-agent-cli/echo-agent-app-core/src/conversation_input.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/journal.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/projection.rs
    acceptance:
      - 编辑通过已有 replace_queued_chat_input 保持 input_id、地址及位置，以 input revision 和
        queue revision 原子替换；并发编辑/排序/消费先赢后拒，重开后原文和顺序仍可恢复，不删后重提。
      - 排序和删除复用原 reorder/cancel 与队列快照；拖拽及可键盘操作入口绑定原目标。channel-bound 输入不能丢
        account、generation、context 和恢复意图；不能无损保持时明确拒绝并保留它的原 owner。
      - 立即执行复用原 steer 入口及 tracked receipt；未到安全点呈现 retained，拒绝呈现
        rejected，真正入上下文才呈现 drained。编辑失败保留输入和理由，不能以 await resolve、移出列表或 toast
        代替成功事实。
      - 切换 Workspace/Conversation、同 ID 换
        generation、编辑新稿及进程恢复不串输入；保留附件、中文/emoji和旧 retained/rejected 回执清理的直接反例。
  - id: edit-or-end-one-canonical-goal
    summary: 在 composer 上方修改或结束唯一 Goal，使安全点、版本失效与失败反馈沿原 TaskRuntime 闭合。
    files:
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.test.tsx
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts
      - echo-agent-cli/src/tauri/commands/task_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/completion_gate.rs
    acceptance:
      - 修改使用已有 update_task_run_goal/update_run_goal、expected_goal_revision 和
        canonical 投影；运行中等待原暂停/安全点，CAS 冲突与持久化失败明确返回，保留编辑草稿，不用调用返回布尔值冒成功。
      - 新 revision 成功后立即刷新同一 run，并使旧 Plan、GoalRequirement 与受影响证据失效；旧
        Subagent/轮询晚回不能把旧版本恢复成当前完成，未受影响证据须核验后复用。
      - 删除沿原取消/结束当前运行实现；拒绝、等待结算和未知均保留原目标与真实操作状态，不写空 Goal、不物理抹除
        journal，不留下可继续执行的无目标运行。
      - 修改和结束覆盖跨 Workspace/run/generation 迟到、重复点击、并发 CAS、失败重试、运行/暂停/终态与重启恢复，既有
        Stop/HITL/渠道 owner 保留。
  - id: consider-steer-before-unstarted-tools
    summary: 在唯一 ReAct 循环及原工具波次的启动安全点优先考虑已接收纠正。
    files:
      - echo-agent/src/agent/react/run/stream_channel.rs
      - echo-agent/src/agent/react/run/phases/tools.rs
      - echo-agent/src/agent/react/run/phases/think.rs
      - echo-agent/src/agent/steer.rs
    acceptance:
      - 扩展原 drain_steer_into_context、tracked mailbox 和
        build_execution_waves/run_tools，精确绑定 turn 与 incarnation；只有输入真正加入原上下文后才标记
        Drained，不另造执行循环、调度器或 EKO 策略层。
      - 用受控模型返回屏障证明：纠正在原启动边界前已 Accepted 时，尚未启动的写入/删除或后续波次先交原模型重新考虑；顺序与并行模式均覆盖，无
        Steer 的正常工具路径保持原行为。
      - 未发布的模型草稿和已发布 ToolCall 分别处理；所有已发布 call ID
        有且仅有完整匹配结果或真实未执行/跳过事实，不伪造工具输出、丢配对、重复副作用或用空结果冒充运行。
      - 工具或并行波次已启动后收到 Steer，保留该批实际执行、结果、流/reader 和资源结算，下一未启动波次再经过原安全点；显式
        Stop、超时、失败和调用方离开仍沿原物理取消/未知结算，不因追加输入提前终态。
  - id: qualify-feedback-and-execution-chain
    summary: 验证持久接收、上下文消费与执行边界形成一条可信反馈链，并完成所属仓文档和交付对账。
    files:
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.test.tsx
      - echo-agent-cli/web-frontend/src/components/chat/
      - echo-agent-cli/src/tauri/commands/chat.rs
      - echo-agent-cli/echo-agent-app-core/src/conversation_input.rs
      - echo-agent/src/agent/react/run/stream_channel.rs
      - echo-agent/src/agent/react/run/phases/tools.rs
      - echo-agent/docs/adr/0003-tracked-steer-lifecycle.md
      - echo-agent/docs/en/10-streaming.md
      - echo-agent/docs/zh/10-streaming.md
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
    acceptance:
      - 在同一原 identity/queue_revision/attempt → tracked Accepted/Drained
        链上验证延迟模型/工具、双 Enter、附件准备中重复提交、持久化前失败、回执丢失重试、新稿与跨目标迟到结果；同时有未追加输入的正常控制，不用
        mock 自己新增的队列证明正确。
      - 工具批前/波次间屏障、已启动副作用、完整 call/result 配对与 Stop 交错使用真实原循环和执行 owner
        验证；保留流式/非流式共用循环、既有 tracked receipt/同 ID incarnation/HITL
        等直接相关覆盖，所有真实失败修复后再结算。
      - 真实 Tauri 场景分别观察已收到、等待、进入上下文与明确失败/未知，并操作多条编辑/排序/删除/立即执行、唯一 Goal
        修改/结束；运行中仍可编辑新稿和使用独立 Stop；保存新源码及截图/事件/回执依据，明确受控模型与真实服务范围，不用 Hook
        或浏览器模拟代替原生体验。
      - 在所属子仓更新生命周期/流式和 EKO 正式双语说明，架构行为沿既有 ADR
        记录；核对可执行示例和官网镜像是否受影响并明确同步归属，独立源码/行为审查及所属仓适用合流门禁完成，既有未授权外部发布和宽治理状态不借本结果关闭。
artifact_id: plan:a7405e25-3e08-436b-bc70-a64d0dcb0db3
lifecycle: ready
---
## Notes

原 S1 的 `0607e1b2`、41 项 Hook 检查与后继 S2/S3 回执保留各自历史范围。2026-10-26 在同一 steer-responsiveness Outcome 内按用户已批准的 composer 多 Steer/唯一 Goal 行为修订此原文件，保留 artifact_id 与 delivery_ref，并修正已落入 Map 而原 summary 尚未同步的文字。当前根为 framework `9c5e202d`、EKO `780ddd0` 加已采用 dirty 增量；37 项 controls 与35项渠道组合检查只是限定前端证据，epoch08 queue已3/3通过，P1 recover子进程stack overflow/cargo101，batch确认无活动target进程、clean0及target_removed；核心线诊断中。原根 Rust 101 和外部 epoch07 中断记录继续保留，不据旧源码/摘要关闭本计划。

核心 Rust 线统一拥有 conversation_input、journal、app_state、共享任务运行时及 chat/task_runtime IPC；正在验证时冻结这些输入。GUI 线当前仅拥有 echo-agent-cli 内六路径：web-frontend/src/components/chat/ChatPanel.tsx、ChatPanel.controls.test.tsx，docs/en/guides/gui-task-runtime-panel.md、docs/zh/guides/gui-task-runtime-panel.md，以及最小 IME 修复的 web-frontend/src/components/workspace/NewTaskDialog.tsx、WorkspaceDialogs.a11y.test.tsx。useTauriChat.ts、useTauriChat.test.tsx、taskRuntimeStore 与共享 API/endpoints/生成类型仍属中央集成接缝，未经另派不得写入；Todo files 表示实现涉及范围，不授予 GUI Hook 或其它文件写权。有效要求实现选择 Plan14，不将其全部义务暗中并入一次 GUI 控件修复。

epoch09 诊断源码已冻结；核心同意该诊断与单一 GUI 可执行 build 并行，合计最多两条 focused Rust，各自 jobs≤2、dev/test debug=0、根内独立 target。完整矩阵和正式性能测量不开。GUI 留存 binary 并明确释放源码使用窗口后，核心才修改生产源码；构建许可不扩大 GUI 六路径或中央 Hook 的写权。

相邻定向 Rust 检查、屏障反例和正常控制合为源码稳定后的一批执行，避免每条小测试重复冷编；框架和应用各使用自有准确 target，完整矩阵只在适用合流点执行。批次结束先保存日志、真实退出结果与必要制品到 target 外，再确认进程和 reader 已结算，及时对该批准确 target 执行 cargo clean 并记录磁盘；没有 Cargo 产物的前端/文档阶段不清他人 target。
