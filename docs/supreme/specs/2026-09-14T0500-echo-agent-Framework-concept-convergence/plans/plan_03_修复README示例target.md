---
schema_version: 3
supersedes: null
slug: echo-agent-Framework-concept-convergence/修复README示例target
goal: 让双语 README 的 echo-agent-learning 命令只引用 Cargo 真实 example/test target 与存在的
  contract filter
ships: 双语 README 的示例命令全部指向 Cargo 真实 example 或 test contract，并由现有 documentation
  contract fail closed 校验
verify: 旧 README 在 Cargo-derived command target contract 中稳定失败并只报告 demo34 缺失
  example target；修复后命令指向
  example_contracts/contract_demo34_workflow_stream，documentation
  contract、formatter、相关 Clippy/check、semantic strict/change-evidence、92/92 Issue
  对账和独立复审全部通过
design_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/design.md
delivery_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/plans/delivery-map.md#repair-readme-example-target
todos:
  - id: derive-readme-command-target-contract
    files:
      - echo-agent/echo-agent-learning/tests/documentation_contract.rs
    summary: 扩展现有文档合同，从 Cargo metadata 校验双语 README 的 learning example/test target 与
      contract filter
    verify: 合同使用 shell lexer 解析 learning 命令，对缺失 package/action/target、未知
      example/test target 和不存在的 contract filter fail closed
  - id: correct-bilingual-demo34-command
    files:
      - echo-agent/README.md
      - echo-agent/README.zh.md
    summary: 把 demo34 的错误 cargo run example 命令改为真实 example_contracts test 命令
    verify: 两份 README 使用同一真实 target/filter，其他 learning 命令、feature 表与 workspace
      topology 不变
  - id: close-readme-target-finding
    files:
      - echo-agent/.echo-semantic
    summary: "写入 repair、verification 与独立复审证据并只关闭 Finding #80"
    verify: "finding.readme-example-target-drift 具备三类关闭 refs；其它 72 个 open Finding
      状态不变；Issue #80 保持 OPEN 等待远端 main"
artifact_id: plan:d8660c46-9723-4654-8029-5f1caa33e63c
lifecycle: completed
design_revision: sha256:769b79d4599ee82922893785238c59a3131bab90b5bc250a6ae8f7aea5d0c875
---
## Approach

- 在新的 `fix/Echoyue/readme-example-target` 分支从当前 `8ab20d11` 基线执行；修改前记录 semantic-preflight，首个有效差异后执行 semantic-diff。
- 复用 `workspace_packages` 的 learning package metadata，并在 `WorkspacePackage` 投影中读取 Cargo targets 的 name/kind；example 与 test target 集合均从 metadata 派生。
- 使用已有 `shlex` 依赖解析两份 root README 中以 `cargo` 开头且 `-p echo-agent-learning` 的命令；`cargo run --example` 必须引用真实 example，`cargo test --test` 必须引用真实 test target。
- 对 `--test example_contracts` 命令额外要求唯一 `contract_*` filter，并在 `tests/example_contracts/*.rs` 中找到同名测试函数；不通过执行嵌套 cargo test 验证自身。
- 先在旧 README 上取得只报告 `demo34_workflow_stream` 不存在 example target 的有效 red，再将中英文命令改为 `cargo test -p echo-agent-learning --test example_contracts --all-features --locked contract_demo34_workflow_stream`。
- 独立 review 后写 rereview Audit 并只把 `finding.readme-example-target-drift` 标为 resolved；GitHub Issue #80 在本地提交进入远端 main 前保持 OPEN。

## Global Constraints

- Cargo metadata 是 example/test target 唯一权威；测试不维护手写 target allowlist。
- 本 Plan 只修复 #80；不得修改已闭合 #114 topology、#79 feature 表或其它 open Finding。
- README 命令必须可由用户直接复制；`demo34_workflow_stream` 是 test contract，不得为了保留旧命令新增同名 example wrapper。
- Contract 只静态校验命令路由与 filter 存在性，目标 contract 本身继续由现有 `example_contracts` test binary 执行。
- 不新增依赖、public Rust API、runtime 状态、协议或 SDK identity；`echo-agent-cli`、`echo-website`、Cargo manifests 与 example 源文件不修改。
- 英文与中文 README 的 demo34 command target/filter/options 必须语义对等。
- Issue #80 已存在且 marker 唯一；发现新独立语义问题时先建立唯一 Issue/Finding，再决定是否进入当前范围。

## Files

- Modify: `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — 增加 Cargo-derived README learning command target/filter 合同。
- Modify: `echo-agent/README.md` — 将 demo34 命令改为真实 test contract。
- Modify: `echo-agent/README.zh.md` — 与英文 README 对等修正 demo34 命令。
- Modify: `echo-agent/.echo-semantic` — 刷新 workspace map/docs asset/source snapshot，增加 repair/verification/rereview 并关闭 Finding #80。

## Reuse

- `echo-agent/echo-agent-learning/Cargo.toml` 与 Cargo metadata — example/test targets — 命令 target 唯一事实源。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — `workspace_packages`、metadata types 与现有 contract harness 检查 — 扩展而不新增测试框架。
- `echo-agent/echo-agent-learning/tests/example_contracts.rs` 与 `tests/example_contracts/demo34_workflow_stream.rs` — test target/filter — 真实执行入口。
- `echo-agent/.echo-semantic/maps/map.workspace-architecture.md` — `public-and-background-entrypoints` — 已定位 #80 与关闭目标。
- `echo-agent/.echo-semantic/audits/audit.workspace-architecture.contract-evidence.md` — demo34 target 直接反例 — 修复前审计基线。

## Todos

### derive-readme-command-target-contract

requirements:
- § 独立 Finding 前置
- § 文档事实源
- § 异常与边界场景
- § 文档校验扩展现有 contract

interfaces:
- consumes: learning package Cargo target metadata、root README cargo command lines、shlex、example_contracts source files
- produces: learning example/test target 与 contract filter fail-closed documentation contract

steps:

1. 扩展 metadata package 投影以读取 target name/kind，并从 `echo-agent-learning` package 派生 example/test target sets。
   verify: metadata 包含现有真实 examples、`documentation_contract`/`example_contracts` tests，且不包含 `demo34_workflow_stream` example。
   expected: Target expected set 完全来自 Cargo JSON，不在测试中手写 examples 清单。

2. 用 shlex 解析两份 README 中的 learning cargo 命令，按 run/test action 校验 `--example`/`--test` target；对 example_contracts 校验唯一 `contract_*` filter 的源码定义。
   verify: 命令解析失败、缺 package/target、未知 target、重复/缺失 filter 均产生明确错误；旧 README 只报告 demo34 不存在的 example target。
   expected: 合同覆盖 README 命令路由，不递归执行 cargo test 或误读普通 prose。

### correct-bilingual-demo34-command

requirements:
- § 独立 Finding 前置
- § Example 优先复用而不增加数量
- § 验收标准

interfaces:
- consumes: Cargo-derived target sets、example_contracts test/filter、旧 README 上的有效 red
- produces: 两份 README 中可复制执行的 demo34 contract command

steps:

1. 将中英文 quick examples 中的 demo34 行改为真实 `example_contracts` test target 与 `contract_demo34_workflow_stream` filter，并保留 `--all-features --locked`。
   verify: 上一 todo 的 command target contract 从 red 变 green；命令的 package、test target、filter 和 options 中英文一致。
   expected: 用户复制命令不再遇到 `no example target named demo34_workflow_stream`。

2. 对比范围，确认其它四条 quick example 命令、workspace tree、feature table、Cargo manifests 和 example sources 零变化。
   verify: git diff 只包含两条 demo34 命令、documentation contract 与语义材料。
   expected: #114/#79 不回归，未新增 wrapper 或平行 example。

### close-readme-target-finding

requirements:
- § 文档发布流
- § 独立 Finding 前置
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: red/green 日志、Cargo target metadata、双语 README diff、Issue #80、最终工程验证与独立 review
- produces: README command repair/verification Evidence、rereview Audit、resolved Finding #80 与更新后的治理计数

steps:

1. 刷新 workspace architecture map/behavior/framework docs asset 与 source snapshot，并记录 repair/verification Evidence。
   verify: Evidence 绑定 Cargo metadata、双语 README、真实 example_contracts target/filter 和有效 red/green，不宣称运行全部 README 命令。
   expected: command target 的事实源、消费者和验证链可从 Finding 追溯。

2. 在独立 review 无 blocker 后写 rereview Audit，将 `finding.readme-example-target-drift` 标为 resolved，并执行最终语义与 Issue 对账。
   verify: semantic strict/change-evidence、92/92 Finding-Issue 映射和 diff check 通过；92 个 Finding 中 71 open、21 resolved；Issue #80 仍为 OPEN。
   expected: 本地语义关闭与远端 Issue 交付状态严格分离。

## Decisions

- 当前 outcome 只修复 #80；前三个 README 前置完成后才进入概念导航发布。
- README demo34 使用既有 test contract，不新增 example wrapper。
- 命令解析复用 `shlex`；target 集合来自 Cargo metadata，filter 存在性来自真实测试源码。
- 本切片不触发 SDK inventory 生成；最终只做 SDK 路径零漂移检查。
