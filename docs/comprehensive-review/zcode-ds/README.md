# ZCode-ds Review Track

> Reviewer: ZCode (deepseek-v4-flash)
> Status: in progress
> Started: 2026-08-12
> Scope: independent comprehensive review of `echo-agent` and `echo-agent-cli`

This directory contains only ZCode-ds conclusions and evidence. The shared task
catalog, reporting protocol, and templates remain one level above. Conclusions
from the other two AI reviewers must not be copied here; independence is
required for a meaningful later comparison.

## Output Paths

```text
zcode-ds/reports/tasks/<task-id>.md
zcode-ds/reports/validations/<task-id>/<validation-id>-<attempt>.md
zcode-ds/reports/synthesis/<deliverable>.md
```

Every validation attempt is immutable. A failed or inaccurate attempt remains
as evidence and a corrected run receives the next attempt number.

## Progress

| Phase | Total | Pending | In progress | Needs evidence | Complete |
|---|---:|---:|---:|---:|---:|
| B - baseline and architecture | 5 | 0 | 0 | 0 | 5 |
| F - framework | 38 | 0 | 0 | 0 | 38 |
| A - EKO application | 29 | 0 | 0 | 0 | 29 |
| X - cross-repository contracts | 10 | 0 | 0 | 0 | 10 |
| Q - dynamic quality gates | 13 | 0 | 0 | 0 | 13 |
| S - synthesis and roadmap | 5 | 0 | 0 | 0 | 5 |

Completed: all 100 tasks (5 B-* + 38 F-* + 29 A-* + 10 X-* + 13 Q-* + 5 S-*)
— 95 task reports + 744 immutable validation reports + 5 synthesis
deliverables. **The comprehensive review is COMPLETE.**

Synthesis deliverables:
- `reports/synthesis/framework-review.md` — 49 canonical P1 (0 P0)
- `reports/synthesis/application-review.md` — 25 canonical P1 (0 P0)
- `reports/synthesis/cross-repository-review.md` — 6 new P1 (0 P0)
- `reports/synthesis/quality-and-validation-review.md` — ledger + gate audit
- `reports/synthesis/iteration-roadmap.md` — 22 milestones (M1-M22), 7 phases,
  all 80 canonical P1 covered exactly once, framework-first merge order,
  12 staged migrations each with a termination milestone, D1-D32 deletion
  matrix mapped

Canonical P1 census: **80 total (F 49 + A 25 + X 6), zero P0 anywhere.**
Leading systemic themes: "silent failure" family (~20 P1), mock invisibility
cloak (tests certify wire shapes no real provider produces), terminal
integrity holding only at the envelope adapter, permission boundary
excluding TaskRuntime Subagents (X-AUT-01-P1-01), parity invariant failing
at the management/control layer (TUI cleanest, GUI/cron/CLI gaps).

Latest completed phase (S, 2026-08-12): all 5 synthesis deliverables — see
above. Q phase gates: framework/EKO/frontend/GUI gates all green; doctest
phase RED (stale CompressionInput initializer, Q-FW-02-P2-02) and invisible
to the literal gate (`--all-targets` excludes doctests).

Latest completed phase (Q, 2026-08-12): 13 tasks — submission gates ALL
GREEN (Q-FW-01 framework gate: fmt 0 diff, all-feature clippy 0 warnings,
panic-safety clippy 0 hits, 1930 tests 0 failed, no-default clean; Q-CLI-01
EKO gate: same 5 commands + dependency-tree zero SQLite; Q-WEB-01 frontend:
prettier 0 / vitest 101 / build 0; Q-GUI-01: gui bin check 0 / 48 tests 0)
except one: Q-FW-02 found the all-features doctest phase RED (stale
`CompressionInput` initializer, 81 passed / 1 failed). 10 new P1 across 5
tasks: `Q-STA-01-P1-01` percent_decode byte-slice panic on remote hrefs
(web_search DuckDuckGo fallback, reproduced exit 101); `Q-TST-01-P1-01..03`
non-streaming ReAct loop has zero tests, a pipeline test enshrines
completion-order terminals (blocks the F-RCT-04 fix), and the Anthropic SSE
parse path has zero tests — the "mock 隐身衣" conclusion is now evidenced;
`Q-FLT-01-P1-01/02` truncated/clean-disconnect streams silently accepted as
complete final answers (no finish_reason check) and mid-stream LLM timeouts
never retried (retry only wraps stream creation, both dynamically
reproduced); `Q-FLT-02-P1-01` crash between request_pause and the Paused
branch leaves a Paused run with a Running+claimed task that no recovery
clears — resume polls forever; `Q-E2E-01-P1-01..03` scenario-level verdicts:
GUI chat cannot complete error/cancel turns with equivalent facts, Task
write work cannot complete on ANY surface (writer silently read-only),
cron can never auto-start any scenario. Also: Q-DEP-01-P2-01 — 6 active
RUSTSEC advisories in the shipped binary (lopdf stack overflow, quick-xml
DoS family, crossbeam-epoch), reachable from untrusted PDF/XLSX/DOCX/XML;
Q-PERF-01-P2-01 — TaskRuntime file shadow O(N²) I/O on the executor
critical path (real-data measurement).

Latest completed phase (X, 2026-08-12): 10 tasks — **6 new canonical P1
(per S-X-01 census; the alias F-EXT-01-P1-01 ↔ A-TOOL-01-P1-01 resolved to
A-TOOL-01-P1-01)** across 4 tasks — `X-AUT-01-P1-01` TaskRuntime writer/readonly Subagents execute
automation entirely outside the permission boundary (no
PermissionService/provider/mode; .git/.ssh/.env protected-path checks
never run; bounded only by plan mode + worktree/sandbox);
`X-STA-01-P1-01` conversation deletion leaves the full runtime transcript
+ plan on disk (RuntimeStateStore trait has no delete API);
`X-INV-01-P1-01/02` two live UTF-8 byte-slice panics (`parse_pdf_date`
pdf.rs:225-227, `extract_number_near_key` eval/runner.rs:728, both
reproduced exit 101); `X-EVT-01-P1-01` cancel/timeout class lost at the
envelope boundary — mid-stream cancelled/timed-out Subagents persist as
`failed` (`subagent_status_from_error` bypassed);
`X-EVT-01-P1-02` chat-turn timeouts have no typed terminal — timed-out
turns end `'completed'`. Invariant audit positives: zero `worker` terms,
zero SQLite in CLI, zero parallel task CRUD, panic-safe macro surface
(CLI production zero unwrap/expect/panic!), all 16 manifest paths
relative. Placement map produced a 32-row deletion-target matrix
(D1-D32) for S-RDM-01. No P0 findings anywhere in the X phase.

Latest completed phase (A, 2026-08-12): 29 tasks — **25 canonical P1
(28 filed − 4 merges + 1 re-rating, per S-APP-01 census)**, key ones — `A-TSK-01-P1-01` torn
tail line in events.jsonl bricks the run permanently (no truncation repair
despite comment promise); `A-CFG-01-P1-01..03` workspace switch chdirs but
freezes watcher/hook/config, exit_workspace never restores CWD, and
switch is GUI-only (TUI/CLI stubs print "Switched" without switching);
`A-HITL-01-P1-02` REPL empty/EOF stdin auto-approves ("" => Approved) and
the blocking read_line defeats the shared 5-min deadline; `A-HITL-01-P1-03`
every surface's "approve all" sends SessionAllTools => framework `"*"`
wildcard = ALL tools allowed for the session (EKO is the sole producer);
`A-INT-01-P1-01` GUI MCP config editor never persists to disk, all
GUI-created servers vanish on restart while UI says "配置已保存并应用";
`A-OBS-01-P1-02` webhook sends raw tool args/error text to external
endpoints with zero redaction; `A-SRF-02-P1-01` build_tauri_app chains two
`.setup()` but Tauri runs only the last — browser://event bridge dead;
`A-SRF-03-P1-01` interrupt_prompt strands frontend turn state, chat input
queues forever until reload; `A-SRF-04-P1-01` REPL/channel turns not
cancellable — Ctrl+C kills the process, skipping shutdown hooks;
`A-EVO-01-P1-01` REPL session exit auto-runs LLM "reflection" that appends
to .eko/memory/PROJECT.md with no user confirmation, review gate, or
change log.

Latest completed wave (batch 5, wave 3, 2026-08-12): the final 12 F-* tasks,
14 P1 findings across 8 tasks (F-MAC-01: 1, F-RCT-05: 3, F-SUB-02: 2,
F-HITL-01: 3, F-WFL-01: 1, F-INTENT-01: 1, F-TST-01: 2, F-MAG-01: 1;
F-PLG-01 / F-NBK-01 / F-EVO-01 / F-TSK-03: 0), all spot-checked:
`F-MAC-01-P1-01` `#[derive(Tool)]` emits `<Self as echo_agent::tools::ToolRunner>`
but the facade never exports `ToolRunner` — documented facade-only usage
fails E0405 (derive_tool.rs:387 vs tools/mod.rs:109-114, reproduced);
`F-RCT-05-P1-01` cancel/error mid-tool-batch saves a checkpoint with
unpaired tool calls, resume validator rejects it and
`restore_thread_context` wipes the whole conversation to the system prompt
(dynamically reproduced); `F-RCT-05-P1-02` same-turn steer silently dropped
— mailbox lease keyed by `turn_id` but drained by `current_run_id` (None in
Chat/Auto) (stream_channel.rs:111-122 vs :333); `F-SUB-02-P1-01/02` Team has
zero CancellationToken, timeout drops the outer future but `tokio::spawn`ed
members keep running/writing detached; `F-HITL-01-P1-01` live approval path
never asks the human — RequireApproval/Ask become opaque tool errors, the
only ask-capable code is dead (`process_steps` uncalled); `F-HITL-01-P1-02`
user-modified tool args silently discarded (sole reader in dead code);
`F-HITL-01-P1-03` SessionAllTools approval inserts a `"*"` wildcard rule —
one session allow unlocks ALL tools (service.rs:900-907);
`F-WFL-01-P1-01` AfterNode checkpoint stores only the join node — resume
skips pending parallel fan-out branches and bypasses before-interrupt;
`F-INTENT-01-P1-01` TriggerSupervisor hook fusion emits confidence 0.6 but
the router re-applies the 0.7 threshold — documented skill-activation retry
silently never fires; `F-TST-01-P1-01` mock emits content+usage in a single
chunk, certifying `usage_reported: true` in a wire shape no real provider
produces (hides F-LLM-03-P1-02); `F-TST-01-P1-02` streaming is not scriptable
at all (one `stream::once` chunk) — ordering/lossless defect classes
structurally unreproducible at loop level; `F-MAG-01-P1-01` handoff runs the
target as a detached uncancellable untimed `tokio::spawn` + oneshot wait
(handoff/mod.rs:262-273, zero cancel tokens) — same defect class as
F-SUB-02-P1-01/02.

Latest completed wave (batch 5, wave 2, 2026-08-12): 6 tasks, 13 P1 findings,
all spot-checked and confirmed —
`F-RCT-03-P1-01` stream events silently dropped when channel full, incl.
terminal Err (stream_macros.rs:42-47, finalize.rs:226/267) — raw stream can
end with no terminal; `F-RCT-03-P1-02` ReactAgent overrides
`execute_stream_with_cancel` (react/mod.rs:2844) bypassing `cancel_aware_stream`
(echo-core/src/agent/mod.rs:896-917) — `AgentEvent::Cancelled` never emitted;
`F-RCT-04-P1-01` concurrent batch results pushed in FuturesUnordered completion
order vs assistant tool_calls in stream-index order (tools.rs:207-240 vs
processor.rs:141-147) — strict providers reject next request 400 after tools
ran; `F-RCT-04-P1-02` batch timeout/cancel end turn without typed terminal,
trace run stays Running, verifier-accepted final_answer discarded;
`F-CMP-01-P1-01` message-count windows never bound tokens (sliding_window.rs:48-66,
prepare never re-checks token_limit); `F-CMP-01-P1-02` one immortal system
summary appended per compression pass, unbounded growth (summary.rs:346);
`F-CMP-01-P1-03` adaptive L1 fold inserts Role::User between assistant
tool_calls and kept tool results, breaking pairing-contiguity invariant
(levels.rs:392-396); `F-OPS-01-P1-01` cron tick can never fire — cron 0.12.1
`upcoming()` is strictly-future so `next <= now` is unsatisfiable, all cron
tasks silently never run (runner.rs:80-93, empirically proven V04-01);
`F-SKL-01-P1-01` circular skill `depends_on` recurses unboundedly → stack
overflow SIGABRT (registry.rs:468-510, reproduced exit 134);
`F-SKL-01-P1-02` dual SkillRegistry divergence — resume marks only the
tracking registry, read tools check the progressive one → "not activated"
after restart (react/mod.rs:1703-1704 vs resource_tool.rs:98-103);
`F-INT-02-P1-01` LSP requests have no timeout/cancel cleanup — hung server
blocks shutdown forever; `F-INT-02-P1-02` QQ channel send task busy-spins a
CPU core after stop() (qq/channel.rs:108-132, `loop{ if let Some }` on closed
channel, handle discarded); `F-INT-02-P1-03` A2A tasks/cancel never cancels
execution and Completed overwrites Canceled (server.rs:404-414, 439-442).

Latest completed wave (batch 5, wave 1, 2026-08-12): 6 tasks, 5 P1 findings —
`F-RCT-02-P1-01` non-streaming turn silently returns `Ok("")` on core-loop
error (react_loop.rs:711-727 logs instead of forwarding; stream_channel.rs
wrapper forwards); `F-SUB-01-P1-01` per-role `tool_filter` has zero production
readers, `agent_tool` cannot restrict subagent tools; `F-INT-01-P1-01` HTTP
transport 202-async path dead — nothing ever fires the pending oneshot
channels, any 202/SSE server hangs every call 60 s (http.rs:161-179);
`F-CTX-01-P1-01` provider window mapping bypassed, EKO runtime hardcodes 396K
(infra.rs:23, kimi k2.x real 256K overflows); `F-TSK-02-P1-01` Skip has no
dependency propagation — skipping a task with Pending dependents stalls the
executor with a misleading "DAG stalled (cycle or blocked)" message.
Also `F-EXT-03-P1-01` research_remember/recall are non-persistent stubs that
fabricate "stored successfully" (echo-tools/research/memory.rs:96-119).
All five P1s above spot-checked against source and confirmed, plus
`F-EXT-03-P1-03` IQR panic on exactly-4-value columns (OOB index, reproduced).

Latest batch (4th, 2026-08-12): `F-LLM-03` complete — 4 P1 (tool-call
accumulator keyed by `tool_call_args.len()` not stream index; strict
`AnthropicUsage` fails real `message_delta.usage` so final usage/finish
chunk is silently dropped → `usage_reported` always false on Anthropic
streaming; multi-system collapse drops base prompt after canonical
reinjection; response thinking blocks unmodeled) + 3 P2 + 1 P3. Both
most-critical P1s spot-checked against source and confirmed. Cross-task:
the Anthropic adapter uses its own inline SSE parser, so F-LLM-01-P1-01's
shared-transport fix does NOT cover it (F-LLM-03-P2-01); MASTER-PLAN M9
usage authority is regressed on the Anthropic streaming path (P1-02).

## ZCode-ds Review Rules

- Read the root `AGENTS.md`, shared `README.md`, `REPORTING.md`, this file,
  and only the assigned task/dependency reports before inspecting source.
- Do not use findings from another AI reviewer while performing atomic tasks.
- Do not edit another reviewer's directory.
- Do not place reports in `docs/comprehensive-review/reports/`.
- Do not overwrite validation attempts. Use `V01-02.md`, `V01-03.md`, and so
  on.
- Source code remains read-only during review; fixes are deferred to the
  final iteration roadmap.
- Every conclusion must carry executable or inspectable evidence; a
  conclusion without a validation link stays `needs_evidence`.
