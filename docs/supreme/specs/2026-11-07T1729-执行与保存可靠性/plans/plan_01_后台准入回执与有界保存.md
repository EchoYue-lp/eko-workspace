---
schema_version: 4
slug: 2026-11-07T1729-执行与保存可靠性/plan_01
outcome:
  summary: 普通Chat和TaskRun的后台命令在接纳前可取消、接纳后回执可追踪，由原cell管理排队执行，并在原保存预算与恢复合同下可靠结算代表性大上下文。
  acceptance:
    - shell占槽及tracked capacity饱和下，未接纳取消无执行，已接纳请求取得/找回唯一原cell
      receipt；排队、超时、停止和App关闭均由原supervisor结算且无资源泄漏。
    - 真实9.5MB同形样本及多scope受控数据在原10秒保存预算内通过声明负载下的冷/暖分段验收；原CAS、pending
      debt、unknown、owner-first恢复、raw/approved原件与scope隔离不变。未达标保留阻塞，不加预算作绿。
    - exit0/101、queue
      deadline、cancel、spawn/持久化失败各有真实结论；调用方超时不丢已接纳任务，原微信业务测试101与本基础设施回归分列。
    - 最终两仓冻结快照的直接与公开合同验证、examples及AGENTS适用门禁通过；框架先于应用集成，真实App、main
      CI和发布分别验收，已知全量失败不得绕过。
out_of_scope:
  - 不自动重跑原用户命令、恢复/写入原数据或对外发送；App验收、提交、推送和发布必须另获准确授权，受控fixture与隔离样本不冒充生产验收。
  - 不提高shell配额、不统一免除ToolManager超时、不自动增加保存预算、不吞错误或重放副作用。
  - 不新增应用命令队列/进程supervisor/Outbox/SQLite，不删除raw/approved增量或把_scope_index当authority。
  - 不在本计划修原微信channel_runtime业务断言；其已知101由原微信恢复计划单独处理，在全量合流前仍是必须解决的真实阻塞。
design_ref: docs/supreme/specs/2026-11-07T1729-执行与保存可靠性/design.md
design_sections:
  - ref: design.md § 审查结论与证据
    digest: sha256:bec375d0461c02bd99366d520d2e588566c60db7d107ad58cef09d93b2c94983
  - ref: design.md § 目标与范围
    digest: sha256:0a27e09d5b1f5c74cd43f68f5b5f950c5663de7a0f2f9ebadace96cf93e3a3d3
  - ref: design.md § 分层与复用
    digest: sha256:be92d878a482a0f9f74353dc00f60481168df347f0b077517a4c994d06ab680a
  - ref: design.md § 目标行为
    digest: sha256:72a6dd0ff8926a4a1ee3d35ddd0caad61f8af71f557fd9045884230eb1cf73e1
  - ref: design.md § 参考实现与候选取舍
    digest: sha256:0236291a977de5c2fe43c6232f3da3ac5e3784696b0231cdbfa8f1abdfbe2490
  - ref: design.md § 验收标准
    digest: sha256:90043c2533e422b2b350b42ee70b3f00ee4401b65059a76583c54fdc2afa5b00
  - ref: design.md § 协作、文档与交付边界
    digest: sha256:e6348e801572d273fbcf56abb1bb406703f1e6297321df9a750a4f5f6e0081f3
delivery_ref: null
todos:
  - id: reproduce-admission-and-profile-settlement
    summary: 先闭合两个准入反例与保存分段证据，固定真实失败而不推测CAS耗时。
    files:
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/echo-orchestration/src/tasks/command_cell.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/src/state/file.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/command_cells.rs
    acceptance:
      - 逐项核对当前相关字节与原App
        inputs.json，保留prepared16:08:47/调用timeout16:11:30/后台exit10116:22:15及原测试错误，不把prepared当真实spawn时间。
      - 沿生产服务的受控shell占槽和tracked-capacity饱和路径取得RED：取消/超时后是否仍创建任务、receipt是否被覆盖、原owner是否继续；普通Chat与run-owned均覆盖。
      - 隔离只读样本或合成同形数据；记录epoch/load/validate/freeze/project/serialize/CAS/apply/ack各段单调耗时、共享剩余预算、read/parse/clone次数及物理写状态，不输出正文/凭据。
  - id: separate-admission-and-cell-lifetime
    summary: 在原cell manager建立准入取消与后台生命周期的明确交接，并在原Queued阶段管理产品容量。
    files:
      - echo-agent/echo-core/src/tools/cell.rs
      - echo-agent/echo-orchestration/src/tasks/command_cell.rs
      - echo-agent/echo-tools/src/shell.rs
      - echo-agent/echo-execution/src/tools.rs
    acceptance:
      - 盘点原trait/调用方后加入最小通用admission context和异步资源准入；tracked
        capacity、manager锁和prepared交接之前服从调用预算/取消，释放容量后旧调用不创建进程。
      - 原start_prepared确认Queued且supervisor持有owner后及时回receipt；runner自身等待产品/内部许可，原cell
        deadline和显式cancel独立，不在应用另起队列。
      - 接纳后取消与交付竞争保留准确cell/receipt，same canonical call至多一个cell；不能用command
        hash禁止用户有意的重复命令；普通前台timeout不放宽。
      - 继承accepted_at+timeout包含排队的语义；queued取消/到期无spawn，执行后物理结算才释放许可与原guard，capacity/shutdown/finalizer反控无泄漏。
  - id: wire-product-capacity-and-receipts
    summary: 同时切换普通Chat和TaskRun的原服务，去掉receipt前的外置shell等待。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/command_cells.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/limits.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
      - echo-agent-cli/echo-agent-app-core/src/chat_driver.rs
    acceptance:
      - CLI只向原runner注入进程级shell=1政策，两个workspace仍共享上限；真实主路径删除旧的先await
        permit再return receipt，不保留双调度。
      - Started持久失败零执行，unknown持久结果走原reconcile；交接后Turn结束/UI detach与run
        cancel/stop_cell不同，原TaskTracker/observer持有同一cell直到terminal。
      - queued、Running及Finished时间/phase保留真实来源，started_at现为accepted_at的事实不得静默改成已运行；receipt丢失可沿原调用身份观察恢复，不重启同命令。
  - id: remove-operation-local-save-amplification
    summary: 依据分段测量减少原Store同操作内的重复读取、解析和上下文冻结，保持所有校验语义。
    files:
      - echo-agent/src/state/file.rs
      - echo-agent/src/state/mod.rs
      - echo-agent/src/agent/snapshot.rs
    acceptance:
      - 一次runtime_records扫描每文件只读/解码一次，原validator复用已解析payload；Coordinator复用runtime
        snapshot所含scope，Absent按原权威查询。
      - 同一冻结messages供投影和checkpoint构造，不在I/O期间持有Context锁；CAS仍核expected
        version/epoch，冷热恢复、scope index损坏和owner-first crash切点不弱化。
      - 原格式及raw/approved身份、sequence、签名、pending batch与ack proof往返无损；原physical
        Store结算与unknown恢复保持，caller abort不生成第二写。
      - 真实大小/多scope样本复测仍超过预算时停止性能切片并保留RED，提交分段证据供同authority缓存/失效补充设计审定；不得删原件、关校验/fsync、换数据库或加大timeout。
  - id: bind-capacity-to-original-physical-operation
    summary: 闭合真实父Verification争槽RED，将同一global1绑定前台/后台/Code/技能原生操作的原物理owner，并保持预算、来源与恢复权威。
    files:
      - echo-agent/echo-core/src/sandbox.rs
      - echo-agent/echo-core/src/tools/mod.rs
      - echo-agent/echo-core/src/tools/native_execution.rs
      - echo-agent/echo-core/src/tools/cell.rs
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/echo-execution/src/sandbox/local.rs
      - echo-agent/echo-execution/src/sandbox/manager.rs
      - echo-agent/echo-execution/src/sandbox/docker.rs
      - echo-agent/echo-execution/src/sandbox/k8s.rs
      - echo-agent/echo-execution/src/skills/external/run_script_tool.rs
      - echo-agent/echo-tools/src/shell.rs
      - echo-agent/echo-tools/src/code.rs
      - echo-agent/echo-orchestration/src/tasks/command_cell.rs
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/limits.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/unattended.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/command_cells.rs
      - echo-agent-cli/echo-agent-app-core/src/infra/factory.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_pool/pool.rs
      - echo-agent-cli/echo-agent-app-core/src/plugin_components.rs
      - echo-agent-cli/echo-agent-app-core/src/terminal/project_ssh/sandbox.rs
    acceptance:
      - 保留R08真实factory/execute_run/MAIN Verification/shell/wait
        RED和原5秒cell期限；后继父任务结束前实际echo/exit0，不手工释放parent、不放宽quota、不禁background输入。
      - 原ToolManager只传provider引用，per-attempt
        timer/childcancel保持；每次资源恰一次取得并仅送本次command/child原owner，root/child/Plugin/pool最终manager共享App同一global1。
      - Local/direct正常root
        exit+EOF而原PGID后代仍活时保留资源；natural/cancel/deadline真实组控制保持同一预算，Unknown保Child/PGID/guard直到真实闭合及exact
        fence确认。
      - 默认false后端能力及runtime-only要求用唯一检查覆盖工具、Queued、Manager实际leaf/fallback/stream
        pre-spawn和Docker/K8s公开共同入口；Some未支持在workload前失败，None按最终有效政策保旧，不盲包controlIO/PTY/MCP、不猜name/role/isolation/guardcount。
      - 有效Some SSH以单一原Python3 stdlib controller
        exec接替，实际session/group/signal/errno/monotonic
        probe在payload前；同一原deadline含probe/执行/组等待/清理，只有原组真结束或明确未启动的匹配nonce完整有界EOF尾帧才结算。缺失/截断/EPERM/non255本身保持原Unknown，不改来源、不假cap、不无依据退役已证目标。
      - 本地probe读取阶段零保留并持续drain，真实双pipe各1MiB控制通过；direct
        script旧输出与关闭stdin保持；原35断言/时限/global1/策略来源负控不变，真实Local前置不代替实际结果。
      - 按最终冻结源编译新cfg(test)/公开Rustdoc/examples并运行原5秒、35、新组/凭据/预算/manager/混合workspace及保存恢复控制；纯本机controller3探针与源码PASS不代签RustSome、受控SSH、App、phone或mainCI。
  - id: validate-full-owned-lifecycle
    summary: 闭合后台/保存组合故障、恢复、公共消费者与正式说明，保持验证层级真实。
    files:
      - echo-agent/echo-orchestration/src/tasks/command_cell.rs
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/src/state/file.rs
      - echo-agent/src/state/current_native_recovery_tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/command_cells.rs
      - echo-agent/docs/zh/41-persistence-concepts.md
      - echo-agent/docs/zh/41-shell-text-tools.md
      - echo-agent/docs/adr/
      - echo-agent/echo-agent-learning/examples/
      - echo-agent-cli/docs/zh/architecture/persistence.md
      - echo-agent-cli/docs/zh/adr/
      - echo-website/
    acceptance:
      - 复用queued_cell_cancellation/timeout、prepared_drop、tracked_capacity、explicit_run_cancel和ordinary_chat/start_append_failure回归；终态/产物失败均保留typed结果，父Turn失败后原cell结果可读。
      - 组合覆盖9.5MB/8028原delta、多scope、pending projection、owner-first crash、corrupt
        index、stale revision、generation隔离、已接纳write caller
        timeout/abort，所有失败自然记录。
      - 分别更新框架准入/资源/持久化ADR与应用政策说明，公共API/枚举/构造消费者及examples真实编译；SDK独立pin和依赖风险合同不自动改变。
      - 原微信计划已有的shutdown_end回归101仍由该owner修复；合流门禁不能忽略它。新基础设施GREEN不冒称原微信、真实App、main
        CI或发布通过。
artifact_id: plan:5470deb5-c11d-4126-abd3-2d7474e98a26
lifecycle: ready
---
## Notes

2026-11-08 阶段验收：原Outcome、六项Todo、原时限与边界保持，Plan仍ready、delivery_ref仍为空。当前R17同一753 native／427a冻结工作树的51项全部实际通过，0失败0忽略；Framework31、保存/恢复/真实snapshot47、15独立feature及同9.5MB样本四阶段原10秒预算分别通过。证据与旧失败保存在正式阶段报告及主根R02原件，不能相加旧cf5结果冒充最终源。

R16的49P2F及原R08父任务争槽RED保留；两fixture仅补普通Chat根消息身份、修正懒factory实际generation的Mock传输并暴露原错误。R17独立registry无client负控已实际观察固定错误且Mock调用0，阳性仍真实typed shell成功/exit0/stdout。没有放宽角色、Schema、global1、原5秒或验证器。

原Outcome第四项尚需协调会话完成CLI workspace集成质量/完整适用测试、公开消费者和官网最终同步、consumer8bd/283的精确复评及原actual-lock/canonical-origin门禁；原Outcome要求的组合App、main CI与发布继续分层，不由受控lib-test或界面专用旧来源包代签。当前不关闭Plan或Goal，不改变SDK独立pin/合同，不自动重绑依赖批准。
