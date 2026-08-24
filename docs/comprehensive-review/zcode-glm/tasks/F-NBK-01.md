# F-NBK-01: Notebook and structured working artifacts

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (9b0e0faf74d35c9a432370b923acabfbb5f32d63)
> `echo-agent-cli` commit: b3b2e81 (b3b2e81f2b2d9fdb319ec604a561beec5f66fea5)
> Worktree state: clean

## Question

Is the notebook capability a coherent, reachable framework API with stable
cell/artifact semantics rather than an isolated or aspirational path?

**Answer: No.** The notebook module is an isolated, aspirational stub. It
compiles and is `pub`, but nothing constructs the tracker, nothing calls any
tracker method, and the `enable_notebook` switch on `AgentConfig` is a
write-only field. Its "cell" semantics are a flat in-memory tool-call log,
not the Jupyter-style cell model its doc comment invokes, and there is no
persistence or artifact-pipeline integration. The principal question's
"coherent, reachable framework API with stable cell/artifact semantics"
branch is unsatisfied on every clause.

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/notebook/mod.rs` — the entire module (112 lines):
  `NotebookCell`, `NotebookTracker`, `record_cell`, `cells()`,
  `export_markdown`, `export_json`, `len`, `is_empty`, `Default`.
- `echo-agent/src/lib.rs:50` — `pub mod notebook;` declaration; absence
  from `prelude` (`lib.rs:137-278`) and `advanced` (`lib.rs:279-331`).
- `echo-agent/src/agent/config.rs:44,171-174,263,807-818` — `AgentConfig`
  struct, `enable_notebook` field, its default, and the `pub fn
  enable_notebook()` builder.
- `echo-agent/Cargo.toml` `[features]` — feature inventory (no `notebook`
  feature; module is always compiled).
- `echo-agent/README.md`, `echo-agent/docs/`, `echo-agent/examples/` —
  documentation / example surface for the capability.
- `echo-agent/AUDIT_REPORT.md:340-356` — historical "RwLock Poison Panics
  in Notebook Module" finding.
- Cross-repo disambiguation in
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:565-594`
  and `.../profiles.rs:213-270` — confirmed unrelated same-name concept
  (`ArtifactKind::Notebook` data-science category label; DomainProfile
  prompt prose).

## Out Of Scope

Deferred to named task IDs:

- Wiring decision (build out the tracker vs delete it) — a product-level
  decision; this review only establishes that the current state is a stub.
  If built out, ownership of the cells (framework vs application) must be
  revisited per the AGENTS.md framework-vs-application gate.
- Long-term memory `Store` semantics — owned by F-MEM-01. The
  `docs/en/03-memory.md` "Notebook" metaphor refers to that surface, not
  to `src/notebook/`.
- Bounded-output / artifact pipeline internals — owned by F-EXT-01 (the
  `ToolOutputArtifactWriter` / `ToolOutputArtifactRef` contract). This
  task only confirms the notebook does not participate in it.
- Tool-call instrumentation generally — owned by F-RCT-03/F-RCT-04
  (streaming / runtime tool execution). This task only notes the absence
  of a `record_cell` call site in that path.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (via system
  reminder — framework-vs-application gate, "first check if it already
  exists", UTF-8 safety, no-panic rule, no-backward-compat cleanup rule,
  no-parallel-implementations rule).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `tasks/F-API-01.md` — established the facade map (`prelude`/`advanced`/
  `workspace`) and the rule that a `pub` surface item is retained unless
  framework-wide evidence shows it is obsolete. F-NBK-01 inherits that
  retention bar: a `pub` stub is not auto-deleted, but its reachability
  must be reported honestly.
- `tasks/F-EXT-01.md` — established the bounded-output artifact contract
  (`ToolOutputArtifactWriter` at `echo-core/src/tools/artifact.rs:145`,
  `ToolOutputArtifactRef`, `ToolResult::truncated`). F-NBK-01 checks
  whether the notebook integrates with that contract (it does not).

Historical documents treated as hypotheses:

- `echo-agent/AUDIT_REPORT.md` "2.1 MEDIUM: RwLock Poison Panics in
  Notebook Module" — treated as a hypothesis that the module still panics
  on lock poisoning. V04 falsifies it (the bug is fixed).

## Layering Decision

Per the AGENTS.md framework-vs-application rule, the notebook module is
classified at the **framework** layer (it lives in `echo-agent/src/`, is
`pub`, and advertises itself as a generic reproducibility primitive in its
doc comment). The retention bar of F-API-01 applies: a public framework
item is **not** deleted merely because `echo-agent-cli` does not call it;
it is deleted only when framework-internal + all-reuse-site evidence shows
it is obsolete or fully replaced.

Repository-wide duplicate-search terms used (cross-crate, both repos):

- Type/struct names: `NotebookCell`, `NotebookTracker`.
- Method names: `record_cell`, `export_markdown`, `export_json`.
- Config field/builder: `enable_notebook`.
- Concept terms: `notebook`, `Notebook`, `cell` (narrowed by adjacency to
  `notebook`).

Result of duplicate search:

- **No parallel definition** of the tool-call notebook concept exists.
  The `echo-agent-cli` hits (`ArtifactKind::Notebook`,
  `profiles.rs` prose) are a different concept (data-science artifact
  category label), not a second implementation. The framework notebook is
  the single definition site for its semantics.
- The module is **not** superseded by anything. It is simply unused.

Layering conclusion: the module is correctly placed at the framework layer
**in intent** (a generic reproducibility primitive is a plausible framework
feature), but it is **not yet a live framework API** — it has no consumer
in any repo. Per the AGENTS.md "no parallel implementations" rule there is
no conflict to resolve; per the "first check if it already exists" rule
there is nothing to consolidate. The decision of whether to wire it in or
delete it is deferred (see Handoff).

## Current Path

Verified state at commit `9b0e0fa`:

1. **Definition.** `NotebookCell` and `NotebookTracker` are defined in
   `echo-agent/src/notebook/mod.rs:13,31`. `NotebookTracker` wraps
   `Arc<RwLock<Vec<NotebookCell>>>` (`mod.rs:32`); it is `Clone` and
   `Default`. Six public methods: `new`, `record_cell`, `cells`,
   `export_markdown`, `export_json`, `len`, `is_empty`.

2. **Facade exposure.** `echo-agent/src/lib.rs:50` declares `pub mod
   notebook;`. The module is **not** `cfg`-gated and **not** behind a
   feature (`Cargo.toml` has no `notebook` feature). It is therefore
   always compiled into every build, including `--no-default-features`.
   It is **not** re-exported in `prelude` (`lib.rs:137-278`) or
   `advanced` (`lib.rs:279-331`). The sole consumer import path is the
   literal `echo_agent::notebook::<Item>`.

3. **Config switch.** `AgentConfig` (`src/agent/config.rs:44`, re-exported
   in the prelude at `lib.rs:140`) carries `pub(crate) enable_notebook:
   bool` (`config.rs:174`), defaulted to `false` (`config.rs:263`), set
   via `pub fn enable_notebook(mut self, enable: bool) -> Self`
   (`config.rs:815-818`).

4. **Reachability dead-end (the core finding).** Whole-workspace grep for
   every notebook symbol, excluding the definition file and the
   config-setter trio (declaration / default / setter body):
   - `NotebookTracker::new()` / `NotebookTracker::default()` construction:
     **zero** callers.
   - `record_cell` / `.cells()` / `export_markdown` / `export_json` /
     `len` / `is_empty`: **zero** callers.
   - `enable_notebook` (the field): **zero** reads. The agent runtime
     (`src/agent/**`) never branches on `self.enable_notebook`.
   - There is no `NotebookTracker` field on `AgentConfig`, `ReactAgent`,
     or any agent-runtime struct. The runtime owns the boolean flag only,
   and the flag is write-only.

5. **Tool registration.** No tool in `src/tools/` or any sub-crate
   references `NotebookTracker`. The tool-execution path
   (`echo-core/src/tools/mod.rs::Tool::execute_with_context` and the
   ReactAgent tool-result handling, per F-EXT-01) does not call
   `record_cell`. The "each tool invocation is recorded as a NotebookCell"
   promise in the builder doc (`config.rs:813`) is unfulfilled.

6. **Persistence / artifact path.** The tracker is purely in-memory
   (`Arc<RwLock<Vec<...>>>`). `export_markdown` / `export_json` return a
   `String`; no `Store`, file, `RuntimeStateStore`, or
   `ToolOutputArtifactWriter` write occurs. See V02.

7. **Cross-repo disambiguation.** The `echo-agent-cli` "notebook" hits
   (`ArtifactKind::Notebook`, DomainProfile prose) are a data-science
   artifact category and prompt wording. They do not construct or
   reference `echo_agent::notebook::NotebookTracker`. The framework
   notebook is unreachable from the application too.

## Findings

### F-NBK-01-P1-01: The notebook capability is an unreachable, aspirational stub with zero live callers

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/notebook/mod.rs:13-112` — full module; defines
    `NotebookCell`, `NotebookTracker`, and six public methods.
  - `echo-agent/src/lib.rs:50` — `pub mod notebook;` (always compiled,
    not feature-gated).
  - Whole-workspace grep (both repos, `*.rs`): excluding the definition
    file and the `enable_notebook` setter trio, there are **zero**
    references to `NotebookTracker`, `NotebookCell`, `record_cell`,
    `cells()`, `export_markdown`, `export_json`, `len`, or `is_empty`.
    See V01-01 for the exact grep.
  - `echo-agent/src/agent/config.rs:174,263,816` — `enable_notebook`
    appears only as declaration, default, and setter body. **No read.**
- Reachability: definition (`src/notebook/mod.rs`) → facade (`pub mod
  notebook;` at `lib.rs:50`) → **no live caller**. The `pub` keyword
  makes the item survive `cargo check` (it is not dead-code-eliminated),
  so the gap is invisible to compilation. It is visible only to a
  reachability grep.
- Expected invariant (per the principal question): a "coherent, reachable
  framework API" has at least one construction site owned by the runtime
  or by example/README code, and at least one read of its enable flag.
- Observed behavior: none of those exist. The module is a closed loop:
  it defines an API that calls itself nowhere.
- Impact: the capability is advertised by its `pub` surface and by the
  `enable_notebook` builder doc but performs no work. A consumer who
  imports `echo_agent::notebook::NotebookTracker` or calls
  `.enable_notebook(true)` on the agent builder gets silent no-ops. This
  is the direct negation of the principal question.
- Root cause: the module was authored ahead of its integration (the
  builder doc and module doc both describe the intended end state), but
  the integration into the ReactAgent tool-execution path was never
  completed. The `enable_notebook` flag was added to `AgentConfig` as
  the intended seam, then never read.
- Direction: product-level decision. Either (a) wire it: give
  `AgentConfig`/`ReactAgent` an `Option<Arc<NotebookTracker>>` field
  constructed when `enable_notebook` is true, call `record_cell` after
  each tool execution in the React loop, and expose `export_*` via a
  callback or a public accessor; or (b) delete the module, the field,
  and the builder (AGENTS.md no-backward-compat cleanup rule permits
  this). Per the framework-vs-application gate, if (a) is chosen, the
  tracker ownership belongs in the framework (it is a generic primitive),
  but any EKO-specific rendering of the export belongs in the
  application adapter.
- Regression validation: under (a), add a test that runs one tool and
  asserts `tracker.cells().len() == 1` and that `export_json()` round-trips
  through `serde_json`; under (b), `cargo test --workspace
  --all-features --locked` and confirm no broken imports (the only import
  path is `echo_agent::notebook::*`, which has zero external users).
- Validation reports: [V01](../validations/F-NBK-01/V01-01.md),
  [V02](../validations/F-NBK-01/V02-01.md),
  [V03](../validations/F-NBK-01/V03-01.md).

### F-NBK-01-P2-01: `pub fn enable_notebook()` is a no-op public setter on a prelude-exported struct

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/config.rs:815-818` — `pub fn enable_notebook(mut
    self, enable: bool) -> Self { self.enable_notebook = enable; self }`.
  - `echo-agent/src/agent/config.rs:44` — `pub struct AgentConfig`.
  - `echo-agent/src/lib.rs:140` — `AgentConfig` is re-exported in the
    prelude.
  - `echo-agent/src/agent/config.rs:807-814` — builder doc: "When
    enabled, each tool invocation is recorded as a `NotebookCell`, and
    the full session can be exported as Markdown or JSON."
  - `enable_notebook` field is never read (see F-NBK-01-P1-01).
- Reachability: the builder is `pub` on a prelude-exported struct; any
  consumer using `echo_agent::prelude::*` can call `.enable_notebook(true)`
  on the agent builder. The call succeeds at compile time and does
  nothing at run time.
- Expected invariant: a `pub` builder method on the agent config either
  performs the documented behaviour or is explicitly documented as a
  no-op/placeholder.
- Observed behavior: the method writes a flag that nothing reads. The
  doc comment promises live recording behaviour. Silent no-op.
- Impact: misleading public API. A consumer building a reproducibility
  feature on this call will ship code that compiles and silently records
  nothing. Worse than dead code (which a consumer would not call) — this
  is *called* code whose effect is zero.
- Root cause: the builder was added as the intended seam for the
  notebook integration; the read site was never implemented (same root
  cause as P1-01).
- Direction: coupled to the P1-01 decision. If the capability is wired,
  the setter becomes live and the doc becomes accurate. If the
  capability is deleted, the setter, the field, and the default must be
  deleted together (AGENTS.md no-backward-compat rule).
- Regression validation: same as P1-01; additionally a doctest on the
  builder that asserts the documented behaviour once wired.
- Validation reports: [V01](../validations/F-NBK-01/V01-01.md),
  [V04](../validations/F-NBK-01/V04-01.md).

### F-NBK-01-P3-01: The "notebook" data model is a flat in-memory tool-call log, not the Jupyter-style cell model the doc invokes

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/notebook/mod.rs:1-5` — module doc: "records agent
    analysis steps for reproducibility … similar to Jupyter notebooks."
  - `echo-agent/src/notebook/mod.rs:13-27` — `NotebookCell` has fields
    `step_index`, `tool_name`, `input_summary`, `output_summary`,
    `duration_ms`, `timestamp`. **No `cell_type` / `kind` discriminator.**
  - `echo-agent/src/notebook/mod.rs:44-62` — `record_cell` appends with
    `step_index = cells.len()`; monotonic and gap-free under the write
    lock (V03), but append-only — no replay, no parent/dependency field.
- Reachability: moot at runtime (no caller — see P1-01), but relevant to
  the "stable cell/artifact semantics" clause of the principal question
  if the capability is revived.
- Expected invariant: a module that frames itself as Jupyter-like would
  distinguish markdown / code / output cells, or at minimum scope its
  doc to "linear tool-call log" so consumers do not expect cell kinds.
- Observed behavior: every cell is the same shape. There is no way to
  record a markdown narrative cell, a code cell distinct from its output,
  or a dependency edge. Out-of-order completion is recorded in completion
  order; the original dependency is lost. The model cannot represent
  what the framework's own revisioned TaskRun graph represents.
- Impact: low today (no consumer). If revived as-is, the model would not
  satisfy the reproducibility use case the doc promises.
- Root cause: the struct models the minimum needed to log a tool call,
  not the richer nbformat-style shape the doc invokes.
- Direction: if revived, add a cell-kind discriminator and an optional
  `parent_id`/`depends_on`; and rewrite the module doc to match the
  implemented shape until the richer model lands. If deleted, moot.
- Regression validation: under revival, tests covering markdown/code/
  output cell round-trip and dependency-tagged replay.
- Validation reports: [V02](../validations/F-NBK-01/V02-01.md),
  [V03](../validations/F-NBK-01/V03-01.md).

### F-NBK-01-P3-02: No persistence, no artifact-pipeline integration, no schema version

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/notebook/mod.rs:32` — backing store is
    `Arc<RwLock<Vec<NotebookCell>>>` (in-memory only).
  - `echo-agent/src/notebook/mod.rs:71,86` — `export_markdown` /
    `export_json` return `String`; the caller (which does not exist)
    would own any durable write.
  - No reference to `ToolOutputArtifactWriter`, `ToolOutputArtifactRef`,
    `Store`, `RuntimeStateStore`, or `ConversationStore` anywhere in
    `src/notebook/` (V02).
  - `NotebookCell` derives `Serialize`/`Deserialize` with no
    `#[serde(...)]` version anchor and no `schema_version` field
    (`mod.rs:12-27`).
- Reachability: moot at runtime (no caller).
- Expected invariant: a reproducibility-oriented artifact either persists
  to a backing store or spills through the bounded-output pipeline, and
  its serialised form carries a version so future field additions are
  detectable.
- Observed behavior: none of those. Dropping the tracker drops every
  cell. Large tool outputs are truncated to 200 chars and the full
  payload is discarded (no artifact ref retained).
- Impact: low today. If revived as-is, "export the full session for
  reproducibility" loses any oversized tool output and survives only
  until the process exits.
- Root cause: the persistence story was never designed; the module stops
  at the in-memory log.
- Direction: if revived, decide the persistence target (`Store`,
  `ToolOutputArtifactWriter`, or a dedicated file), retain a
  `ToolOutputArtifactRef` for truncated outputs, and add a
  `schema_version` field. If deleted, moot.
- Regression validation: under revival, a test that re-opens a persisted
  notebook across a fresh `NotebookTracker` and round-trips the cells.
- Validation reports: [V02](../validations/F-NBK-01/V02-01.md).

### F-NBK-01-P3-03: Module doc and builder doc overstate the implementation (documentation drift, aspirational framing)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/notebook/mod.rs:1-5` — "records agent analysis steps
    for reproducibility … similar to Jupyter notebooks."
  - `echo-agent/src/agent/config.rs:807-814` — "When enabled, each tool
    invocation is recorded as a `NotebookCell`, and the full session can
    be exported as Markdown or JSON."
  - `echo-agent/README.md` — zero matches for "notebook" or
    "reproducib"; the capability is not advertised at all.
  - `echo-agent/examples/` (68 files) — zero references to the notebook.
  - `echo-agent/docs/en/03-memory.md:11` — "Notebook" appears only as the
    analogy column for the long-term `Store` (F-MEM-01 surface), not as a
    reference to `src/notebook/`.
- Reachability: public doc comments on `pub` items; visible to anyone
  browsing rustdoc or reading `AgentConfig`'s builder list.
- Expected invariant: doc comments on `pub` items describe behaviour the
  code performs, or explicitly mark the capability as planned/unstable.
- Observed behavior: the module doc and the builder doc both describe
  live Jupyter-like behaviour; the runtime performs none of it (P1-01).
  The README is silent (internally consistent with the stub, but
  inconsistent with the `pub` surface).
- Impact: consumers reading rustdoc are misled; consumers reading only
  the README never see the capability. Two documentation surfaces
  disagree.
- Root cause: docs were written for the intended end state; the
  implementation did not catch up.
- Direction: coupled to the P1-01 decision. Either make the docs true
  (wire the capability) or make the docs match (delete the module and
  builder, or rewrite the module doc to scope it as a no-op linear log).
- Regression validation: a doctest on the builder that asserts the
  documented behaviour once wired; otherwise a grep that confirms
  zero `notebook` references in `src/` after deletion.
- Validation reports: [V04](../validations/F-NBK-01/V04-01.md).

### F-NBK-01-P3-04 (positive): AUDIT_REPORT.md "RwLock Poison Panics in Notebook Module" is fixed

- Priority: P3 (informational)
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/AUDIT_REPORT.md:340-356` — cites `src/notebook/mod.rs`
    lines 50, 64, 69, 83, 89 and quotes `.write().unwrap()` /
    `.read().unwrap()`; severity MEDIUM.
  - Current source: `src/notebook/mod.rs:52,67,73,88,95,103` all use
    `.unwrap_or_else(|e| e.into_inner())` (poison-recovering). No bare
    `.unwrap()` / `.expect()` on any lock guard in the module.
- Reachability: historical document only; not a live spec.
- Expected invariant: AGENTS.md no-panic rule. The module now satisfies
  it for the lock-poisoning case.
- Observed behavior: the cited bug is resolved. The report is stale.
- Impact: positive — the code improved and the audit did not catch up.
- Root cause: the fix landed without updating the audit report.
- Direction: delete the "2.1 MEDIUM" section from `AUDIT_REPORT.md`
  (AGENTS.md no-backward-compat rule permits this). Pure doc cleanup.
- Regression validation: none beyond the V04 static check.
- Validation reports: [V04](../validations/F-NBK-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Public/reachability map: exports, agent ownership, tool registration, `enable_notebook` read site | yes | failed | [V01-01](../validations/F-NBK-01/V01-01.md) |
| V02 | Persistence/artifact mapping: backing store, cell kinds, artifact-pipeline integration, schema version | yes | failed | [V02-01](../validations/F-NBK-01/V02-01.md) |
| V03 | Malformed cell and execution-order cases: validation, monotonicity, replay/dependency representation | yes | inconclusive | [V03-01](../validations/F-NBK-01/V03-01.md) |
| V04 | Documentation drift: README/docs/examples match code; historical audit finding current | yes | failed | [V04-01](../validations/F-NBK-01/V04-01.md) |

V01/V02/V04 fail because the principal question's "reachable", "stable
cell/artifact semantics", and "documentation contract" clauses are each
negated by the evidence. V03 is inconclusive rather than failed because the
out-of-order / missing-dependency edge cases are not expressible in the
data model — there is no behaviour to falsify, only an absence to record.
The monotonicity and panic-safety properties that *are* expressible pass
(V03 positive observations).

No executable validation (V05-style `cargo` run) was performed: the
conclusion rests on structural facts (zero call sites, zero reads, in-memory
backing store) that a reachability grep establishes deterministically, and
a `cargo check` would not flag the dead `pub` surface.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `src/notebook/mod.rs:1-5` "records agent analysis steps for reproducibility … similar to Jupyter notebooks" | overstated / aspirational | V02-01: in-memory flat tool-call log; no cell kinds, no persistence. F-NBK-01-P3-01/P3-03. |
| `src/agent/config.rs:807-814` builder doc "each tool invocation is recorded as a NotebookCell … exported as Markdown or JSON" | inaccurate (runtime never reads the flag) | V01-01: zero reads of `enable_notebook`; zero `record_cell` call sites. F-NBK-01-P1-01/P2-01. |
| `AUDIT_REPORT.md:340-356` "2.1 MEDIUM: RwLock Poison Panics in Notebook Module" citing `.write().unwrap()` | fixed (stale report) | V04-01: current source uses `.unwrap_or_else(\|e\| e.into_inner())` at `mod.rs:52,67,73,88,95,103`. F-NBK-01-P3-04. |
| `docs/en/03-memory.md:11` "Long-term knowledge \| Store \| Notebook" | coincidental name overlap (not about `src/notebook/`) | V04-01: row describes the `Store` memory backend (F-MEM-01), not the tool-call tracker. |
| `echo-agent-cli` `ArtifactKind::Notebook` (`types.rs:567`) and `profiles.rs` prose | unrelated same-name concept | V01-01: data-science artifact category label + DomainProfile prompt wording; does not reference `NotebookTracker`. |

## Coverage And Uncertainty

Code not inspected:

- `src/agent/` beyond `config.rs` — the ReactAgent runtime, callbacks, and
  subagent paths were spot-checked for any `NotebookTracker` field or
  `record_cell` call (zero hits via grep), but were not read line-by-line.
  Behaviour of the React tool-execution loop belongs to F-RCT-03/F-RCT-04.
- `echo-agent-cli` beyond the cross-repo disambiguation grep — the
  application's notebook-shaped concerns (if any) belong to application-
  layer tasks. The grep confirmed no CLI code references
  `echo_agent::notebook::*`.
- `examples/` beyond the notebook grep — F-API-01 already audited the
  canonical demo00-03 entry points; none reference the notebook.

Validations not run:

- No `cargo build` / `cargo test` / `cargo run --example`. The conclusion
  is a structural reachability fact (zero call sites, zero reads) that a
  grep establishes deterministically; compilation would not change it
  (the `pub` surface compiles cleanly precisely because it is public).

Claims that remain uncertain:

- Whether the notebook was *intended* to be wired in a recent commit that
  was abandoned, or has been dormant since inception, was not established
  from git history (out of scope for a read-only review). The conclusion
  ("stub today") holds regardless.
- The framework-vs-application placement of a *revived* tracker is
  tentatively framework (it is a generic primitive), but the final call
  belongs to the product-level wiring decision (Handoff).

## Handoff

Conclusions downstream tasks may rely on:

- The notebook module at `echo-agent/src/notebook/` is **not** a live
  framework API. It is a `pub` stub with zero construction sites, zero
  call sites, and a write-only `enable_notebook` flag. Any downstream
  task that assumes notebook-based reproducibility exists today is wrong.
- The module is **not** superseded by a parallel implementation and
  **not** in conflict with the framework-vs-application gate. There is
  no duplicate to consolidate; there is only an unused primitive.
- The bounded-output artifact pipeline (F-EXT-01) and the notebook are
  disjoint: the notebook does not spill through
  `ToolOutputArtifactWriter` and does not retain
  `ToolOutputArtifactRef`. A task considering "structured working
  artifacts" should treat them as two unrelated surfaces.
- The `AUDIT_REPORT.md` "2.1 MEDIUM" notebook section is stale and may
  be deleted in any cleanup pass.

Reports downstream tasks must read:

- A future framework-cleanup task deciding the fate of `src/notebook/`
  must read V01-01 (reachability evidence) and the P1-01 direction.
- A documentation-cleanup task touching `AUDIT_REPORT.md` must read
  V04-01 (the RwLock-poison fix).

Conditions that make this report stale:

- Any commit that adds a `NotebookTracker` field to `AgentConfig` or
  `ReactAgent` and reads `enable_notebook` invalidates P1-01, P2-01,
  P3-02, and V01.
- Any commit that calls `record_cell` from the tool-execution path
  invalidates P1-01.
- Any commit that adds persistence (Store / artifact writer / file) to
  the notebook invalidates P3-02 and V02.
- Any commit that adds a cell-kind discriminator or dependency field
  invalidates P3-01 and V03.
- Any commit that deletes `src/notebook/` and the `enable_notebook`
  builder invalidates the entire report (it becomes a historical record
  of a removed stub).

Follow-up task IDs (recommended, not implemented in this review):

- A product-level task to decide "wire vs delete" for the notebook
  capability. This is the only finding that cannot be resolved inside
  this review's read-only scope. If "delete" is chosen, the change is
  small (one module file + one config field + one builder + the stale
  audit section) and is permitted by the AGENTS.md no-backward-compat
  rule since there are zero consumers.
