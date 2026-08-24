# F-SEC-01: Guards, sandbox, secrets, panic safety

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean (read-only static review)

## Question

Do generic local execution protections prevent framework bugs, data loss,
secret logging, and sandbox escape without product-specific overreach?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/security.rs` — secret scanner, 19 patterns, redaction
  helpers (`scan_secrets`, `redact_secrets`, `contains_secrets`,
  `scan_summary`).
- `echo-agent/src/sandbox.rs` — re-export façade for
  `echo_execution::sandbox`.
- `echo-agent/src/guard/{mod,llm,rule}.rs` — re-export façades for
  `echo_core::guard`.
- `echo-agent/echo-core/src/sandbox.rs` — `SandboxExecutor` trait,
  `SandboxCommand`, `ExecutionResult`, `ResourceLimits`,
  `IsolationLevel`, UTF-8-safe `retain_utf8_prefix`.
- `echo-agent/echo-core/src/guard/{mod,content,llm,rule}.rs` — `Guard`
  trait, `GuardManager`, `RuleGuard`, `LlmGuard`, `ContentGuard` (PII).
- `echo-agent/echo-execution/src/sandbox/{mod,manager,policy,local,docker,k8s}.rs`
  — three-layer sandbox + policy router + LocalSandbox backend.
- Live caller surfaces (read for reachability, not for review):
  `tools/builtin/spawn_task.rs`, `trace/mod.rs`,
  `agent/snapshot.rs`, `agent/react/run/execution.rs`,
  `agent/react/builder.rs`, `echo-agent/src/evolution/security.rs`.

## Out Of Scope

- Application-layer permission/HITL policy in `echo-agent-cli` (F-HITL-01).
- Tool registration and routing (F-EXT-01).
- MCP and channel transport security (F-EXT-01, F-LLM-01).
- Docker/K8s manifest correctness beyond what is needed for fallback
  reasoning (their `is_available` and isolation level reporting only).
- Persistence/identity of runs (X-STA-01, F-RCT-05).

## Inputs

- `AGENTS.md` "产品定位与安全边界" section — threat-model authority:
  EKO is local personal assistant; only data-loss prevention, framework-bug
  prevention, and local-universal safety (no secrets in logs) are in scope;
  web-service threats (XSS, SSRF, multi-tenant) are not.
- `AGENTS.md` "Rust 编码硬性约束" — UTF-8 safety (no byte slicing) and
  panic safety (no `unwrap`/`expect`/`panic!`/direct indexing).
- `AGENTS.md` historical lessons: `require_full_auto` gates that broke
  terminal/MCP were removed; new features must not reintroduce them.
- `docs/comprehensive-review/REPORTING.md`, both templates, and
  `zcode-glm/tasks/B-REF-01.md` (read for format and the permission-mode
  reference constraint C4 / P3-01).
- No dependency task report other than B-REF-01 (no F-HITL-01 report
  exists yet at the time of writing).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | Sandbox executor abstraction, sandbox policy router, secret pattern catalog, `Guard` trait and `GuardManager`, UTF-8-safe truncation, fail-closed LLM guard parsing, path-injection validation. Any `echo-agent` consumer building an agent that runs tool code may want these. Keep at framework layer. |
| EKO product policy (application) | The *choice* of which permission mode (Trusted/Strict/Maximum) to launch with, which guard chain to wire, and which secrets to additionally block — these are product calls and live in the CLI. The framework correctly does NOT bake in `full-auto`/`default` user-permission gates on `create_terminal`/`connect_mcp_server` (per AGENTS.md historical lesson; grep confirmed no such gates exist). |
| Adapter boundary | N/A — this task inspects only framework code. `echo-agent-cli` adapter wiring is F-HITL-01. |

Repository-wide duplicate search terms:

- `scan_secrets | redact_secrets | contains_secrets | scan_summary`
  → root `security.rs` is the main-runtime scanner; `evolution/security.rs`
  is a *parallel* scanner with its own `SECRET_PATTERNS` for the evolution/
  memory-persistence path. See finding F-SEC-01-P3-02.
- `SandboxExecutor | SandboxManager | LocalSandbox | DockerSandbox | K8sSandbox`
  → single authority in `echo_core::sandbox` (trait) +
  `echo_execution::sandbox` (impls). No duplicate.
- `Guard | GuardManager | RuleGuard | LlmGuard | ContentGuard`
  → single authority in `echo_core::guard`. Root `src/guard/*.rs` are
  one-line re-export façades (`pub use echo_core::guard::*`), not parallel
  implementations.
- `validate_sandbox_path | seatbelt | bwrap | bubblewrap`
  → single authority in `local.rs`. No duplicate.

## Current Path

Secret redaction flow (verified end-to-end):

```
tool output / spawn_task stdout+stderr
  └─► tools/builtin/spawn_task.rs:171,175  redact_secrets(…)          ──► trace
tool output (ReAct loop)
  └─► agent/react/run/execution.rs:229     contains_secrets → redact   ──► LLM input
tool output (snapshot path)
  └─► agent/snapshot.rs:885                contains_secrets → redact   ──► snapshot
arbitrary trace string
  └─► trace/mod.rs:434                     redact_secrets              ──► trace sink
```

Sandbox routing flow:

```
SandboxCommand
  └─► SandboxManager::execute / execute_with_limits
        ├─► SandboxPolicy::evaluate_with_limits   (policy.rs:152-199)
        │     returns required IsolationLevel
        ├─► select_executor(required)             (manager.rs:344-403)
        │     prefer lightest executor ≥ required;
        │     if none and allow_fallback → best available + warn
        │     if none and !allow_fallback → Err(Unavailable|PermissionDenied)
        └─► executor.execute_with_limits_and_cancel
              └─► LocalSandbox | DockerSandbox | K8sSandbox
```

Guard pipeline flow:

```
content
  └─► GuardManager::check_all                 (echo-core/guard/mod.rs:126-187)
        ├─► spawns all guards under Semaphore(16) + CancellationToken
        ├─► first Block cancels in-flight checks, returns Block{reason}
        ├─► collects Warn{reasons} into Vec<String>
        └─► Guard task JoinError → ReactError::Other  (no panic propagation)
```

Terminal recovery / cancellation:

```
cancel.cancelled()
  └─► execute_with_limits_and_cancel           (echo-core/sandbox.rs:73-90)
        tokio::select! { cancel → Err(SandboxError::Cancelled); inner → result }
```

## Findings

### F-SEC-01-P2-01: ContentGuard Redact mode is a no-op through GuardManager

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/content.rs:222-232`;
  wiring at `echo-agent/src/agent/react/builder.rs:760-764`.
- Reachability: `ReactAgentBuilder::with_content_guard(ContentGuardMode::Redact)`
  (builder.rs:760) pushes a `ContentGuard` into the guard chain →
  `GuardManager::check_all` invokes `Guard::check` → `ContentGuard::check`
  produces `ContentGuardResult::Redacted(String)` → the `Guard` impl at
  content.rs:222-232 converts it to `GuardResult::Warn { reasons: vec!["内容
  包含敏感信息，已脱敏处理"] }` and **drops the redacted String on the
  floor**. The inline comment at content.rs:223-227 explicitly admits this.
- Expected invariant: a guard advertising a `Redact` mode, when wired into
  the runtime guard chain, should produce redacted content visible
  downstream (or the API should not advertise Redact as a chainable mode).
- Observed behavior: callers wiring `with_content_guard(Redact)` see only a
  warning; PII still reaches the LLM / trace / snapshot. The user must
  discover and call `ContentGuard::redact()` separately, which the
  `GuardManager` pipeline does not do.
- Impact: a developer who enables `ContentGuardMode::Redact` reasonably
  believes PII is being stripped from content visible to the model. It is
  not. This is misleading API surface; not a data-loss event by itself, but
  erodes the guarantee AGENTS.md draws for "no secrets in logs" (PII is a
  superset concern).
- Root cause: `GuardResult` is non-mutating (`Pass`/`Block`/`Warn`); there
  is no variant for "transformed content". `ContentGuard` was retrofitted
  onto a trait that cannot carry redacted text, so the impl silently
  discards it.
- Direction: pick one of:
  (a) extend `GuardResult` with `Replace { content: String, reason: String }`
      and have `GuardManager::check_all` return the transformed string;
  (b) keep `GuardResult` as-is but remove `ContentGuardMode::Redact` from
      the `Guard` impl and document that redaction must be called directly
      via `ContentGuard::redact()` outside the guard chain — then remove the
      misleading `with_content_guard(Redact)` builder helper or rename it;
  (c) wire `ContentGuard::redact()` into the tool-output path explicitly
      (alongside `security::redact_secrets`) so the runtime gets redaction
      without depending on the GuardManager shape.
  Recommended: (a) — the smallest change that makes the existing public API
  honest. The old `Redact→Warn` mapping and its inline comment should be
  deleted.
- Regression validation: a unit test that constructs a `GuardManager` with
  a `ContentGuard::new(ContentGuardMode::Redact)`, calls `check_all` on
  input containing `13812345678`, and asserts the returned content has
  `[REDACTED:PHONE]` replacing the phone digits (today this fails).
- Validation reports: [V04-01](../validations/F-SEC-01/V04-01.md).

### F-SEC-01-P2-02: Password-in-URL secret pattern produces documented false positives

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/security.rs:9-12` (TODO comment) and
  `security.rs:58` pattern `r"://[^:]+:[^@]+@"`.
- Reachability: `SECRET_PATTERNS` is consumed by `scan_secrets`/
  `redact_secrets`/`contains_secrets` (security.rs:82-136), all of which
  have live callers in the main runtime
  (`execution.rs:229`, `snapshot.rs:885`, `trace/mod.rs:434`,
  `spawn_task.rs:171,175`).
- Expected invariant: a secret scanner should not regularly flag benign
  content such as documentation URLs, example connection strings in code
  comments, or tutorial fragments.
- Observed behavior: the pattern `://[^:]+:[^@]+@` matches any
  `scheme://anything:anything@` text, including `https://user:pass@` in
  markdown documentation, READMEs, or agent-retrieved web pages that contain
  example URLs. Such content gets `[REDACTED: Password in URL]` injected in
  place of benign text — corrupting documentation output that the agent
  returns to the user or writes to files.
- Impact: false-positive redaction of benign content in tool output,
  traces, and snapshots. Not a panic or data-loss event, but a
  usability/correctness defect that the codebase has already acknowledged
  in a TODO. The AGENTS.md "framework bug prevention" scope covers this:
  the framework should not silently corrupt legitimate content.
- Root cause: pattern is too broad; it lacks context discrimination
  (comment-line exclusion, configuration-context requirement).
- Direction: implement the documented fix in `security.rs:9-12` — exclude
  lines starting with `//` or `#`, and require the match to appear in a
  configuration/URI-assignment context. Alternative: gate this single
  pattern behind a "heuristic" tier (warn-only) and keep only
  high-confidence patterns (AWS `AKIA…`, GitHub `ghp_…`/`github_pat_…`,
  Anthropic `sk-ant-…`) in the blocking tier, as the same TODO suggests.
- Regression validation: extend `redact_secrets` tests with a fixture
  containing a markdown URL example (`see https://user:pass@example.com in
  docs`) and assert it is not redacted, plus a fixture with a real config
  assignment (`DATABASE=postgres://u:p@host/db`) and assert it is.
- Validation reports: [V02-01](../validations/F-SEC-01/V02-01.md).

### F-SEC-01-P3-01: RuleGuard max_length uses byte length, not character count

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/rule.rs:51-55`.
  ```rust
  if let Some(max_len) = self.max_length
      && content.len() > max_len
  {
      return Ok(GuardResult::Block {
          reason: format!("Content length {} exceeds limit {}", content.len(), max_len),
      });
  }
  ```
- Reachability: `RuleGuard::check` is invoked by `GuardManager::check_all`
  (echo-core/guard/mod.rs:147) for every registered `RuleGuard`. The
  framework doc example at `echo-core/src/guard/rule.rs:21-25` advertises
  `.max_length(10000)`.
- Expected invariant: AGENTS.md "Rust 编码硬性约束 §1" mandates
  `chars().count()` for length checks on potentially unicode strings, not
  byte `len()`.
- Observed behavior: `content.len()` returns byte length. For CJK text
  (3 bytes/char) this means `max_length(10000)` actually blocks at ~3333
  Chinese characters, and the block reason reports a misleading "length"
  (bytes). The example in the doc comment `.max_length(10000)` would block
  a 4000-character Chinese document as "exceeds limit 10000" when its byte
  length is ~12000.
- Impact: false-positive blocking of legitimate non-English content; the
  reported "length" in the block reason does not match any user-meaningful
  count. Low severity because the guard is opt-in and the limit is
  caller-chosen, but it is a direct AGENTS.md violation.
- Root cause: byte length used instead of character count.
- Direction: replace `content.len()` with `content.chars().count()` in both
  the comparison and the block-reason format string. Add a regression test
  with a multibyte-only input that exercises the boundary.
- Regression validation: unit test `max_length_uses_char_count` that
  builds `RuleGuardBuilder::new("t").max_length(3).build()` and asserts
  `"中文"` (2 chars, 6 bytes) passes while `"中文文"` (3 chars, 9 bytes)
  also passes at exactly the limit and `"中文文文"` blocks.
- Validation reports: [V04-01](../validations/F-SEC-01/V04-01.md).

### F-SEC-01-P3-02: Parallel secret scanner implementations in framework

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/src/security.rs:33-77` — 19 secret patterns
    (`SECRET_PATTERNS: LazyLock<Vec<(&str, Regex)>>`).
  - `echo-agent/src/evolution/security.rs:22-50` — separate 19-pattern list
    (`SECRET_PATTERNS: LazyLock<Vec<SecretPattern>>` with struct
    `{ name, regex, _example }`).
- Reachability: root `security.rs` is consumed by the main ReAct/trace/
  snapshot/spawn_task surfaces (four live callers, see V02-01).
  `evolution/security.rs::SecretScanner` is consumed only by the evolution
  system (memory/skill persistence), confirmed by grep on
  `evolution::security::SecretScanner`.
- Expected invariant: AGENTS.md "动手前先查是不是已经有了" and
  "严禁平行实现同一语义" — a single semantic concept (secret detection)
  should have one authoritative implementation. The framework's secret
  catalog should not silently diverge across two scanners.
- Observed behavior: the two lists are *almost* the same but diverge in
  coverage. Examples of divergence:
  - Evolution has GitHub OAuth Token (`gho_…`); root does not.
  - Evolution's Anthropic pattern requires the `sk-ant-apiNN-…`/`adminNN-…`
    suffix and 80+ chars; root accepts any `sk-ant-` prefix with 20+ chars.
  - Evolution's HuggingFace pattern accepts 20+ chars; root requires
    exactly 34.
  - Evolution has a separate "API Key Env Var" pattern
    (`access_token|api_key|…`); root has a similar but differently-named
    "Generic API Key" pattern.
- Impact: a secret detected in one path may be missed in the other. Adding
  a new secret type requires updating both lists. This is a maintenance
  hazard and a latent coverage gap, not a runtime defect today.
- Root cause: independent implementation growth; no consolidation pass.
- Direction: extract a single `SecretPattern` catalog (the evolution
  scanner's struct shape is the better one because it carries an example
  for documentation/tests) and have both `security.rs` and
  `evolution/security.rs` consume it. The evolution scanner can keep its
  strictness flag (it scans persisted memory/skills, where higher
  confidence is wanted) but the pattern *source* should be shared.
- Regression validation: a cross-cutting test that asserts both scanners
  produce the same match set on a fixture containing one example of every
  secret type, after consolidation.
- Validation reports: [V02-01](../validations/F-SEC-01/V02-01.md).

### F-SEC-01-P3-03: LocalConfig.enable_os_sandbox naming is misleading on Windows

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-execution/src/sandbox/local.rs:50-66` —
    `Default::default()` sets `enable_os_sandbox: true` whenever
    `cfg!(any(macos, linux, windows))`.
  - `local.rs:948-956` — `isolation_level()` returns `OsSandbox` only on
    macOS/Linux; Windows always returns `Process`.
  - `local.rs:587-593` — `configure_command_process` is a no-op on
    non-Unix; no rlimit, no job object, no confinement.
- Reachability: any consumer that constructs `LocalSandbox::default()` on
  Windows.
- Expected invariant: a config field named `enable_os_sandbox` should
  reflect whether OS-level isolation is actually applied.
- Observed behavior: on Windows the field is `true` by default but the
  Windows backend is just `cmd /C` with timeout/output limits — no OS
  sandbox. The reported `isolation_level()` correctly says `Process`, so
  there is no *runtime* misreport, but the config field name and default
  mislead anyone reading the configuration or logs.
- Impact: documentation/naming inconsistency; users may believe Windows
  has seatbelt/bubblewrap-equivalent isolation. No security regression
  because `isolation_level()` is honest.
- Root cause: the field was added when only macOS/Linux had real OS
  sandbox support; the Windows default was set to `true` to keep the
  `effective_os_sandbox_enabled()` predicate non-empty, but the backend
  never gained matching enforcement.
- Direction: either rename the field to `enable_platform_isolation` (and
  document that Windows uses process-level only), or split into
  `enable_seatbelt` (macOS) / `enable_bubblewrap` (Linux) /
  `enable_windows_process_hardening` (Windows, currently a no-op). Update
  `Default` truthfully. The doc comment at local.rs:33-35 already
  partially explains this; tighten it.
- Regression validation: a test that asserts `isolation_level()` and the
  config field agree on every platform.
- Validation reports: [V03-01](../validations/F-SEC-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Threat boundary classification vs AGENTS.md | yes | passed | [V01-01](../validations/F-SEC-01/V01-01.md) |
| V02 | Secret/log redaction coverage and correctness | yes | passed (with documented gaps) | [V02-01](../validations/F-SEC-01/V02-01.md) |
| V03 | Sandbox unavailability fallback behavior | yes | passed | [V03-01](../validations/F-SEC-01/V03-01.md) |
| V04 | Path traversal / UTF-8 / panic safety in guard/sandbox | yes | passed (two findings surfaced) | [V04-01](../validations/F-SEC-01/V04-01.md) |
| V05 | Historical-document drift check | conditional | not_applicable | No prior F-SEC-01 audit document exists in the repo to drift-check against. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: `require_full_auto` gates on terminal/MCP were removed and must not be reintroduced | current (verified absent) | grep across primary paths found zero `require_full_auto`/`full_auto` gates on guard/sandbox/security code. V01-01. |
| AGENTS.md: UTF-8 safety — use `chars().take()`, no byte slicing | current (with one violation) | Mostly honored (`retain_utf8_prefix`, `IncrementalUtf8Decoder`, `split_stream_chunks` all iterate chars). Violated at `rule.rs:51-55` (`content.len()`). F-SEC-01-P3-01; V04-01. |
| AGENTS.md: panic safety — no `unwrap`/`expect`/`panic!` on untrusted input | current (verified) | Zero matches in non-test guard/sandbox code. V04-01. |
| B-REF-01-P3-01: permission is launch-time mode + isolation, not an approval state machine | current (supported) | `SandboxPolicy` is per-command risk analysis, not a user-permission state machine. No user-vs-agent carve-out in framework code. V01-01. |
| AGENTS.md: only add guards for data-loss / framework-bug / local-universal safety | current (verified) | All framework protections in scope map to one of these three categories. V01-01. |

## Coverage And Uncertainty

- **Docker/K8s backends inspected shallowly**: only their `is_available`,
  `isolation_level`, and manager routing were traced. The actual
  `docker.rs`/`k8s.rs` manifest construction (image selection, mount
  validation, network policy) was not audited for path traversal or
  image-pull safety. `SENSITIVE_MOUNT_PATHS` (sandbox/mod.rs:68-72) is
  defined but I did not verify each backend consults it before binding a
  user-supplied `writable_path`. Recommend a follow-up task if container
  backends are wired into EKO's default path.
- **`echo-agent-cli` not inspected**: the task is framework-only. Any
  application-layer permission gate would be F-HITL-01's responsibility.
- **Evolution scanner coverage**: I read the head of
  `evolution/security.rs` and its `contains_secrets` test but did not
  fully trace its `redact` path. The parallel-implementation finding
  (F-SEC-01-P3-02) is based on pattern-list comparison, not on a runtime
  defect.
- **ContentGuard char_boundary**: V04-01 notes that `content.rs:114`
  lacks the defensive `is_char_boundary` check that `security.rs:113-125`
  has. I did not promote this to a separate finding because the `regex`
  crate guarantees UTF-8-aligned matches in default unicode mode, so the
  practical risk is nil; it is a defense-in-depth inconsistency only.
- **No executable validation**: the 8-minute target for this task favored
  static inspection; no `cargo test` or `cargo clippy` was run. The
  regression-validation sections of each finding describe the tests that
  should accompany a fix.

## Handoff

- **Conclusions downstream tasks may rely on**:
  - The framework satisfies AGENTS.md's local-desktop threat model. No
    web-service overreach gates exist in guard/sandbox/security. V01-01.
  - Secret redaction is wired into all four main runtime surfaces
    (spawn_task, trace, snapshot, ReAct execution) with correct UTF-8
    safety and overlap deduplication. V02-01.
  - Sandbox fallback is fail-safe: typed error or logged warning, no
    silent downgrade, no panic. V03-01.
  - No `unwrap`/`expect`/`panic!` in non-test guard/sandbox code. V04-01.

- **Reports downstream tasks must read**:
  - F-HITL-01 (application permission/HITL) should read V01-01 to confirm
    the framework layer does not pre-empt its policy choices, and
    B-REF-01-P3-01 (C4) for the launch-mode-vs-state-machine constraint.
  - F-EXT-01 (tool registration) should read V02-01 to confirm
    `spawn_task.rs` already redacts tool output before returning.

- **Conditions that make this report stale**:
  - If `echo-core/src/guard/content.rs` gains a `Replace`/redact-carrying
    `GuardResult` variant, finding F-SEC-01-P2-01 should be re-evaluated.
  - If `security.rs` and `evolution/security.rs` are consolidated, finding
    F-SEC-01-P3-02 should be marked fixed.
  - If a new `require_full_auto`-style gate is added to guard/sandbox, the
    V01-01 boundary classification must be re-run.
  - If Docker/K8s backends become EKO's default execution path, the
    container-backend coverage gap noted above must be closed.

- **Follow-up task IDs (no fixes implemented in this review task)**:
  - F-HITL-01 should consume V01-01 for the framework/application
    threat-model split.
  - A future F-SEC-02 (not opened by this task) could audit the Docker/K8s
    manifest construction path that this task deferred.
