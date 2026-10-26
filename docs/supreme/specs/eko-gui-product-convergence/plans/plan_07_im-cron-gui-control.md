---
schema_version: 4
slug: eko-gui-product-convergence/im-cron-gui-control
outcome:
  summary: 在 GUI 中管理已有 IM 渠道的配置与连接，并完成定时任务启停、执行、结果与恢复的共享运行时接线。
  acceptance:
    - GUI能管理现有QQ
      Bot与飞书（long_poll及已有webhook模式）的配置、启停/重连和可定位结果；分别呈现配置持久化、真实连接事实和实际发送结果，不把DTO存在、start返回、health_check或本地队列接受当作平台已投递。失败/未知如实保留，凭据不回显、不记录到日志/普通事件。
    - 桌面应用持有原ChannelManager/session/input
      pump及scheduler/TaskRun服务；窗口X只隐藏，托盘显式显示/退出，macOS既有Cmd+Q/应用菜单退出意图仍真正退出。hide/restore不暂停IM或cron；真正quit关闭准入并沿原run_return/ApplicationLifecycleOwner.settle等待原owner，清理失败/未知保留，不以process::exit绕过。
    - 复用现有SchedulerPanel、SchedulerRunner、CronTaskStore、occurrence
      journal与TaskRuntime：创建/修改/启停/手动执行/历史结果均可定位同一task/definition/occurrence/run事实；同occurrence重投不重复模型/工具执行，独立手动请求不误合并，无配置不默认创建cron，不新增崩溃后自动续跑政策。
    - 每次真实HITL投递在原请求owner内调用同一个EKO局部配对入口，原request_id/scope保留，缺失ID在此生成一次；GUI-only、TaskRun恢复、browser
      approval单端每次实际request各得一次非空response_target，继续由原provider
      future结算。GUI+IM组合在克隆前配对并共享同一target，仍由原HitlDispatcher只裁决一个有效答复，不把单端强塞全局dispatcher。IM统一/reply
      <response-target> <answer>；空/旧token或错scope/generation不得降级裸text或进入普通chat。
    - 正式文档/配置示例/官网影响与真实交付对账；源/锁/平台、mock与真实服务、窗口行为、未知结算和所用target有精确证据。当前候选阶段未实现或运行这些验收；实际实现必须从届时明确交接的消费者去依赖/GUI合流来源开始，不能回灌db501旧整文件。
out_of_scope:
  - 不新增IM平台；Lark构造函数、其它provider名或配置字段不视作EKO已接入平台。先完善QQ
    Bot/飞书，新增平台仍由独立new-im-channel按需具名。
  - 不新增IM消息持久化/重试事实层、第二TaskRun/调度Store或执行器，不引入SQLite，不以页面控制替代原ChannelManager、DeliveryFence、scheduler
    occurrence或TaskRuntime owner。
  - 不退役CLI/TUI/JSONL/channel产品入口；本Outcome只归位仍被GUI/IM/cron需要的共享实现并保留当前调用者薄适配。console-surface-retirement另行验收。
  - HITL视觉美化、Modal布局与可发现队列交互属于后继gui-hitl-dialogs；Plan07仅含必要的原请求身份/response_target贯通、路由/生命周期与最小回复参数传递，不重做弹窗/任务列表，不裁决旧ADR0034/0039委派政策。不新增Framework
    HumanLoopRequest/Response/InboundMessage字段、平台reply关联、第三queue/store、HITL
    terminal状态或Agent通信控制台。
  - 不把后台常驻解释为进程退出后继续运行，不自动公开端口/搭建穿透或远程daemon；真实IM收发需协调者指定凭据与私测目标，未获具名私测目标及明确发送授权时不发送真实消息。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § IM 与定时任务管理
    digest: sha256:0d9fc52c40b8eff51cac3a953a25738351f53e00ae3728e7fdab947d383aa9fa
  - ref: design.md § HITL 弹窗
    digest: sha256:6d1ace366e78550f0a0eda37546aab3abd4c7597fa872e4a6ddd1ae9e7febb49
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:6f59f80f3c5fe48e08bac490651377bf47f90e5dbde6560e47c4e5d16d9fcde5
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#im-cron-gui-control
todos:
  - id: relocate-shared-channel-runtime
    summary: 把现有EKO渠道执行与关闭逻辑归位共享应用运行时，桌面与旧入口共用同一owner。
    files:
      - echo-agent-cli/src/cli/channels.rs
      - echo-agent-cli/src/cli/channels/input_pump.rs
      - echo-agent-cli/src/cli/channels/outbound.rs
      - echo-agent-cli/src/cli/channels/tool_render.rs
      - echo-agent-cli/src/cli/modes.rs
      - echo-agent-cli/src/main.rs
      - echo-agent-cli/echo-agent-app-core/src/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/api/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/Cargo.toml
      - echo-agent-cli/echo-agent-app-core/Cargo.toml
      - echo-agent-cli/echo-agent-app-core/src/lib.rs
    acceptance:
      - 从run_channels_mode及AppChannelMessageHandler/ChannelSessionCoordinator沿真实调用归位；保留SessionHandler的channel/conversation/sender隔离、incarnation/foreground
        lease、ConversationInputService持久准入、TaskRuntime/pool/工作空间、附件/工具渲染、脱敏和投递fence。旧cli路径只作薄调用，不复制第二个manager或coordinator。
      - 继续使用ApplicationServices/ApplicationLifecycleOwner.track_external_owner及原取消/共享join收据；停止时先关闭渠道/pump准入，再按原manager
        stop_all和input-pump join排空，失败/取消不丢原资源与清理义务、不按observer消失假报物理完成。
      - 无渠道启用是正常空态，不沿旧--channels无可启动渠道错误让GUI启动失败；单渠道失败不关闭另一正常渠道，重复启动/停止不会多装handler。gui原已包含channels
        feature，迁到app-core时显式保持feature组合和featureless检查可达，不能靠all-features掩盖缺口。
      - 共享模块channel_runtime.rs和管理组件目录仅为拟新增落点；迁移承载原应用逻辑，不产生新IM任务权威。runtime/app_state/channels.rs/GUI注册等与现有writer串行合流；正式Build从协调者明确交接的后继SHA/lock/受保护diff开始，不从db501整文件回灌。
  - id: manage-existing-channel-configuration-and-lifecycle
    summary: 为QQ与飞书接通真实的配置、连接控制和状态观察，GUI只呈现原运行时事实。
    files:
      - echo-agent-cli/echo-agent-app-core/src/config.rs
      - echo-agent-cli/echo-agent-app-core/src/state/config.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/types/request.rs
      - echo-agent-cli/echo-agent-app-core/src/types/response.rs
      - echo-agent-cli/src/tauri/commands/config.rs
      - echo-agent-cli/src/tauri/commands/mod.rs
      - echo-agent-cli/src/tauri/mod.rs
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/components/layout/SettingsDialog.tsx
      - echo-agent-cli/web-frontend/src/components/channels/
      - echo-agent/echo-integration/src/channels/manager.rs
      - echo-agent/echo-integration/src/channels/types.rs
      - echo-agent/echo-integration/src/channels/channels/qq/
      - echo-agent/echo-integration/src/channels/channels/feishu/
      - echo-agent/echo-integration/src/channels/mod.rs
      - echo-agent/src/channels.rs
    acceptance:
      - 保存复用AppState.update_app_config_owned和不可变config_path；现有QQ/飞书enabled/app
        id/secret、飞书mode及已有webhook bind/path/verification
        token可按原配置能力管理。省略secret表示保留，明确清除与普通空输入区分，不把掩码保存成凭据；只返回脱敏/是否已配置等必要投影，错误与Debug同样脱敏。
      - 保存与连接应用分别回执；控制携带应用当前配置revision以拒绝陈旧GUI意图，并只操作所选渠道。替换凭据/模式前await原ChannelHandle.stop及handler/pump结算；仅原entry确认stopped、无handler/活动invocation后允许replace_stopped_plugin（拟新增原handle方法，同id/label/capabilities，递增现有mutation_epoch）。旧stop收据失效；部分stop/rollback失败保留旧entry和清理义务，不启动替代连接。复用原adapter重连，无第二manager/重试层。
      - 拟新增ChannelPlugin.transport_status() ->
        Option<watch::Receiver<ChannelTransportSnapshot>>（默认None=>Unknown），由原ChannelHandle薄转发；snapshot只含原adapter当前事实phase(Stopped/Connecting/Ready/Disconnected/Failed/Unknown)、evidence、脱敏error和observation
        time。QQ仅READY/RESUMED、飞书仅WS握手完成、webhook仅TcpListener::bind成功可发布相应Ready；webhook只标本机listener
        ready而非平台可达。断线/重连/致命错误由原循环发布，watch关闭不等于成功，原start/token/health不提升状态。
      - 新管理组件目录是候选位置，复用已有Settings
        lazy/common组件及当前GUI视觉；菜单、invoke注册、共享endpoints由协调者唯一写。关闭页面/隐藏窗口只取消观察，不停止连接或正在投递的原owner；发送成功文案限定在adapter/API实际确认范围，不推断对方已读。
      - adapter状态与原manager清理事实分开：handle只读snapshot从原handler、generation与stopped_at_epoch投影handler_attached/shutdown_settled；transport
        Stopped但handler.close失败仍显示清理失败并保owner。每次start/配置epoch换独立状态源，GUI拒收旧epoch观察；发送API成功仅表示平台API接受，既不等同listener
        ready也不推断已读。新增接口均为本候选建议，不宣称冻结提交已有。
  - id: complete-existing-cron-management-and-result-projections
    summary: 补齐现有cron的原子修改和可追踪结果，继续使用唯一occurrence与TaskRun事实。
    files:
      - echo-agent/echo-orchestration/src/scheduler/cron_task.rs
      - echo-agent/echo-orchestration/src/scheduler/runner.rs
      - echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/occurrence.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/boot_reconciler.rs
      - echo-agent-cli/src/tauri/commands/scheduler.rs
      - echo-agent-cli/web-frontend/src/components/scheduler/
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent/echo-state/src/delivery.rs
      - echo-agent/echo-state/src/journal/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
    acceptance:
      - "拟新增原SchedulerRunner.edit_task(expected: CronTask,
        CronTaskEdit{name,cron_expr,prompt})->Result<CronTask,SchedulerEditErro\
        r>，内部复用run_owned_mutation/control_lock及原Store
        mutation_lock单次保存；活runner仅经owner调用Store内部edit_owned，独立Store编辑仍遵守既有owner\
        排他。仅取expected.id及原same_definition比较（非空definition_id，否则created_at），其它exp\
        ected字段不回写，保留最新status；稳定id/created_at；实际改变产生新definition_id并checked_add
        control_revision，清空当前定义last_run摘要；无变化返回原快照。InvalidCron/NotFound/Definit\
        ionConflict/RevisionExhausted/Closed/AuthorityUnavailable或CommitOutcome\
        Unknown明确区分，后两者保留原authority poison与收据，禁止盲重试。"
      - 拟新增run_once_with_receipt(id)->Result<SchedulerManualReceipt,SchedulerManualError>，只在原run_once
        admit/drain路径保留SchedulerOccurrenceRef{task_id,definition_id,occurrence_id,scheduled_at,trigger}和attempt
        identity；成功是同一callback物理完成且原terminal
        commit确认，返回原callback字符串但不解析它。错误保留已知identity、原typed cause和原journal
        receipt的NotCommitted/Committed/Unknown边界；未知不能自动再run。原run_once成为薄映射，不复制循环、不新增executor或后台manual队列；观察者取消后已准入工作仍归原callback
        owner。
      - 拟新增原Runner.read_occurrence_history(cursor,limit)->SchedulerHistoryPage：原occurrence
        journal的只读、按sequence有界typed事件页，cursor绑定journal identity/retained
        floor/首屏snapshot上界；只转投影现有DeliveryEvent语义并暴露Persisted定义/slot和后续attempt/结算事实，无写入/claim/recovery副作用。GUI可按task筛选并分组已读页，TaskRun/result链接由原OccurrenceIdentity与原run/artifact读取生成；找不到run就显示尚无run，不能假造。热projection保留上限不能冒充完整历史，分页未读/缺失不是完成。
      - edit提交是未来准入相对旧definition的边界；旧tick/仅Persisted未EffectStarted的旧定义不能启动，已经EffectStarted的旧occurrence继续使用捕获快照，不被edit/disable/delete隐式取消。旧callback结果保留原occurrence/TaskRun；更新当前last_result时定义已替换/删除属于Superseded投影no-op，不能把成功执行误报失败，也不能覆盖新定义。真正持久化/unknown错误仍上报。两个独立manual各有occurrence；同一occurrence各状态重投均不再执行。
      - history默认128/max512个原journal事件/页是拟定资源边界，非测得性能阈值；limit非法、cursor
        generation不匹配、cursor低于retained_floor、journal损坏/poison分别返回明确错误并给可重新读取边界，不静默reset或拼接不同generation。无定义不建默认cron；保持scheduler先于TaskRun
        boot reconcile以及protected EffectStarted不自动重放的既有政策。
  - id: preserve-hitl-and-desktop-owner-settlement
    summary: 接通同请求HITL观察与桌面后台生命周期，保持隐藏和真正退出的事实边界。
    files:
      - echo-agent-cli/echo-agent-app-core/src/hitl/channel_provider.rs
      - echo-agent-cli/echo-agent-app-core/src/hitl/dispatcher.rs
      - echo-agent-cli/echo-agent-app-core/src/runtime.rs
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/src/tauri/mod.rs
      - echo-agent-cli/src/tauri/commands/chat.rs
      - echo-agent-cli/src/tauri/commands/task_runtime.rs
      - echo-agent-cli/src/cli/channels.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/web-frontend/src/types/api.ts
      - echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts
      - echo-agent-cli/web-frontend/src/stores/chatStore.ts
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/event.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/projection.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/tests.rs
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.test.tsx
      - echo-agent-cli/web-frontend/src/hooks/chatEventHandler.contract.test.ts
      - echo-agent-cli/web-frontend/src/types/api.contract.test.ts
      - echo-agent-cli/docs/adr/
      - echo-agent-cli/echo-agent-app-core/src/hitl/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/browser/mod.rs
    acceptance:
      - 原请求owner采用同一EKO局部pair_request入口（拟放现有hitl模块）：保留原HumanLoopRequest及宿主scope/request_id，缺失request_id只补一次，逐次实际投递生成非空response_target，不在handler构造时按整turn复用。GUI-only、TaskRun恢复及browser
        approval的Tauri
        provider.request直达路径调用它后沿原future/pending/审批与恢复政策结算；GUI+IM组合由原dispatcher在clone前调用一次，将同一已配对上下文传给两端request_paired，后者只验证/消费，不再生成target。空的已配对target拒绝，不退回直达分支。scope保原workspace/conversation/turn/task及channel
        incarnation/host generation，browser
        registration/session_id含义不重写；不新增全局广播、queue/store或Framework字段。
      - IM统一/reply <response-target>
        <answer>，在普通chat/steer/管理命令准入前消费。在原ChannelHumanLoopState同锁内prune、匹配当前front的target/原request_id、验证kind、pop/send。非front、缺target/answer、过期/错sender/错会话/已回答target一律Rejected且不流入chat；审批沿原y/a/n
        reason/m feedback，Input保留余下多字节/多行文本，Selection沿原数字/标签；取消写/reply <target>
        /cancel。有pending的裸文本只提示目标不消费请求，无pending普通文本仍可chat。
        空白/缺失/旧target或错scope均无request_id/front/裸文本降级路径；不查其它会话来补配目标。
      - 原reserved_ids保留至future结束；来源测试允许request_id释放后重用，因此不能只靠request_id抗晚答。原PendingChannelRequest及GUI
        PendingRequest增加response_target/scope/kind，GUI原pending
        map以target定位，remove/send前核对原request_id和精确scope/kind；新active请求事件和三个答复IPC要求response_target；原历史请求事件若无target仅作已过期只读投影，不合成target或重新激活。格式/类型错误不移除等待项；send失败明确expired。send成功仅回submitted；单端沿原provider
        future结算，组合由原dispatcher一次裁决后投影resolved/withdrawn；双端同时submitted也仅一个effective
        winner。该EKO合同增量须协调者独审，不修改框架InboundMessage或HumanLoopRequest。
        字段必须经过真实ChatDriverEvent→JournaledChatSink/ChatEventEnvelope→chat://event→手写TS
        ChatDriverEvent/ChatEvent→handler/PendingHitlRequest→ChatPanel→useTauriChat.ts三个apiInvoke→原Tauri
        pending/reservation；useTauriChat是现有生产答复调用点，不用只改endpoints冒充接通。队列去重/移除以原scope+target而非可重用request_id；点击传原请求scope，不借当前页面refs重绑。
        旧历史缺target的缺省字段处理须保持原payload规范化/事件hash身份，不把null补写或重放变成新active请求；以原日志序列化/重放契约验证，不新增迁移store。
      - 仅GUI+IM组合由原dispatcher将有效Approval/Rejected/ModifiedArgs/Text/Selection视为人类答复，Timeout/Deferred和provider
        unavailable不抢先赢；原provider内部关闭/断连用trait既有Err通道，明确用户拒绝仍是Ok(Rejected)。EKO共享deadline保原300秒上界，req.timeout更短时取min并从该原请求投递起算，不按provider重置；deadline到期沿Timeout结算，全部provider不可用沿原无人可答拒绝，root
        cancel/drop仍沿原取消；隐藏/页面卸载不销毁durable provider。无需修改HumanLoop公共类型或新建中央仲裁状态机。
        特别修正固定Tauri
        Input超时/receiver关闭返回空Text的路径，不能让它抢赢有效人答；失效通知只派生原owner释放事实用于移除相同scope+target的投影，不新增审批terminal状态/独立事实表。
      - 窗口CloseRequested先prevent_close再hide；hide失败保持窗口可见并报告，不转成退出。托盘只有显示/退出，显示可恢复聚焦，macOS既有Cmd+Q/应用菜单真正quit意图保留。所有quit意图经同一begin_shutdown，再由run_return后的ApplicationServices.settle等待原owner；不把X/hide当root
        cancel，也不添加daemon。
      - 复用原provider reservation Drop/timeout与dispatcher丢弃loser
        future清理，原事件投影精确撤销同response_target；验证GUI先答/IM先答/同时提交、A过期且B重用同一request_id后/reply
        target-A及裸y/输入/选择都不能回答B、错会话/类型、超时/断连、重复退出、发送中断及未知结算。真quit等待channel/pump/foreground/scheduler/task/bridge原收据，不能用WebView消失当物理完成。
      - 来源冻结：EKO db5010ed888cf50f2400fbba31075ab63c5626e9 的
        echo-agent-app-core/src/hitl/channel_provider.rs，Git blob
        9f75bb2b94e10d8ae0e04e5d36666122daebd0bd、整文件SHA256
        6960adeed683e79c7ae800bf85837de3fb2cbbb8bc540af15c343132b5b017e6；aborted_request_releases_exact_id_without_cleanup_task
        测试定义677–715原字节行片段SHA256
        17557a80d75bee62fca367d5b8ec52bc602f81ffd369bae6488014cf31bbf40b。这是原测试定义证据，未在规划阶段运行。补三类回复的空token/旧token/错scope、同ID新target、跨incarnation/host
        generation与超时/drop契约用例。新增EKO字段和IPC合同必须在echo-agent-cli/docs/adr/所属ADR记录来源、候选、决定、失败行为和影响；与框架对外API无关，ADR随本Outcome实现维护。
        本增量必须覆盖chat.rs:815/894直达GUI、task_runtime.rs:771恢复、chat.rs:831-837与browser/mod.rs:1155-1178复用handler的caller；单端配对/handler按Approval/Input/Selection参数化验nonemptytarget、原ID/scope、有效与过期回复，browser真实调用另验既有Approval；不宣称browser新增Input/Selection能力。普通GUI/恢复/provider构造链使用原owner，不能为必填字段增加全局dispatcher。
  - id: qualify-existing-channels-and-scheduler-through-gui
    summary: 在真实桌面和具名私测目标上验收已有渠道与cron，交接后续console退役所需证据。
    files:
      - echo-agent-cli/src/cli/modes.rs
      - echo-agent-cli/src/cli/channels/
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/src/tauri/commands/
      - echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs
      - echo-agent-cli/echo-agent-app-core/src/hitl/
      - echo-agent-cli/echo-agent-app-core/src/state/tests.rs
      - echo-agent-cli/web-frontend/src/components/scheduler/
      - echo-agent-cli/web-frontend/src/components/channels/
      - echo-agent-cli/docs/en/
      - echo-agent-cli/docs/zh/
      - echo-agent/docs/
      - echo-website/docs-sync-manifest.json
    acceptance:
      - 复用现有channel session隔离/重置、input-pump
        recovery、投递fence、退订/停止与慢handler回归，加无配置/错误凭据/部分启动失败/断线重连/配置保存失败/绑定端口失败的管理反例；固定源码没有专用channel管理页或SchedulerPanel组件测试，按实际新行为补测试，不编造既有通过数。
      - QQ、飞书long_poll及已配置webhook分别按协调者提供的凭据与私测用户/群/可达入口执行真实收发、停止/重连、GUI状态和结果定位；未具备真实目标时只记录mock/本地证据及未验范围，不发送到未经指定的对象、不以token或spawn成功替代真实通信。
      - cron覆盖原子edit的stale/invalid/commit
        unknown、旧tick拒绝与已EffectStarted旧callback完成、两个独立manual各自identity、失败前后有/无TaskRun、history分页/cursor/retention/poison；HITL覆盖同target竞争、晚答和跨类型/会话拒绝；窗口X/hide、托盘restore、显式quit和macOS退出做真实Tauri窗口取证。此候选不宣称这些测试已执行。
      - 正式文档、配置示例、examples可执行链与官网镜像按实际交付同步/说明不适用；没有新增平台或新crash-retry政策。租约/目标由协调者分配：独立target、jobs<=2、dev/test
        debug=0、全量重矩阵一套，批次前后df/du，<5GiB停新构建；保存Host/报告/真实退出到target外且确认无读取后cargo
        clean。
      - 正式Build开始前重新冻结LSP消费者去依赖与GUI的交接SHA/锁/现有diff并正常预检；当前唯一writer维护runtime/appstate/shared
        endpoints/Settings接缝，其他贡献以小范围接缝合流，不整文件拷贝db501旧版。两消费者Build/其它Outcome/lifecycle不由本准备任务推进。
artifact_id: plan:1bd5cb90-ae0e-4399-b0ee-0ccd9dbfa60c
lifecycle: ready
---
## Notes

调查基线为 EKO `db5010ed888cf50f2400fbba31075ab63c5626e9` 与 framework `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`。本 Plan 的方法/类型名描述待实现接缝，不能当成已有 API。实施者先接收届时已审的精确后继源码、锁及所有权；当前 EKO 消费者去依赖独占的共享运行时/配置/IPC 接缝不得并行覆盖。新增通用框架机制先形成可验证贡献，再由 EKO 薄适配消费。

用户已选择先完善现有 QQ Bot／飞书，新增平台按需。真实渠道验收需在可审阅实现具备后，取得具名私测对象和明确发送授权；缺少条件时保留相应未验收项。产品实现授权与真实消息发送、公开 webhook 入口、发布及部署分别处理。
