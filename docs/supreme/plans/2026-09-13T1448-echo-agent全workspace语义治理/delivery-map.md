---
schema_version: 1
artifact: delivery-map
design_ref: null
outcomes:
  sdk-final-slice-merge:
    ships: "将 CI 全绿的 PR #23 作为最后一个逐 identity SDK 切片 squash merge 到 echo-agent
      main，并更新 superproject 的 echo-agent 指针"
    depends_on: []
  sdk-governance-route-reset:
    ships: 停止创建逐 identity PR，将 9,682 项 inventory 重定位为 SDK 漂移监控和分类 backlog，并把全
      workspace semantic discovery 设为主路线
    depends_on:
      - sdk-final-slice-merge
  workspace-semantic-baseline:
    ships: 建立绑定当前 revision 的全 workspace 语义基线，闭合路径库存、Capability Map、核心
      Asset、状态权威、生命周期、外部副作用和未知入口
    depends_on:
      - sdk-governance-route-reset
  high-risk-boundary-audits:
    ships: 对运行时、持久化、并发恢复、权限与外部副作用、协议契约等高风险边界形成可反证 Audit 和当前 Finding 集合
    depends_on:
      - workspace-semantic-baseline
  sdk-contract-scope-reset:
    ships: 将 SDK inventory 重分类为真实对外 contract、Host 或 Rust-only、language intrinsic、内部
      helper 与 deferred backlog，并保留漂移监控而非完成度门禁
    depends_on:
      - workspace-semantic-baseline
  repair-task-patch-claim-cas:
    ships: Task relation patch commit 同时校验其读取时的 execution snapshot，runtime
      claim、retry 或 settlement 的并发变化返回 typed conflict 且不覆盖 live claim
    depends_on:
      - high-risk-boundary-audits
  repair-task-patch-claim-cas-sdk-contract:
    ships: Task relation patch 以读取时 execution snapshot 做条件提交并拒绝覆盖 runtime 变化，同时把新增的
      public commit precondition 同步到 SDK inventory、operation catalog 和三语言合同
    depends_on:
      - high-risk-boundary-audits
      - sdk-contract-scope-reset
  repair-subagent-factory-singleflight:
    ships: Subagent lazy factory 以 registration revision scoped single-flight
      原子完成同代构造与发布，取消后可恢复、旧代结果不发布，并关闭 cancellation 与 publication race 两个 Finding
    depends_on:
      - high-risk-boundary-audits
  repair-improve-singleton-split:
    ships: ImprovementLoop 对单例 criteria 分组采用 train-only disposition，消除合法单 case 输入的
      panic，同时保持多例分组的独立 holdout
    depends_on:
      - high-risk-boundary-audits
  repair-improve-iteration-config:
    ships: EvalDrivenImprovement 将 public max_iterations 配置无损传入唯一
      ImprovementLoop，包含零迭代语义，并使实际成本与调用方请求一致
    depends_on:
      - high-risk-boundary-audits
  repair-streaming-tool-validation:
    ships: ToolManager 的 streaming 与 non-streaming 执行在 cache、permit 和外部 effect 前复用同一
      schema 与 custom parameter validation kernel
    depends_on:
      - high-risk-boundary-audits
  repair-tool-read-cache-authority:
    ships: ToolManager read cache 以 effective workspace 与 invocation lineage
      定义作用域，并用 write-lifetime epoch CAS 阻止 in-flight Read 在状态变更后重新发布旧结果
    depends_on:
      - high-risk-boundary-audits
  framework-concept-convergence:
    ships: 在已批准 Finding 修复完成后同步中英文架构、核心概念、生命周期、状态权威、示例与公共契约文档
    depends_on:
      - high-risk-boundary-audits
      - sdk-contract-scope-reset
      - repair-task-patch-claim-cas-sdk-contract
      - repair-subagent-factory-singleflight
      - repair-improve-singleton-split
      - repair-improve-iteration-config
      - repair-streaming-tool-validation
      - repair-tool-read-cache-authority
      - repair-tool-registry-mutation-lifecycle
      - repair-eval-workspace-generation-isolation
      - repair-eval-timeout-settlement
      - repair-eval-trace-correlation
      - repair-background-task-terminal-authority
  governance-final-verification:
    ships: 全部语义对象、已批准修复、正式文档、examples、contracts 和适用工程门禁形成可重算的最终验收证据
    depends_on:
      - framework-concept-convergence
      - repair-sdk-deferred-backlog-count-drift
  repair-tool-registry-mutation-lifecycle:
    ships: ToolManager执行路径不再持有registry
      guard跨await，动态replace/unregister可在current-thread runtime完成，并保留Tool
      generation与result cache freshness fence
    depends_on:
      - high-risk-boundary-audits
      - repair-tool-read-cache-authority
  repair-eval-workspace-generation-isolation:
    ships: EvalRunner为每次case执行创建唯一workspace
      generation，settled路径显式清理，timeout/caller-drop保留隔离目录；Improve与Comparator不再使用固定或无人拥有的temp
      parent
    depends_on:
      - high-risk-boundary-audits
      - repair-improve-singleton-split
      - repair-improve-iteration-config
  repair-eval-timeout-settlement:
    ships: Eval超时后取消并有界等待唯一TurnReceipt；已结算才读取终态trace并清理generation，未结算则跳过评分并保留隔离目录
    depends_on:
      - high-risk-boundary-audits
      - repair-eval-workspace-generation-isolation
  repair-eval-trace-correlation:
    ships: Eval以每次invocation唯一run/turn/execution correlation从RunStore解析真实trace
      identity，使EvalResult、trace criteria、constraints和metrics消费同一个已验证Run
    depends_on:
      - high-risk-boundary-audits
      - repair-eval-timeout-settlement
  repair-background-task-terminal-authority:
    ships: BackgroundTask以单一共享state原子发布status与单消费者result，可克隆handle的多waiter不会挂起，admission/执行共享取消与绝对deadline，execution
      task panic、零并发和提前取消都结算
    depends_on:
      - high-risk-boundary-audits
      - sdk-contract-scope-reset
  repair-sdk-deferred-backlog-count-drift:
    ships: 将全workspace discovery和protocol map中的SDK backlog统一为1441个deferred
      identity的capability级决策，并明确Host/Rust-only、language intrinsic与internal
      helper不属于语言parity backlog
    depends_on:
      - sdk-contract-scope-reset
  repair-semantic-baseline-squash-ancestry:
    ships: 将squash merge后失效的semantic baseline
      ancestor重新绑定到远端main，并增加target-main稳定祖先的pre-merge合同与squash反例，恢复strict/continuity且防止同类回归
    depends_on:
      - governance-final-verification
design_revision: null
---
该交付图以语义边界和 Finding 为推进单位。discovery 或 audit 产生的每个独立修复必须先作为新的 outcome 插入本图，并补到 framework-concept-convergence 的依赖，再创建对应 Plan；不得用一个大修复 Plan 吞并多个可独立合并的 Finding。repair-task-patch-claim-cas 绑定已 superseded 的范围不足 Plan 06；前向交付由包含 public SDK contract 同步的 repair-task-patch-claim-cas-sdk-contract 取代。Subagent factory 的 cancellation 与 publication race 共享并替换同一个 registry-wide instantiating authority，必须在一个 revision-scoped single-flight outcome 中原子收敛。
Improve singleton split是私有分组算法的独立bugfix，不与max_iterations、workspace generation或Eval timeout合并。
EvalDrivenImprovement iteration wiring只修复既有public配置到唯一loop的传递，不与split、workspace或timeout合并。
Streaming Tool validation是同一ToolManager入口对等修复，不与cache scope/invalidation或permission policy合并。
Tool read cache的scope与in-flight invalidation共享一个key/epoch authority，必须原子修复，不能拆成两个长期阶段。
每个Finding必须先建立唯一GitHub Issue并回写URL后才可进入repair；Issue使用稳定echo-semantic-finding marker查重，只有远端main交付与三类关闭证据齐备后才能关闭。
Tool registry mutation lifecycle独立修复跨await DashMap Ref阻塞，不回退已闭合的cache scope/epoch authority。
Eval workspace generation isolation以每次EvalRunner::run唯一generation闭合并发与early-stop；timeout/caller-drop因Issue #48未结算而保留目录，不以本outcome宣称cleanup完成。
Eval timeout settlement复用唯一AgentTurnDriver与现有6秒grace；cancel请求不等于终态，只有TurnReceipt允许终态trace与generation cleanup，grace失败保持隔离。
Eval trace correlation复用ExternalRunContext与RunStore parent/turn/execution关联，不扩TurnReceipt或SDK，也不再用Agent共享product run ID猜测trace。
BackgroundTask terminal authority只修复process-local future handle；Clone新增identity归入Rust language intrinsic，不扩三语言facade，也不替代revisioned Task DAG。
SDK deferred backlog口径漂移由Issue #116跟踪；修复只统一语义材料中的1441 deferred capability backlog，不改变manifest、route、language status或runtime。
Semantic baseline squash ancestry由Issue #118跟踪；修复更新基线祖先，并在现有learning contract与Rust CI中增加target-main ancestry和squash反例，不放宽Echo Semantic verifier或改变runtime/API。
