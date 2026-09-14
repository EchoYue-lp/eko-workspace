---
schema_version: 3
supersedes: null
slug: echo-agent-Eval-workspace-generation/隔离EvalWorkspaceGeneration
goal: 让每次EvalRunner执行拥有唯一workspace generation，并按producer是否settled区分显式清理与保留
ships: EvalRunner为每次case执行创建唯一workspace
  generation，settled路径显式清理，timeout/caller-drop保留隔离目录；Improve与Comparator不再使用固定或无人拥有的temp
  parent
verify: 同case与无fixture并发隔离、settled cleanup、cleanup failure、timeout/caller-drop
  retain和early-stop均有确定性red/green；Eval/Improve测试、双语文档合同、eval/improve feature
  Clippy/check、SDK artifact零diff、semantic/Issue门禁与独立review全部通过
design_ref: docs/supreme/specs/2026-09-14T0057-echo-agent-Eval-workspace-generation/design.md
delivery_ref: null
todos:
  - id: own-per-run-workspace-generation
    files:
      - echo-agent/src/eval/runner.rs
    summary: 以私有generation guard统一per-run cwd与settled/unsettled cleanup disposition
    verify: 同case与无fixture并发run的cwd唯一；success/error显式close；cleanup
      failure可见；timeout/caller-drop保留路径
  - id: converge-eval-consumers-and-docs
    files:
      - echo-agent/src/improve/loop.rs
      - echo-agent/src/eval/comparator.rs
      - echo-agent/docs/en/24-eval-system.md
      - echo-agent/docs/zh/24-eval-system.md
      - echo-agent/docs/adr/0036-eval-workspace-generation-lifecycle.md
    summary: 移除Improve/Comparator固定temp parent并同步双语生命周期合同
    verify: early-stop与并发loop不创建improve_i；Comparator不留ab_compare
      parent；双语文档与ADR明确parent/generation/timeout边界
  - id: close-eval-workspace-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: "写入repair、verification和独立复审并只关闭Issue #50对应Finding"
    verify: "目标Finding具备三类关闭refs；Issue #48和其它77个open
      Finding不变；92个Finding与Issue一对一且semantic gates通过"
artifact_id: plan:b33e8464-343a-412e-8936-1893aa3b5e8f
lifecycle: completed
design_revision: sha256:df00897e02965d12120fffccbc8b090944cccf0e9fb729af6fd2636a909dcde8
---
## Approach

- 在EvalRunner::run入口创建私有EvalWorkspaceGeneration；Builder在workspace_root下生成随机eval-目录并默认disable_cleanup，避免caller-drop误删未结算producer。
- Fixture复制到空generation root；无fixture也使用该root，彻底删除workspace_root/case.id复用和remove_dir_all reset。
- Success、typed Agent error与criteria/trace完成后显式TempDir::close；cleanup error写入同一EvalResult violation并使结果失败。
- Timeout显式keep并记录路径；run future被drop时guard Drop warning且保留。Issue #48继续拥有Turn settlement与后续回收。
- ImprovementLoop复用以系统temp为parent的EvalRunner，删除improve_i和手工cleanup；AbComparator同样不创建额外UUID parent。
- 同步双语Eval文档与ADR 0036。Public类型和签名不变，SDK artifacts必须零diff；echo-website按用户冻结要求不动。
- 独立review后只关闭finding.eval-workspace-generation-isolation；Issue #50保持open直到远端main交付。

## Global Constraints

- workspace_root只表示generation parent，Agent与criteria永远只接收唯一generation path。
- 不得按case ID、iteration index或进程全局固定名称构造可复用workspace；不得递归删除调用方提供的parent。
- Generation默认不在Drop时删除；只有已证明settled的路径显式close并处理错误。
- Timeout/caller-drop不得清理，必须保留并诊断；Issue #48保持open，不在本Plan伪造Turn settlement。
- Fixture setup失败不启动Agent，并显式处理generation cleanup结果。
- 不新增worktree/container/sandbox manager、background reaper、public状态、依赖、wire/schema或SDK identity。
- 不改变Eval评分、trace、grader、factory、run_all串行和Improvement建议语义。
- docs/en与docs/zh保持对等；echo-agent-cli与echo-website不修改，website镜像延后到统一文档收敛。
- 不使用unwrap/expect/panic/unreachable/todo、UTF-8字节切片或Worker术语。
- Issue #50已存在且唯一；只关闭该Finding，其它open Finding不顺带处理。

## Files

- Modify: `echo-agent/src/eval/runner.rs` — generation guard、fixture/cwd、cleanup disposition与确定性测试。
- Modify: `echo-agent/src/improve/loop.rs` — 删除固定improve_i与手工cleanup，复用EvalRunner。
- Modify: `echo-agent/src/eval/comparator.rs` — 删除无人拥有的ab_compare UUID parent。
- Modify: `echo-agent/docs/en/24-eval-system.md` — 说明per-run generation与cleanup。
- Modify: `echo-agent/docs/zh/24-eval-system.md` — 同步中文合同。
- Create: `echo-agent/docs/adr/0036-eval-workspace-generation-lifecycle.md` — 记录隔离、settlement与回滚。
- Modify: `echo-agent/.echo-semantic` — source snapshot、repair/verification/rereview和Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复进度与下一frontier。

## Reuse

- 现有`tempfile = 3`、`TempDir::close/keep`与Builder disable_cleanup — 唯一目录lifecycle原语，无新依赖。
- `AgentInvocationContext.working_dir`、`EvalResult.violations/recompute_score`、`copy_dir` — 保持执行与结果权威。
- `EvalRunner::run/run_all_async` — 每个case唯一入口，不在Improve/Comparator重复generation逻辑。
- Issue #48与`finding.eval-timeout-settlement` — 未结算producer的独立关闭责任。
- OpenAI Evals per-sample TemporaryDirectory、Inspect AI per-sample sandbox/cleanup interruption — 设计依据。
- `finding.eval-workspace-generation-isolation`与Issue #50 — 唯一关闭目标。

## Todos

### own-per-run-workspace-generation

requirements:
- § 目标行为
- § 核心结构与数据流
- § 异常与边界场景
- § 复用与实现约束

interfaces:
- consumes: EvalRunner.workspace_root、EvalCase.project_fixture、AgentInvocationContext.working_dir、EvalResult。
- produces: 私有EvalWorkspaceGeneration、唯一per-run cwd与settled/unsettled cleanup disposition。

steps:

1. 添加同case并发、无fixture并发、caller-drop、timeout retain与cleanup-error red。
   verify: 旧实现出现相同cwd/互删、固定root暴露、无可靠cleanup disposition或固定目录残留；失败证据不依赖随机sleep。
   expected: Issue #50的generation、cleanup和timeout边界各有可复核反例。

2. 实现generation guard并把fixture、Agent、criteria和结果cleanup接入唯一run路径。
   verify: 每次run目录唯一；fixture内容完整；settled close删除；timeout/caller-drop路径存在且可诊断；cleanup error使结果失败。
   expected: 同名case和无fixture并发不共享、删除或污染目录，parent永不被递归reset。

3. 运行EvalRunner focused lifecycle与既有criteria/trace测试。
   verify: generation tests和既有EvalRunner tests全部通过，score/trace/criteria不回归。
   expected: 唯一workspace owner不改变评测结果语义。

### converge-eval-consumers-and-docs

requirements:
- § 目标行为
- § 关键取舍与业界依据
- § 公共合同与兼容
- § 验收标准

interfaces:
- consumes: EvalRunner per-run generation合同。
- produces: 无固定temp命名的Improve/Comparator调用、双语文档与ADR 0036。

steps:

1. 收敛ImprovementLoop与AbComparator temp parent。
   verify: 源码不再包含improve_i、ab_compare_或手工remove runner.workspace_root；early-stop和并发调用只留下unsettled generation。
   expected: consumer不再拥有第二套workspace命名或cleanup逻辑。

2. 更新双语Eval文档并写Accepted ADR 0036。
   verify: en/zh都说明parent、per-run cwd、settled close、timeout retain与Issue #48边界；文档合同通过。
   expected: public行为变化、兼容影响、业界依据和回滚可追踪。

3. 运行Eval/Improve feature Clippy/check、相关tests和SDK零diff。
   verify: eval/improve feature组合与docs checks exit 0，contracts/sdk和sdks/shared相对基准零差异。
   expected: 无public identity或跨语言合同漂移。

### close-eval-workspace-finding

requirements:
- § 验收标准
- 用户要求语义修复严格执行一Finding一Issue。
- Echo Semantic resolved Finding关闭合同。

interfaces:
- consumes: red/green日志、ADR 0036、文档、最终diff、Issue #50与独立review。
- produces: resolved workspace Finding、repair/verification Evidence、rereview Audit和更新后的MASTER-PLAN。

steps:

1. 刷新Eval Map/Behavior/Rule/Asset与source snapshot，写repair/verification Evidence。
   verify: before绑定e59fe773，after绑定current source digest；Issue #50 URL唯一；Issue #48及其它Finding状态不变。
   expected: generation authority、cleanup disposition、timeout residual、验证与回滚可追踪。

2. 独立review后写rereview Audit并执行最终语义与Issue门禁。
   verify: reviewer无blocker；strict snapshot、change evidence、Issue reconciliation和git diff check通过。
   expected: Finding本地resolved，Issue #50仍open等待远端main交付。

## Decisions

- 使用私有TempDir guard而非public Workspace类型、手工UUID或第二store。
- Generation在创建时禁用隐式cleanup，settled路径显式close；这是为了让caller-drop fail-safe地保留，而非长期关闭cleanup。
- Timeout retain不是成功清理，只是把未结算producer限制在唯一generation；最终回收属于Issue #48。
- workspace_root继续公开但语义收窄为parent；不新增Result构造器或迁移字段。
- ImprovementLoop与Comparator不再拥有temp命名和删除逻辑，全部复用EvalRunner。
- 本Plan只关闭Issue #50。