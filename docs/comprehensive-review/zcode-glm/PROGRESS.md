# ZCode (GLM-5.2) Review Progress

> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Output root: `docs/comprehensive-review/zcode-glm/`
> Baseline: echo-agent `9b0e0fa`, echo-agent-cli `b3b2e81`
> Last updated: 2026-08-12

## Directory Layout

```
zcode-glm/
  tasks/          # task reports (<TASK-ID>.md)
  validations/    # validation reports (<TASK-ID>/<VALIDATION-ID>-<ATTEMPT>.md)
  synthesis/      # phase synthesis reports
```

## Task Status

| Task | Status | Validations | Notes |
|---|---|---|---|
| B-BASE-01 | ✅ complete | V01-V04 | 7 findings (P2×4, P3×3) |
| B-ARCH-01 | 🔄 in_progress | V01-V04 done | task report pending (agent writing) |
| B-PATH-01 | ✅ complete | V01-V04 | 6 findings (P2×3, P3×3) — see tasks/B-PATH-01.md |
| B-REF-01 | ✅ complete | V01-V06 | 14 reference findings (P1×5, P2×5, P3×4); 7 convergent patterns (C1-C7) + Temporal contrast; dual-attempt reconciliation (V01-01/02, V02-01/02, V04-01/02, V05-01/02) — see tasks/B-REF-01.md |
| Q-STA-01 | ✅ complete | V01-V04 | 5 findings (P1×1, P2×3, P3×1) + 2 positive confirmations — see tasks/Q-STA-01.md |
| F-FEAT-01 | 🔄 re-running | — | first agent stalled; re-launched |
| B-DOC-01 | ⏳ pending | — | blocked on B-ARCH-01 + B-PATH-01 |
| F-CORE-01 | ⏳ pending | — | blocked on B-ARCH-01 |
| A-BOOT-01 | ⏳ pending | — | blocked on B-PATH-01 |
| F-EXT-02 | ✅ complete | V01-V04 | 10 findings (P1×2, P2×5, P3×4) — V01 path/UTF-8 passed, V02 process-tree cancellation passed (exec-test), V03 atomic-write/duplicate-tool FAILED, V04 worktree traversal FAILED (exec-confirmed) — see tasks/F-EXT-02.md |
| F-OPS-01 | ✅ complete | V01-V04 | 10 findings (P1×3, P2×4, P3×3) + 1 positive (telemetry feature gate clean) — V01 scheduler lifecycle/shutdown/block_in_place, V02 headless no events/no RunStore/no cancel, V03 secret leakage into Run.input/output_preview + unbounded JsonlRunStore, V04 dead Metrics::record_*/shutdown_telemetry — see tasks/F-OPS-01.md |
| F-LLM-01 | ✅ complete | V01-V04 | 4 findings (P2×2, P3×2) — V01 contract type inventory + dup search, V02 ProviderCapabilities + streaming neutrality (Anthropic reasoning gap noted), V03 thinking translation + usage/cache authority (signature round-trip limitation noted), V04 adapter contract thinness + compile/test — see tasks/F-LLM-01.md |
| F-RCT-03 | ✅ complete | V01-V04 | 5 findings (P1×1, P2×2, P3×2) — V01 single streaming pipeline (passed), V02 backpressure lossy on intermediates (FAILED), V03 error-terminals droppable + no Cancelled emitted (FAILED), V04 loop-body equivalence holds with 3 documented divergences (passed) — see tasks/F-RCT-03.md |
| F-LLM-02 | ✅ complete | V01-V04 | 8 findings (P2×4, P3×4) — V01 request field mapping (cache_hints silent drop noted), V02 streamed/non-streamed response mapping (null-delta deserializer + SSE silent-drop noted), V03 tool-call assembly edge cases (empty-args drop noted), V04 usage + LlmError mapping ([DONE] sentinel noted) — AdapterClient/DefaultLlmClient dormant, see tasks/F-LLM-02.md |
| F-RCT-05 | ✅ complete | V01-V04 | 4 findings (P2×2, P3×2) — V01 two-snapshot-type model + resume=trajectory+new-turn (passed), V02 structural replay protection (completed_tool_call_ids trace-only) (passed), V03 think/tool/compact interrupt points + cooperative steer (passed), V04 corrupt-checkpoint silent-swallow + no version field (FAILED) — see tasks/F-RCT-05.md |
| F-WFL-01 | ✅ complete | V01-V04 | 8 findings (P1×1, P2×3, P3×4) — V01 workflow/task cleanly distinct (passed, zero cross-refs), V02 Graph+DAG build validation (passed; conditional-target + no-cycle-in-Graph are intentional gaps), V03 resume() parallel branch diverges from run()/run_until_interrupt()/run_stream() — no fork/merge (FAILED, P1), V04 checkpoint/resume lifecycle coherent (passed w/ gaps: no version field, silent list-skip, traversal surface) — workflow distinct from dynamic tasks, see tasks/F-WFL-01.md |
| F-TST-01 | ✅ complete | V01-V04 | 5 findings (P2×3, P3×2) — V01 mock-vs-provider matrix (type-faithful, behaviour-subset; passed w/ gaps), V02 call-level scripting yes / stream-level scripting no / mid-stream error+cancel no (passed w/ gaps), V03 testing feature isolation clean (passed — gated, not in default/full, CLI dev-dep only), V04 mock-driven tests green but 7 hard paths unreachable (passed — coverage gaps noted) — mocks model single-chunk streaming, text-only tools, single-FinalAnswer agents; see tasks/F-TST-01.md |
| A-TSK-06 | ✅ complete | V01-V04 | 3 findings (P2×1, P3×2) — V01 two-projection result preservation (full-result on SubagentReleased + structured summary on Note) + thinking-excluded-by-construction (passed), V02 execution_checks vs. acceptance_criteria two-gate separation (passed), V03 retention/cleanup cascade MISSING for ~/.eko/tasks/ (FAILED), V04 restart-equivalent review input via durable full_output (passed) — see tasks/A-TSK-06.md |
| A-FE-01 | ✅ complete | V01-V04 | 6 findings (P2×1, P3×5) — V01 DTO field matrix (ToolInfo wire vs manual drift confirmed; SkillInfo/McpServerInfo manual matches hand-built wire, generated matches unused Rust struct), V02 enum coverage (McpConnectionStatus generated misrepresents wire; SubagentRunEventKind has dead 'artifact' variant), V03 Option/null semantics (FileEntry/DiffLine null-vs-undefined cosmetic drift), V04 ts-rs orphan inventory (5 orphan generated files incl. SubagentRun; no contract tests; 3 shadowed nominal types) — see tasks/A-FE-01.md |
| Q-TST-01 | ✅ complete | V01-V04 | 7 findings (P1×2, P2×2, P3×3) + 2 positives — **reviewed at echo-agent 3aa7929 (baseline 9b0e0fa + 1 post-baseline "M1 mock 隐身衣 removal" commit); CLI b3b2e81**. V01 module map (1942 framework tests; react_loop/Anthropic+OpenAI streaming parse/revisioned_adapter = 0; react/tests.rs 81 tests = wiring/builder only), V02 10-test A/B/C grading (7 meaningful / 2 restating / 1 print-only), V03 ignored inventory clean (6 documented #[ignore], 0 frontend skips), V04 mock-invisibility: M1 RESOLVED single-chunk-streaming + usage-cloak + completion-order-batch (3 high-impact seams); 5 remain open (text-only MockTool, single-FinalAnswer MockAgent, one-variant FailingMockAgent, mid-stream cancel, ToolContext-ignoring). P1-01 non-streaming react_loop zero tests, P1-02 Anthropic+OpenAI streaming parse zero tests, P2-01 revisioned_adapter zero tests, P2-02 MockTool text-only, P3-01/02/03 mock-agent + cancel + print-only compressor — see tasks/Q-TST-01.md |

## Key Findings So Far (B-BASE-01)

- **B-BASE-01-P2-01**: Cross-repo path hygiene clean (positive)
- **B-BASE-01-P2-02**: CLI never enables sqlite (positive, AGENTS.md invariant holds)
- **B-BASE-01-P2-03**: 13 auto-discovered examples lack required-features
- **B-BASE-01-P2-04**: CI runs NO conditional matrices (feature/GUI/frontend)
- **B-BASE-01-P3-01**: echo-agent test uses --lib --tests not --all-targets
- **B-BASE-01-P3-02**: @tailwindcss/vite duplicated in package.json deps+devDeps
- **B-BASE-01-P3-03**: CI ubuntu-latest only, macOS unexercised

## Rules For This Reviewer's Agents

- All reports go to `zcode-glm/tasks/` and `zcode-glm/validations/`.
- Do NOT write to the shared `reports/` directory.
- Do NOT modify shared `TASKS.md` or `README.md`.
- Use reviewer: `ZCode (builtin:bigmodel-coding-plan/GLM-5.2)`.
