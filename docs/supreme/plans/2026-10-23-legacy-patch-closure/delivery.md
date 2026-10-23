# 四仓旧补丁审查与修复交付

原目标是“以现有干净的四仓集成工作区为对照，审查剩余旧补丁；优先 CLI 的 250、215、62 项变化及 LSP 改动”。全部 **14 组、718 处文件状态变化**已有处置；718 是各来源工作树的变化项合计，包含未跟踪材料及重复生成文件，不代表718个独立修复。审查采用行为、状态权威和消费者对照，不用提交是否祖先判断是否已合入。

原审查对照为框架 `4343f0b736beb5c8606e31146528a2fe8be80510`、CLI `0d1bbd39cb598175fcf307942d97b770d1b0b8e2`、SDK `a9f267f636cd4d1553a8bff335c3203ab0b8a5ac`、官网 `0dc2dc5097e787dc9ba26f7919074c82db718aaa`。四个子仓当时均干净；顶层既有修改单独保留。后继续证对照框架 `287ed2d6`／CLI `b85ec037`，再把已验证的局部修复接入本轮交付。下列审查结论与交付状态分开记账。

## 全部旧补丁的最终处置

来源名称相对各仓旧工作树；精确路径与历史 HEAD 见同目录 `legacy-patch-inventory.json`。

| 仓库／来源 | 变化项 | 审查及后继处置 |
|---|---:|---|
| CLI `observation-settlement` | 250 | 提取 MCP 禁用断连与失败保留、插件清理凭据、Cron occurrence 稳定身份、Scheduler 持久化失败显式报错、GUI sandbox 隔离下限五个切片。MCP／插件后继生命周期问题继续定向修复；删除会话、ReviewIntegration、TaskRuntime旧整文件不回灌。 |
| CLI `sdk-projection` | 215 | 15个非生成文件的有效行为已由当前实现覆盖；200个生成文件按下述规则排除，无独有修复迁移。 |
| CLI `task-runtime-framework-consumer` | 62 | 提取取消获胜后拒绝迟到 summary/review、从真实结算结果构造终态事件两项修复；canonical执行状态保真及boot中断候选在后继修复。历史验证材料不回灌。 |
| CLI `lsp-runtime-consumer` | 5 | 生产行为已覆盖；剩余 Result 传播没有证明当前主路径可触发，仅属防御增强，未采纳。 |
| CLI `main-only-ci` | 13 | 已被当前更完整的生命周期实现覆盖，无独有迁移。 |
| 框架 `acp-recovered-turn-identity` | 23 | ACP恢复身份及其余有效行为已覆盖；不恢复旧实现。 |
| 框架 `database-describe-fidelity`（分支 `subagent-stream-settlement`） | 37 | 提取 FinalAnswer 后首个清理尾错；后续再修默认 Agent 包装器同类吞错，其余变化已覆盖或被取代。目录名不作为行为归属依据。 |
| 框架 `lsp-response-fidelity` | 18 | 核心生产代码、正式文档及ADR共8文件与原对照逐字节相同；其它历史材料不回灌，无剩余迁移。 |
| SDK `semantic-120-55` | 68 | 已完成源码审查；不恢复旧Host/Protocol合同。父关闭后子句柄本地失效片段先保留为参考，后由真实Host取证并在当前三语言SDK边界修复。 |
| SDK `sdk-accepted-continuity-ci-01a13eb8` | 13 | 当前合同与治理已覆盖，不用旧裁决替换当前精确源绑定。 |
| SDK `sdk-semantic-governance` | 4 | 仅Python缓存，不进入产品补丁。 |
| 官网 `semantic-60-81-83` | 6 | 不整体迁移；按当前权限源码修正保护路径/Bypass顺序及相关镜像，再生成discovery。旧manifest不回灌。 |
| 官网 `semantic-review-remediation` | 2 | 长任务说明已覆盖，无迁移。 |
| 官网 `unified-turn-docs` | 2 | EKO存储说明已覆盖，无迁移。 |
| **合计** | **718** | **14组全部分类，无未审组；旧工作树不整包回灌。** |

250与215中的两批各200个生成文件彼此逐字节相同。每批189个AST类型结构等价，11个删掉必要import；原内存noEmit对照为当前 **0诊断**、旧补丁覆盖后 **16个未定义类型诊断**。这两批均拒绝回灌，不能用生成覆盖破坏现有人工补齐的跨crate import。

## 发现的修复闭合

原只读审查有8项Important提取建议、3项Minor，boot与SDK父子句柄当时仅待补证据。后续调查明确了3项Important、4项Minor及abandonment问题，并按用户Goal实施。它们是同一原目标的后继处置，不把修复Goal完成冒充原始全清单审查完成。

| 问题／后继风险 | 最终行为与证据范围 |
|---|---|
| MCP与插件生命周期 | 保留原Agent／registry／scheduler／lifecycle清理权威；精确token不跨registry释放；接管未结清不发布；restore／pool rollback保留typed receipt；shutdown调用者取消不丢清理；MCP generation准入与陈旧withdrawal按真实client身份核对。原CLI250提取价值落实，未恢复旧整文件。 |
| Cron、Scheduler、sandbox | 稳定occurrence身份、持久后端失败显式退出、GUI sandbox minimum isolation均已覆盖。没有由此引入新的Cron自动续跑政策。 |
| 取消、Superseded、abandonment | 拒绝迟到成功证据；事件与summary跟随最终durable结算；Superseded不追加旧attempt证据、不发终态事件、不唤醒续接；abandonment保留精确claim诊断，并补齐现有run锁/journal批次下的execution CAS。 |
| 默认Agent／Subagent清理尾错 | FinalAnswer后仍保留第一个清理错误；已有失败/取消优先级、原ExecutionNotSettled身份及物理EOF所有权保持。不是只修Subagent而留默认包装器。 |
| R1进程崩溃恢复 | 新intent由唯一写边界强制绑定进程epoch；持root OS排他租约后，Running/Paused在同一journal批次解析旧进程fence与副作用barrier；框架结算已接受的精确claim取消/暂停并保完整execution。当前进程、畸形epoch、无plan/orphan unsafe身份继续fail closed；终态确认债务不改原业务状态；不重放模型/工具、不伪造物理完成。 |
| SDK父子关闭 | TS、Java、Python成功父关闭后收束本地子scope，覆盖late reply、首次订阅、别名和幂等；失败关闭可重试，observer取消不取消真实清理。已缓存事实仍可观察，EOF不冒充业务终态。Host／wire／framework pin／accepted合同不扩张。 |
| Canonical replay与Review展示 | 完整TaskExecution优先且畸形canonical拒绝，legacy-only回退保留；GUI/TUI独立展示candidate、错误、operation身份、持久化状态及真实Inbox receipt。 |
| 权限及删除文档 | 正式权限文档、注释、流式示例、官网镜像/discovery跟随实际顺序；managed会话删除文档使用当前稳定request/helper。正式产品事实源仍在各子仓。 |

本轮不采纳的边界：Cron自动续跑涉及产品策略；browser readonly重连没有证明必须实施更严格关闭，consequential调用不重放；StoreUnsupported已经归为internal/backend失败；LSP仅防御性Result传播没有当前可达故障证明。它们均有明确结论，未当作“漏修”或未经证明的缺陷自动改动。

## 本地验收历史

下表是交付前冻结源码的已执行验收，保留失败与修复重跑原件。它不替代最终main提交CI，也不把后续CI修复结果倒填为原命令结果。

| 范围 | 已验证结果 |
|---|---|
| 框架全workspace、all-targets、all-features | 3671 passed、0 failed、3 ignored；fmt、两类Clippy、no-default及demo10编译均exit0 |
| CLI七项Rust/GUI门禁 | 全部exit0；workspace 2063 passed、0 failed、15 ignored；GUI 232 passed、0 failed、3 ignored |
| SDK正式全语言入口 | exit0；TS 380 passed／1 skipped；Python 463 passed／1 skipped另18 subtests；Java 393 passed／1 skipped |
| SDK真实Host定向用例 | TS2、Python2、Java1方法覆盖两个窗口，均无skip；Host使用SDK既有`f7bac62e`框架pin |
| EKO前端 | 格式、lint、build均exit0；292 tests passed |
| 官网 | 39 tests passed；discovery sync/check、site check、build均exit0；286 routes |
| CLI双语文档 | 60 pairs、50 ADRs，exit0 |

Rust totals排除嵌套子进程的filtered重复summary；ignored/skipped单列。编译warning为0，预期错误fixture运行日志可有warning。六个独立定向实现审查通过；R1独审发现的原子性、Paused unsafe、已接受取消三项均先取得真实失败回归再修复，额外orphan边界在最终全量测试覆盖。

仅本机原件定位：本次工作区的 `../evidence/commands.json`、`../evidence/completion-audit.json`、`../evidence/final-build-inputs.json` 与各原始log，以及 `../verification-sdk/`。这些路径相对四仓交付工作区布局，不随顶层报告发布；原件不可获得时只能引用报告所述历史范围，不能据此宣称重新执行。

## 交付阶段的额外修复与验收

真正发布时的 fresh 扫描先发现 Windows `clipboard-win 5.4.1` 已 yanked，后发现新进入数据库的 [RUSTSEC-2026-0386](https://rustsec.org/advisories/RUSTSEC-2026-0386.html) 影响 Tokio 1.53.1。它们均未被豁免：CLI 只升级 clipboard-win 到未撤回的5.4.3；三个Rust仓都定向升级Tokio到官方已修补的1.53.3。各次定向变更的其它包块和全部依赖边保持，Cargo重解出的无关边变化排除，并由 full locked metadata 与独立源审复核。

| 仓库 | 本次最终锁 SHA-256 | 风险与 fresh 扫描范围 |
|---|---|---|
| Framework | `f76fe10e64ec3faf062ff5bc7a66d6cd82f82cf95e989c824f40fe6f244d234c` | Tokio以外678包块不变；actual/origin audit0；独立保留原49组／107身份duplicate合同，fresh deny四检查自然0 |
| CLI | `cca90ea328dbdbb5ebf819b5afb8845714c202e5f5fd1b9c032811c35f0e1307` | Tokio以外838包块不变，clipboard5.4.3保留；consumer仍`47bb8f6a…`；actual/origin均仅原六项warning，scanner1／精确gate0，无yanked或0386 |
| SDK | `822ee9a6936f25978283d38d4d07e4e8d7e006ec5834bfeaf80542f79e1ed52e` | Tokio以外655包块不变；fresh audit0；SDK自身原49组／107身份duplicate合同精确不变，fresh deny四检查自然0 |

上述扫描观察官方RustSec数据库`84d943a6deeb8cc6c6f632531aaf4f1c62275239`、1351条advisories；各自既有policy/ignore保持，0386未忽略，未新接受风险。Framework、SDK的独立duplicate合同互不继承，不能用于接受新的advisory、compiler warning或其它诊断；CLI也没有继承它们的例外。原拒绝、HTTPS网络失败及scanner自然出口均保留。

Framework远端CI还暴露临时FakeDocker可执行文件的Linux `ETXTBSY`。后继只把测试脚本迁为跟踪的不可变0755 fixture，每实例仍用私有UUID目录的mode/log；原故障注入分支与输出原样迁出（仅模式读取／插值方式变化），产品重试、超时、断言均不变。30项Docker定向测试及独审通过，后继精确main的Linux与Windows检查也已通过。

| 新锁最终本地门禁 | 实际结果 |
|---|---|
| Framework完整适用门禁 | fmt、全量Clippy、panic lint、no-default均0；workspace/all-targets/all-features **3672 passed、0 failed、3 ignored** |
| CLI完整七项验证 | 全部0；workspace **2063 passed／15 ignored**，GUI **232 passed／3 ignored**，均0failed；原六policy单测0 |
| SDK完整16项执行步骤 | fmt、Clippy、panic lint、workspace、no-default、doctest、Protocol／Host／bridge／facade矩阵、合同及三语言正式入口全部0；workspace **523 passed、0 failed／ignored**；28个合同profile中**113项blocking输出匹配，4项非阻断inventory遥测漂移保留**；新Host下TS **380 passed／1 skipped**、Python **463 passed／1 skipped，另18 subtests**、Java **393 passed／1 skipped**；三个quickstart实际执行0 |
| 官网正式源同步与完整verify | source-aware sync/check及全部入口0；**290**框架镜像逐字相同，**120**CLI源文档hash匹配；保90个ADR对、8个EKO投影，新增ADR0049两语2项／更新既有hash10项；**39**单测、**306**静态路由、E2E **15 passed／1既有desktop skip** |

SDK新Host SHA-256为`d167b366a0288c91ce9debca8725b11425f05b22e4fb40bc8b280e68291b389d`，真实语料观察到同一二进制；旧Host及旧失败日志保留。合同生成器的rustdoc反射使用既有`f7bac62e`框架pin及其独立锁，与本次SDK根锁编译的Host运行时分别记账。4项非阻断遥测为public-api、parity-manifest、full-facade-catalog及inventory-telemetry；没有把它们写成117项全匹配，也未更新accepted合同。门禁期间唯一工程输入变化是AGENTS中历史A03仅执行no-deps的文字澄清，Rust、manifest和锁输入保持；最终非`.echo-semantic`源码摘要`56b1a7e62931cc9edb5c58e3a7be71bd34baf408d9860101da82f8def8e6da9c`已绑定公开main源码祖先`449934d22f3ab69b867f02cfb64565e073c8660e`，后继只修改三份治理元数据，不将候选树执行冒充后来Git提交的执行。

原宿主NO_COLOR与Playwright FORCE_COLOR冲突的首轮warning保留；仅移除冲突环境变量、原样完整重跑后无warning。磁盘不足前主动中断的编译也保留signal及原日志；只在进程检查后退役本任务可重建的旧build cache，源码、原始收据、失败日志及SDK旧Host原件均保留，后继用相同入口重新验证，不把中断计为通过。

## 远端交付状态

四个子仓的最终精确main均已通过远端CI；本提交包含对应四个gitlink、完整inventory及本报告。

| 仓库 | 本轮main提交／PR | 精确main CI及绑定 |
|---|---|---|
| Framework | `51726454985fd6f064b21a7e9c539e22d494c987`；[PR199](https://github.com/EchoYue-lp/echo-agent/pull/199)、[PR200](https://github.com/EchoYue-lp/echo-agent/pull/200) | [run39984523075](https://github.com/EchoYue-lp/echo-agent/actions/runs/39984523075) **success，7／7** |
| CLI | `c262cfed8e77abed38a27c1fff93547997ec9486`；[PR13](https://github.com/EchoYue-lp/echo-agent-cli/pull/13)、[PR14](https://github.com/EchoYue-lp/echo-agent-cli/pull/14) | [run39989318123](https://github.com/EchoYue-lp/echo-agent-cli/actions/runs/39989318123) **success，4／4** |
| SDK | `0930eef9f878cf436ed88384030b6bc03d6f6c53`；[PR10](https://github.com/EchoYue-lp/echo-agent-sdk/pull/10)／[PR11](https://github.com/EchoYue-lp/echo-agent-sdk/pull/11)／[PR12](https://github.com/EchoYue-lp/echo-agent-sdk/pull/12)／[PR13](https://github.com/EchoYue-lp/echo-agent-sdk/pull/13) | [run39997245246](https://github.com/EchoYue-lp/echo-agent-sdk/actions/runs/39997245246) **success，9／9**；永久源码锚`449934d2`是最终main的祖先，非临时分支SHA |
| 官网 | `a926ef7cf10bdc03cc0a19b2e406cc6164477485`；[PR8](https://github.com/EchoYue-lp/echo-website/pull/8) | [run39991323673](https://github.com/EchoYue-lp/echo-website/actions/runs/39991323673) **success**；manifest精确绑定FW51726454／CLIc262cfed |
| 顶层 | 本提交包含本报告、inventory及上述四仓gitlink | 无独立workflow；四个gitlink与已推送、精确CI成功的四个main逐项一致，diff check通过 |

SDK远端Windows job另通过原生Host执行TS、Python、Java各1个具名路径consumer，三项均通过、各0失败／0case skip；这只覆盖指定路径窗口，不等于三语言完整Windows回归。上述本地各1项平台skip仍按原结果保留，不混合平台计数。

本轮遇到的旧CI失败保留为原失败，不借邻近提交的green：Framework旧`a39a4cde`失败于Linux fixture；SDK原`4acd5b81`失败于源摘要与同次语义对象，`3daaa573`随后又被新Tokio advisory阻断；CLI旧`fb98ffc2`的Linux测试与前端通过，但quality准确拒绝新advisory。SDK中间源码提交`449934d2`的run39996250155被既有concurrency策略取消，不计为成功。最终交付只以上表最终提交及各自CI为准。

## 保留边界

原primary checkout的分支、未提交文档、子仓指针及未跟踪历史记录均保留；修复在独立任务树完成，本报告不授权清理旧补丁或删除恢复材料。顶层只保存跨仓阶段审查与交付事实，各子仓ADR、合同及正式文档继续作为各自产品事实源。

本地GUI feature检查与单元测试未证明原生窗口、跨平台执行或远端工具真正完成；真实Host证据只覆盖列明窗口。SDK固定框架pin与CLI使用新框架是两条不同依赖证据，不混称同一集成版本。原会话曾有Echo Semantic预检失败，这不是通过记录。用户已关闭本机会话插件；本轮未调用或伪造SessionStart，也未启用它。SDK原已配置的远端canonical semantic CI保留，补齐真实源码证据后以其实际结果验收，不关闭或弱化该检查。

审查来源：会话`01a1630a-bf4a-7812-b9dc-6cd0e4b20123`的2026-10-22原始rollout第48行全量inventory、第168行AST对照、第179行类型诊断、第538行最终审查；后继本地验收为本轮冻结源码、原始命令退出与独立审查记录。最终交付以本节精确main、各自CI及本提交四仓gitlink为准。全部旧补丁组审查与已采纳问题修复闭合；明确不采纳的候选保持上述结论，旧补丁与恢复材料继续保留。
