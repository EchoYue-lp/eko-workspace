---
slug: eko-gui-product-convergence/task-list-dialog
outcome:
  summary: 任务列表与弹窗复用后端任务事实、进度和执行回执，统一任务层级、等待原因、详情及操作，并在高数据量下保持有界呈现。
  acceptance:
    - 当前 Workspace 的正式 TaskRun 可发现并按原 run_id 精确进入既有 TaskRuntimePanel；同一会话存在多个历史
      run 时不被 latest 查询替换，普通未提交计划的 conversation turn 不因打开列表而升级为用户任务。
    - 列表/详情沿 TaskRun → PlanTask → SubagentRun
      原事实展示来源、状态、等待原因、真实进度、时间、产物与操作回执；未知值不虚构，accepted/no-op/rejected
      与物理结算有明确区别，迟到结果不串 Workspace、run 或 task。
    - 列表/详情与 composer 消费同一 canonical Goal/plan revision
      及覆盖报告；目标修改后原详情不能继续显示旧计划/旧证据为当前完成，结束未结算不冒物理停止。多 Steer/Goal 写操作由 Plan12
      的原控制链拥有，完整有效要求由 Plan14 拥有，三者不可互相借测试关闭。
    - 保留每页20项PlanTask呈现；原后端运行查询及事件历史均返回有限页和明确后续位置，不全Vec拉回截断、不虚构total，读取失败不冒完整结果。列表相关查询与渲染的同源测量、合法单图1/10/50/100及具名run/事件集合和真实Tauri验收完成后才能交付；完整performance-baseline继续作为整个Goal的独立必要结果。
    - 适用前端行为/类型/lint/格式/构建、所改 Rust 查询/IPC
      与独立审查通过；正式双语文档、examples/官网适用性对账，并按所属仓规则完成交付门禁。
out_of_scope:
  - 不新增 Task/Todo/Conversation/Subagent store、状态机、并行任务
    CRUD、调度器或数据库，不改变原任务准入、权限、取消/物理结算和 revision 权威。
  - 不重新实施780已有计划任务样式/Modal/20项分页及已覆盖的详情行为；不改聊天首页、HITL、微信/其它IM、cron配置或远程项目执行/认证。
  - 不在本结果实施
    memory/history、工具或存储性能优化，不把10k事件当10k合法PlanTask，不修改性能阈值、规避单图约束或以客户端截断声称服务端分页。
  - 不扩Framework/SDK公共API、Cargo依赖/锁、全局主题或TaskRun领域模型；复用原EKO运行/事件有限页及已有控制回执，只消费Plan12/Plan14的canonical变化，不在列表另造Goal/Steer写入、要求store或journal合同。不发布、推送、合main或关闭其它Outcome。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:e1430cc82e11f733fff0050ba572561937893ca3317cb0cd6b63bb8f08faa381
  - ref: design.md § 首页与工作空间导航
    digest: sha256:d3885ae131866409bf68ec9f0573af0f57bec76beab6a3af087ba073008c4198
  - ref: design.md § 统一视觉和信息层级
    digest: sha256:ae3294ad6b1fdac64513ac849bf6f49a48364a90ce2d945221959a39ff3b5862
  - ref: design.md § 任务列表弹窗
    digest: sha256:7ac0fd1ae1a0128ff6e30f398576e891b1e7c65573709f2af0708cb8bbcb78b5
  - ref: design.md § 唯一 Goal 与当前有效要求
    digest: sha256:2d810ec9aca486a6a1ef7d20ceb15490a9dfa04ef58be81941fa955cfe83ad12
  - ref: design.md § 测量与持续回归
    digest: sha256:9ce2b5788eed53f142334b396eaf369ff071833ebfdd0f9851844d40e1954392
  - ref: design.md § 12. 复用结论与完成标准
    digest: sha256:1e7084072aa642175303e9e30b436fb4f9b2aa0e460d654bb4e74abb3ad51815
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#task-list-dialog
todos:
  - id: discover-and-select-formal-task-runs
    summary: 在原任务入口发现正式运行，并将选中的历史run精确加载到既有TaskRuntime投影。
    files:
      - echo-agent-cli/web-frontend/src/components/layout/RightRail.tsx
      - echo-agent-cli/web-frontend/src/components/layout/ContextPane.tsx
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts
      - echo-agent-cli/web-frontend/src/api/endpoints.ts
      - echo-agent-cli/src/tauri/commands/task_runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/runtime.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
    acceptance:
      - 复用原task
        UI可见性判定，保留无plan的合法orchestrated/direct运行，排除尚无真实plan的普通conversation
        turn；GUI/IM/cron定位已有同一TaskRun，不复制任务或后台任务清单。
      - 选择历史R1后所有详情/轮询/控制仍使用捕获的Workspace与R1，不重新选同会话latest
        R2；切换目标、同id换generation、并发加载/旧轮询/失败晚回均不覆盖新目标。
      - 扩原list_task_runs内部IPC：捕获原Workspace/product_data generation、typed
        statuses、limit与opaque
        continuation；结果复用TaskRun并返回next_continuation。按created_at降序＋run_id降序稳定keyset续页，沿原authority逐项校验并只保有限候选，失败明确返回Err，不借全Vec截断/猜route或新增store。
      - 缺省/空statuses语义明确；非法状态/limit/cursor、跨scope或筛选续页拒绝，刷新清cursor。相同时间戳、新run插入、状态/plan在页间变化、空末页、读损坏与可恢复tail均有真实后端控制；说明这是live分页而非事务快照，不用updated_at排序导致行跨页漂移。
      - 原事件IPC复用RuntimeEventQuery/query_events_bounded的after_sequence与limit，返回明确后续位置和是否仍有尾部；历史详情先按页加载并沿选定run继续，MAX_EVENTS等前端保留量不能替代传输边界。读取/恢复错误、缺口和retention诊断不当空或完整，禁止另建事件store或改变journal。
  - id: complete-task-facts-and-typed-receipts
    summary: 沿已有列表与详情补齐等待原因、执行层级及准确操作回执，保留原任务事实和有界呈现。
    files:
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimeList.tsx
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.css
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimeList.test.tsx
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.test.ts
    acceptance:
      - 保留现有20项PlanTask页面、搜索/状态筛选、详情Modal/焦点与执行完成和任务提交完成区分；详情按原run/task/execution/attempt呈现依赖、等待/恢复原因、错误、产物和实际可用执行记录。
      - 暂停/取消使用原TaskRunControlReceipt，继续/重试使用原typed
        receipt；显示真实请求结果或明确no-op/rejection，不以Promise返回、按钮点击或最新Subagent完成替代TaskRun终态/物理停止。
      - pending与回执捕获原Workspace/run/task身份；A请求未settle时打开B、换run或重载后，late
        success/error仍不能写成B回执，已有跨任务反例与失败保留/重试、未知时间/进度、空/错/长Unicode标题控制继续通过。
      - composer修改Goal后同一run的列表/详情重新取canonical revision、计划和completion
        report；刷新失败保留明确stale/error，旧轮询/旧Subagent晚回不得恢复旧完成标签。结束请求accepted/no-op/rejected与真实终态分别展示，不增加第二个Goal写入口。
  - id: qualify-bounded-task-dialog-and-document
    summary: 验证真实入口、目标隔离与合法高数据量表现，完成双主题原生视觉和正式说明。
    files:
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimeList.test.tsx
      - echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.test.ts
      - echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.test.ts
      - echo-agent-cli/web-frontend/src/components/layout/ContextPane.test.tsx
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store/tests.rs
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/architecture/overview.md
      - echo-agent-cli/docs/zh/architecture/overview.md
    acceptance:
      - 先完成精确历史run选择、formal-vs-turn、跨Workspace/generation迟到、typed no-op/失败/late
        receipt及既有Modal键盘行为的定向反例和正常控制，再通过实际改动对应Node/Rust检查。
      - 使用经原TaskRevisionService/PlanValidator接受并冻结字节身份的GUI数据；分别测1/10/50/100合法单图、10/100/1000
        run集合与独立10k/100k事件历史，不将setState型fixture或超上限Todo当性能证据。
      - 消费列表相关query与GUI基线，按同源输入取得Tauri/WebView支持的常用和最小窗口、浅/深色下的列表/详情/加载/空/错/长标题/键盘焦点及高数据量结果；jsdom、浏览器观测（若使用）与原生证据分开，缺项保留未完成，不互相替代。
      - 正式双语说明解释Workspace内普通任务会话与TaskRuntime运行列表的区别，列出查询和结算边界；examples无公共API改动时记录不适用，官网按最终合格EKO源统一镜像，不手补旧pin。
schema_version: 4
artifact_id: plan:b814efe9-63c9-4946-9cb5-ee75e7d8af68
lifecycle: ready
---
## Notes

源码实现以 EKO780 已冻结并通过工程检查的主题、Modal、任务投影与控制接口为基线，复用已有每页20项的 PlanTask 与详情行为；GUI基础本身尚待的原生视觉反馈仍保留。本结果补齐正式运行历史发现、精确选择、有限事件读取与类型化回执。运行分页不保证底层ID枚举/authority恢复为常量成本，须如实测量，不把transport有界称作索引优化。

2026-10-26 接续以根 framework9c5、EKO780加已迁入dirty源码为准，旧外部v7 focused与零selected DTO命令不冒当前根完整资格。Plan12的37项controls或新的Goal/CAS回归也不结算本结果的历史run选择、DTO、合法高数据量/同源测量和真实Tauri义务。Plan14即使先通过原文修复，也不证明完整有效要求/覆盖报告已经可用；列表只呈现后端实际提供的事实。

任务组件与 taskRuntimeStore 的写入由协调者显式分配；共享 endpoints 仅 taskRuntimeApi 段、ContextPane/RightRail 仅任务目标段、只读查询和生成类型串行合流。核心Rust交汇由核心线拥有；GUI仅有next-actions所列六路径（ChatPanel及控件测试、双语guide、NewTaskDialog及工作空间a11y测试）的写权，Hook/Hook tests、Store/API仍属中央集成接缝。Plan引用文件不代表自动取得writer权限，没有接管前只读核对，不与两线并发改其文件。入口复用“任务运行”，保持侧栏普通“新建任务/最近任务”的Conversation语义。

列表相关查询、合法GUI数据和同源原生测量属于本结果验收；memory10k与工具等完整基准场景仍由原performance-baseline独立交付，不作为列表源码的技术等待条件。轻量实现与其它源码工作并行，正式性能测量与重型编译按协调窗口错峰。
