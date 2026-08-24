# S-QA-01: Quality and validation synthesis

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework external changes and CLI `Cargo.lock` were excluded; this synthesis reads Codex Q reports and writes Codex S-QA reports only.

## Verdict

The Q catalog is report-complete but not executable-gate complete. All 13 Q
tasks have a task report and validation directory, and the frozen evidence graph
contains 191 immutable validation attempts with resolvable links and required
metadata. Five static review tasks are `complete`; eight command/scenario tasks
remain `needs_evidence`. No reviewed commit has a current fully executed
framework, EKO Rust, GUI, frontend, fault-injection, or multi-surface parity gate.

The correct reading is therefore:

- static topology, source invariants, test credibility, dependency inventory,
  performance lifecycle and documentation drift are extensively evidenced;
- compile/test/build/fault/E2E acceptance remains unknown, not green;
- `failed` static validations often mean an invariant was disproved while the
  inspection command itself exited 0; they must not be counted as 44 failing
  shell commands;
- `not_run` reports are explicit evidence gaps, not skipped or passing commands.

## Scope And Inputs

- Exact Phase-Q catalog in `TASKS.md`: Q-FW-01/02, Q-CLI-01, Q-GUI-01,
  Q-WEB-01, Q-STA-01, Q-TST-01, Q-DEP-01, Q-PERF-01, Q-DOC-01,
  Q-FLT-01/02 and Q-E2E-01.
- All corresponding Codex task reports and all validation reports linked or
  stored beneath those 13 IDs.
- `Q-DISK-01` is auxiliary disk-monitoring evidence, not a catalog task, and is
  excluded from catalog totals.
- No atomic source finding was re-proved or copied. Existing IDs are referenced
  only to preserve ownership and organize future validation.

## Evidence Ledger

| Task | Task status | Attempts | Passed | Failed | Inconclusive | Not run |
|---|---|---:|---:|---:|---:|---:|
| Q-FW-01 | needs_evidence | 12 | 3 | 4 | 0 | 5 |
| Q-FW-02 | needs_evidence | 9 | 3 | 3 | 0 | 3 |
| Q-CLI-01 | needs_evidence | 10 | 5 | 0 | 0 | 5 |
| Q-GUI-01 | needs_evidence | 14 | 7 | 2 | 3 | 2 |
| Q-WEB-01 | needs_evidence | 9 | 5 | 1 | 0 | 3 |
| Q-STA-01 | complete | 30 | 10 | 9 | 10 | 1 |
| Q-TST-01 | complete | 13 | 6 | 6 | 0 | 1 |
| Q-DEP-01 | complete | 14 | 6 | 4 | 3 | 1 |
| Q-PERF-01 | complete | 13 | 6 | 5 | 0 | 2 |
| Q-DOC-01 | complete | 13 | 3 | 9 | 0 | 1 |
| Q-FLT-01 | needs_evidence | 13 | 3 | 0 | 0 | 10 |
| Q-FLT-02 | needs_evidence | 13 | 3 | 0 | 0 | 10 |
| Q-E2E-01 | needs_evidence | 28 | 4 | 1 | 0 | 23 |
| **Total** | **5 complete / 8 needs_evidence** | **191** | **64** | **44** | **16** | **67** |

Counts are file-header attempt counts. They are not command counts. The reporting
protocol permits one indivisible static scenario to use several read commands,
and several Q tasks intentionally compress a future command matrix into one
`not_run` planning report. In particular, Q-FW-02's three `not_run` reports
describe at least 64 feature/library checks, 27 example groups and eight package
doctest commands. The future executable backlog is therefore materially larger
than 67.

## Executable Evidence Gap

| Gate family | Required current evidence | Frozen disposition |
|---|---|---|
| Framework submission | fmt, all-target/all-feature Clippy, panic Clippy, all-target/all-feature tests, no-default library check | 5 not_run; static CI/docs drift found |
| Framework public matrix | independent feature checks, 27 example groups, 8 package doctests | compressed into 3 not_run reports; historical passes are not current |
| EKO Rust submission | fmt, two Clippy modes, all-feature tests, app-core no-default | 5 not_run; committed no-SQLite selection statically confirmed |
| GUI | `gui && !tui` bin check and GUI-only tests | 2 not_run; CI lacks this exact matrix |
| Frontend | Prettier, Vitest, production build | 3 not_run; Node floor mismatch must be fixed/pinned first |
| ReAct/Tool faults | 10 deterministic fault families | 10 not_run |
| Task/Subagent faults | 10 deterministic fault families | 10 not_run |
| Multi-surface parity | 23 scenario/surface pairs | 23 not_run; typed noninteractive adapter absent for one pair |
| Static-review regressions | advisory DB, performance fixtures, doc commands, mutation/negative controls, safety probes | 6 not_run; static findings remain valid within their scope |

No static success can substitute for these gates. Conversely, running every
command once is not sufficient to close test-credibility findings: Q-TST-01
requires mounted frontend transport fixtures, an active known-red ReAct
regression, supported-platform compile lanes and production-connected cache
propagation tests with negative controls.

## Quality Risk Register

The 24 Q-level atomic findings remain canonical in their task reports:

| Priority | Count | Canonical IDs and quality theme |
|---|---:|---|
| P1 | 8 | `Q-FW-01-P1-01`, `Q-TST-01-P1-01..02`, `Q-DEP-01-P1-01`, `Q-PERF-01-P1-01`, `Q-STA-01-P1-01..03` - missing mandatory target execution, non-credible critical tests, JWT contract failure, persistence backpressure, panic/unsafe/overflow paths |
| P2 | 12 | `Q-FW-01-P2-02`, `Q-GUI-01-P2-01`, `Q-WEB-01-P2-01`, `Q-TST-01-P2-03..04`, `Q-DEP-01-P2-02`, `Q-PERF-01-P2-01..02`, `Q-DOC-01-P2-01..04` - gate/config/toolchain drift, platform/test gaps, policy absence, unbounded lifecycle, false operator contracts |
| P3 | 4 | `Q-GUI-01-P3-02`, `Q-DEP-01-P3-03`, `Q-DOC-01-P3-01`, `Q-STA-01-P3-04` - duplicate/stale configuration, dependency and documentation cleanup |

This synthesis does not merge these with framework/application/cross-contract
findings; `S-RDM-01` owns final canonical deduplication and prioritization.

## Finding

### S-QA-01-P2-01: Validation filenames cannot reliably encode immutable retry lineage

- Priority: P2
- Confidence: high
- Layer: adapter (review evidence protocol)
- Evidence: Q-FW/Q-DEP corrected reports use `V01-02.md` as `V01 / Attempt 02`;
  Q-STA and Q-GUI also use suffixes as distinct IDs such as
  `V00-02 / Attempt 01` and `V03-05 / Attempt 02`.
- Reachability: synthesis, primary acceptance and roadmap automation enumerate
  filenames to count attempts, failures and superseding evidence.
- Expected invariant: validation identity and attempt number are separate,
  unambiguous immutable fields; a rerun creates the next attempt for the same ID.
- Observed behavior: the same filename shape represents either a separate
  validation or a retry, and some headings repeat the suffix inside the ID.
- Impact: tooling cannot safely infer attempt lineage, latest corrected evidence
  or retry counts from paths; humans must reconstruct them from task matrices and prose.
- Root cause: validation subcase numbering and attempt numbering share one
  two-part filename namespace without a schema-enforced metadata field.
- Direction: keep historical files immutable. For new reports, use an explicit
  validation key plus explicit numeric attempt metadata (and preferably a small
  machine-readable ledger); enforce path/header/task-link consistency without
  renaming old evidence.
- Regression validation: feed fixtures covering one validation with three
  attempts and three independent subcases into the report linter; assert unique
  lineage, deterministic latest-attempt selection and preserved failed history.
- Validation reports: [V04](../validations/S-QA-01/V04-01.md), [V05](../validations/S-QA-01/V05-01.md)

## Flaky, Failed And Inconclusive Classification

- No Q report establishes a flaky test or command. No executable test/build ran
  during this review, so flakiness is unmeasured.
- The 44 failed attempts include static invariant failures, deliberate negative
  controls, integrity-script failures and a small number of nonzero read-only
  probes. They are evidence, not a count of broken builds.
- The 16 inconclusive attempts are preserved isolation/path/counting or
  dependency-policy limitations. Corrected evidence exists where task reports
  claim acceptance; excluded attempts remain immutable.
- Q-DEP's current advisory result remains unknown without network-backed data.
  Its static absence-of-policy conclusion is separate and source-conclusive.
- Q-GUI capability authority has medium confidence until generated Tauri context
  is observed; its current finding claims duplicate authority drift, not a proven
  runtime permission failure.

## Recommended Execution Order

1. Establish reproducible inputs: isolated clean checkouts at both reviewed
   commits, pin the CLI's framework revision, pin a Node/npm line compatible
   with the lock, and record native prerequisites.
2. Run fast deterministic submission gates: framework and EKO fmt/Clippy/tests/
   minimal checks, frontend Prettier/tests/build, then GUI-only check/tests.
3. Run framework feature/example/doctest matrix with one immutable report per
   actual command. Preserve historical results as historical only.
4. Fix/activate credibility prerequisites: known-red ReAct terminal test,
   mounted frontend transport harness, production-connected cache test and
   supported-platform compile lanes.
5. Execute Q-FLT-01 and Q-FLT-02 deterministic local scenarios; use scripted
   providers/tools/clocks/stores, not network services.
6. Execute Q-E2E pairs after underlying adapter blockers and GUI/Web gates close;
   external credentials are optional compatibility evidence, not substitutes.
7. Run advisory/license/network compatibility checks separately and record
   environment failures rather than silently skipping them.

Every execution creates a new attempt and never edits a `not_run` or failed
report into passed. Any failure must remain visible and be interpreted by its
canonical atomic owner before the next attempt.

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V00-01 | Reserved zsh variable inventory attempt | inconclusive, excluded | [report](../validations/S-QA-01/V00-01.md) |
| V00-02 | Newline catalog iteration attempt | inconclusive, excluded | [report](../validations/S-QA-01/V00-02.md) |
| V01 | Catalog/task status coverage | passed | [report](../validations/S-QA-01/V01-01.md) |
| V02 | Validation attempt/status reconciliation | passed | [report](../validations/S-QA-01/V02-01.md) |
| V03 | Unexecuted matrix audit | failed: current gates absent | [report](../validations/S-QA-01/V03-01.md) |
| V04 | Immutable attempt-lineage consistency | failed -> S-QA P2-01 | [report](../validations/S-QA-01/V04-01.md) |
| V05 | Links, status, executor and exit fields | passed | [report](../validations/S-QA-01/V05-01.md) |
| V99-01 | Initial integrity gate with self-referential pending row | inconclusive, excluded | [report](../validations/S-QA-01/V99-01.md) |
| V99-02 | Pre-write synthesis integrity and terminology gate | passed then superseded | [report](../validations/S-QA-01/V99-02.md) |
| V99-03 | Post-write terminology gate | inconclusive, excluded | [report](../validations/S-QA-01/V99-03.md) |
| V99-04 | Final integrity, count and terminology gate | passed | [report](../validations/S-QA-01/V99-04.md) |
| V30-01 | Primary recount accidentally included auxiliary Q-DISK | failed, excluded | [report](../validations/S-QA-01/V30-01.md) |
| V30-02 | Corrected primary exact-catalog recount and acceptance | passed | [report](../validations/S-QA-01/V30-02.md) |

## Coverage And Uncertainty

- This is a report/evidence synthesis, not a source re-review. It preserves the
  priorities and ownership of Q atomic reports.
- Counts freeze the report tree after Q-E2E V99-01..03 and Q-GUI/Q-WEB primary
  attempts landed. Later attempts make all counts stale and require regeneration.
- Command-result prose is not normalized enough to derive a reliable executed
  shell-command count. Attempt counts and explicit `not_run` matrices are exact;
  future command lower bounds are stated only where task reports enumerate them.
- Product worktrees remain dirty from external implementation. No dynamic gate
  should run there and be attributed to the reviewed commits; use isolated clean
  checkouts or an explicitly accepted new commit pair.

## Handoff

- `S-RDM-01` should treat the eight `needs_evidence` tasks as an ordered
  validation program, not as eight additional product defects.
- Preserve all 24 Q atomic IDs; merge them only with genuinely identical F/A/X
  root causes and retain backlinks.
- Do not claim release readiness until framework, EKO Rust, GUI and frontend
  submission gates pass at one pinned pair and the high-value deterministic
  fault/parity scenarios cover terminal, cancellation, recovery and artifact facts.
- Regenerate V02/V03 counts whenever any Q validation file or task status changes.
- This synthesis deliverable is complete. The eight executable Q task reports
  remain `needs_evidence`; no release, merge or runtime-readiness claim follows.
