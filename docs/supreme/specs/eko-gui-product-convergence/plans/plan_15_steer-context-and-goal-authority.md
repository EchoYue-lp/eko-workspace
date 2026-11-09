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
    - 主模型在有界capsule中发现准确Run/journal
      identity及序号切点的原始输入历史入口，沿现有read_artifact分页读取十三条以上输入前的早期限制与完整长原文尾部；真实压缩/冷重开、跨页/并发追加/不同Run或来源代际/旧cursor均不混读。派生缓存可从原journal重建，必要源/校验/最终reader资格缺失沿原required_context_unavailable暂停相关新副作用并保原owner和expected-resume。
    - 真实消费者、原生与历史切片逐项对账，只修证实的缺口，同源证据可复用；中文ADR/运行时、examples/官网适用性及所属仓门禁明确，focused不替代完整/native/main
      CI资格。
out_of_scope:
  - 不调用独立模型理解意图，不新增结构化EffectiveRequirement解释协议、要求store、分类器、GoalStore、actor权限层或统一强制要求/计划/验收流程；不自动改已有Goal。
  - 不把正常执行取消当作Goal编辑/历史删除而禁止既有cancel_run；不裁剪Framework/SDK合理公开API，不改TaskRun→PlanTask→SubagentRun权威、依赖锁/风险合同或其它UI切片。
  - 多Steer编辑/排序/立即执行、Goal
    composer视觉与完整Tauri体验仍按原Plan12验收；任务列表高数据量按Plan13验收。不以撤销旧方案为完成新结果；Git仅主协调者按已有授权保存，不push/merge/发布/部署。
  - 原文历史只是同journal派生来源族和不可变文件入口，不新增输入/要求store、语义去重/分类器、模型工具或Framework公开API，不把源记录数当有效要求分母；分页/token参数沿原read_artifact，不另造产品阈值。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 运行中的追加输入（Steer）
    digest: sha256:d461e3bdbe7a126228e65a72069ca7dc201882caedd2a5a093b6236cd374ccf1
  - ref: design.md § 唯一 Goal、用户修改权与原文恢复
    digest: sha256:735f5f8f2b4579dfbdffbf31cbb19ef5ac458ec832084ebcbaa6542f0588ceaa
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
    summary: 沿原journal派生读模型和read_artifact补齐全部原文的模型可发现入口，验证固定来源快照、压缩及失败恢复。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/history_projection.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/run_authority.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/docs/zh/adr/0037-unified-turn-run-binding.md
      - echo-agent-cli/docs/zh/adr/0048-context-compression-product-policy.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
    acceptance:
      - 先保留当前真实缺口：原journal十三条全文可冷恢复，但capsule近期4/状态近期8不提供早期全文的最终模型可发现入口。新增回归用原已有API写十三条输入、冷重开和对应真实invocation历史/旧执行输出，触发既有真实压缩；当前输入仅“继续”，在实际主模型request上取得旧生产缺可读入口的自然行为失败。不能把新API缺失、fixture.first误改、重新注入旧正文或前置reader校验失败冒充该RED；单条当前长PreparedUserTurn绿色不替代历史组合。
      - 复用HistoryProjection现有apply/rebuild/段校验，扩原始输入来源族及run_authority/file_shadow薄readview；复用recorded_user_steer_artifact→materialize_recorded_user_text→InputResourceRef和原read_artifact，prepared_turn现实现够用时不改。已有artifact/review族保留，不新增第二来源权威、工具或公开框架接口。
      - 在原authority/产品数据lease下捕获准确journal
        identity和RunStateSnapshot.journal_sequence切点；现query_events_bounded按既有256事件批次读取，调用侧只收seq<=cut并继续到完整cut，不拿结果limit一页当全集，不把后继append混入旧快照。暖projection及重建均需核准确来源/through_sequence/段内容；同run_id/head但journal换代不能复用旧来源。
        原文文件以来源族最后源事件序号/内容身份稳定发布；无新输入的普通工具事件推进全局through_sequence时不因head/时钟重写原文文件，capsule可记录观察切点但缓存必须证明同源覆盖。
      - RunInputAccepted沿原(address,input_id,revision,payload_sha)校验及幂等；legacy
        RunSteerRecorded按journal
        identity/seq保其实际原文事实。合法相同文本或turn不被语义去重，接纳/投递分别表达、分别计源记录，不伪造input总数或Applied；不自动改Goal/计划/验收。
        原初始输入与用户更新后的canonical Goal允许不同，按原输入负载身份验证，不拿新Goal文字/SHA否定旧原文。
      - 将源事实及全文或不可变原文引用物化为一个现有InputResourceRef入口，capsule只给有界可发现引用；快照按来源/cut/字节SHA不可变发布，重复刷新同字节保inode/mtime，新增事件产生新ref。GREEN中的path/SHA必须来自实际模型request已发布引用，真实read_artifact工具结果按实际call_id进入下一模型请求并覆盖首条全文/末尾；不把原文重新作为当前消息注入。跨页、大于一个事件扫描批次、旧cursor/并发append/多Run和工作空间换代有正反控制，不另造token/page上限、不覆盖共享旧快照。
      - 冷重开及派生索引/原文缓存删除或损坏从准确journal重建并复核同源字节；源不可读、SHA失配或最终真实工具范围无read_artifact沿原required_context_unavailable暂停。正式PlanTask与普通GUI/IM消费者保原owner、Goal及物理结算，原expected-resume可恢复；不得降空历史、放宽allowed_tools正上界或重放未知副作用。
        同内容无损刷新保旧cursor；损坏重建若文件身份变化，旧cursor明确拒绝并无cursor重读。路径固定本机产品data-root，SSH项目FileSystem/read_file不调用；保原only-read_file负控制零模型/零远程探针且Pause，修正原reader资格后沿expected-resume恢复。
      - 中文ADR0037/0048及运行时说明同步原文入口、源快照/派生缓存与主模型理解边界；原成功/失败和证据范围保留。examples/生成类型未变时说明不适用，官网待所属精确源同步，不新建英文副本。
artifact_id: plan:12b8ff3d-31c9-4d2a-8fa9-3cbc8ef60760
lifecycle: ready
---
## Notes

本结果取代已撤销的独立理解/强制更新Outcome，保持原文、幂等与恢复成果。当前源调查尚未发现普通Steer改Goal、幂等初始化覆盖Goal或执行取消删Goal的产品反例；先补/复用消费者证据，不因计划存在而制造实现改动。若发现真实违约，再按已确认规则修复。当前编号由磁盘最大编号计算，不表示优先级；执行必须传本文件精确路径。

2026-10-27原文恢复细化仍属本Outcome：已确认十三条后早期原文缺少最终模型可发现入口，需要补同journal派生历史来源族与不可变引用；这不是新增独立意图理解方案。现HistoryProjection只含artifact/review，raw来源族为本次待实现扩展，不称已有API。正式设计只约束来源、数据流与失败验收，具体token/page常量沿原reader与实际实现，不在计划另造限额。原官方参考原件在CLI `.echo-semantic/local/steer-goal-validation-20261027/raw-history-references/receipt.json`。
