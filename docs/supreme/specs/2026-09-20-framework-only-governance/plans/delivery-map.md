---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/2026-09-20-framework-only-governance/design.md
outcomes:
  framework-finding-closure:
    ships: "Reclassify #55, #106, #46, #84, and #99 against the framework-only
      completion contract, close every framework-complete Finding, and preserve
      each real framework gap as an explicit repair outcome."
    depends_on: []
  mcp-construction-close-owner:
    ships: "Replace Drop-spawned MCP construction cleanup with an awaited, retryable
      owner and close #55 without relying on SDK Host delivery."
    depends_on:
      - framework-finding-closure
  scheduler-store-cache-authority:
    ships: "Make public CronTaskStore mutations and SchedulerRunner cache
      observation converge on one durable definition authority, then close #84."
    depends_on:
      - framework-finding-closure
  agent-lifecycle-foundation:
    ships: "Close every non-A2A Agent adapter through stop-admission, cancellation,
      terminal wait, persistent settlement, and awaited resource release, then
      resolve #36."
    depends_on:
      - framework-finding-closure
  checkpoint-plan-authority:
    ships: "Retire AgentCheckpoint.current_plan as a second recovery authority and
      resolve #42 on the canonical runtime state path."
    depends_on:
      - agent-lifecycle-foundation
  memory-provenance-authority:
    ships: "Establish memory provenance, Draft-to-Active transitions, and one recall
      authority for #76."
    depends_on:
      - checkpoint-plan-authority
  background-review-settlement:
    ships: "Give Background Review an owned task, receipt, cancellation, deadline,
      and durable terminal settlement for #38."
    depends_on:
      - memory-provenance-authority
  effect-cleanup-owner:
    ships: "Assign awaited cleanup ownership for Artifact, Sandbox, and Worktree
      effects and resolve #47."
    depends_on:
      - background-review-settlement
  permission-hard-floor:
    ships: "Enforce one non-bypassable permission and isolation floor across Hook
      protected paths, readonly custom tools, and sandbox fallback for #60, #81,
      and #83."
    depends_on:
      - framework-finding-closure
  permission-terminal-and-approval:
    ships: "Type stream-construction failure, unify scoped approval receipts, close
      Plan mode evidence, and normalize Guard direction and error policy for
      #82, #37, #70, and #57."
    depends_on:
      - permission-hard-floor
  observation-delivery-contract:
    ships: "Separate business facts, recoverable delivery debt, and best-effort
      telemetry across #61, #103, #104, and #58."
    depends_on:
      - agent-lifecycle-foundation
  provider-structured-output-foundation:
    ships: "Use one model facts resolver and canonical ReAct structured-output
      request with strict schema and provider-visible tokenizer calibration for
      #68, #77, #96, #97, and #100."
    depends_on:
      - framework-finding-closure
  framework-contract-finalization:
    ships: "Reconcile evolution namespace, typed channel attachments, executable
      tool pipeline contracts, public facade, docs, and examples for #53, #40,
      and #101."
    depends_on:
      - effect-cleanup-owner
      - permission-terminal-and-approval
      - observation-delivery-contract
      - provider-structured-output-foundation
design_revision: sha256:f0af8f42a79da626c0e88082dbfa8c305ba09e213286aa5f1ee8ab66ff0f32ff
---
