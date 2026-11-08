---
title: 执行报错治理：后台准入、回执所有权与有界保存
artifact: design
carrier: markdown
---

# 执行与保存可靠性设计

## 审查结论与证据

本设计针对截图中 `Tool 'shell' execution timed out` 与 `Runtime state deadline exceeded: runtime checkpoint CAS` 的整条生产路径。框架快照 `f2e44dc9be8972e286f9a84520ddb263f0012424`、应用快照 `5f5fd4912855cfda2326c54f294f11cb879257fa`，为最初审查快照，均含当时的未提交增量。该次独立只读审查未运行 Cargo、App 或原任务，当时实现未通过、保存耗时占比尚未测量；以下后继实测不改写原先可知事实。

原数据根为应用 `.echo-semantic/local/persistent-GUI-owner-66760c3-preparation-20261031-r01/isolated-runtime/data`。原 Run 为 `runs/run_0e6c96fc-9b2c-408a-9357-462afe1ac266.jsonl`；原 TaskRun 事件为 `workspaces/Lp-agent/.eko/tasks/taskrun:df382922-de01-47a4-8219-25e7d01a0779/events.jsonl`，对应聊天 journal 的流目录为 `chat-events/sha256_c8cf992d9d37c3920b270d89945295407b3f6be81b0ed2963fb46bc502f17de3`。不将旧报告里“命令结果未知”的早期结论覆盖成当时已知；以下为读取后续原记录所得。

| 北京时间 | 原事实 |
|---|---|
| 16:08:47 | 最后模型请求成功；后台 shell 请求 `background=true, timeout=3600`，call `call_5034791ef36446a9a36f29d2`，cell `d4006718-e930-4473-9018-de8e80294ec6` 持久 prepared。prepared 时间不等于 OS 进程开始时间。 |
| 16:11:29.729 | 占用前一执行的 cell `e33bdfd2-c80d-4e81-9616-85b4d2aa59bc` 以 101 结束。 |
| 16:11:30 | 新启动调用经过约 163.247 秒返回工具超时，但已创建的后台 cell 并未随之结束。 |
| 16:11:44～45 | 工具批次之后保存报 CAS 入口 deadline 已过期；transcript 为 blocked，整轮失败。工具结果到保存观察相隔约 14.43 秒。 |
| 16:22:15.748 | 新 cell 最终以 101 退出；原测试 `shutdown_end_releases_the_coordinator_for_the_same_surface` 失败，报 `a successful shutdown did not release the exact incarnation`。 |

### 根本性 findings

| 编号 | 严重程度 | 源码事实与影响 | 处理 |
|---|---|---|---|
| EXEC-01 | Important | CLI `command_cells.rs:1754,2080` 在 prepared/Started 持久化后、返回 receipt 前等待外置 shell semaphore；`executor/limits.rs:129` 默认 shell=1。与 ShellTool 的后台立即回 cell 体验和外层 120 秒工具预算组合失配，普通 Chat 与 run-owned 均受影响。 | 全局资源等待进入原 cell 的 Queued runner，接纳回执不等待执行槽。 |
| EXEC-02 | Important | `echo-execution/src/tools.rs:103-135` 的 started timeout 取消后等待原 future，最终可能返回 timeout；`shell.rs:334-337` 的 lifetime cancel=None 使后台可继续，成功 launch receipt 可丢失。`command_cell.rs:633-641` 的 tracked-capacity 等待同样不能被当前启动调用取消。 | 区分接纳前取消与接纳后 cell 生命周期，准确保留 receipt，覆盖两类 capacity。 |
| EXEC-03 | Important | `snapshot.rs:1098` 从操作开始共享默认 10 秒；错误精确来自 `state/file.rs:1512` 的 CAS 入口检查，而非已证明 CAS 写了 10 秒。`reconcile_scope_owner_sync:1002` 扫描全记录，`runtime_records_sync:1201` 同记录再次读/解析，owner 验证和 coordinator 又多次恢复完整 payload。 | 先分段取证，再在同一原操作内消除重复工作，保留 CAS 和恢复权威。 |
| EXEC-04 | Important | 原检查点 9,526,506 字节、130 条消息，包含 8,028 个 raw delta blocks、69 个 request/block 身份；raw 与 approved 两组 JSON 各约 4.11 MB。增量原件包含身份、sequence 与发布证据，并非可随意删除的冗余。 | 不删 provenance、不改存储格式来伪造快检；验证读取/校验/序列化成本后优化。 |
| EXEC-05 | Important | 101 的实际测试失败与工具超时、保存失败是不同事实。 | 回到原微信恢复计划修复其业务断言；本设计不代替或扩大该业务任务。 |

`CommandCellRegistry` 合同明确允许有界等待容量（`echo-core/src/tools/cell.rs:335-337`），因此不能笼统宣称所有 framework launch 都违反立即返回。根因在 Eclip 外置准入与工具调用/后台生命周期的配合；实际哪个保存子阶段占主导仍需受控 profile，不能把当前风险判断写成已测根因。

2026-11-08 的真实 factory→execute_run→MAIN Verification→shell(background=true)→wait 补充取得有效 RED：父任务持有唯一进程 shell 许可，cell 首次观察为 Queued，随后在原5秒内 Failed/cause TimedOut、exit=None、output=0；四次模型请求所见 available=[0,0,0,0]，父结束后才恢复1。不是 fixture timeout，不以扩大原期限解决。后继完整 r03 SourceReview 为 C0/I0/M0，仍只有源码资格；纯 Python controller 三项本机原组探针通过，不代签 Rust Some／SSH／App／原5秒／35项或最终冻结源。


### R20阶段证据

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

以下 R19 及 R17 段落保留其原历史时态，不覆盖本段 R20 实际结果。

### R19 最新阶段资格

2026-11-08 的联合切片 `c6577d2968db22b6898026d976a40c5aecb1b09d49a5e8acdb26dbcf411a9e47` 冻结 1814 项 Rust／构建库存及 687 项额外输入；主程序为 `349a8949ec6bf24e8fa8702c0312218b9ef2425fb1c4539c5380dead7a86f4a7`。六项质量／编译全 0/W0/M0，命令 cell 35、执行环境 9、Git/worktree 27 的默认并行回归共 71P/0F/0I。原完整 workspace 命令仍实际自然退出 101：主库 2365P/1F/15I、324.82 秒，唯一 A2 失败；独立持久化目标及后续套件因 failfast 未到。当前不关闭原 Outcome、Plan 或 Goal。

test-utils 分支实际注册为主程序 2381 项、两个独立持久化目标各 1 项，总数仍为原 2383。无 test-utils 的原无默认 feature 单元程序也实际编译 0，列表有 2180 项并包含两个原长用例；此处只证明入口注册，不签默认长用例运行。Framework 原 1499 输入保持，以下 R17 的 Frame31、保存/恢复/快照47及真实样本四阶段继续按原程序、feature 和声明负载分 scope 保留。

协调者后继将 IM2／GUI1／SRE5 八个已通过测试修复分别本地提交为 `972263e6`、`b123dbdf`、`d6eefd456b93e3c944fa62512f90e2cb2e60267d`，当前 CLI HEAD 为后者，工作树字节不因提交改变；七文件隔离增量与五文件风险 WIP 未夹带，不能把局部提交当作全量或整个任务通过。R19 编译时元数据与 UI3 原构建来源仍是各自的 `b6c23b7d`，不改写旧程序资格。

R17 自然完整回归 2358P/10F/15I 和 R18 同 cf6 十项单控通过均保留。R18 完整运行因六个夹具同步 join 阻塞原 CurrentThread driver，由协调者仅对实际测试 PID 发 SIGINT 后结束，不能记为自然 libtest 完整结果；R18 同命名空间 32P/3F/0I 也保留。后继六个原同步 Git body 放入原阻塞池并完整 await，真实 Local、原 runtime、断言和预算保持。测试服务各自建真实默认 governor 实例，服务内 Chat/Run 仍共享容量1；生产全局共享由不注入 governor 的真实构造器整合控制验证。fallback 的原失败 phase 未捕获，不反推相同首因。上述均为夹具与证据修闭，不把生产 global1 或状态权威改成测试政策。

原 Local typed Cancelled 夹具的第一版 Important 与 r02 修正、R19 四文件独审 C0/I0/M0 分别归档。Framework Fork 同步 isolate/finalize 的任意 current_thread Host 支持仍是独立 Source 边界，未由本次夹具修复签署或修复。最新原件与剩余门禁见阶段报告；此前通过不代签 A2、后续 integration、GUI/phone、真实 SSH、精确风险门禁、main CI 或发布。

### R17 受控验收证据（历史切点）

2026-11-08 的同源补充已取得实际运行结果。Framework 原 1499 输入字节保持，通用机制由协调会话本地提交 `f479f52c26ca5664bed960f2793100382852ffdb`；CLI 的 `aecf90c161bcc5b4808a11f7311de696831c7122` 是界面提交元数据，不能冒充包含工作树全部 SRE 增量的提交资格。应用以冻结工作树 manifest `427a916cf7d182f463252237ed07613dc494a00a859c8057e4f5cbfe7105d4cf` 与唯一测试程序 `753140f44bf96e48676e7665c0a0c73e484f207db29cf7f05202dc2740ccdf0c` 为权威。

- Framework 新控制与脚本相邻控制 31 项、保存／恢复／快照 47 项均自然通过；15 个原独立 feature check 及必要叶 crate 质量／编译检查通过。它们绑定各自真实 feature 和程序，不跨切点冒充资格。
- R17 同一新应用程序的 51 个不同用例全部通过，0 失败、0 忽略。先复验两项旧失败，再执行原5秒、其余新控制、原35及旧None SSH 6项；完整 argv、PID、自然出口和原件在 `.echo-semantic/local/exec-save-reliability-20261107-r02/app-runtime-controls-r17/`。原父 Verification 在原5秒内实际获得 Running→Output→Succeeded/Exited/exit0、36B stdout，父结束后容量为1，未提高配额或改变原断言。
- R16 的 49P2F 保留。混合用例当时缺普通 Chat 根消息身份、后台在准入前被拒绝；子 Agent 用例把 Mock 接到被工厂注册替换的旧 handle，原 helper 未收集终端错误。R17 独立 registry 的实际负控已观察 `missing_llm_client=true` 且 Mock 调用0，随后在真实懒工厂的 generation 正确绑定传输并取得阳性；共享 helper 保留原错误分类、工具名和计数，不输出 prompt／正文／schema／凭据。两项属于测试前置／协议修正，不放松产品校验、角色、时限或全局容量。
- 同一原9,526,506B只读样本的显式 profile 实际执行一次；Store 冷／暖584/548ms，coordinator 冷／暖3361/1044ms，四阶段均在原10秒内且原件字段保持。cold 指新隔离 Store／authority，未清除OS页缓存，不据此宣称同条件提速。初次汇总器97只遗漏与testname同行的冷指标，后续只解析同一完整raw，没有重跑或修改测试。

完整证据入口为 `.echo-semantic/local/exec-save-reliability-20261107-r02/r17-app-runtime-summary.json`、`same-source-save-profile-r16/summary.json` 与正式阶段报告。原Outcome第四项的仓库集成检查、完整适用测试、精确依赖复评／门禁及组合App资格仍须按原范围闭合；当前Plan保持`ready`、Goal保持`active`，不以51项或GUI专用界面包代签完整交付。

## 目标与范围

后台启动在接纳边界前可取消且无执行，接纳后总能找回准确 cell 回执；排队、运行、取消、期限和退出由原 supervisor 持有。保存维持真实、有界和可恢复，不把未知写成成功。两者共同保证长工具与大上下文不会因错误的观察/所有权边界使整轮失真。

覆盖普通 Chat、TaskRun、后台 shell、原 CommandCell manager、ToolManager、FileRuntimeStateStore 与 coordinator 的直接路径。没有新队列、执行器、Outbox、SQLite、TaskRun 状态机或资源策略放宽。原微信业务测试修复、自动重跑旧命令、重发通知、删除历史及登录/原数据修改均不属于本计划。

补充覆盖前台 shell、run_code、技能脚本探测／执行的真实物理准入，是移除父 Verification 物理占槽所需的完整接线。逻辑 Verification 串行与真实进程容量分开，自动命令仍共享同一个 App governor.shell=1；不缩小到只修 parent。用户直接 PTY／MCP、项目控制 I/O 与扩展配置保持原路。

## 分层与复用

| 层次 | 责任 |
|---|---|
| 框架通用机制 | 原 CommandCell manager 中的准入上下文、取消/期限、Queued 执行等待、资源保有；原 Store 中的 CAS/恢复和操作内验证复用。 |
| Eclip 产品策略 | 全进程 shell=1 等资源上限、TaskRun/普通 Chat 接线、原 Started/Finished 事实及现有 UI 观察。 |
| 薄适配边界 | 提供通用异步资源准入回调及原资源 guard；引用原 manager/TaskTracker，不拥有第二个 ready frontier 或执行循环。 |

复用 `Prepared/Queued/Running`、`CommandCellReservation`、`start_prepared/abort_prepared`、原 watcher、TaskTracker、同一 journal 和 `PersistenceCallContext`。现有 `resource_guards` 只能保有已取得资源，同步 ToolAdmissionFn 也不能替代异步等待；需要时在原通用 API 增加最小明确接缝，并核对全部合理公共消费者。

框架只增加可选 NativeExecutionResourceAdmission、原 ToolContext/ToolManager 能力引用、原 command 的运行时要求和 backend 能力合同，默认不启用。App 只提供同一个进程 governor 并接正常 factory／child／Plugin／最终 shared manager；不将 Eclip 角色、配额、任务字段塞进框架。native→cell 薄转换只转发原 cancel/deadline 和 guard，后台仍只在 Queued 获取一次；不保留第二套映射或队列。

## 目标行为

### 接纳之前：调用方可以撤回

启动调用携独立准入取消和有界调用预算，与后台 cell 的生命周期 token/deadline 分开。tracked-capacity、manager 准入锁、prepared 产生及应用 Started 持久化均受准入上下文监督；不能仅取消外层 future 而让容量释放后继续创建新任务。

接纳前取消/超时必须证明没有 OS/远端执行；若已生成 prepared reservation 但尚未转移所有权，由原 reservation 的中止路径结算。持久 Started 失败不启动命令，原失败真实保留。取消/关闭/接纳竞争只产生一次明确结果，不制造第二个 cell。

### 接纳之后：回执与 owner 一起转移

prepared 的原 cell、必要资源 guard、原持久 Started 及 supervisor 接纳形成唯一交接点。完成交接后返回准确 `cell_id/accepted_at/deadline`，不等待全局 shell 槽空闲。外层观察预算随后到期，不能把已接纳的成功回执降为“未创建”；调用取消与回执传输竞争时，按原 invocation/call 身份仍可查询原回执，不为重试自动创建副本。

这只要求原作用域内回执可恢复，不自动授予再次执行同命令的幂等复用：AI 有意重复运行命令仍是新调用。相同提交身份的传输重试与新的业务调用必须区分。

### 原 Queued runner 管理容量与生命周期

选择把 Eclip 的全局容量准入注入现有 `BackgroundCommandManager` 原 runner：start 被接受后进入已有 Queued 并确认接纳，再由同一 runner 等待产品容量和内部执行许可。准入只提供异步获取与持有 guard，不另起应用调度队列。

排队使用原 cell 绝对 deadline；本设计保留当前从 accepted_at 起计算、包括排队时间的 timeout 语义，不悄悄改成进程启动后才计时。排队取消/到期/关闭不启动进程；开始执行后显式 stop_cell、run cancel 或 shutdown 沿原 cleanup 等真实物理结算。普通前台 shell 的既有限时不放宽。后台脱离前台 Turn 的政策保持，不能简单把 Turn token 当 lifetime token。

真实 Started/Queued/Running/Finished 不混称；prepared 记录和日志时间不能冒充进程执行时间。全局 shell=1 不改变，所有 workspace 仍共享原上限。任何路径都恰一次获得/释放许可；原观察和 terminal 持久化失败保留原 owner 债务，不提前释放后伪造结束。

### 保存的分段证据与优化

同一次保存记录脱敏 scope/runtime/operation、共享 deadline 和各阶段进入/离开：epoch、scope/generation 读取、payload 校验、冻结上下文、投影/摘要、序列化、CAS 前/后、transcript apply、ack 与错误清理。使用原运行身份和单调耗时，不记录正文/密钥。先使用隔离复制的 9.5 MB 样本和合成多 scope 数据，原用户数据仅只读。

首段优化保持文件格式、raw/approved 内容与原物理写入路径不变：一次扫描每个 owner 只读/解码一次；同一次原操作复用已解析 payload 和冻结 messages；已有 runtime load 返回的 scope 不重复查询。CAS 的预期 version/epoch 与真实提交校验不省略，I/O 期间不持有 Context 锁，不从缓存承诺提交。

`_scope_index` 是可重建投影，不是 authority，不能直接信任它跳过崩溃恢复。cold reopen、部分 CAS、scope revision 冲突、退休 tombstone、pending transcript 和外部取消继续核对原权威事实。

若分段实测仍不能满足原预算，维持未通过状态并进入同一 FileRuntimeStateAuthority 的缓存/失效补充设计评审；不自动上第二数据库、不删增量、不改格式、不全局放宽 10 秒。受控验收须解释每个耗时阶段和仍有的慢路径，而不是用“加大 timeout 后绿”代替修复。

### 失败与恢复

真实保存失败仍阻止无依据的最终发布，前端展示原 blocked/unknown，不能吞掉异常继续宣称已保存。调用方超时与实际效果未知要分别记录；已经准入的 blocking write 由原 Store 等待/结算和原恢复路径接管，不能因删除 Future 生成第二次无身份写。

恢复只对原 pending operation 做幂等核对；旧确认不覆盖后续内容，回执丢失不触发工具重放。晚到成功、失败、取消均不能改写其它 execution 或释放错误 owner。

### 父逻辑任务与实际原生资源分离

父 Verification 保留每 run 的逻辑串行和其它 write／LLM／Subagent 约束，移除覆盖模型调用／wait 的进程 shell 许可。前台 shell、run_code、run_skill_script 的解释器探测／执行在校验后、dispatch 前于原 tool future 获取资源；原 manager attempt timer 与 child cancellation 保持，checked 原操作期限扣除准入耗时，不重启预算。ToolContext 只带能力引用，取得的 guard 只进入该次 SandboxCommand／原 direct child owner，不能写回长寿命 context 或父 invocation。

root child.wait、exit0、stdout/stderr EOF 不能代替 owned group 结束。请求新合同的 Local/direct 原 owner 在父退出后仍观察捕获的原 PGID，正常／取消／期限都沿同一 owner 和原计时器结算；未知清理保留原 Child、PGID、guard 与 exact fence，真实组结束后才释放。合法 nested sh、重定向后台输入保持。

### 后端能力与原 SSH controller 的闭合

SandboxExecutor 默认不声明 physical resource ownership；新 command 的 require 标志只在 runtime 生效，不进工具 wire 或持久格式。唯一检查只使用实际 backend 能力或已审 direct owner 实现能力，不从名字、隔离等级、角色、guard 数量推断。Manager 在实际 select/fallback 返回的同一 Arc 交接前检查，stream 在原 spawn 前保留 typed Unavailable；Docker/K8s 的公开共同 workload 入口也拒绝未支持的新要求，不在本次改其旧清理机制。原 availability/control 探测不误记成 workload。

有效 Some 的 SSH 在同一原 channel exec 单一 Python3 stdlib controller 接替原期限控制器角色；实际 session/group、errno、signal、monotonic 原语在 payload 前探测，不凭命令名／版本或 CAS 已用 Python 推定全体目标支持。一个原 monotonic 截止时间覆盖启动、负载、组等待和清理。只有原组实际结束或 payload 明确未启动，才能发绑定本次 nonce 的内部 EOF 尾部凭据；non255、root EOF、EPERM、nonce本身均不是证明。stderr读取阶段有界分离匹配完整尾部，保留原 payload／限额／计数；缺失、截断或不匹配保持原 Unknown owner。

generic 最终未使用新政策／require=false 保持旧路；App 最终 scoped Some 不能被输入 None 降级。显式 local(None)/custom executor 来源不替换；未证明 Windows/direct/custom／远端运行时可显式 Unsupported，是可见新合同边界而非全部平台修复。迁移需要原适配器的 trait 声明与真实 owner 证明，不批量给 mock true。当前 Mac 默认、已证 SSH 与 Unix sourceNone 正常路径若回归仍阻断采用。

## 参考实现与候选取舍

[Tokio Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown) 区分通知取消、由原任务清理、等待 TaskTracker 中任务结束；[Tokio Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) 的异步准入排队和取消规则支持把接纳前撤回与已接纳执行分开。官方正文已通过只读 HTTPS 核对；web 搜索工具 404 不当作浏览成功。

| 候选 | 结论 |
|---|---|
| 提高工具/保存 timeout，或 ShellTool 全局 manages_own_timeout | 不解决回执丢失和准入取消，还会放宽前台合同，不采用。 |
| 应用另起延迟启动队列与 owner | 可暂时回避等待，但复制 supervisor 和取消责任，维护成本高，不采用。 |
| 原 cell runner 加中立异步准入及独立 admission context | 采用；框架拥有机制，应用提供资源政策，复用已有 Queued。 |
| 用 `_scope_index`/新数据库/删除 raw deltas 加速 | 破坏恢复或 provenance，未获设计资格，不采用。 |
| 同操作消除重复读取/解析/冻结 | 采用；先测量、保持格式和权威，仍不达标时不能宣称完成。 |

后继参考固定 Codex unified_exec 修订 `2bc8e7f5459b084afa482c7aac3f56fef36286b3` 的真实 process owner 与限时输出观察分离，以及 Python 官方 subprocess/os 的 `start_new_session`、`killpg`、monotonic 原语；不复制其配额或淘汰策略。官方原文、固定 blob 校验和原始读取失败事实在受控证据中保留。App Tool decorator／全量 executor wrapper 因能力转发漂移、scoped Arc 身份变化、误管控制 I/O 和后台重复准入而不采用；显式能力合同复用原接缝，未支持 Some 在 workload 创建前失败，None 不伪授新资格。

## 验收标准

| 场景 | 必须成立 |
|---|---|
| shell 槽被长任务占用，连续后台提交 | 后续请求快速取得准确 cell 回执并处于原 Queued；无额外进程、无第二队列。 |
| tracked-capacity 已满，调用方超时/取消 | 容量随后释放也不创建或运行该未接纳任务；无 guard、tracked slot、future 泄漏。 |
| prepared/Started/接纳点逐切点竞争 | 持久失败无执行；接纳前可中止；接纳后原 receipt 可找回，相同传输身份不重复执行。 |
| 排队到期、stop_cell、run cancel、shutdown | 原期限/取消生效，物理退出后释放原许可，terminal 和输出完整，普通 Chat/run-owned 都覆盖。 |
| 前台 shell 与其它工具 | 原限时和权限不放宽，不借本修复统一免除 timeout。 |
| 当前 9.5 MB 与多 scope 保存样本 | 分段证据完整，原 10 秒内在声明硬件/负载条件下正常保存；未满足即保持失败，不改预算作绿。 |
| CAS/epoch 竞争、部分写、ack 丢失、cold reopen | 原成功/冲突/未知分类、精确 operation、恢复幂等和退休围栏保持；不信任不完整投影。 |
| 注入保存慢/失败与工具已执行 | 不重跑工具，不虚报保存成功，不丢已接纳副作用的 owner。 |
| 原业务测试 101 | 与新受控执行回归分列，由原微信计划独立验收；本计划不代签其修复，也不豁免合流门禁。 |

完整交付区分源码 review、定向回归、真实调用链、App、main CI。保存分段和原预算已有上述受控实测；仓库集成、完整适用测试、精确依赖门禁与组合App资格继续按原Outcome分别验收。

补充验收保留原5秒真实父 Verification 用例：父结束前实际 echo、exit0，不手工放 parent、不提高配额。Local/direct/SSH Some 的正常 root exit/EOF 而原 PGID 后代仍活场景，须用 natural/cancel/deadline 真实组控制证明保有且原预算不重启。实际 leaf/fallback、公开容器入口、SSH runtime前置／完整nonce尾部／限额与丢凭据 Unknown、最终各 manager 及跨workspace实际物理峰值≤1分别验收。原35断言、时限、global1、策略／来源绑定负控保持；真实Local测试前置不代签35结果。上述受控 Mac／Unix、Rust SSH启动接缝与保存／CAS已取得当前切点资格；真实SSH网络、GUI/组合App、公开消费者及最终门禁仍分别核验，不把本机受控程序代签其它环境。

## 协作、文档与交付边界

长期通用合同与 ADR 归框架；资源政策、普通 Chat/TaskRun owner 接线、配置与用户说明归应用。修改公共 Rust 接缝前盘点整个框架合理消费者，现有 examples 加入编译；SDK pin 和已接受跨语言合同独立，不自动续签依赖风险或升级。官网只同步已验证的行为说明。

本主题以“已接纳后台命令及保存结果始终可追踪”为一个交付结果；原 Plan 的观察、准入、保存和恢复验收都不能省略。落地以已确认增量、精确源码窗口和冻结程序为准；源码单一 writer，保留原日志、自然退出、原 RED 与不可变审查切片。源码审查／格式／纯controller探针、Rust Some／受控consumer、App、main CI和发布分别验收；文档/Plan不因局部通过自动完成，提交／精确风险绑定与对外动作继续服从明确授权。
