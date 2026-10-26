# EKO Issue / Finding 当前逐项台账

本轮观察：2026-10-26 **15:36:53–15:38:05 Asia/Shanghai**（API原始时间为07:36:53–07:38:05 UTC）。范围是旧59条资料台账中已具名对象及其明确关联：7个Issue、16个PR、五仓main ref；共28次只读GitHub GET，全部HTTP200、gh自然退出0。不是全仓Issue普查，也不重审已交付14组/718处旧补丁。

结论：**framework #32–35、#188、#189仍open；#55已closed/completed。16个具名PR均merged，五仓main与旧快照相同。** #188本地Finding已resolved，但本轮远端main Git对象中该Finding仍open；#189是来源缺口，SDK managed首Host UNKNOWN是有界risk_accepted，不能相互替代。本文不执行comment、close、reopen、merge、push、fetch、发布或部署。

## 当前切点追加与状态维护交接

2026-10-26 主线程已核18份里程碑清单，交付图与全部14份计划首次入Git核验共15条命令自然0；只证明文档结构/绑定，不是业务重审。CLI设置/import的9路径已本地提交 `262e7a6`，侧栏清单3路径随后提交 `96495df`；CLI `80f3a72`、framework `05023bc`、SDK `525453f` 只同步AGENTS规范，所有Rust字节未因这些提交改变。未push/merge/gitlink晋升，不将本地提交扩大为main/CI或新原生Rust资格。

P1/P2/N1/N2具名恢复验证绿保持；长reflect第三次仍失败，处于排查决策门，完整13项和原文/有效要求两项原失败继续开放。本文不推断决策已完成，不新增修复/风险豁免或外部Issue操作，也不更改原GitHub观察切点。ubuntu真实成功与echo支持义务仍分别验收。

本线仅在next-actions和本文追加当前澄清，随后冻结其它交接文档，不改Design、Delivery Map、全部Plan/hash、旧reconciliation、tool-inventory或主线程独占的MASTER/AGENTS。主线程接管这两份后继状态与候选SHA；next-actions旧808…和本文旧5f364…不再代表新增字节，提交前须按实际文件刷新。本线不stage/commit/gitlinks，不运行Cargo或重新GET。

## 观测方法与固定输入

原审计为冻结宿主树中的 `docs/supreme/audits/eko-issue-finding-reconciliation-2026-10-25.md`，SHA `96cbf37d46d2ef0404744506d108f23dff32b6bc9013cfb4bee7d8d67559c0a7`；本轮完整读取其后继段但不改原件。旧阶段 [reconciliation](reconciliation.md) SHA `7c5bd10669996b590dbc6ad37cd71809ddb06ffcc89d14c2eebe1c03664bdbe7` 保持不变，旧API观察和候选时态不回写。

旧59条ledger/15个PR的来源根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktree-backups/goal-maintenance-2026-10-24/issue-finding-reconciliation/`：

| 来源 | SHA-256 |
| --- | --- |
| ledger.json | `982fde95c45b97f4ffed0f56e8bacfd7308681a0c64636b4cc037aebf3bfc384` |
| ledger.md | `057d7b3e457b6fa41be6850b89ce9796e4d2c30d4f02c3e4401c31497f7ec452` |
| pr-state-ledger.json | `5a998453b6b76f1d5b24637d85852abe39d5924e23206fc489894e4b3a0881a3` |

使用 `gh api --hostname github.com --method GET --include <endpoint>`。Issue端点为 `repos/EchoYue-lp/<repo>/issues/<number>`，PR为 `.../pulls/<number>`，main为 `.../git/ref/heads/main`。未运行auth/status、未读取token、凭据或环境变量；不保存/打印请求认证头。只保留响应中的对象身份、状态、时间、merge/source SHA及GitHub request ID，详见文末回执表。Issue返回均不是同号PR，PR的base repo均与行内仓库一致且baseRef为main。

Git核对只对本机已有精确对象执行 `git show <observed-main>:<finding-path>` 和 `git merge-base --is-ancestor <merge-sha> <observed-main>`，没有移动任何ref或改变checkout。本文“main对象内状态”指API所得精确SHA的本机Git对象，不是工作分支名或旧origin/main推断。该API观察时根HEAD为superproject a91d5deb、F9c5e202d、CLI780ddd0、SDK0930eef9、websitea926ef7c；后继本地提交见当前切点追加，dirty运行时代码不由HEAD代替，也不由本次API刷新取得验收资格。

## 七个Issue与本地/主线Finding状态

下表完整仓库均为 `EchoYue-lp`；状态字段顺序是“当前根正式Finding / 本轮观测远端main对象内Finding”。API的updated_at/closed_at原样使用UTC，不能把它误认为本轮查询时刻。

| Issue完整身份 / GitHub id | 本轮外部状态 / state_reason | updated_at; closed_at（UTC） | 本地 / main Finding |
| --- | --- | --- | --- |
| [EchoYue-lp/echo-agent#32](https://github.com/EchoYue-lp/echo-agent/issues/32) / `5440931501` | open | 2026-09-13T15:25:43Z; 未关闭 | `finding.a2a-advertised-capability-binding`：open / open |
| [EchoYue-lp/echo-agent#33](https://github.com/EchoYue-lp/echo-agent/issues/33) / `5440931654` | open | 2026-09-13T15:25:45Z; 未关闭 | `finding.a2a-stream-cleanup`：open / open |
| [EchoYue-lp/echo-agent#34](https://github.com/EchoYue-lp/echo-agent/issues/34) / `5440931834` | open | 2026-09-13T15:25:47Z; 未关闭 | `finding.a2a-task-id-admission-authority`：open / open |
| [EchoYue-lp/echo-agent#35](https://github.com/EchoYue-lp/echo-agent/issues/35) / `5440931970` | open | 2026-09-13T15:25:48Z; 未关闭 | `finding.a2a-terminal-authority`：open / open |
| [EchoYue-lp/echo-agent#188](https://github.com/EchoYue-lp/echo-agent/issues/188) / `5878932374` | open | 2026-10-16T05:32:46Z; 未关闭 | `finding.framework-yanked-zip-ci`：resolved / open |
| [EchoYue-lp/echo-agent#189](https://github.com/EchoYue-lp/echo-agent/issues/189) / `5879381252` | open | 2026-10-16T06:19:00Z; 未关闭 | `finding.framework-historical-source-observation-retention`：open / open |
| [EchoYue-lp/echo-agent#55](https://github.com/EchoYue-lp/echo-agent/issues/55) / `5440934637` | closed / completed | 2026-09-24T11:58:26Z; 2026-09-24T11:58:26Z | `finding.extension-cleanup-settlement`：resolved / resolved |

**#188**：本地修复三链确实存在，不能再按旧10月24台账的“待正式落盘”开一次相同修复。生产ZIP修复已由PR190 squash为9886502进入main；原23项本地门禁与main517的run39984523075是原Evidence记录的历史资格，本轮没有重跑或刷新CI。后继本地语义结算4399c344与F0a181的这三个正式文件逐字相同，F0a181是当前根F9c5祖先；4399自身不是F9c5祖先，不能把同字节采用写成原提交直接合入。相同三文件在远端main517仍是旧open版本，因此**本地语义结算、生产修复合入、正式对象main晋升、外部Issue关闭是四个不同层级**。下一步由协调者在允许交付时采用已存在三文件增量并按所属门禁晋升；当前只保留候选，不重复代码开发，不关闭Issue。

**#189**：repair/verification/rereview字段虽有引用，三者支持的是当前7110源码重观察；七个原历史树仍不可得、原因UNKNOWN，这个Finding在本地和main均open。用户现码审查决定不是风险接受，也不证明恢复旧对象；不猜测GC、Goal删除或人为丢失。本轮不重新展开无界历史恢复。后续只有原对象恢复或具名新来源证据才改变该缺口，当前代码交付仍按已裁决边界继续。

**A2A #32–35**：本地与main均open，三修复链均为空。用户批准的是具名退役，不能用删除候选或Issue关闭制造历史缺陷已修复。消费者先解除依赖、精确main/CI与保留能力验收、framework物理退役、正式semantic结算均仍各有前置；最终semantic自然1仍未放行。外部echo-semantic工具合同修复被排除，不代表该门禁豁免。

**#55**：当前GitHub为closed、state_reason=completed，closed_at为2026-09-24T11:58:26Z。本地Finding与远端main均resolved；正文和rereview中“仍OPEN、待PR/CI”的段落是旧切点文字，不用它覆盖新API结果。已有三链明确覆盖framework transport/client/manager、retained preparation/construction cancellation；**不能继续把整类preparation当从未实现**。原T8“通用复用路径待核”的剩余动作只针对尚未证明的具体consumer/责任边界；SDK Host、LSP adapter、真实第三方MCP和其它独立owner未因Issue55关闭取得资格。本文不因标题含SDK LSP就宣称它们全部通过，也不修改正式对象的历史正文。

## 修复、验证、复审三链逐项读取

计数仅表示字段中的引用数以及文件已读到，不是测试通过数，不是“字段非空即可关闭”。下面的metadata与文件SHA来自本轮根内实际读取。

| 根仓库 / Finding | 本地 / main状态 | 修复/验证/复审引用数 | 根Finding SHA-256 |
| --- | --- | --- | --- |
| echo-agent / `finding.framework-yanked-zip-ci` | resolved / open | 1/1/1 | `6f14b1723b833fb2f32da191c0e4568f75a0ed753a924d0ab9663698c19b034c` |
| echo-agent / `finding.framework-historical-source-observation-retention` | open / open | 1/1/1 | `5f9487b800b188ed58bbbace33ed4acfc8900f06cbacd5fc38fba204bfa511c7` |
| echo-agent / `finding.a2a-advertised-capability-binding` | open / open | 0/0/0 | `b660b2d5a8b0bd641e5df602f73c45c9b932640b8c338d9a5f4cead71f11e35c` |
| echo-agent / `finding.a2a-stream-cleanup` | open / open | 0/0/0 | `5bc181566b11b448cd5c46475fab2e442687df5dfbc9409dcc0ace3447bbfbe2` |
| echo-agent / `finding.a2a-task-id-admission-authority` | open / open | 0/0/0 | `5fdcfac07cb09f2aa7f440119a8bdda3ce22599cd85d1431b82debbb0f305f41` |
| echo-agent / `finding.a2a-terminal-authority` | open / open | 0/0/0 | `d632adec246611df4bb2b1eebf625ce381689e3cc1923c7166f344b5b9cc8718` |
| echo-agent / `finding.extension-cleanup-settlement` | resolved / resolved | 1/1/1 | `6898142e2fa87764656db4e500db5bb15fb3ab5151e6b07ccd1d7d682f71abb9` |
| echo-agent-sdk / `finding.sdk-managed-restart-passive-host-exit-timeout` | risk_accepted / risk_accepted | 0/0/0 | `5ccf446612b6aa10860452072bc3a1528545a6effc24de76230231959d650953` |

| Finding / 链 | 已读取的唯一引用 | 文件SHA-256 |
| --- | --- | --- |
| `finding.framework-yanked-zip-ci` / 修复 | [evidence.framework-yanked-zip-repair](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.framework-yanked-zip-repair.md) | `4b360005e75f34e69135a53a5d1edbe9a38495a5f788f944a5650177bae7d86c` |
| `finding.framework-yanked-zip-ci` / 验证 | [evidence.framework-yanked-zip-repair](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.framework-yanked-zip-repair.md) | `4b360005e75f34e69135a53a5d1edbe9a38495a5f788f944a5650177bae7d86c` |
| `finding.framework-yanked-zip-ci` / 复审 | [audit.framework-yanked-zip-source-review](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/audits/audit.framework-yanked-zip-source-review.md) | `b87570ad3af3801e8ec7c4d1440bb8a0f667dac0c59c66c8959f2f20ea10bd3c` |
| `finding.framework-historical-source-observation-retention` / 修复 | [evidence.framework-history-current-reobservation](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.framework-history-current-reobservation.md) | `2f9f9887b787eac9af0667500ee79e28d026bc9161ad82a42d42fcb8ab0e49c5` |
| `finding.framework-historical-source-observation-retention` / 验证 | [evidence.framework-history-current-reobservation](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.framework-history-current-reobservation.md) | `2f9f9887b787eac9af0667500ee79e28d026bc9161ad82a42d42fcb8ab0e49c5` |
| `finding.framework-historical-source-observation-retention` / 复审 | [audit.framework-history-current-reobservation](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/audits/audit.framework-history-current-reobservation.md) | `8da9a3343e6206e989f4ca34f5190c9f9c000720d94ecaca6ad67a68afe2e991` |
| `finding.extension-cleanup-settlement` / 修复 | [evidence.extension-cleanup-settlement-repair](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.extension-cleanup-settlement-repair.md) | `317b80b861fa8d206b219c05d576e8bfda26ee402fa73ce1b404d663c6887b3d` |
| `finding.extension-cleanup-settlement` / 验证 | [evidence.extension-cleanup-settlement-verification](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/evidence/evidence.extension-cleanup-settlement-verification.md) | `61a72323b551f0610f93e4fe3ebb6e22a836177eb172c528a96be2978e10fab2` |
| `finding.extension-cleanup-settlement` / 复审 | [audit.extension-cleanup-settlement-rereview](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.echo-semantic/audits/audit.extension-cleanup-settlement-rereview.md) | `715c4a3c1af0c5baf45475a8b38e518be4f76d2593c7390b67a8cbf238724f50` |

#188的repair和verification复用同一个Evidence，是原正式合同，不另造两份证据补数；原Evidence.observed_at和Audit.revision仍保留7110执行/审查时期，后继main资格另节记载。#189三链非空不修复“七旧树缺失”。#55三链覆盖framework边界，不自动覆盖Host/LSP。上述链只按既有Evidence/Audit解释，本文没有新的运行结果或独立实现复审。

## 风险接受与未分配外部对象

| 对象 | 当前处置与已有依据 | 明确不覆盖 / 下一步 |
| --- | --- | --- |
| SDK `finding.sdk-managed-restart-passive-host-exit-timeout` | 本地与main0930均risk_accepted；repair/verification/rereview继续0/0/0。唯一decision_refs为 `evidence.sdk-managed-passive-exit-residual-human-decision`，SHA `c51cac12539d8ad85f8c361d3d7676e642ba02afd4a6b375cfde30a942913b02`，记录真实用户2026-10-22T03:28:19.660Z对精确历史首Host10秒被动退出UNKNOWN的接受 | 不是resolved，不延长10秒，不以kill/retry-as-pass通过，不称原Host exit0，后续同类新失败仍阻断；不覆盖R2其它Host/持久化/资源边界。PR7/main a9资格是原范围，不为新pin/consumer续期 |
| R1 / contract第13节A1–A7 | 旧审计指定CLI fb98ffc/PR13、ADR0049已有process epoch/lease/typed cleanup后继；本轮PR13merged及main祖先均核实 | 没有为该材料新分配Issue号；不重开已交付R1，不把原调查“未测”改成当时已测，也不扩到任意退出/恢复场景 |
| R2本地child scope | SDK4acd5b8/PR10及ADR0041已有三语言local/late/first/失败重试资格；本轮PR10merged及main祖先核实 | 未新增Host终态/父关闭后replay合同，EOF不是业务成功。R2 Host等待/持久化失败/资源保留仍是未验证边界，不能凭managed风险接受或本地child修复关闭；不猜外部Issue号 |
| T8默认source-tail | framework a39a4cde/PR199已有默认/Subagent首清理错误保留；本轮PR199merged及main祖先核实 | 不重复修默认包装器。未具名通用consumer取消假设只做有界可达路径与反例核对；与已闭合#55分开，不先承诺新增Drop/store |
| CLI依赖六风险；framework/SDK各自duplicate合同 | 沿各所属AGENTS及原独立gate接受的精确lock/consumer/source/advisory集合，不是implementation Finding resolved；本轮未运行scanner、未更改policy或SDK pin | 官方scanner自然出口与精确风险gate分开，历史合同不自动接受新源码/锁、没有新增ignore或跨仓继承。风险输入维护不是本次Issue关闭前置已全部完成的证明 |
| SDK四项非阻断遥测 | 库存遥测与accepted合同阻断分开 | 不是risk修复，不扩accepted、不刷新117项产物；本轮不重审其完整API库存 |

## 源码候选、历史合入与过期执行材料

所有“非main祖先”仅表示该本机提交未以祖先关系进入本轮观测main，不据此否认squash或同字节采纳。当前候选内容范围以旧审计、根迁入记录和相应owner为准，本轮不重新实现。

| 精确来源 | 本轮只读核对 | 状态与下一动作 |
| --- | --- | --- |
| F4399c344 ZIP三对象结算 | 不是main517或根9c5祖先；对F0a181的三个正式文件diff0，当前根同SHA | 已本地resolved的正式对象候选；等待获准晋升，不再写“尚待本地结算” |
| F0a181571 | 是根F9c5祖先，非main517祖先 | 已进入本地集成，不能冒远端main；这颗commit树不是物理删除完成证明，原dirty删除候选另由归档/退役owner持有 |
| SDK1eac40fb去依赖候选 | 非main/root0930祖先 | 当前SDK仍旧pin/accepted检出；精确269 identities/39 operations差集及5352/431保留集合按原正式checker/consumer验收，不因文档分支存在自动采用 |
| CLI01137a1e去依赖/console候选 | 是根CLI780祖先，非mainc262祖先 | 已本地采用切片，原独立console实际退役仍待GUI/IM/cron/共享恢复接管和原门禁；不按目录名整删共享能力 |
| Website dc4dc197文档候选 | 非main/roota926祖先 | 旧镜像/宣传只保其原pin，不冒当前合格producer；最终源验收后统一镜像，当前不部署 |
| PR18 / eadd2aa8 | 本轮PRmerged，merge是远端root-main9a96祖先 | 历史压缩交付与tree/用户有效尾节按原证据保留；不冒本轮真实模型、native或部署 |
| 旧SDK Plan06 ready/175487 pin与旧unified kernel计划 | 原拆仓/纯Protocol/精确Host pin/accepted与telemetry分离及共享admission已有后继；不是从零待实现 | 旧ready/旧pin作为历史来源保留。kernel不重建，旧ADR depth差异已由本轮完整Goal和当前Design裁决，实际接线仍由原owner验收 |
| 旧“新增IM尚未指定”、旧GUI已验/未收结果、旧外部运行路径 | 已被个人微信选定、根内迁入、后继raw及唯一开发根替换 | 过期作为当前执行依据；原报告/失败与恢复资料不删除，不用旧receipt声称当前根通过 |
| Exa4343f0b7与pre-retirement tag | 继续原明确保护边界 | 本地hold，暂不push/合main；不是为收尾必须合并的候选。本文没有重置tag或处理保留refs |

## 五仓本轮远端main与具名PR

| 精确仓库 | 本轮API main SHA | 读取结果 |
| --- | --- | --- |
| EchoYue-lp/eko-workspace | `9a96ff9675c340c9d6ef18d63dd2a2b16663962e` | GET 200 / exit0 |
| EchoYue-lp/echo-agent | `51726454985fd6f064b21a7e9c539e22d494c987` | GET 200 / exit0 |
| EchoYue-lp/echo-agent-cli | `c262cfed8e77abed38a27c1fff93547997ec9486` | GET 200 / exit0 |
| EchoYue-lp/echo-agent-sdk | `0930eef9f878cf436ed88384030b6bc03d6f6c53` | GET 200 / exit0 |
| EchoYue-lp/echo-website | `a926ef7cf10bdc03cc0a19b2e406cc6164477485` | GET 200 / exit0 |

这里没有为五仓重新执行CI查询、Rust/Node或消费者验证；旧报告的7/4/9/1 jobs及399系列run仍按原观察时间引用。main SHA未变不等于根dirty源码、当前GUI binary或后继consumer已经通过。

下列16个PR为旧15项加SDK风险接受证据明确关联的PR7；不是任意全仓PR扫描。所有GET返回closed且merged=true。最后一列在所属本机仓对**API返回的merge SHA与该仓本轮main SHA**执行 `merge-base --is-ancestor`，16项均exit0，无fetch。PR日期为API UTC原文。

| PR完整身份 | state / merged | merge_commit_sha | merged_at（UTC） | main祖先检查exit |
| --- | --- | --- | --- | --- |
| [echo-agent#178](https://github.com/EchoYue-lp/echo-agent/pull/178) | closed / true | `e5372b8ca3dc308ce8dde4a0092ea58b8d7dd21d` | 2026-10-10T02:47:35Z | 0 |
| [echo-agent#179](https://github.com/EchoYue-lp/echo-agent/pull/179) | closed / true | `1486d2f4a26adbefab393679c58da2804e5f4a75` | 2026-10-10T04:05:51Z | 0 |
| [echo-agent#199](https://github.com/EchoYue-lp/echo-agent/pull/199) | closed / true | `a39a4cdea2e5142479bce0894dd68c3333cf5da7` | 2026-10-23T12:37:18Z | 0 |
| [echo-agent-cli#10](https://github.com/EchoYue-lp/echo-agent-cli/pull/10) | closed / true | `f98f50985c9dbc8aad8c1c7e873a8997fe5024f0` | 2026-10-10T04:26:46Z | 0 |
| [echo-agent-cli#14](https://github.com/EchoYue-lp/echo-agent-cli/pull/14) | closed / true | `c262cfed8e77abed38a27c1fff93547997ec9486` | 2026-10-23T16:44:40Z | 0 |
| [echo-agent-sdk#10](https://github.com/EchoYue-lp/echo-agent-sdk/pull/10) | closed / true | `4acd5b81cf3f64b17a7aed546d4aaa2e145e52cb` | 2026-10-23T12:37:16Z | 0 |
| [echo-agent-sdk#11](https://github.com/EchoYue-lp/echo-agent-sdk/pull/11) | closed / true | `3daaa573a60d83e7064ff0796c48bc67ddd6297b` | 2026-10-23T15:28:51Z | 0 |
| [echo-agent-sdk#12](https://github.com/EchoYue-lp/echo-agent-sdk/pull/12) | closed / true | `449934d22f3ab69b867f02cfb64565e073c8660e` | 2026-10-23T17:34:56Z | 0 |
| [echo-agent-sdk#13](https://github.com/EchoYue-lp/echo-agent-sdk/pull/13) | closed / true | `0930eef9f878cf436ed88384030b6bc03d6f6c53` | 2026-10-23T17:42:13Z | 0 |
| [echo-website#4](https://github.com/EchoYue-lp/echo-website/pull/4) | closed / true | `b1609b126014bf0cb120e68a2a665a9c64b8fa54` | 2026-10-10T04:30:34Z | 0 |
| [echo-website#8](https://github.com/EchoYue-lp/echo-website/pull/8) | closed / true | `a926ef7cf10bdc03cc0a19b2e406cc6164477485` | 2026-10-23T16:59:15Z | 0 |
| [eko-workspace#18](https://github.com/EchoYue-lp/eko-workspace/pull/18) | closed / true | `eadd2aa86f6674c67124b3218f0d1a01bf254be6` | 2026-10-10T04:43:55Z | 0 |
| [echo-agent#190](https://github.com/EchoYue-lp/echo-agent/pull/190) | closed / true | `9886502c83d6b1f6fa9212eb221db0e5874deaee` | 2026-10-16T06:37:26Z | 0 |
| [echo-agent#200](https://github.com/EchoYue-lp/echo-agent/pull/200) | closed / true | `51726454985fd6f064b21a7e9c539e22d494c987` | 2026-10-23T16:10:13Z | 0 |
| [echo-agent-cli#13](https://github.com/EchoYue-lp/echo-agent-cli/pull/13) | closed / true | `fb98ffc26a51a526018feb2a6adf3ab23b7b117f` | 2026-10-23T15:27:56Z | 0 |
| [echo-agent-sdk#7](https://github.com/EchoYue-lp/echo-agent-sdk/pull/7) | closed / true | `a9f267f636cd4d1553a8bff335c3203ab0b8a5ac` | 2026-10-22T02:01:33Z | 0 |

## 可立即接续的具名工作

此前验收反馈（2026-10-26，由总调度派发转达）的切点是：用户已确认原生布局方向；`ubuntu@111.229.245.175` 在系统终端可用密码登录，EKO却停在密码提示且目录验证未通过。该修复前失败保留；**当前已由下节原生epoch10用户确认登录测试与创建成功替代这两项的阻塞状态**，不能再将旧失败描述为最新结论，也不能反向把全部SSH矩阵改为通过。原 `echo` 账号探测与历史原件不改写。

后续新增或修改的文档说明统一使用中文；代码、API/协议固定标识及原始日志保留原文，历史证据不因翻译而改写。上述反馈只补充产品验收状态，不改本台账15:36–15:38的GitHub观察切点或任何已完成对账结果。

1. **#188正式对象晋升对账**：协调者保留本地三链与4399/F0a同字节证明。用户已允许且要求及时本地里程碑提交，只有主线程协调暂存/提交；未来main合流、push及外部Issue关闭仍未获授权。当前不重修ZIP，不在本线操作正式对象或外部Issue，本地提交不能冒正式main晋升。
2. **#55旧状态文案与覆盖边界**：所属文档owner可在独立授权切片追加本轮closed观察，不改写旧OPEN历史。工具治理只验证具名尚未覆盖的consumer/Host/LSP组合；不重新实现已证的framework retained preparation/construction owner。
3. **#189来源缺口保留**：不把当前源码重观察的非空三链改成旧树恢复；无新具体原件则保持open/UNKNOWN。该项不重新阻塞已批准的现码交付，也不因此豁免当前新增失败。
4. **#32–35退役结算继续排在消费者之后**：EKO/SDK去依赖、剩余合同/GUI/IM/cron/恢复和精确main/CI先成立，再处理framework删除与正式semantic。外部echo-semantic修复仍排除且semantic1继续阻塞，不能从Issue台账“已完成”反推退役通过。
5. **当前根目录产品链继续由原负责人执行**：核心epoch10的2f7d generation产品修复后原P1及P2/N1/N2、5项邻近各退出0；durable_reflect默认栈SIGABRT，完整13未过，原文两项原失败仍开放。SSH登录测试/创建及侧栏刷新已获用户确认，148文件设置/信息气泡实现已载；完整视觉、HITL、终端/远程项目/扫码与完整门禁仍归 [下一动作表](next-actions.md)。保留旧d205失败及原生PID43779/b330；正式性能/完整矩阵仍按总调度窗口。

### 原生epoch10与后继修复的具名对账

本节是产品验收后继增量，不改变15:36–15:38的GitHub观察切点，不新增正式Finding状态或外部Issue操作。

| 本地事项 / 来源 | 本轮采用的真实结果 | 当前状态与下一工作 |
| --- | --- | --- |
| 原生制品：`gui-native-epoch10/binary-retained.receipt.json`及`ssh-password-debug-20261026/handoff.json` | Rust构建退出0、`warnings: []`；二进制 `b330bf25ce33a0eab716673ee498a23acbc66bc64173ed20142b9b43cfcd7ae2`、consumer `d205acd2b9e31c8d7b99bcbdc706d76686b04043e8171e953f88b83fa44d547f`，制品保全后clean0、移除6.8GiB | 限定构建已成立，不能扩到后继源码、完整矩阵或全部原生交互 |
| 启动：`gui-native-epoch10/native-runtime/startup.receipt.json` | 总调度实际启动PID43779、窗口163930；前台元数据独立确认，真实Tauri读取已观察 | 是启动/读取事实；后继核心源码不在该二进制范围，全局与新工作空间memory不冒全部隔离 |
| 受控模型第二版：`native-steer-debug-20261026/fixture-v2/交付回执-v2.json`与`http-smoke/receipt.json` | 30项离线检查通过，JSON/SSE两项冒烟通过、服务退出0 | 受控协议合格，不冒真实provider或全部HITL场景 |
| SSH：`gui-native-epoch10/native-runtime/ssh-human.receipt.json` | 用户明确“登录测试与创建都成功”，目标 `ubuntu@111.229.245.175:22 /home/ubuntu/ssh-test`，绑定上述原生制品/前端 | 已解决本次登录测试及远程工作空间创建/进入两项；完整终端中文/resize/Ctrl-C/exit/多会话等仍未验。不据两项成功将Plan06/项目结果或外部Issue关闭 |
| 侧栏同步修复与用户确认 | 原store只set current未更新list，根GUI4项失败后转绿、独立复审0项问题；用户明确“刷新后已显示ssh-test”（总调度核验转达） | 本次侧栏显示已确认，不再记待重载；原生PID43779/b330与SSH连接保留，没有重建SSH，不扩为全部生命周期矩阵通过 |
| HITL用户“其它错误”澄清：`hitl-human-clarification.json` | 实际指 `[Info] Execution path: formal_plan`；总调度移除普通chat气泡但保留journal，114项回归通过、lint0，该实现后继已加载 | 旧回执的pending仅保其观察时态；现已载入不等于完整弹窗矩阵通过，不伪称原工具有同一错误 |
| HITL协议：同目录`hitl-protocol-observations.json` | sequence34为input_request，35为同request/response_target的release，36为原human_in_loop工具结果；澄清回执确认success=true且无error | 该次真实协议往返成立，不扩为审批/输入/选择、过期/取消、竞争/焦点等完整矩阵通过 |
| 统一前端增量已加载 | settings-layout交接SHA `71b16bf465544cc8a970bca182047fc3773a524afd0700872617823fe158e09b`：148文件、manifest `25d8584a2c05450e21e99ffb39f222abc5eeb71308ab7848b402e18c27315ef9`，154测试通过、构建0；设置去overview/默认模型/5组分类及气泡修复已载（总调度核验） | 保留初次TS2、组合测试1及两项既有Vite同chunk提示，不称零warning。独审C/I0，原“高级设置分组保持展开”的基线Minor保留；实现加载不扩为完整设置视觉/交互矩阵通过 |
| 微信P1 / 核心epoch09历史 | 旧batch-receipt记录P1退出101，原10秒pending超时及6次accepted保留；末二进制保全、clean0约7.2GiB | 这是修复前失败切点，不能继续写成当前P1；后继通过见下行，原失败不抹去 |
| 核心epoch10 / generation产品修复 | renderer-fix-cut consumer `2f7d326dcd89e9e02b7f9361b648a29afb745f0312feb17a7392375bd9be81e4`；原注册回执携带generation cut，物理回合已返回且TaskRun终态时退休原被动renderer，保留后继generation。原P1选中1、通过1/失败0、退出0；P2/N1/N2及5项邻近各选中1、分别0 | 不再记“无产品修复”；旧diagnostic-cut仅为历史假设。原生b330仍绑定较早Rust，未因前端刷新采用该修复 |
| 完整13项与原文/有效要求 | 首个durable_reflect原run的exitCode=null、signal=SIGABRT，默认栈失败保留；原文冷重开与有效要求真实压缩两项原失败仍开放 | long-control-steps的包装exit1不能冒自然测试出口。完整13未过，新增邻近5项不能替代原清单；recovered-original-13恢复回执executed=false，不把旧命令复原当新执行。完整Goal/Design不缩小 |
| 工程回收累计状态 | internal-reclaim/postcheck：累计51=外41+内10，原路径23,881,756,672B（22.24GiB），failures为空；剩9注册路径=5主检出+4host | 本轮仅读回，不新增删除；旧外41批次/根内15是历史切点，不再作为当前剩余。refs/stash/tag/Exa/用户数据与恢复包保留 |

路径均相对根CLI `.echo-semantic/local/`。本线已读取上述指定小回执与协议摘要；侧栏/设置的当前收件状态按总调度已核实派发标注，不伪称本线执行。没有重新GET、启动Cargo/进程、生成大包、修改正式Finding三链或关闭Issue；旧失败保留，新成功只在明确范围结算。

上一阶段读取的回执身份如下，保留原观察切点；旧diagnostic-cut或pending字段不能覆盖本次renderer-fix-cut与用户刷新反馈。文件可由原负责人追加后继结果，后继变化不自动为旧SHA续期。

| 回执（上述根CLI证据目录内） | 本次SHA-256 |
| --- | --- |
| gui-native-epoch10/native-runtime/startup.receipt.json | `69575a28f00aa218505a192edc7bc8f2f0ecda207ec13bdaa139e5ce4d3c0fb8` |
| gui-native-epoch10/native-runtime/ssh-human.receipt.json | `c7b7687944c2a266b3911d06eac8f6ffe554199b5fc73278ac4f2b7932ce9460` |
| gui-native-epoch10/native-runtime/hitl-human-clarification.json | `dc9df76409d4118a2893a45828d7226b3802f4d52294641087fa5be8f17ef5f8` |
| gui-native-epoch10/native-runtime/hitl-protocol-observations.json | `ac128a98175196b9355322693f8a9c39b2992f392b9ebd6c0bb7aeef9958a642` |
| ssh-password-debug-20261026/handoff.json | `d59fd4cbd7f39da14d472a1996ab5dd7ae77dd276675fb4328344ba651fb357f` |
| core-repair-20261026/epoch-09/batch-receipt.json | `22eca73eac90d3d7567e0062e8bd91d609136a560eda9f5ecf370b0d113072fa` |
| core-repair-20261026/epoch-10/diagnostic-cut.json | `a81ff2af808183720c36409c8960613c5bc4f2e247bce30b060bbb2f5e1d59ab` |

本次只检查三份文档增量：交付图文档校验退出0、20条本地文件链接可解析、空白与git diff --check通过；旧reconciliation SHA未变。不重新运行已完成的工程检查、广泛GET或大归档，也不修改设计范围与计划生命周期。

后继当前证据（2026-10-26）：

| 输入 | SHA-256 / 实际结论 |
| --- | --- |
| `settings-layout-20261026/handoff.json` | `71b16bf465544cc8a970bca182047fc3773a524afd0700872617823fe158e09b`；前端148文件manifest `25d8584a2c05450e21e99ffb39f222abc5eeb71308ab7848b402e18c27315ef9`，154测试/构建0，两项既有Vite提示保留 |
| `core-repair-20261026/epoch-10/renderer-fix-cut.json` | `26f09c4cb7025e02a4ed75614e7a5320fd818aacac34b4a28abb3deb192416e9`；generation退休产品修复，consumer2f7d。文件的raw_product_fix=false不等于没有该launcher产品修复，也不证明原文要求已修 |
| 同目录 `p1-renderer-fixed-run.json` | `b36f10d1c93aaf9b6f12eeed068f1b7c1aa9fcb35072c1fa736d5661f2efffe4`；原P1选中1、通过1/失败0、自然退出0，无signal |
| 同目录 `recovery-control-steps.json` | `806e48ba466298467b54c240532406aa8d948700a8ba54429045595c0bc01d83`；P2/N1/N2及5项邻近各选中1、分别退出0，不计成原13已全过 |
| 同目录 `long-control-1-run.json` | 原durable_reflect自然exitCode=null、SIGABRT；包装long-control-steps的1只是失败汇总，不冒原signal为普通exit1 |
| 根 `workspace-convergence-20261026/internal-reclaim/postcheck.json` | `2b4144928ec07115a7b2f927621fae976dd007cf47ec92813fe5e8380c95575c`；51=外41+内10，22.24GiB，剩5主检出+4host，无failure。本次未新增删除 |
| 根CLI `recovered-original-13/recovery.receipt.json` | 路径在 `core-repair-20261026/` 内，原13精确命令/计划/proposal已恢复；executed=false只证明恢复，不证明新执行 |

下一独立工具治理切片已经按当前源码收敛为“primary verification实际ModelRequest.tools曝光”：现有tool_exposure预算用例从真实目录过滤后统计，unattended主路径也已填visible/allowed，缺的是该consumer最终模型请求的直接证据。复用原factory/TaskRuntime/snapshot，先核默认与非空allowed收窄/无交集控制，原预算和资格上界不变；不是重做全盘目录或新建曝光机制。精确源码与验收边界见[下一动作表](next-actions.md#下一可独立交付切片primary-verification实际模型请求的工具曝光)。本台账不据此修改正式Finding、风险policy或外部Issue。

最新里程碑交接：本地commit已由用户允许且要求及时执行，Git index/commit由主线程唯一协调，本线不自行stage/commit。当前26项、Plan14原文/有效要求两项原失败及长reflect默认栈失败继续保留；P1/P2/N1/N2绿只结算具名范围。ubuntu@111.229.245.175的真实成功不替代Goal中的echo@111.229.245.175支持义务，两者分别验收。本次没有更新外部Issue状态或重新GET，既有API切点保持；文档精确路径/SHA与本轮验证结果交主线程，不扩张为整体工程已审通过。

## GET回执与复核边界

以下保存本轮每个响应的结束时刻、HTTP/进程退出、GitHub request ID与完整响应文本SHA-256。完整原始HTTP响应未另存，本文保存的是本轮实际返回的精选字段与响应指纹，**不声称新增了可独立重放的raw归档包**。重新GET可能得到后继响应或不同header，不能以SHA不同自动判数据损坏；身份/状态以本表时间和同一对象ID为准。若以后端点失败，应新增HTTP/错误缺证记录，不以本表旧成功填“最新”。

| 对象 | HTTP/exit | finishedAt（UTC） | GitHub request ID | response SHA-256 |
| --- | --- | --- | --- | --- |
| echo-agent#32(issue) | 200/0 | 2026-10-26T07:36:54.929207+00:00 | `2526:3FD491:141A3E0:1EB3D4A:6ADF0316` | `78d9717f33f28961bdfe9114164ac20c54fede378b5d064eb6b3f3e9bbfe01dd` |
| echo-agent#33(issue) | 200/0 | 2026-10-26T07:36:54.920971+00:00 | `2529:15A356:13CDF3B:1E5F1FE:6ADF0316` | `fec5069a51b2bba19e0d6ab42b62b02a5b487284d8e74374529e241729781b24` |
| echo-agent#34(issue) | 200/0 | 2026-10-26T07:36:54.907606+00:00 | `2527:345442:146FD6F:1F09B26:6ADF0316` | `12d00a1b497531caf3ef7cb0fc57874d4c9c6b0570e95a661d7f8ece53b15ceb` |
| echo-agent#35(issue) | 200/0 | 2026-10-26T07:36:54.915503+00:00 | `2528:1ECE85:14DE77E:1F79410:6ADF0316` | `a8bdd8c6903d3664811bf4c345d7678d7df89118fd24c5cda9da82b249682165` |
| echo-agent#188(issue) | 200/0 | 2026-10-26T07:36:55.773240+00:00 | `252A:1042F5:143984E:1ED2445:6ADF0317` | `fb415487c943c533503037cf8bbb2f72f9938f301f318af7d9ed2490bd6e37b5` |
| echo-agent#189(issue) | 200/0 | 2026-10-26T07:36:55.751352+00:00 | `252B:2E446:146FB0C:1F080DB:6ADF0317` | `41dfdfc271b666bc554c18b2b76992a017566bef6a5e154537780e4524fc47c6` |
| echo-agent#178(pr) | 200/0 | 2026-10-26T07:36:55.922403+00:00 | `252D:BED79:143C6FC:1ED6AD0:6ADF0317` | `b1c73266149b9299052e10acffce83639096faec81b1fb27cdda4005d79a4b4b` |
| echo-agent#179(pr) | 200/0 | 2026-10-26T07:36:56.027303+00:00 | `252C:30EF9D:14A8354:1F441C0:6ADF0317` | `dbe1e55854e77e7ad4e39ba2df657b109b365102ead3fe75b90832574ff77e53` |
| echo-agent#199(pr) | 200/0 | 2026-10-26T07:36:56.727950+00:00 | `252E:30AC84:1463E3C:1EFC767:6ADF0318` | `a6e384ada286b97d7f721bf509e2fa7d50feeb7ab7da1fad52b513ccd98e343e` |
| echo-agent-cli#10(pr) | 200/0 | 2026-10-26T07:36:56.674838+00:00 | `252F:3C5237:140DD64:1EA5B37:6ADF0318` | `5ab9a9b84e0685b5496e362272e68211f5cdfff7c36bedb78acc3c22169d05ae` |
| echo-agent-cli#14(pr) | 200/0 | 2026-10-26T07:36:57.253543+00:00 | `2530:2B1A8:13DF73A:1E7448D:6ADF0318` | `a9b3338a8900fa0809bdad027bf5dd1cfe2740dcae315d2832f70294547bc598` |
| echo-agent-sdk#10(pr) | 200/0 | 2026-10-26T07:36:57.031538+00:00 | `2531:2F070C:1420713:1EBA210:6ADF0318` | `79fece84cfab80325876d7df92e456011cddd75205410676b5d130bdc34cec7d` |
| echo-agent-sdk#11(pr) | 200/0 | 2026-10-26T07:36:57.569505+00:00 | `2532:399471:1440CA0:1EDA2CD:6ADF0319` | `13e571a28459e66ef5c5507dbecb179e4db14e40d004e4596cedb11b8c12d3ef` |
| echo-agent-sdk#12(pr) | 200/0 | 2026-10-26T07:36:57.606467+00:00 | `2533:414D7:145BDF8:1EF68C3:6ADF0319` | `1521535f42bbefac32551df1f5cfd31488df6b4495c9abc6325b5d567a509e65` |
| echo-agent-sdk#13(pr) | 200/0 | 2026-10-26T07:36:57.945699+00:00 | `2534:2E93B0:1484B16:1F20ECF:6ADF0319` | `5d41298a1786fac4d4f2e9712fa7863b2ec68054b51a631f040f9dd89934f0fc` |
| echo-website#4(pr) | 200/0 | 2026-10-26T07:36:58.240444+00:00 | `2535:2ECB6E:13E903B:1E7E993:6ADF0319` | `6770465fbaadb27484931bc59794514055fdfe7fda020d7fba3690d1dc7b4d66` |
| echo-website#8(pr) | 200/0 | 2026-10-26T07:36:58.393277+00:00 | `2536:348740:147CD66:1F157E6:6ADF0319` | `5a58189227f1d0022d85911b76fb1e315f46b3492b76cf8d91016c4b735177e8` |
| eko-workspace#18(pr) | 200/0 | 2026-10-26T07:36:58.510304+00:00 | `2537:13C31A:148997E:1F24A46:6ADF0319` | `a78d0f55759bc56df1c31d8a19d4aa0789bc9866a1b9165fdf7bec311cb6f55b` |
| echo-agent#190(pr) | 200/0 | 2026-10-26T07:36:58.806239+00:00 | `2538:1A446E:1425E66:1EBC9B2:6ADF031A` | `2872786a60c4fd8a0b0b61ba59c650ad10c52d311f02d970e7a4bbc60e8ae1ae` |
| echo-agent#200(pr) | 200/0 | 2026-10-26T07:36:59.040116+00:00 | `2539:389656:141BBEB:1EB5E26:6ADF031A` | `28d4a258a9fd6506b1db33f104855e9384ade57ae90926e27e4e46d5402db562` |
| echo-agent-cli#13(pr) | 200/0 | 2026-10-26T07:36:59.365978+00:00 | `253A:1A2ADA:149BDED:1F38FD6:6ADF031A` | `f159ec2d9984c2cd643ea8071168f4cdd8c0e6ca8beb23aa493ff4643f6df6b1` |
| eko-workspace@main | 200/0 | 2026-10-26T07:36:59.198745+00:00 | `253B:19A25E:142B1CA:1EC3DBE:6ADF031A` | `8afc8e397ba53514366570c23aa6a2a6eee4944c5057409416b277b91923d5eb` |
| echo-agent@main | 200/0 | 2026-10-26T07:36:59.503189+00:00 | `253C:3803B6:145F776:1EFA3A8:6ADF031B` | `fb5df1777a4894f077e6f1fb6a953970a090898eb7cf2b08d76555e478b211ee` |
| echo-agent-cli@main | 200/0 | 2026-10-26T07:36:59.755399+00:00 | `253D:7F4CB:14563FC:1EF3454:6ADF031B` | `8bcc88e3df9071d09885f20fb255c3898a73563355aa6969d40a3c241fcbb811` |
| echo-agent-sdk@main | 200/0 | 2026-10-26T07:36:59.966554+00:00 | `253E:2FD396:143BAC5:1ED5E19:6ADF031B` | `0cf237f131364b58cc82ea722ebaa7b6d80276fd8e52198246a94de4900b4aa8` |
| echo-website@main | 200/0 | 2026-10-26T07:37:00.037624+00:00 | `253F:3D243F:1435442:1ED0EFA:6ADF031B` | `920e66fec4f7dbae9fdc235d386d25703b9eb98a2df52fdf5522ffb3c2027f0a` |
| echo-agent#55(issue) | 200/0 | 2026-10-26T07:38:04.381874+00:00 | `2542:36829F:1487889:1F278B6:6ADF035B` | `ffd970be3937d05492b1da02b122dfec0406a82fa604d997ebb6e80b47b0995e` |
| echo-agent-sdk#7(pr) | 200/0 | 2026-10-26T07:38:05.401254+00:00 | `2543:25C04F:140C891:1EAD655:6ADF035C` | `0889a3696eca67646c115a16a175b03ec2c77434416af373373deee2c2c0fe30` |

本轮28GET没有HTTP错误、超时或重试；这不抹去旧台账的API超时原件。PR/Issue外部状态只证明其本次可读状态，不证明运行因果、完整测试或剩余产品验收。

本文及next-actions导航是本轮唯一写入。未改正式Finding/Evidence/Audit、runtime、AGENTS、SDK pin、风险policy、其它Plan/lifecycle或旧reconciliation；未修改Git index/HEAD/ref，无Cargo产物，cargo clean不适用。正式产品文档/examples/官网无本次行为变化，实际实现同步由各Outcome owner承担。

文档校验：28条GET回执与7 Issue/16 PR/5 main分母相符；8个具名Finding状态及三链已读；16个PR的merge/main祖先检查均0。两份本轮文档的17条本地文件链接可解析，空白和git diff --check通过，旧reconciliation SHA保持不变。这些是台账一致性检查，不是新增工程、CI、native或运行行为验收。
