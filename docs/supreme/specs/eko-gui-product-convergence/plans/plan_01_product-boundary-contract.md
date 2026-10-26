---
schema_version: 4
slug: eko-gui-product-convergence/product-boundary-contract
outcome:
  summary: 在各自正式规则与 ADR 中固化 EKO 的 GUI 主入口、IM/定时任务保留边界及 A2A/LSP/TUI/CLI
    退役合同，提供精确可恢复的跨仓能力处置清单。
  acceptance:
    - 顶层与所属子仓的规则、正式 ADR 及产品说明一致表达已确认方向；明确保留 GUI、IM、cron、终端、MCP、Subagent、SDK Host
      与文件持久化，当前仍存在的入口和协议标明尚待实际退役。
    - 退役处置清单覆盖框架/EKO 的生产入口与共享 owner、SDK A2A/LSP accepted 身份和路由、配置/插件/文档/示例/CI
      消费者；每一类均能定位到实际源码或正式合同，后续删除无需依赖旧 worktree 名称猜测。
    - 五仓保留分支/tag、顶层 gitlink、SDK 自身 framework pin、Exa 独立提交和已有差异的边界可核对；本 Outcome
      的最终差异仅含必要规则、ADR、正式文档及其官网投影，运行代码、依赖锁、生成合同和保留引用未改变。
    - 完成所属仓适用的文档/产物验证与独立只读复审；交付时按仓库规则核验提交、main 与适用 CI。只能报告本 Outcome
      合同已固化，不能声称运行入口或 A2A/LSP 已删除。
out_of_scope:
  - 不删除 A2A/LSP/TUI/CLI/JSONL，不切换运行入口，不改 Agent/SDK/EKO 的 Rust/TypeScript
    运行逻辑、feature、manifest、依赖锁或 SDK 生成合同；官网既有 TSX 中的静态产品文案同步属于本 Outcome。
  - 不实施 GUI 美化、IM/cron 新接线、HITL、任务列表、SSH、工具机制或插件运行时；它们由 delivery map 的独立结果承接。
  - 不推送/合并 Exa，不新增未指定 IM 平台，不引入 SQLite，不丢弃旧工作区、stash、保留分支/tag 或历史证据。
  - 不执行包发布、网站部署或未授权的 Issue 关闭；不重新实施已经完成的旧补丁修复。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 1. 文档定位与已确认目标
    digest: sha256:0f17d2cbbec5131fac3d3c4b955710ddcf09a84c2872177492af6425a4a7ac4f
  - ref: design.md § 2. 已固定基线与当前事实
    digest: sha256:280479f678d0b5f130eb4852b8ff24c0b120d8dd64931611fc40a8113665cfa4
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 5. 入口收敛与 A2A/LSP 退役
    digest: sha256:c9c8f28566cfe1936dad26af7d33c6ec5a8fa0a0b9af1215b1ae3d8e7cffc09d
  - ref: design.md § 工程收尾
    digest: sha256:41bb60277b81fc0461da2509637a85afbeeaab2ac76bf83131262c66d289d2cb
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#product-boundary-contract
todos:
  - id: align-product-authorities
    summary: 将 GUI 主入口及 IM/cron 保留方向写入一致的项目规则与所属产品 ADR，明确目标状态与当前行为的区别。
    files:
      - AGENTS.md
      - echo-agent/AGENTS.md
      - echo-agent/docs/adr/
      - echo-agent-cli/AGENTS.md
      - echo-agent-cli/docs/en/adr/
      - echo-agent-cli/docs/zh/adr/
      - echo-agent-cli/docs/en/architecture/overview.md
      - echo-agent-cli/docs/zh/architecture/overview.md
      - echo-agent-sdk/AGENTS.md
      - echo-agent-sdk/docs/adr/
    acceptance:
      - 去掉要求继续扩张 TUI/CLI 等入口的旧目标约束，同时保留本地个人产品、交互操作与 Agent 自动权限的区别、框架独立复用和 EKO 不用
        SQLite 的边界。
      - ADR 说明退役原因、实际影响、保留能力和恢复起点；文档未将未来删除误写为当前已不存在。涉及子仓的规则与顶层对应条款同步，正式事实源仍在子仓。
      - 正式合同区分新增 API 的框架先交付与撤销 API 的消费者先解除依赖，明确 CLI 跟踪 framework main 与 SDK 独立
        pin 的合流条件，避免主线中间态不可构建。
  - id: bind-exact-retirement-surface
    summary: 从实际源码和已接受合同建立可审阅的退役处置清单，标明共享实现的去向、SDK 精确退役范围及各消费者保留义务。
    files:
      - echo-agent/docs/adr/
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/adr/
      - echo-agent-cli/docs/zh/adr/
      - echo-agent-sdk/docs/adr/
      - echo-agent-sdk/docs/sdk/governance/
    acceptance:
      - 对 A2A/LSP 核对 feature、公开实现、注册、配置、插件组件、SDK accepted source/kind
        与路由、语言包装、示例、文档及条件矩阵；不把完整 Rust inventory 当成三语言合同，也不把应用未调用当作删除通用 API 的依据。
      - 对 TUI/REPL/CLI/JSONL 核对启动、参数、HITL provider、呈现、补全及其消费者；指出
        src/cli/channels、桌面启动和共享 task/terminal 逻辑的真实 owner，明确 IM、cron、终端和 SDK
        Host 不被连带退役。
  - id: verify-and-publish-boundary-contract
    summary: 完成合同文档与官网投影的一致性审阅，并证明完整能力恢复锚点和现有运行输入在本 Outcome 中保持。
    files:
      - AGENTS.md
      - echo-agent/docs/
      - echo-agent-cli/docs/en/
      - echo-agent-cli/docs/zh/
      - echo-agent-sdk/docs/
      - echo-website/src/components/HomePage.tsx
      - echo-website/src/docs/content/
      - echo-website/docs-sync-manifest.json
      - echo-website/public/llms-full.txt
    acceptance:
      - 从实际保存的五仓分支/tag核对固定提交和顶层 gitlink，单独核对 SDK 原 pin 与 Exa 本机提交；旧检出和 dirty
        worktree 只作为保护对象，不作为已验收基线，也不从本 Outcome 发起清理。
      - 正式文档与官网按现有源同步机制对齐，示例现状和是否需要改动有明确处置；完成适用的双语/文档/格式/生成内容检查与跨仓差异必要性复审，保留代码和合同输入不变的对比证据。
      - 子仓按独立仓库交付，顶层仅管理跨仓材料、规则和已推送的 gitlink；执行任何提交、推送或合并前核对该动作的会话授权，不把写 Plan
        视为实施或发布授权。
artifact_id: plan:323eb7f6-d95b-48f0-a3d7-c29dbf4823df
lifecycle: ready
---

## Notes

当前只执行“固化产品与退役边界”这一结果。三个 Todo 按同一结果结算，文档贡献可以按以下范围并行，验收不拆成多份独立通过：

| 贡献范围     | 独占写入                                                                             | 接缝与汇总                                                       |
| ------------ | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------- |
| 框架合同     | echo-agent/AGENTS.md 与框架 docs/adr                                                 | 精确 API/feature/消费者处置提交给集成负责人，不删除实现          |
| EKO 产品合同 | echo-agent-cli/AGENTS.md、双语 docs/en/adr 与 docs/zh/adr、overview/runtime 正式说明 | 明确 GUI/IM/cron 保留和桌面对 CLI 的真实共享引用，不修改运行代码 |
| SDK 合同     | echo-agent-sdk/AGENTS.md、docs/adr、docs/sdk/governance                              | 精确列出 accepted 两类能力及保留义务，不运行生成器或改 Host pin  |
| 跨仓合流     | 顶层 AGENTS.md、官网正式源投影/manifest/discovery                                    | 负责人收齐三方事实后统一写入，核对恢复 tag/gitlink 与不变输入    |

新增 API 的框架先行顺序与撤销 API 的消费者去依赖顺序要在正式合同中区分：CLI CI 跟踪框架 main，不能让框架删除先破坏应用 main。SDK 的独立 pin 也不能被顶层指针替代。这个接缝必须在本 Outcome 固化，不能留给实现时临时猜测。

实施从精确已验收基线和届时 main 增量核对开始。当前规划 checkout 只有顶层 docs，不把未初始化的子模块或 primary 旧 HEAD 当作可编辑的四仓实现；由 Build 在获得执行授权后选择正确隔离工作区。各仓正式 ADR 编号以实际目录查重确定，不在本 Plan 预占编号。
