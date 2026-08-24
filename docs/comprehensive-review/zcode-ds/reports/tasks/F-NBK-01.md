# F-NBK-01: Notebook and structured working artifacts

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories; verified `git status --porcelain` empty)

## Question

Is the notebook capability a coherent, reachable framework API with stable
cell/artifact semantics rather than an isolated or aspirational path?

**Answer: No.** The notebook capability is an isolated/aspirational API
surface: definition and facade registration exist (`pub mod notebook;` and an
`AgentConfig::enable_notebook` builder), but there is zero runtime
reachability — no construction site, no `record_cell` caller, no reader of
`enable_notebook`, no tool registration, no example, no test, and no
documentation. The cell semantics that exist are sound in isolation
(sequential stable indices, insertion order, UTF-8-safe truncation, poison
recovery), but there is no persistence or artifact mapping of any kind. The
EKO side mirrors this: `ArtifactKind::Notebook` is a reserved variant with no
producer. The live "structured working artifact" mechanisms are elsewhere
(framework trace `RunStore`, workflow `data_pipeline`, EKO file-backed
`analysis.rs`), and EKO's own analyst prompts explicitly forbid an in-memory
notebook as source of truth.

## Scope

- `echo-agent/src/notebook/mod.rs` (full read, 113 lines).
- Registration points: `echo-agent/src/lib.rs:50`, `echo-agent/src/agent/
  config.rs:171-174, 807-816`; prelude/advanced export check.
- Related tool registration: all registries searched for notebook-named
  tools.
- EKO side: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  types.rs` (`ArtifactKind`), `file_store.rs:239-242`, `store.rs:2254`,
  frontend `generated/ArtifactKind.ts`.
- Live neighbors for the duplicate search: `src/trace` (`RunStore`/
  `JsonlRunStore`), `echo-orchestration/src/workflow/pipelines/
  data_pipeline.rs`, `echo-agent-cli/echo-agent-app-core/src/analysis.rs`.
- Docs: root `docs/MASTER-PLAN.md:81,848`, `echo-agent/AUDIT_REPORT.md:340-349`,
  `echo-agent-cli/docs/MASTER-PLAN.md:88`, `echo-agent-cli/docs/
  2026-07-18-file-backed-analysis-workbench.md`, analyst prompts
  (`profiles.rs:213-218`, `subagents/data/analyst.md`),
  `echo-agent/docs/en/03-memory.md:11`, both READMEs.

## Out Of Scope

- Correctness of the trace/`RunStore` subsystem (`F-OPS-01` complete).
- EKO analysis workbench behavior (`A-DOM-01`).
- Tool artifact contract (`F-EXT-01` complete — the artifact writer is the
  live framework mechanism for bounded output, unrelated to notebook cells).
- TaskRuntime artifact retention (`A-TSK-06`).
- Real doctest compilation of the whole crate (`Q-FW-02`).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md` (F-NBK-01
  card), `zcode-ds/README.md`.
- Dependency task reports read: zcode-ds `F-API-01` (complete) and
  `F-EXT-01` (complete); plus zcode-ds `B-ARCH-01`/`B-DOC-01` cross-checked
  for the module list and the AUDIT 2.1 classification (not copied).
- Historical documents treated as hypotheses: `AUDIT_REPORT.md` (2.1), root
  `docs/MASTER-PLAN.md`, `echo-agent-cli/docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/2026-07-18-file-backed-analysis-workbench.md`.

## Layering Decision

- Generic mechanism: a hypothetical live "record analysis steps for
  reproducibility" tracker could be a generic framework capability, but the
  framework already owns the generic record/persist mechanics via `src/trace`
  (`RunStore`/`JsonlRunStore`, file-based JSONL) and the workflow
  `data_pipeline` (code-first reproducible analysis). The notebook module is a
  third, in-memory-only, unpersisted surface for the same semantic.
- EKO product policy: structured working artifacts are the file-backed
  analysis contract (`analysis.rs`: `manifest.json` + script + `runs/`) and
  task artifacts (`ArtifactKind`). EKO prompts explicitly direct agents to
  treat the file-backed record as source of truth and never an in-memory-only
  notebook (`profiles.rs:218`, `subagents/data/analyst.md:16`). Root
  MASTER-PLAN `:848` confirms notebook/报告 experience lives in the EKO layer
  with no notebook kernel.
- Adapter boundary: none — there is no adapter; there is also no reachable
  call path at all.
- Duplicate-search terms (both repositories): `notebook`, `NotebookTracker`,
  `NotebookCell`, `record_cell`, `enable_notebook`, `*Cell` (struct/enum/type),
  `reproducib*`, `export_markdown`, `export_json`, `step_index`,
  `ArtifactKind`, `NotebookPanel`. Result: exactly one notebook definition; no
  parallel notebook implementation in `echo-agent-cli`; the live mechanisms
  (trace, data_pipeline, analysis.rs) are semantic neighbors with different
  API shapes, not duplicates; `ArtifactKind::Notebook` is a reserved variant
  with no producer.

## Current Path

Definition -> registration -> reachability trace (V02):

- `src/notebook/mod.rs:13` `NotebookCell` (step_index, tool_name,
  input_summary, output_summary, duration_ms, timestamp) and `:31`
  `NotebookTracker` (`Arc<RwLock<Vec<NotebookCell>>>`) with
  `record_cell`/`cells`/`export_markdown`/`export_json`/`len`/`is_empty`.
- `src/lib.rs:50` `pub mod notebook;` — unconditional public facade module,
  not in `prelude` (lib.rs:137+) or `advanced`.
- `src/agent/config.rs:174` `enable_notebook: bool` (pub(crate)) + `:815`
  `pub fn enable_notebook` on public `AgentConfig` (`config.rs:44`) — the only
  documented entry point.
- Reachability outcome: `enable_notebook` has zero readers; `NotebookTracker`
  has zero construction sites; `record_cell` has zero callers; no tool
  registration; no example; no test; no doc. Git history (`a5feccf`,
  v0.2.1-dev) shows the API was never referenced by any later commit — it was
  aspirational from birth (V02).
- EKO: `ArtifactKind::Notebook` (`types.rs:567`) round-trips in
  `as_str`/`from_str` (`:579,591`) and can be parsed from a JSON artifact
  record (`file_store.rs:239-242`) but is never constructed; frontend
  `generated/ArtifactKind.ts:7` carries the generated `'notebook'` union
  member; no `NotebookPanel` exists.

## Findings

### F-NBK-01-P2-01: Notebook capability is an aspirational/dead framework API — public module plus config flag whose documented contract no code honors

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/notebook/mod.rs:1-113` (whole module);
  `echo-agent/src/lib.rs:50`; `echo-agent/src/agent/config.rs:171-174`
  (field + doc "each tool invocation is recorded as a NotebookCell"),
  `:807-816` (public builder `enable_notebook`); zero hits for
  `enable_notebook` readers, `NotebookTracker::` construction, and
  `record_cell` callers across both repositories (V02); git history: all three
  symbols introduced together in `a5feccf` and never referenced since; module
  last touched `c1ae71f` (2026-07-09) by a time-serialization change only.
- Reachability: definition yes (module compiles unconditionally, V04-01/02);
  facade registration yes (lib.rs:50, public builder); runtime registration
  no (no tool, no construction); live callers no; exercised behavior no
  (V04-03: 0 tests match). Per REPORTING.md the three claims are separate:
  definition and registration are satisfied, reachability and behavior are
  not — this is a dead/aspirational API face, not an intentional exclusion.
- Expected invariant: a public `AgentConfig` option that documents
  "when enabled, each tool invocation is recorded as a NotebookCell" must
  either be honored by the tool pipeline or not be advertised; a facade module
  should be reachable or absent.
- Observed behavior: `.enable_notebook(true)` compiles and returns an
  `AgentConfig` whose flag is never read — a silent no-op; `NotebookTracker`
  is public API that nothing can reach in a real run; the module is entirely
  undocumented (no README, no guide, no MASTER-PLAN entry — V05).
- Impact: (a) a framework consumer adopting the documented contract gets
  silent no-op recording and loses all cells at process exit (in-memory only,
  see P3-01); (b) the public facade advertises a "reproducibility" surface
  that competes conceptually with the live `trace`/`RunStore` and workflow
  `data_pipeline` authorities without being wired to either; (c) per
  AGENTS.md cleanup rules the module is a candidate for deletion, but its
  public status means it must be deliberately wired or deliberately removed —
  leaving it is a maintenance and contract hazard.
- Root cause: the tracker was added in the v0.2.1-dev feature batch
  (`a5feccf`) as a self-contained record/export utility with a config flag,
  but the tool-execution pipeline integration was never written; later
  milestones built reproducibility on the trace JSONL store and the
  file-backed analysis contract instead, leaving the notebook module without
  a consumer. The poison-panic audit fix (`61730cc`, AUDIT 2.1) touched only
  the module internals, not its wiring.
- Direction (decision belongs to X-BND-01): either (1) delete — remove
  `pub mod notebook;` (lib.rs:50), the `enable_notebook` field (config.rs:174)
  and builder (config.rs:807-816), and the module file, per AGENTS.md cleanup
  (no consumers anywhere, and EKO policy explicitly rejects in-memory-only
  notebooks as source of truth); or (2) wire — construct a tracker when
  `enable_notebook` is set, record cells from the tool pipeline (or map onto
  the existing trace run events), persist via a file store, add an example,
  tests, and docs. Given the product direction and the existence of trace
  `RunStore` + `analysis.rs`, deletion is the lower-cost and lower-risk
  option; if cell-level recording is wanted, it should reuse the trace
  record/event machinery rather than a parallel in-memory tracker.
- Regression validation: after deletion — `cargo check -p echo_agent
  --locked` and `cargo check -p echo_agent --no-default-features --locked`
  exit 0 (same commands as V04-01/02); grep for `NotebookTracker|NotebookCell|
  enable_notebook` returns zero hits; `cargo doc` builds. After wiring —
  a mocked tool turn with `enable_notebook(true)` asserts one cell per tool
  call with sequential `step_index`, plus a restart persistence round-trip.
- Validation reports: [V02-01](../validations/F-NBK-01/V02-01.md),
  [V04-01](../validations/F-NBK-01/V04-01.md),
  [V04-02](../validations/F-NBK-01/V04-02.md),
  [V04-03](../validations/F-NBK-01/V04-03.md)

### F-NBK-01-P3-01: Notebook cells have no persistence or artifact mapping — Serialize/Deserialize unused and exports silently discard data

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/notebook/mod.rs:13` (`derive(Serialize,
  Deserialize)` on `NotebookCell`), `:25` (local-RFC3339 timestamp
  serializer, resolving via `src/utils/mod.rs:2` re-export of
  `echo_core::utils::time`), `:71-90` (`export_markdown`/`export_json` as
  in-memory string snapshots), `:89` (`serde_json::to_string_pretty(...).
  unwrap_or_else(|_| "[]".to_string())`); zero persistence/deserialization
  sites and zero export consumers in both repositories (V01/V03).
- Reachability: only reachable through the unexercised `NotebookTracker`
  (P2-01); no run, task, or session path ever persists a cell.
- Expected invariant: a tracker documented as "export the full session as
  Markdown or JSON for reproducibility and sharing" must either persist its
  cells or be an explicitly in-memory scratchpad; a failed export must not
  silently masquerade as an empty notebook.
- Observed behavior: cells exist only for the object's lifetime; the derives
  are dead; `export_json` returns `"[]"` on serialization error, discarding
  the session record without any error signal; there is no cell ID, so the
  only identity is the positional `step_index`.
- Impact: if a future consumer adopts the API per its docs (reproducibility),
  restart loses everything and a serialization failure is invisible; the
  "[]"-fallback makes data loss look like an empty notebook. Low today
  because nothing calls it (P2-01 governs), but it is the concrete persistence
  gap the task card's persistence/artifact mapping validation must record.
- Root cause: the module was written as a standalone utility before any
  persistence decision; the file-backed analysis contract (EKO) and JSONL run
  store (framework) later became the persistence authorities, and the
  notebook module was never retrofitted.
- Direction: resolved by the P2-01 decision — deletion removes this entirely;
  if wiring instead, add a file-backed store for cells, remove the `"[]"`
  fallback (return `Result`), and add a cell identity field.
- Regression validation: (wiring case) cell round-trip across process
  restart; `export_json` failure test asserts an error is surfaced, never an
  empty array.
- Validation reports: [V03-01](../validations/F-NBK-01/V03-01.md),
  [V01-01](../validations/F-NBK-01/V01-01.md)

### F-NBK-01-P3-02: `ArtifactKind::Notebook` is a reserved variant with no producer — the EKO artifact mapping for notebooks is aspirational

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  types.rs:567` (variant), `:579`/`:591` (`as_str`/`from_str` round-trip);
  `file_store.rs:239-242` (parses `kind` from a stored artifact record,
  defaulting to `File`); `store.rs:2254` (constructs `ArtifactKind::Trace`
  for run records — the only construction site for any variant other than
  parsing); frontend `web-frontend/src/generated/ArtifactKind.ts:7`.
- Reachability: the variant round-trips through serialization (a JSON
  artifact record containing `"kind":"notebook"` would parse), but no code
  path in either repository ever constructs it; the generated TS type exposes
  `'notebook'` to the GUI with no producing backend.
- Expected invariant: an `ArtifactKind` variant represents an artifact
  category the system can actually produce (per A-TSK-06 artifact contract),
  or it does not exist.
- Observed behavior: `Notebook` is a placeholder that only exists as string
  round-trip; no task, run, analysis, or tool path emits it.
- Impact: minor today — harmless serialization surface, but it advertises a
  notebook artifact category that cannot occur, and the frontend
  `'notebook'` union member can never be received from the backend;
  a reviewer/consumer reading the type would overestimate the capability.
- Root cause: the variant was added as part of the TaskRuntime artifact
  model's initial category list, anticipating a notebook artifact that the
  file-backed analysis workbench later superseded; nobody pruned it.
- Direction: either remove the variant (and regenerate `ArtifactKind.ts` via
  the TS export) or keep it only when a notebook-artifact producer exists;
  do not treat it as a live capability in docs or roadmap.
- Regression validation: `cargo check -p echo-agent-app-core --locked` after
  removal; grep for `ArtifactKind::Notebook` and `'notebook'` in
  `ArtifactKind.ts` returns zero hits; frontend `npx tsc -b` stays green.
- Validation reports: [V02-01](../validations/F-NBK-01/V02-01.md),
  [V01-01](../validations/F-NBK-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (both repos) | yes | passed | [V01-01](../validations/F-NBK-01/V01-01.md) |
| V02 | Registration and runtime reachability trace | yes | failed (reachability absent) | [V02-01](../validations/F-NBK-01/V02-01.md) |
| V03 | Invariant/edge cases (persistence mapping, malformed cell, execution order) | yes | passed | [V03-01](../validations/F-NBK-01/V03-01.md) |
| V04 | `cargo check -p echo_agent --locked` (default features) | yes | passed (exit 0) | [V04-01](../validations/F-NBK-01/V04-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --locked` | yes | passed (exit 0) | [V04-02](../validations/F-NBK-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib notebook --locked` | yes | passed (exit 0; 0 tests match) | [V04-03](../validations/F-NBK-01/V04-03.md) |
| V05 | Historical-document drift check | yes | passed | [V05-01](../validations/F-NBK-01/V05-01.md) |

The failed V02 (reachability) is the finding itself, not an execution error:
the trace ran to completion and its conclusion is negative evidence.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `AUDIT_REPORT.md:340-349` 2.1 "RwLock Poison Panics in Notebook Module" (lines 50, 64, 69, 83, 89) | fixed | poison recovery at `src/notebook/mod.rs:52,67,73,88,95`; matches zcode-ds B-DOC-01 V01-01:53; [V03-01](../validations/F-NBK-01/V03-01.md) |
| Root `docs/MASTER-PLAN.md:81,848`: Notebook/报告 UI is EKO product layer; framework provides generic primitives only, no notebook kernel | current | no kernel in either repo; EKO analysis is file-backed; [V05-01](../validations/F-NBK-01/V05-01.md) |
| `echo-agent-cli/docs/MASTER-PLAN.md:88`: EKO does not add a second notebook kernel / statistical DSL | current | [V05-01](../validations/F-NBK-01/V05-01.md) |
| `echo-agent-cli/docs/2026-07-18-file-backed-analysis-workbench.md:28,31`: NotebookPanel is a localStorage-only stub; `notebooks/analysis.md` template exists | stale (historical audit) / forward contract current | no `NotebookPanel` and no `notebooks/analysis.md` in current code; `analysis.rs` implements the manifest contract; [V05-01](../validations/F-NBK-01/V05-01.md) |
| Analyst prompts (`profiles.rs:218`, `analyst.md:16`): do not make an in-memory-only notebook the source of truth | current | live prompt text; product direction corroborated; [V05-01](../validations/F-NBK-01/V05-01.md) |
| `echo-agent/docs/en/03-memory.md:11`: "Notebook" analogy for long-term `Store` | not drift (analogy) | [V05-01](../validations/F-NBK-01/V05-01.md) |
| Framework notebook API exists as a documented feature | no such claim in any doc — the API is undocumented | zero README/guide/MASTER-PLAN hits; [V05-01](../validations/F-NBK-01/V05-01.md) |

## Coverage And Uncertainty

- Static evidence only; no dynamic run exists to observe because there is no
  call path (V02 establishes this exhaustively). No end-to-end "record a
  turn" fixture is possible without wiring, which is out of scope for a
  read-only review.
- `src/trace` and the workflow `data_pipeline` were inspected only for the
  duplicate/reachability comparison; their own correctness is owned by
  F-OPS-01/F-WFL-01.
- The exact commit that removed `NotebookPanel` from the frontend was not
  located (it predates or coincides with the M13 workbench migration); the
  doc's audit section is dated and classified as historical rather than
  live drift.
- `git log -S` history is authoritative for "never referenced since
  introduction" for the three symbols; it cannot prove consumers never
  existed in unmerged branches (irrelevant to current code).
- No feature-gating questions arise: the module is unconditional (V04-02).

## Handoff

- Downstream tasks may rely on: single-definition result (V01); the
  definition/registration-present but reachability/behavior-absent matrix for
  the notebook API (V02); the sound-in-isolation cell invariants and missing
  persistence mapping (V03); compile/test status (V04-01..03); doc
  classifications (V05).
- Reports to read: this report + the 6 validation reports; F-EXT-01 (artifact
  writer as the live framework mechanism), F-API-01 (facade contract context).
- Conditions that make this report stale: any change to `src/notebook/`,
  `src/lib.rs:50`, `src/agent/config.rs` (`enable_notebook`), or the
  TaskRuntime `ArtifactKind` enum; wiring of a notebook recording path;
  deletion of the module.
- Follow-up task IDs (fixes not implemented in this review):
  - `X-BND-01`: decide delete-vs-wire for the notebook module (P2-01) and
    record the EKO notebook-artifact decision (P3-02);
  - `F-API-01`/`Q-DOC-01`: if wiring is chosen, the API must gain docs,
    example, and tests (currently entirely undocumented);
  - `A-DOM-01`: the file-backed analysis workbench is the live
    structured-artifact mechanism; notebook-adjacent product claims there
    must not reference the framework tracker.
