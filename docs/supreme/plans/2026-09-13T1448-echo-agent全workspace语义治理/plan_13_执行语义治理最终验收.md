---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/执行语义治理最终验收
goal: 对从远端 main 到当前累计治理分支的语义、工程、SDK、文档和 Issue 合同执行一次可重算最终验收
ships: 全部语义对象、已批准修复、正式文档、examples、contracts 和适用工程门禁形成可重算的最终验收证据
verify: main/origin-main 均为当前 HEAD 祖先且工作树干净；./scripts/verify.sh、17 个独立 feature
  check、SDK contracts、三语言 SDK、documentation/examples/facade、semantic
  strict、各高风险切片 task-scoped change-evidence 历史、b21aba01 到候选结果的 semantic
  continuity、93/93 Issue 对账和最终独立 review 全部零失败
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#governance-final-verification
todos:
  - id: prove-integration-baseline
    files:
      - echo-agent/.echo-semantic
    summary: 确认 main 祖先、线性累计提交、工作树和语义基线适合最终验收
    verify: main 与 origin/main 指向 b21aba01 且都是当前 HEAD 祖先；无 merge/rebase 或未提交业务改动；93
      个 Finding 状态和 refs 结构有效
  - id: run-full-rust-gates
    files:
      - echo-agent/.echo-semantic
    summary: 执行仓库规定的完整 formatter、Clippy、panic-policy、all-feature tests 与 no-default
      check
    verify: ./scripts/verify.sh exit 0，所有命令零失败、零 warning、零 fmt diff
  - id: run-feature-sdk-doc-contracts
    files:
      - echo-agent/.echo-semantic
    summary: 执行独立 feature、SDK/三语言和正式文档/examples/facade 合同
    verify: 17 个 root feature check、SDK 90
      artifacts/inventory、TS/Python/Java、documentation 10、all-feature examples
      21 与 facade 10 全部通过且生成物零漂移
  - id: verify-semantic-and-issue-closure
    files:
      - echo-agent/.echo-semantic
      - echo-agent/docs/adr/0041-semantic-governance-continuity.md
    summary: 验证最终语义快照、累计语义连续性、Finding 关闭条件和 93/93 Issue 映射
    verify: semantic strict 与 b21aba01 到候选结果的 continuity 通过；各高风险 repair slice 保留
      task-scoped change-evidence；71 open/22 resolved 与三类关闭 refs 对账；93 个 marker
      唯一；本地 resolved Issue 在远端 main 前保持 OPEN
  - id: publish-final-verification-evidence
    files:
      - echo-agent/.echo-semantic
      - echo-agent/docs/adr/0041-semantic-governance-continuity.md
      - echo-agent/docs/en/README.md
      - echo-agent/docs/zh/README.md
      - docs/MASTER-PLAN.md
    summary: 在独立复审后记录最终验收 Evidence/Audit 和顶层治理结论
    verify: 最终 reviewer 无 blocker；Evidence 包含命令、revision、退出码、覆盖和残余风险；本地提交创建但不
      push/PR/merge
lifecycle: completed
artifact_id: plan:b28cf38c-c009-4854-86b3-d167194309d4
design_revision: null
---
## Context

- `main` 与 `origin/main` 当前同为 `b21aba01`，且均是治理分支 `1cb25e80` 的祖先；因此无需执行未授权 merge。
- 累计链已完成全 workspace baseline、高风险 Audit、SDK scope reset、十四个runtime/doc Finding repair slice、三个README contract repair和framework concept navigation；最终复审新增的SDK backlog口径Finding由独立Plan 14处理，完成后当前93 Finding为71 open/22 resolved。
- 这是验证和证据 outcome，不是批量修复 outcome。任何新失败或语义问题必须先建唯一 Issue/Finding，并作为新 delivery outcome 处理。

## Approach

- 从 `1cb25e80` 创建 `test/Echoyue/semantic-governance-final-verification`，记录 main/origin-main/merge-base/HEAD 与干净工作树；不执行 merge、rebase、push、PR 或 cleanup。
- 先运行仓库等价完整门禁 `./scripts/verify.sh`，其顺序覆盖 fmt、workspace all-target/all-feature Clippy、panic-policy Clippy、workspace all-target/all-feature tests 和 no-default lib check。
- 因累计治理包含跨 crate public API 与 SDK inventory 变化，额外执行 AGENTS.md 指定的 17 个独立 root feature check。
- 运行 `scripts/check-sdk-contracts.sh` 与 `scripts/check-language-sdks.sh`，并单独保留 documentation/examples/facade 的可读计数证据。
- 在所有工程命令通过后写 final-verification Evidence；为四个仍存在但指纹已变化的SDK义务新增ADR 0041连续性裁决并同步双语ADR索引；运行semantic-diff/strict snapshot，复核各高风险repair slice的task-scoped change-evidence，并以semantic continuity对比`b21aba01`与最终候选结果；同时对账Finding refs、Issue marker与remote state。
- 将完整branch diff、命令日志和语义证据交给独立reviewer；review通过后提交echo-agent内本Plan全部受控产物，包括`.echo-semantic`、ADR 0041和双语索引。顶层MASTER-PLAN在同一checkout更新但不进入child commit。

## Global Constraints

- 所有仓库门禁必须真实执行并读取完整退出码/失败数；不得以先前 focused 证据替代最终工程验收。高风险 repair 的 change-evidence 必须继续由各自 task-scoped preflight 证明，最终任务不得回溯伪造跨切片 preflight。
- 任何失败，包括既有或看似无关失败，都不得跳过、降级、抬阈值或用子集掩盖。
- 发现新独立语义问题立即创建唯一 GitHub Issue 和 Finding；最终 Plan 保持 ready，先新增 repair outcome/Plan。
- Final Evidence 不得把71个open Finding表述为已修复；语义治理完成表示基线、追踪、已批准repair、文档与门禁闭合。ADR 0041只处置历史义务连续性，不关闭任何Finding或改变SDK external contract。
- 22个resolved Finding必须各有repair_evidence_refs、verification_evidence_refs和rereview_audit_refs；对应GitHub Issue在提交进入远端main前保持OPEN。
- SDK inventory 继续作为 drift/scope contract，不作为项目完成百分比；当前 canonical/scope 数以生成物为准。
- 不修改业务代码、Cargo manifest、examples或SDK；正式文档只允许新增ADR 0041并同步英中ADR索引，用于记录continuity决策权威。如验证暴露运行问题，转新Issue/Plan，不在验证提交中修复。
- 不执行 merge、rebase、push、PR、publish、release、分支删除或 worktree cleanup。
- 只使用 Subagent 术语；不使用禁用 panic API 或 UTF-8 字节切片。
- `echo-agent-cli` 与 `echo-website` 不进入本 Plan；它们没有消费新的 runtime/API，网站同步等待未来明确对外交付。

## Files

- Modify: `echo-agent/.echo-semantic` — 最终验证 Evidence/Audit、continuity/equivalence证据、baseline/source refs和治理状态。
- Create: `echo-agent/docs/adr/0041-semantic-governance-continuity.md` — 记录从`b21aba01`到最终候选结果的义务处置、兼容影响和回滚。
- Modify: `echo-agent/docs/en/README.md` — 将ADR 0041加入英文架构决策索引。
- Modify: `echo-agent/docs/zh/README.md` — 将ADR 0041加入中文架构决策索引。
- Modify: `docs/MASTER-PLAN.md` — 当前revision、门禁结果、Finding/Issue计数、残余风险和远端待交付状态。

## Reuse

- `echo-agent/scripts/verify.sh` — 仓库完整合并前 Rust 门禁入口。
- `echo-agent/AGENTS.md` — 17 个独立 feature 条件矩阵和提交约束。
- `echo-agent/scripts/check-sdk-contracts.sh` — 90 artifacts、Rust inventory 与协议合同。
- `echo-agent/scripts/check-language-sdks.sh` — TypeScript/Python/Java facade 与连接检查。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — README/docs/examples/Cargo-derived 合同。
- `echo-agent/.echo-semantic` — 93 Finding、44 Audit、43 Evidence 与 main change-evidence 基线。
- `docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md` — 最终 outcome 与依赖真理源。

## Todos

### prove-integration-baseline

requirements:
- 用户确认的全 workspace 语义治理目标
- 仓库分支规范与一 Finding 一 Issue 约束

interfaces:
- consumes: main/origin-main/HEAD refs、git graph/status、当前 semantic baseline 与 completed delivery outcomes
- produces: 可重算 base revision、线性差异范围和最终命令日志前置

steps:

1. 创建 final-verification 分支，核对 main/origin-main/ref/merge-base、工作树与累计提交链。
   verify: 两个 main ref 均为 HEAD 祖先且 merge-base=b21aba01；child工作树干净；不需要 merge。
   expected: 最终验证绑定单一线性 base，不产生新 Git 历史副作用。

2. 对账当前 semantic 对象计数、22个resolved Finding 的三类 refs 与 71 个 open Finding 状态。
   verify: 对象结构通过 strict snapshot；任一关闭证据缺失或 open 状态漂移立即失败。
   expected: 验证前治理状态已知且未把 backlog 当完成项。

### run-full-rust-gates

requirements:
- 仓库 `全量提交门禁(合并到 main 前强制)`
- Plan 顶层 verify

interfaces:
- consumes: 线性累计治理 HEAD、Cargo lockfile、incremental target cache
- produces: formatter、两档Clippy、all-target/all-feature tests 与 no-default check 完整日志

steps:

1. 运行 `./scripts/verify.sh` 并保留完整日志。
   verify: 脚本及内部每个命令 exit 0；测试报告零失败，Clippy零warning，formatter零diff。
   expected: 当前累计分支满足仓库完整 Rust 合并门禁。

### run-feature-sdk-doc-contracts

requirements:
- 仓库 `条件矩阵(仅触及对应风险面时)`
- 用户要求 SDK contract 作为独立分层验证而非完成度
- 文档/examples同步约束

interfaces:
- consumes: root Cargo features、SDK generator/contracts/language clients、learning docs/examples、root facade
- produces: feature隔离、SDK reproducibility、语言 facade、文档/example/facade 可读计数日志

steps:

1. 依次运行 `acp a2a mcp lsp sqlite telemetry topology subagent web media data statistics channels git database rag chart` 的 `cargo check -p echo_agent --no-default-features --features <feature> --locked`。
   verify: 17个feature逐项exit 0，任何隔离编译失败立即阻断。
   expected: Public API/feature拓扑变化没有破坏单feature组合。

2. 运行 `./scripts/check-sdk-contracts.sh` 与 `./scripts/check-language-sdks.sh`。
   verify: 所有生成artifact current，Rust inventory/fixture tests与TS/Python/Java gates零失败，工作树无生成diff。
   expected: SDK scope分类与实际source/Host合同可重算。

3. 从完整门禁提取documentation、all-feature example contracts与facade smoke结果；必要时单独重跑相应test target以保留清晰日志。
   verify: documentation 10、example contracts 21、facade smoke 10通过，六份概念文档结构/链接/导航合同有效。
   expected: 正式文档与真实consumer/target保持一致。

### verify-semantic-and-issue-closure

requirements:
- 用户要求每个语义问题唯一 Issue 且远端 main 前不关闭
- Echo Semantic strict snapshot、task-scoped change-evidence 历史、semantic continuity 与关闭条件

interfaces:
- consumes: final command logs、`.echo-semantic`对象、base b21aba01、GitHub Issues #24-#116
- produces: final-verification Evidence候选、semantic gate与Issue reconciliation结果

steps:

1. 写入最终 Evidence候选，记录每条命令、base/HEAD、退出码、覆盖、未验证项与71个open Finding。
   verify: Evidence不宣称远端CI/docs.rs/发布已完成，不改变Finding状态。
   expected: 所有本地结论有明确证据边界。

2. 运行semantic strict snapshot，复核各高风险repair slice的task-scoped change-evidence，并以semantic continuity比较base `b21aba01`与最终候选结果；对账93个Finding URL、remote marker唯一性和resolved/open状态。
   verify: strict和continuity命令exit 0，所有非保全义务均有可重算resolution；93/93映射无重复；22 resolved具备三类refs；全部本地resolved Issue仍OPEN。
   expected: 累计治理语义被连续性合同覆盖，本地语义关闭和远端交付生命周期保持分离。

### publish-final-verification-evidence

requirements:
- Delivery map `governance-final-verification`
- Build review与本地提交门禁

interfaces:
- consumes: 全部最终日志、branch diff、semantic/Issue结果和独立review
- produces: final-verification Audit、最终 Evidence、MASTER-PLAN 状态与本地验证提交

steps:

1. 将完整diff、命令日志、语义对象与残余风险交给独立review。
   verify: reviewer的Critical/Important/Minor为0；任何新Finding先建Issue并退出本Plan。
   expected: 最终结论由独立反证而非自证形成。

2. 写入最终Audit/Evidence结论，重跑受影响semantic/Issue/diff门禁，并提交echo-agent内本Plan全部受控产物：`.echo-semantic`、ADR 0041和双语索引。
   verify: child提交包含全部受控产物且工作树干净；MASTER-PLAN准确记录child commit但保留顶层未提交；无push/PR/merge。
   expected: 治理目标在本地达到可交付状态，远端交付和71个backlog Finding保持显式。

## Decisions

- main/origin-main已是当前HEAD祖先，不执行无必要且未授权的merge。
- 最终验证使用仓库标准 `scripts/verify.sh`，不机械重复同feature组合的cargo命令。
- 累计链含public API/feature/SDK变化，因此17个单feature check属于适用条件矩阵。
- 最终Evidence只记录验证；新发现的SDK backlog口径漂移由Issue #116和Plan 14独立修复。最终task preflight授权`.echo-semantic`、ADR 0041和双语索引，累计跨切片语义使用continuity而非扩大或伪造历史preflight。
- Concept navigation没有新runtime/API consumer，`echo-agent-cli`与`echo-website`本Plan不修改。
