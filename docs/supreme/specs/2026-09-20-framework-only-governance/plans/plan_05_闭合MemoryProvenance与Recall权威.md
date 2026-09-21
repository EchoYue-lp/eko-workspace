---
schema_version: 4
slug: framework-only-governance/memory-provenance-authority
outcome:
  summary: "Establish memory provenance, Draft-to-Active transitions, and one recall authority for #76."
  acceptance:
    - Both pre-compaction memory writers and the framework's automatic memory
      producers preserve verifiable source role, exact evidence, and trust.
      Model output or tool-derived text cannot assert user approval or enter
      Active/Hot memory merely through confidence, source labels, or repetition.
    - Draft candidates remain durable and inspectable. An explicit framework
      promotion operation commits Draft to Active only against the exact current
      content, provenance, and revision; stale, cancelled, failed, or restarted
      attempts cannot publish an unintended Active record.
    - Automatic context recall and layered recall/search share one eligibility
      rule for warm and hot entries. Draft, unverified legacy records, and
      Superseded items cannot enter the model prompt; approved Active and
      previously approved Archived memories retain their documented behavior.
      Recall telemetry cannot undo a concurrent status or provenance change.
    - Focused tests cover normal extraction and approval, mixed user/assistant/
      tool evidence, malformed or forged provenance, failure, cancellation,
      restart, stale approval/recall, and File/SQLite or other applicable Store
      options. Public API, bilingual docs, examples, and semantic evidence are
      aligned before full framework gates, applicable feature checks, independent
      rereview, PR/main CI, signed squash delivery, and closure of #76.
out_of_scope:
  - SDK Host, EKO CLI/GUI/TUI policy, language SDKs, website, and A2A.
  - "Background Review task ownership, deadline, and terminal settlement in #38;
    only its memory candidate boundary is in this outcome."
  - A second memory Store, agent-generated human approval, or a product-specific
    review UI; preserve public Store and File/SQLite options.
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
design_sections:
  - ref: design.md § 按事实划分唯一权威
    digest: sha256:74546583db427504373034f8aff6f4f65d109f84e651f10230cbaf495e6cbdd2
  - ref: design.md § 可恢复持久化
    digest: sha256:43c8e0afbbec1f7ae6a79e7b815c77ef404d64331abf3a48b87b564b89768c6c
  - ref: design.md § 异常和边界场景
    digest: sha256:da1c1ece9b3e9dcd204419109ff38e9b7803ad3f6abe30ab5e30586383afe28f
  - ref: design.md § 公共合同一致性
    digest: sha256:2adf014c1953b46750887bb0268b6e35f6459d16ef38407136d957cfa4bf7e8e
  - ref: design.md § Finding 生命周期与验收
    digest: sha256:94fae8475b6fdfd6b4ba9eb041c47aa9543dff6fca16996186658e0b904b79d9
delivery_ref: docs/supreme/specs/2026-09-20-framework-only-governance/plans/delivery-map.md#memory-provenance-authority
todos:
  - id: bind-memory-provenance-to-one-store-authority
    summary: Extend the existing typed MemoryMeta/HotEntryMeta contract with
      source evidence and trust, and make MemoryLayerManager the only durable
      Draft-to-Active transition owner with an exact stale fence.
    files:
      - echo-agent/echo-core/src/memory/types.rs
      - echo-agent/echo-state/src/memory/typed_store.rs
      - echo-agent/src/evolution/layer.rs
      - echo-agent/src/evolution/security.rs
      - echo-agent/docs/adr
    acceptance:
      - File and SQLite Store round trips and hot/warm moves retain provenance;
        failed or cancelled transitions reconcile through the existing memory
        operation journal. Legacy records remain readable for review without
        becoming implicit approval, and stale approval cannot overwrite newer
        content, status, or provenance.
  - id: make-automatic-producers-proposal-only
    summary: Bind exact user/assistant/tool excerpts to pre-compaction LLM and
      heuristic candidates, and keep trigger, remember-tool, and Background
      Review candidates within the same Draft/approval policy.
    files:
      - echo-agent/src/agent/react/run/context.rs
      - echo-agent/src/agent/react/run/phases/compact.rs
      - echo-agent/src/memory_promoter.rs
      - echo-agent/src/evolution/triggers.rs
      - echo-agent/src/evolution/background_review.rs
      - echo-agent/src/tools/builtin/memory.rs
    acceptance:
      - A fabricated model quote, wrong source role, tool-text claimed as a
        user preference, missing evidence, or secret-bearing evidence cannot
        publish Active; repeated compaction preserves approved values and
        deduplicates Draft candidates without downgrading a reviewed record.
  - id: converge-recall-and-lifecycle-admission
    summary: Route automatic injection and layered tool search through one
      provenance-aware recall rule; fence telemetry, hot promotion, and
      Dreaming revival against stale Draft/approval state.
    files:
      - echo-agent/src/evolution/recall.rs
      - echo-agent/src/evolution/layer.rs
      - echo-agent/src/evolution/dreaming.rs
      - echo-agent/src/agent/react/run/context.rs
      - echo-agent/src/tools/builtin/memory.rs
      - echo-agent/echo-state/src/memory/typed_store.rs
    acceptance:
      - The same record is either eligible or hidden in automatic context,
        recall, search_memory, and hot lookup; a concurrent demotion, delete,
        or Draft edit cannot be reactivated by recall-count updates or a
        stale search result.
  - id: align-public-contract-and-deliver
    summary: Update the public facade and framework documentation/examples,
      focused contracts, ADR, Capability Map, and repair/verification/independent
      rereview evidence for #76, then deliver the outcome to main.
    files:
      - echo-agent/src/lib.rs
      - echo-agent/docs/en
      - echo-agent/docs/zh
      - echo-agent/echo-agent-learning
      - echo-agent/.echo-semantic/findings/finding.pre-compaction-memory-trust-provenance.md
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
      - echo-agent/.echo-semantic/audits
    acceptance:
      - Executable examples and bilingual docs describe Draft review,
        provenance, Active approval, and recall exactly as public APIs behave;
        strict semantic verification, full framework gates, relevant feature
        matrix, independent rereview, remote CI, and post-merge main evidence
        support closing #76.
artifact_id: plan:f7dbe9fe-977f-4ff2-8655-42d55b496c1b
lifecycle: ready
---

## Notes

The existing `MemoryLayerManager` owns durable prepare/project/audit/settle
and `MemoryRecaller` already ranks warm memories. The repair strengthens those
owners instead of adding a parallel memory database or a second recall loop.
MemorySource describes the extraction mechanism, not the underlying speaker's
trust or a user's approval. A model's confidence score cannot substitute for
exact evidence or a caller-owned promotion decision.

The industry boundary is consistent: [Claude Code's memory documentation](https://code.claude.com/docs/en/memory)
separates human-written instructions from agent-written auto memory and treats
both as context rather than enforcement. [LangGraph's memory documentation](https://docs.langchain.com/oss/python/langgraph/add-memory)
separates thread checkpoints from cross-session long-term memory, and its
[Store contract](https://docs.langchain.com/oss/python/langgraph/stores) keeps
the long-term records under one namespaced authority. For this framework,
untrusted and model-derived observations remain reviewable Draft records until
an explicit, revision-bound promotion; approved memory may be recalled as
reference context, never as a permission or task-state authority.
