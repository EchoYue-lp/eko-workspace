---
schema_version: 4
slug: framework-only-governance/permission-hard-floor
outcome:
  summary: "Enforce one non-bypassable permission and isolation floor across Hook
    protected paths, readonly custom tools, and sandbox fallback for #60, #81,
    and #83."
  acceptance:
    - Hook and custom-tool extension points can only preserve or tighten the
      canonical Agent permission decision; protected paths and readonly
      classification cannot be bypassed.
    - A command with minimum_isolation never executes through a lower-isolation
      fallback and returns a typed failure when no qualifying executor exists.
    - Normal, denied, fallback, cancellation, and relevant stale-policy tests
      plus repair/verification/independent rereview, ./scripts/verify.sh,
      applicable feature checks, and remote CI pass before all three Issues
      close.
out_of_scope:
  - Gate direct user-operated terminal, file-picker, or MCP configuration with
    Agent automation permission modes.
  - Add EKO-specific approval UI, reviewer policy, worktree policy, or resource
    semaphores to the framework.
  - "Implement scoped approval receipts #37, Plan-mode evidence #70, stream
    failure typing #82, or Guard direction #57."
  - Change SDK Host, EKO, language SDK, website, or A2A code.
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
design_sections:
  - ref: design.md § 权限下限
    digest: sha256:71823c404da1a35d790d5755199edb0d387ff3dc4efd4415a0709e5ae1f8052a
  - ref: design.md § 系统边界
    digest: sha256:216cbd249ce1aaa7e575631c848f440fe4633144d1a48417b4ff0ee2c3463b5b
  - ref: design.md § 异常和边界场景
    digest: sha256:da1c1ece9b3e9dcd204419109ff38e9b7803ad3f6abe30ab5e30586383afe28f
  - ref: design.md § Finding 生命周期与验收
    digest: sha256:94fae8475b6fdfd6b4ba9eb041c47aa9543dff6fca16996186658e0b904b79d9
delivery_ref: docs/supreme/specs/2026-09-20-framework-only-governance/plans/delivery-map.md#permission-hard-floor
todos:
  - id: make-protected-path-floor-unconditional
    summary: Route Hook Allow and rewrite outcomes through the canonical
      protected-path and permission minimum before any automated tool effect.
    files:
      - echo-agent/src/agent/react/run/pipeline.rs
      - echo-agent/echo-core/src/tools/permission.rs
      - echo-agent/echo-orchestration/src/human_loop/service.rs
      - echo-agent/.echo-semantic/findings/finding.hook-protected-path.md
    acceptance:
      - A Hook cannot authorize a protected mutation denied by the canonical
        permission service, while explicit deny and stricter Hook outcomes
        remain effective.
  - id: classify-custom-tools-under-readonly
    summary: Apply one typed tool-effect classification to standard and custom tool
      registration so readonly and Plan surfaces hide or reject every Write or
      Execute tool.
    files:
      - echo-agent/src/agent/react/builder.rs
      - echo-agent/src/agent/react/mod.rs
      - echo-agent/echo-core/src/tools/mod.rs
      - echo-agent/echo-execution/src/tools.rs
      - echo-agent/echo-tools/src/registry.rs
      - echo-agent/.echo-semantic/findings/finding.readonly-tools-custom-registration-bypass.md
    acceptance:
      - Custom mutation tools cannot become model-visible or executable through
        a readonly Agent, and read-only custom tools retain normal behavior.
  - id: enforce-sandbox-minimum
    summary: Separate preferred fallback from required minimum isolation and fail
      closed before execution when no executor satisfies the command minimum.
    files:
      - echo-agent/echo-core/src/sandbox.rs
      - echo-agent/echo-execution/src/sandbox/policy.rs
      - echo-agent/echo-execution/src/sandbox/manager.rs
      - echo-agent/.echo-semantic/findings/finding.sandbox-minimum-isolation.md
    acceptance:
      - Fallback selection never returns an executor below minimum_isolation;
        unavailable and cancellation paths return typed results without
        launching the command.
  - id: synchronize-and-verify-permission-contract
    summary: "Update public docs, examples, semantic evidence, and combined
      regressions, then deliver the integrated permission floor and close #60,
      #81, and #83."
    files:
      - echo-agent/docs/en
      - echo-agent/docs/zh
      - echo-agent/echo-agent-learning/examples
      - echo-agent/tests
      - echo-agent/.echo-semantic/maps/map.tool-permission-sandbox.md
      - echo-agent/.echo-semantic/rules/rule.permission-effect-order.md
      - echo-agent/.echo-semantic/evidence
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/baseline.md
    acceptance:
      - One documented effect-order contract covers all three paths, all
        repository gates pass on the delivered main revision, and GitHub Issue
        states match resolved Findings.
artifact_id: plan:2ac24a86-687f-402d-84f5-695871f5448d
lifecycle: ready
---
