---
schema_version: 4
slug: framework-only-governance/finding-closure
outcome:
  summary: "Reclassify #55, #106, #46, #84, and #99 against the framework-only
    completion contract, close every framework-complete Finding, and preserve
    each real framework gap as an explicit repair outcome."
  acceptance:
    - "#106, #46, and #99 are resolved and closed with framework-main evidence;
      #55 and #84 remain open only for concrete framework defects found by
      current independent rereview."
    - Repair, verification, and independent rereview evidence agree with the
      final source digest, and strict semantic snapshot plus change-evidence
      verification passes.
    - Framework focused tests, ./scripts/verify.sh, and the applicable
      independent feature matrix pass before delivery to main.
out_of_scope:
  - "Implement the MCP construction cleanup owner required to close #55."
  - "Implement the SchedulerRunner public-store/cache authority repair required
    to close #84."
  - Change SDK Host, EKO, language SDK, website, or A2A code.
  - "Implement #36 or the permission hard-floor outcomes."
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
design_sections:
  - ref: design.md § 范围与非目标
    digest: sha256:6ee631b16434e7a648224b901c6b20d32337809a206905157ded2e930b765b6c
  - ref: design.md § Finding 生命周期与验收
    digest: sha256:94fae8475b6fdfd6b4ba9eb041c47aa9543dff6fca16996186658e0b904b79d9
  - ref: design.md § 总体验收标准
    digest: sha256:ec2f7a27d354cd9251b75367bab3972a81ef7fc7a7166ae976dd3a1c32f6f964
delivery_ref: docs/supreme/specs/2026-09-20-framework-only-governance/plans/delivery-map.md#framework-finding-closure
todos:
  - id: freeze-framework-only-dispositions
    summary: Bind each of the five Issues to current main code, tests, ADRs,
      semantic evidence, and an explicit framework-only Close or Keep-open
      disposition.
    files:
      - echo-agent/.echo-semantic/findings/finding.extension-cleanup-settlement.md
      - echo-agent/.echo-semantic/findings/finding.transcript-projection-settlement.md
      - echo-agent/.echo-semantic/findings/finding.diagnostic-persistence-failure-visibility.md
      - echo-agent/.echo-semantic/findings/finding.scheduler-cache-delivery.md
      - echo-agent/.echo-semantic/findings/finding.task-subagent-attempt-link.md
      - echo-agent/docs/adr/0042-scheduler-occurrence-authority.md
      - echo-agent/docs/adr/0049-mcp-transport-close-settlement.md
      - echo-agent/docs/adr/0053-trace-audit-persistence-visibility.md
      - echo-agent/docs/adr/0056-durable-transcript-projection-settlement.md
      - echo-agent/docs/adr/0058-task-claim-subagent-attempt-control.md
    acceptance:
      - Every disposition names the framework owner, current main revision,
        applicable scenarios, semantic evidence, and residual risk without an
        SDK/CLI/website blocker.
  - id: repair-closure-contract-drift
    summary: Correct framework documentation and current semantic closure records
      for the code-complete transcript, diagnostic, and exact-attempt Findings.
    files:
      - echo-agent/docs/en/27-tracing.md
      - echo-agent/docs/zh/27-tracing.md
      - echo-agent/docs/en/26-multi-agent.md
      - echo-agent/docs/zh/26-multi-agent.md
      - echo-agent/src/agent/subagent/events.rs
      - echo-agent/.echo-semantic/evidence
      - echo-agent/.echo-semantic/audits
    acceptance:
      - Tracing, Subagent identity, repair, verification, and rereview text
        matches current public APIs and the framework-only boundary.
  - id: preserve-real-framework-gaps
    summary: "Keep #55 and #84 open with only their current framework defects and
      connect those defects to separate delivery-map outcomes."
    files:
      - echo-agent/.echo-semantic/findings/finding.extension-cleanup-settlement.md
      - echo-agent/.echo-semantic/findings/finding.scheduler-cache-delivery.md
      - echo-agent/.echo-semantic/maps/map.extension-lifecycle.md
      - echo-agent/.echo-semantic/maps/map.task-subagent-workflow.md
      - echo-agent/docs/adr/0042-scheduler-occurrence-authority.md
      - echo-agent/docs/adr/0049-mcp-transport-close-settlement.md
    acceptance:
      - "#55 identifies the Drop-spawned construction cleanup owner gap and #84
        identifies public store mutation/cache divergence; neither retains
        external repository prerequisites."
  - id: verify-deliver-and-close
    summary: Run current framework verification, obtain an incremental independent
      rereview, deliver the closure changes to main, and close only Issues whose
      framework contracts are complete.
    files:
      - echo-agent/.echo-semantic/baseline.md
      - echo-agent/.echo-semantic/behaviors
      - echo-agent/.echo-semantic/rules
      - echo-agent/.echo-semantic/assets
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/findings
    acceptance:
      - All required local and remote gates pass on the delivered revision,
        GitHub closure comments cite framework evidence, and the final Issue
        states match canonical Finding states.
artifact_id: plan:26f9cac3-3526-4710-b225-02362f58df70
lifecycle: ready
---
