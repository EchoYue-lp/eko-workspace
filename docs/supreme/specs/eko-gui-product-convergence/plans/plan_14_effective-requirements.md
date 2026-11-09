---
slug: eko-gui-product-convergence/effective-requirements
outcome:
  summary: 在同一 TaskRuntime
    中持久保存完整原始要求及来源、范围、版本和替换关系，幂等连接输入接纳、上下文消费与目标/计划/证据更新，压缩和冷恢复保留仍有效约束，关键上下文缺失时暂停相关副作用，并以独立原始要求分母检查工作项、交付物及验证证据的覆盖。
  acceptance:
    - 完整原文、来源、作用范围、版本和替换关系沿原journal冷恢复；十余条输入后的最早有效约束及长消息末尾限制在真实压缩后的模型请求中仍可读取，不以最近4/8条或截断预览替代权威。
    - 接纳持久保存、进入上下文、应用到目标/约束/计划、取消/替换/失败分别可追踪，稳定输入ID消费幂等；关键持久化或Goal/约束/范围读取失败不能继续相关副作用，旧投影须证明版本匹配才可复用。
    - 目标/要求变更沿已有安全点及Goal/Plan
      revision使受影响证据失效；旧Subagent晚回不冒当前完成。独立原始要求分母检出计划遗漏、未验证和过期证据，程序检查与语义审查分别成立。
    - 分阶段反例、正常控制、冷重开、真实压缩、独立审查和适用完整门禁完成；原文首切片通过只结算其范围。正式ADR/双语文档、可执行examples/官网适用性有对账，native/真实服务/main
      CI缺项仍未完成。
out_of_scope:
  - 不新建要求/任务/Goal store、平行调度器、第二套DAG/完成状态机或SQLite；不把所有Steer永久化，不强迫问答/短实现建立复杂计划。
  - composer多Steer与手动唯一Goal控制由Plan12交付；本结果沿原输入身份和TaskRuntime接线，不接管GUI独占六路径、中央Hook/Hook
    tests或Store/API，也不在验证期间修改其冻结Rust输入。
  - 不改Exa
    hold、依赖风险策略、外部echo-semantic工具合同，不扩大SDK公共API或权限门控；本地里程碑提交由主线程唯一协调Git暂存/提交，本线不自行stage或commit，push/merge、关闭Issue、发布或部署仍禁止。
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
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#effective-requirements
todos:
  - id: retain-full-source-and-idempotent-ingress
    summary: 保留完整原文和稳定来源，闭合接纳与原任务journal记录的失败和幂等边界。
    files:
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/conversation_input.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_event_log/
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
    acceptance:
      - 先核record_run_steer、RunSteerRecorded、已有objective
        artifact和输入journal真实读写；复用已有原文/引用，去除journal前take2000永久丢失，展示预览仍有界且UTF-8安全。
      - 长消息尾部限制、原输入ID/地址/来源冷重开后可校验，历史不因event_rebuild仅留8条而消失；渠道account/generation/context与恢复意图不丢。
      - 持久化失败及进程退出交错不能出现已声称接纳并继续相关副作用、要求记录却丢失；同ID重试不重复应用，短输入/附件正常控制保留。
      - 首切片用真实冷重开与原压缩路径验证原文可读，只结算本Todo，不宣称来源/范围/替换/独立覆盖已全部实现。
  - id: project-current-effective-requirements
    summary: 维护有效要求及范围/替换关系，使真实压缩和恢复不丢持续约束。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs
    acceptance:
      - 沿同一TaskRuntime区分临时引导、持续约束、目标变更、状态问答；默认当前任务，明确会话持续要求才扩范围。完整来源/版本/显式替换可重建，不把全部输入永久化。
      - 解除约束只使被替换项失效；十余条输入后最早仍有效要求保留，不以compact capsule最近4条代表全部要求。
      - 走真实压缩/上下文重建入口核最终模型请求和完整原文读取，不仅断言新helper字符串；长消息尾部、跨任务/会话范围、取消替换和冷恢复有反例与正常控制。
  - id: stop-on-missing-critical-context
    summary: 在原执行入口重读和核版本，阻止关键要求缺失时继续相关副作用。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/
      - echo-agent/src/agent/react/run/
    acceptance:
      - 沿现有projector/admission核.ok/unwrap_or降为空路径；应用在原可失败准入/安全点处理关键读取失败，框架只承接通用机制，adapter不复制循环。
      - Goal、有效约束或范围读取失败时重读并核当前revision，仍无法恢复则暂停相关副作用并报告缺失项；辅助信息可降级，旧投影须证明版本匹配。
      - 故障注入沿直接执行、具名Subagent和恢复路径证明真实工具副作用未启动；已启动操作保留物理结算，逐路径记实际覆盖，不能用一个helper概括全系统。
  - id: bind-revisions-and-audit-original-coverage
    summary: 连接要求和Goal/Plan/证据版本，以独立原始要求分母检查遗漏并验收整条链。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/completion_gate.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/adr/
      - echo-agent-cli/docs/zh/adr/
    acceptance:
      - 复用update_run_goal、原安全点与goal/plan
        revision，应用成功才记录应用事实；变更使旧Plan/GoalRequirement/受影响证据失效，未受影响证据经范围/版本重验可复用。
      - 原始Goal/有效要求不再只由requirements_for_plan的PlanTask生成；用户要求A/B/C而计划仅A/B时，即使所有PlanTask完成，门禁仍指出C缺工作项/交付物/证据。
      - 未验证、失效证据、显式替换、Goal变更后旧Subagent晚回与用户具名skip分别核验；程序检查命令/hash/版本，语义审查不能代替客观证据。
      - 前三阶段串成接纳→真实压缩→冷恢复→变更→覆盖→完成的实际路径验证并独审；ADR/双语文档同步，examples/官网记录同步或不适用，保留原失败并分列focused/完整/native/精确main
        CI。
schema_version: 4
artifact_id: plan:638b311e-e284-4489-814f-d0bc01e93f5a
lifecycle: abandoned
---
## Notes

2026-10-27 用户明确撤销独立意图理解及统一强制“有效要求更新 → 计划调整 → 验收调整”方案；普通 Steer 交主模型结合历史处理，已有 Goal 仅用户显式修改/结束。**本 Plan 的原 Outcome 不再交付，仅保留原文和具名历史证据，不是当前执行来源，也不表示完成。** 完整原文、输入幂等、必要上下文压缩/冷恢复及用户 Goal 权限继续由当前交付图中的新结果承接。原 design_sections 与 delivery_ref 保留当时身份，不刷新摘要伪装仍适配当前设计；历史结构可解析，当前设计/交付绑定预期失配。


本计划承接已批准完整Goal及2026-10-26四线持续授权。原文take2000、event_rebuild最近8条、compact最近4条、projector读取降空及PlanTask反推要求为核心线已定位的输入；修改前核最新调用点。先交付完整原文、冷重开与真实压缩切片，剩余来源/范围/替换、关键失败和独立覆盖仍属于本Outcome，不得移出以关闭整个Goal。

epoch08的queue3/3与P1 stack overflow/cargo101保留原历史，准确target已清理。后继核心epoch10的2f7d generation产品修复已使原P1/P2/N1/N2分别通过，但长reflect默认栈仍失败，原文冷重开与有效要求真实压缩两项原失败仍开放；本Plan完整Outcome未完成。当前精确源码/原13执行、GUI窗口与资源交接以next-actions及各实际回执为准，不沿用旧epoch09冻结窗口启动新批次。通用安全点/revision归框架，要求作用域与EKO证据策略归应用；Plan12/14的files不授予GUI或本线Hook/Store/API等其它writer权限。用户现允许且要求及时本地里程碑commit，Git暂存/提交由主线程唯一协调，本线不自行操作；push/merge/Issue关闭仍禁止。
