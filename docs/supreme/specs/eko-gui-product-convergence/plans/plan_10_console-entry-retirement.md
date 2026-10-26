---
schema_version: 4
slug: eko-gui-product-convergence/console-entry-retirement
outcome:
  summary: EKO 独立 TUI、REPL/CLI/JSONL 和专用启动入口退出主线，GUI、IM、cron、内置终端及共享任务恢复保持可用。
  acceptance:
    - 独立 TUI、REPL/CLI/JSONL、standalone channels
      模式及其专属分派、provider、参数、补全和依赖退出生产构建；GUI-only 构建也没有 --jsonl
      等旧入口逃逸，现有桌面启动器继续有效。
    - GUI 最小启动参数、会话/任务恢复、IM 收发与 GUI/IM HITL 竞争、cron occurrence 幂等与结果、内置本地/SSH
      终端和 MCP 保持原语义；隐藏/恢复窗口不停止后台服务，真正退出仍由原 owner 结算执行与清理义务。
    - 最终同源快照通过所属仓适用的工程、依赖政策、合同和语义门禁及独立审查；真实桌面、现有渠道和终端行为分别取证，未执行的平台/服务不称通过。正式交付仍需获授权的
      main 晋级及精确 main CI 全绿，本地准备或 checkpoint 不替代该门槛。
out_of_scope:
  - A2A/LSP 的物理退役、工具治理、领域插件、新 IM 平台、GUI/HITL/任务美化和性能优化各由其既有 Outcome 负责，不借本计划扩张。
  - 不改包/crate 品牌、版本或发行目标，不新建启动器、daemon、dispatcher、任务/记忆 store、数据库或人工 Agent
    协作组控制台；不移除 framework 合理公共能力或 SDK Host stdio。
  - 不删除用户会话、任务、配置、记忆及历史 JSONL journal，不改保留分支/tag，不合并 Exa；本 Plan 不授权 push、PR、main
    合并、Issue 关闭、包发布或网站部署。
  - 人类 B 排除的外部 Echo Semantic 工具修复不在本结果内；遇到真实门禁阻断保留原失败并报告，不以结构检查或排除声明冒充最终通过。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 1. 文档定位与已确认目标
    digest: sha256:0f17d2cbbec5131fac3d3c4b955710ddcf09a84c2872177492af6425a4a7ac4f
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 5. 入口收敛与 A2A/LSP 退役
    digest: sha256:c9c8f28566cfe1936dad26af7d33c6ec5a8fa0a0b9af1215b1ae3d8e7cffc09d
  - ref: design.md § IM 与定时任务管理
    digest: sha256:0d9fc52c40b8eff51cac3a953a25738351f53e00ae3728e7fdab947d383aa9fa
  - ref: design.md § 7. SSH 远程终端
    digest: sha256:1f39a94fc5f02d8044d7d738533dc97c2cef2f6109d70ab47dcdb96b2e3dfd43
  - ref: design.md § 10. 异常与边界验收场景
    digest: sha256:6f59f80f3c5fe48e08bac490651377bf47f90e5dbde6560e47c4e5d16d9fcde5
  - ref: design.md § 12. 复用结论与完成标准
    digest: sha256:1e7084072aa642175303e9e30b436fb4f9b2aa0e460d654bb4e74abb3ad51815
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#console-surface-retirement
todos:
  - id: desktop-args-decoupling
    summary: R1：接入并验证已有 DesktopArgs 归位，使桌面启动不再依赖 cli::Args。
    files:
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
    acceptance:
      - 复用已实现的四字段 model/project/mcp_config/config 和原解析入口，保留 -m、MODEL_NAME、Finder
        -psn_ 过滤、configure_data_root 先于 store、MCP 参数/YAML/env 优先级及原
        bootstrap/compose/rollback/退出结算。
      - 在稳定 APP 后继源码上核准既有补丁的真实接线与适用验证；原 load_config Result/map_err、wait_final
        和其余启动行为不丢失，不把原 GUI 未消费的 resume/web/port 等参数新增为 GUI 功能。
  - id: auto-memory-owner-decoupling
    summary: R2：接入并验证已有自动记忆状态归位，GUI 与过渡中的 console 消费同一原 atomic。
    files:
      - echo-agent-cli/echo-agent-app-core/src/auto_memory/mod.rs
      - echo-agent-cli/src/tauri/commands/panels.rs
      - echo-agent-cli/src/cli/cmd_impls/all.rs
      - echo-agent-cli/src/cli/repl.rs
      - echo-agent-cli/src/tui/events.rs
    acceptance:
      - 原 AtomicBool 仅一份并归既有 app-core auto_memory 模块，经现有 api::auto_memory 导出；默认
        true、进程全局范围、Relaxed 读写和原 14 处读写语义保持，不新增持久化或第二权威。
      - GUI 仍先验证 workspace/generation/evidence lease
        再写开关；非法或过期输入不改变状态，原生成/关闭/取消结算控制保留。待退役 console 引用只在删除前作薄过渡，不作为长期兼容层。
  - id: remove-console-exclusive-surfaces
    summary: R3：前置接缝与 GUI/IM/cron 资格交接满足后，删除独立 console 入口及其专属呈现和启动包装。
    files:
      - echo-agent-cli/src/main.rs
      - echo-agent-cli/src/cli/
      - echo-agent-cli/src/tui/
      - echo-agent-cli/echo-agent-app-core/src/hitl/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/hitl/repl_provider.rs
      - echo-agent-cli/echo-agent-app-core/src/hitl/tui_provider.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/config.rs
      - echo-agent-cli/echo-agent-app-core/src/infra/logging.rs
      - echo-agent-cli/config/eko.example.yaml
      - echo-agent-cli/tests/jsonl_subprocess.rs
    acceptance:
      - 删除旧 main 的模式分派、CLI/TUI 模块和 JSONL 专属 sink/provider，撤销
        --tui/--cli/--jsonl/--channels 等 console 模式、补全与专属参数；不只关闭 feature
        而长期保留原控制器。
      - ChannelRuntime 中仅删除 ChannelsModeArgs/run_channels_mode、专属 finish helper
        及确属该入口的测试；已归 app-core 的真实渠道收发、管理、text_commands、身份/附件/结果/HITL、transport
        watch/epoch 和 stop debt 保持。TuiConfig/logging 分支按最后实际使用核准后退出。
      - 保留共享 drive、TaskRuntime/ConversationStore/boot recovery、scheduler
        store/delivery journal、GUI/resume/browser 和原 GUI/IM
        provider/pairing；JSONL journal、ChatEventEnvelope/replay、PTY/系统 SSH、MCP
        shell-words、SDK Host stdio及自动 Subagent
        协作不被名称或目录连带删除。专属测试退出时，将其中仍适用的取消、未结算和身份断言保留到真实 retained owner 测试。
  - id: converge-entry-build-contracts
    summary: R4：唯一集成 owner 一次归位模块注册、桌面构建目标、manifest/lock 与退役说明。
    files:
      - echo-agent-cli/src/lib.rs
      - echo-agent-cli/Cargo.toml
      - echo-agent-cli/Cargo.lock
      - echo-agent-cli/echo-agent-app-core/Cargo.toml
      - echo-agent-cli/.github/workflows/rust-ci.yml
      - echo-agent-cli/README.md
      - echo-agent-cli/docs/en/adr/0050-gui-product-convergence.md
      - echo-agent-cli/docs/zh/adr/0050-gui-product-convergence.md
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/configuration.md
      - echo-agent-cli/docs/zh/configuration.md
      - echo-agent-cli/.cargo/accepted-dependency-risks.json
      - echo-agent-cli/docs/supreme/specs/cancellation-observation-consumer/dependency-risk-decision.md
    acceptance:
      - 复用 src-tauri/src/main.rs → desktop::run_desktop_entry 与已有
        echo-agent-tauri 目标；移除旧 CLI bin/default-run/default tui feature
        和失效模块/exports/CI 引用，保留桌面必要 clap、GUI IPC 与构建/发布工具命令，不造第二 launcher。
      - 按最后实际消费者删除 console 专属直接依赖；root shell-words 的 MCP 用途、app-core channels 的
        shell-words/async-stream、输出层自身依赖、portable-pty、Tauri 和 xterm 保持。Cargo
        正常生成最终锁，不能手补锁或从同名包推断两层依赖均可删。
      - 最终 lock/consumer
        对原风险政策重新核准；原接受不自动继承新输入，新增、减少或身份/暴露变化按原政策闭合，保留官方原输出与自然出口，不扩大 ignore
        或接受集。正式双语文档/ADR、示例和官网退役引用由其所属 owner 同步或说明不适用；源码与宣传一致，生成/共享接缝集中合流。
  - id: qualify-retained-runtime
    summary: R5：对唯一稳定组合源码完成保留行为回归与独立资格结算。
    files:
      - echo-agent-cli/src/tauri/desktop.rs
      - echo-agent-cli/src/tauri/commands/panels.rs
      - echo-agent-cli/echo-agent-app-core/src/auto_memory/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/channel_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/state/app_state.rs
      - echo-agent-cli/echo-agent-app-core/src/terminal.rs
      - echo-agent-cli/echo-agent-app-core/src/hitl/
      - echo-agent-cli/tests/
      - echo-agent-cli/web-frontend/
    acceptance:
      - 沿原入口验证桌面启动/配置/会话恢复；隐藏再显示保持 IM/cron，真正退出等待原任务、pending
        handler、channel、PTY/SSH 的物理结算，取消等待者/关闭观察者不丢清理 owner，不把点击成功或 UI 关闭当作执行已停。
      - 保留 QQ/Feishu 配置/epoch/ready/身份/收发、GUI/IM 一次有效 HITL 回答、cron
        创建/修改/启停/手动/历史/occurrence 幂等与冷恢复控制，以及本地/SSH terminal 的
        create/write/resize/close。受控 fixture、真实渠道输入/授权、真实 SSH 与 native
        窗口分别列证据和未验范围，不以 mock 替代实际验收。
      - R1–R4 最终源冻结后独立审查一次，再按 EKO AGENTS 执行适用 full/feature/GUI/Node/合同/依赖/示例与正常
        semantic 门禁；原命令的失败须修复复验，不能因入口删除而静默删保留能力覆盖。验证按真实租约使用本树准确 target，原件出
        target、确认无 reader 后及时 cargo clean 并记磁盘；纯文档/只源准备无 target 时注明不适用。
artifact_id: plan:4a051cf8-db89-4d10-8713-8fee1ddd1da3
lifecycle: ready
---
## Notes

R0 与事实截点：本 Plan 复用已审 R0（候选 aacb2780/d3277d8e，独审 9418d066）；这些是历史 source-readiness，不能替代后继资格。APP 97502 已有真实 identity UI；当前 ee89a2c3 批正在验证 R1/R2 与 cancel 回执，已知 step0 为 1 passed，后两条尚无结果。本 Plan 不把它们标通过，也不重复实现已有 DesktopArgs/atomic 补丁。已有八路径实现的源审 b7ebb7bb 与 APP overlay 544029b6 仅作接线输入，落地须绑定实际稳定后继源与真实回执。

原依赖仍是 im-cron-gui-control。协调者可按用户已授权的文件独立并行要求准备和核准 R1/R2；R3 物理删除须等协调者接收该前置的实际适用资格、GUI/IM/cron 保留路径及 R1/R2 稳定交接，不能从本 Plan ready、UI 截图或局部绿推断已满足。无需为只源准备新增外部 main 授权屏障；main 晋级与最终交付门槛保持原合同。

R1 与 R2 的源码互不重叠，可按明确文件 owner 并行；相关测试和文档准备须同样锁定写权。R3 的专属目录/provider 删除补丁可在前置接缝冻结后按不重叠文件准备。desktop/runtime/AppState 生命周期、模块 exports/注册、manifest/lock/feature/CI、生成合同和风险绑定由同一集成人串行合流，R4 后仅对一个稳定同源快照执行 R5；独立源码工作不等待无关性能、发布或其它整批结果。
