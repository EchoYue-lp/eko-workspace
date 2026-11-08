---
schema_version: 4
slug: 2026-11-07T1550-微信渠道恢复/plan_01
outcome:
  summary: 恢复 Eclip 原微信会话在停止、重连及历史输入待恢复场景下的可靠收发：用户可明确放弃阻塞项，新消息沿原运行时执行并回复，原失败与未知结果保持真实。
  acceptance:
    - 生产 ChannelRuntime 管理入口在同一进程完成连接、收消息、停止、重连并再次收消息；连续两轮新消息均只产生一次 durable
      submit、claim 和原发送回执，旧 incarnation 的回复不能进入新会话。
    - 含历史 MailboxAccepted、TurnSettled(Failed, drained=false)、RecoveryUncertain
      的同一会话可以保留原阻塞，或由用户明确放弃准确排队项后继续处理新消息；旧原始日志、TaskRun 状态、drained 与副作用/发送
      UNKNOWN 不被改写为成功、撤销或已送达。
    - 准入失败可在现有渠道页或原对话中明确看到；authenticated poll Ready、消息持久准入、模型执行、发送
      Accepted、手机真实收件分别有证据，不互相代替。
    - 最终同一源码快照的回归和适用仓库门禁通过，新本地签名 App 绑定准确源码与前端资产；使用原数据目录、原账号及原工作空间完成手机真实收发后才标记该
      Outcome 完成。真实验收未执行或失败时保留待验/失败状态。
out_of_scope:
  - 不自动执行本机旧业务消息、重发 UNKNOWN 通知、修改原 TaskRun
    结果、删除原日志/凭据/会话历史、清空运行时目录或借切换账号/工作空间绕过阻塞。
  - 不全局修改 Failed 且 drained=false 的通用 FIFO/重试合同，不新增 inbox、store、消息执行器、SQLite
    或平行恢复状态机。
  - 不增加 IM 平台、群聊、媒体支持或协作控制台；SDK 协议与 framework pin 不因新增 Rust 会话结束原因自动升级。
  - 本计划不授权提交、推送、合并、发布、自动操作 GUI、扫码或向外发送消息；原会话放弃动作和手机验收由用户在交付界面明确操作。
design_ref: null
design_sections: []
delivery_ref: null
todos:
  - id: settle-session-on-shutdown
    summary: 补齐 Framework 关闭成功后的唯一会话结束通知，使原应用协调器能正确接纳重连后的新会话。
    files:
      - echo-agent/echo-integration/src/channels/session.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/management.rs
    acceptance:
      - 先扩展现有 session_close_keeps_failed_agent_owner_for_retry、取消 close 重试及
        stream settlement 测试取得缺失 Shutdown 回调的 RED；使用现有 SessionEndReason 新增通用
        Shutdown，并复用 settle_session_end/defer_end。
      - 只在 inner close 成功且 remove_if 实际删除同一 session 后，释放 session mutex/map
        引用并同步通知；删除与通知之间无 await。失败或取消保留原 owner，重复/并发 close 回调恰一次，callback panic
        containment 保持。
      - 真实 SessionHandler A 首次准入、close A、同一 coordinator 重开、SessionHandler B
        准入的回归证明 current 为 A→None→B；不得手写 record_session_end 替代生产回调。
      - 应用继续保留 retirement gate 和准确旧 runtime obligation，沿现有
        settle_pending_retirement_for_runtime 处理；reopen 在原任务未物理 drain 或仍有
        current incarnation 时明确拒绝，不能清空 lifecycle 来伪造通过。
      - 旧回调迟到不影响 B 的 pump，旧 foreground/delivery 未结算不准提前开启新效果；QQ/飞书共享会话
        reset、timeout、close 回归和 SessionEndReason 所有消费者编译通过。
  - id: abandon-exact-recovery-input
    summary: 在原 IM 对话增加单条待恢复消息的明确放弃操作，由原渠道 owner 使用原日志权威解除阻塞并唤醒后续输入。
    files:
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/management.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/weixin_control.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/input_pump.rs
      - echo-agent-cli/echo-agent-app-core/src/conversation_input.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/journal.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/projection.rs
      - echo-agent-cli/src/tauri/commands/channels.rs
      - echo-agent-cli/src/tauri/commands/chat.rs
      - echo-agent-cli/src/tauri/mod.rs
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx
      - echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts
    acceptance:
      - 先用原 journal 和生产 claim 路径复现旧 RecoveryRequired 阻塞同地址新消息；用现有前端队列测试复现
        channelOwned 无可用移除入口。随后只为准确 RecoveryRequired
        项显示“放弃这条待恢复消息”，说明停止排队不等于撤销已发生操作。
      - 新的薄 IPC 委托 ChannelRuntime，按输入 identity、观察版本及原账户/credential
        generation、workspace generation 绑定核验；在同一原队列权威锁内确认当前 phase、无 live owner
        后复用 ConversationInputService::cancel 的追加逻辑，不先检查后无条件取消。
      - 保留现有公开 cancel 调用的适用行为；所需的版本/phase 校验在同一实现中扩展，不复制取消状态机。处理该历史项不重绑到当前 GUI
        项目或最近账号；账号/工作空间变化不得静默替换原输入身份。
      - 取消后的排队 receipt 可为 Cancelled，但界面分别显示“队列项已放弃”和原执行状态；历史失败、原
        reason、drained=false 及远端效果/发送 UNKNOWN 保持，未添加虚构 settle_turn 或 drain 事件。
      - 恢复写入沿同一 authority 执行版本与终态检查；含 expected_queue_revision=None 的原路径也不能用迟到
        RecoveryUncertain 复活 Cancelled。并发放弃/恢复/claim、陈旧版本、正在执行、重复放弃均有回归，重复成功幂等。
      - 放弃成功后由原 slot kick/start_input_pump 路径唤醒原处理循环，不生成第二个 owner；同地址下一消息执行一次，不同
        workspace 独立推进。平台再次投递被放弃消息时沿原 tombstone 去重，GUI 编辑/执行其它渠道输入的隔离保持。
      - 真实 Failed,false 通用输入回 FIFO 并产生下一 attempt
        的既有回归继续通过；本操作通过用户明确动作产生取消事实，不自动把旧失败解释为永久完成。
  - id: surface-admission-failure
    summary: 保留消息准入错误并在现有渠道状态中显示真实阻塞，保持 poll、执行和发送各自证据。
    files:
      - echo-agent/echo-integration/src/channels/channels/weixin/channel.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/weixin_control.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/management.rs
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/components/channels/ChannelsPanel.tsx
    acceptance:
      - 用现有真实 poll loopback fixture 先复现 admit_with_ingress 返回错误被忽略；随后由原
        admission task/poll owner 收集失败，保留有界脱敏分类和关联标识，不输出
        token、context、消息正文或原始账户信息。
      - 原渠道页分别显示微信传输已连接与消息接入失败/待处理；成功 poll 不能清除尚无持久准入成功证据的错误。准入错误不得伪造
        DurableInput、TerminalRejected、执行成功或发送 Accepted，也不得释放未确认 frontier 或推进
        cursor。
      - 明确 owner 初始化失败、服务关闭、历史待恢复等失败类别；诊断作为原 owner 的观察投影，不能拥有调度、重试或结算权。
      - start_configured 的渠道启动失败与启动后的 completion recovery pending 分别记录；SSH
        工作空间未准备只报告原通知恢复待处理，不再统一误报为整个 IM 未启动。
      - 前端渠道状态测试证明“连接已建立但消息接入失败”可见，重复错误有界且解决后按准入事实清除；不增加为了提示错误而重发消息的路径。
  - id: validate-original-control-path
    summary: 以生产渠道管理入口验证重连、阻塞解除、恢复及去重闭环，并同步所属仓库文档与示例。
    files:
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/management.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime/management/weixin_a2_consumer_tests.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx
      - echo-agent-cli/web-frontend/src/components/channels/ChannelsPanel.test.tsx
      - echo-agent-cli/web-frontend/src/api/weixin-invoke-contract.test.ts
      - echo-agent/docs/zh/15-im-channels.md
      - echo-agent/docs/adr/
      - echo-agent-cli/docs/zh/adr/0056-personal-weixin-channel.md
      - echo-agent-cli/docs/zh/configuration.md
      - echo-agent/echo-agent-learning/examples/demo38_im_channels.rs
      - echo-website/
    acceptance:
      - A2 fixture 使用与生产相同的 with_on_session_end 接线，新增同一进程、同一
        ChannelRuntime/coordinator 的 stop/reconnect 回归；不能销毁 fixture 后新建来替代重连。
      - 组合样本证明 provider 消息→持久输入→唯一 claim→原 driver→原
        reply→发送回执→cursor；覆盖两轮重连、冷重开、平台重复投递、长回复停止、放弃与恢复竞争以及新旧 workspace 隔离。
      - 回归先保留修改前失败，再在最终同一源码快照运行直接相关 Rust/前端测试、fmt 与 Clippy；应用和 framework
        适用的仓库级门禁及公共 API 条件矩阵按各自 AGENTS 执行，任何已执行失败必须修复。局部通过不能替代全量或 App 验收。
      - Framework 的 Shutdown 通知合同和 Eclip 的输入放弃策略分别写入所属中文文档/ADR；检查 examples
        与官网，需同步时由对应写入责任方更新，不适用项说明原因。既有 IM 示例进入适用编译检查。
      - 源码冻结期间不接纳其它会话对相关文件的写入；提交前的 main 合流、精确依赖门禁与跨仓顺序依原 AGENTS
        执行，不借本计划改锁或自动续签风险基线。
  - id: deliver-and-accept-native-app
    summary: 交付包含本次修复的准确签名 App，并在原数据目录完成用户手机真实收发验收。
    files:
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/tauri.conf.json
      - .echo-semantic/local/
    acceptance:
      - 沿当前已验证的 App 构建配方使用本任务独立 target，绑定两仓实际 HEAD、WIP
        文件哈希及前端资产；保留原始构建日志、实际退出码、程序哈希、签名/strict verify 和启动器只检查结果，保全必要产物后按 AGENTS
        清理准确 target。
      - 在用户正常保存退出旧 App 后提供新启动入口，继续使用原配置、凭据、数据目录及原渠道绑定；本计划自身不启动或停止 GUI，也不改本机历史记录。
      - 由用户在新 App
        手动重连，观察旧阻塞项并明确选择放弃，再从手机发送两条不同的新消息；实际确认每条收到一次回复且原聊天记录/运行状态正确。另做一次停止重连及
        App 重开后的新消息收发。
      - 真实验收按消息关联标识逐层记录准入、执行、发送 Accepted 与手机收件；发送失败或 UNKNOWN
        保留原结果且不自动重发。新包可启动、poll Ready 或模拟测试通过均不计作手机验收通过。
      - 未取得真实验收时只交付已验证候选，不把本 Plan 标为 completed；不把本地候选、已推送、已合并、精确 main CI
        与发布混写为同一状态。
artifact_id: plan:ff675784-0933-485c-8d05-03413ce26f4c
lifecycle: ready
---
## Notes

### 当前交付指针（2026-11-08 归档，Asia/Shanghai）

本节仅补交付导航与时态，不改变上方 Outcome、Todo acceptance、绑定或 `lifecycle: ready`。准确本地源码组为 Framework IM `1f35f0d2f9274c60f9cb009f29675efc33f80422`、CLI 后端/IPC `70ac06a52479ea59b6bd9f67df1173e891b80b24`、Accepted 观察 `bd2a083ee6b24ab27864b8becdb3082f1dff3494`。

R08 的 293 个唯一相关测试、R09 长回复及四项相邻 exact、R13 观察及四项相邻 exact，分别绑定各自源码票，不能跨切点累加或推为独立提交树/最终 main 资格。同源 phone 材料仅离线准备 Final C0/I0/M0，不是真实 App 日志或手机验收。准确 source manifest、原件及保留项见[阶段报告当前归档指针](../../reports/weixin-channel-recovery-2026-11-07.md)。

新密度包取 CLI `a1ea6579…`＋Framework `2b296b7e…`＋已接受的 CLI `48eece73…` 锁叠加，Ready 文件为 `86f01cac…`、签名 Mach-O 为 `c9d1b2fb…`；仅作 GUI 密度交付，未并入未提交 SRE，不继承 IM 手机或最终组合风险门禁资格。完整 Plan 继续 `ready`，最终 source-risk/gates、真实日志及原账号手机未验；本次不自动 GUI。

### 原始计划撰写与取证（历史）

以下至文末保留原计划编写时的取证、假设与动作边界；“当前”“本轮”“尚未”等只指当时，不撤销后继已取得的切点证据，也不改变尚未满足的整体验收。

本计划按 Todo 列表顺序执行。一个 Outcome 是“恢复原微信会话可靠收发”；Framework、应用与前端是共同实现该结果的切片，不单独签成完整修复。

取证切点：2026-11-07 15:50（Asia/Shanghai）。Framework HEAD `f2e44dc9be8972e286f9a84520ddb263f0012424`，CLI HEAD `5f5fd4912855cfda2326c54f294f11cb879257fa`，均含当前工作树改动；不能只按 HEAD 重建。两仓 HEAD 较上一轮诊断已变化，下列关键文件字节与上一轮一致，已增量复核缺陷仍在。当前主目录为唯一允许开发根；实施前重读当前 AGENTS 并核对文件写权和实际增量。

| 当前相关文件 | SHA256 |
|---|---|
| `echo-agent/echo-integration/src/channels/session.rs` | `5d6aa856500b6f8b851bf83ffd1b2dfe3baded9428dc5e06c55fdf65f706e0e1` |
| `echo-agent/echo-integration/src/channels/channels/weixin/channel.rs` | `a8ac1eb9bbe1f0c3e8f766f89f06004fc7e83653c023c617a7af29a21b0b9ab8` |
| `echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs` | `71b6f2de1512edce0cad8bae596e014927cb36b817fa855496250ddb6e6d486c` |
| `echo-agent-cli/echo-agent-app-core/src/channel_runtime/management.rs` | `08102a40ff5d6a4a86ea1cce2a61aa3bcf55e3b3f214976c146c2df555d2ebd4` |
| `echo-agent-cli/echo-agent-app-core/src/channel_runtime/weixin_control.rs` | `df660ab7953a0eeada5d410942e7d7c86b2a9a0e2854bf5ff4146d636f82d60c` |
| `echo-agent-cli/echo-agent-app-core/src/chat_event_log/journal.rs` | `2b8e8ac2325eed5a4fa8710d39fbacdf2d83b1b88b1b8d610e4aaac2b63e6567` |
| `echo-agent-cli/echo-agent-app-core/src/chat_event_log/projection.rs` | `8827d30200253bc2608d3b44fbe712cad9239d851d25f0760704fb52ae27997a` |
| `echo-agent-cli/echo-agent-app-core/src/conversation_input.rs` | `29c132cce10c2a42a963ec9298e4fd049ac570825cfd1b175806aafb08f95b20` |

源码证据：`session.rs:1186` 的 close 成功删除缺结束通知；应用 `channel_runtime.rs:640/690` 分别为 register/record_session_end，`:2055` 的 frontier 与 `journal.rs:1310` 的原子 claim 均阻断 RecoveryRequired；`weixin/channel.rs:843` 忽略准入错误；`chat.rs` 的 require_gui_owned_input 与 ChatPanel 的 channelOwned 禁用说明现有 GUI 移除入口不能处置此项。`journal.rs:727` 的无条件/可选版本 uncertain 写入与 `projection.rs:927` 的覆盖投影须一起覆盖迟到恢复竞争，不能只加按钮。

本次现场 `owner is not available` 来自多个条件合并的错误；停止通知缺口和旧输入阻塞已由源码确认，但尚未用受控运行证明现场只由其中一个条件触发。Todo 的失败复现必须沿生产调用路径保留具体分类，不能把假设当作已执行结论。

参考已核对的官方实践：[Tokio Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown) 在取消后等待原任务结束；[python-telegram-bot Application.stop](https://docs.python-telegram-bot.org/en/stable/telegram.ext.application.html#telegram.ext.Application.stop) 区分 poller 与应用任务停止并等待原任务。本计划沿用既有取消/settlement/callback 原语，补足准确通知和用户明确处置的薄适配。

本轮只写本计划，不修改业务源码、测试、配置或原数据，不执行 Cargo/App/手机验收。列出的构建与验收均为待执行义务。
