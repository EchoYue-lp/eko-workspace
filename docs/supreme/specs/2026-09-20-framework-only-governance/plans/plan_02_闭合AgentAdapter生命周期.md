---
schema_version: 4
slug: framework-only-governance/agent-lifecycle-foundation
outcome:
  summary: "Close every non-A2A Agent adapter through stop-admission,
    cancellation, terminal wait, persistent settlement, and awaited resource
    release, then resolve #36."
  acceptance:
    - ACP, Headless, Channel, and ReactAgent ownership paths implement and
      document the canonical close order without Drop-spawned cleanup or a
      second Turn terminal.
    - Focused tests cover success, close failure, caller cancellation, retryable
      retained ownership, in-flight work, and stale/late completion where
      applicable.
    - "Repair, verification, independent rereview, ./scripts/verify.sh,
      applicable feature checks, and remote CI pass before #36 closes."
out_of_scope:
  - Audit, remove, or redesign A2A.
  - "Implement MCP construction cleanup tracked by #55 unless a non-A2A adapter
    cannot close without it."
  - Change SDK Host, EKO, language SDK, or website code.
  - Create a framework-wide close state machine or another Turn terminal.
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
design_sections:
  - ref: design.md § 生命周期与终态
    digest: sha256:97cd871fe84b3def0aaab45d642b65b73bae15fd74d4a48fc4b0c985b9428289
  - ref: design.md § 副作用 owner
    digest: sha256:2d47cdeb4b31968c657e6c4fed657bbec3ee0c9a51dc6bcaaf9338b9a7277783
  - ref: design.md § 异常和边界场景
    digest: sha256:da1c1ece9b3e9dcd204419109ff38e9b7803ad3f6abe30ab5e30586383afe28f
  - ref: design.md § Finding 生命周期与验收
    digest: sha256:94fae8475b6fdfd6b4ba9eb041c47aa9543dff6fca16996186658e0b904b79d9
delivery_ref: docs/supreme/specs/2026-09-20-framework-only-governance/plans/delivery-map.md#agent-lifecycle-foundation
todos:
  - id: reconcile-current-adapter-owners
    summary: Recheck current ACP, Headless, Channel, and ReactAgent close paths and
      keep one explicit owner from admission fence through retryable settlement.
    files:
      - echo-agent/src/acp
      - echo-agent/src/headless.rs
      - echo-agent/src/agent/react/mod.rs
      - echo-agent/echo-integration/src/channels
      - echo-agent/src/channels.rs
      - echo-agent/docs/adr/0066-agent-adapter-close-ownership.md
    acceptance:
      - Every accepted resource remains reachable until close succeeds or
        returns typed debt; no non-A2A production path treats Drop, EOF,
        transport stop, or log output as settlement.
  - id: close-cancellation-and-retry-gaps
    summary: Repair any current non-A2A cancellation, in-flight terminal wait, or
      retained-close retry gap using the existing Agent, Run, MessageHandler,
      and transport owners.
    files:
      - echo-agent/src/acp/adapter.rs
      - echo-agent/src/acp/runtime.rs
      - echo-agent/src/acp/session.rs
      - echo-agent/src/headless.rs
      - echo-agent/echo-integration/src/channels/manager.rs
      - echo-agent/echo-integration/src/channels/session.rs
      - echo-agent/echo-integration/src/channels/types.rs
    acceptance:
      - Cancellation and close races converge on one terminal and the same owner
        can be awaited or retried after an interrupted or failed close.
  - id: synchronize-lifecycle-contracts
    summary: Align public API comments, bilingual lifecycle documentation,
      executable examples, and semantic evidence with the non-A2A close
      contract.
    files:
      - echo-agent/docs/en
      - echo-agent/docs/zh
      - echo-agent/echo-agent-learning/examples
      - echo-agent/.echo-semantic/findings/finding.agent-adapter-close-settlement.md
      - echo-agent/.echo-semantic/evidence/evidence.agent-adapter-close-settlement-repair.md
      - echo-agent/.echo-semantic/evidence/evidence.agent-adapter-close-settlement-verification.md
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/maps
    acceptance:
      - The Finding contains no A2A blocker, all public close responsibilities
        are documented, and executable examples await close where they own
        resources.
  - id: verify-and-close-agent-lifecycle
    summary: "Run the final framework lifecycle validation and independent rereview
      on the integrated snapshot, deliver to main, and close #36."
    files:
      - echo-agent/tests
      - echo-agent/echo-agent-learning/tests
      - echo-agent/.echo-semantic/baseline.md
    acceptance:
      - The final main revision satisfies the repository gates and the GitHub
        Issue state matches a resolved canonical Finding.
artifact_id: plan:20cfc03c-3cdb-448d-ba28-ee58dd5ff749
lifecycle: ready
---
