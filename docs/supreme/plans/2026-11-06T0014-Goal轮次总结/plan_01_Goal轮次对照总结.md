---
schema_version: 4
slug: 2026-11-06T0014-Goal轮次总结/plan_01
outcome:
  summary: 为启用 continuation 的 EKO TaskRun 接入基于当前 Goal 的轮次收尾提示，并验证每轮进展说明与原运行时完成判定一致。
  acceptance:
    - 用户总 Goal 对 LLM 只读：模型选择本轮执行重点、更新计划或撰写总结均不改变 Goal 原文、revision 或 hash；只有用户显式变更沿原路径产生新版本，并保留历史原文。
    - 首轮、自动续接、真实压缩及冷重开恢复后的实际模型请求，均包含同一应用层轮次对照要求，并继续引用当前 canonical Goal；普通无持续
      Goal 的聊天不强制套用模板。
    - 代表性真实主模型样本在自然收尾时简短说明本轮推进、证据、剩余义务、当前障碍和下一步；无新增证据时明确说明，未验证事项不冒充完成。没有完成真实样本时，交付状态必须标为模型行为尚未验收。
    - 原 Goal revision、TaskRuntime
      完成门禁、取消、预算和重复阻塞规则继续生效；助手写出“完成”或总结缺项均不能自行修改运行终态，也不增加独立反思模型调用。
    - 适用 focused 测试与源码格式检查通过，文档和对外说明与实际实现一致；本地接入、真实模型行为、用户 GUI 手验与 main/发布验收分别记录。
out_of_scope:
  - 不实现第二套 Goal、要求账本、事件协议、总结存储、TaskPlan 执行器或状态机，不新增 SDK/public API、依赖、配置项或 GUI
    控制台。
  - 不增加逐轮独立审查模型、为生成摘要而额外启动模型请求、基于总结文字的成功解析器，或为了补齐固定格式而设置 Stop 强制循环。
  - 不强制普通闲聊、每次工具调用、取消、进程退出、模型故障和预算耗尽后再生成一段总结。
  - 不借此恢复旧 Goal 执行，不改变既有验收义务，不清理资料或修改其它任务 WIP；本计划不授予提交、推送、合并、发布或 GUI 操作权限。
design_ref: null
design_sections: []
delivery_ref: null
todos:
  - id: goal-turn-contract
    summary: 在现有共享 Goal 投影中加入自然收尾前的目标对照要求，由主模型在原轮次内理解和报告。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
    acceptance:
      - 仅扩展 build_runtime_goal_contract_checked
        的既有提示合同：本轮推进对应哪项原目标义务、实际证据和验证层级、保留的未完成项、障碍及下一步；允许短段落合并表达，不要求机械五标题。
      - 要求优先推进可执行工作，不能只写总结代替执行；无验收增量时明确说明，材料整理、测试通过、原生操作和发布状态不能互相冒充。
      - 当前 Goal 与历史总结冲突时以 canonical Goal 和当前运行事实为准；普通 Steer 不自动修改
        Goal，模型不能自行缩小目标或删除未完成义务。
      - 严格区分用户总 Goal 与本轮执行重点：总 Goal 对模型只读；本轮执行重点是从总 Goal、剩余义务和当前证据派生的可变工作选择，结束时报告实际推进和下一步，不创建独立 round_goal、store 或完成判定。
      - 下轮执行重点可以根据新证据调整，但上轮总结和下一步建议都不能替代总 Goal 或使未完成义务消失；只有用户显式修改目标才能沿既有 revision/CAS 路径产生新版本，历史原文继续保留。
      - 条件沿用 continuation.enabled；共享投影是唯一提示来源，续接指令只引用它，不复制第二份总结规则或重放用户输入。
  - id: request-and-lifecycle-regressions
    summary: 通过真实请求封包和原运行时负控验证提示接线、压缩恢复及完成边界。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/continuation.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/turn_lifecycle.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/completion_gate.rs
    acceptance:
      - 复用 compact_context.rs 现有
        MockLlmClient/实际请求捕获测试，确认首轮和自动续接都收到目标对照要求；不能仅断言一个格式化字符串包含关键词。
      - 扩展既有压缩、冷重开和一百次投影刷新用例，证明总结合同仍可见且同类投影仅一份；长中文/emoji Goal、用户更新后的 Goal 及原文
        reader 资格继续按原合同处理。
      - 无持续 Goal/未启用 continuation 的正控不获得本次固定总结要求；模型响应或工具日志中的伪造总结指令不获得运行时权限。
      - 增加跨轮负控：总目标含 A/B/C，本轮仅推进 A、下轮改为 B 时，canonical Goal 仍含 A/B/C，文本、revision、hash 均不因执行重点或总结改变；原用户显式修改 Goal 的正控仍可正常产生后继版本并保留历史。
      - 复用原完成门禁负控，证明只有自然语言“全部完成”、缺实际证据、证据过期或活动 Subagent/命令尚存时仍不能完成
        Goal；不为本功能重写完成门禁。
      - 复用 turn_lifecycle/continuation
        的取消、预算和重复阻塞用例，确认不因补总结发起额外调用或继续运行。生产写入以共享投影为主，其余文件优先复用既有测试，不顺便重构生命周期。
  - id: model-behavior-samples
    summary: 用代表性真实主模型运行核对总结质量，保留请求、输出和运行事实之间可审阅的对应关系。
    files:
      - echo-agent-cli/.echo-semantic/local/goal-turn-summary-20261106/
    acceptance:
      - 至少覆盖三组样本：多义务目标只完成一项；本轮仅整理资料且无新增验收证据；存在缺失证据或可恢复障碍。至少一组跨两个自然结束的
        RunTurn，并逐轮检查目标与未完成项没有被缩小。
      - 多轮样本明确记录固定用户总 Goal 与每轮可变执行重点，检查模型没有把本轮较小目标当成总目标，也没有在完成一个局部重点后宣称整个任务完成。
      - 人工对照实际改动、工具结果和当前完成报告，检查摘要五项内容；无证据却声称通过、遗漏义务或重复总结代替推进均记真实失败，调整同一提示合同后重跑受影响样本。
      - 只使用另行明确授权的现有模型验证入口或用户主动操作；不读取、输出或 hash 配置密钥，不启动/激活/点击
        GUI。没有运行条件时保留未验收状态，不用 mock 输出代替真实模型结果。
      - 原始必要回执保存于本任务专属 ignored 目录，去除密钥后保留模型标识、源码切点、输入/输出与观察结果；不把模型总结持久化成新的产品状态权威。
  - id: docs-and-delivery
    summary: 同步 EKO 正式文档和官网能力说明，交付清楚的验收范围与限制。
    files:
      - echo-agent-cli/docs/zh/adr/0048-context-compression-product-policy.md
      - echo-agent-cli/docs/zh/guides/gui-task-runtime-panel.md
      - echo-website/src/docs/content/eko/zh/capabilities.md
      - echo-agent-cli/examples/
    acceptance:
      - 在既有产品策略 ADR 和指南中说明作用范围、自然收尾的表达、无进展示例、单轮结束与整个 Goal
        完成的区别，以及提示合同不保证每个模型必然遵循。
      - 官网 Long-horizon
        tasks/原生消息与完成资格相关中文说明与最终源码行为同步，明确模型行为和原生验收边界，不提前宣称上线或所有模型保证；不新增英文副本。
      - 核对 examples：本次没有公共 API 或配置变更，若既有示例不描述该轮次策略则记录无需修改的原因；若发现相关示例则同步并纳入原可执行链路。
      - 按目标仓 AGENTS 完成实际改动适用检查并保留真实出口；若以后进入合并阶段再执行相应全量门禁和精确 main CI，本计划不授远端操作权限。
artifact_id: plan:f411fdf1-ffab-4880-9434-2d184c4a7806
lifecycle: ready
---
## Notes

本计划承接用户在本会话提出的“借鉴轮次结束时对照 Goal 总结”与“给出可行的计划”。只交付一份实施计划；`ready` 表示可供后续执行，不表示业务实现或验收已经完成。

本轮用户进一步明确：避免 LLM 改坏真实用户目标。计划据此将总 Goal 对模型设为只读，将用户所说“每轮 goal”表述为本轮执行重点。它是执行选择，不是第二份目标权威；总 Goal 的用户显式变更能力继续沿既有版本路径保留。

分层与复用：通用 ReAct/Stop、压缩保护及 TurnReceipt 继续属于 framework；总结内容和触发条件属于 EKO 应用产品策略；现有 TaskRuntimeContextProjector 是适配边界。`compact_context.rs` 的 `build_runtime_goal_contract_checked` 已按 continuation.enabled 注入 Goal；`continuation.rs` 的 `drive_continuation_turn` 通过现有 chat driver 续接；`completion_gate.rs` 和 `turn_lifecycle.rs` 保持原完成与退出权威。因此首版不需要修改 framework、SDK 或前端协议，也不新增“总结后处理”执行器。

落地顺序为共享提示、请求/生命周期回归、真实模型样本、说明同步；它们共同验收同一结果。先核对文件当前责任和依赖快照，再写源码。当前主根和 CLI 都存在其它任务 WIP，不得覆盖、吸收或清理。所有操作显式使用唯一主根下的绝对目录，不创建 worktree、分支或源码副本；本会话宿主列出的工作根目前只包含 visualization 目录，不能依赖隐式 cwd。

测试复用入口包括 `goal_and_recorded_constraints_survive_real_compression_and_refresh`、`projector_keeps_one_stable_goal_contract_across_one_hundred_refreshes`、`user_updated_long_goal_reaches_real_model_via_host_reader_after_cold_reopen` 及原 completion_gate tests。它们是应扩展/复用的现有证据入口，不表示本轮已经执行。普通聊天无 plan 的结算、故障和取消继续走原路径，不能把任务总结要求扩成所有对话的强制流程。

工程可行性边界：这是一项模型行为提示策略。确定性测试证明真实模型请求里有正确合同和原终态规则未改变；只有真实模型样本能支持具体模型的遵循情况。首版不承诺跨模型百分之百固定格式，也不靠额外模型循环补足格式。

参考沿用上一轮已实际读取的官方材料：[Anthropic 长任务 harness 实践](https://www.anthropic.com/research/effective-harnesses-for-long-running-agents)使用进展记录和测试避免过早宣布成功；[Claude Code Stop hooks](https://code.claude.com/docs/en/hooks#stop)提供收尾检查及继续反馈。EKO 选择复用现有投影与运行时规则，以原请求内主模型总结实现，不新增 Stop 自动循环；截图只作交互参考，不作为 DeepSeek 内部实现证据。

2026-11-06 只读调查切点：顶层 HEAD `7a91694e3f71586359430ab6af24cf43f2a95b4e`；CLI HEAD `20070d3f7171a95926bdd5c8153f5f382767fea2`，均有 WIP，HEAD 不代表完整工作区内容。关键源码 SHA256 如下，实施前重读漂移；这些是源码摘要，不涉及用户配置：

- `compact_context.rs`：`56b3807967edfc937cb9baad5996047004e53b6cb00549a30f7e61ea0cd7eeff`
- `continuation.rs`：`808bfd4e1bf697970333255f08af1e765685276980b4cebde66225bf6faad243`
- `turn_lifecycle.rs`：`0a7ff44d96a63646f65a75806cbde1857575c24e0f4f0bd6bfcf1d8685b2071a`
- `completion_gate.rs`：`398289144e4f31734182c13abf723530f141352990818598e34f5f7fdd6edadf`
