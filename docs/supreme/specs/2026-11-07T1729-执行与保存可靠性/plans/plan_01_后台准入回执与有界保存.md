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
    digest: sha256:cf92c5bd72ecaf996f8bc980792b36324c509953f1a3edfacbe55d2ab2a6e0e6
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

2026-11-08 R20 阶段证据（本次采用）：最终本地完整 workspace 命令已自然退出 0，零失败、零编译警告、零源码漂移。原 Outcome、六项 Todo 和七个 Design 章节不变，Plan 仍为 `ready`，`delivery_ref` 仍为空，Goal 保持 `active`。精确风险门禁、同源 App 边界、官网及最终交付尚未收口。本节为本次正式采用的R20阶段证据，整体交付仍未完成。

正式采用日期：2026-11-08（Asia/Shanghai）。技术证据切点：原完整 workspace 终局 2026-11-08T09:23:23.793963+00:00，本次 fresh raw audit 终局 2026-11-08T17:27:06.569183+08:00。风险人类裁决状态：PENDING。

应用原 `ChatEventLog` safe-point 复用既有 retention 判定：没有需要裁剪的 segment、也没有待收敛 cleanup 时，先返回，避免每次追加都主动追平可派生的 native projection；真正读取仍经原 API 自动追平。真实裁剪及 cleanup 路径继续原 journal sync、checkpoint 覆盖确认、pin 和 prune。此处只记录已审查的应用调度优化，不向 Framework 新增产品策略，不修改原耐久屏障、10 秒保存预算或 20 秒综合用例期限。

| R20 验证 | 原程序的实际结果 | 可证明的范围 |
| --- | --- | --- |
| 旧生产源加新精确回归 | 自然 101；coverage save 尝试 1、成功 1、失败 0 | 新断言取得有效 RED；后续 live/cold 断言未到。 |
| 新生产源的新精确回归 | 1P/0F/0I，自然 0；coverage save 尝试、成功、失败均 0 | 非空原 journal、实时及冷重开 publication 完整断言均到达并通过。 |
| App-core `native_history_` 族 | 15P/0F/2 项原有 ignored，自然 0 | 邻接 correctness；两项原 opt-in I/O 诊断没有被执行或改为普通样本。 |
| 旧 CLI 原邻接 1、2 | 各 1P/0F/0I，自然 0 | 分别绑定原 CLI 程序，不相加冒称整个 CLI 套件通过。 |
| 旧 CLI 原邻接 3 | 0P/1F/0I，自然 101，7.52 秒 | 完整 DTO 比较实际失败；该旧批遇首错停止，邻接 4–7 当时未执行。 |
| 同一 R20 App-core 程序无过滤默认并行诊断 | 外层 2367P/0F/15 项原有 ignored，自然 0，308.44 秒，输入无漂移 | 包含原 A2 综合用例在原 20 秒内通过；不是完整 Cargo workspace 命令。 |
| 最终六项质量与编译 | 六阶段自然 0/W0，输入无漂移，四个程序及各自来源制品保全 | 包含全 workspace 的 NoRun；编译不代签运行。 |
| 新 CLI 原七项邻接 | 7 项各 1P/0F/0I，自然 0，输入无漂移 | 含原失败的完整 DTO 用例、真实裁剪、首次冷读及 Fork；不代签原完整 workspace 或 release 性能。 |
| 原完整命令内 App-core 主库 | 外层 2367P/0F/15 项原有 ignored，296.73 秒，无过滤 | 主库结束后同一 Cargo 已继续后继目标；与上方独立主库诊断分列，不相加或用时差作提速结论。 |
| 同一原完整命令随后三个集成目标 | 分别 6P、5P、2P，均 0F/0I/0 filtered | 逐目标保留实际结果；两个重持久化目标、CLI 和完整终局另列于下方。 |
| 原完整命令内两个独立持久化目标 | presentation 1P/0F/0I、260.96 秒；usage 1P/0F/0I、293.09 秒 | 两者均在本次原命令实际完成，不借 R19 独立程序的旧通过结果代签。 |
| 原完整命令内 CLI 主库 | 143P/0F/3 项原有 ignored，60.25 秒，无过滤 | 与上方七项 exact 分列，不将相同用例重复相加。 |
| 原完整 workspace 最终终局 | 原命令自然退出 0，零编译警告、终局源码 guard 无漂移 | 后续 r4/runtime_state、Tauri 目标及 doctest 均实际完成；零测试目标和原 ignored 不作为新增断言或 GUI 实跑资格。 |
| 本次批准前的 actual-lock／canonical-origin 官方 raw audit | 两次各自然退出 1；均零漏洞、恰原五项未修复 warning | 原始扫描与精确 gate 分开；候选 preview 不是正式风险接受或 gate 通过。 |

App-core 实际程序 SHA256 为 `97c3df25aeaf3ae4983450969fead680da1fb8b188187a3246d378c5315c0e74`，CLI 旧邻接程序为 `c570fe1f0743585133aa359b8b3e953e8eaf3af1f75e6955bd96b2b58f306121`。该 R20 构建批次自然 0/W0，冻结 2503 项输入；程序、`.d`、fingerprint 和来源回执分别保全。主库输出内嵌的 1P/2381 filtered 行不作为外层终局。

旧 CLI 第 3 项失败保留。源码核对发现，原 baseline 只使用纯 publication mapper；实际 GUI consumer 还会根据原日志附加一次 presentation supplement。single-final 与 progress+final batch 的承载位置不同，不能删字段或把该差异归为可忽略 metadata。此结论来自原路径源码及该次真实失败，不反向宣称更早旧程序已实际复现；后继新程序的通过也不覆盖原 RED。

单文件完整 oracle 候选已独立 Source PASS，C0/I0/M0，审查 SHA256 为 `e8bdd939d2575457681b3383b30e67293614e730143005a0a5115b6ac082be97`。expected 从受控原 fixture journal 独立构造，保留 source、identity、event/cursor、固定 head、prefix 和全部 record 字段；single、batch、cold 分别建模承载位置。原 publication baseline 值和完整 JSON/Vec 比较保留，未从被测 GUI 响应复制答案，未在 tail 后调用会追平 native projection 的查询预热。此处原 replay 的无维护路径依赖具名 fixture 无 pending barrier 的前置，不泛化为所有 replay 均无副作用。

候选已由唯一 writer 采用到原测试文件并限定格式化，最终文件 SHA256 为 `069b42f77c54705843b1b5df2f471622b17e5ba904b72aefd66d871c5226ff45`。仅两个 helper 和一个必要 caller 改变，生产 consumer 与其他保护源未改；Final 增量审查已 PASS，其完整回执由本次记录 manifest 绑定。后继最终质量与 NoRun 批次（own91375，`r20-final-quality`）六阶段均自然 0/W0/M0，冻结 2503 项输入；App-core、CLI、presentation、usage 四个程序及 `.d`/fingerprint 已保全。新 CLI 程序为 `dfe8ee6e12c2f0df2bce7aca31198dd73a1d53637b4d336e91a69154b9eef9f1`，353,745,568B。`r20-final-cli-neighbors` 的 recipe 同样绑定该程序和 2503 项输入，七项原 exact 各自 1P/0F/0I、自然 0/M0；原失败场景的完整比较确已在新程序通过。这是测试 oracle 修正及对应真实复验，不以 Source PASS 或编译成功替代运行证据。

原 `cargo test --workspace --all-features --locked` 无 selector/默认并行命令（own99974）现已完整自然退出 0/W0/M0，包含两个独立长测试、后续 r4/runtime_state、CLI、Tauri 和 doctest。终局 raw SHA256 为 `016f1d84a4f124e5d6c8fbf56064ab2e50df7e89f7b4773df630ae76db963252`。`central-terminal-summary.json` 逐目标保留原结果；内嵌的 1P/2381 filtered 不作为外层主库终局，Tauri 的零测试目标也不构成 GUI 运行。原 Cargo PID 已结束，四个保全 native 的真实 reader 均为空。R04 的 `full-progress.json` 继续以其中途观察的原日志前缀身份保留，不能冒充终局摘要；本次记录另外绑定真正的完整 execution、terminal summary 和最终 raw。

六项质量、七项 CLI 邻接及原完整命令的本地测试门禁已闭合。协调者后继将本次精确六文件代码／测试／ADR／性能说明本地提交为 `efbf200c0d2ff9c4ff20ba567e2660fa0200e248`，本次记录以真实 Git 对象元数据绑定六条路径；编译和测试资格仍准确绑定其原冻结工作树与程序，不把后继提交元数据倒填成原编译时 HEAD。

本次 fresh 官方 actual-lock 和 canonical-origin 扫描均使用原 `cargo audit --json --deny warnings`，两次真实出口均为 1，零漏洞，保留原五项未修复 warning：glib 0.18.5 为 unsound；proc-macro-error 1.0.4、zvariant 5.13.1、zbus_names 4.3.4、zvariant_derive 5.14.0 为 unmaintained。原 settings 未设置 ignore、target 或 severity 过滤；完整 advisory/source/checksum 身份及原始输出由批准材料和本次记录 manifest 绑定。该事实不改写为零 warning 或正式精确 gate 已通过。

新源码的官方 consumer 摘要为 `43ed071e6471d4ab627520a74843fdd8c3524a023b30c85205270176257c3143`（283 个输入，相对原接受源码 78 个路径变动）；锁仍为 `48eece730daaa9eff539e416861454441c23c54abf02bacaa2c3bdb375bd6329`（769 包）。本切点的批准材料仍明确 `humanAuthorized=false`、`formalAdoption=false`，正式接受仍是原 `42a1dd…`。协调聊天已提出仅限本次的精确绑定问题，尚不据此授予新源码接受；原八项控制及两项 fresh 正式精确 gate 继续待完成。本阶段工程记录已在本次采用中更新；官网的最终Git文档来源及同源App边界仍未完成；原 UI3 `b6c23b7d`/Framework `f479f52c` 包不包含后继 R20 生产 guard，不能代签当前 App。release 性能、真实 GUI/phone、SSH 网络、公开 Host、main CI、推送、合并和发布继续分别验收，不能由 debug correctness、旧风险批准或源审继承。

### R19 既有 Notes（原文保留）

2026-11-08 R19 阶段记录：原 Outcome、六项 Todo、不做事项及七个 Design 章节保持，Plan 仍 ready、delivery_ref 仍为空、Goal 仍 active。联合切片 c6577d 冻结 1814 项 Rust/构建与 687 项额外输入；新主程序 349a8949 及两个独立持久化程序已保全。六项质量/编译全0/W0/M0，命令cell35、执行环境9、Git/worktree27 的默认并行回归共71P/0F/0I。原完整命令主库实际2365P/1F/15I、324.82秒，Cargo自然101，唯一A2失败，独立integration及后续workspace因failfast未到。不能据专项或编译完成关闭Plan。

历史证据保持：R17自然full2358P/10F/15I与专项51分列；R18原cf6十项exact实际10P/0F/0I，后继full因六个CurrentThread Git等待环受控SIGINT，不冒自然libtest完整结果；同cf6命令cell命名空间实际32P/3F/0I。后继夹具使用原spawn_blocking+await驱动真实Local，各测试服务复用真实默认governor并独立作用域，同服务Chat/Run仍共享1，生产全局身份走不注入governor的真实构造器及原断言。fallback旧phase未捕获，保原5秒与失败metadata，不反推首因。R18 typed Cancelled第一版Important及修正、R19独审均留原件；Framework Fork任意current_thread Host支持边界独立归档，未宣称修复。

test-utils分支真实注册2381+1+1仍为原2383；无test-utils的原无默认feature单元程序编译0并list2180，含两个原长case，未实际运行该分支长case。Frame1499输入保持，Frame31、保存/恢复/快照47、15feature及真实9.5MB原10秒四phase沿各自原程序分scope保留，不叠加冒称新App或全量资格。原始失败、源码/程序/样本SHA及最新剩余项见正式阶段报告和R02证据。官网最终同步、当前精确consumer人类复评及actual-lock/canonical-origin、App/phone、真实SSH、mainCI与发布分别验收；旧风险批准不自动续到后继源，当前不改变SDK独立pin/合同。

协调者后继把IM2/GUI1/SRE5八个已通过测试修复分别本地提交972263e6、b123dbdf、d6eefd456b93e3c944fa62512f90e2cb2e60267d，当前CLI HEAD为后者；七文件隔离增量与五文件风险WIP未夹带，源码字节未因提交改变。局部提交不代表完整门禁或全任务完成，UI3仍绑定原b6构建来源；本Plan仍ready且delivery_ref不变。
