# F-EXT-01: Tool contract, registry, schema, and artifacts

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Is the generic Tool contract typed, cancellable, paginated, and capable of
bounded model output plus complete artifacts?

## Scope

- `echo-core/src/tools/mod.rs` (Tool trait, ToolFailure, ToolResult,
  ToolParameters/ParamValue/ToolCallParams, ToolContext, ToolExecutionConfig,
  ToolVisibilityState — full read), `echo-core/src/tools/pagination.rs`,
  `artifact.rs`, `permission.rs`, `skill.rs` (full reads).
- `echo-execution/src/tools.rs` (ToolManager registry, execution, concurrency,
  retry gating, streaming, budget metrics — full read), `risk.rs` (full read).
- Root `src/tools/mod.rs` facade (WRITE_TOOLS/READ_TOOLS classification),
  `src/tools/builtin/` (think/answer/memory/spawn_task/check_task/
  human_in_loop/agent_dispatch), `echo-tools/src/registry.rs`
  (register_all_tools / register_readonly_tools).
- Macro schema/deserialization path (`echo-macros/src/derive_tool.rs:330-415`,
  `lib.rs:236-290`), `echo-core/src/llm/types.rs:605-614` (ToolDefinition).
- Bounded-output/spill pipeline (`src/agent/snapshot.rs:926-1060`,
  `src/agent/react/run/pipeline.rs` ExecuteStage/TruncationStage/PlanModeStage,
  `execution.rs:270-310`), `echo-tools/src/files/artifact.rs`
  (ReadArtifactTool).
- Cross-repo registration consumers: `echo-agent-cli/echo-agent-app-core/src/
  agent_pool.rs`, `infra.rs` (subagent builders), `tasks/task_runtime/executor.rs`
  (run_writer_subagent), `src/agent/react/mod.rs` memory tool registration.

## Out Of Scope

- Domain tool correctness (shell/file/git/web…) → F-EXT-02, F-EXT-03.
- Approval/HITL and permission-mode policy internals → F-HITL-01 (permission
  types here reviewed only for the contract).
- Subagent executor lifecycle (only the tool-visibility/plan-mode interface is
  traced here) → F-SUB-01.
- TaskRuntime DAG/plan machinery → F-TSK-* (only the writer-subagent tool
  surface is referenced).
- Unified retry migration mechanics → F-REL-01 (cross-referenced, V05).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-EXT-01 card), `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-CORE-01` (complete), `B-ARCH-01`
  (complete), `B-REF-01` (complete), and `F-REL-01` (complete — required for
  V05).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  tool-contract claims (lines 98, 115).

## Layering Decision

- Generic mechanism (framework, `echo_core`/`echo_execution`): Tool trait +
  context + failure classification, pagination contract, artifact writer,
  ToolManager registry/execution, macro schema generation — all correctly
  placed in framework crates.
- EKO product policy (application): AgentPool shared-ToolManager pattern,
  per-agent memory-layer-manager tool registration, writer-subagent
  plan_mode wiring, WRITE_TOOLS-driven UI/read-before-edit policy usage.
- Adapter boundary: none new; the ToolContext build in `pipeline.rs:495-512`
  is the framework-internal adapter from run snapshot to tool context.
- Duplicate search terms (both repositories): `fn name() -> &str` literals
  (101 sites), `result_cache`, `ToolRiskClassifier`/`ToolRiskCategory`,
  `WRITE_TOOLS`/`READ_TOOLS`/`is_write_tool`/`is_read_tool`,
  `allows_automatic_retry`, `retry_delay_ms`, `set_plan_mode`, `plan_mode`,
  `ToolOutputArtifactWriter`/`persist_tool_output`, `PageRequest`/
  `read_artifact`/`next_cursor`. Results: 4 intentional same-name pairs
  (memory tools), 1 dead cache, 1 dead risk classifier, 3rd+4th retry backoff
  copies, 3 parallel "is this tool a writer" classifications (trait
  risk_level, WRITE_TOOLS consts, ToolRiskClassifier).

## Current Path

Verified data flow: LLM tool call → react loop (`phases/think.rs:399`
`tools_for_llm()`) → `execution.rs:270-310` builds `ToolExecutionContext`
(plan_mode from agent config) → 13-stage `ToolExecutionPipeline`
(approval/read-before-edit/skill/plan-mode/execute/truncation/callbacks) →
`ExecuteStage` builds `ToolContext` (`pipeline.rs:495-512`: working_dir,
ids, call_id, output_artifacts, visibility, cancel, trace_sink,
delegation_policy) → `ToolManager::execute_tool(_stream)_with_context`
(semaphore → timeout → retry gated on `ToolFailure::allows_automatic_retry`)
→ `Tool::execute_with_context` → `ToolResult` → TruncationStage spills
oversized output to `ToolOutputArtifactWriter` and replaces it with a bounded
preview + `read_artifact` recovery path (`snapshot.rs:926-1060`).
Collection-returning tools (search_memory, …) paginate via
`PageRequest`/`PageInfo` (opaque fingerprint-bound cursors). Registry is a
DashMap shared via Arc; `get_openai_tools` is sorted + version-cached for
provider prefix caching; AgentPool shares one manager across agents
(`agent_pool.rs:882`); memory tools are re-registered per agent with the
agent's own layer manager (`agent_pool.rs:671-672,923`).

## Findings

### F-EXT-01-P1-01: Writer subagents are silently read-only — `set_plan_mode(true)` in the writer builder collides with the framework plan-mode tool filter

- Priority: P1
- Confidence: medium (static chain fully verified; no end-to-end dynamic run)
- Layer: application (wiring) — framework behavior is by design
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:963`
  (`subagent.set_plan_mode(true)` at the end of `build_writer_subagent_agent`,
  whose own log line says "full write tools"); framework filter
  `echo-agent/src/agent/snapshot.rs:282-285` (`!self.plan_mode ||
  !is_write_tool(...) && != shell && != delete_file` in `tools_for_llm`);
  `snapshot.rs:152` (`plan_mode: config.plan_mode`); `snapshot.rs:227-236`
  (same filter in `available` set); `src/agent/react/run/pipeline.rs:1000-1018`
  (PlanModeStage blocks write tools when `ctx.plan_mode`);
  `src/agent/react/run/execution.rs:293` (`plan_mode: self.config.plan_mode`);
  semantics pinned by framework test `src/agent/snapshot.rs:1610-1648`
  (plan_mode=true ⇒ only `read_file` visible). Framework commit `2266d0f`
  (2026-07-12) introduced the plan-mode filter; the CLI writer wiring landed
  in Sprint 9 commit `420f062` (2026-07-01) — the filter post-dates the
  wiring.
- Reachability: TaskRuntime routes `is_writer_task` to `run_writer_subagent`
  (`tasks/task_runtime/executor.rs:2205-2213`) → `delegate_to_agent_with_
  prompt_payload` → `SubagentExecutor::execute_agent_streaming`
  (`src/agent/subagent/executor.rs:1148-1170`) drives the subagent's own
  react loop; no code anywhere resets `plan_mode` to false (grep
  `set_plan_mode(false)` across both repos: none). Writer tasks therefore run
  with write tools invisible to the model and blocked at execution.
- Expected invariant: a "writer" subagent with full write tools in an isolated
  worktree can actually write (comment `infra.rs:877-885`: "OMIT
  `.readonly_tools()` → full tool set (write capability)").
- Observed behavior: `plan_mode=true` removes every `WRITE_TOOLS`-listed tool,
  `shell`, and `delete_file` from the LLM-visible surface and blocks them in
  PlanModeStage; writer subagents can only read/think/finalize.
- Impact: Implementation/Debugging writer tasks (TaskRuntime complex runs)
  cannot modify the worktree — a major capability failure of the flagship
  complex-run feature, silent since framework 2266d0f (2026-07-12).
- Root cause: `set_plan_mode(true)` was copied into the writer builder from
  the read-only builder (identical tail code, `infra.rs:963` vs `:1040`) when
  plan mode did not yet filter tools; the framework's plan-mode filter landed
  later and flipped the meaning of the flag for this agent without any
  compatibility check.
- Direction: remove `subagent.set_plan_mode(true)` from
  `build_writer_subagent_agent` (keep it in `build_readonly_subagent_agent`);
  add a framework test that a `plan_mode` agent cannot see write tools and an
  EKO test that a writer subagent's LLM-visible toolset contains
  `write_file`/`shell`/`run_code`; if read-only planning is desired for
  writer tasks, gate it on the task's own contract instead of the tool
  surface.
- Regression validation: `cargo test -p echo_agent --lib` plan-mode visibility
  tests stay green; a CLI fixture that builds a writer subagent and asserts
  `tools_for_llm()`-equivalent visibility contains write tools; an end-to-end
  mock-LLM writer task that calls `write_file` and asserts the worktree file
  exists.
- Validation reports: [V02-01](../validations/F-EXT-01/V02-01.md),
  [V04-02](../validations/F-EXT-01/V04-02.md)

### F-EXT-01-P1-02: AgentPool shares one ToolManager and re-registers per-agent memory tools under the same four names — silent overwrite routes all pooled agents' memory calls to one agent's layer manager

- Priority: P1
- Confidence: medium
- Layer: application (AgentPool pattern) with framework root cause
- Evidence: silent overwrite `echo-execution/src/tools.rs:529-532`
  (`tools.insert(name, tool)` — no collision detection, no warning);
  shared manager `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:96,127`
  (SharedResources.tool_manager extracted from primary agent), `:882`
  (`agent.set_tool_manager(tm.clone())` replaces the per-agent manager,
  `echo-agent/src/agent/react/mod.rs:1442-1445`); per-agent memory runtime
  registration `agent_pool.rs:671-672` (`install_memory_store` +
  `install_memory_layer_manager(Arc::new(layer_manager))` inside
  `apply_memory_store_inner` iterating every pool agent), `:923`
  (new-agent path), `:650-657` (layer_manager built per agent with its own
  evolution observer); framework registration `react/mod.rs:1083-1110`
  registers `LayeredRememberTool`/`LayeredRecallTool`/`LayeredSearchMemoryTool`/
  `LayeredForgetTool` — same names as the legacy set (`builtin/memory.rs:49,
  154, 262, 531, 353, 776, 437, 607`).
- Reachability: `apply_memory_store` / `apply_memory_store_global` are called
  on workspace switch and startup (`state.rs:1013,1160`); every pooled agent
  (parallel conversations, background tasks) shares the registry; iteration
  order over the agent map is nondeterministic, so the surviving tool binding
  is whichever agent registered last.
- Expected invariant: each agent's `remember`/`recall`/`forget`/
  `search_memory` calls route to its own memory runtime (its layer manager,
  hot-layer state, review counters, session observer).
- Observed behavior: all pooled agents' memory tool calls go through one
  agent's `MemoryLayerManager` (the last registrant); the other agents'
  hot-layer state and per-session observer attribution are silently replaced
  in the shared registry.
- Impact: cross-agent memory state contamination — recall/search results and
  write attribution (observer/review/promotion counters) come from the wrong
  agent's runtime; the failure is invisible (no error, no log), and the shared
  underlying store masks part of it while hot-layer behavior diverges.
- Root cause: `ToolManager::register` is a silent last-wins insert; the
  AgentPool "replace your manager with the shared one" + "register your
  agent-scoped tools" pattern was never reconciled with per-agent tool
  identity, and `install_memory_layer_manager`'s documented "replace default
  memory tools" pattern made silent overwrite look sanctioned.
- Direction: make duplicate registration observable — `register` returning a
  replaced-tool or logging a warning, plus an agent-scoped registry wrapper
  for pool agents — and/or move memory tools out of the shared manager
  (per-agent managers, or a per-invocation resolution via ToolContext). The
  immediate EKO-side fix: register memory tools once on the shared manager
  with the primary agent's layer manager and bind per-agent identity through
  ToolContext instead of re-registering.
- Regression validation: unit test registering two tools with the same name
  asserting the overwrite is detected/logged; pool fixture with two agents
  asserting agent A's memory write is observed by A's own observer and is
  visible to A's recall through the shared manager.
- Validation reports: [V02-01](../validations/F-EXT-01/V02-01.md),
  [V04-02](../validations/F-EXT-01/V04-02.md)

### F-EXT-01-P2-01: `WRITE_TOOLS` static name list has drifted from the registered writer set — plan mode and read-before-edit miss run_code, git writes, and file-export tools

- Priority: P2
- Confidence: high (list vs registry verified); impact reachability is
  framework-wide, EKO-internal today limited
- Layer: framework (classification authority) + application (ad hoc
  special-casing)
- Evidence: `echo-agent/src/tools/mod.rs:118-139` (`WRITE_TOOLS` =
  edit_file/write_file/append_file/create_file/delete_file/update_file/
  move_file); consumers `src/agent/react/run/pipeline.rs:1004-1016`
  (PlanModeStage adds `name == "shell" || name == "delete_file"` — the first
  is drift compensation, the second is redundant), `snapshot.rs:230-236,
  282-285` (same special cases), `pipeline.rs:359-390` (ReadBeforeEditStage),
  `snapshot.rs:231,283`; registered mutating tools NOT in the list:
  `run_code` (`echo-tools/src/code.rs`), `git_commit`/`git_add`/`git_push`/
  `git_tag`/`git_branch` (`echo-tools/src/git.rs`), `enter_worktree`/
  `exit_worktree` (`echo-tools/src/worktree_tool.rs`), `write_excel`
  (`echo-tools/src/excel.rs`), `excel_load`, `data_export`
  (`echo-tools/src/data.rs`), `text_export` (`echo-tools/src/text.rs`) —
  all registered by `echo-tools/src/registry.rs:195-401`.
- Reachability: any framework consumer enabling `plan_mode` with a full tool
  registry gets a read-only filter that still exposes run_code/git writers/
  exports (they are visible and executable); EKO today enables plan mode only
  on subagents whose toolset is physically read-only, so in-repo impact is
  latent — but the writer-subagent defect (P1-01) shows exactly how the list
  interacts with real wiring.
- Expected invariant: "plan mode = read-only" and "force_read_before_edit"
  apply to every mutating tool the agent can call.
- Observed behavior: the filter/block lists are a hardcoded name set that has
  already drifted (shell missing, delete_file duplicated) and omits
  run_code/git-write/export tools entirely.
- Impact: plan mode and read-before-edit provide a false sense of
  write-protection; any consumer relying on them with mixed toolsets can have
  writes performed (arbitrary code via run_code, git commits/pushes, file
  exports) while believing the surface is read-only.
- Root cause: three parallel "is this a writer" classifications — per-tool
  `Tool::risk_level()` (echo-core), `WRITE_TOOLS` consts (root facade), and
  the dead `ToolRiskClassifier` name list (P3-02) — with no single authority
  and no test tying the list to the registry.
- Direction: derive the write classification from tool declarations
  (`risk_level() != ReadOnly` or an explicit capability marker) at registry
  build time, or add a registry test asserting `WRITE_TOOLS ∪ {shell}`
  covers every registered mutating tool name; remove the redundant
  `delete_file` special cases; delete `ToolRiskClassifier` (P3-02) once the
  authority is single.
- Regression validation: all-features registration fixture asserting
  `tools_for_llm()` under plan_mode contains no tool whose risk_level is
  non-ReadOnly; read-before-edit fixture with `write_excel`/`data_export`.
- Validation reports: [V01-01](../validations/F-EXT-01/V01-01.md),
  [V02-01](../validations/F-EXT-01/V02-01.md)

### F-EXT-01-P3-01: `ToolManager::result_cache` is dead — never written, never invalidated, doc claims caching that does not exist

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: field `echo-execution/src/tools.rs:71` (doc: "Only caches
  read-only tool results. Cleared on write operations."); constructed `:500,
  :519`; the only access is the read-hit probe `:635-641`; grep for
  `result_cache` across both repositories shows zero insert/clear sites.
- Reachability: every read-only call misses the (always-empty) cache — the
  60-second dedup behavior is absent; `invalidate_cache` (`:470-474`) only
  bumps the definitions version and never touches the result cache.
- Expected invariant: either the cache works (read-only result reuse with
  write invalidation) or it does not exist.
- Observed behavior: dead mechanism with a misleading doc contract; if a
  future developer adds the missing insert, the "cleared on write" guarantee
  is still unimplemented (no write hook), so stale read results could then
  surface.
- Impact: none today (always misses); maintenance and doc-contract hazard.
- Root cause: cache was scaffolded but the store path was never wired; the
  TruncationStage metadata pipeline (snapshot.rs:926-1060) took over output
  processing and the cache was left behind.
- Direction: delete the field, its two initializers, and the probe
  (`tools.rs:71,500,519,632-641`) per AGENTS.md cleanup; if read dedup is
  wanted later, implement it with write-path invalidation and a test.
- Regression validation: `cargo test -p echo_execution --lib` after deletion;
  no behavioral change expected (cache never hit).
- Validation reports: [V04-02](../validations/F-EXT-01/V04-02.md)

### F-EXT-01-P3-02: `ToolRiskClassifier`/`ToolRiskCategory` (echo-execution/src/risk.rs) is dead and a third risk-classification authority

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-execution/src/risk.rs:1-127`; grep for
  `ToolRiskClassifier|ToolRiskCategory` across both repositories: only the
  definition sites (the CLI's `BrowserActionRisk` in
  `echo-agent-app-core/src/browser/mod.rs:30` is a different type).
- Reachability: zero callers; the module compiles as public API but nothing
  constructs or calls it.
- Expected invariant: one classification of tool risk/write semantics
  (AGENTS.md: no parallel implementations of one semantic).
- Observed behavior: a second name-list classification (defaulting unknown
  tools to ReadOnly — a fail-open default) sits unused next to
  `Tool::risk_level()` and `WRITE_TOOLS`.
- Impact: future consumers may pick it up and get a third, divergent, and
  fail-open authority (unknown tools classified ReadOnly); dead code per
  AGENTS.md cleanup rules.
- Root cause: risk classification was attempted per-name before the
  per-tool `risk_level()` trait method existed; never wired.
- Direction: delete `echo-execution/src/risk.rs` (keep the module removed
  from `echo-execution/src/lib.rs:11`) after the P2-01 authority decision;
  if a safety-notice formatter is needed, build it from `risk_level()` +
  `capability_description()`.
- Regression validation: `cargo check -p echo_execution` after removal;
  grep for the two type names returns nothing.
- Validation reports: [V01-01](../validations/F-EXT-01/V01-01.md)

### F-EXT-01-P3-03: `spawn_task::truncate_output` cuts UTF-8 at a byte boundary

- Priority: P3
- Confidence: high (code fact); low (user impact)
- Layer: framework
- Evidence: `echo-agent/src/tools/builtin/spawn_task.rs:234`
  (`&bytes[..MAX_OUTPUT_LEN]` inside the `bytes.len() > MAX_OUTPUT_LEN`
  branch of `truncate_output`, `:228-238`).
- Reachability: `check_task_status` output formatting for spawned background
  task output containing multibyte characters at the truncation point.
- Expected invariant: AGENTS.md UTF-8 rule — character-boundary truncation
  (`chars().take(n)`), never byte slicing on possibly-multibyte text.
- Observed behavior: the slice cannot panic (guarded), but cutting mid-char
  produces a U+FFFD replacement character at the boundary.
- Impact: cosmetic corruption of task output previews (replacement char),
  divergence from the UTF-8-safe pattern used elsewhere.
- Root cause: byte-length-based truncation written before the chars() rule
  was applied to tool output paths.
- Direction: replace with `String::from_utf8_lossy(&bytes).chars().take(
  MAX_OUTPUT_LEN).collect()` (lossy decode then char truncation) or operate
  on the decoded string.
- Regression validation: unit test with a `MAX_OUTPUT_LEN`-sized prefix ending
  mid-多字节 character asserting no U+FFFD at the cut.
- Validation reports: [V01-01](../validations/F-EXT-01/V01-01.md)

### F-EXT-01-P3-04: Tool-level retry backoff `retry_delay_ms` is a fourth retry-math implementation duplicating `RetryPolicy`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-execution/src/tools.rs:26-46` (saturating math, exponent
  cap 5, 30 s cap, DefaultHasher jitter); classification authority is single
  (`ToolFailure::allows_automatic_retry`, `echo-core/src/tools/mod.rs:154-162`,
  consumed only at `echo-execution/src/tools.rs:707,884`); F-REL-01-P2-01
  established three retry/backoff implementations already.
- Reachability: every tool retry attempt (default `max_retries=2`,
  `retry_delay_ms=200`) — live on the main tool path.
- Expected invariant: one backoff implementation; consumers choose policy
  values (AGENTS.md).
- Observed behavior: tool-level retries use their own math instead of
  `echo_core::retry::RetryPolicy`; it is overflow-safe (better than the
  engine copy) but not unified.
- Impact: policy changes (caps, jitter) must be made in up to four places;
  F-REL-01's unification direction must include this site.
- Root cause: tool retry written alongside the manager before/without the
  unified policy; F-REL-01's P2-01 already flags the same class of defect.
- Direction: during the F-REL-01 unification, route tool-level backoff
  through `RetryPolicy` (keeping `ToolFailure::allows_automatic_retry` as the
  retryable predicate and the `retry_after_ms` honor); delete
  `retry_delay_ms` when migrated.
- Regression validation: after migration, tool retry tests in V04-02 plus a
  max-delay-cap assertion stay green.
- Validation reports: [V05-01](../validations/F-EXT-01/V05-01.md),
  [V04-02](../validations/F-EXT-01/V04-02.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Schema/execute contract (trait, ctx, ToolFailure, schema gen, serde failure handling, panic scan) | yes | passed | [V01-01](../validations/F-EXT-01/V01-01.md) |
| V02 | Name collision/registration (dup search, silent overwrite, AgentPool cross-agent) | yes | passed | [V02-01](../validations/F-EXT-01/V02-01.md) |
| V03 | Cursor and artifact integrity (opaque/bounded cursors, spill, read_artifact, UTF-8) | yes | passed | [V03-01](../validations/F-EXT-01/V03-01.md) |
| V04 | `cargo test -p echo_core --lib --locked tools` | yes | passed (exit 0, 50 passed) | [V04-01](../validations/F-EXT-01/V04-01.md) |
| V04 | `cargo test -p echo_execution --lib --locked tools` | yes | passed (exit 0, 19 passed) | [V04-02](../validations/F-EXT-01/V04-02.md) |
| V05 | Unified-retry cross-reference with F-REL-01 | conditional | passed | [V05-01](../validations/F-EXT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root MASTER-PLAN: "工具已有 call_id、流式 stdout/stderr/log、明确成功/失败/取消终态、UTF-8 安全截断和大结果 spill" | current | [V01-01](../validations/F-EXT-01/V01-01.md), [V03-01](../validations/F-EXT-01/V03-01.md) |
| Root MASTER-PLAN: "超长工具结果已统一为完整 artifact + 有界模型/会话投影；共享路径、大小、SHA-256 和 retention，conversation 删除级联清理" | current | [V03-01](../validations/F-EXT-01/V03-01.md), [V04-01](../validations/F-EXT-01/V04-01.md) |
| Root MASTER-PLAN (M13/Phase C): retry 复用 checkpoint/artifact | current for tool path (classification single; math duplicated — P3-04) | [V05-01](../validations/F-EXT-01/V05-01.md) |

## Coverage And Uncertainty

- Not inspected in depth (delegated): `shell.rs`/`database.rs`/`web/fetch.rs`
  artifact-writer usage details (F-EXT-02), permission-approval policy internals
  (F-HITL-01), subagent executor lifecycle beyond tool surface (F-SUB-01).
- P1-01 and P1-02 are statically verified chains; no end-to-end dynamic run
  was executed in this read-only task (no source modification allowed for
  fixtures). Both carry explicit regression validations.
- `validate_tool_parameters_async` has no production caller but is retained
  public framework API (per AGENTS.md framework rules) — recorded, not a
  finding.
- `ToolContext.cancel` is cooperative-only at the manager level (manager
  never polls it; timeout drop is the enforced bound) — recorded in V01,
  not a finding.
- AgentPool iteration order makes the P1-02 survivor nondeterministic; the
  finding stands regardless of which agent wins.

## Handoff

- Downstream tasks may rely on: contract inventory (V01), collision map (V02),
  artifact/pagination integrity (V03), test green state (V04-01/02), retry
  authority map (V05).
- `F-EXT-02/F-EXT-03`: domain tools must honor `ctx.cancel` and the artifact
  writer contract (writer usage reachability listed in V03).
- `F-SUB-01`: evaluate P1-01's subagent-surface implications (plan-mode filter
  on subagents) and P1-02's shared-registry pattern; the fix direction for
  P1-01 (remove `set_plan_mode(true)` from the writer builder) lives in
  `echo-agent-cli`.
- `A-TSK-*` (TaskRuntime): P1-01 directly gates writer-task capability;
  P1-02 gates pooled memory correctness.
- `F-REL-01`: P3-04 extends its P2-01 unification target to
  `echo-execution/src/tools.rs:26-46`.
- `X-BND-01`: record the write-classification authority decision (P2-01) and
  the AgentPool shared-registry ownership question (P1-02).
- This report becomes stale if the Tool trait, ToolManager registration,
  plan-mode filter, artifact writer, or pagination contract changes.
