# F-SEC-01: Guards, sandbox, secrets, and panic safety

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Do generic local execution protections prevent framework bugs, data loss, secret logging, and sandbox escape without product-specific overreach?

## Scope

- `echo-core/src/guard/` (mod/content/llm/rule), `echo-core/src/sandbox.rs`, `echo-core/src/plugin/variables.rs` (export_to_env).
- `echo-execution/src/sandbox/` (mod/policy/manager/local/docker/k8s), `echo-execution/src/risk.rs`.
- Root `src/security.rs` (secret scanner/redaction) and its call sites (`src/agent/react/run/pipeline.rs`, `src/agent/snapshot.rs`, `src/agent/react/run/execution.rs`, `src/trace/mod.rs`, `src/tools/builtin/spawn_task.rs`).
- `echo-tools/src/security.rs` (PathValidator, SSRF/pinned client), `echo-tools/src/shell.rs` (command classification), `echo-tools/src/code.rs` (run_code minimum isolation), `echo-tools/src/web/fetch.rs` + web/media fallback clients.
- `echo-agent/src/eval/runner.rs`, `src/a2a/serve.rs` (parse_method), `echo-agent-cli/src/tauri/error.rs` (IpcAuth residue), `echo-agent-cli/src/tauri/terminal.rs`, `echo-agent-cli/echo-agent-app-core/src/infra.rs`.

## Out Of Scope

- Echo-agent-cli application-side sinks (chat display, file persistence redaction) — A-* tasks.
- Full panic inventory (`unwrap` distribution across all crates) — Q-STA-01.
- Dependency advisories — Q-DEP-01.
- echo-tools files-tool path resolution in depth (files/mod.rs resolver) — F-EXT-01/A-* tools tasks; only the validator-level gap is noted here.

## Inputs

- Root `AGENTS.md` (local threat model), shared `REPORTING.md`, `TASKS.md` (F-SEC-01 card), `zcode-ds/README.md`.
- Dependency reports: zcode-ds `F-CORE-01` (event/error envelope), `B-REF-01` (convergence matrix: sandbox/approval separation, subagents excluded from interactive approval), `B-DOC-01` (AUDIT index — 6 current items re-rated here).

## Layering Decision

- Generic mechanism: guard trait/manager (echo_core, feature `guard`, opt-in), sandbox executor/policy/manager (echo_core + echo_execution), secret scanner + redaction (root src/security.rs), SSRF/pinned client + PathValidator (echo_tools), shell command classification (echo_tools ShellTool) — all correctly placed in the framework; EKO consumes them via `infra.rs`.
- EKO product policy: `SandboxManager::local_sandbox()` selection (OS-sandbox cap, fallback disabled) and the terminal-vs-agent distinction are application decisions; the SSRF private-range policy has no EKO opt-out (see P3-03).
- Adapter boundary: `infra.rs:265-300` (sandbox injection into agents) is thin; no scheduling/state authority duplicated.
- Duplicate search terms: `GuardManager`, `ContentGuard`, `RuleGuard`, `LlmGuard`, `SandboxExecutor`, `SandboxManager`, `IsolationLevel`, `with_minimum_isolation`, `redact_secrets`, `contains_secrets`, `validate_url`, `ssrf_safe_get`, `PathValidator`, `validate_within_base`, `check_tool_output_guard`, `ToolRiskClassifier`, `IpcAuth`, `require_full_auto`. Results: two `check_tool_output_guard` variants (one dead, P3-07), two guard façades (root `src/guard` re-exports echo_core — intentional), `ToolRiskClassifier` (echo-execution/src/risk.rs) has zero production callers (dead framework API), `validate_within_base` has zero callers while `validate_output_file` duplicates its intent lexically (P3-06), `IpcAuth::require_full_auto` dead in CLI (P3-04).

## Current Path

- Guard pipeline (framework, opt-in): `GuardManager::check_all` (parallel, cancel-on-block, concurrency-capped) → `GuardDirection`-filtered `Guard` impls; live wiring: input guard at `react_loop.rs:518-521` + `stream_channel.rs:141-143`; output guard at `pipeline.rs:707` (→ `snapshot.rs:878` check_tool_output_guard, includes secret redaction). EKO wires no guards (grep: no GuardManager usage in echo-agent-cli).
- Sandbox (framework): `SandboxCommand.minimum_isolation` → `SandboxPolicy::evaluate_with_limits` (floor via max, cap via `max_isolation_level`) → `SandboxManager::execute_at_level` → `select_executor` (lightest-first Local→Docker→K8s, best-available fallback when `allow_fallback`) → per-backend execution with timeout/cancel/cleanup. EKO: `local_sandbox()` (`infra.rs:265`, fallback disabled) → `run_code` fails closed unless OS sandbox (`code.rs:319-345`, `infra.rs:269`).
- Secrets: `scan_secrets` (19 patterns) → `redact_secrets` (UTF-8-boundary-safe replace) at 5 live sinks (output guard stage, ToolCall trace events, spawn_task output, snapshot truncation) + 1 dead sink (execution.rs).
- URL safety: `web_fetch` uses `ssrf_safe_get` (resolve-once → pin-IP → per-hop re-validation) `fetch.rs:201`; PathValidator `validate_file`/`validate_output_file` gate data tools (pdf/excel/image/word/statistics).
- Shell classification: `ShellTool::check_command_safety` — metacharacter rejection (no sandbox), DANGEROUS blocklist always, REQUIRE_APPROVAL list, strict whitelist; sandbox mode relaxes metacharacters + SANDBOX_SAFE_COMMANDS.

## Findings

### F-SEC-01-P3-01: Eval runner `TestPass` criteria still executes unvalidated `sh -c`; SweBench metacharacter check misses newline (re-rates B-DOC-01 P2-01)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/eval/runner.rs:226-237` (`SuccessCriteria::TestPass` → `run_command(command, cwd)` with no validation), `:695-704` (`run_command` → `sh -c`), `:680-689` (`validate_shell_command` list = `; | & $ \` > <` — no `\n`), `:343-346` (SweBench variant validates `test_command`; B-DOC-01's anchor "`:340` validates only repo_url" is stale at this same commit)
- Reachability: `EvalCase` is a pub serde type (`src/eval/mod.rs:66-92`, `SuccessCriteria` tagged at :96) — case files are user/local or shared datasets (SWE-bench style); `TestPass` reaches `run_command` via the criteria evaluation path (`runner.rs:226`).
- Expected invariant: arbitrary command strings from case data must not reach a shell without the same checks as the shell tool.
- Observed behavior: `TestPass { command }` reaches `sh -c` unvalidated; `SweBench` rejects `; | & $ \` > <` but a newline-separated second command (`"true\nrm -rf x"`) passes and executes.
- Impact: with local case files this is user-to-self (dev tool, local workspace); the framework's reusability (shared/public eval datasets) keeps it real but low — arbitrary command execution with the user's privileges under the local threat model.
- Root cause: the audit-fix item 4 was only applied to the SweBench arm, and the metacharacter list was written without newline.
- Direction: route `test_command` through `validate_command_safety` (echo-tools shell.rs:139) or `shlex` + argv; add `\n` (and `\r`) to the rejection list or reject non-single-token commands; regression test with `test_command = "true\ntouch <marker>"`.
- Regression validation: eval fixture with a newline payload must fail closed; benign `TestPass` still runs.
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-02: `SandboxManager` fallback can execute below `command.minimum_isolation` when `allow_fallback=true`

- Priority: P3
- Confidence: high (mechanism), medium (impact)
- Layer: framework
- Evidence: `echo-execution/src/sandbox/manager.rs:291-319` (`execute_at_level`: `actual < required` → `warn!` + proceed when `allow_fallback`), `:344-403` (`select_executor` returns best-available below required), `policy.rs:195-198` (minimum folded into `required` via max — not independently enforced)
- Reachability: `SandboxManager::auto_detect()` sets `allow_fallback=true` (`manager.rs:194`); on a Process-only local backend (Windows default, or mac/Linux with `enable_os_sandbox=false`) with Docker/K8s absent, a command with `minimum_isolation=OsSandbox` executes at Process with only a warn. EKO is unaffected: `local_sandbox()` has `allow_fallback=false` (`manager.rs:229-242`) and `run_code` pre-checks declared level (`code.rs:324`) + availability (`infra.rs:269`); the only production floor-setter is `code.rs:313` (OsSandbox).
- Expected invariant: `minimum_isolation` is a hard floor independent of degradation policy.
- Observed behavior: the floor is only an input to the policy evaluation; degradation can undercut it silently (warn-level log; `ExecutionResult.sandbox_type` exposes it).
- Impact: generic framework callers that skip run_code-style pre-checks can run code below their declared minimum on degraded hosts; no live production path.
- Root cause: `allow_fallback` and `minimum_isolation` are not reconciled — the fallback branch never re-checks the floor against the chosen executor.
- Direction: in `execute_at_level`/`execute_stream`, fail closed when `actual < command.minimum_isolation` even with fallback enabled (distinguish the caller floor from the policy `required`); add a manager test with a Process-only executor + OsSandbox minimum + fallback=true expecting an error.
- Regression validation: `SandboxManager::local_only()` with `allow_fallback=true` executing a `with_minimum_isolation(OsSandbox)` command must error (currently passes — see `manager.rs:472-479` for the weak test that only checks name).
- Validation reports: [V03](../validations/F-SEC-01/V03-01.md)

### F-SEC-01-P3-03: SSRF validation blocks all loopback/private addresses with no opt-out — `web_fetch` cannot fetch localhost/LAN in a local-assistant context

- Priority: P3
- Confidence: high (behavior), medium (defect judgment)
- Layer: framework
- Evidence: `echo-tools/src/security.rs:529-570` (`validate_url_with_addrs` rejects any private/link-local resolved address), `:615-663` (`is_private_ip` covers loopback/RFC1918/link-local/CGNAT/TEST-NET), `:716-721` (`ssrf_safe_get`); `echo-tools/src/web/fetch.rs:199-201` (unconditional `ssrf_safe_get` in the fetch tool; no allow-list/opt-out anywhere in the tool surface)
- Reachability: `web_fetch("http://localhost:3000")` or `web_fetch("http://192.168.1.10/")` → `AccessDenied` ("SSRF protection: rejecting access to private IP address …"). Default configuration, no escape hatch.
- Expected invariant: guard mechanisms must not break legitimate user-requested functionality in the local model ("默认不加权限门控"; user-interactive/developer capabilities stay usable).
- Observed behavior: private-range rejection is unconditional at the framework tool level.
- Impact: fetching a local dev server or LAN device — a plausible EKO user request — is impossible via web_fetch; also `web_fetch` of a public URL that redirects to a private one fails (by design of redirect re-validation).
- Root cause: SSRF is a server-side threat-model control (B-REF-01 lists it for multi-tenant products); applied unconditionally to a single-user local assistant's fetch tool without a product-configurable policy.
- Direction: keep the framework default safe but expose an explicit opt-in (e.g., `ResourceLimits`-style or tool parameter `allow_private: bool`, or an EKO-side policy hook) so the local product can permit loopback/LAN fetches; document the trade-off (prompt-injected agent fetching internal services vs. user-requested local fetches).
- Regression validation: a `web_fetch` localhost request succeeds with the opt-in and still fails without it; private-IP golden tests (V05-01) remain green.
- Validation reports: [V01](../validations/F-SEC-01/V01-01.md), [V05-01](../validations/F-SEC-01/V05-01.md)

### F-SEC-01-P3-04: Dead `IpcAuth::require_full_auto` gate + stale module doc in echo-agent-cli (AGENTS.md historical-lesson residue)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/error.rs:5-12` (module doc: "Commands that spawn processes, write files outside the workspace, or execute arbitrary code are gated behind `IpcAuth::require_full_auto()`"), `:45-56` (`require_full_auto`), `:31-43` (`IpcPermission::FullAuto/NotStrict`); zero callers of `IpcAuth::` / `IpcPermission::` across `src/` and `echo-agent-app-core/src/` (grep)
- Reachability: helper compiles but nothing invokes it; `create_terminal` (the exact feature the removed gates once blocked) runs ungated with an explicit local-model comment (`src/tauri/terminal.rs:281-295`).
- Expected invariant: no residue of the removed over-gating; docs reflect current behavior.
- Observed behavior: the gate API and its doc survive, claiming protections that no longer exist; a future contributor could re-apply the gate based on the doc.
- Impact: misleading API inventory + doc; risk of regression to the historical "terminal unusable under default permission" bug.
- Root cause: the removal commit deleted call sites but not the helper or its module doc.
- Direction: delete `IpcAuth`, `IpcPermission` and the doc paragraph (keep `IpcError` used by command modules); re-run `cargo check` for the gui feature.
- Regression validation: `cargo check --no-default-features --features gui` after deletion; grep `require_full_auto` → zero hits.
- Validation reports: [V01](../validations/F-SEC-01/V01-01.md)

### F-SEC-01-P3-05: Fallback `Client::new()` remains at 5 web-provider sites (AUDIT 3.2 re-rate); fetch.rs fallback is a dead field

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/web/fetch.rs:43-46` (builder-failure fallback stored in `#[allow(dead_code)] client` at `:56-60`, never used for requests — the fetch path uses `ssrf_safe_get` at `:201`), `echo-tools/src/media/image_fetch.rs:125` (default `Client::new()`; fetch paths use `ssrf_safe_get`/`ssrf_safe_request` at `:74,:198,:227`), `echo-tools/src/web/providers/tavily.rs:37-39`, `web/providers/duckduckgo.rs:35-37`, `web/providers/brave.rs:31-33` (fallback when builder fails; used for fixed provider endpoints)
- Reachability: builder failure requires invalid TLS/connection config (rare); provider fallbacks drop only the request timeout (reqwest default has none) — a hung provider request would hang the tool call.
- Expected invariant (AUDIT 3.2): no request path falls back to an unconfigured client.
- Observed behavior: unchanged since the audit; B-DOC-01 classifies it current.
- Impact: under the local model: a rare hung provider request (timeout lost); the SSRF concern is not reachable because the fallback client never performs URL-driven requests in the current code.
- Root cause: `unwrap_or_else(Client::new)` convenience retained at each site; fetch.rs kept a vestigial client field after switching to `ssrf_safe_get`.
- Direction: replace fallbacks with `ToolError` (fail the tool) or a builder default that always sets a timeout; delete the dead `client` field in `WebFetchTool` (fetch.rs:54-60) and the `#[allow(dead_code)]`.
- Regression validation: unit test asserting builder-failure surfaces a tool error rather than a client; `cargo clippy -D warnings` after deleting the dead field.
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-06: `validate_output_file` is lexical-only — symlinked subdirectory escapes the declared root for not-yet-existing outputs; canonical `validate_within_base` has zero callers

- Priority: P3
- Confidence: high (mechanism), medium (impact)
- Layer: framework
- Evidence: `echo-tools/src/security.rs:352-398` (`validate_output_file` uses `normalize_absolute_path` — lexical `..`/`.` handling, no symlink resolution, `:829-850`), `:272-343` (`validate_within_base` canonicalizes the nearest existing ancestor, re-appends the suffix, re-checks denies — the documented "canonical validator" at `:269-271`), zero callers of `validate_within_base` (grep)
- Reachability: `validate_output_file` gates data-tool outputs (`excel.rs:351,925,1356`); a path like `<root>/link/out.xlsx` where `<root>/link -> /etc` passes the lexical check and the write lands outside the declared root at runtime.
- Expected invariant: output paths are contained within the configured scope, symlink-safe like `validate_file`/`validate_within_base`.
- Observed behavior: containment holds only lexically for non-existent outputs; existing-file paths are safe because `validate_file` canonicalizes.
- Impact: local model: unintended write outside the workspace root via a symlinked subdirectory — visible, single-user, no data-loss amplification; a genuine framework-bug-class gap in the declared policy.
- Root cause: two validators with different strengths; the stronger one is unused (the codebase documents convergence to `validate_within_base` but never did it).
- Direction: make `validate_output_file` resolve the nearest existing ancestor canonical path and re-append the suffix (port of `validate_within_base`), then delete/redirect `normalize_absolute_path` usage; add a symlink-escape regression test.
- Regression validation: fixture with `<root>/link -> /etc` and output `<root>/link/x.txt` must be rejected; existing `test_validate_output_file_normalizes_parent_segments` stays green.
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-07: Dead duplicate output-guard with secret redaction at `execution.rs` (AUDIT 6.2 re-rate)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/execution.rs:221-260` — `check_tool_output_guard` marked `#[allow(dead_code)]` at `:225` (includes `contains_secrets`/`redact_secrets` at `:229-232`); the live variant is `src/agent/snapshot.rs:878-920`, invoked from `pipeline.rs:707` (OutputGuardStage); `execution.rs:402-416` uses only `truncate_tool_output` (no guard/redaction)
- Reachability: dead — the run loop never calls the execution.rs variant.
- Expected invariant: one authoritative output-guard/redaction path.
- Observed behavior: two near-identical implementations; the dead one silently forks the redaction logic (drift risk — e.g., the snapshot variant's redaction ordering is the one actually enforced).
- Impact: maintainability; a future fix applied only to the dead copy would leave secrets unredacted on the live path.
- Root cause: pipeline refactor moved the stage to snapshot while the old method remained.
- Direction: delete `execution.rs:221-260` and its `#[allow(dead_code)]`; keep snapshot.rs as the single authority.
- Regression validation: `cargo clippy --workspace --all-targets --all-features -- -D warnings` and the guard-pipeline tests.
- Validation reports: [V02](../validations/F-SEC-01/V02-01.md), [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-08: Plugin `unsafe { set_var }` stays current but bounded (B-DOC-01 P3-01 re-rate)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/plugin/variables.rs:177-212` — doc contract "caller must ensure single-threaded plugin initialization", key charset `[A-Z0-9_]` validated at `:198-207`, `unsafe` block at `:190`
- Reachability: `export_to_env` called at plugin load; framework API not documented as startup-only beyond the fn doc.
- Expected invariant: no data race on `libc environ`.
- Observed behavior: unchanged since audit §1.9; contract is documented but unenforced (no assertion, no mutex).
- Impact: theoretical UB on Rust 1.84+ if a second thread reads env concurrently during export; no live trigger in current call patterns.
- Root cause: env export predates the 1.84 unsafety; no alternative mechanism designed.
- Direction: add a `#[track_caller]` startup-only assertion or a global mutex; if mutex, keep the fn signature and document the serialization; B-DOC-01's suggested abstraction can be deferred.
- Regression validation: unit test spawning an env-reader thread during `export_to_env` under the mutex (or asserting the startup-only panic fires in a spawned-thread test).
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-09: `parse_method` first-occurrence heuristic can misroute A2A requests (AUDIT 2.8 re-rate)

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/src/a2a/serve.rs:296-321` (`parse_method` finds the first `"method":` occurrence), `:245-254` (only use: route `tasks/sendSubscribe` to SSE vs sync), proptest-hardened tests at `:323+`
- Reachability: any JSON-RPC body whose `params` contain an earlier `"method"` string (e.g. `{"params":{"method":"tasks/get"},"method":"tasks/sendSubscribe"}`) misroutes to the sync handler; conversely a body with `"method"` in params but no real method routes to SSE and hangs as an SSE stream until the server handler errors.
- Expected invariant: routing decisions derive from the parsed JSON-RPC method field.
- Observed behavior: string-heuristic on raw text.
- Impact: functional misrouting only (no privilege/security step — the JSON-RPC handler itself validates the method); a malformed/malicious body can turn a request into a hanging SSE connection locally.
- Root cause: hand-rolled parsing predates serde_json usage in the same handler.
- Direction: parse the body once with `serde_json` at `handle_json_rpc` and pass the method through; delete `parse_method` and its tests after the switch.
- Regression validation: the existing golden tests plus a params-first-`"method"` fixture asserting correct routing.
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md)

### F-SEC-01-P3-10: Security-scanner TODOs remain — false-positive redaction is the only local impact (AUDIT 6.1 re-rate)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/security.rs:3-12` (TODO v0.3: split high-confidence vs heuristic patterns; "Password in URL" boundary checks), `:58` (`Password in URL` pattern `://[^:]+:[^@]+@`), `:62` (`Generic Token` pattern), applied at the live sinks (V02)
- Reachability: any content passing through the output guard/redaction sinks.
- Expected invariant: redaction precision separates real credentials from documentation/example text.
- Observed behavior: heuristic patterns redact harmless matches (e.g., a doc line `mysql://user:pass@host` example, `token: 12345678` placeholder), degrading tool output and trace content.
- Impact: local model: annoyance/UX corruption of content, no exposure (over-redaction, not under-redaction); the under-redaction direction (missing high-confidence splits) has no concrete exposure identified at the sinks.
- Root cause: deferred design work from the 2026-05-31 audit.
- Direction: implement the TODO split (high-confidence patterns block; heuristic warn-only) or downgrade the TODO to a documented precision trade-off; keep `redact_secrets` boundary safety as-is.
- Regression validation: existing 30 root security tests stay green; add a doc-comment false-positive fixture asserting no redaction.
- Validation reports: [V02](../validations/F-SEC-01/V02-01.md), [V05-04](../validations/F-SEC-01/V05-04.md)

### F-SEC-01-P3-11: Raw URL logged unredacted in `web_fetch`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/web/fetch.rs:203` (`tracing::info!("WebFetch: url='{}', max_length={}", url, max_length)`) — no `redact_secrets`; the "Password in URL" pattern that would catch `https://user:pass@host/` exists at `src/security.rs:58` but is not applied here
- Reachability: any `web_fetch` call with credentials embedded in the URL (common for internal tooling) logs them verbatim to the local tracing log.
- Expected invariant: "不把密钥打进日志" (AGENTS.md).
- Observed behavior: the URL is logged raw before any redaction.
- Impact: credentials in local logs (files/trace output); local model keeps it minor, but it is exactly the log hygiene the AGENTS.md mandates.
- Root cause: log line predates the redaction helper; no review pass over logging sites.
- Direction: run the URL through `redact_secrets` before logging (or log only scheme+host).
- Regression validation: unit test asserting the log string for `https://user:pass@host/` contains no `pass`.
- Validation reports: [V02](../validations/F-SEC-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Threat-boundary classification per mechanism; terminal vs run_code | yes | passed | [V01](../validations/F-SEC-01/V01-01.md) |
| V02 | Secret/log redaction call-site coverage | yes | passed | [V02](../validations/F-SEC-01/V02-01.md) |
| V03 | Sandbox fallback/minimum-isolation invariant | yes | passed | [V03](../validations/F-SEC-01/V03-01.md) |
| V04 | Panic/UTF-8/path-traversal scan + AUDIT re-rating | yes | passed | [V04](../validations/F-SEC-01/V04-01.md) |
| V05 | `cargo test -p echo_tools --lib --locked security` (37 passed, exit 0) | conditional | passed | [V05-01](../validations/F-SEC-01/V05-01.md) |
| V05 | `cargo test -p echo_core --lib --locked guard` (0 tests) and `--features guard guard` (16 passed, exit 0) | conditional | passed | [V05-02](../validations/F-SEC-01/V05-02.md) |
| V05 | `cargo test -p echo_core --lib --locked sandbox` (3 passed, exit 0) | conditional | passed | [V05-03](../validations/F-SEC-01/V05-03.md) |
| V05 | `cargo test -p echo_agent --lib --locked security::` (30 passed, exit 0) | conditional | passed | [V05-04](../validations/F-SEC-01/V05-04.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT 1.4 / B-DOC-01 P2-01: eval runner executes unsanitized `sh -c` | partially fixed (SweBench validated; TestPass + newline gap remain); re-rated P3 under local model | `src/eval/runner.rs:343-346,:680-689` vs `:226-237`; finding F-SEC-01-P3-01; B-DOC-01's `:340` anchor is stale at the same commit |
| AUDIT 1.9 / B-DOC-01 P3-01: production `unsafe { set_var }` | current; stays P3 (bounded contract) | `echo-core/src/plugin/variables.rs:177-212`; finding F-SEC-01-P3-08 |
| AUDIT 2.8: `parse_method` hand-rolled | current; stays P3 (proptest-hardened; routing-only impact) | `src/a2a/serve.rs:296-321`; finding F-SEC-01-P3-09 |
| AUDIT 3.2: fallback `Client::new` | current; re-rated P3 (fetch.rs fallback unreachable for requests; timeout-only loss) | `echo-tools/src/web/fetch.rs:43-46,:56-60,:201`, `tavily.rs:37-39`, `duckduckgo.rs:35-37`, `brave.rs:31-33`, `image_fetch.rs:125`; finding F-SEC-01-P3-05 |
| AUDIT 6.1: security TODOs | current; stays P3 (false-positive redaction only) | `src/security.rs:3-12`; finding F-SEC-01-P3-10 |
| AUDIT 6.2: dead-code annotations | current; security-relevant instance identified | `execution.rs:225`, `fetch.rs:56`; finding F-SEC-01-P3-07 |
| AGENTS.md lesson: `require_full_auto` gates on terminal/MCP removed | fixed (no callers); residue = dead helper + stale doc | `echo-agent-cli/src/tauri/error.rs:5-12,:45`; `terminal.rs:281-295`; finding F-SEC-01-P3-04 |
| B-DOC-01: "run_code 最小隔离 OsSandbox at code.rs:313" | current (whole chain verified fail-closed in EKO) | `echo-tools/src/code.rs:313-345`; `infra.rs:265-271`; [V03](../validations/F-SEC-01/V03-01.md) |
| Root MASTER-PLAN "已知需要校正的事实" (run_code isolation) | current | [V03](../validations/F-SEC-01/V03-01.md) |

## Coverage And Uncertainty

- Not executed: echo-execution local/docker/k8s backend unit tests (require OS backends/docker; static + core tests only). Q-dynamic tasks may run the full matrix.
- `echo-tools/src/files/mod.rs` resolver reviewed only at the grep level (its canonicalize-parent approach is sound); deep ownership is F-EXT-01/A-* scope.
- The `ToolRiskClassifier` (echo-execution/src/risk.rs) is dead framework API (zero callers) — noted, not re-rated into a finding; Q-STA-01 owns dead-code sweep.
- EKO-side redaction (chat display, persistence) not inspected — A-* tasks.
- SSRF "over-gating" (P3-03) is a judgment call: the framework default is defensible; the direction requires an EKO product decision, so it stays a recommendation.
- All findings are P3: under the local threat model no P0/P1/P2 case surfaced (no data-loss path, no reachable secret exposure beyond local logs, no core-path breakage; over-gating residue is dead code).

## Handoff

- Downstream tasks may rely on: (1) run_code's OsSandbox floor holds on the EKO path with fail-closed degradation (V03); (2) redaction covers all primary framework sinks; the only raw-URL log is `fetch.rs:203` (V02); (3) the 6 B-DOC-01 current items re-rated P3 with fresh anchors (V04); (4) `IpcAuth` is dead — A-HITL-01/A-AUT-01 must not treat it as an existing gate; (5) `validate_within_base` is the intended canonical validator — A-FILE/A-TOOL tasks should converge there instead of extending `validate_output_file`.
- Reports to read: all 8 validation reports in `validations/F-SEC-01/`.
- Stale conditions: changes to `SandboxManager` fallback logic, `evaluate_with_limits`, `code.rs` pre-checks, `eval/runner.rs` criteria paths, `tauri/error.rs`, or the redaction sinks.
- Follow-up task IDs: A-HITL-01 / A-AUT-01 (permission-mode semantics; IpcAuth removal), F-EXT-01 (web fetch / path validators), Q-STA-01 (dead-code sweep incl. ToolRiskClassifier, fetch.rs field, IpcAuth), Q-DEP-01 (provider clients), A-SRF-04 (unused-arg cleanup overlaps).
