---
title: Echo-Agent Framework Boundary, Bilingual Documentation, and Repository Hygiene
artifact: design
carrier: markdown
---

# Echo-Agent Framework Boundary, Bilingual Documentation, and Repository Hygiene

## 问题与目标

`echo-agent` 的产品定位是独立、通用、可被第三方采用的 Agent 开发框架和工具箱；
`echo-agent-cli` 是基于该框架构建的 EKO 应用。当前边界原则总体正确，但 R4 计划把
“已有第二个真实消费者”写成创建 framework API 或迁移通用机制的前置条件。这会形成
循环依赖：框架必须先有完整、可复用的能力，才可能吸引新的消费者，不能要求消费者先
存在再完善框架。

同时，`echo-agent-cli/docs` 当前包含 35 份长期文档，其中 25 份 ADR 和 10 份入门、
配置、功能、架构、运维及状态文档。文档内容本身有明确价值，但目录分类、中英文边界和
当前事实重复不够清晰。25 份 ADR 中 12 份包含中文或中英混写，13 份为英文；文档入口
没有形成与 `echo-agent/docs/{zh,en}` 一致的双语镜像。

仓库中还存在大量 `.txt` 和空目录。盘点表明多数 `.txt` 是被 Git 忽略的 EKO runtime
trace，不是源码；Git 跟踪的 `.txt` 只有网站 discovery 资产和一份被验证报告引用的审计
证据。空目录均不受 Git 跟踪，混合了缓存、运行残留和无实现的源码占位目录。按扩展名或
目录是否为空统一删除会误伤数据或失去证据。

本设计的目标是：

1. 用产品无关性和契约质量决定能力是否进入 `echo-agent`，撤销外部采用量硬门槛。
2. 保持 framework 机制与 EKO 产品策略的单一 authority，不整体搬运耦合模块。
3. 将 EKO 长期文档整理为 `zh` 编辑主源、`en` 审阅翻译的严格镜像结构。
4. 合并重复事实，但保留教程、指南、参考、架构说明、ADR 和项目状态的不同职责。
5. 按所有权和生命周期清理 `.txt`、运行产物和空目录，不按文件扩展名做删除判断。
6. 为后续实现提供可独立验证和停止的交付边界，不在本设计中预设文件级实施步骤。

## 已确认决策

| 主题 | 决策 |
| --- | --- |
| Framework 下沉门槛 | 不要求已有第二个外部消费者；由产品无关性、依赖纯度、契约完整性和独立验证能力决定。 |
| EKO 内部 crate 拆分 | 可以继续把多消费者、依赖隔离和编译收益作为是否新建 EKO contracts/domain/runtime crate 的证据。 |
| 双语维护 | `zh` 是编辑主源，`en` 是经过审阅的同步翻译；两者同等发布。 |
| 双语目录 | 所有长期 EKO 文档分别位于 `docs/zh` 和 `docs/en`，相对路径严格镜像。 |
| 文档合并 | 合并重复事实，不把所有内容合成一个大架构文件；ADR 保留决策历史。 |
| 清理原则 | 不按 `.txt` 扩展名清理；按 Git 所有权、生成关系、运行生命周期和引用关系分类。 |

## 目标行为

### Framework 能力归属

每项可能同时放在 framework 和应用的能力，都必须形成以下三部分结论：

1. **通用机制**：任何基于 `echo-agent` 构建的 Agent 产品都可能合理复用的协议、状态转换、
   并发原语、生命周期、receipt、调度、持久化 trait 或工具实现。
2. **EKO 产品策略**：依赖 EKO 本地桌面产品、workspace、DomainProfile、review/worktree、
   文件布局、surface 投影、direct-user policy 或特定资源预算的行为。
3. **适配边界**：只进行无损类型转换、EKO metadata 注入、产品 policy 选择和事件投影，
   不拥有第二套执行、DAG、mailbox、retry、terminal 或 publication authority。

能力进入 `echo-agent` 必须同时满足：

- 语义在没有 EKO 的情况下仍然完整，不需要 `AppState`、Tauri、EKO workspace 或 UI DTO。
- 公共名称和类型可以直接表达通用 Agent 概念，不通过泛化字段隐藏 EKO 决策。
- 依赖方向保持 `echo-agent <- echo-agent-cli`，framework 不引用应用类型或应用数据目录。
- 能以 framework 自有测试、示例和文档证明，不依赖启动 EKO。
- 对可选能力使用现有 crate/feature 边界，默认消费者不承担无关产品依赖。
- 迁移后只有一个真实 authority；EKO 主路径切换到 framework 后，被替代机制必须删除。

以下条件不能作为拒绝下沉的理由：

- 当前只有 EKO 一个消费者。
- 尚无外部开发者调用该 API。
- framework 中暂时没有相邻模块。
- 新能力需要先由 EKO 主路径验证。

外部采用量仍是 API 稳定程度、文档优先级和拆包收益的证据，但不是通用能力存在的资格门槛。

### 当前候选边界

| 当前区域 | 目标判断 |
| --- | --- |
| `AgentPool` | 不整体迁移。Agent 实例复用、lease、容量、idle eviction 和关闭结算是 framework kernel 候选；`EkoConfig`、workspace generation、tool visibility、模型与 plugin publication 留在 EKO wrapper。 |
| `AgentRouter` | 不整体迁移。通用 address/route、durable delivery receipt 和 inbox lifecycle 可评估为 framework 能力；EKO `WorkspaceId`、文件根、retirement policy 和 surface projection 留在应用。迁移不得创建 framework turn mailbox 之外的第二个输入生命周期。 |
| `ChatEventLog` | 不整体迁移。framework 继续拥有 segmented journal、checkpoint 和 replay 原语；EKO chat event payload、retention pin、conversation identity 和 UI/channel projection 留在应用。 |
| `PluginRuntimeService` | framework preparation、校验和不可变 generation 保持 framework authority；EKO target publication、偏好、workspace generation 和现有/未来 Agent 投影留在应用。 |
| `ExtensionControlService` | 继续留在应用。它组合 Skill、Hook、MCP、LSP、Browser 和 Plugin 的 EKO mutation policy；只把其中可独立证明的通用协议原语下沉。 |
| Task DAG、retry、cancel、claim、revision | 已由 framework 拥有，不建立新迁移项目；EKO 保留文件事实、review/worktree 和 surface 投影。 |
| `AppState`、workspace registry、DomainProfile、research/analysis/browser policy | 明确保留应用层。 |

候选不等于预先批准迁移。后续设计或计划必须按 symbol、trait、状态和调用路径证明具体边界，
并先搜索 framework 与 app-core 是否已有同语义实现。

### Framework 迁移数据流

```text
现有 EKO 主路径
        |
        v
识别产品无关机制 + 全仓重复性审计
        |
        v
framework contract / implementation / test / example / docs
        |
        v
EKO 薄 adapter 切换一条真实生产路径
        |
        v
验证 wire、持久化和 surface 行为不变
        |
        v
删除被替代的 app-core 机制
```

任何阶段都不得长期保留两套 authority。若无法在当前交付结果中切换真实路径并明确删除
目标，该候选留在应用层，不以“先抽象以后迁移”的方式新增空壳 API。

## 双语文档信息架构

### 目录结构

`echo-agent-cli/docs` 的长期目标结构如下：

```text
docs/
  README.md                     # 仅语言入口和维护规则，不承载产品行为事实
  zh/
    README.md                   # 中文导航
    getting-started.md          # 教程
    features.md                 # 产品能力参考
    configuration.md            # 配置参考和操作指南
    project-status.md           # EKO 应用当前状态
    architecture/
      overview.md               # framework/app 边界与总数据流
      runtime.md                # application lifecycle、TaskRuntime、Agent control
      persistence.md            # Store、Journal、Checkpoint、Trace
      providers.md              # Provider、模型协议和动态配置
    operations/
      skill-sync.md             # Skill 同步与运维行为
    adr/
      0001-*.md
      ...
  en/
    # 与 zh 完全相同的相对路径和 ADR 编号
```

根 `docs/README.md` 只允许包含语言选择、文档类型说明和维护门禁，不成为第三份行为事实源。
中英文页面使用相同相对路径作为配对 identity，不引入第二套页面编号或独立导航体系。

### 文档职责

| 文档类型 | 负责内容 | 不负责内容 |
| --- | --- | --- |
| Tutorial | 从安装到完成首个真实任务的学习路径 | 完整 API、全部配置和历史取舍 |
| How-to / Operations | 完成配置、同步、诊断等具体目标 | 架构历史和里程碑流水账 |
| Reference | 功能、配置、命令、wire 和文件布局的可查事实 | 为什么选择该设计 |
| Architecture / Explanation | 当前 authority、数据流、不变量和模块边界 | 被替代方案的完整历史 |
| ADR | 背景、候选、决定、理由、影响和 superseded 关系 | 当前产品全景和操作教程 |
| Project status | 当前阶段、SHA、剩余门禁和事实源入口 | 长期架构说明和历史实施日志 |

这一区分采用 Diataxis 的 tutorial、how-to、reference、explanation 四类认知模型，同时保留
软件架构项目需要的 ADR 和 project status。

### 合并与迁移规则

- `architecture.md` 的当前总览进入 `architecture/overview.md`。
- `persistence.md` 进入 `architecture/persistence.md`。
- `architecture/providers.md` 保持专题职责并迁入双语镜像。
- `architecture/runtime-task-service.md` 中的当前行为合入 `architecture/runtime.md`；候选方案、
  选择理由和后果只保留在对应 ADR，不继续维护一份平行的“已采纳决策页”。
- `getting-started.md`、`configuration.md`、`features.md` 和 `skill-sync.md` 不并入架构总览。
- `MASTER-PLAN.md` 改为语言目录内的 `project-status.md`，只保存当前状态和证据入口。
- ADR 编号、状态和相对路径在 `zh`、`en` 中一致。历史 ADR 不因翻译而重新编号。
- 超长外部能力目录、竞品快照和一次性调研不作为 ADR 长期正文。保留有约束力的最终决定，
  原始研究材料迁到顶层跨仓阶段材料或删除过时副本。
- 移动后更新仓库 README、顶层事实源、website manifest、website source links 和所有内部链接；
  不保留失效路径作为长期兼容层。

### 翻译与 parity gate

`zh` 是编辑主源，表示新事实先在中文页面完成代码核对和审阅；`en` 必须在同一交付结果中
完成语义等价翻译。两者对外同等正式，不允许英文长期落后或标记为非权威摘要。

Parity gate 至少验证：

1. `docs/zh` 与 `docs/en` 的 Markdown 相对路径集合完全相同。
2. ADR 编号、状态、标题主题和 superseded 指向一一对应。
3. 中文主源发生行为变化时，同一变更必须包含对应英文页面；纯拼写或链接修正可以通过
   明确的等价检查豁免内容变化，但不能缺失配对文件。
4. 两个语言目录分别通过 Markdown link check，所有源码路径和跨仓链接可解析。
5. Website 同步清单同时记录完成审阅的 CLI revision，不从只更新一个语言的 checkout 发布。
6. 页面自然语言只使用所属目录语言；代码标识、协议字段、命令、引用标题和专有名称除外。

Parity gate 复用 Git diff、相对路径和现有 Markdown/website 检查能力，不引入新的运行时状态库。

## `.txt` 与空目录清理设计

### 分类原则

文件扩展名不是删除依据。每个候选项必须归入以下一种所有权：

| 类别 | 当前示例 | 行为 |
| --- | --- | --- |
| 正式源码/发布资产 | website `robots.txt`、`llms.txt`、`llms-full.txt` | 保留并继续由生成/静态检查验证。 |
| 被文档引用的工程证据 | `generated-dirty-metadata.txt` | 随引用它的完整审计材料保留或整体归档，不单独删除。 |
| EKO 用户或任务运行数据 | `.eko/runtime/**/artifacts/traces/*.txt` | 受数据生命周期保护；不得按扩展名删除或留下断裂引用。 |
| 测试/soak 运行残留 | 已完成且不再作为验收证据的专用 run root | 按完整 run/root 清理，先证明无活动 owner、无保留证据和无用户历史价值。 |
| 构建/工具缓存 | `target/test-tmp`、`.playwright-mcp`、`.worktrees`、`.claude/worktrees` | 无活动进程且可重建时清理；必要目录由工具重建并应被正确 ignore。 |
| 无实现源码占位目录 | 空的 `src/handoff`、`src/notebook`、`src/bin`、frontend component 目录等 | 全仓无引用、无生成器要求时删除；未来功能随首个真实文件重新创建。 |

### 运行数据保护

`.eko` 是 EKO 的本地数据根，可能包含仍可观察的 TaskRun、conversation、artifact、journal
或验收证据。清理必须以完整 scope 为单位，并满足：

- scope 已 terminal，且没有 live handle、workspace lease、recovery debt 或后台 owner。
- journal、projection 和 artifact 引用关系已经检查，不单删某个 trace 文件。
- soak/release ledger 明确不再引用该 scope，或证据已经迁到批准的长期位置。
- 用户数据与自动测试数据可以从 root、metadata 或创建方式可靠区分；无法区分时默认保留。
- 删除目标使用已解析的明确路径，不使用宽泛 glob、未解析环境变量或递归 workspace 根。

本轮盘点到的 runtime `.txt` 数量多但体积很小，清理主要解决仓库卫生和生命周期问题，
不是磁盘容量优化项目。

### 空目录

Git 不跟踪空目录。除非构建工具、平台打包器或测试 fixture 明确要求目录预先存在，否则不使用
`.gitkeep` 保留占位。清理后：

- 运行时和工具缓存目录通过 ignore 规则和创建逻辑管理。
- 源码目录只在含真实模块或资产时存在。
- 文档目录只在含当前事实源时存在。
- 不把删除空目录宣称为代码功能变更或架构迁移。

## 范围

本设计覆盖：

- `echo-agent` 与 `echo-agent-cli` 的 framework/application 归属判定规则。
- `AgentPool`、`AgentRouter`、`ChatEventLog`、Plugin/Extension 等候选区域的重新评估边界。
- `echo-agent-cli/docs` 的双语结构、内容归类、ADR 整理和 parity gate。
- README、顶层跨仓事实源与 `echo-website` 的链接和投影同步。
- 三个子仓库及 superproject 中 `.txt`、空目录、缓存和 runtime 数据的分类清理合同。

## 非目标

- 不批准把任何 app-core 模块整体搬入 framework。
- 不在本设计中决定每个 symbol 的最终 crate 和公开 API 形状。
- 不重写 EKO wire、serde、TS binding、文件布局或 surface 行为。
- 不把 EKO workspace、review/worktree、DomainProfile 或 UI projection 下沉 framework。
- 不要求删除 framework 中当前无 EKO 调用但合理公开的能力。
- 不把历史 ADR 合并为一个不可追溯的大文档。
- 不自动翻译或发布未经人工审阅的英文文档。
- 不在文档迁移过程中顺便删除 `.eko` 用户数据或 release evidence。
- 不把清理扩展为 Git 历史重写、push、release 或远端资源删除。

## 系统边界

```text
echo-agent
  product-neutral runtime / Agent / Tool / Task / Store / receipt primitives
        |
        v
echo-agent-cli / echo-agent-app-core
  EKO composition / workspace / policy / persistence / projections
        |
        v
CLI / TUI / Tauri / Channel / Background

echo-agent-cli docs/zh  --reviewed translation-->  docs/en
          |                                      |
          +-------------- parity gate -----------+
                             |
                             v
                    echo-website projection
```

顶层 `lp-agent/docs` 继续只保存跨仓规则、阶段设计、计划、审计和验收证据；正式 framework
文档归 `echo-agent/docs`，正式 EKO 产品文档归 `echo-agent-cli/docs/{zh,en}`。

## 异常与边界场景

### 候选机制仍依赖 EKO

若通用候选在抽取后仍需要 `EkoConfig`、workspace generation、Tauri DTO、产品文件布局或
surface event，它不是完整 framework primitive。处理方式是缩小候选，仅提取可独立成立的
kernel；无法形成有意义 kernel 时整体留在应用。

### Framework 已有相同能力

若全仓搜索发现 framework 已有同语义 trait、service 或 state machine，禁止新建平行 API。
应扩展现有能力或删除 EKO 重复实现，并保持现有 authority 的序列、receipt 和错误语义。

### 翻译落后

中文主源行为变化但英文未同步时，parity gate 失败，文档变更不能合入或发布。不能以
“稍后翻译”把英文长期降级为非正式文档。

### 双语内容语义冲突

代码和测试是行为证据，中文主源是编辑起点，不自动覆盖已经发现更准确的英文内容。
发生冲突时先核对代码和 ADR，修正事实错误，再同步两个语言版本；不能机械把错误翻译两遍。

### ADR 与当前架构冲突

ADR 记录当时决策，architecture 记录当前状态。旧 ADR 被替代时保留原文并标记 superseded，
由新 ADR 说明替代关系；不能修改历史 ADR 使其看起来从未采用旧方案。

### 运行数据无法确认归属

无法判断 `.eko` scope 是用户任务、测试运行还是保留验收证据时默认保留，并将其列为需要
人工裁决的清理项。空间压力不是绕过归属检查的理由。

### 清理时仍有活动 owner

发现活动进程、文件 handle、workspace lease、background owner 或恢复任务时跳过该 scope。
清理失败不得留下半个 run；可重试清理由明确 scope 重新开始。

### Website 生成资产扩展名为 `.txt`

生成资产继续保留。其正确性由生成器、site check 和 clean-checkout projection 判断，不能因
扩展名与 runtime trace 相同而删除。

## 关键取舍

### 不使用外部采用量作为 framework 资格门槛

收益是 framework 可以主动形成完整能力并吸引使用者；代价是公共 API 设计必须承担更严格的
独立示例、feature、文档和未来演进审查。该代价符合通用框架定位。

### 不整体迁移当前大模块

当前 `AgentPool`、router、plugin 和 extension 类型同时组合通用机制与 EKO policy。整体迁移
会污染 framework。先确定 kernel 和 wrapper 边界，能够保持依赖方向和单一 authority。

### 双语镜像而不是一个混合目录

严格镜像让读者、website 和自动检查可以稳定定位同一主题。代价是每个行为变更需要同步翻译，
由同一交付结果和 parity gate 控制。

### 不把所有文档合成单文件

单文件会混淆学习、操作、查询、架构解释和决策历史。按用途组织能降低单页复杂度，并允许
不同变更只触及对应事实源。

### 不按扩展名清理

`.txt` 同时承载发布资产、审计证据和 runtime artifact。按所有权与生命周期分类比扩展名规则
更安全，也能避免数据丢失和引用断裂。

## 复用与实现约束

- 复用 `echo-agent` 已有 Agent turn mailbox、tracked steer receipt、Subagent control、Task DAG、
  journal/checkpoint、plugin preparation、ToolManager 和 feature 体系，不创建替代 authority。
- 复用 `echo-agent/docs/{zh,en}` 已验证的相对路径镜像作为 CLI 文档结构先例。
- 复用现有 Markdown link check、website docs sync manifest、site check、build 和 clean-checkout
  验证，不引入新的文档站生成器。
- 复用 Git 路径集合、diff 和 ignore 信息实现 parity/cleanup 盘点；不为文档维护引入数据库。
- 复用 Cargo workspace 与 feature gate 验证 framework 抽取后的独立编译。
- 任何新 framework public API 都必须具备 rustdoc、可执行 example 或 contract test，并遵守
  Rust API Guidelines 的命名、互操作、文档、可预测性和未来演进约束。
- OpenAI Agents SDK 展示了轻量、provider-agnostic 核心与可选能力的组合方式；本项目采用
  相同原则，但不复制其 Python API 或产品安全模型。
- Docusaurus i18n 和 Diataxis 只作为语言与内容分类参考；实际目录遵循本仓库已确认的
  `docs/{zh,en}` 约定。

## 候选交付结果与依赖

### Framework 归属修订

形成新的长期边界 ADR、symbol-level 审计和明确 disposition，删除 R4 中“第二个消费者才评估
framework 迁移”的硬门槛。该结果独立可审阅，不要求同时搬迁候选机制。

### 首批通用 kernel 收敛

以归属修订为前置，只对通过门槛的 Agent pool/router 候选形成 framework contract，并在同一
交付链中切换 EKO 主路径和删除重复机制。每个 kernel 可以独立合并和停止。

### EKO 双语文档重组

将长期文档迁入严格镜像的 `zh`、`en` 结构，合并重复当前事实，整理 ADR 与外部调研材料，
更新所有仓库链接。该结果不依赖 framework kernel 实施，但内容必须反映最终已合入的边界事实。

### Parity 与 website 发布门禁

以双语结构为前置，形成路径集合、配对变更、双语链接和 website revision 检查。门禁通过后，
website 才同步新的 EKO 文档路径。

### Repository hygiene 清理

以只读分类清单为输入，分别处理可重建缓存、无引用空目录、历史工程证据和受保护 runtime
scope。该结果不依赖文档或 framework 迁移，但不得删除仍被它们引用的材料。

## 验收标准

### Framework 边界

- 所有长期规则不再把“已有第二个 framework 消费者”作为创建通用 API 的必要条件。
- 每个迁移候选都有“通用机制 / EKO 产品策略 / 适配边界”结论和全仓重复性证据。
- framework 不依赖 EKO 类型、目录、DTO 或产品配置。
- 新通用能力能在 `echo-agent` 独立编译、测试、示例运行和文档阅读。
- EKO 所有 surface 继续功能对等，wire、文件和持久化合同无意外变化。
- 任一迁移完成时只有一个 authority，被替代 app-core 实现已经删除。

### 双语文档

- `docs/zh` 与 `docs/en` 的长期 Markdown 相对路径集合完全相同。
- 所有页面分别使用所属语言，ADR 编号、状态与替代关系匹配。
- 当前架构事实只有一个主题 owner，不在 architecture、ADR 和 project status 中复制整段合同。
- 根 `docs/README.md` 只做语言入口，不形成第三份产品事实。
- 双语 Markdown links、源码路径、跨仓链接和 website source links 全部通过检查。
- Website manifest 指向完成双语审阅的 CLI commit，discovery、site check、build 和 tests 通过。

### 清理

- Git 跟踪的 `.txt` 均有生成器、发布用途或引用证据；不存在无主 tracked `.txt`。
- `.eko` runtime 数据没有按扩展名或模糊 glob 删除；所有删除都绑定完整、已确认可清理的 scope。
- 无引用、无工具要求的空源码占位目录已消失；缓存目录由 ignore 和创建逻辑管理。
- 清理不改变源码行为、不破坏验证证据、不留下断裂链接或 artifact 引用。
- 三个子仓库和 superproject 不包含与本任务无关的工作区变化。

## 行业参考

- OpenAI Agents SDK: <https://github.com/openai/openai-agents-python>
- Rust API Guidelines: <https://rust-lang.github.io/api-guidelines/>
- Docusaurus i18n tutorial: <https://docusaurus.io/docs/i18n/tutorial>
- Diataxis documentation framework: <https://diataxis.fr/>
