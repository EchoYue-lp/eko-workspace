---
schema_version: 3
lifecycle: completed
supersedes: null
slug: echo-agent全workspace语义治理/修复SemanticBaselineSquash祖先
goal: "修复PR #117 squash后baseline ancestor失效，并让后续PR在合并前拒绝feature-only baseline
  revision"
ships: 将squash merge后失效的semantic baseline
  ancestor重新绑定到远端main，并增加target-main稳定祖先的pre-merge合同与squash反例，恢复strict/continuity且防止同类回归
verify: baseline绑定d492c676；learning contract在目标main祖先上通过并以squash反例拒绝feature-only
  revision；Rust CI提供真实base SHA与完整历史；./scripts/verify.sh、strict、continuity、94/94
  Issue对账、独立review及远端CI全部通过；merged main复验后关闭23个resolved Issue
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-semantic-baseline-squash-ancestry
todos:
  - id: repair-baseline-ancestry
    files:
      - echo-agent/.echo-semantic
    summary: "记录Issue #118并将baseline祖先从squash丢失的中间提交改为已合并main"
    verify: 原strict失败稳定复现；修复后base_revision为d492c676且source
      digest、closure、maps和既有Finding状态不变
  - id: add-squash-aware-contract
    files:
      - echo-agent/echo-agent-learning/tests/semantic_baseline_contract.rs
      - echo-agent/.github/workflows/rust-ci.yml
      - echo-agent/AGENTS.md
      - AGENTS.md
    summary: 在现有learning测试和Rust CI中增加target-main ancestry合同及真实squash反例
    verify: 合同解析真实baseline；feature-only SHA在feature HEAD通过但对target main及squash
      result失败；CI使用PR base SHA或push before SHA并fetch完整历史
  - id: verify-review-and-deliver
    files:
      - echo-agent/.echo-semantic
    summary: 补齐repair、verification、rereview证据并通过follow-up PR交付到main
    verify: 完整本地门禁、strict、continuity、94/94 Issue对账和review通过；follow-up PR
      CI全绿并merge；merged main再次通过合同、strict和continuity；23个resolved Issue关闭
artifact_id: plan:7188c52c-e999-4b9b-ad87-6db613d1435b
design_revision: null
---
## Context

- PR #117已squash merge为`d492c676d1bf0744452d96a6960124546ed3fff9`，10/10远端CI成功。
- 合并后strict稳定失败，因为baseline仍绑定只存在于原任务分支祖先链的`7ba1f122`。
- 独立review确认一次性rebind只能修当前实例；仓库强制squash，而strict只检查PR HEAD ancestry，必须增加target-main稳定祖先门禁。
- Issue #118已建立。用户已明确授权提交、PR和merge，本Plan包含follow-up远端交付。

## Approach

- baseline YAML与正文改绑已合并main `d492c676`，保持原source digest和闭合对象不变。
- 在`echo-agent-learning`新增语义baseline合同，复用`serde_yaml_ng`结构化解析和`tempfile`构造Git反例；不手写YAML解析器。
- 合同默认本地校验`origin/main`，CI显式传入PR base SHA或push before SHA；feature-only revision即使是PR HEAD祖先，也必须因不是target main祖先而失败。
- Rust CI checkout完整历史，使merge-base检查可重算；顶层和child AGENTS同步记录该CI职责。
- 完整门禁和独立review通过后提交、推送、创建follow-up PR；远端CI全绿后squash merge，在真实main复验再关闭Issue。

## Global Constraints

- 不修改Echo Semantic verifier、runtime API、Cargo manifest、SDK、contracts、examples或正式产品docs。
- 复用现有`serde_yaml_ng`与`tempfile`，不新增依赖。
- target revision必须由调用方显式环境或本地`origin/main`/`main`解析，不能猜当前feature HEAD。
- 顶层和child AGENTS的CI职责说明必须同步。
- 原22个resolved Finding Issue只能在merged main strict恢复后关闭；Issue #118只在follow-up进入main后关闭。
- 不删除分支或执行其它cleanup。

## Files

- Modify: `echo-agent/.echo-semantic` — baseline ancestor、Finding #118、repair/verification/rereview证据与刷新后的source snapshot。
- Create: `echo-agent/echo-agent-learning/tests/semantic_baseline_contract.rs` — target-main ancestor合同与squash反例。
- Modify: `echo-agent/.github/workflows/rust-ci.yml` — 提供PR base/push before revision及完整Git历史。
- Modify: `echo-agent/AGENTS.md` — 记录semantic baseline CI职责。
- Modify: `AGENTS.md` — 与child同步semantic baseline CI职责。

## Reuse

- `.echo-semantic/baseline.md` — 唯一repository baseline与strict ancestor合同。
- `echo-agent-learning/Cargo.toml` — 已有`serde_yaml_ng`和`tempfile`依赖。
- `.github/workflows/rust-ci.yml` — 现有Linux learning test job，避免第二测试执行器。
- Echo Semantic strict/continuity verifier — 保留fail-closed HEAD ancestor校验，由target-main合同补足pre-merge边界。

## Todos

### repair-baseline-ancestry

requirements:
- 用户要求完成提交、PR和merge
- Issue #118及post-merge strict失败证据

interfaces:
- consumes: merged main d492c676、baseline source_snapshot和strict ancestor check
- produces: 有效main ancestor及可追踪Finding/repair Evidence

steps:

1. 写入Finding #118并更新baseline YAML与正文。
   verify: base_revision精确等于d492c676，原content_digest与closure保持不变。
   expected: 当前baseline描述与真实squash主线一致。

### add-squash-aware-contract

requirements:
- Issue #118防回归关闭条件
- 仓库强制squash merge与CI职责边界

interfaces:
- consumes: baseline frontmatter、target main revision、PR/push事件SHA和Git ancestor relation
- produces: learning contract及CI target revision输入

steps:

1. 新增结构化baseline读取和target ancestry检查，并以临时Git仓库构造feature commit与squash result。
   verify: feature SHA对feature HEAD为ancestor，但对target main和squash result均不是ancestor；target main自身通过。
   expected: 同类错误在PR合并前可确定失败。

2. 给Rust CI的learning tests提供完整历史和真实target revision，并同步两份AGENTS职责说明。
   verify: PR使用pull_request.base.sha，push使用before，缺显式环境的本地运行解析origin/main/main。
   expected: 本地、PR和main push共享同一target-main稳定祖先合同。

### verify-review-and-deliver

requirements:
- 仓库完整合并门禁、语义门禁与一Finding一Issue生命周期

interfaces:
- consumes: ancestry修复、持久合同、Issue #118和PR #117 continuity基线
- produces: verification Evidence、rereview Audit、follow-up merged main和关闭后的23个Issue

steps:

1. 运行focused合同、完整仓库门禁、strict、change-evidence、b21 continuity和94/94 Issue对账并完成独立review。
   verify: 所有命令exit 0，review Critical/Important/Minor为0。
   expected: 候选可提交且防回归边界真实可执行。

2. 提交、推送、创建follow-up PR，等待远端CI全绿后squash merge。
   verify: PR状态MERGED且真实main包含baseline修复与持久合同。
   expected: ancestry修复及pre-merge保护进入远端main。

3. 在merged main重跑learning contract、strict和continuity，再关闭Issue #118及PR #117已交付的22个resolved Finding Issue。
   verify: merged main semantic gates通过，23个已交付Issue CLOSED，71个open Finding Issue保持OPEN。
   expected: 语义状态、远端交付和Issue生命周期一致。

## Decisions

- 根因是baseline只验证feature HEAD ancestry，未约束target main稳定性；一次性rebind不是完整修复。
- 持久合同属于echo-agent仓库CI集成，不修改或复制Echo Semantic verifier。
- 不新增依赖或第二测试runner，复用learning test与现有CI。
