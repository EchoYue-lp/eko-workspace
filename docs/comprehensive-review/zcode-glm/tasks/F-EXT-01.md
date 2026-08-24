# F-EXT-01: Tool contract, registry, schema, and artifacts

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Is the generic Tool contract typed, cancellable, paginated, and capable of
bounded model output plus complete artifacts?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/tools/mod.rs` — `Tool` trait, `ToolRegistrar`,
  `ToolRunner<P>`, `ToolResult`, `ToolResultKind`, `ToolFailure`,
  `ToolFailureCategory`, `ToolRecoveryAction`, `ToolSideEffect`,
  `ToolExecutionConfig`, `ToolRiskLevel`, `ToolContext`.
- `echo-agent/echo-core/src/tools/artifact.rs` —
  `ToolOutputArtifactConfig`, `ToolOutputArtifactIdentity`,
  `ToolOutputArtifactRef`, `ToolOutputArtifactWriter`.
- `echo-agent/echo-core/src/tools/pagination.rs` — `PageRequest`, `PageInfo`,
  `PageError`.

## Out Of Scope

- Concrete builtin tool implementations (shell, file, code, git) — deferred
  to F-EXT-02.
- Tool permission / risk-gating runtime behavior — deferred to the
  permission / risk-gating task.
- Per-provider tool-call serialization (how `parameters()` JSON Schema is
  mapped onto OpenAI / Anthropic tool-call wire formats) — deferred to
  F-LLM-01 / F-LLM-02 / F-LLM-03.
- Application adapter's tool registry wiring and duplicate-name enforcement
  at construction — flagged in V02 for a downstream registry task.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — framework-vs-application layering gate, dead-code
    cleanup rule, UTF-8 safety, no-panic rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-CORE-01` (this reviewer) — relied on its conclusion that
    `echo-core` exposes a single typed identity/event/error surface and that
    `CancellationToken` is the canonical cancellation primitive threaded
    through the agent runtime. This task extends that conclusion to the tool
    layer.
- Historical documents treated as hypotheses: none.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The `Tool` trait, `ToolResult` / `ToolFailure` taxonomy, cursor pagination, and artifact spill describe generic tool-execution concepts that any `echo-agent` consumer (CLI, third-party headless, future reuse) needs. They live correctly in `echo-core` (V01 confirms single definition site). |
| EKO product policy | None at this layer. `ToolExecutionConfig` (timeout/concurrency/retry), `ToolRiskLevel` classification, and the recovery mapping are product-agnostic; EKO's permission gating is a separate concern (out of scope). `ToolContext::working_dir` / `resolve_path` is a generic session-aware-path primitive, not an EKO-specific policy. |
| Adapter boundary | The framework exposes `Tool` + `ToolResult` as the contract; application tools implement the trait and the application adapter feeds `ToolContext` at execution time. The trait's `execute` vs `execute_with_context` split is the only seam — no product logic leaks into the framework here. |
| Duplicate search | Searched names: `Tool`, `ToolRegistrar`, `ToolRunner`, `ToolResult`, `ToolResultKind`, `ToolFailure`, `ToolFailureCategory`, `ToolRecoveryAction`, `ToolSideEffect`, `ToolExecutionConfig`, `ToolRiskLevel`, `ToolContext`, `PageRequest`, `PageInfo`, `PageError`, `ToolOutputArtifactConfig`, `ToolOutputArtifactIdentity`, `ToolOutputArtifactRef`, `ToolOutputArtifactWriter`. Result: no duplicate definition of the same semantics inside `echo-core/src/tools` for these concerns. |
| Migration deletion | No migration proposed in this task. No deletion candidate identified at the contract layer. |

## Current Path

Verified tool-contract data flow at commit `9b0e0fa`:

1. **Contract surface.** `Tool` trait (`mod.rs:739`) requires `name()`,
   `description()`, `parameters() -> serde_json::Value` (JSON Schema), and
   two executors: `execute(parameters) -> BoxFuture<Result<ToolResult>>` and
   `execute_with_context(params, ctx: &ToolContext) -> BoxFuture<Result<ToolResult>>`.
   `ToolRunner<P = ToolParameters>` (`mod.rs:733`) is the typed-runner
   extension (`Tool + Sized` with parameter type `P`).
   `ToolRegistrar` (`mod.rs:723`) is the registration surface.

2. **Result shape.** `ToolResult` (`mod.rs:288`) is a rich struct:
   `kind: ToolResultKind` discriminator (`Text`/`Data`/`Bytes`/`Error`/
   `Stream`), `success: bool`, `output: String`, `error: Option<String>`,
   `failure: Option<ToolFailure>`, `bytes: Option<Vec<u8>>`,
   `data: Option<serde_json::Value>`, `truncated: bool`,
   `mime_type: Option<String>`, `metadata: HashMap<String, String>`.

3. **Structured failure.** `ToolFailure` (`mod.rs:78`) carries `category:
   ToolFailureCategory` (7 variants, `mod.rs:20`), `recovery:
   ToolRecoveryAction` (5 variants, `mod.rs:47`), `side_effect:
   ToolSideEffect` (`None`/`Possible`/`Confirmed`, `mod.rs:70`),
   `retry_after_ms: Option<u64>`, `idempotency_key: Option<String>`,
   `postcondition: Option<String>`.

4. **Recovery mapping.** The default `category -> recovery` mapping
   (verified in V04) is exhaustive:
   `InvalidArguments -> CorrectArguments`, `Unavailable -> RestoreThenRetry`,
   `Timeout -> VerifyThenRetry`, `PartialSideEffect -> VerifyThenRetry`,
   `Transient -> Retry`, `Cancelled -> Stop`, `Permanent -> Stop`. The
   partial-side-effect and timeout cases route through verification rather
   than blind retry — the key safety pattern.

5. **Cancellation.** Tool execution returns `BoxFuture`, so the runtime's
   `CancellationToken` (established as canonical by F-CORE-01) propagates
   naturally: dropping/cancelling the future aborts the tool, and
   `ToolFailureCategory::Cancelled` is the structured surface for cancelled
   tool results.

6. **Pagination.** `PageRequest` (`pagination.rs:14`) is cursor-based (opaque
   cursor, not offset/limit), `PageInfo` (`pagination.rs:21`) carries
   next-cursor and has-more metadata, `PageError` (`pagination.rs:32`) is
   the typed error channel.

7. **Bounded output.** `ToolOutputArtifactWriter` (`artifact.rs:145`) is the
   single spill entry point: oversized `ToolResult` payloads are written to
   the backing store, identity is stabilized via
   `ToolOutputArtifactIdentity` (`artifact.rs:70`), and consumers hold a
   `ToolOutputArtifactRef` (`artifact.rs:92`). `ToolResult::truncated`
   signals in-band that the artifact holds the full content. Spill behavior
   is configured via `ToolOutputArtifactConfig` (`artifact.rs:29`).

8. **Execution policy.** `ToolExecutionConfig` (`mod.rs:525`) carries
   timeout / concurrency / retry settings; `ToolRiskLevel` (`mod.rs:710`)
   classifies risk. Both are product-agnostic framework primitives.

9. **Context.** `ToolContext` provides `working_dir` and `resolve_path` for
   session-aware execution — the seam through which the application adapter
   injects session state without polluting the trait.

## Findings

### F-EXT-01-P3-01: `ToolResult` exposes parallel output channels (`output: String`, `data: Option<Value>`, `bytes`, `kind`) without documenting which is authoritative per `kind`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/tools/mod.rs:288` — `ToolResult` struct.
  - `kind: ToolResultKind` (`Text`/`Data`/`Bytes`/`Error`/`Stream`) is the
    discriminator, but the struct does not document which payload field a
    consumer must read for each variant.
- Reachability: live. Every tool execution returns a `ToolResult`; every
  consumer (ReactAgent tool-result handling, application adapter, future
  third-party consumer) must decide which field to read.
- Expected invariant: a typed result contract should make it unambiguous
  which field holds the payload for a given `kind`, so consumers can
  pattern-match exhaustively without guessing.
- Observed behavior: a consumer reading a `ToolResult` with `kind == Data`
  must know to read `data`; with `kind == Bytes`, to read `bytes`; with
  `kind == Text`, to read `output`. But nothing in the contract enforces or
  documents this, and `output: String` is always present (not `Option`),
  suggesting it may be a fallback or a human-readable mirror. A tool that
  fills both `output` and `data` leaves the consumer to guess which is
  authoritative.
- Impact: low today (builtin tools and the application adapter follow the
  convention by consistency), but a third-party tool implementor or a new
  consumer could read the wrong field and silently drop structured data.
  Not a correctness defect in audited paths.
- Root cause: documentation gap. The struct grew fields as the contract
  expanded (`data` for structured, `bytes` for binary, `kind` added later as
  a discriminator) without a doc comment pinning the per-`kind` payload
  field.
- Direction: add a doc comment on `ToolResult` (and/or on `ToolResultKind`)
  stating the per-`kind` authoritative field, e.g.
  "`Text -> output`, `Data -> data`, `Bytes -> bytes`, `Error -> error +
  failure`, `Stream -> output is the accumulated text so far`". Optionally,
  a constructor per `kind` that sets only the authoritative field would
  make the invariant structural rather than conventional.
- Regression validation: doc-only change is safe; if constructors are added,
  `cargo test --workspace --all-features` and confirm builtin tools migrate
  cleanly.
- Validation reports: [V01](../validations/F-EXT-01/V01-01.md)

### F-EXT-01-P3-02: Default `ToolFailureCategory -> ToolRecoveryAction` mapping is hardcoded and not per-tool overridable without bypassing `ToolFailure::new`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-core/src/tools/mod.rs:47` — `ToolRecoveryAction`.
  - `echo-agent/echo-core/src/tools/mod.rs:20` — `ToolFailureCategory`.
  - `echo-agent/echo-core/src/tools/mod.rs:78` — `ToolFailure` struct
    (`category` and `recovery` are independent fields, so an explicit
    construction can set any pairing).
  - The default mapping is a single hardcoded `match` used by the default
    `ToolFailure` construction path (verified in V04).
- Reachability: live. Every tool that reports a structured failure without
  explicitly setting `recovery` inherits the hardcoded mapping.
- Expected invariant: a tool whose semantics warrant a non-default recovery
  (e.g. a tool that always carries an `idempotency_key` and is therefore
  safe to blind-retry despite `PartialSideEffect`) should be able to declare
  that policy without manually constructing `ToolFailure` field-by-field.
- Observed behavior: the mapping is a framework-wide constant. A tool that
  wants `PartialSideEffect -> Retry` (because it provides an idempotency
  key and a verify-then-retry gate would be redundant) cannot express that
  through the default constructor; it must bypass `ToolFailure::new` and
  build the struct with an explicit `recovery` field. The framework has no
  per-tool override hook.
- Impact: low. The hardcoded mapping is conservative and safe (it errs
  toward verification for ambiguous cases), so tools that cannot override
  are merely slower (extra verification), not incorrect. But the framework
  is opinionated about recovery policy in a way that may not fit every
  idempotent tool.
- Root cause: design choice. The framework picked one conservative mapping
  rather than a per-tool policy table. This is defensible (a single safe
  default is simpler than a configuration surface) but undocumented and
  inflexible.
- Direction: either (a) document that the mapping is intentionally a single
  conservative default and that tools needing different recovery must
  construct `ToolFailure` explicitly (cheapest, preserves current safety);
  or (b) add a per-tool override hook (e.g. an optional
  `fn recovery_policy(&self) -> RecoveryPolicy` on the `Tool` trait with a
  default that returns the conservative mapping) so idempotent tools can
  opt into blind retry without bypassing the constructor. Prefer (a) unless
  a concrete tool demonstrates the verification gate is a real performance
  problem.
- Regression validation: doc-only under (a); under (b),
  `cargo test --workspace --all-features` and a test that a tool overriding
  the policy sees its recovery action on a reported failure.
- Validation reports: [V04](../validations/F-EXT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Tool contract inventory — typed, cancellable, complete result/failure vocabulary | yes | passed | [V01-01](../validations/F-EXT-01/V01-01.md) |
| V02 | Registration reachable, name-based, collision-free for unique names | yes | passed | [V02-01](../validations/F-EXT-01/V02-01.md) |
| V03 | Cursor pagination and artifact spill preserve identity and bounded output | yes | passed | [V03-01](../validations/F-EXT-01/V03-01.md) |
| V04 | Error classification exhaustive; PartialSideEffect -> VerifyThenRetry safety pattern | yes | passed | [V04-01](../validations/F-EXT-01/V04-01.md) |
| V05 | Historical-document drift check | not-applicable | n/a | No historical document is reused for a claim in this report. |

## Historical Claim Status

No historical documents are cited as evidence for any claim in this report.
All findings are based on code at commit `9b0e0fa` / `b3b2e81` and the four
validation reports above.

## Coverage And Uncertainty

- Code not inspected: the concrete builtin tool implementations (shell,
  file, code, git) and how they construct `ToolResult` / `ToolFailure` in
  practice — that is F-EXT-02's scope. The contract itself is fully
  enumerated.
- Validations not executed at runtime: all four validations are static
  inspections (no `cargo test` run). The contract surface and enum
  exhaustiveness are structural facts that do not require execution to
  confirm; the cancellation claim inherits from F-CORE-01's `CancellationToken`
  analysis.
- Environmental limits: none. Both repos are clean at the audited commits.
- Claims that remain uncertain:
  - Whether the application adapter or any builtin tool relies on reading a
    specific `ToolResult` field per `kind` (the F-EXT-01-P3-01 ambiguity).
    The convention is consistent in audited code but not enforced.
  - Whether any tool in practice needs to override the recovery mapping
    (the F-EXT-01-P3-02 inflexibility). No audited tool does today; the
    finding is preventive.

## Handoff

- Conclusions downstream tasks may rely on:
  - The `Tool` trait + `ToolResult` + `ToolFailure` taxonomy is the single,
    typed, versioned tool contract in `echo-core`. Downstream tasks
    (F-EXT-02 builtin tools, F-LLM-* provider tool-call mapping,
    F-RCT-04 tool runtime) should treat it as authoritative.
  - Cancellation is `CancellationToken`-based and propagates through
    `BoxFuture`; tool-runtime tasks can rely on the same primitive
    established by F-CORE-01.
  - Bounded output is via `ToolOutputArtifactWriter` + `ToolResult::truncated`
    + `ToolOutputArtifactRef`. Consumers holding the ref (not the bytes) is
    the contract — context-budget tasks can rely on this.
  - Error classification is exhaustive and safety-conservative
    (`PartialSideEffect`/`Timeout -> VerifyThenRetry`). The F-RCT-04
    tool-runtime retry loop should respect the `recovery` field rather than
    re-deriving policy.
- Reports they must read:
  - [V01-01](../validations/F-EXT-01/V01-01.md) for the full contract
    inventory.
  - [V04-01](../validations/F-EXT-01/V04-01.md) for the recovery mapping
    table.
- Conditions that make this report stale:
  - Any new field on `ToolResult` or new variant on `ToolResultKind` /
    `ToolFailureCategory` / `ToolRecoveryAction` invalidates V01 / V04 and
    the corresponding findings.
  - Any change to the default category-to-recovery mapping invalidates
    F-EXT-01-P3-02 and V04.
  - Introduction of a per-tool recovery override hook resolves
    F-EXT-01-P3-02.
- Follow-up task IDs (no fixes implemented in this review):
  - F-EXT-02 should verify builtin tools construct `ToolResult` with the
    correct authoritative field per `kind` (relates to F-EXT-01-P3-01) and
    that they report `ToolFailure` with accurate `side_effect` classification.
  - A future tool-registry task should confirm whether the registry enforces
    or merely assumes name uniqueness (flagged in V02).
