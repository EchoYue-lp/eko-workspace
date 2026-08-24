# S-X-01: Cross-Repository Review Synthesis (ZCode-ds)

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Synthesis date: 2026-08-12
> `echo-agent` commit reviewed: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (= baseline 9b0e0fa)
> `echo-agent-cli` commit reviewed: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (= baseline b3b2e81)
> Worktree state: both repositories clean; HEAD verified identical to the baseline by S-X-01 V01-01/V03-01
> Inputs: all 10 X-* task reports (zcode-ds), S-FW-01 (framework-review.md), S-APP-01 (application-review.md), shared README.md / REPORTING.md / TASKS.md (S-X-01 card), zcode-ds README.md
> Deliverable contract: TASKS.md S-X-01 — canonical cross-repository P1 summary, final capability-placement ruling, per-contract consistency rulings, the permission-boundary ruling, contradiction reconciliation and stale check. Validations: V01 (boundary-gate completeness), V02 (canonical duplicate merge), V03 (adapter loss/authority recheck) — see validation reports.

## 1. Scope And Method

This synthesis consumes the ten completed cross-repository task reports
(X-TSK-01, X-EVT-01, X-TOL-01, X-PLG-01, X-MEM-01, X-AUT-01, X-SRF-01,
X-STA-01, X-BND-01, X-INV-01) and the two phase syntheses (S-FW-01, S-APP-01)
as declared by the S-X-01 task card. Per REPORTING.md Synthesis Rules it
merges duplicates under one canonical ID with backlinks, resolves the
cross-phase alias the phase syntheses delegated to S-X-01, classifies
producer/consumer and verdict/root-cause pairs as non-duplicates, preserves
minority/uncertain conclusions as open questions, and runs a stale-commit
check against the shared baseline. No `codex/` or `zcode-glm/` material was
read; no shared file was modified; no source file was modified (read-only).

**Bottom line: the two repositories are structurally one system.** The
framework/app/adapter split is correct for every core concept, there is
exactly one live authority per semantic (task graph, events, tools,
subagents, memory, permission, plugin lifecycle, identity), the principal
EKO adapters are thin and lossless on the reachable paths, and all six
product invariants from the review (surface parity, truthful terminals,
identity continuity, one task graph, no parallel semantics, local security
boundary) hold at the architecture layer — each with a bounded set of
enforcement/wiring defects at the boundaries. **Six new P1 findings** (see
section 2), 13 new P2, and 12 new P3 were filed by the X phase; zero P0
anywhere in the cross-repository review. The dominant cross-cutting theme:
the framework owns the contracts, EKO owns the policy, and the defects are
almost all at the seams — adapters that drop typed classes
(X-EVT-01-P1-01/P1-02), wireings that were never made (X-AUT-01-P1-01),
lifecycle primitives that were never given counterparts (X-STA-01-P1-01),
and a read-back that fabricates data instead of erroring (X-TSK-01-P3-01).

## 2. Canonical Cross-Repository P1 Summary

Census: the X-phase reports file **31 new findings = 6 P1 + 13 P2 + 12 P3**.
The zcode-ds README's preliminary summary claims "7 new P1"; its own
enumeration lists exactly the 6 IDs below — the README count is an
off-by-one preliminary slip, superseded by this census (recorded in
V02-01). Every row below is canonical; no X P1 collides with any F/A P1
(V02-01).

| Canonical ID | Layer | Tight anchor | Finding (one line) | Cross-phase relation |
|---|---|---|---|---|
| X-EVT-01-P1-01 | framework (envelope + subagent executor) with adapter consequence | `echo-agent/src/agent/subagent/executor.rs:138-147,1182,1400-1402`; `echo-core/src/agent/event_envelope.rs:134-140` | Typed cancel/timeout class is lost at the envelope boundary — every raw `Err` is normalized to `Error{"agent_stream": ...}`, so `subagent_status_from_error` is bypassed and mid-stream cancelled/timed-out subagents persist and render as `failed` (unless the token race fires first); the durable `SubagentReleaseRecord` persists the same lie | EKO-side consumer half of F-RCT-03-P1-02 (producer gap); X-EVT-01-P3-01 is the wire-variant dead arm |
| X-EVT-01-P1-02 | adapter (envelope) with application wire/reducer consequence | `event_envelope.rs:134-140`; `echo-agent-cli/src/tauri/commands/chat.rs:30-112`; `web-frontend/src/stores/chatStore.ts:354-362`, `hooks/chatEventHandler.ts:140-150` | Chat-turn timeouts have no typed terminal at any layer — the envelope collapses the typed `Timeout` classes, `ChatEvent` has no timeout variant, and the TS reducer ends timed-out turns at `'completed'` (canonical A-SRF-03-P1-02 chain) | User-visible arm of the same producer gap; pairs with A-SRF-03-P1-02 / A-CHAT-01-P1-01 (error class) |
| X-AUT-01-P1-01 | adapter (EKO subagent factory wiring; framework opt-in default is the mechanism) | `echo-agent-cli/echo-agent-app-core/src/infra.rs:881-966,968-1010` (zero permission wiring); `echo-agent/src/agent/react/mod.rs:535`; `snapshot.rs:857-862` | TaskRuntime writer/readonly subagents execute automation entirely outside the permission boundary — no `PermissionService`, no provider, no mode; protected-path checks (.git/.ssh/.env) never run; `run_code`/web/browser/git/data tools execute with `Ok(None)` in every mode | Resolves A-HITL-01's delegated open question ("subagents inherit neither provider nor dispatcher — no service at all"); same builder as A-TOOL-01-P1-01 |
| X-STA-01-P1-01 | application (cascade scope) with a framework API gap | `echo-agent-cli/src/tauri/commands/conversations.rs:585-640`; `echo-agent/src/state/mod.rs:246-273` (trait, no delete); `~/.eko/runtime_state/<id>/` written every turn (snapshot.rs:570-631) | Conversation deletion leaves the full unfiltered runtime transcript, plan DAG, and workspace-scope copy on disk — the `RuntimeStateStore` trait has no delete API and no EKO path cleans `runtime_state/<id>/`; a reused id would silently restore deleted context | Framework/application boundary decision for S-RDM-01 (add `delete_conversation` to the trait) |
| X-INV-01-P1-01 | framework | `echo-agent/echo-tools/src/pdf.rs:225-227` | `parse_pdf_date` byte-slices a lossy-decoded String at fixed offsets — deterministic panic (reproduced, exit 101) on any non-ASCII/invalid-UTF-8 PDF date; aborts the whole agent run (no `catch_unwind` barrier) | Same defect class as F-EXT-02-P1-01 / F-EXT-03-P1-03 / Q-STA-01-P1-01 (panic family) |
| X-INV-01-P1-02 | framework | `echo-agent/src/eval/runner.rs:728` | `extract_number_near_key` slices the original text at a lowercased-text byte offset — deterministic panic (reproduced, exit 101) on multilingual LLM output (any key followed by ~17+ CJK chars); eval numeric metrics unusable for non-ASCII text | Same panic family; fix belongs with the UTF-8 byte-slice lint pass |

### 2.1 Overlap with the B / F / A phases (folded, not re-filed)

The X phase re-verified and re-anchored **every established finding in its
areas** rather than re-filing them (each X report carries a canonical
cross-reference matrix and V05 stale checks). The canonical overlaps
consumed by this synthesis:

- **F phase (framework)**: 49 canonical P1s (S-FW-01). Cross-repo-relevant
  families: silent failure / terminal integrity (F-RCT-02..05,
  Q-FLT-01-P1-01/02, F-LLM-01/03, F-CMP-01-P1-01/02, F-MEM-01-P1-01,
  F-OPS-01-P1-01, F-REL-01-P1-01), mock invisibility cloak
  (F-TST-01-P1-01/02, Q-TST-01-P1-01..03), detached execution
  (F-SUB-02-P1-01/02, F-MAG-01-P1-01, F-INT-02-P1-01..03), approval
  boundary (F-HITL-01-P1-01/02/03), panic family (X-INV-01-P1-01/02,
  Q-STA-01-P1-01). The X phase consumes F-TSK-01..03, F-SKL-01, F-PLG-01,
  F-CMP-01, F-CTX-01, F-MEM-01, F-RCT-03/04, F-EXT-01..03, F-SUB-01/02,
  F-HITL-01, F-SEC-01, F-INT-01 as dependency facts.
- **A phase (application)**: 25 canonical P1s (S-APP-01). X re-anchors
  A-TSK-01..06 (file authority, crash recovery, worktree),
  A-STATE-01 (store roots), A-CHAT-01 / A-SRF-03 (terminal lie),
  A-MEM-01 (hot-memory refresh), A-HITL-01 (approval surfaces),
  A-CFG-01 (workspace scope), A-SRF-01/02/04 (surface parity),
  A-OUT-01/A-EVO-01/A-INT-01 (management surfaces), A-TOOL-01 (writer
  subagent).
- **B phase (baseline)**: B-ARCH-01-P2-01 explicitly delegated "produce
  the authority map" to X-BND-01 (consumed as X-BND-01-P2-02's origin);
  B-BASE-01's manifest/lock inventory is the base for X-INV-01's
  no-SQLite and relative-path verdicts.
- **Q phase (dynamic)**: Q-E2E-01-P1-01..03 are scenario-level verdicts of
  canonical defects (Q-E2E-01-P1-02 = A-TOOL-01-P1-01; Q-E2E-01-P1-03 =
  F-OPS-01-P1-01); Q-FLT-01/Q-STA-01 added the two panic reproductions and
  the stream-truncation evidence the X phase cites.

## 3. Capability Placement — Final Ruling

Source of truth: X-BND-01 (the final placement map with the 32-row
deletion-target matrix D1-D32). Verdict, endorsed by this synthesis and by
S-FW-01/S-APP-01:

### 3.1 Framework-correct (no movement)

Task graph (`TaskSpec`/`TaskRevisionService`/`PlanValidator`/
`RuntimeDagExecutor`/`RevisionedTaskStore` trait + tools), subagent
(`SubagentRegistry`/`SubagentExecutor`), approval (`PermissionService`
pipeline), memory/context (`Store` 4+2 impls, `MemoryLayerManager`,
`ContextManager` + 6 compressors), skills (`SkillRegistry`), plugin
(`PluginRegistry`/`PluginIntegrator`/`PluginLifecycle`), MCP
(`McpManager`), workflow engine, intent router, diff tool, retry policy,
trace `RunStore`. All independently reusable; zero EKO policy leaked into
the framework (X-BND-01 V01).

### 3.2 Application-correct (no movement)

Task projections + capability catalog + ownership wave, subagent loader
policy, `HitlDispatcher` + leaf providers, instruction protocol +
`UnifiedMemory`, skills-hub marketplace, plugin reload transaction,
worktree merge/branch policy, all surface adapters/sinks/commands,
per-surface services. All EKO-owned (X-BND-01 V01).

### 3.3 Adapter-correct (thin, lossless, no scheduling/state authority — with the filed boundary defects)

`EkoRevisionedTaskStore` (load/CAS only — re-verified at
revisioned_adapter.rs:30-47 by S-X-01 V03-01), `EkoRuntimeDagController`
(filters the wave, never recomputes readiness), `EkoTaskToolPolicy`,
`EkoSubagentPromptCompiler`, `apply_compressor`, the envelope adapter,
the four chat sinks. The two adapter-loss classes are defects inside
correct placement: (a) fabricated data on the read-back
(X-TSK-01-P3-01, store.rs:694-696); (b) typed-class collapse at the
envelope (X-EVT-01-P1-01/P1-02). The one adapter-boundary *state* defect is
X-AUT-01-P2-01 (process-global permission-handler slot shared by primary +
pool), a shared-state defect, not a second authority.

### 3.4 Semantic-duplicate residue (condensed from the D1-D32 matrix)

No cross-repository live second engine exists for any concept; the residue
is (a) framework-internal dead/parallel APIs, (b) a small application-side
copy set, (c) inert schema fields. Condensed deletion targets (full matrix
in X-BND-01; every row carries its canonical finding, authority, and
deletion impact):

| Group | Targets (D rows) | Canonical sources |
|---|---|---|
| Legacy task surface | D1 `TaskManager`/`TaskExecutor`/`TaskHooks`/`VerifierFactory`; D2 inert `execution_mode: sequential`; D3 dead `refresh_in_flight` | F-TSK-01-P3-01, F-TSK-03, F-TSK-02-P2-02/P3-01 |
| Dead subagent/team/handoff surface | D4 7 dead `SubagentDefinition` fields; D5 `ContextBuilder`/`OutputSchema`/`MemoryScope`/`isolated.rs`; D6 `TeamCoordinator`/`TeamRunner`/mailbox; D7 `src/handoff/` | F-SUB-01-P2-01/P2-03, F-SUB-02-P2-03, F-MAG-01-P2-01 |
| Dead approval surface | D8 `run/approval.rs` + `process_steps`; D9 `TauriHumanLoopHandler` parallel transport; D10 `IpcAuth`/`IpcPermission` | F-HITL-01-P2-03, A-HITL-01-P2-01/P2-04, A-TOOL-01-P3-01, A-SRF-02-P3-01 |
| Dead context/LLM surface | D11 `ContextAssembler`/`ContextSelector`; D12 `ProviderAdapter`/`AdapterClient` | F-CTX-01-P2-03, F-LLM-01-P2-03 |
| Tool/retry/download duplication | D13 `ToolRiskClassifier`; D14 `ToolManager::result_cache`; D15 3 of 4-5 URL-download tools; D16 duplicate `parse_page_range` | F-EXT-01-P3-02/P3-01, F-EXT-03-P2-01/P3-06 |
| Skill/plugin duplication | D17 3 of 5 frontmatter parsers; D18 hub inline binary probe; D19 second plugin data-dir computation | F-SKL-01-P3-01/P3-02, F-PLG-01-P3-03 |
| Framework dead modules | D20/D23 EKO `WorkflowDef`/`WorkflowStep` + `save_trace` second ledger; D22 `retry_llm_call` backoff; D29 `NotebookTracker` | F-WFL-01-P3-08, A-OBS-01-P2-01, F-REL-01-P2-01, F-NBK-01-P2-01 |
| Application copies | D21 three diff engines + duplicate `DiffHunk`/`DiffLine` + dead `DiffViewer.tsx`; D24 second `export_conversation`; D25 `panels.rs` worktree helpers; D26 three project-root resolvers; D27 `safe_segment` copy; D28 dead `reflect_on_session`; D30 `web_config` + dual `DEFAULT_CONTEXT_WINDOW`; D31 duplicate tool-event projection producer; D32 second frontend auth-check | A-PROJ-01-P2-03, A-OUT-01-P2-01, A-TSK-05-P2-04, X-BND-01-P2-01/P3-01, A-EVO-01-P3-02, A-CFG-01-P2-03/P2-05, A-SRF-02-P2-01/A-FE-02-P2-01, A-FE-03-P3-05 |

Boundary-gate completeness: all 10 X-* reports carry the five gate answers
(generic-mechanism / product-policy / adapter / duplicate-search /
migration-deletion); 4 of 10 express the deletion answer in the findings'
Direction fields instead of a dedicated row — substance complete, format
inconsistent (V01-01). Deletion execution order, per-repo ownership, and
acceptance criteria belong to S-RDM-01 (X-BND-01-P2-02).

## 4. Consistency Rulings (per contract area)

| Contract | X task | Ruling |
|---|---|---|
| Task graph | X-TSK-01 | **Single authoritative revisioned TaskRun graph confirmed**: one `TaskRevisionService` + `PlanValidator` + `RuntimeDagExecutor` + one store CAS (`EkoRevisionedTaskStore` implements only load/compare_and_commit); EKO projection field-lossless and status-lossless on reachable paths; zero forbidden CRUD (`todo_write`/`plan_*`); legacy `TaskManager`/`TaskExecutor` production-unreachable (re-verified by S-X-01 V03-01: one production `RuntimeDagExecutor::new` at EKO executor.rs:1645). One boundary defect: fabricated-Pending read-back inside the A-TSK-01-P2-01 crash window (X-TSK-01-P3-01). |
| Event lifecycle | X-EVT-01 | **Identity and ordering conformant; terminal truth does not.** One `EventEnvelope` contract with deterministic id/sequence consumed by all surfaces; sequences contiguous until the first terminal; the one-terminal invariant holds at the envelope boundary and the subagent store. Fails at: typed cancel/timeout class loss (P1-01, P1-02), the GUI wire dropping envelope identity/sequence (P2-01), chat-turn terminal absent from persistence (P2-02), dead `ChatEvent::Cancelled` (P3-01). All consumer halves align with the F-RCT-03-P1-02 producer fix. |
| Tool contract | X-TOL-01 | **Normal path conformant 1:1 at all four layers** (schema, 7-way `ToolFailureCategory`, SHA-256 artifact checksum, byte-cursor paging). Kill paths collapse: `cancel()` records bare `Cancelled` with no failure and no partial-side-effect warning on every kill path (P2-01; EKO half of F-RCT-04-P1-02/P2-02). Two projection defects: artifact `output_bytes` label (P3-01), failed-streamed-tool error text lost (P3-02). |
| Plugin seam | X-PLG-01 | **Reversible and source-scoped for skills/hooks/Subagents/monitors/LSP/themes/styles with one runtime authority and a dynamically verified transactional rollback** (plugin_runtime tests green). Not failure-safe for: MCP ownership (P2-01, destructive name-keyed replacement with no restore), dual-registry unload consistency (P3-01), pool descriptor freshness (P3-02). Ten archived canonical findings (F-SKL-01/F-PLG-01/A-PLG-01) re-anchored current. |
| Memory/context | X-MEM-01 | **Single instruction authority, projections survive repeated compression exactly once, content-hash dedup; the two task invariants do not hold today** (A-MEM-01-P1-01 stale hot memory; F-CMP-01-P1-01/P1-02 unbounded/over-limit context). Two dormant parallel mechanisms filed: `memory_context_suffix` (P2-01 — must be deleted, not re-wired), dual MEMORY.md parsers (P2-02), plus a misleading pool-refresh doc (P3-01). |
| Permission boundary | X-AUT-01 | **Separation holds on the primary/pool and direct-user paths; fails on subagent automation** (P1-01, section 5 below). Also: process-global permission-handler slot defeats per-conversation isolation (P2-01), CLI `/permission` does not propagate to the pool (P3-01). Zero over-gating residue on direct-user paths; four indexed secret gaps at EKO outbound boundaries (A-OBS-01-P1-02, F-OPS-01-P2-01/04, F-SEC-01-P3-11). |
| Surface parity | X-SRF-01 | **Shared core holds for all six entry classes** (one `drive_chat`, one TaskRuntime, one pool, one `PreparedUserTurn`); parity fails at the management/control layer. Three new gaps: REPL browser management (P2-01), channel task-run management (P2-02), REPL/channel steer (P3-01); 21 canonical findings re-anchored. TUI is the completeness baseline (S-APP-01 parity matrix). |
| Identity continuity | X-STA-01 | **Steady-state identity generation is single-sourced and stably keyed per class; restart survival is broken by the filed recovery defects and by one new deletion-cascade gap** (P1-01 runtime_state never cleaned), store-root divergence at exit (P2-01), `"message:task"` grouping contamination (P2-02), client-only id generation (P3-01). |
| Repository invariants | X-INV-01 | **Five of six invariants hold with zero violations** (zero `worker` terms, zero CLI SQLite, zero parallel CRUD, macro-level panic safety, all 16 manifest paths relative). UTF-8 safety fails on exactly two live framework sites (P1-01 pdf, P1-02 eval, both reproduced exit 101) plus one latent slice (P3-01). |

## 5. Permission Boundary Ruling (X-AUT-01-P1-01)

**Ruling: the TaskRuntime writer/readonly Subagents — the executor of every
Implementation/Debugging task and every forked subagent — execute
automation entirely outside the permission boundary, and this must be the
first permission-area fix in the roadmap.**

The evidence chain (verified at source by S-X-01 V03-01): the subagent
factories (`infra.rs:881-1010`) contain zero permission wiring (grep
returns only comments); the framework default is opt-in
(`permission_service: None`, react/mod.rs:535) with an allow-when-no-
service fallback (`snapshot.rs:857-862` → `Ok(None)`); the plan-mode
blocklist (`snapshot.rs:227-236`, `pipeline.rs:1004-1018`) covers only
write tools/shell/delete_file, so `run_code`, web/network, browser, git,
and data tools execute with no decision in every permission mode; the
protected-path checks (.git/.ssh/.env) live inside the service and never
run. Physical containment (plan mode, worktree checkout, sandbox floor)
bounds the data-loss surface, which is why this is P1 and not P0, but the
product's stated automated-action control (the mode matrix) silently does
not govern a major automation surface.

Fix direction (adapter-side first, framework-side optional): give
writer/readonly subagents the shared `PermissionService` + current mode at
factory construction, mirroring `agent_pool.rs:928-932`, with a fail-closed
empty-dispatcher provider policy for background runs; or, framework-side,
make the no-service fallback deny instead of allow for
Write/Execute/Network/Sensitive tools. **Must be coordinated with
A-TOOL-01-P1-01** (the same builders are currently silently read-only via
plan mode; permission wiring must not be masked by that collision) — both
fixes touch `build_writer_subagent_agent`/`build_readonly_subagent_agent`.
Regression targets: subagent `run_code`/web call in default mode must
produce an approval request or denial, never `Ok(None)`; `.git/config`
writes must deny on subagents.

## 6. Contradiction Reconciliation And Stale Check

### 6.1 Reconciled contradictions

1. **Cross-phase alias F-EXT-01-P1-01 ↔ A-TOOL-01-P1-01** (writer subagent
   read-only). S-FW-01 kept the F ID canonical "for this synthesis";
   S-APP-01 merged under the A ID. **S-X-01 rules: canonical =
   `A-TOOL-01-P1-01`** (the defect's removal point is EKO `infra.rs:963`;
   verified in V02-01); `F-EXT-01-P1-01` retained as a cross-phase alias
   backlink. The roadmap counts this defect once.
2. **X-phase P1 count**: zcode-ds README "7 new P1" vs the reports'
   6 P1 IDs. The reports are authoritative (their own enumeration is the
   census); the README "7" is an off-by-one preliminary summary
   (README not modified per task rules). Canonical count: **6 new P1**.
3. **Pause-in-wave priority** (A-TSK-03-P2-01 vs A-TSK-04-P1-01): already
   resolved to P1 by S-APP-01 with the smallest validation re-opened; this
   synthesis confirms and does not reopen (the X phase re-anchored it
   current in X-TSK-01 V05).
4. **Gate green vs doctest red** (Q-FW-01 vs Q-FW-02 V14): resolved by
   S-FW-01 (different scopes: `--all-targets` excludes doc tests; a gate
   blind spot recorded as open question for S-QA-01). No X-phase claim
   depends on the doctest phase; no conflict here.

### 6.2 Non-duplicate pairs (must land together, not merged)

Producer/consumer: F-HITL-01-P1-03 ↔ A-HITL-01-P1-03 (wildcard contract);
F-RCT-03-P1-02 → X-EVT-01-P1-01/P1-02/P3-01 (cancel terminal producer →
consumer arms); F-RCT-03-P1-01 → X-EVT-01-P2-01 (drops → undetectability).
Verdict/root-cause: Q-E2E-01-P1-02 → A-TOOL-01-P1-01; Q-E2E-01-P1-03 →
F-OPS-01-P1-01. Related-fix (same surface, distinct invariants):
X-AUT-01-P1-01 ↔ A-TOOL-01-P1-01 (infra.rs builders); X-TSK-01-P3-01 ↔
A-TSK-01-P2-01 (shared crash-window fix); X-STA-01-P2-01 ↔ A-STATE-01-P1-01
(exit_workspace roots); X-SRF-01-P3-01 ↔ A-SRF-04-P1-01 (REPL/channel turn
handles). Full classification in V02-01.

### 6.3 Open questions preserved (minority/uncertain, not erased)

- X-STA-01-P1-01's framework-side question: add `RuntimeStateStore::delete_conversation` vs application-side directory removal — a framework/application boundary decision (AGENTS.md framework gate applies: the trait is a framework capability menu item; adding a delete mirrors `ConversationStore::delete_conversation`).
- X-SRF-01-P2-01/P2-02/P3-01 and the research-workbench placement
  (A-OUT-01-P2-04): documented product decisions may close a parity row
  without a fix.
- X-EVT-01-P1-01's `timeout_secs = 0` residual trigger: EKO-side
  reachability of the zero-override not confirmed end-to-end.
- X-MEM-01-P2-02 trigger (frontmatter-less MEMORY.md) not exercised
  dynamically; impact argued from the enforcement loop.
- X-PLG-01-P3-02 staleness impact conditional on pooled agents exposing
  progressive-disclosure tools.
- X-TSK-01-P3-01 is a deterministic trace, not dynamically reproduced
  (needs the torn-projection fixture; Q-FLT-02 owns it).
- Dynamic multi-surface/restart confirmations remain `not_run`
  (environmental; Q-E2E-01/Q-FLT-02 own them).

### 6.4 Stale check

Both repositories verified at the baseline commits with clean worktrees by
S-X-01 V01-01 and V03-01 (`git rev-parse HEAD` = 9b0e0fa / b3b2e81; zero
uncommitted changes). Every X-phase finding and every anchor re-verified by
this synthesis (single-executor/validator construction, adapter method
surface, P3-01 default, envelope normalization, permission wiring absence,
trait surface, store roots, `"message:task"` fallback, cancel signature,
panic sites) is current at the reviewed commits. **Zero stale findings.**
All X-* reports' own stale triggers remain in force for any post-baseline
commit.

## 7. Handoff

- **S-RDM-01 may rely on**: the canonical 6-P1 table (section 2) with
  backlinks; the placement ruling and condensed D1-D32 matrix (section 3);
  the alias ruling (A-TOOL-01-P1-01 canonical) and the non-duplicate pairs
  (section 6.2) so each defect is counted and bundled exactly once; the
  permission-boundary ruling (section 5) as the first permission-area item;
  the consistency rulings (section 4) as the per-area acceptance frame.
  Suggested cross-repo merge order (per README.md priorities): (1)
  correctness/data integrity — the panic pair (X-INV-01-P1-01/P1-02) is
  small and testable; X-STA-01-P1-01 cascade; X-TSK-01-P3-01 with
  A-TSK-01-P2-01; (2) authority/terminal convergence — the F-RCT-03-P1-02
  fix with its EKO halves (X-EVT-01-P1-01/P1-02/P3-01) and A-CHAT-01-P1-01;
  (3) the subagent factory bundle (X-AUT-01-P1-01 + A-TOOL-01-P1-01 +
  F-EXT-01-P1-01 alias) and X-AUT-01-P2-01; (4) surface parity gaps
  (X-SRF-01-P2-01/P2-02/P3-01 with A-SRF-04-P1-01); (5) tool kill-path
  classification (X-TOL-01-P2-01 with F-RCT-04-P1-02); (6) D1-D32 deletion
  batches per the X-BND-01 matrix.
- **S-QA-01 may rely on**: the validation matrix below (all three S-X-01
  validations executed, statuses recorded); the X-phase validation
  inventories are all complete (every X task's matrix has a report per
  row).
- Reports to read: this report + S-X-01 V01-01/V02-01/V03-01; all 10 X-*
  task reports; S-FW-01, S-APP-01 (their merge tables are the backlinks
  for every canonical ID cited here).
- Stale triggers for this synthesis: any commit on either repo after
  9b0e0fa/b3b2e81 touching the anchors in sections 2-5 (event_envelope,
  subagent executor/status mapping, chat.rs wire, infra.rs builders,
  state.rs store roots, conversations.rs cascade, store.rs load path,
  pdf.rs/eval runner, McpManager connect, capability unregister, unified
  memory refresh sites); the alias ruling becomes stale if the writer
  builder's plan-mode line changes; the D1-D32 matrix row-by-row stale
  triggers are the owning findings' own.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Boundary-gate completeness — every cross-repo finding classified with all five gate answers | yes | passed (substance complete; 4/10 reports carry the deletion answer in findings instead of a row) | [V01-01](../validations/S-X-01/V01-01.md) |
| V02 | Canonical duplicate merge — X census (6 P1/13 P2/12 P3), alias ruling (A-TOOL-01-P1-01), non-duplicate pair classification, README count reconciliation | yes | passed | [V02-01](../validations/S-X-01/V02-01.md) |
| V03 | Adapter loss/authority recheck — single live executor/validator/store-CAS; adapter method surfaces; adapter-loss and wiring-gap anchors at source | yes | passed | [V03-01](../validations/S-X-01/V03-01.md) |

All required validations executed with immutable reports; no validation is
pending; no source file was modified (read-only).
