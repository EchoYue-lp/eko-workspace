---
schema_version: 4
slug: eko-gui-product-convergence/steer-context-and-goal-authority
outcome:
  summary: 普通 Steer 通过既有持久输入与历史交给主模型理解和按需调整计划，必要原文在压缩与冷恢复后可读取；初始 Goal
    可由智能体在尚未建立时设置，已有 Goal 仅由用户显式修改或结束，保持原任务版本、输入幂等和失败恢复边界。
  acceptance:
    - 普通Steer完整原文经原持久接纳、tracked
      mailbox和真实主模型请求消费；主模型可沿既有task_update调整计划，但普通输入/消费不修改已有goal、goal_revision或goal_hash，且没有额外意图模型请求。
    - 尚无运行目标时既有初始化可设置初始Goal；已有运行的重试/初始化重入、模型任务更新和冷恢复保留原Goal。用户GUI/IM显式CAS修改是正控制，冲突/失败/迟到不覆盖原状态；用户结束与普通取消保留历史Goal，不写空目标或新增取消权限门控。
    - 必要历史和完整长原文在真实压缩/冷重开后仍可读取，末尾限制不被预览截断；必要Goal/原文/范围不可恢复时暂停相关副作用并保原owner，修复后沿原恢复入口继续。
    - 真实消费者、原生与历史切片逐项对账，只修证实的缺口，同源证据可复用；中文ADR/运行时、examples/官网适用性及所属仓门禁明确，focused不替代完整/native/main
      CI资格。
out_of_scope:
  - 不调用独立模型理解意图，不新增结构化EffectiveRequirement解释协议、要求store、分类器、GoalStore、actor权限层或统一强制要求/计划/验收流程；不自动改已有Goal。
  - 不把正常执行取消当作Goal编辑/历史删除而禁止既有cancel_run；不裁剪Framework/SDK合理公开API，不改TaskRun→PlanTask→SubagentRun权威、依赖锁/风险合同或其它UI切片。
  - 多Steer编辑/排序/立即执行、Goal
    composer视觉与完整Tauri体验仍按原Plan12验收；任务列表高数据量按Plan13验收。不以撤销旧方案为完成新结果；Git仅主协调者按已有授权保存，不push/merge/发布/部署。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 运行中的追加输入（Steer）
    digest: sha256:d461e3bdbe7a126228e65a72069ca7dc201882caedd2a5a093b6236cd374ccf1
  - ref: design.md § 唯一 Goal、用户修改权与原文恢复
    digest: sha256:9ef2dc8f4bdfaa1d9e11f9927c5ef350ff749a4b45fa4cfb308244ee77d02893
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#steer-context-and-goal-authority
todos:
  - id: prove-main-model-steer-and-goal-boundaries
    summary: 沿已有输入/任务消费者证明主模型理解Steer与Goal初始化/用户修改边界，只修具名违约。
    files:
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs
      - echo-agent-cli/src/tauri/commands/task_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx
    acceptance:
      - 复用现ensure_scope/get_run、幂等create_run和已有用户RunGoalActorSource；已有Run不覆盖、task_update无Goal字段，普通接纳/record_run_steer只记原输入事实。取得原值并检查goal/revision/hash稳定，不为验证引入模型/状态权威。
      - 以真实GUI更新入口和IM
        /task-goal原CAS为用户正控制；模型/普通Steer无写入入口为负控制；重试/并发/冷重开及旧回执不得覆盖已存在Goal。
      - 复用request_cancel/RunCancelled与已有GoalBar消费者，证明取消仅结算执行/隐藏活动呈现且保留Goal三字段和历史；用户结束失败/未结算与物理终止分别记录。
  - id: qualify-full-source-and-recoverable-context
    summary: 对账并补齐主模型取得完整原文和必要历史的压缩/恢复资格，保留原关键失败暂停路径。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/docs/zh/adr/0037-unified-turn-run-binding.md
      - echo-agent-cli/docs/zh/adr/0048-context-compression-product-policy.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
    acceptance:
      - 先对账原完整原文、引用/read_artifact和required_context_unavailable已交付切片，仅对源变化或缺场景追加验证，不重复全仓调查/全量冷编译。
      - 真实压缩/最终模型请求和冷重开读取完整长原文，覆盖十余条输入后的早期限制和长消息末尾；不以最近4/8条预览证明全集，也不新增独立解释/有效要求流水线。
      - Goal/必要原文/范围读取失败沿原准入阻止相关新副作用；正式PlanTask与普通GUI/IM各保原物理结算、Goal和恢复状态，修复后原expected-resume继续，工具正上界不放宽。
      - 中文ADR0037/0048及运行时同步新裁决，原成功/失败及证据范围保留；examples/生成类型未变时说明不适用，官网待所属精确源同步，不新建英文副本。
artifact_id: plan:12b8ff3d-31c9-4d2a-8fa9-3cbc8ef60760
lifecycle: ready
---
## Notes

本结果取代已撤销的独立理解/强制更新Outcome，保持原文、幂等与恢复成果。当前源调查尚未发现普通Steer改Goal、幂等初始化覆盖Goal或执行取消删Goal的产品反例；先补/复用消费者证据，不因计划存在而制造实现改动。若发现真实违约，再按已确认规则修复。当前编号由磁盘最大编号计算，不表示优先级；执行必须传本文件精确路径。
