---
schema_version: 4
slug: eko-gui-product-convergence/performance-baseline
outcome:
  summary: 提供会话/任务查询、文件冷恢复、记忆写入、GUI 列表和工具 schema/结果开销的可复现基准及同源比较方法。
  acceptance:
    - 基于冻结的Framework/EKO实际源码、path依赖、locks/features/release
      profile和纯合成dataset，交付可执行的重复测量入口、结果与覆盖说明；会话list/search、TaskRun
      list/filter、完整journal冷恢复、完整memory写、真实GUI任务组件与实际工具成本均有具名数据，缺项不得写为已测或通过。
    - 每个场景报告规模单位、setup/warmup/measurement边界、原始样本数与耗时分布、结果/身份/错误校验及CPU/RSS/I/O证据和限制。区分进程cold、OS
      page
      cache、热authority、有效/缺失/损坏checkpoint、jsdom/browser/native；必要原生证据不足时保留对应未完成项，不以脚本或报告存在宣告完整验收。
    - 比较双方复用同一冻结字节快照和受控主机条件，逐项解释后继源码、锁、工具集合、durability与GUI
      build变化；不将不同负载或策略作优化收益，不按源码复杂度或旧gate结果声明回归，不新增产品SLA或性能阈值。
    - 新测量入口与fixture通过适用的直接和仓库工程检查、独立复审；既有正确性/资源/预算门槛保持，文档/examples/官网同步或不适用有具名结论。按协调租约错峰执行并在每个Cargo批次保存原件后清理准确自有target，完成受限条件及重跑说明；性能优化另归后继Outcome。
out_of_scope:
  - 不实施存储/索引/缓存/渲染或工具曝光优化，不改变Task/Conversation/Memory/产物的唯一权威，不新增Store、SQLite、公开benchmark专用API、批次/收据schema或产品状态机。
  - 不执行A2A/LSP或独立入口退役、GUI/HITL/任务弹窗业务重构、SSH/新增IM渠道、插件运行时改造，不推送/合并Exa、不发布包或部署官网。
  - 不以无效超上限Todo、不同工具集合、不同durability、InMemoryStore/NullChangeLog或未await的派发代替真实产品负载；不启动LLM/真实外部服务、不运行sudo
    purge、不干扰其它测量/构建或清理它们的缓存。
  - 不重做已交付tool-inventory审计或把SDK旧Host
    pin当本次framework构建身份；不借历史性能快照承诺当前main/新锁/原生平台验收。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 工具的统一语义
    digest: sha256:5b0b172fe617df162c9e94a499e31811b770495ace98a22d550885deb9654a1f
  - ref: design.md § 测量与持续回归
    digest: sha256:9ce2b5788eed53f142334b396eaf369ff071833ebfdd0f9851844d40e1954392
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#performance-baseline
todos:
  - id: freeze-reproducible-measurement-inputs
    summary: 固定真实源码与同源合成数据、合法负载和计时边界，形成可重复消费的测量输入。
    files:
      - echo-agent/benches/journal_bench.rs
      - echo-agent/echo-orchestration/src/planning/validator.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_runtime.rs
      - docs/supreme/specs/eko-gui-product-convergence/audits/
    acceptance:
      - 以F c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa、E
        db5010ed888cf50f2400fbba31075ab63c5626e9建立独立干净基线，验证实际path依赖和lock；后继dirty/合流另冻结精确revision与受控文件/patch身份，保护现有LSP消费退役差异，不用未知工作树测量。
      - 定义并冻结conversation/message、run/event、task/todo、memory/store/history和tool/artifact/page各自单位、规模档位、UTF-8内容、状态比例与query命中率；现有10k/100k明确仅指对应事件/历史记录。GUI数据经过实际TaskRevisionService/PlanValidator接受，遵守当前单图100
        tasks/depth10约束，ID/revision/状态一致，不将超产品上限的setState数据当真实负载。
      - public
        seed的now/UUID保留：只生成一次真实合法快照并保存相对路径/大小/SHA与dataset身份，两侧复用同一字节输入；生成/构建/复制在计时外。冻结硬件/OS/文件系统、features/profile、同机负载、各场景样本与cold/hot定义，失败/正确性对照及原始观测格式；不凭固定seed宣称文件摘要相同。
      - 本轮冻结默认协议：conversation与run集合各10/100/1000对象，查询命中/未命中与分页分列；会话固定每对象10条含中文/emoji的消息；journal另用10k/100k事件历史，不能称10k/100k
        runs。真实GUI单图1/10/50/100个合法任务、depth<=10，Subagent/event历史另计。memory
        entry集合10/100/1000，audit/operation历史0/1k/10k单独轴，先核对真实格式/guard。避免全部轴笛卡尔积：每次只改变一个轴，保存具体固定对照；任何资源原因缩减须在测量前形成带理由的新协议身份并如实保留原档位未测，不能静默改负载。
      - 预注册每场景5次warmup+30个热样本，完整进程cold10个独立进程样本、完整memory成功写20个样本且每manager<=50次；慢场景/工具合法边界可用明确更小独立诊断组，不混入默认分布。报告全部原始N/min/median/max与有支撑的经验分位数，N<100不报p99，统计区间方法和计时器分辨率明确；warmup、fresh-process输入复制、启动及settle边界分别标注，超时/拒绝不能算快成功。
  - id: measure-persisted-queries-and-complete-recovery
    summary: 沿既有会话与TaskRuntime权威测量查询、筛选和完整文件冷恢复，验证结果与恢复事实。
    files:
      - echo-agent/echo-state/src/memory/file_conversation.rs
      - echo-agent/benches/journal_bench.rs
      - echo-agent/echo-state/src/journal/file.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/run_authority.rs
    acceptance:
      - 复用现有Criterion journal primitives、ignored
        release性能夹具及conversation正确性fixture，测真实list/search与list_runs_in/list_runs_for_conversation/get_run_state；命中/未命中、状态/limit/offset和Unicode控制都有可校验身份与计数。区分底层空statuses直接空返回与GUI空筛选转换，不让无操作或错误吞掉变成快查询。
      - 完整进程cold包括公开store构造和首次触发RunAuthority/FileEventJournal验证、checkpoint/tail恢复及投影结算的调用；分别测有效checkpoint、指定尾长、缺失/损坏checkpoint与必要boot
        reconciliation。现有已open且Flush的reducer
        Criterion和仅根authority构造单独标原语范围，不能冒充EKO SyncData完整冷恢复。
      - 同实例热查询与fresh process结果分列，记录LRU/authority复用和OS page cache条件，不sudo
        purge或以换目录伪造磁盘cold；逐样本校验完整run/status/sequence/查询与恢复事实和显式错误。保留已有性能/正确性断言及超限反例，不因新测量抬旧阈值。
  - id: measure-complete-memory-write-settlement
    summary: 测量真实分层记忆完整持久化与应用投影结算，保持原安全与审计路径。
    files:
      - echo-agent/src/evolution/runtime_integration.rs
      - echo-agent/src/evolution/layer.rs
      - echo-agent/src/evolution/mutation.rs
      - echo-agent/echo-state/src/memory/store.rs
      - echo-agent-cli/echo-agent-app-core/src/evolution/review_integration.rs
      - echo-agent-cli/src/tauri/commands/memory.rs
    acceptance:
      - 使用真实FileStore、JsonlChangeLog、MemoryLayerManager和operation
        journal/observer，完整await
        write_memory的prepare/apply/audit/settle/observer/promotion-consideration；应用范围另覆盖原ReviewGeneration的settle_hot_memory_projection。核心写、应用结算和单独Store原语成本分列，不把InMemoryStore、NullChangeLog、无observer或未await派发当完整写入。
      - entry/store字节与operation/audit
        history规模分列，每次写从同一冻结起点测量；断言内容/身份、Draft与结算事实，区分None
        promotion与未写。保持默认每manager50次guard，不无限Criterion迭代并把速率拒绝算成功；成功写、显式拒绝和重开/reconcile样本分列，不修改guard或公开私有写入口。
  - id: measure-valid-task-render-and-tool-costs
    summary: 测量合法TaskRuntime投影的真实GUI组件与既有ToolManager/产物读取成本，界定渲染与完整结果证据。
    files:
      - echo-agent-cli/web-frontend/src/components/task/
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts
      - echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts
      - echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs
      - echo-agent-cli/echo-agent-app-core/src/tool_execution.rs
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent/echo-tools/src/files/artifact.rs
      - docs/supreme/specs/eko-gui-product-convergence/audits/tool-inventory.md
    acceptance:
      - 沿RTL/Vitest与真实TaskRuntimePanel/有型Zustand数据测mount、合法状态/单行/整批更新、DOM数量与正确呈现；区分当前active
        run任务数、run集合和长subagent/event历史。固定合法任务图、theme/viewport/build与mock边界，不把现有纯函数测试或无效10k
        Todo压力值称真实产品渲染。
      - jsdom/React
        Profiler只报告mount/reconciliation；browser引擎与原生Tauri/WebView的layout/paint/交互/RSS分别取得证据或保留缺口，不能互相替代。必要原生证据未具备时该场景保持未完成，报告不能声称原生性能已通过。
      - 复用已交付inventory及实际schema_stats_for/visibility/request与spill/artifact/page
        reader，测registered/eligible/visible和实际集合、UTF-8 bytes/estimated
        tokens、preview/payload/artifact/SHA/分页恢复及GUI wire
        bytes。既有budget门槛不变，估算token不冒充provider实际token，模型选择成功率或真实服务未测如实标明；工具集合/durability变化逐项解释，不作为存储优化收益。
      - GUI先复用既有RTL/Vitest建立真实组件观测，再复用本Goal已隔离的Tauri/WKWebView观测通道取同源native样本；不新装自动化/终端依赖，不启动用户主实例，不用浏览器替代native。工具payload沿现有budget和artifact
        reader实际语义选小/近阈值/超阈值三个分列组，以现有公开默认值冻结具体字节，按原spill、clamp、truncation或显式拒绝分别记录，不新增容量或拒绝策略；非法参数、越界引用和损坏仅在原逻辑会拒绝时作拒绝反例，不将拒绝当完整结果。
  - id: qualify-and-report-reproducible-baseline
    summary: 汇总可重复基线、同源比较与覆盖限制，完成验证、交接和批次资源结算。
    files:
      - docs/supreme/specs/eko-gui-product-convergence/audits/
      - echo-agent/benches/
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/web-frontend/src/components/task/
      - echo-agent-cli/docs/en/architecture/persistence.md
      - echo-agent-cli/docs/zh/architecture/persistence.md
    acceptance:
      - 结果报告逐项绑定source/locks/features/profile、实际path
        dependency、硬件/系统/负载与同源dataset
        manifest，给出真实命令/出口、原始N/样本/分布、正确性/错误校验、CPU/RSS/I/O或明确不可取得的指标。区分进程CPU/峰值与单操作值、逻辑文件bytes与物理I/O、Criterion区间与业务分位数，小样本不足时不编造精度；保留真实失败和未覆盖场景，不从受限本机外推生产。
      - 同源比较入口可重跑；新harness/fixture按实际改动完成格式/Lint/类型/测试及必要编译与独审，原性能/预算测试不改期待或抬阈值。仅提交必要长期结论、精简fixture与可执行入口，raw
        logs/样本/制品放ignored或CI
        Artifact；正式双语说明、examples可执行链和官网同步或不适用均有具名结论，官网共享接缝由协调者合流。
      - 测量与SDK full matrix、framework
        focused及归档/复制I/O错峰，先取得协调租约再编译/测量；每任务独立target、jobs=1/debug0，批次前后记df/du，低于5GiB不启新重任务。每个Cargo批次实际结束后先保存日志/自然出口/必需制品到target外，确认自有target无cargo/rustc/Host读取再cargo
        clean；不抢占或清其它任务产物。
artifact_id: plan:113e3749-4c2a-4e11-80d5-317fee896c89
lifecycle: ready
---
