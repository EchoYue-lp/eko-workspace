---
schema_version: 4
slug: framework-only-governance/checkpoint-plan-authority
outcome:
  summary: "Retire AgentCheckpoint.current_plan as a second recovery authority and
    resolve #42 on the canonical runtime state path."
  acceptance:
    - ReactAgent checkpoint save, reset, and hydration no longer publish or
      restore a parallel plan state; a historical nonempty current_plan cannot
      override the revisioned Task graph after restart or an identity switch.
    - Existing public checkpoint and File/SQLite Store options remain usable for
      framework consumers; focused tests cover ordinary recovery, legacy
      nonempty values, cancellation or failed hydration, restart, and stale A ->
      B -> A identity transitions without weakening transcript settlement.
    - "Repair, verification, independent rereview, framework full gates,
      applicable feature matrix, public API and bilingual documentation/example
      checks, remote CI, and post-merge main evidence close #42."
out_of_scope:
  - Create another Plan or Todo store, CRUD surface, validator, or Task
    execution loop.
  - Remove public AgentCheckpoint or the File/SQLite RuntimeStateStore options
    merely because one embedding application does not use them.
  - "Change SDK Host, EKO, language SDK, website, A2A, or unrelated memory
    provenance work in #76."
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
design_sections:
  - ref: design.md § 按事实划分唯一权威
    digest: sha256:74546583db427504373034f8aff6f4f65d109f84e651f10230cbaf495e6cbdd2
  - ref: design.md § 可恢复持久化
    digest: sha256:43c8e0afbbec1f7ae6a79e7b815c77ef404d64331abf3a48b87b564b89768c6c
  - ref: design.md § 公共合同一致性
    digest: sha256:2adf014c1953b46750887bb0268b6e35f6459d16ef38407136d957cfa4bf7e8e
  - ref: design.md § Finding 生命周期与验收
    digest: sha256:94fae8475b6fdfd6b4ba9eb041c47aa9543dff6fca16996186658e0b904b79d9
delivery_ref: docs/supreme/specs/2026-09-20-framework-only-governance/plans/delivery-map.md#checkpoint-plan-authority
todos:
  - id: retire-react-plan-recovery
    summary: Remove the private ReactAgent plan_state capture, reset, and restore
      path so checkpoint data cannot act as Task or Plan authority.
    files:
      - echo-agent/src/agent/react/mod.rs
      - echo-agent/src/agent/react/run/context.rs
      - echo-agent/src/agent/snapshot.rs
      - echo-agent/echo-orchestration/src/tasks/revisioned.rs
    acceptance:
      - Only the existing TaskRevisionService may commit the revisioned task
        graph; a restored checkpoint cannot set an independent executable or
        user-visible plan, and new ReactAgent checkpoints do not propagate a
        prior current_plan.
  - id: preserve-legacy-checkpoint-decoding
    summary: Keep legacy nonempty checkpoint records readable without reactivating
      their plan text across File and SQLite recovery, CAS, or transcript
      acknowledgement.
    files:
      - echo-agent/src/state/mod.rs
      - echo-agent/src/state/file.rs
      - echo-agent/src/state/sqlite.rs
      - echo-agent/src/agent/react/run/stream_channel.rs
    acceptance:
      - File and SQLite tests prove restart and A -> B -> A stale-value
        isolation; normal messages, skills, working directory, and pending
        transcript settlement retain their existing guarantees.
  - id: align-checkpoint-contract-and-evidence
    summary: "Synchronize public checkpoint comments, bilingual persistence
      documentation, repository examples, tests, and semantic Finding evidence
      with the canonical Task authority, then deliver the independent #42
      outcome."
    files:
      - echo-agent/docs/en/03-memory.md
      - echo-agent/docs/zh/03-memory.md
      - echo-agent/docs/en/13-chat.md
      - echo-agent/docs/zh/13-chat.md
      - echo-agent/docs/en/41-persistence-concepts.md
      - echo-agent/docs/zh/41-persistence-concepts.md
      - echo-agent/echo-agent-learning
      - echo-agent/.echo-semantic/findings/finding.checkpoint-current-plan-orphan-authority.md
      - echo-agent/.echo-semantic/maps/map.context-memory.md
      - echo-agent/.echo-semantic/evidence
      - echo-agent/.echo-semantic/audits
    acceptance:
      - "Public text and executable examples no longer claim AgentCheckpoint
        plan recovery, the Finding has repair/verification/rereview evidence on
        the same framework snapshot, and #42 closes only after main delivery."
artifact_id: plan:a8df56af-9a64-43c1-8787-cbd15e5aea21
lifecycle: ready
---
