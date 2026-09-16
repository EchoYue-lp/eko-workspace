---
schema_version: 4
slug: transcript-projection-settlement/framework-projection-authority
outcome:
  summary: Framework provides atomic idempotent transcript projection, durable
    pending settlement, and closed compact/finalize/recovery/clear lifecycle
    behavior.
  acceptance:
    - ConversationStore epoch/projection/delete, RuntimeStateStore
      CAS/retirement, and the single persistence coordinator are merged to
      echo-agent main with no unresolved crash-cut, admission, delete replay, or
      terminal-path gap.
    - A ConversationStore configuration without RuntimeStateStore is rejected
      before model or persistence side effects, while the fully unconfigured
      path preserves existing behavior.
    - "Framework complete local merge gate, public API feature matrix, required
      remote CI, semantic strict/change-evidence, and independent rereview all
      pass; Issue #106 remains open for the dependent SDK and cross-repository
      outcomes."
out_of_scope:
  - Modify echo-agent-sdk protocol, Host, or language clients in this outcome.
  - "Close Issue #106 before the dependent SDK and cross-repository closure
    outcomes merge."
  - Add EKO-specific UI projections, SQLite requirements, a second Outbox, or a
    general background queue.
design_ref: docs/supreme/specs/2026-09-16-transcript-projection-settlement/design.md
design_sections:
  - ref: design.md § 目标行为
    digest: sha256:e21d2e06a75f50f5ea22573741962c349dcdfa1ca6d508a340566d7372b9abaa
  - ref: design.md § 系统边界
    digest: sha256:c302f96cd844006d1e57a845644901ae9fae4a2e2ec810812c03e5ee7d06a7a7
  - ref: design.md § 核心结构与数据流
    digest: sha256:f2c744ba60551040f623b609108538ef8a1af5ebb60bd61731c2445133914f68
  - ref: design.md § 异常和边界场景
    digest: sha256:2593d4854938784867692f598ac3891b68d65141ddf15db1e64b2c6c6d5b046f
  - ref: design.md § 复用与实现约束
    digest: sha256:69a47855848976ce09a43b52352077a04c0a8b355a422a72d86cbe6614aaae6f
  - ref: design.md § 验收标准
    digest: sha256:607b5dc1346e167c2235b35b8d19cdfd25511425f491c5f94fc1c1938824e914
delivery_ref: docs/supreme/specs/2026-09-16-transcript-projection-settlement/plans/delivery-map.md#framework-projection-authority
todos:
  - id: atomic-store-authorities
    summary: Add the framework public atomic ConversationStore
      epoch/projection/delete contract and RuntimeStateStore
      compare-and-save/retirement contract, with identical File and SQLite
      authority semantics and managed legacy-mutator fences.
    files:
      - echo-core/src/error.rs
      - echo-core/src/memory/mod.rs
      - echo-core/src/memory/conversation.rs
      - echo-state/src/memory/mod.rs
      - echo-state/src/memory/conversation.rs
      - echo-state/src/memory/file_conversation.rs
      - echo-state/src/memory/sqlite_conversation.rs
      - src/state/mod.rs
      - src/state/file.rs
      - src/state/sqlite.rs
    acceptance:
      - File and SQLite fault/concurrency tests prove stable operation receipts,
        atomic epoch acquisition, no lost updates, generation/scope CAS
        retirement, durable delete manifests, new-incarnation isolation, and
        legacy mutators cannot bypass managed authority.
  - id: durable-persistence-coordinator
    summary: Persist one canonical pending projection per runtime generation and
      route admission, compact, every terminal path, recovery, clear, delete,
      timeout, observation, and reconcile through one framework coordinator.
    files:
      - echo-core/src/agent/mod.rs
      - echo-core/src/agent/event_envelope.rs
      - src/agent/snapshot.rs
      - src/agent/config.rs
      - src/agent/mod.rs
      - src/agent/react/mod.rs
      - src/agent/react/builder.rs
      - src/agent/react/subsystems/memory.rs
      - src/agent/react/run/context.rs
      - src/agent/react/run/direct.rs
      - src/agent/react/run/react_loop.rs
      - src/agent/react/run/stream_channel.rs
      - src/agent/react/run/stream_macros.rs
      - src/agent/react/run/phases
      - src/trace/mod.rs
    acceptance:
      - Failure-first tests cover store-only admission rejection with no side
        effects in streaming, direct, direct-chat, and multimodal runs; direct
        resume_from_state_store, load_messages, force_checkpoint, plus cold and
        warm reconcile before Hydrated; pre-compact fail-closed; Completed,
        NoResponse, cancel, consumer disconnect, provider/tool failure,
        guard/intervention stop, and max-iteration ordering; bounded
        unknown-outcome retry; exact clear; and product delete replay after
        every crash cut. Settlement observation precedes the single terminal,
        and event-envelope schema/trajectory tests cover the new public event.
  - id: framework-contract-evidence
    summary: Document the framework authority and breaking managed-store contract,
      refresh semantic objects, and deliver the framework outcome while
      preserving the explicit SDK follow-up obligation.
    files:
      - docs/adr
      - docs/en
      - docs/zh
      - .echo-semantic
    acceptance:
      - "Documentation and examples applicability checks, semantic verification,
        independent review, merge gates, and repository delivery evidence
        describe the exact merged behavior and keep the SDK outcome and Issue
        #106 open."
artifact_id: plan:6ff30090-a6c9-405c-acbe-450f2bc93e7d
lifecycle: ready
---
