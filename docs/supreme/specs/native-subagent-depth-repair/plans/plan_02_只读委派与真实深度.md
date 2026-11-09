---
slug: native-subagent-depth-repair/readonly-depth-enforcement
outcome:
  summary: 真实内置只读 reviewer 按运行绑定恰达 1／2／3 层，所有后代保留实际只读与父工具上界，并完成同 conversation 双
    Run 的新原生验收。
  acceptance:
    - 同一调用级只读上界沿原
      AgentInvocationContext、ToolContext、SubagentContext、Sync/Fork/Teammate/Team/后台路径传递，锁后
      snapshot 以现 catalog、父正向工具上界 W 和有效只读能力求交；模型 schema、tool_search 与实际执行不能互相绕过。
    - 原内置 reviewer 经过真实 loader/readonly factory、TaskRuntime 与原
      Fork，在绑定上限1/2/3时分别恰到对应层数；下一层 factory、model 与工具调用为零，lineage
      与真实请求一致。变更默认值不重写旧 Run，Framework 合法 max4、None/unscoped 和普通 writer 对照保留。
    - 受限委派只准入原同一不可变 registration 的只读目标，并逐一核 Team 的实际
      manager/member；Fresh/access-only、writer/错误只读标签、同名替换、排队后注册、空交集、缺
      ToolContext/visibility、子请求Mutating和默认 Agent 回退负控制均不能开始写副作用。
    - 旧真实 readonly 工厂和原消费路径先取得行为断言失败，再在修复后取得同源红绿、保留契约和示例编译证据；协调者在精确新版
      Tauri/前端指纹与恢复材料上，使用同一固定 task ID 完成同 conversation 双 Run
      及层数1/3原生验收。未取得原生或完整门禁时不将此 Outcome 标完成。
out_of_scope:
  - 不重做已交付 I1 的 identity/guidance 权威，不新增
    registry、store、mailbox、executor、状态或第二套权限策略；不改等待、栈、超时、工具预算或产品层数默认，不以随机task
    ID/清settled账/放宽能力规避失败。
  - 不把普通 AgentDispatchTool::new 或全部 agent_tool 全局标只读；不移除真实 readonly 工厂的
    .readonly_tools()，不把用户直接操作终端/MCP 加入 Agent 自动权限门控。
  - 不改 zstd456 候选、依赖锁/风险接受、SDK pin/accepted、A2A/LSP/终端退役、品牌或外部 echo-semantic
    工具合同；不因本片公开字段追加自动迁移独立SDK或扩大接受集。
  - 不在实现者自有进程中擅自重启/切换原生、重写旧模型夹具或用户数据。新制品准备、维护切换、人工输入和原生收据由主协调者按当时真实状态排窗；不push/merge/发布/部署或晋升未push
    gitlink。
design_ref: docs/supreme/specs/native-subagent-depth-repair/design.md
design_sections:
  - ref: design.md § 问题与目标
    digest: sha256:9b7bcd510cf1cd7847ce8b6aeab18288d71c17778ce2c25e36f2af1e7879abf8
  - ref: design.md § 目标行为与范围
    digest: sha256:e5fb5b07fc65dff37f6a46d15016114e9290e5fb23f0031b06df401eaf4f26ca
  - ref: design.md § 系统边界与复用依据
    digest: sha256:9d3e0d97588180c58dccf47df7abed9999d4b422dc39f8db887ef8a8f31f3c0e
  - ref: design.md § 只读委派与工具数据流
    digest: sha256:352b64e8c8dedf64592d46de4d205c94cad2d359c8dee78916df2d15d7c33acd
  - ref: design.md § 异常、并发与恢复
    digest: sha256:04da2f40f7ddc57e96794cdf3a16d06bfc679e1b107b6297e70feac79ca25e91
  - ref: design.md § 验收标准
    digest: sha256:a44eaa75a449834b3bb6c4c17c3ba97936b04b1505cf68932ad649cf07320cc8
  - ref: design.md § 现有设计与交付边界
    digest: sha256:88fdfee5fc480c5e5dbd8852d646f6383cbd4a4b9e5d791bc3b0bc5ec6c23b70
delivery_ref: docs/supreme/specs/native-subagent-depth-repair/plans/delivery-map.md#readonly-depth-enforcement
todos:
  - id: retain-readonly-in-original-invocation-carriers
    summary: 让同一可选只读上界在原调用载体、Fresh/空上下文和默认Agent边界内完整保留且只能收窄。
    files:
      - echo-agent/echo-core/src/agent/mod.rs
      - echo-agent/echo-core/src/tools/mod.rs
      - echo-agent/src/agent/subagent/context.rs
      - echo-agent/src/agent/subagent/executor.rs
    acceptance:
      - 复用既有ToolAccess，在原三个carrier携带Option值，不增加ExternalRunContext/Run
        profile/事件wire平行字段；仅上界的上下文也保留，None旧路径不变，祖先ReadOnly不被后继Mutating扩大。
      - 默认Agent的三个invocation回退收到新ReadOnly时在原execute/chat开始前明确不支持，底层调用计数为零；ReactAgent实际override、Arc与EKO适配仍整体转交原值。完整literal按实际必要性适配，不机械修改使用Default的调用方。
  - id: restrict-dispatch-and-enforce-original-tool-snapshot
    summary: 原dispatcher选择受限只读实例，并在原registration准入与锁后工具快照执行同一只读及父工具上界。
    files:
      - echo-agent/src/tools/builtin/agent_dispatch.rs
      - echo-agent/src/agent/subagent/registry.rs
      - echo-agent/src/agent/subagent/executor.rs
      - echo-agent/src/agent/react/mod.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/src/agent/react/run/pipeline.rs
    acceptance:
      - 原装配根据readonly_tools与dispatch注册组合选择受限实例；只有受限实例risk/capabilities为只读，普通new保持Mutating，catalog只展示允许目标；实际调用检查与执行使用同一resolved
        registration，Team的各实际成员分别准入。
      - 受限调用必须持有真实ToolContext及有效visibility，从不可变available集合继承W，原inherit_tool_scope拒空交集不回退无限制。Sync/Fork/Teammate/Team/后台均携带约束，Fresh或无history/goal不丢上界。
      - snapshot在原执行锁后统一算config.readonly_tools与调用ReadOnly的有效值，沿原readonly_tools/plan/visibility过滤schema、search及强制调用；排队后新工具、同名Mutating替换或错误只读角色标签不获得写副作用，原disabled/model/skill/depth限制继续收窄。
  - id: qualify-real-readonly-factory-and-depth-consumers
    summary: 通过真实EKO readonly工厂和原任务消费者证明受限委派恰达运行层数，补齐必要公开字段字面量。
    files:
      - echo-agent-cli/echo-agent-app-core/src/infra/factory.rs
      - echo-agent-cli/echo-agent-app-core/src/infra/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/events.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/unattended.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
      - echo-agent-cli/echo-agent-app-core/src/project/agent_adapter.rs
    acceptance:
      - 先经实际builtin
        reviewer定义、register_default_subagents及build_readonly_subagent_agent取得旧生产工具曝光/执行的行为RED；不得以手工造未开启readonly的child或缺新字段编译错误替代。保留原readonly、can_delegate、模型代际、workspace/资源与薄factory路径。
      - 绑定1/2/3的同源测试核真实schema/ToolContext/lineage/调用计数及下一层零准入；writer、附件、有/无硬上界、默认变更后的旧Run及Framework
        max4正控均保留，不能只核设置保存或直接for_run夹具。
      - chat_driver和正式任务foreground/unattended完整AgentInvocationContext字面量补同义None；原Default调用与project
        adapter转交按实际API编译和字段级断言验证，无新增产品配置或角色名单。
  - id: close-retained-contracts-documents-and-exact-native-evidence
    summary: 在同源快照闭合保留合同、中文说明与适用示例，并由协调者完成具名新版原生双Run验收。
    files:
      - echo-agent/docs/adr/0094-subagent-tool-boundary-inheritance.md
      - echo-agent/docs/zh/06-subagent.md
      - echo-agent-cli/docs/zh/adr/0052-gui-subagent-depth-policy.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent/echo-agent-learning/examples/demo61_agent_factory.rs
      - echo-agent-cli/examples/lh6_product_soak.rs
    acceptance:
      - Framework ADR0094与Subagent说明、EKO
        ADR0052/runtime用中文解释同一调用上界/默认回退/真实factory及保留限制；示例未改签名时说明原因并纳适用编译，lh6只做编译不意外启动长soak；官网由源稳定后的原owner同步，SDK独立pin/合同资格不继承。
      - 正式focused/公共API条件矩阵、合并完整门禁、新GUI
        build与mainCI按各自规则报告；独立审查、正常source/staged/final和可恢复本地提交覆盖实际diff。每批按冻结输入分阶段，保存红绿日志/自然exit/选中与忽略数及必要制品，确认准确target无reader后及时clean。
      - 协调者具名保全新版binary/完整frontend/assets/hash与恢复包后按当时维护授权切换；使用同一native-depth-probe在同conversation两个Run均成功，层数1/3及下一层零派发、原HELLO/HITL保留明确。未开始/失败/需人工输入如实开放，不拿模型夹具文本、前端build或旧epoch16替代实际IPC/journal证据。
schema_version: 4
artifact_id: plan:542d2e91-1c4f-4337-9029-6421e5660b10
lifecycle: ready
---
## Notes

I1 机制与消费者前置已由主协调者具名交付：Framework `0e404185c8f252da3e335c35547e52249c74dc65`、EKO `0ebd8984be26affc32a130df7a26b128494d6c65`，旧4项真实RED、34项GREEN、2个示例编译及独立审查、正常source/staged/final均有原件。本Plan不据此自动修改Plan01生命周期，也不把局部I1资格扩为I2或整体Goal完成。

I1准确target已经清理；原清理前reader检查因collector顺序异常未完成的工程残余与原错误继续保留，不能补造清理前事实，也不把该已不存在的target当I2功能依赖。I2使用其实际新批次的冻结、资源和恢复证据；focused并发、jobs及debug上限遵循当前仓库规则，旧I1实际jobs2不构成后继长期上限。新原生维护许可、风险接受和进程句柄均不从旧epoch自动迁移。
