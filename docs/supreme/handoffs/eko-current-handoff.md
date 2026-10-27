# EKO／Eclip 当前协调权交接

> **唯一当前入口；新协调者已完成只读核验并接管。** 本文件是跨仓协调与阶段状态，正式产品／API 文档仍在所属子仓。旧的 `2026-10-27-current-handoff.md`、MASTER、next-actions 仅作具名历史来源，不是与本文件竞争的另一份最新状态。
>
> 旧主会话：`01a162f1-952b-7773-8b8f-e4a3e4ca8e7a`（lp-agent迭代总调度）。新主会话：`01a17c1f-939d-7311-bba9-41e529912138`。
>
> **旧协调者已停止新增派发与业务源码写入。** 用户转交请求到达后立即逐个 interrupt 原子任务；停止切点最迟于 **2026-10-27 06:57:44 UTC／14:57:44 Asia/Shanghai** 现场时钟确认。之后只做了既有子任务的只读交回、现有日志／状态／Git 快照保全、已终结测试二进制保全和本文件写入，没有新编译、重启、清理、源码修复、stage、commit 或外部消息。
>
> **新协调者于 2026-10-27T15:12:28+08:00（Asia/Shanghai）完成只读核验并登记接管。** 首批写权和资源责任见第11节。旧Goal、SID、审批、进程句柄和验证资格不自动迁移；旧会话只保留历史来源。

## 1. 协调权、授权和冻结状态

- 原 Goal 保持 active、完整目标未缩减、未声明完成或暂停。新会话应核对并显式建立自己可执行的 Goal／追踪关系；旧会话的服务端 Goal 不会因本文件自动迁移。
- 所有已确认实现、局部调试、独立审查、本地里程碑 commit、维护与可恢复清理授权继续保留；仅当前交接阶段停止执行，不能把交接当产品工作全部完成。
- 唯一开发根 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`；四子仓独立 Git，不是 monorepo。不新建外部 worktree，已有外部宿主管理目录保留，禁止未经核对手动删除。
- 新会话接管前只读；接管后按文件单一写入、冻结验证输入，并行推进真正独立的任务。只串行冲突写入、重型编译和正式性能测量。
- Root 统一 stage/commit，全部 commit 显式 `-c commit.gpgsign=false`。当前不 push、merge、关闭 Issue、发布包或部署官网，不晋升指向未推送子提交的顶层 gitlink。
- Exa `feature/Echoyue/exa-web-search` 的 `4343f0b736beb5c8606e31146528a2fe8be80510` 本机保护，暂不 push、不合 main。固定分支及 `pre-retirement-2026-10-24` tag 保护。
- **本交接未清 target。** 既有 F 批次已终结并保全测试二进制，但 target 控制责任明确交回新协调者；设置预览仍活，控制责任也交回，禁止把交接当作可随意终止的依据。
- 用户完整新交接／继续要求原文保存在附录 B，原 Goal 原文保存在附录 A；后继明确裁决见第 2 节。原说明中的 EKO 为历史产品名，当前用户已采用 Eclip 品牌，技术标识不随之改名。

## 2. 后继用户裁决与替换关系

| 主题 | 最新有效裁决 | 来源及历史替换关系 |
| --- | --- | --- |
| 品牌 | Eclip `/ˈɛk.lɪp/`，猫头图标用于品牌位置；技术仓库名、`com.eko.desktop`、二进制名、协议和 `.eko` 数据位置保留 | 用户品牌会话已实现后交主流程采用，CLI／官网已本地提交。不能把源码提交当旧窗口已更新。 |
| 首页 | 主标题**把想法变成成果**；删除“继续上次对话”卡，从明确最近任务／工作空间继续 | 用户在本会话候选问答中明确选择该句，覆盖旧“EKO，和你一起把事做好”和品牌-only标题方案。 |
| 顶部分叉 | **删除顶部“新建支线对话”入口** | 用户明确要求，覆盖旧的保留并澄清顶部入口建议；不误删已有 Side Conversation 历史、恢复或底层 API。当前六文件 WIP 已实现。 |
| 回复处分叉 | 每轮**完整回复末尾**一个入口，评估并落实当前工作空间／新 Worktree 两形式 | 用户截图是方向与需求，现有源码只完成普通历史 fork；dirty 继承、Goal／模型继承及非 Git／SSH 边界仍未裁决，不可静默补入。 |
| 聊天呈现 | 去 assistant 左侧小机器人，修空消息外壳留白，正文对齐；工具／执行紧凑、详情可展开，不能吞真实状态 | 用户 Codex 参照截图；52px 空壳根因已真实浏览器复现。正式 MessageBubble 还未修。 |
| 设置 | 具体表单统一排版、间距、控件、分组和保存反馈；基础配置常显；IM 独立 | 基础／IM 已提交，设置美化只有候选，没有成功视觉验证。模型仍默认入口，不恢复总览。 |
| SSH | 原目标 `echo@111.229.245.175` 支持义务保留；后继用户使用 `ubuntu`，密码仅 GUI 输入 | 用户确认 ubuntu 测试连接、`/home/ubuntu/ssh-test` 创建／进入及刷新后侧栏显示成功；不替代 echo 账号或完整终端／远程操作验收。 |
| IM | 完善 QQ／飞书，新增**个人微信扫码本人**；其它平台按需 | 用户明确个人微信；不把企业微信或其它平台代入。真实手机验收尚未取得。 |
| 层数 | 主 0，默认只首层子 1，额外嵌套默认 0；GUI 最大总层数 3 | 覆盖旧 ADR 单层冲突；Framework 泛用合理上限与产品默认分开。配置变化只影响新 Run。 |
| 输入／Goal | 普通Steer由主模型结合历史理解并在执行中调整计划；新输入不自动改Goal；无Goal时智能体可设置初始Goal，已有Goal只能用户更新/删除 | 2026-10-27新会话用户明确替代旧A/B及统一强制“有效要求→计划→验收”流水线，原文/耐久接纳/幂等/恢复保护继续。 |
| zstd | 已采用 Framework 唯一两 Polars vendor／五来源方案，不增 ignore／接受风险，不复制 SDK vendor | 用户明确 A 采用。工程验证通过；外部 sourceTask1/402 仍阻塞正式来源。 |
| 外部工具 | echo-semantic 合同修复继续排除，原阻塞保留 | 用户曾明确 B 不纳；不能造 400 条形式 Evidence、扩大 exclusion 或改工具绕过。 |
| 发布／部署 | 主流程纳品牌进后继构建；**没有**授权执行包发布／官网部署 | 用户交接的“下一次构建发布”按既有明示边界做制品准备，不作为外部写入授权。 |
| 清理／资源 | 最多两个 focused Cargo，jobs≤2、debug=0、自有准确 target；每批先保原件／制品，确认无 reader 后及时 cargo clean | 交接本身用户明确“勿新增编译／重启／清理”，因此此刻只保全而未 clean；新协调者接管后续办。 |

**接管后替换关系：** 原有效要求A/B选择已由最新用户裁决撤回，不再待选，也不将其叫作旧B采用。Fork两形式的文件改动继承仍待裁决，其余优先保留现普通fork语义；依赖风险与实际精确合同冲突（第8节）仍待具体复评。这些不阻塞独立UI实现。

## 3. 五仓现场与每一项未提交内容

下表为 **07:00 UTC 左右的冻结观察**，由旧会话原 Git 命令重新采集；新会话另已独立确认同组 HEAD、初始 index 空和 refs 数。创建本文件后 Root 会新增这一份未跟踪文件，此为预期增量；表内统计是写本文件之前的冻结输入，不能把本文件计作未知旧源码。

| 仓库／实际根 | 分支 | HEAD | 暂存路径 | 未暂存 tracked | 未跟踪文件 |
| --- | --- | --- | ---: | ---: | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent` | `chore/Echoyue/workspace-maintenance` | `309cf0b42710dd5c74ac535d52e736de854dbe70` | 0 | 5 | 114 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent` | `chore/Echoyue/root-workspace-convergence` | `f3c5af6443fff36dd7088dcb32a665b5fc173489` | 0 | 13 | 447 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli` | `chore/Echoyue/root-workspace-convergence` | `1a3551a0dac37463b8dd2848fb73b82d4d9f3c07` | 0 | 5 | 2 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-sdk` | `chore/Echoyue/root-workspace-convergence` | `525453fc9b9448358cc8a2f46bbcf401b10943ec` | 0 | 85 | 9 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-website` | `fix/Echoyue/website-source-sync` | `2d35de85c077f25576eea0cc588feb6099f7b274` | 0 | 0 | 0 |

完整逐路径的 status、SHA、归属、保护方式、全部 branch／stash／tag 精确对象及 worktree 注册表：[五仓冻结清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/coordinator-handoff-20261027/five-repo-inventory.json)，SHA-256 `bdea771bd8af2a653835c944fbc145641a3e80f2176a91496322b4fe1839ef37`。没有仅按目录计数隐去 vendor 成员或 SDK 删除项：该清单对每个 Git status 路径逐项登记，`??` 使用 `--untracked-files=all` 展开。

归属与处置规则：

- Root：README 是用户品牌会话交接改动；四 gitlink 仅当前子仓位置与顶层固定指针不同，不得晋升未 push revision。`outputs/branding/` 的 114 源／脚本／派生图均保留，不是待删垃圾。
- Framework：**I1 四路径**单独写权见第 5 节；**zstd 456 路径**是另一个未通过 source 合同的冻结候选，445 是两个官方 vendor 成员，两个未跟踪第一方文件是 ADR0100／`zstd_data_compat.rs`。不能把 I1 和 zstd 混成一个提交或清理。
- CLI：**首页六路径**已实现未提交；另一个 `scripts/__pycache__/check_cli_dependency_risks.cpython-312.pyc` 来源未充分确认，只保护，不采用、删除或借其降低范围检查。
- SDK：原 94 路径（85 tracked、9 untracked）包括具名 A2A/LSP 接受合同／Host／三语言删除与消费者适配；保留原 task，不允许由 UI 或 I1 顺带扩大 accepted／升级 pin。
- 官网：当前 `2d35de85…` 干净，品牌与独立 IM 文案／已审应用来源均已本地提交；后继首页文案变化尚未同步，不自动当当前产品页面全对齐。

### 当前 CLI 六路径（全部冻结、暂存为空）
- `echo-agent-cli/.echo-semantic/assets/asset.eko-workspace-task-navigation.md` — SHA-256 `3006e49695b3f9515251952b83c42460785d7fb50adb1857c30f164878e03bd8`。
- `echo-agent-cli/docs/zh/getting-started.md` — SHA-256 `3c5f5609eafeb1ac2b591cc7370c2d1ec5bc5c0f2d5d6d3b4c18b63b15e0dc71`。
- `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.controls.test.tsx` — SHA-256 `22d44055f7d48b1bf12c641a82f934ecaf6fea383f7b9fd50b8781f0827c6570`。
- `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx` — SHA-256 `a88689ab76aa124c17a0332d11dd96f97335d30f602ea8e5c8605d6ed642ff39`。
- `echo-agent-cli/web-frontend/src/components/chat/WelcomeScreen.test.tsx` — SHA-256 `0f734ba55185f231bcaad80e7f7fade2ea4b09d9f3c9f266fd03e0ac2291a09f`。
- `echo-agent-cli/web-frontend/src/components/chat/WelcomeScreen.tsx` — SHA-256 `17d721025b1819343e8a0c942b3efeb61a21c7848b16c256014eadcb02a62d60`。

该六源唯一原作者 `gui_delivery`，真实 SID `01a17c1a-f77a-7e93-9db0-588f666f5adc`，task `gui-home-entry-resume-20261027`。原 preflight 仍存在、未消费，高风险／publicAPI；允许恰这六路径。旧任务 `gui-home-explicit-task-entry-20261027` 的 source1/3 及 orphan scope 已保全后由 Root 正常释放，不是成功收据。新 task 已正常补真实导航 Asset 说明、保护未知 pycache，source0；staged／final 未执行。

### 当前 Framework I1 四路径（全部冻结、暂存为空）
- `echo-agent/src/agent/subagent/control.rs` — SHA-256 `402e5619b17b31fcc3230c09164c4ebcaee99beaa1ac898a030b65c137f6ab68`。
- `echo-agent/src/agent/subagent/executor.rs` — SHA-256 `12453b7c0a8b4b588ce46cf8bb3b2ce44843b4e59d8a438b8c5329a6a2457c1d`。
- `echo-agent/docs/zh/06-subagent.md` — SHA-256 `64a2ea4017ddec52d5503b6116418f7de9a54536f7f8ac773eca153d636bbee2`。
- `echo-agent/docs/adr/0058-task-claim-subagent-attempt-control.md` — SHA-256 `4c8e7ef5774cf707d16806876dc1c11ac4a06ebcb8a8789d3cdd335ad535aa3e`。

原作者 `run_control_i1`，真实 SID `01a17c1c-7d9b-7013-ae18-75ca48deca73`，task `native-run-control-i1-resumed-20261027`。当前 preflight 仍存在、未 final；只准四路径，456 zstd 路径全部 preserve。SDK task 的旧 SID 仍为 `01a16da7-416a-77d0-abdb-ce1472d975fa`，task `sdk-consumer-root-integration-20261027`，原合同未消费。新协调者不能复制旧 SID 伪装原生 writer 或验证；应先保全原件，再按真实当前 owner 与工具正常释放／登记新任务流程处理。旧 Root 此刻不会替新会话释放这些新 scope。

### refs、worktree 和恢复材料

五仓合计 **66 本地分支、18 stash、123 tag**，本轮未新增删除。9 个注册路径 = 5 主检出＋4 host 管理路径；多仓对同一 host 路径的注册不得当额外独立目录重复删除。51 个历史非 host 工作树已经可恢复回收（41 外部＋10 内部），原路径合计22.24GiB是历史逻辑尺寸，不是本轮净磁盘收益。host queued 归档不算已完成。必要 refs、47 恢复归档及其原索引、Exa／pre-retirement 全部保护，不因本交接重压缩或重清理。

## 4. 运行现场、制品与控制责任

**当前进程原件：** `.echo-semantic/local/coordinator-handoff-20261027/process-scene.json`，含 ps 原 PID／PPID／启动时间／完整命令、lsof cwd 和各端口原出口。旧、新两会话独立核验一致：

| 资源 | 现场事实 | 所属与移交控制 |
| --- | --- | --- |
| 原生 `56017`／模型夹具 `55840` | 均不存在；ps 无进程，1420／1451 无监听 | 历史 epoch16 的成功及失败仍保，但**当前无可核活原生窗口**。不能继续沿用此前“已置前”或旧授权自动重启。 |
| 设置预览 supervisor `72559` | 2026-10-27 14:50:32 本地启动；`run-command.mjs … --label settings-preview-native-loader …`；实际 cwd CLI | 原 `settings_candidate`，已冻结，控制责任交回新主会话；不在交接中终止。 |
| 设置 Vite `72576`／esbuild `72861` | `127.0.0.1:1438`，14:50:32／14:50:59 启动；cwd `echo-agent-cli/web-frontend`；Vite `--config ../.echo-semantic/local/settings-visual-refresh-20261027/vite.preview.config.mjs --configLoader native` | 只展示 ignored 候选，未通过视觉验收；旧 exec 55701 不假定新会话可等待，优先用 PID／birth／端口／日志监督。 |
| 聊天预览1423 | 无监听；旧 before 证据保留 | 无当前服务控制义务，不要仅凭 serve 脚本重启。 |
| F I1 Cargo 批次 | 原 exec44714，06:53:49开始，**06:55:27自然结束**；ps无 Cargo／rustc／runner | 两个原命令0，19测试；target未clean，新协调者接管保全后精准清理。 |
| GUI source 校验／测试 | 原句柄70332／38020已经有终结0回执；无活动测试进程 | 不重复发起同源已过命令；仍缺审查／文档parity／最终门禁。 |
| 其它项目服务／moss端口 | 不属本项目 | 不停止、不清理，旧会话未把全机 Node 进程当 EKO。 |

设置预览准确日志为 CLI `.echo-semantic/local/settings-visual-refresh-20261027/preview-server-native-loader.log` 与 `resume-run.json`。最后浏览器句柄79867自然退出1，并执行 browser.close；没有“after截图成功”资格。

### 历史原生版本与不能假设的连接状态

最后实际被用户验收的 epoch16：binary SHA `f3fedd879bc361da5f6068d7005ea84e014448a8072f218031723e4a0f5b9f12`，保存在 CLI `.echo-semantic/local/tool-search-request-consumer-20261027/actual-implementation/artifacts/echo-agent-tauri`；148 前端资产清单 SHA `70c64653bc468e01791ebbff4b12e6cedcbd104ea42c6de70731666554e5617b`，目录 `.echo-semantic/local/gui-native-epoch16-preparation-20261027/frontend-dist`。这对制品不含 `7045853` 基础／IM、`1a3551a` 品牌和后继首页／聊天／设置／I1。不能把单纯 reload 旧148资产说成新界面。

当前无原生 PID，不声明仍有 SSH 活连接，也不声明草稿／未答 HITL 已安全保存或消失。交接没有读取用户密码、私钥、微信 token、草稿或原始 prompt；新协调者后续启动／切换具名新版前，应核当前真实用户状态。历史维护许可已被对应 epoch 消费，不能套用到下一次新构建切换。

### F 已终结批次的 target 与制品

- manifest：`echo-agent/Cargo.toml`。
- 准确 `CARGO_TARGET_DIR`：`echo-agent/.echo-semantic/local/native-run-control-i1-20261027/cargo-target`，回读尺寸约917640KiB；jobs2、dev/test debug0、incremental0。
- 输入：`echo-agent/.echo-semantic/local/native-run-control-i1-resumed-20261027/source-freeze-before-batch.json`，SHA `c5f72457bc39b8a3c102e957e7b390f5c2fa62a59cdb0f6d5fdd18a04526f0e2`，1460构建输入；实际 WIP 锁 c48，与 I1 父提交旧锁不同，不证明精确提交或 all-features 资格。
- 原 test binary：`debug/deps/echo_agent-228d70b730583953`，130710992B，SHA `af10225356d9826ed5d930b38adef3228097019c946a2dfd7ff531d45168722b`。
- **旧 Root 已作必要批次保全**：同SHA字节已用APFS `cp -c`复制到主根 `.echo-semantic/local/coordinator-handoff-20261027/retained-i1-test-binary/echo_agent-228d70b730583953`；复制前原二进制 `lsof`自然1且stdout/stderr空，复制后hash一致；回执 `i1-binary-preservation.json`。没有 clean／删除，准确 target 交新主会话。应在它重新确认无 reader 后，按既有批次规则及时 `cargo clean --manifest-path <准确manifest> --target-dir <准确target>`，不要清其它任务缓存。

## 5. 每条子任务的安全切点与下一文件责任

全部原子任务已被 interrupt，之后仅以只读交接结束；**没有任何旧子任务获得继续业务写入的许可**。下面的 owner 是来源／交回责任，不是新会话已经授予的租约。新主会话核验后应明确重派，不能让两个 writer 同时恢复。

| 子任务／来源 owner | 当前处置 | 精确成果／下一责任 | 还缺什么 |
| --- | --- | --- | --- |
| gui_delivery／SID01a17c1a… | 交回，六源冻结 | 当前首页标题、卡片与topside删除已实现；责任六路径见第3节 | 最新独审、文档parity及必要组合构建、staged／final／Root commit。旧四源审查不自动扩大到新增topside／Asset。 |
| chat_render_candidate | 交回，仅调查／未实现修复 | 后继候选 `MessageBubble.tsx`、新 `MessageBubble.rendering.test.tsx`，建议独立 `docs/zh/features.md`短段（尚未授权正式采用） | 当前 patched MessageBubble仍与生产同bf33…，无修复diff／after GREEN；完成null空壳与avatar删除、保状态后真实浏览器验证。 |
| settings_candidate | 交回，仅ignored候选 | patched `SettingsDialog.tsx`、新增局部CSS、`ConfigPanel.tsx`、`providers/ProviderPanel.tsx` | 22candidate单测保原切点；浏览器三次失败，需修预览模块加载，再浅深／窄窗／portal／焦点／保存草稿与正式采用。旧diff文件已过期。预览1438控制一并交回。 |
| run_control_i1／SID01a17c1c… | 交回，F4源冻结／19项实际测试通过 | F4见第3节；CLI后继三路径 `tasks/task_runtime/subagent_control.rs`、`executor/tests.rs`、`docs/zh/adr/0013-subagent-control-receipt-projection.md`目前**未改** | 未取得旧EKO helper真实registry RED；未切CLI `.attempt_identity()`／scoped guidance；三真实readonly/writer分支及相关例子仍待；I1最终review／语义／commit待。 |
| I2 readonly-depth-enforcement | 仅已审Design／Map，尚无Plan／实现 | 原ToolAccess／dispatcher／snapshot／三carrier与真实工厂接缝，框架通用机制、应用产品适配分开 | I1隔离交付后再建唯一Plan；保同registration／只读∩W／每spawn+1／默认Agent新ReadOnly fallback拒绝，不全局把agent_tool当只读。 |
| fork_boundary_explorer | 已结束，只读memo | `.echo-semantic/local/fork-two-forms-20261027/源码事实与边界.md`，SHA `1740b2c4d8f47f9078a141ba58a738721b78adce5f18a5637df0c3a0eafb5121` | 当前UI未保证每Turn唯一；Worktree只建Git目录，缺workspace／conversation／cwd组合；dirty／非Git／SSH／Goal模型继承需真实方案裁决。 |
| risk_boundary_recovery | 已结束，只读／候选未采用 | `.echo-semantic/local/next-native-consumer-reassessment-20261027/` current readback／近25commits／review-onlypolicy | b0真实批准有但无可复原全集；最新用户指令精确旧锁与当前合同冲突；不得只换hash自动接受。 |
| zstd_formal_design及cleanup_review | 已结束；F456冻结保全 | 已实现／工程审查过，source1/402；外部工具排除。旧编辑scope已合法clear并备份，不验证完成 | 不能stage/commit冒source0；正式可恢复来源与SDK后继因此未形成。 |
| SDK原channel_control合同 | 冻结保护，未结算 | HEAD525453fc＋94path，pin f7bac62，5352保留／431routes／39退役operations | 原4vuln／10unsound、安全源／实际HostParquet／三语言／完整门禁仍待，UI不消费该scope。 |
| brand／官网 | 已结束，分别已提交 | CLI1a3551a、官网83ff57f＋2d35de8，最新已审来源CLI1a355 | 后续首页改动需所属docs／网站来源再独立对账；未打新桌面包、未部署。 |
| delivery_status_refresh／nativeDesign | 已结束，文档已提交 | Root d94796f＋309cf0b；正式I1 Plan ready | 文档通过不等实现完成，不能自动改Plan completed。 |

### 设置候选准确身份与失败限制

候选根 CLI `.echo-semantic/local/settings-visual-refresh-20261027/patched/`：

| 相对候选根路径 | SHA-256 |
| --- | --- |
| web-frontend/src/components/layout/SettingsDialog.tsx | `13060f72f20865fb66ec6cf9674b364e9ac598cd3c8122b1cb09a220be0f1f1b` |
| web-frontend/src/components/layout/SettingsDialog.css | `c410755da133b39c2f2f7fbb1939a67f9d9db85f11007772aad7bacc6a1409e4` |
| web-frontend/src/components/config/ConfigPanel.tsx | `736b695394aee22c1b24bff7eae4b5093931c72e794495bdffecc09ece982f5e` |
| web-frontend/src/components/providers/ProviderPanel.tsx | `3bdff282d9d7d3c91bd0bdaeb54679b81976729b021b9cb46cea02c2f9d4b29d` |

原三个正式基底仍一致，CSS尚未正式新增。`base-manifest.json` SHA `1d01f238a36d1f6192b80d0b5e76592079446f8cd03c4de311df973e099da5fe`。`settings-visual-refresh.diff`已过期，不能直接apply；最后只读重算当前候选diff为 `9ad5aff4e9e31f3e352cddc974e1b442d19fb417eba4515bcf0cc14fab401698`，采用前按真实patched字节重新生成。最后浏览器失败是绝对导入 `react-dom/client.js`没有 `createRoot` named export，非视觉成功；错误截图在 `browser-resume/`、`browser-epoch03/`。不能用candidate22 tests代替生产、浅深主题或截图通过。

### 聊天留白因果边界

生产 `MessageBubble.tsx` SHA `bf33e26a014756b5b62b8d40b5dd488b4bfe69e72506d0f200c6e326ec9e8c1c` 未改。真实日志的空assistant/tool_calls行没有UI正文、步骤、附件，原MessageBubble仍无条件py外壳与Bot；真实React/CSS四条各52px共208px、5Bot、pageerror0。旧测试8项为5fail/3pass，其中2fail是工具suffix／Subagent可及名夹具错误，不计生产红。没有修复GREEN／after图，候选同原源码。只隐藏真正已终结且没有可见内容的壳；流式／active run需要单个紧凑状态，工具错误／HITL／附件／思考／Subagent不能吞。历史raw tool_calls未hydration成steps是另一个已知缺口，本修复不假装已完成其恢复投影。

## 6. 已完成提交、验证和不继承资格

| 当前成果 | 已落地提交 | 实际证据与边界 |
| --- | --- | --- |
| 基础配置常显、QQ／飞书独立IM | CLI `7045853c0442f2a556f268bdafa8a4081b00d5f1` | 六路径独审、source/staged/final0；当时39focused及组合608前端0。尚非新原生通过。 |
| Eclip名称／品牌图标与两条诊断路径 | CLI `1a3551a0dac37463b8dd2848fb73b82d4d9f3c07` | 32路径独审和原门禁0；当时465前端输入、75files608tests，lint／format／TS／Vite0。不覆盖后继六源WIP。源码Git archive1066blob回读通过。 |
| 官网品牌 | `83ff57fd24c7beaebd82939138f756edf5d5e32b` | 原49tests、browser15pass／1平台skip与新4share/discovery文件后继light／326routesbuild0，原30source/final0；不部署。 |
| 官网IM说明＋应用源绑定 | `2d35de85c077f25576eea0cc588feb6099f7b274` | 四path原app-only source／format／discovery／docs／326build及final0。绑定CLI1a355的138blob，仅configuration／zhREADME两项源SHA变；Framework310维持f3c5，不是F dirty源检查通过。 |
| 全部26项阶段对账 | Root `d94796f5611a4b4b3c9609b795162725a027802c` | 3中文文档，原历史tail保、source/staged/final0、globalnotchecked。当前本文件覆盖其时效，不重写旧记录。 |
| I1／I2正式设计与I1Plan | Root `309cf0b42710dd5c74ac535d52e736de854dbe70` | Design388498…／Map6b1c5d…／Plan c1dea9…；独审及artifact/source/final0。Plan `plan:42888472-07db-434b-bcd0-2b2167a55882`／ready，精确执行路径 `docs/supreme/specs/native-subagent-depth-repair/plans/plan_01_run隔离与指导.md`。 |

### 最新六源GUI验证（当前冻结）

目录 CLI `.echo-semantic/local/gui-home-entry-resume-20261027/`：

- `run.json`记录原旧top入口测试自然1（1真实断言失败），随后同源相关 **6files／61tests自然0**，覆盖 WelcomeScreen、ChatPanel controls、LeftSidebar、App navigation、普通 MessageBubble fork、SideConversationDialog。
- 命令：`node node_modules/vitest/vitest.mjs run --config <本任务vitest.config.mjs> --configLoader runner --maxWorkers=1 --reporter=verbose <上述6testpaths>`；cwd为真实frontend。逐命令完整argv／bytes／signal／truncation由run.json维护，不凭摘要重建。
- focused lint、Prettier和真实app `tsc --project tsconfig.app.json --noEmit --incremental --tsBuildInfoFile <任务内独立path>`均0。source-task正常0／passed，原stderr空；**未staged/final／新独审／文档parity／新组合build或完整前端**。
- 旧home3业务RED＋1图标正控及55test是旧四源切点，不能代当前61测试或取代完整当前资格。

### 最新F I1验证（当前冻结）

目录 F `.echo-semantic/local/native-run-control-i1-resumed-20261027/`：

```sh
cargo test -p echo_agent --lib --no-default-features --features subagent,testing --locked agent::subagent::control::tests::
cargo test -p echo_agent --lib --no-default-features --features subagent,testing --locked controlled_dispatch_claims
```

实际17／2通过，0fail／0ignored／0compilerwarning，两命令自然0，无signal／无截断。原log SHA分别 `a122646cd61bede31a372ede76c7f5c6879c0329ce3dd860594795f80c32e2b0`、`56e408edfb29edcadd51bc22f2de9a9eb5ecc078375bfa04d3d56f7a825321ba`。1460输入与456保护无漂移；这仅Framework实际WIP锁的定向资格，**不证明CLI旧helper修复、I2深度、完整矩阵、main／CI或原生**。原fmtall/check0保在上个同四源epoch；最终source／staged／final未完成。

### zstd有限工程通过与正式阻塞

F `.echo-semantic/local/zstd-safe-chain-implementation-20261027/`：28实际Rust测试、16origin控制、14policy控制、demo46编译与fmt均0；actual／五来源视图完整原audit在DB `3279c99911354c400e1d41e47c7d60e39258b147`、1378条上均0、stderr0，无no-fetch/no-yanked；deny四类及正式原精确caller0，原49组／107身份未扩大。首fixture运行时失败、原HTTP失败、no-fetch五索引错误及更正均保原件，不当绿色资格。

但source-task实际 **1／402**（两个启发signal＋400新增vendor路径literal依据检查）。独立工具源审查确认task未纳baseline而把vendor误作in-scope，且不展开整包归档／目录provenance；没有现合同合法整包入口。外部修复排除，不改工具／排除／填形式Evidence，**未stage／commit、无可正式继承的新F revision**。最终freeze `dcb8c4b35b46f7426bacec73fe634481bcd7e63b017bd7f0b9e065ad063908b7`／diff `f6c0092753ae3e36395bf6c124acb9326c16c8240ac6f78e674170f1d8149605`。

## 7. 原生、IM、SSH、有效要求、退役和全Goal剩余范围

- HELLO及同对话HITL：用户真实确认、journal双核；只证明接纳／消费／原工具结果／settled，不等Applied、跨journal原子或全部crash recovery。
- depth1：Run `64916d5c…`，真实readonlyreviewer与lineage／下一层零请求；depth3 Run `9c54a00f…`保存3但搜不到agent_tool，`63da81cc…`因丢Run的attempt碰撞失败，用户未再被要求在旧binary反复重试。
- SSH：ubuntu登录测试／创建目录／侧栏刷新用户通过；中文、resize、全屏、Ctrl-C、exit、断线／关闭／多会话、远程文件／命令／Git／产物同目标完整矩阵未通过；echo原目标支持未被替代。
- 微信：自动化13项等各旧切点保留，但扫码／必要确认／取消／过期／重连、私聊／HITL及手机实际收件尚无完整真人资格。QQ／飞书与cron完整owner／恢复／通知也按原Plan继续。
- 有效要求：原完整输入／durable admission已有切片；scope／replacement／Applied／目标计划证据失效／独立要求覆盖全集未完成。A/B解释路线待裁决，不把“最近4条”或Plan派生分母当完整用户要求。
- 工具治理：已有模型实际tool exposure预算／tool_search激活后请求回归、结果预算等是具名局部证据；readonly真实工厂是新增确认缺口，完整消费者／插件／真实Host取消／错误组合仍待。
- A2A／LSP／TUI／CLI／REPL／JSONL退役：GUI／IM／cron／共享恢复和SDK消费者／保留契约main资格仍未闭合，不能物理删在用共享代码。固定历史保留；其它合理FrameworkAPI不因EKO未调用删除；内置终端／AgentShell／构建命令／SDKHoststdio保留；CLI不启SQLite。
- SDK：HEAD525453fc＋94path，独立Host pin `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`。accepted原5621→5352，routes454→431，269身份／23route／39operations退役；原safe scans4vuln／10unsound仍阻塞，F probe或候选不替代SDK实际pin／根patch／五源／真实HostParquet／三语言资格。
- 完整GUI／HITL／tasklist／SteerGoal／历史大数据／主题／窗口／恢复、完整workspacefeatureexamples／精确mainCI都未因局部结果通过。性能ready24／blocked8只是准备分类，正式基准静默窗口未完成；新增IM平台／发行目标／包发布／官网部署按需及授权，外部动作未执行。
- 26项Outcome的逐项未完成对账保存在 `docs/supreme/plans/2026-10-26-root-convergence/2026-10-27-current-handoff.md` 第“26项剩余结果对账”；该列表范围继续有效，但HEAD／当前进程／当前子任务状态以本文件重新观察为准。

## 8. 依赖风险与当前指令冲突（未裁决）

本轮最新用户AGENTS替换文字明确写 Framework只锁`7607c1458036adc04e64858215e21d45f7f8ae1ffa992ad82fba97e7a72aa528`、CLI锁`d2e7b7ebc035d35940f20fd9a95628a37ddf13bba2c1d5c828cdac142ae41680`。当前磁盘实际F锁为`c48e0a273f6ac000836d7781744dbba3270f8ff08b00c8814c6998ce6b7cadc6`，CLI锁`3fb8955b56e86eb4bef2454d3328b6ef8eb7f36d7663806387f3b990244e7ed6`（856包），最新用户要求又强调按真实当前精确合同。正式历史文档证明7607／d2e7是过去锁，但**不能因此静默重写最新用户的精确指令**。

CLI机器policy SHA `03cf671a288612d6d57284fa99505d296a8f31c81323371b40dce084a4b52e40`绑定3fb／consumer `b0b83f384877d9ab3b8bdaaa403cd0bd439e6bc9be04106209282ab8bd0fc93f`；用户真实epoch08批准b0的原件在CLI `.echo-semantic/local/root-consumer-reassessment/epoch-08/policy-binding.json`／`reassessment-candidate.json`。当前321输入consumer为`27d5871c5704c324bce7045bcc14310ff8b44d937d4361b8edc8ae3c40123f31`（1a355的desktop两字符串后），六项name/version/source/checksum/advisory/kind均相同，但**尚无可复原b0逐path全集**。9bcc把b0首次入Git却明确“最终冻结再复评”，不能伪称其树就是b0；近25提交也不匹配。

只读候选 `.echo-semantic/local/next-native-consumer-reassessment-20261027/current-policy-candidate-review-only.json`未采用；不能只因六身份相同或hash变化就自动续期。新协调者应在最终Rust输入冻结后，先厘清适用当前锁／基底未知的真实取舍，再完成一次具体风险复评；不在UI批次反复要求每个无关hash确认，也不绕原script65的human reassessment。本交接未修改AGENTS、任何policy／lock／ignore或官方scanner结果。

## 9. 恢复入口与不能删除的资源

| 材料 | 哈希／恢复与保护 |
| --- | --- |
| 本次五仓完整逐路径／refs清单 | `.echo-semantic/local/coordinator-handoff-20261027/five-repo-inventory.json`；第3节SHA。只读对照，不按清单批量删除。 |
| 本次状态原件 | `.echo-semantic/local/coordinator-handoff-20261027/state-originals/<repo>/`，当前preflight／continuation／verification原件；不能用旧verification覆盖新身份。 |
| 本次原Goal与附件 | 同目录`goal-state.json`／`original-goal.txt`／`new-coordinator-user-request.txt`，原文保留；附录A/B内嵌。 |
| F456候选恢复包 | `.echo-semantic/local/zstd-framework-blocked-preservation-20261027/framework-456-frozen-sources.tar.gz` SHA `b71fb65a079ce8704c54ef27a054463ecdc8eeca21762a2f41a0eae4e773b6e0`，逐456 bytes／mode／SHA回读。只能在对应base f3c5的独立受控恢复目标恢复，拒覆盖当前用户差异；不把tar当正式Gitrevision。 |
| CLI已提交GUI／品牌源码 | CLI `.echo-semantic/local/local-milestones-20261027/gui-brand-checkpoint/gui-settings-and-brand-source.tar.gz` SHA `44b9677d842020a8a2ff186430149273bba5084d3bd885f82c7f280ebf911271`；1066Gitblob回读，base1a355，不含ignored用户数据／凭据。 |
| F I1测试二进制／logs | 本次retained目录与`i1-binary-preservation.json`，第4节SHA；保全后target仍在，不得在新协调者核无reader前清。 |
| epoch16用户数据／旧制品 | CLI `.echo-semantic/local/gui-native-epoch16-preparation-20261027/maintenance-switch/`，59file五权威offlinebackup／scratchrestore，回执SHA `2212327f4dab03081ead1ede6704a770682c44f9995e6a0c00c880e7c5dbf739`；旧14与16binary／fixture／148assets保留，不把它们当缓存清。 |
| 原orphan释放／来源 | `.echo-semantic/local/interrupted-scope-recovery-20261027/receipt.json`备旧已消失actor的scope并正常clear，仅释放编辑，不标完成；新F／CLIscope后来正常重建且此刻仍在。 |
| 旧worktree恢复包与保护refs | 历史archive-inventory／internal-reclaim／ignored-evidence索引及已核47归档，Exa／pre-retirementtag／18stash／66branch／123tag；不因移交重清，具体对象见冻结inventory。 |
| 品牌素材outputs／设置／聊天候选 | 全部保留。设置服务还读其候选与Vite配置，不能提前删；聊天未完成候选不要误当修复通过；未知pycache保留归属待核。 |

恢复要先核源哈希、目标空闲与拒覆盖，使用原对应base；不能从旧绝对路径继续开发。任何“保全成功”只证明材料可读，不替代业务实现／native／安全或发布验收。

## 10. 新协调者接管后的下一批（不是旧会话的新派发）

| 优先／可并行 | 建议明确的唯一文件负责人及交付 | 依赖／验收／需要人工 |
| --- | --- | --- |
| 首先只读核验并登记 | 新主会话核对本文件、最新AGENTS、五仓状态／冻结／现场；登记真实接管时间与授权写权 | 不自动迁移SID／Goal／句柄；核到漂移先记录。无需重跑旧完整矩阵。 |
| 首页收尾 | 一名CLI集成writer接第3节六源 | 当前61tests+source0可复用，补新独审／文档parity／必要最终检查、精确stage/final/commit；不夹未知cache。 |
| 聊天／设置同时生产候选 | 各自只改独立render／settings源，集成writer统一接预检与提交，不能互覆 | chat修空壳/机器人；settings修moduleloader与视觉。浅深／窄窗／焦点／portal／工具／流式真实截图，不拿CSS测试代替。 |
| F I1已终结批次收口 | 一名Fwriter接四源＋target／binary责任 | 先确认binary原件／日志与reader，准确clean（用户原要求），再定向review与原语义／独立commit；F456不混入。CLI后继writer需等当前GUIscope串接但可先只读／候选。 |
| CLI身份／guidance隔离 | 明确同一writer只改subagent_control／executor/tests／ADR0013 | 先旧helper真实registry RED，后`.attempt_identity()`与同Run scopedqueue；sameRun重复拒、不同Runactive/settled、guidance一次归属和三真实分支。风险合同未明前不擅启CLI Cargo。 |
| I2真实readonly委派 | 依I1结果创建后继Plan，框架／应用薄适配共同负责 | 保既有ToolAccess、同registration准入／Team每member、snapshot/pipeline只读∩W、默认fallback拒新受限ctx；真实1/2/3及越权负控制。 |
| Fork两形式 | 只读事实memo→关闭dirty／Goal模型／非GitSSH边界→正式Design/Plan | 用户方向已确认，不能把当前create_worktree直接当完整workspacefork。每Turn唯一边界和失败后资源归属必须明确。 |
| 新Tauri统一批次 | 源稳定后一个构建owner准备完整bundle／assets／指纹／恢复与准确clean | 目前没有liveGUI，依赖风险／完整适用门禁分别处理；不能只替裸binary。具名制品准备好才安排用户维护切换及真人验收。 |
| 有效要求／IMcron／SSH／SDK／退役等 | 按第7节与26Outcome保完整目标、分文件owner并行 | A/B待答／手机扫码与GUI密码由用户；Fsource402外部排除仍保；SDK等正式来源，mainCI／发布部署分别授权。 |

交接后不得所有线路都停留在候选与审查：前端／独立模块／测试准备／中文文档／问题对账可并行；只有同文件写、重型矩阵／性能测量和真实未满足依赖串行。已受控结束的旧子任务不重复“唤醒”来取得假活状态。

## 11. 新协调者登记区

- 交接正文与附录已于 2026-10-27 07:10 UTC 完成只读完整性核对：原 Goal／新用户完整要求各逐字出现一次，五仓 HEAD／index／原 status 集合及冻结源码无漂移，仅增加本交接文件。回执 `.echo-semantic/local/coordinator-handoff-20261027/handoff-integrity-check-epoch02.json`；首轮把 `git diff --no-index` 的存在差异自然1误读为空白失败，原stdout/stderr均空已保留，第二轮按真实语义核准，无源码或文档修正伪装通过。
- 补充调度现场：`steer_goal_review2` 宿主状态仍显示 `pending_init`，旧Root已两次请求 interrupt，不能把它当作正在运行的验证或已有新审查；它无本轮生产写权／scope。其余六条任务均已返回只读交回终态。本文件没有采用该 pending reviewer 的任何新 pass。新协调者不应等待该旧句柄来取得当前六源审查资格。
- 新会话已经只读提供：五仓HEAD／index0／66branches18stash123tag，旧native56017／fixture55840不存在，settings1438的72559/72576/72861实际cwd，I1四源／1460输入／19test0及CLI未修，聊天未有修复、设置视觉仍失败。旧Root已用本次独立观察与这些结论交叉核对。
- **接管完成时间：2026-10-27T15:12:28+08:00（Asia/Shanghai），UTC 2026-10-27T07:12:28+00:00。**
- **接管后唯一主协调者：`01a17c1f-939d-7311-bba9-41e529912138` 的 `/root`。** 旧主会话已经停止新增调度／集成写入，只保留历史来源。
- 新会话应列出首批实际writer／精确paths／冻结和资源控制，再恢复实施；不默认本表来源owner自动在新会话继续。
- 本文件尚未提交；按“只写唯一交接文件”指令没有执行stage／commit。它在Root产生唯一预期新未跟踪路径，未改submodule pointers。

### 接管核验与首批实际写权

新root核对附录A/B逐字完整；五仓HEAD/branch/index/完整status及647现存文件SHA无漂移，25具名证据SHA一致。Root新增本交接文件为预期增量；初次观察至旧冻结间由旧owner补齐的CLI导航Asset已核验并保护。新会话Goal已active且引用用户最新goal-objective.md，整体未完成。原交接完整字节、当前核验及后继收据保存于根`.echo-semantic/local/new-coordinator-takeover-20261027/`。

| 当前唯一owner | 精确写入范围与交付／验证 |
| --- | --- |
| 新root | 首页第3节六路径整合与正常门禁／本地提交；Framework第3节四路径冻结、独审、门禁与提交；全部index/commit、共享语义预检、MASTER/next-actions/唯一handoff。首页五路径独审pass C0/I0/M0，61测试与466前端输入一致可复用，额外导航Asset补读审查。 |
| `/root/chat_ui_refresh` | CLI `web-frontend/src/components/chat/MessageBubble.tsx`、新增`MessageBubble.rendering.test.tsx`，以及`docs/zh/features.md`对应呈现说明；移除机器人、修真实空壳，保留流式／工具／错误／HITL／附件／思考／Subagent并浏览器前后核验。不写ChatPanel/Store/API。 |
| `/root/settings_ui_refresh` | CLI `web-frontend/src/components/layout/SettingsDialog.tsx`、新增`SettingsDialog.css`、`components/config/ConfigPanel.tsx`、`components/providers/ProviderPanel.tsx`及既有对应组件测试；复用patched，修深色红底按钮与预览loader，组件与浅深／窄窗／焦点／保存反馈验证。独占旧1438预览和本任务ignored支撑，按实际PID/birth接管监督及必要重配。不改变配置意义／默认值／保存语义。 |
| `/root/run_control_i1_eko` | CLI `echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs`、`executor/tests.rs`和`docs/zh/adr/0013-subagent-control-receipt-projection.md`；按精确Plan01先补旧helper真实RED，再切Run/guidance及相关控制。当前先源码／测试准备，风险指令／合同未裁决前不运行CLI Cargo、不改风险policy；批次由root统一排窗。 |
| `/root/fork_reuse_boundary` | 仅只读事实与brainstorm，复用既有memo／原语；dirty、Goal模型与SSH等未确认行为不实施。I2待I1交付后创建唯一后继Plan。 |

Framework准确target与已保全binary责任由新root接管；再次核无reader后按授权精准clean。旧native/fixture不存在，不假定旧维护许可允许新启动。所有子代理不stage/commit、不改manifest/lock/vendor/policy/SDK；源码验证期冻结相关依赖。未知pycache、zstd456、SDK94、全部refs/恢复包和用户数据继续保护。

新会话问有效要求A/B后，用户回复“啥意思？看不懂，请解释清楚。”；已用“先单独整理再干活／主模型同次理解执行”解释。助手倾向A只是建议，仍待用户选择，独立三条实现线继续。没有因此缩小26项Outcome或把交接／focused／本地提交当整体完成。

### 接管后的最新用户裁决与里程碑

用户在新会话明确补充（原文）：

> 普通 steer 都由主模型结合历史理解，新用户输入不能自动改写 Goal，也不需要统一强制的“有效要求更新 → 计划调整 → 验收调整”流程。
> 用户Steer由主模型在执行中更新计划，不需要经过独立的模型进行用户意图理解与更新。
> 只有在没有goal 时只智能体可以设置 goal ，在存在 goal 之后 goal 的更新、删除 只能由用户进行调整。

这取代第7节历史记录中的A/B未决、独立理解和强制自动传播方案。`/root/steer_goal_contract_revision`独占原GUI产品收敛Design/Map及必要Plan更新；`/root/goal_actor_authority`只读核对真实模型/用户Goal路径。当前源事实为普通Steer已有原文journal→tracked mailbox→主模型、模型task_update无Goal字段、用户GUI/IM沿原CAS；没有新建独立理解流水线的理由。取消执行与Goal改删先沿真实权威核对，不因此默认禁止合理取消。

首页六路径已于2026-10-27 15:19:01 Asia/Shanghai保存本地提交`3edd5200476c5ed7d570c9dc8461e341affc9cef`，docs parity及新root正常source/staged/final均0，独审pass；仅首页切片，不扩原生/全前端/Goal。原未知pycache未采用。CLI当前共享预检为`gui-presentation-takeover-20261027`（新root真实SID），只允许chat/settings精确10路径，backend/新Goal文档仍自有ignored候选。

旧Framework已终结target已在保全binary和日志、重新核空reader后准确cargo clean，实际自然0、移除2388文件/888.9MiB、target不存在；retained binary SHA仍`af10225356d9826ed5d930b38adef3228097019c946a2dfd7ff531d45168722b`。原件/回执位于`.echo-semantic/local/new-coordinator-takeover-20261027/old-i1-target-close/`，不是对其它target的清理或净磁盘收益声明。设置旧1438由其新owner核birth/cwd后换成自有监督预览，旧runner自然143作为停机事实保留；新预览不代表Tauri已更新。

### 15:45 前后接管进展（资格分开记录）

- 旧协调者已结束turn，宿主显示idle；旧会话自己的Goal后继为blocked（协调权已移交），原目标保留未标完成。新root的Goal继续active；第1节“旧Goal active”属于交接前事实，不代表旧协调者会恢复写入。
- 聊天/设置10路径已正式实现并冻结。聊天工具/空壳38测试与宽窄浅深4场景、设置24测试与42浏览器检查实际0；独审发现空thinking_start等待提示接缝后补真实Store→ChatPanel→Bubble两项RED，自然1，最小修复后12文件98测试/71输入无漂移自然0，等待同一reviewer增量复审。设置focus两项真实RED已修。两自有预览已结束，必要截图/日志保留，未冒新Tauri或完整前端资格。
- Framework I1四路径独审pass C0/I0/M0；接管后source首轮65项缺既有owner map引用，已只读定位并通过正常record追加3个HEAD字节一致map引用，不改工具/地图/排除、不扩大4路径写集。正式校验结果未出前不声明通过，zstd原402保持独立。
- 最新Steer/Goal裁决的Root Design/Map/现行Plan已由文档owner同步，旧Plan14经官方CLI标abandoned，不假装完成；新Plan15为原文/恢复和已有Goal用户权威验收，当前14份ready及Map机械validate0，整体Design/Plan独审进行。现有取消仅改变执行状态并保留Goal原文/revision/hash，未发现需要新增权限门控的反例。
- 依赖复评仍只读准备：CLI当前锁3fb856包与321消费输入27d587，六项具名身份与epoch08相同，机器policy仍绑定旧b0且源全集无法恢复；不把消费者漂移拒绝误叫新增漏洞或“退出码65”（它是脚本第65行）。原六测试有855包断言需按实际856维护，任何真正风险采用仍需具名裁决。I1应用RED/GREEN候选已准备，尚未新CLI Cargo。

### 已保存本地里程碑与接续（15:52后继）

Framework四源已提交`0e404185c8f252da3e335c35547e52249c74dc65`，3个既有map只读引用正常补齐后source/staged/final全部0，旧65与zstd402分别保留。CLI UI10路径已提交`9e58e706a0c601cb3f74b0d36c4d3c0fe6924ef5`；77文件629测试/lint/format0，真实build首轮3处测试ByRoleOptions.exact类型错误已修，后继22定向/真实类型/普通与Tauri前端build0，458输入无漂移；原行为独审pass延续并做非语义增量确认。提交说明曾误写24，已仅修文字为22、源码树不变，旧829a提交身份与修正回执保留。不等新Tauri或整体Goal通过。

新CLI依赖复评实际锁与canonical视图官方扫描各自然1、零漏洞/恰原6警告，完整身份新增/缺失/变化均0；官方HTTPS失败及首镜像HEAD失败原件保留，后继SSH fresh至官方3279c999后各scanner实际fetch同源本地镜像，未关fetch/yanked/增ignore。根`dependency-reassessment-exact-facts-20261027/`保存完整比较、321输入及锁322成员可恢复包，逐bytes/mode/hash回读；旧b0全集未复原。当前用户复评问题已明确为本次继续原六风险，不将扫描或数量相同自动晋升policy。用户尚未答前保留待决，不阻塞源码/测试准备。

原GUI产品Design/Map与17份实际文件改动已独审pass C0/I0/M0，新Plan15及14份ready和Map机械校验全0；旧abandoned14保历史不冒现绑定0。CLI ADR0037/0048/runtime正式说明候选待下一scope采用。`/root/steer_goal_validation`只在自有ignored目录准备原store/tests.rs边界回归及既有原文/恢复证据对账，不与I1的executor/tests.rs冲突，不运行额外Cargo。

## 附录 A：原 Goal 完整原文（逐字保留，不改写）

下列内容来自本会话Goal工具的 objective 字段，原文中的反斜线及HTML转义保留；后继裁决由第2节及附录B补充，不回写原Goal制造历史一致。

```text
作为 EKO 项目协调者，完成 EKO GUI 主入口和任务运行时的产品收敛，并在不破坏现有框架、SDK、IM、定时任务和共享恢复能力的前提下，交付可验证的本地个人超级助理。\
\
一、产品主入口与工作空间\
1\. GUI 左侧固定提供“新建任务、最近任务、工作空间”。\
2\. 工作空间、项目、工程使用同一概念；支持打开已有本地文件夹、新建本地文件夹、连接远程 SSH 设备作为项目工作空间。\
3\. 输入框左下提供工作空间选择和权限配置，右下提供模型选择；界面完成统一主题、排版、间距、导航、HITL 弹窗、任务列表弹窗和状态反馈的视觉收敛。\
4\. 保留并完善现有 IM 渠道和定时任务；个人微信通过 GUI 扫码连接，扫码、重连、过期、取消、单聊、HITL 和消息结果必须有可观察回执。\
5\. GUI 内置远程终端使用系统 OpenSSH，支持 echo@111.229.245.175；密码只能由用户在 GUI 中输入。验收中文显示、终端 resize、全屏程序、Ctrl-C、exit、断线、关闭和多会话隔离。\
\
二、TaskRun、PlanTask、SubagentRun 与用户要求\
1\. 保留并统一使用 TaskRun → PlanTask → SubagentRun，TaskPlan 是版本化可编辑/可审阅 artifact，Todo 只是 GUI 投影。\
2\. “当前有效要求”必须在同一 TaskRuntime 权威状态中持久化，保存原文、来源、作用范围、替换关系和版本；历史输入保留在 journal，压缩后仍能恢复最早尚未失效的约束，长输入末尾的限制不得因截断丢失。\
3\. 用户输入必须区分已接纳并持久保存、已进入模型上下文、已应用到目标/约束/计划、已取消/替换/失败；消费按稳定输入 ID 幂等，关键状态持久化失败时阻止相关副作用继续执行。\
4\. 关键 Goal、有效约束和执行范围恢复失败时必须重读、核对版本；无法恢复时暂停相关副作用并明确报告缺失内容。\
5\. 新要求影响目标、计划或验收时，更新对应 revision，重新检查受影响的计划和证据；旧结果只能在范围和版本仍然有效时复用。\
6\. 完成门禁检查“用户要求 → 工作项 → 交付物 → 验证证据”的覆盖率，能指出计划遗漏、未验证和已失效要求；程序证据检查与语义审查分开。\
\
三、输入框上方的 Steer 与 Goal 控制\
1\. composer 上方显示 Steer 列表和 Goal 条。\
2\. Steer 可以有多条，属于当前会话/TaskRun 的持久化输入：\
&#x20;  \- 支持新增、编辑、删除和拖拽调整顺序；\
&#x20;  \- 编辑保留稳定 input_id、目标 workspace/conversation 身份和队列顺序，使用 revision/queue fence，不能通过“删除后重新提交”伪造原子更新；\
&#x20;  \- 每条 Steer 显示 queued、accepted、retained、rejected、drained 或 settled 等可观察状态；\
&#x20;  \- “立即执行”只通过已有安全点和真实 steer receipt 触发；未到安全点时保留在队列并明确提示，不能冒充已执行；\
&#x20;  \- 工作区、会话切换、迟到回执、进程退出和恢复不能把旧 Steer 投影到新目标。\
3\. Goal 在同一 TaskRun 中只能有一个：\
&#x20;  \- 支持在 composer 上方直接编辑；\
&#x20;  \- 修改使用 expected_goal_revision 和 CAS，成功后立即刷新 canonical TaskRuntime projection；\
&#x20;  \- 修改后旧 Plan、GoalRequirement 和受影响证据立即标记失效，不能自动把旧计划当成新目标已完成；\
&#x20;  \- 修改期间必须显示暂停/安全点、失败原因和新 revision；\
&#x20;  \- 删除 Goal 的语义是结束并删除当前目标运行，不能向后端写入空 Goal，也不能留下仍可继续执行的无目标 TaskRun；\
&#x20;  \- 删除、修改和立即执行都必须有迟到回执、并发冲突和恢复场景的回归验证。\
4\. 子智能体默认不能再次派发子智能体；GUI 提供配置项，最大嵌套深度为三层，默认深度为零。产品和代码统一使用 Subagent 术语，不新增 Worker 概念。\
\
四、能力收敛与退役\
1\. 在消费者解除依赖、main/CI 验证和 GUI/IM/cron/共享任务恢复验收完成后，从 EKO 主线退役 A2A、LSP、TUI、独立 CLI、REPL 和 JSONL 专用入口。\
2\. 退役前保留一个固定分支和 tag，保存这些能力的完整历史，未来可从该分支继续丰富；不得先删除仍被框架、SDK 或合理复用方需要的公共 API。\
3\. 保留 GUI、IM、cron、内置终端、任务恢复、HITL、记忆、附件、工具和 Subagent 的核心能力对等性。\
4\. 不纳入本 Goal 的外部 echo-semantic 工具合同修复，保留现有校验阻塞和证据。\
\
五、框架、应用和存储边界\
1\. 通用依赖 DAG、重试、取消、revision safe point 等机制放框架；EKO 专属的本地工作空间、任务投影、权限策略、HITL 和 GUI/TUI/CLI 展示放应用。\
2\. 新增代码前先检查四个仓库是否已有实现，禁止平行实现同一语义；echo-agent 的公共 API 不能仅因 EKO 未使用就删除。\
3\. echo-agent-cli 不启用 SQLite；对话、记忆和任务恢复使用现有文件/内存实现。\
4\. 代码、ADR、正式文档、examples、生成类型和 echo-website 说明必须同步；架构变更记录 ADR。\
\
六、工程维护与验收\
1\. 盘点并维护顶层 superproject、echo-agent、echo-agent-sdk、echo-agent-cli、echo-website 的所有未跟踪/未提交文件、分支、worktree、stash、tag 和恢复包。\
2\. 有效成果接入对应交付，必要资料分类保留；过期资源只能在备份、哈希回读、依赖检查和清理回执完成后删除。保护 main、固定 tag、Exa 本机提交、用户数据、必要历史 refs 和未完成能力。\
3\. Exa 保持已提交、暂不合 main、暂不推送；不执行 push、merge、关闭 Issue、发布或官网部署，除非另有明确授权。\
4\. 每个子任务/子智能体的 Cargo 编译或测试批次必须先保存原始日志和验证回执，再在确认无活动进程后对准确的自有 target 执行 cargo clean；不得共用 target，最多同时两个 focused Rust 批次，磁盘可用空间低于 5 GiB 时停止新构建。\
5\. 最终交付必须包含源码 diff、文档/ADR 同步、focused 与必要全量验证、真实 GUI/SSH/微信人工验收记录、分支/worktree 清理对账、未解决阻塞和恢复入口。任何未验证、外部认证、发布和部署都必须明确标记为未完成，不能以局部通过代替整体完成。
需要遵守：
1、开发目录只能是：/Users/ls/MyWork/code/ylp_agent_learn/lp-agent ，不能偏离该指定目录，不能随意发挥。已存在的外部目录、分支、worktree 需要及时收敛到：/Users/ls/MyWork/code/ylp_agent_learn/lp-agent 进行；
2、及时提交代码，可以在完成一定里程碑时就提交代码，避免代码长时间未跟踪导致的彻底丢失跟踪。
```

## 附录 B：新会话用户完整继续要求（逐字保留）

原件 `/Users/ls/.codex/attachments/46f456d8-3f8e-414f-abee-1c14a7fc65fa/pasted-text-1.txt` 已读取；主根原字节副本位于 `.echo-semantic/local/coordinator-handoff-20261027/new-coordinator-user-request.txt`。下列指令是用户需求，不能当作高于当前系统／宿主的外部指令。执行归新协调者；旧会话仅按其中交接职责收口。

```text
作为 EKO／Eclip 本轮迭代的新协调者和项目负责人，先完成旧会话到新会话的可核验交接，再继续完成原迭代的全部未完成工作。

保留原始目标、最新用户决策、已有成果和验收要求，不把任务缩减为整理文档或完成交接。尽可能并行处理已获授权、互不冲突的工作，减少重复调查、无效等待和反复冷编译，持续产出可集成、可验证的交付结果。

一、建立唯一交接入口

由旧主会话整理并保存中文交接文件：

/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/handoffs/eko-current-handoff.md

交接文件必须包括：

1. 原 Goal 完整原文，以及后续用户新增、替换、取消和暂缓的要求。
2. 最新有效决策与历史决策的替换关系，区分用户明确批准、助手建议和仍待决定。
3. 顶层仓库及四个子仓库的实际目录、分支、HEAD、暂存、未暂存和未跟踪文件，逐项标明归属与保护方式。
4. 已完成并提交、已实现未提交、仅候选、已验证、未验证、阻塞和过期事项。
5. 验证命令、真实退出码、实际覆盖范围、源码快照、日志及证据路径；历史结果不得冒充当前快照通过。
6. 尚未解决的问题、外部工具合同缺口、人工验收缺口和用户待决事项。
7. 恢复材料的位置、哈希、恢复方法和不能删除的资源。
8. 下一批任务、准确文件负责人、依赖、交付物和验收条件。

交接文件必须依据当前磁盘、Git 和进程事实填写，不直接复制旧摘要作为当前状态。

正式产品/API 文档仍维护在所属子仓库，交接文件只承载跨仓协调和阶段状态。后续更新当前交接入口时保留必要历史依据，避免出现多份互相矛盾的“最新状态”。

二、记录并保护运行现场

记录仍在运行的编译、测试、GUI、模型夹具、页面服务、终端及其它相关进程，包括：

- PID、启动时间、命令、工作目录、端口和所属任务。
- 实际 manifest、CARGO_TARGET_DIR、输入快照、日志、制品和监督句柄。
- 当前 GUI 实际加载的二进制及前端资源版本。
- 不能中断的进程、未保存草稿、未答 HITL、SSH 活连接，以及退出或切换的影响。
- 已停止但尚未完成证据保存、制品保全或清理的批次。

不能仅凭状态文件认定进程仍活跃或已经结束。不得为交接盲目终止进程、重启服务、重跑编译或清理 target。

三、完成协调权交接

1. 旧协调者在明确切点停止新增派发和源码写入，仅完成必要的交接记录、证据保存和已有批次收口。
2. 正在验证的源码保持冻结。已有子任务逐项标记为继续执行、交回、已结束或需人工处理，避免重复调度。
3. 新会话先只读核对交接文件、当前规则、Git、进程、源码和证据；发现漂移时记录差异并更新事实。
4. 核对完成后明确记录接管时间、当前任务负责人和写入范围。任何时刻只有一个主协调者拥有新增派发和集成写入责任。
5. 旧会话保留为历史来源。新会话按需查阅旧历史，不把几天的完整聊天重新装入上下文。
6. 不假定 Goal、审批、子任务、进程句柄或验证资格会随新会话自动迁移。
7. 接管后沿用已经明确的用户授权；只有真实的新取舍、权限缺口或不可逆操作才需要再次询问，不重复请求已有授权。

四、继续完成产品与运行时目标

1. GUI、品牌与主交互页面

- 使用 Eclip 品牌，统一首页、侧栏、页面品牌图标、桌面图标及相关展示文案。
- 技术仓库名、协议标识、应用 identifier 和数据位置不得因品牌变化随意改名。
- 首页主标题使用“把想法变成成果”。
- 删除首页“继续上次对话”卡片，任务从明确的最近任务或工作空间入口继续。
- 删除顶部“新建支线对话”入口，不因此误删已有支线历史、共享对话能力或恢复机制。
- 去掉助手消息左侧的小机器人，修复空消息外壳造成的大块留白。
- 参考用户提供的 Codex 截图简化聊天布局：正文直接对齐，工具和执行过程紧凑呈现，详情可展开。
- 保留正文、附件、流式状态、工具执行、错误、HITL、Subagent、复制及其它必要操作，不能通过隐藏真实状态制造整洁。
- 同步优化设置页具体表单的排版、间距、控件、分组和反馈，不改变配置含义、默认值或保存逻辑。
- 基础配置常显；QQ、飞书和个人微信统一在独立 IM 渠道页面管理。
- 保持浅色/深色主题、响应式布局、键盘操作、焦点和可访问性。
- 评估并落实每轮完整回复末尾的分叉入口，以及“当前工作空间／新 Worktree”两种形式。先复用现有对话分叉、项目和 Worktree 能力；未确定的行为边界明确裁决，不另造平行机制。
- 必须核验用户实际运行的新版页面和桌面制品，不能把源码已修改或前端构建通过当作旧窗口已经更新。

2. 工作空间与远程能力

- 工作空间、项目、工程使用同一概念。
- 左侧提供新建任务、最近任务、工作空间。
- 支持已有本地目录、新建本地目录和远程 SSH 项目。
- 输入框左下提供工作空间与权限，右下提供模型选择。
- 验证远程项目文件、命令、Git、终端和产物使用绑定的同一目标。
- 保留原目标支持义务和后继已确认的 SSH 验收事实；不同账号、目录和验证范围分别记录，不能互相替代。
- 密码、私钥只由用户在 GUI 输入，不进入聊天、日志或提示词。
- 完整验收终端中文、resize、全屏程序、Ctrl-C、exit、断线、关闭及多会话隔离。

3. TaskRun、Steer、Goal 与有效要求

- 保留 TaskRun → PlanTask → SubagentRun 单一权威模型。
- TaskPlan 是版本化可编辑/可审阅 artifact，Todo 是 GUI 投影，不另建任务 CRUD、store 或执行器。
- Steer 支持多条、编辑、删除、排序及立即执行。
- 稳定 input_id、revision/CAS、目标身份、队列顺序和真实投递状态必须成立。
- 编辑不能通过“删除后重新提交”伪造原子更新。
- 立即执行沿已有安全点和真实回执处理；尚未执行时不能冒称已执行。
- Goal 在同一运行只能有一个；修改立即更新权威状态，并使受影响的旧计划和证据失效。
- Goal 删除不得写入空目标，也不得留下仍可执行的无目标运行。
- 完成输入耐久接纳、幂等消费、恢复和失败反馈。
- 在同一 TaskRuntime 中维护完整原文、当前有效要求、作用域、替换关系和版本，不能只保留最近几条输入。
- 长输入末尾的约束不得因截断丢失。
- 关键目标、约束或范围恢复失败时，阻止相关副作用继续执行，并明确报告。
- 建立“用户要求 → 工作项 → 交付物 → 验证证据”的覆盖关系，能够识别计划遗漏、未验证和证据失效。
- 程序证据检查与语义验收分别处理。
- 有效要求解释路线等仍待用户选择的事项保持待决，不把推荐方案当作已批准。

4. Subagent

- 主智能体为第 0 层；EKO 默认只允许首层子智能体，额外嵌套深度默认 0，GUI 可配置最大总层数至 3。
- 框架通用复用方的合理能力与 EKO 产品默认分别处理。
- 修复跨 TaskRun 的 attempt 身份碰撞及 guidance 串流。
- 修复真实只读子智能体工厂的受限委派，保留只读限制、工具正上界和每层深度约束。
- 验证同任务 ID 的跨 Run 隔离、重复尝试拒绝、指导一次消费，以及真实工厂的 1／2／3 层和越界负控制。
- 不通过随机任务 ID、清空去重记录、放宽权限、等待或栈限制制造通过。
- 产品、代码和文档统一使用 Subagent，不新增 Worker 术语。

5. IM、定时任务及共享恢复

- 完善现有 QQ、飞书、个人微信和 cron 的 GUI 管理、运行状态、真实回执和恢复。
- 个人微信验收扫码、确认、取消、过期、重连、私聊、HITL 和手机实际收件。
- 区分配置成功、连接 Ready、发送 Accepted 和对方实际收到。
- 验证定时任务定义更新、启停、触发、运行结果、通知和恢复的真实归属。
- 人工扫码、密码输入和真实服务验收缺口明确保留，不用 mock 或局部测试代替。

6. 框架、SDK、工具治理与退役

- 通用机制归 Framework，EKO 产品策略归应用；新增前查全仓既有实现，禁止重复权威。
- 不因 EKO 未调用就删除合理框架公开 API。
- 继续内置工具治理和真实消费者验证。
- 保留 zstd 修复成果、原失败及外部 source 校验阻塞。
- 不修复本 Goal 已明确排除的 echo-semantic 工具合同，不造形式证据、不扩大排除项、不冒称通过。
- SDK 按自身精确 pin、保留合同和依赖政策验收，不能自动继承 Framework 或 CLI 的资格。
- 消费者解除依赖并完成 GUI／IM／cron／共享恢复和保留合同验收后，退役 A2A、LSP、TUI、独立 CLI、REPL、JSONL 产品入口。
- 保护固定分支、pre-retirement tag 和完整历史恢复入口。
- GUI 内置终端、Agent shell、构建命令和 SDK Host stdio 保留。
- CLI 不启用 SQLite。

五、并行优先，持续交付

尽可能并行处理已获授权、互不冲突的任务，加快整体交付速度。协调者应主动拆分独立工作、明确文件负责人并及时派发，不能让可执行任务长期等待。

具体要求：

1. 前端、后端独立模块、测试准备、文档、问题对账和资源治理并行推进。
2. 只串行处理冲突文件写入、依赖尚未满足的操作、重型编译和正式性能测量；不得把局部资源限制扩大成所有任务等待。
3. 并行任务持续产出可审阅、可集成的实现、测试、文档或证据，不能全部停留在调查、方案或等待交接。
4. 某项前置完成后立即启动其下游，不等待整批任务全部结束。
5. 完成一个可交付切片就及时审查、验证、集成并保存本地提交，不长期积压未跟踪成果。
6. 同一文件保持单一写入者，验证期间冻结相关输入；共享文件由明确负责人统一整合。
7. 协调者持续检查闲置任务、重复劳动和无效等待，在既有授权内及时重新调度，使并行度转化为实际交付速度。
8. 避免重复盘点、重复全量验证和无必要的流程材料。优先复用有明确来源与适用范围的现有证据，只有输入变化、失败或未解决风险才追加验证。
9. 合理组织相邻验证批次，减少反复冷编译；不能以保留缓存为由违反批次结束后的清理要求。
10. 正式性能测量使用安静窗口，不能与重型编译混跑后宣称测量有效。

六、下一批任务与执行顺序

交接后依据当前事实排出明确批次，优先：

1. 并行完成首页、聊天留白、机器人移除、分叉入口梳理及设置页视觉收尾。
2. 修复 Subagent 运行身份/guidance 隔离，再完成只读委派和真实深度验证。
3. 文档、示例、生成类型、官网来源和问题对账随对应实现持续同步，不全部拖到最后。
4. 源码稳定后统一准备新版 Tauri 制品，保存指纹与恢复材料，准确清理编译产物，再安排原生切换和人工验收。
5. 按依赖推进有效要求、微信、SSH、工具治理、SDK、能力退役及完整门禁。
6. 文件存储性能基准、新增其它 IM 平台、包发布和官网部署按已确认需要及授权安排；未授权外部动作只做准备。

每个任务必须写明：

- 具体交付结果与准确文件负责人。
- 当前输入、依赖和不能修改的范围。
- 验证方法、完成条件和证据位置。
- 是否需要人工输入、外部授权或独立资源窗口。

七、工程维护与授权边界

- 唯一开发根为：
  /Users/ls/MyWork/code/ylp_agent_learn/lp-agent
- 四子仓独立维护和提交，不新建外部 worktree。
- 维护全部未提交/未跟踪文件、分支、worktree、stash、tag 和恢复包。
- 保护未知归属、用户数据、有效成果、必要历史及活动任务。
- 清理前完成来源、依赖、备份、哈希回读及恢复验证。
- 保持 Exa 已提交、本机保留，暂不 push、不合 main。
- 及时保存已验证的本地里程碑，主协调者统一 stage/commit；所有 commit 显式关闭 GPG 签名。
- 不擅自 push、merge、关闭 Issue、发布包或部署官网。
- 不晋升指向未推送子提交的顶层 gitlink。
- 依赖风险按各仓当前真实用户裁决及精确 lock/consumer/source 合同执行，漂移不得自动续期。
- 最多同时两个 focused Cargo 批次，各自独立 target、jobs≤2、debug=0；完整重型矩阵同一时间只运行一套。
- 可用空间低于 5GiB 不启动新重构建。
- 每批编译/测试结束后，先保留日志、真实退出码和必要制品，确认准确 target 无活动读者，再及时 cargo clean。
- 纯文档或只读任务未产生 Cargo 产物时，不清理其它任务的 target。
- 新增或修改的工程文档使用中文。

八、完成标准

交接完成必须证明：

- 原目标和最新用户要求完整。
- 决策可追溯，五仓状态准确。
- 运行现场和不能中断的进程明确。
- 下一批任务、文件负责人及待决事项清晰。
- 旧协调者已停止新增派发和写入。
- 新协调者已只读核对并明确接管。

整个 Goal 完成还必须逐项证明产品行为、源码、文档、适用测试、真实原生/SSH/微信验收及工程维护达到要求。

交接、局部测试、本地提交或构建成功均不能替代整体交付。原生、完整矩阵、精确 main/CI、发布和部署分别报告资格。

证据不足、仍有缺口或尚未取得必要人工输入时，保留完整目标与未完成项，不缩小成功标准，不提前标记完成。
```
