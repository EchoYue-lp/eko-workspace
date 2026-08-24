# S-QA-01: Quality And Validation Synthesis

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean in both repositories (before and after this task's
> re-runs; only `target/` artifacts and /tmp logs were touched)
> Output: `reports/synthesis/quality-and-validation-review.md`
> Dependencies: all 13 Q-* task reports and their validation reports;
> F-TST-01 and A-FE-03 (test-credibility evidence); relevant F/A/X validation
> reports where referenced.

This synthesis consumes completed task reports and validation summaries per
REPORTING.md Synthesis Rules. It reconciles the quality/validation corpus of
the ZCode-ds track: the verification ledger, the gate-status table and its
CI/local consistency conclusion, the test-credibility conclusions, the
not_run/inconclusive classification, and the flaky/platform-gated inventory.
Every claim below links to a validation report. This task's own three
validations: [V01-01](../validations/S-QA-01/V01-01.md) (command/report count
reconciliation), [V02-01](../validations/S-QA-01/V02-01.md) (unexecuted-matrix
audit + gate contradiction resolution), [V03-01](../validations/S-QA-01/V03-01.md)
(flaky/inconclusive/not_run classification).

---

## 1. Verification Ledger (验证总账)

### 1.1 Count reconciliation

The ledger claimed by `zcode-ds/README.md` ("95 task reports + 729 immutable
validation reports") is exact. Verified at the file-system level
([S-QA-01 V01-01](../validations/S-QA-01/V01-01.md)):

| Phase | Task reports | Validation reports |
|---|---:|---:|
| B — baseline and architecture | 5 | 21 |
| F — framework | 38 | 282 |
| A — EKO application | 29 | 243 |
| X — cross-repository contracts | 10 | 57 |
| Q — dynamic quality gates | 13 | 126 |
| S — synthesis | 5 (0 before this task) | 3 (this task) |
| **Total** | **95** | **729** |

Additional reconciliation facts (V01-01):

- All **1,894** validation-file links in the 95 task reports resolve; zero
  missing links.
- Status census of the 729 reports: **673 passed** (including passed-with-
  caveat variants), **52 failed**, **1 inconclusive**, **3 not_run**. Sum 729.
- The 52 failed reports are all system-under-test invariant violations
  recorded as findings (19 Q-E2E-01 scenario rows, 6 Q-FLT-01, 5 Q-FLT-02,
  2 Q-FW-02, 1 Q-STA-01, plus 18 earlier-phase A-/B-/F-/X- validations).
  None is a harness or execution failure; per REPORTING.md a failed
  validation becomes a finding and does not block task completion.
- No placeholder reports (zero files under 400 bytes), no duplicate attempt
  IDs, no overwritten attempts: 58 task directories contain more report files
  than unique validation IDs, all explained by immutable re-attempts (e.g.,
  A-BOOT-01 V03-01..03, F-EVO-01 V04-05-01/02 harness-quoting retry,
  Q-FLT-02 V03-01/V03-02, A-SRF-02 V04-02/03) or folded conditional
  validations (F-LLM-01 V05, F-MEM-01 V05) or declared not-applicable rows
  (Q-GUI-01 V03–V05). One report covering two commands (A-FE-03 V04: vitest +
  tsc) is a minor granularity deviation, noted, not a gap.

### 1.2 Per-Q-task execution audit ([V02-01](../validations/S-QA-01/V02-01.md))

Every required validation declared by the 13 Q-* task cards has an executed
report: Q-FW-01 (5 gate commands → V01–V05), Q-FW-02 (12 per-feature
compiles V01–V12 + grouped examples V13 + doctests V14 + doc links V15),
Q-CLI-01 (5 gate commands + SQLite-absence V01–V06), Q-GUI-01 (bin check V01,
tests V02; V03–V05 declared not-applicable), Q-WEB-01 (prettier V01, tests
V02, build V03), Q-STA-01 (8 rule-family reports), Q-TST-01 (V01–V05),
Q-DEP-01 (V01–V05), Q-PERF-01 (V01–V05), Q-DOC-01 (V01–V05), Q-FLT-01
(V00 + V01–V08), Q-FLT-02 (V01–V08 with V03 split), Q-E2E-01 (46
scenario/surface pairs + 3 environmental not_run). Not-applicable pairs
(Attachment on cron/background, Browser/MCP management on cron/background)
are stated without fake reports, per REPORTING.md.

---

## 2. Gate Status Table And CI/Local Consistency

### 2.1 Gate status

| Task | Gate surface | Verdict | Evidence |
|---|---|---|---|
| Q-FW-01 | `echo-agent` mandatory gate: fmt check, all-feature Clippy `-D warnings`, panic-safety Clippy, all-target/all-feature tests, no-default lib check | **GREEN** — all 5 commands exit 0 at 9b0e0fa | Q-FW-01 V01–V05 (1,930 tests, 0 failed, 3 ignored) |
| Q-CLI-01 | `echo-agent-cli` mandatory gate: fmt check, both Clippy configs, all-feature tests, app-core no-default check, dependency-tree SQLite absence | **GREEN** — all 6 exit 0 at b3b2e81 | Q-CLI-01 V01–V06 (zero sqlite crates in the reachable graph) |
| Q-WEB-01 | Frontend: prettier check, vitest, production build | **GREEN** — exit 0 each (26 files / 101 tests; build 32.15 s) | Q-WEB-01 V01–V03 |
| Q-GUI-01 | Tauri/GUI matrix: gui bin check + gui-feature tests | **GREEN** — exit 0 each (48/48 tests, bin linked) | Q-GUI-01 V01–V02 |
| Q-FW-02 | Feature/examples/docs matrix | **RED on two rows** — doctests 81 passed / 1 failed / 25 ignored (exit 101); demo45_customer_service fails E0433 under its declared required-features (7/8 examples pass) | Q-FW-02 V14 (failed), V13 (failed) |

Gate-attached caveats that qualify "green":

- **Q-GUI-01-P3-01**: the GUI gate is green with zero tests covering boot/
  setup composition — the known double-`.setup()` defect (A-SRF-02-P1-01,
  dead `browser://event` bridge) is invisible to the matrix (bin harness runs
  0 tests).
- **Q-WEB-01-P3-01** (= canonical A-FE-01-P3-02): prettier passes only
  because the tree is clean; a fresh ts-rs regeneration writes 79 unformatted
  files and turns the gate red until a manual `prettier --write` — the
  mechanism is unchanged in source.
- **Q-FW-02-P2-01**: `demo45_customer_service` cannot compile with exactly
  its declared `required-features` (`content-guard` API used but undeclared);
  all-features CI builds mask it.

### 2.2 The doctest contradiction and the CI/local consistency conclusion

Q-FW-01 reports the full gate green; Q-FW-02 V14 reports the all-features
doctest phase red (81/1/25) and asserts the mandatory gate is therefore red.
Resolved empirically at the reviewed commit by re-running both commands
([S-QA-01 V02-01](../validations/S-QA-01/V02-01.md)):

- `cargo test --workspace --all-targets --all-features --locked` → **exit 0,
  78 test targets, zero "Doc-tests" targets in the log**. The gate command as
  written does NOT run doctests: `--all-targets` selects lib/bins/tests/
  examples/benches but excludes the doctest unit.
- `cargo test --doc -p echo_agent --all-features --locked` → **exit 101,
  81 passed / 1 failed / 25 ignored**, `error[E0063]: missing field
  focus_instructions` at `src/testing/mod.rs:39` (stale `CompressionInput`
  initializer) — byte-for-byte reproduction of Q-FW-02 V14 at the same
  commit (9b0e0fa, clean tree).

**Consistency conclusion**: local and CI execute the same gate command, so
both are "green" and both are blind to the doctest failure. Gate greenness is
consistent across CI and local but certifies only compiled/unit/integration
targets — not the doc-example contract, which is RED at this commit. Two
factual corrections follow from the re-run: Q-FW-01 V04-01's claim "Doctests
included in the run" is incorrect (they were not); Q-FW-02-P2-02's mechanism
claim that the mandatory gate "includes lib doctests" is incorrect for
`--all-targets` invocation — its substance (stale doc example, P2 doc-contract
defect) is confirmed and remains a finding. Roadmap consequence: fix
`src/testing/mod.rs:39` and add a doctest check (default `cargo test` or an
explicit `--doc` step) to the gate/CI so the red doc surface becomes visible;
otherwise the gate will keep certifying a broken doc contract.

---

## 3. Test Credibility Conclusions (Q-TST-01 "mock 隐身衣")

The framework's green gate certifies mock-shape behavior, not provider
contracts. Three P1 findings establish the credibility conclusion; F-TST-01
supplies the root causes in the mock layer.

- **Q-TST-01-P1-01 — non-streaming ReAct loop has zero tests.**
  `react_loop.rs` (`run_react_loop`, the non-streaming loop behind every
  direct-answer/scheduler path) has no tests; `react/tests.rs` (81 tests)
  drives `MockAgent` orchestration but never a real loop with
  `MockLlmClient`. The F-RCT-02-P1-01 error-swallow defect (`Ok("")` on
  core-loop error) shipped in exactly this gap; every non-streaming regression
  passes the gate green. `tests/react_smoke.rs`'s "deferred" full-loop mock
  tests never landed even though the `llm_client` injection field shipped.
- **Q-TST-01-P1-02 — a test enshrines the wrong (completion-order) contract.**
  `pipeline.rs:1634-1720` (`multiplexed_streams_preserve_identity_and_terminal_order`)
  asserts terminals `["call-b","call-a"]` in completion order, matching the
  production `FuturesUnordered` push order (tools.rs:215-227) that violates
  the provider-legal stream-index order (F-RCT-04-P1-01; strict providers
  reject the next request 400 after tools ran). The suite therefore certifies
  the defect, and the correct fix cannot land without changing this test
  (negative control proven: the test would fail if production were fixed).
- **Q-TST-01-P1-03 — Anthropic SSE parse path has zero tests.**
  `anthropic.rs:410-617` (streaming `convert_response`, `AnthropicStreamEvent`
  handling, `message_delta` arm) is compile-tested only; all 7 adapter tests
  cover request/cache conversion. F-LLM-03-P1-01/P1-02 and P2-01 (tool-call
  accumulator by length, strict usage struct dropping the final usage chunk,
  silent event drops) all shipped through a green gate in the single most
  defect-dense untested seam of the framework.

Root causes (F-TST-01-P1-01/P1-02/P2-01..03): `MockLlmClient` emits content+
usage in one `stream::once` chunk — a wire shape no real provider produces —
so `usage_reported: true` is certified in the loop suite while the real
Anthropic path reports false; streaming is not scriptable at all (no
multi-chunk ordering, mid-stream errors, incremental tool-call deltas);
`MockTool` scripts only Permanent failures; `MockAgent` emits a single
`FinalAnswer` and ignores the cancel token; cancellation is modeled as a loud
error instead of the real silent end-of-stream.

Positive pole of the map: the EKO task-runtime store/executor layer (34 + 46
meaningful tests incl. stale-claim `:3100`, illegal-transition `:2300`,
stale-revision `:3039`, mock-driven executor loop) and the frontend
subagent/task stores have genuinely meaningful tests; terminal-monotonicity
claims are the well-tested side.

Secondary credibility items: zero-assertion/toy compressor tests
(Q-TST-01-P2-01: `test_sliding_window_compressor` is print-only, the only
`summary.rs` test exercises a bare `Mutex`); the EKO→framework
`revisioned_adapter.rs` (388 lines) has zero round-trip tests despite the
AGENTS.md adapter-losslessness rule (P2-02); `chatStore.toolExecution.test.ts`
pins id-keyed dedupe while the live two-producer duplicate class of
A-SRF-03-P2-01 is untested (P2-03); `chatStore` core reducer incl. the
MAX_MESSAGES=500 cap has zero direct tests (A-FE-03-P3-04); six frontend
stores have no test files (Q-TST-01-P3-01).

**Net conclusion for the roadmap**: the framework's loop-level test suite
must be re-based on scriptable chunk-sequence fixtures (F-TST-01 direction)
before the streaming/usage/tool-ordering fix families can land with
failing-then-passing tests; the non-streaming loop needs its own mock-driven
test family; the Anthropic response side needs literal wire fixtures.

---

## 4. not_run / Inconclusive Classification

- **3 not_run — all in Q-E2E-01 (V47–V49), all with concrete environmental
  reasons and attempted prerequisite checks**
  ([V03-01](../validations/S-QA-01/V03-01.md)):
  - V47: live GUI + real-LLM turns — zero credential matches in env scan, no
    model config, GUI launch forbidden by the review protocol.
  - V48: real IM channel traffic — no bot tokens/accounts, no config.
  - V49: browser sidecar + MCP server — playwright absent, no browser cache,
    no `mcp.json`, no server.
  The Q-E2E-01 card explicitly allows `not_run` when the scenario was
  attempted and prerequisites were checked; the 46 static scenario/surface
  pairs remain the authoritative verdicts.
- **1 inconclusive — F-FEAT-01 V04-01**: static-only by task definition;
  its `needs_evidence` rows were resolved by Q-FW-02 V01–V12 (all 12
  standalone feature compiles exit 0). Preserved immutably; conclusion
  deferred is now supported.
- **Q-DEP-01 advisory scanning is a passed validation, not a skipped one**:
  github.com unreachable was handled by snapshotting the rustsec DB from the
  codeload tarball (1,216 advisories, 2026-08-12) and serving it locally with
  `--stale`; `cargo audit` exit 1 = "vulns found" recorded as expected; DB
  freshness window documented. Same pattern for `npm audit`: run against the
  npmjs registry override because the configured npmmirror mirror returns 404
  on the audit endpoint (deviation → finding Q-DEP-01-P3-02). No network
  check was silently skipped; every workaround is recorded in V03-01 of
  Q-DEP-01.
- **Declared conditional not_run rows without separate files**: A-DOM-01 V04
  and F-EXT-03 V04 (live-network provider/Zotero smokes; opt-in `#[ignore]`,
  API keys required, read-only review) — protocol-compliant, stated in their
  matrices.

---

## 5. Flaky / Platform-Gated Inventory

Complete inventory from Q-TST-01 V03-01 (cross-checked with Q-FW-01 V04,
Q-GUI-01 V02, Q-FLT-01/02 deviations; classification in
[V03-01](../validations/S-QA-01/V03-01.md)):

- **Zero flaky tests in the reviewed corpus**: no `#[ignore]` used as a flaky
  escape hatch, no retry wrappers, no sleep-tolerance assertions.
- **5 `#[ignore]` tests, all documented opt-in live smokes**: LSP smoke
  (`EKO_LSP_SMOKE=1`, lsp/manager.rs:236), provider smokes
  (`EKO_PROVIDER_SMOKE=1`, research/clients.rs:979, research_connectors.rs:
  616), 2 Zotero credential-gated tests (clients.rs:1005,
  research_connectors.rs:649). None hides a required unit invariant; the
  gates' greenness is not inflated by skips.
- **Platform-gated tests** (`#[cfg(unix)]`/`#[cfg(target_os)]`) are standard
  POSIX/Windows arms (sandbox/local.rs, tui/clipboard.rs:24-154,
  tauri/ipc.rs:135-138, shell.rs:577, a2a/serve.rs:176, config.rs:714,
  state/file.rs:238, etc.); both CI host arms run. No permanently-false
  `#[cfg(any())]` guards in test modules.
- **Frontend**: zero `describe.skip`/`it.skip`/`test.skip`.
- **Documented nondeterminism risks, not filed as flaky**: the
  pipeline.rs:1634 controlled-delay test uses a biased select (minor risk;
  the test is independently a wrong-contract test — section 3); one
  harness-quoting failure pair (F-EVO-01 V04-05-01 failed → V04-05-02
  passed) is an immutability-correct retry, not product flakiness.
- **Ignored-test tallies in gate runs**: 3 ignored tests in Q-FW-01 V04
  (echo-tools web/media groups + 1) and 3 GUI-matrix ignored doctests
  (CLI-side, pre-existing) — all documented.

---

## 6. Contradiction Reconciliation And Minority Conclusions

- **Resolved**: Q-FW-01 green-gate vs Q-FW-02 red-doctest (section 2.2,
  V02-01 re-runs) — both reports are truthful for the commands they ran;
  the divergence is cargo's `--all-targets` doctest exclusion. No finding is
  discarded; Q-FW-02-P2-02 keeps its P2 finding with corrected mechanism.
- **Merged/canonical duplicates retained with backlinks**: Q-WEB-01-P3-01 =
  A-FE-01-P3-02 (ts-rs prettier drift); Q-FLT-01's scenario failures
  reference canonical F-RCT-03-P1-02, F-RCT-05-P1-01/P2-01/P3-01,
  F-RCT-04-P1-02/P2-02, F-LLM-01-P1-01, F-LLM-03-P1-02/P2-01, X-TOL-01-P2-01;
  Q-E2E-01's P1/P2 syntheses reference canonical A-SRF-03-P1-01/P1-02,
  A-TOOL-01-P1-01, F-OPS-01-P1-01, A-HITL-01-P1-02/P2-02/P2-03, A-SRF-02-P1-01,
  A-INT-01-P1-01/P2-01, X-SRF-01-P2-01/P2-02, X-EVT-01-P1-01/P1-02/P2-02.
- **Minority/low-confidence conclusions preserved as open questions**:
  Q-E2E-01-P3-01 (GUI assistant-text fold, low confidence — only absence of a
  cap verified); Q-FLT-02-P1-01 (pause-crash strand, high mechanism
  confidence / medium trigger probability); Q-DEP-01-P3-04 (polars-ops build
  panic, medium — environment-dependent); Q-PERF-01-P3-01 (unbounded fanout
  impact, medium — requires a slow consumer); Q-STA-01-P2-02 (dead
  read-before-edit duplicate, medium). None is erased; each carries its
  confidence and validation links.
- **Stale-commit check**: all consumed reports are anchored at 9b0e0fa /
  b3b2e81; the two re-runs confirmed the anchors still hold (same commits,
  clean trees, identical outcomes). No finding was marked stale by commit
  drift.

---

## 7. Quality-Verification Findings Consolidated For The Roadmap

No new P0/P1/P2 findings are filed by this synthesis task itself (it is a
synthesis; its only new evidence is the gate/doctest reconciliation). The
roadmap-relevant quality items, in correctness-first order:

1. **Doctest contract red + gate blind spot** (Q-FW-02-P2-02): fix
   `src/testing/mod.rs:39`; add a doctest phase to the gate/CI; acceptance =
   `cargo test --doc -p echo_agent --all-features --locked` exits 0 and the
   gate fails on a future stale doc example.
2. **Test-credibility re-basing** (Q-TST-01-P1-01..03 + F-TST-01-P1-01/P1-02):
   chunk-sequence mock scripting; non-streaming loop test family (regression
   for F-RCT-02-P1-01); Anthropic wire fixtures (message_delta, interleaved
   blocks, malformed events); reconcile pipeline.rs:1634 to call-order
   terminals (unblocks F-RCT-04-P1-01). Acceptance = each prescribed fixture
   fails before its fix and passes after.
3. **Second-tier test gaps** (Q-TST-01-P2-01..03, A-FE-03-P3-04):
   compressor assertion tests, revisioned_adapter round-trip, two-producer
   dedupe fixture, chatStore reducer tests.
4. **Example contract** (Q-FW-02-P2-01): add `content-guard` to demo45's
   required-features; sweep all examples.
5. **Gate-hygiene**: GUI boot-composition test (Q-GUI-01-P3-01), ts-rs
   generation wrapper (Q-WEB-01-P3-01), 28 unresolved intra-doc links
   (Q-FW-02-P3-01), advisory gate + dependency bumps (Q-DEP-01-P2-01/P3-06).
6. **Scenario-level fixes** are canonical (writer plan_mode A-TOOL-01-P1-01,
   cron tick F-OPS-01-P1-01, GUI terminals A-SRF-03-P1-01/P1-02 +
   X-EVT-01-P1-02, HITL leaves A-HITL-01-P1-02/P2-02/P2-03, Browser/MCP
   management A-SRF-02-P1-01 + A-INT-01-P1-01/P2-01) — Q-E2E-01's matrix is
   the acceptance baseline per surface.

Deletion targets tied to QA work: the print-only compressor test and the toy
Mutex summary test (replaced by assertion tests), the stale "deferred"
react_smoke.rs claim, the dead `TasksPanel`/`McpManagerPanel` frontend
components (A-FE-03-P3-03), dompurify + @types/dompurify (Q-DEP-01-P3-01).

---

## 8. Coverage And Uncertainty

- The ledger reconciliation is file-system-exact but not semantic: a report
  could in principle describe a command it did not run. This task re-executed
  only the two commands where reports contradicted each other (both matched
  the recorded outcomes); all other exit codes are taken from the immutable
  reports.
- The gate/doctest re-runs used a warm cache; results are keyed by exact
  feature/target sets, so they are valid for the same inputs (a cold rebuild
  would not change exit codes).
- Q-E2E-01 verdicts remain static determinations; the dynamic dimension stays
  `not_run` until credentials and a GUI-capable host exist (V47–V49 record
  the prerequisite checks).
- Advisory posture is a 2026-08-12 snapshot; advisories published after the
  snapshot are out of scope (Q-DEP-01 V03-01).
- Sibling synthesis reports (framework-review.md, application-review.md)
  exist alongside this one; cross-synthesis conflict resolution is a
  later-phase concern, not handled here.

---

## 9. Handoff

- **Downstream tasks may rely on**: the exact ledger (95 tasks / 729
  validations, per-phase); the gate-status table with the corrected
  CI/local consistency conclusion (gates green; doctest contract red and
  invisible to the gate); the mock-隐身衣 test-credibility conclusion with
  its three P1 anchors; the not_run/inconclusive classification (3 + 1,
  all protocol-compliant); the zero-flaky inventory.
- **Reports to read**: this report + S-QA-01
  [V01-01](../validations/S-QA-01/V01-01.md) /
  [V02-01](../validations/S-QA-01/V02-01.md) /
  [V03-01](../validations/S-QA-01/V03-01.md); the 13
  Q-* task reports; F-TST-01 and A-FE-03 for test-credibility detail;
  Q-TST-01 V03-01 for the flaky/platform inventory; Q-DEP-01 V03-01 for the
  advisory-scan method.
- **Stale triggers**: any new commit on either repository (gate verdicts and
  the doctest reproduction), a toolchain change (rustc 1.97.1 assumption),
  changes to `src/testing/mod.rs` / compression.rs (doctest finding), or a
  change to the gate definitions in AGENTS.md.
- **Follow-up task**: S-RDM-01 consumes sections 2, 3, and 7 to build the
  iteration roadmap with the acceptance criteria listed above.
