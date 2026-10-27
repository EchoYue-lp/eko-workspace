---
slug: native-subagent-depth-repair/run-attempt-isolation
outcome:
  summary: 同一主 Agent 中不同 TaskRun 可复用 task/attempt 并独立执行，下一次尝试指导按同一 scope/Run 隔离且只消费一次。
  acceptance:
    - 共享真实 primary/control registry 的两个独立 Run 使用相同 task、revision、attempt、不同
      claim/execution；先后 settled 与同时 active 均可独立开始/结算，同 Run 重复 attempt 仍拒绝。
    - 两个 Run 的原 NextAttempt journal 经真实应用 transfer、原 scoped queue 和实际 admission
      各消费自己的 guidance 一次；scope/run 隔离、旧 revision、重复 command、空身份及原 unscoped
      正控制有具名红绿证据。
    - readonly 与 writer 有／无附件三条 EKO 真实分支共同使用正确 Run identity，原消息、interrupt、物理
      settle 与有界去重不改变；源码/正式文档与 focused 资格经独审和原正常门禁形成可恢复本地修订，不冒 main/CI 或原生深度3通过。
out_of_scope:
  - 不实施后继只读委派实例、三 carrier 的 tool_access、readonly 工厂深度修复或新原生切换；这些仍属于交付图
    readonly-depth-enforcement，不能据本结果宣称已完成。
  - 不改 zstd 的456既有差异、品牌源、依赖锁/风险合同、SDK
    pin/accepted、原native/fixture或模型；不修改外部校验工具，不push/merge/关闭Issue/发布/部署。
  - 不新建control registry、队列、store、executor或身份类型，不用随机task
    ID、清账、修改retention/等待/栈/深度来消除碰撞。
design_ref: docs/supreme/specs/native-subagent-depth-repair/design.md
design_sections:
  - ref: design.md § 问题与目标
    digest: sha256:c6d44aa92febbb7a327467fad423d91bcd2573229a3d157943a5395983344d7c
  - ref: design.md § 目标行为与范围
    digest: sha256:e5fb5b07fc65dff37f6a46d15016114e9290e5fb23f0031b06df401eaf4f26ca
  - ref: design.md § 系统边界与复用依据
    digest: sha256:9d3e0d97588180c58dccf47df7abed9999d4b422dc39f8db887ef8a8f31f3c0e
  - ref: design.md § 运行身份与指导数据流
    digest: sha256:37a88f39f064d180d4b10d518d1fa8ae7a49fab04dfb50fae4b517e88ef41ea3
  - ref: design.md § 异常、并发与恢复
    digest: sha256:04da2f40f7ddc57e96794cdf3a16d06bfc679e1b107b6297e70feac79ca25e91
  - ref: design.md § 验收标准
    digest: sha256:a44eaa75a449834b3bb6c4c17c3ba97936b04b1505cf68932ad649cf07320cc8
  - ref: design.md § 现有设计与交付边界
    digest: sha256:88fdfee5fc480c5e5dbd8852d646f6383cbd4a4b9e5d791bc3b0bc5ec6c23b70
delivery_ref: docs/supreme/specs/native-subagent-depth-repair/plans/delivery-map.md#run-attempt-isolation
todos:
  - id: retain-run-attempt-identity
    summary: EKO 从同一个 durable control identity 投影保留 Run 的 Framework attempt identity。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/tests.rs
    acceptance:
      - 新增回归先调用旧 helper 与同一真实 registry，取得不同 Run 同 task/attempt 误拒的自然断言失败；修复只复用现有
        identity.attempt_identity()/for_run，不伪造 command/execution，不把缺 API
        编译错误当行为红。
      - 同主 Agent 的两个 Run 在首个真正 settle 后或同时 active 时互不阻断；同 Run 重复逻辑 attempt/错误
        exact execution 继续拒绝；三条现有 readonly/writer text/message 分支的消费者断言均保留。
  - id: carry-guidance-on-original-scoped-queue
    summary: 为原 guidance queue 追加 scoped 入口，应用转交使用同一 Run/scope 投影。
    files:
      - echo-agent/src/agent/subagent/control.rs
      - echo-agent/src/agent/subagent/executor.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs
    acceptance:
      - 原 queue 与追加 queue_guidance_scoped 共用同一私有 enqueue、mutex、active/settled
        检查及 queued_guidance 表；legacy None/None 键、原签名和错误类型不变。
      - 原两个 Run journal 各排不同指导，真实 transfer→queue→admission 后只各自消费一次；单独变化 scope 或
        Run 的正反控制、空身份/attempt0/空instruction、重复投递及旧revision明确拒绝或沿原幂等回执，不串流。
  - id: qualify-control-consumers-and-source
    summary: 冻结两仓真实输入，验证原控制消费者并同步所属正式说明，形成独立本地交付。
    files:
      - echo-agent/src/agent/subagent/control.rs
      - echo-agent/src/agent/subagent/executor.rs
      - echo-agent/docs/zh/06-subagent.md
      - echo-agent/docs/adr/0058-task-claim-subagent-attempt-control.md
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/tests.rs
      - echo-agent-cli/docs/zh/adr/0013-subagent-control-receipt-projection.md
    acceptance:
      - 原 live-message、interrupt/settlement、guidance transfer 与三条 Fork
        消费者相关测试、fmt/check
        和适用示例编译在实际同源快照通过；新回归先红后绿，原始日志/自然出口/实际选中与忽略数保存。完整合并门禁、公共API条件矩阵与main/CI按既有规则独立报告，不用focused替代。
      - Framework ADR0058与原子智能体说明、EKO ADR0013用中文补充本次 scope/Run 行为；现有
        demo61_agent_factory 与 lh6_product_soak
        调用签名无需改，纳入适用编译链。官网镜像由原owner待源稳定后同步，本任务不手改品牌或源镜像；SDK pin/accepted不动。
      - 两仓按正常独立task合同保护既有456/品牌差异；Root唯一stage/commit，本批实际结束后保全日志/必要制品并确认准确target无reader，再及时cargo
        clean并记录回读。
schema_version: 4
artifact_id: plan:42888472-07db-434b-bcd0-2b2167a55882
lifecycle: ready
---
## Notes

本 Plan 只交付 Run/attempt 与 guidance 隔离。先在旧 EKO helper/原 registry 上固定真实行为红，再追加 Framework 原 queue scoped 入口，冻结该机制后整合 EKO；Framework 与 EKO 可以在同一协调窗口内串接，避免无必要的重复冷编译。每批 jobs≤2、dev/test debug0、空间至少5GiB，使用主根内准确自有 target，日志和制品保在 target 外。不得为运行本 Plan 复用或覆盖 zstd、品牌和 Root 状态文档的预检 owner。

Scope/Run 隔离修复可以在 zstd 既有改动作为受保护差异时独立开始，不依赖它的 source402 工具合同或正式提交。后继只读委派的产品结果仍在 Delivery Map；仅在本结果依赖取得实际交付证据后，为该结果创建下一份 Plan。
