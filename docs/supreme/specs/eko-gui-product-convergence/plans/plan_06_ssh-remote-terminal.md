---
schema_version: 4
slug: eko-gui-product-convergence/ssh-remote-terminal
outcome:
  summary: 用户可从 GUI 经现有终端服务和系统 OpenSSH 连接并操作远端，真实验证认证、交互、resize、中断、多会话及断线/关闭。
  acceptance:
    - 用户可在既有 GUI 终端打开本地或 SSH 会话，输入 host/现有 SSH alias 及可选
      user/port，看到与后端真实会话绑定的目标；系统 OpenSSH 负责认证与 host-key 处理，直接用户操作不受 Agent
      full-auto 门控。
    - SSH工作空间创建表单提供临时掩码密码栏；测试连接只验证SSH认证，不打开终端、不依赖项目目录。创建时再经原连接校验远程目录，相对路径按实际远端home解析；本次目标为ubuntu@111.229.245.175，ssh-test对应/home/ubuntu/ssh-test。外部系统终端成功不代替EKO真实验收。
    - ubuntu账号的真实成功只结算该目标；Goal要求的echo@111.229.245.175支持仍须具名验证，不由ubuntu或侧栏刷新成功替代，也不改写旧echo探测失败。
    - 密码不进入持久项目配置、日志或模型，取消/退出表单后清理临时输入；连接目标变化或迟到结果不能把旧认证成功用于新目标。普通完整终端的交互、输出和生命周期验收全部保留，不把“测试连接无需终端”扩为删除终端能力。
    - 首批输出、认证提示、中文/emoji
      和多会话切换/隐藏期间的输出可观察；输出与渲染内存有界，超出可保留范围或传输缺口可发现，不把缺失历史、进程启动或界面动画冒充完整恢复或远端登录成功。
    - 连接/认证/退出失败、resize/Ctrl-C、断线及关闭交错都有真实反馈和原会话owner；成功关闭须证明该Session拥有的本地进程与输出读者已结算后才移除/publishClosed，未知可重试不是正常关闭的替代完成。OS/kill/wait或读者失败仍保留owner/目标和重试；断线不自动重放命令，本地SSH结束不伪称远端副作用已停止。
    - 在明确的平台、源码/依赖锁与隔离开发数据下完成真实 Tauri
      加真实目标的交互验收及相关回归、独立复审和文档/示例/官网处置；记录实际退出、未验范围和批次后准确自有 target 的 cargo
      clean，主线/CI/发布状态分别报告。
out_of_scope:
  - 不引入 SSH 协议库、远程 daemon、远程 IDE、文件同步或端口管理；不新增 Agent 远程工具 schema、TaskRun/任务
    store、框架终端执行器或第二终端生命周期 owner。
  - 不强制实现保存连接列表、EkoConfig 扩展、密码/私钥存储或自动读取/复制 SSH
    config；允许本次明确批准的临时掩码密码输入，禁止将其持久化。OpenSSH继续使用用户既有配置与认证设施，不新增权限门控或安全审批。
  - 不扩张为浏览器 HTTP/WebSocket 远程终端产品或 WebView/应用重启后的活跃 SSH
    恢复；保留已存在本地消费者合同，已知缺口不得伪称完全可恢复。
  - 不改 A2A/LSP 退役、IM/cron、HITL/任务运行状态或全局主题权威；共享接缝仅在当前 sole writer
    串行交接后由指定负责人集成。Exa、保留基线/tag、stash 和用户原件不在本结果清理范围，实际
    push/merge、包发布与网站部署按各自授权处理。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 7. SSH 远程终端
    digest: sha256:1f39a94fc5f02d8044d7d738533dc97c2cef2f6109d70ab47dcdb96b2e3dfd43
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:f028430a0b75b26899c68a1a5ed27f687da9f70ce63aa96ce0d46ec11216132c
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#ssh-remote-terminal
todos:
  - id: launch-an-explicit-terminal-target
    summary: 复用原连接与终端链路表达真实目标，工作空间表单将临时密码认证测试与创建目录校验分开。
    files:
      - echo-agent-cli/echo-agent-app-core/src/terminal.rs
      - echo-agent-cli/src/tauri/terminal.rs
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/web-frontend/src/components/terminal/TerminalDrawer.tsx
      - echo-agent-cli/web-frontend/src/components/workspace/NewTaskDialog.tsx
      - echo-agent-cli/web-frontend/src/components/workspace/WorkspaceDialogs.a11y.test.tsx
      - echo-agent-cli/src/tauri/commands/workspace.rs
      - echo-agent-cli/echo-agent-app-core/src/terminal/project_ssh.rs
    acceptance:
      - 创建表单以掩码显示临时密码；测试连接只通过原SSH连接验证认证可用，目录为空、相对或不存在均不能成为认证测试失败原因，不打开黑色终端或要求在终端输入密码。不以按钮完成、子进程启动或普通提示文本猜认证成功。
      - 创建时再通过同一目标连接校验远程目录；ssh-test按实际远端home解析为本次/home/ubuntu/ssh-test，不按本机cwd或硬编码默认home解析。保留绝对路径及明确的认证/目录错误，测试成功不绕过创建校验。
      - 密码仅用于当次认证，不写入持久项目配置、日志或模型；取消/退出清理临时输入。目标或账号/凭据改变、表单重开和旧结果迟到时保留原身份约束，不把先前成功用于新目标；复用现有连接和清理owner，不另造状态机。
      - 在原 TerminalService/portable-pty 中使用系统 ssh 程序和独立 argv 表达 destination、可选
        user/port 与交互 PTY；host 可为现有 alias，未填 user/port 不覆盖 OpenSSH 配置；不通过 shell
        拼串或向默认本地 shell 写 ssh 命令冒充远端目标，缺少 ssh 或错误输入明确失败且不回退本地执行。
      - 本地默认 shell、cwd、id 去重及尺寸校验继续工作；创建/list 返回的会话目标元数据由同一后端 owner 产生，Drawer
        的目标标识与之对应，修正现有 TerminalSession/close 响应声明与 Rust 实值不一致，不增加虚假的
        cwd/created_at。
      - 核对既有DeveloperCommand、HTTP与模型工具的真实注册和调用路径；用户手工SSH连接不构成Agent自动执行授权。若存在自动调用方，只能显式绑定同一目标并复用既有权限/执行证据；无自动消费者时保留源码证明。本结果不新增Agent远程工具或专用schema，也不给直接用户终端操作增加Agent权限模式门控。
  - id: preserve-interactive-output-observation
    summary: 闭合输出订阅与渲染生命周期，保证 SSH 认证提示和持续交互不因 tab/抽屉显示切换丢失。
    files:
      - echo-agent-cli/echo-agent-app-core/src/terminal.rs
      - echo-agent-cli/src/tauri/terminal.rs
      - echo-agent-cli/web-frontend/src/components/terminal/Terminal.tsx
      - echo-agent-cli/web-frontend/src/components/terminal/TerminalDrawer.tsx
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
    acceptance:
      - xterm及Tauri输出/退出listeners就绪后才调用既有create，不新建后端reserve/start两阶段；各会话组件跨切tab/抽屉隐藏保持挂载。证明输出早于create返回、xterm异步就绪、快速退出、切tab与隐藏/重新打开抽屉均不吞掉原会话首输出/认证提示；pending创建取消与监听cleanup竞争不产生孤儿会话，不给其它tab写入或重复写入。
      - PTY bytes/base64反解为raw Uint8Array交给xterm，中文/emoji跨多个输出块不出现错误替换；不逐包new
        TextDecoder，不新建Rust解码器或并行输出store。维持有界scrollback和write输入队列，实际broadcast
        lag/队列超限产生明确可见的失败或输出缺口，不把缺失字节称为完整历史或完整VT屏幕可恢复。
      - 显示的运行/等待输入/退出状态依据可获得的后端事实；启动本地 ssh 子进程与远端完成认证明确区分，不靠 shell
        prompt、普通输出字符串或 Connected 动画制造登录成功证据。
  - id: settle-the-original-terminal-process
    summary: 让关闭、断线与应用退出结算原终端子进程，保留失败和未知边界。
    files:
      - echo-agent-cli/echo-agent-app-core/src/terminal.rs
      - echo-agent-cli/echo-agent-app-core/src/runtime.rs
      - echo-agent-cli/src/tauri/terminal.rs
      - echo-agent-cli/web-frontend/src/components/terminal/TerminalDrawer.tsx
      - echo-agent-cli/web-frontend/src/components/terminal/Terminal.tsx
      - echo-agent-cli/src/cli/channels/outbound.rs
      - echo-agent-cli/src/cli/channels.rs
      - echo-agent-cli/src/cli/cmd_impls/developer.rs
      - echo-agent-cli/src/tui/events.rs
    acceptance:
      - 自然退出保留真实exit code/必要错误。成功close/close_all必须由原TerminalSession
        owner证明其本地child wait和输出reader
        join均完成后才remove/publishClosed，不把kill返回或未知可重试当正常关闭完成。优先保留/调用portable-pty原Child::kill的现成HUP后强杀升级能力，沿同一owner结算kill/wait；不复制framework私有tokio-child
        owner。UI等待有界，OS/kill/wait失败或读者未结束时保留owner/目标、明确未证明关闭并可重试，不无限阻塞UI、不abort原wait伪造完成。caller取消、重复close、自然退出和旧id晚到清理不丢句柄、不误删新会话或重复终态；不把direct-child回收夸成process-group/远端副作用全部停止。
      - 关闭失败不静默删除 tab 或掩盖仍在运行的会话，错误可重试；渲染观察者隐藏/释放不等于关闭进程。应用真正退出经既有
        ApplicationLifecycleOwner 和 Tauri bridge supervisor 完成原资源收束，不新增退出
        owner；修改事件合同后同步保留中的 CLI/TUI/channel 消费者。
      - 断线、认证失败、host-key 冲突和未知 SSH
        退出均保留原目标及可解释诊断，不自动重连重放副作用命令；用户本地关闭仅报告可证明的本地结算，不宣称远端命令/子进程或已产生副作用全部停止。
  - id: prove-native-and-remote-terminal-behavior
    summary: 用有意义的本地反例和真实桌面远端交互验证 SSH 终端合同。
    files:
      - echo-agent-cli/echo-agent-app-core/src/terminal.rs
      - echo-agent-cli/echo-agent-app-core/src/developer_commands.rs
      - echo-agent-cli/src/cli/cmd_impls/developer.rs
      - echo-agent-cli/src/tauri/terminal.rs
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/web-frontend/src/components/terminal/
    acceptance:
      - 复用现有 oversized_writes、slow_subscriber、process_exit、concurrent_create 及
        pre_dispatch_receiver 场景，补齐真实 argv/目标边界、首输出、跨包 UTF-8、订阅
        setup/cleanup、tab/隐藏、多会话隔离、窗口 resize、Ctrl-C、非零退出、kill 失败可重试和关闭物理 wait
        的反例及本地正常控制；验证方法不得仅镜像实现或抬阈值绕过失败。针对clone_killer仅向Unix direct
        child发送SIGHUP的真实依赖，提供忽略SIGHUP的direct
        child反例，证明原Child::kill等既有primitive的可行终止、原wait及输出reader结算；同步证明本地默认shell正常关闭。OS/kill/wait失败、读者未结束或观察等待超时明确未证明关闭，保留同一owner和重试UI；需要新组信号策略时先回协调者，不把未验证组终止写成成功。
      - 在真实 Tauri 1400×900 与 900×600、浅/深主题下，通过用户最新指定 ubuntu@111.229.245.175
        完成交互认证、主机/账号身份核对、中文/emoji、全屏程序、resize 后远端尺寸、Ctrl-C、多会话、断线/关闭/应用退出；使用隔离本地
        EKO_DATA_DIR 和不写远端业务文件的验收操作，认证/host-key 交互不进入保存日志。
      - 表单独立验收掩码密码、无终端认证测试、认证失败、创建时目录失败/成功、相对ssh-test按远端/home/ubuntu解析及目标切换迟到结果；外部系统终端登录和mkdir成功只作对照，EKO须分别取得认证测试、创建目录校验与普通完整终端的真实结果。
      - 真实认证不可用或任一原生/远端场景未通过时保留该项未完成，mock、jsdom、编译或已到认证阶段不能替代。保存对应源码/锁、工具/系统版本、命令、真实出口、场景截图及未验平台；执行实际修改触发的
        EKO Rust/GUI/前端门禁，所有已执行失败均修复后同入口通过。
  - id: document-and-deliver-the-qualified-terminal
    summary: 同步正式产品说明、架构依据和官网影响，交付可复核的 SSH 终端结果。
    files:
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/features.md
      - echo-agent-cli/docs/zh/features.md
      - echo-agent-cli/docs/en/adr/
      - echo-agent-cli/docs/zh/adr/
      - echo-agent-cli/docs/doc-parity-manifest.json
      - echo-agent-cli/examples/lh6_product_soak.rs
      - echo-website/docs-sync-manifest.json
      - echo-website/src/docs/content/
    acceptance:
      - 在所属EKO中文正式文档与ADR记录已选输出/生命周期方案、创建表单临时密码、测试认证与创建校验目录的边界、目标/退出含义及实际支持平台，不为形式新增英文副本，既有代码标识与历史证据保原文；更新现有运行时清单中
        SSH仍属未来的描述，保留其它未交付退役事实。文档检查按所属仓当前中文规则及实际修改范围执行；examples 无
        SSH/终端关联时明确不适用，如修改或新增示例则纳入可执行编译/测试链路。
      - 独立代码/真实桌面复审通过后，由唯一接缝负责人按交接后的准确来源集成并核对原视觉/LSP
        去依赖和保留消费者；官网只镜像实际已交付源文档，绑定精确 source/manifest，未经批准不部署。完整门禁、精确 main CI
        与发布分别结算，不以本地贡献完成替代。
      - 每个实际 Cargo 编译/测试批次结束后先将必要日志、真实退出与需保留制品移至 target 外，再确认无
        cargo/rustc/Host/桌面制品读取者，cargo clean 本任务准确 manifest/target 并记录
        df/du；使用自有 target、jobs<=2、dev/test
        debug=0，低于5GiB不启新重任务，全量/GUI重矩阵按协调者资源租约串行。
artifact_id: plan:08c22af5-243e-4460-a70c-2c5e2ec186b6
lifecycle: ready
---
## Notes

本候选的调查输入为 EKO db5010ed888cf50f2400fbba31075ab63c5626e9、framework c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa。GUI 基础和工具盘点依赖已由协调者报告本地审查交付，main/精确 CI 仍单独待验；不把本地贡献写成主线或发布完成。

实际实现必须等待 a2a-lsp-consumer-decoupling 的 EKO 唯一写入者冻结并交接来源，再由协调者记录新的精确 EKO/framework/lock 身份及 SSH 写入范围。AppState、runtime、Tauri invoke 注册、共享 endpoints、Settings/导航目前由该会话独占；SSH 不并发写这些接缝，也不将 db5010e 的整文件覆盖到其后继。TerminalService、Tauri terminal adapter 与 terminal UI 的增量沿该明确后继串行集成，保留已交付视觉与 LSP 去依赖结果。

协调者已冻结输出方案A：xterm与Tauri listeners ready后才调用既有create，不新增后端两阶段spawn；各会话组件跨切tab/抽屉隐藏保持挂载。首输出、切换/隐藏、raw Uint8Array跨包UTF-8和广播lag/队列超限的明确缺口诊断均为本结果必要验收。WebView重建、崩溃或应用重启后的VT屏幕恢复未被当前Design承诺，不作为已确认能力延期，也不隐含声称完整恢复/重放；不新增store/cursor缓存。

历史echo@111.229.245.175的非交互探测exit255 / Permission denied保留原账号与时态。2026-10-26实际成功验收使用ubuntu@111.229.245.175、home=/home/ubuntu；它不取消Goal对echo账号支持的要求。ubuntu具名原生成功与后继侧栏反馈见下段，echo支持仍待具名验证；这些记录不授权本会话再次连接或读取密码/密钥。

用户最新明确要求工作空间创建表单使用临时掩码密码栏，测试连接只验SSH认证且不出现终端，创建时再校验远程目录，相对ssh-test按远端home解析。后继原生epoch10用户已确认登录测试与创建成功，又确认“刷新后已显示ssh-test”；保留PID43779/b330制品和SSH连接，前端148文件manifest25d8584a已加载。普通完整终端及原输出/结算、远程项目剩余验收继续开放，Plan仍ready；不把侧栏显示或设置实现加载扩大为完整矩阵通过。密码不入持久项目配置、日志或模型，不新增状态机或审批。文件清单仅表示相关实现范围，写权按总调度最新精确派发，不能由本Plan自动取得Hook/API或Rust写权。

下段保留2026-10-25原范围澄清的历史时态，不覆盖上述2026-10-26账号/交互变更与真实失败反馈。

2026-10-25 范围澄清：用户已明确侧栏“任务”相当于项目，新建来源包括已有本地文件夹、新建本地文件夹或远程项目。本 Plan 的 artifact_id、`ssh-remote-terminal` Outcome 与全部原终端验收保持不变；它可以独立交付为项目工作环境的可复用终端底座，但独立 SSH 终端登录或本 Plan 完成不代表 `project-workspaces` 的远程项目文件、Agent 执行、Git、隔离与产物验证已经完成。原 SSH 真实验收询问已由用户说明项目初衷与范围作答，不再记作未回复；该答复不是 SSH 登录通过或失败的结果。尚未取得的原生、认证及远端交互证据继续保持未完成，不改写为通过或用户反馈失败，也不再催取密码；前述本机非交互尝试记录保留其原历史边界。
