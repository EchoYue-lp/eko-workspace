# Sprint 10b: `run_code` Tool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `run_code` framework tool that lets data/research worker subagents execute arbitrary Python/R/JS/... code snippets, automatically running inside the worker's isolated tmpdir workspace (Sprint 10's `working_dir` chain).

**Architecture:** New greenfield tool `RunCodeTool` in `echo-tools/src/code.rs`, modeled on `ShellTool` (`echo-tools/src/shell.rs`): holds an `Option<Arc<dyn SandboxExecutor>>` (the echo-core trait, no echo-execution dependency), overrides `execute_with_context` to read `ctx.working_dir` and bind it to the `SandboxCommand`. Languages: python/r/javascript/ruby/perl/php/bash. RCE guardrail = "warn-not-deny" when no sandbox (EKO local-assistant model). Also patches the `Code` backend (`local.rs` + `docker.rs`) to natively support R (currently omitted → silent mis-run on docker, error on local).

**Tech Stack:** Rust 2024, `echo_core` (`Tool` trait, `SandboxExecutor`, `SandboxCommand`, `ToolContext`), `echo_tools` (domain tools crate), `echo_execution` (sandbox backends). No new dependencies (tokio/tracing/serde_json already in echo-tools Cargo.toml).

**Spec:** `docs/superpowers/specs/2026-07-01-sprint-10b-and-11-design.md` §二 (Sprint 10b).

**Scope rule (AGENTS.md):** framework pub API stays in `echo-agent`. The tool itself is a domain tool → `echo-tools` (alongside `data.rs`/`statistics.rs`/`shell.rs`). Backend patches are framework sandbox internals → `echo-execution`.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `echo-agent/echo-execution/src/sandbox/local.rs` | Modify (line 132-145) | Add R to `build_code_command` interpreter match |
| `echo-agent/echo-execution/src/sandbox/docker.rs` | Modify (line 273-283) | Add R to `build_inner_command` Code branch (fix silent `sh -c` fallback) |
| `echo-agent/echo-tools/src/code.rs` | Create | New `RunCodeTool` (the tool itself) |
| `echo-agent/echo-tools/src/lib.rs` | Modify | Add `pub mod code;` (gated under `shell` feature, same as shell) |
| `echo-agent/echo-tools/src/registry.rs` | Modify (line 197+) | Register `RunCodeTool` in `register_all_tools` |
| `echo-agent-cli/echo-agent-app-core/src/infra.rs` | Modify | Register `RunCodeTool` on data-worker agents (data-shaper/analyst) |
| `echo-agent-cli/echo-agent-app-core/src/subagents/data/data-shaper.md` | Modify | Tell LLM `run_code` is available + working_dir semantics |
| `echo-agent-cli/echo-agent-app-core/src/subagents/data/analyst.md` | Modify | Same |

**Existing helpers reused (do NOT reinvent):**
- `echo_core::sandbox::{SandboxCommand, SandboxExecutor, ExecutionResult}` — `SandboxCommand::code(language, code)` constructor (`echo-core/src/sandbox.rs:101`), `.with_working_dir()` builder (`:142`), `ExecutionResult::success()` / `combined_output()` (`:183-198`).
- `Tool` trait shape (`echo-core/src/tools/mod.rs:453`): override `execute_with_context`, set `permissions()` → `vec![ToolPermission::Execute]`, `risk_level()` → `ToolRiskLevel::Dangerous`.
- `ToolResult::success(output)` / `ToolResult::error(msg)` constructors (`echo-core/src/tools/mod.rs:69-78`).

---

## Task 1: Patch `local.rs` to natively support R in `Code` backend

**Files:**
- Modify: `echo-agent/echo-execution/src/sandbox/local.rs:132-145` (`build_code_command` match)

- [ ] **Step 1: Add a unit test asserting R maps to `Rscript -e`**

Edit `echo-agent/echo-execution/src/sandbox/local.rs` — find the `#[cfg(test)] mod tests` at the bottom of the file (or add one if absent) and add:

```rust
    #[test]
    fn build_code_command_supports_r() {
        // Sprint 10b: R must map to ("Rscript", "-e"), not fall through to
        // the Unavailable error. Mirrors python/ruby/perl's -e flag convention.
        let sandbox = super::LocalSandbox::default();
        let cmd = SandboxCommand::code("r", "print(1+1)");
        let built = sandbox.build_code_command("r", "print(1+1)", &cmd);
        assert!(built.is_ok(), "R should be supported in Code backend");
        // We can't easily assert the exact Command argv here (platform-specific),
        // but is_ok proves the match arm fired instead of the Unavailable error.
    }
```

If `LocalSandbox::default()` is not accessible / the method is private, instead write the test at the `build_code_command` granularity by checking the public `execute()` path. Fallback test (use this if the above doesn't compile):

```rust
    #[tokio::test]
    async fn code_backend_supports_r_language_mapping() {
        // Sprint 10b: R is a first-class language. Before the patch this
        // returned SandboxError::Unavailable("Unsupported language: r").
        // After the patch it must proceed to interpreter resolution.
        // We can't assume Rscript is installed in CI, so we only assert
        // the error is NOT "Unsupported language" (i.e. the match arm fired).
        let sandbox = super::LocalSandbox::default();
        let cmd = SandboxCommand::code("r", "print(1)");
        match sandbox.execute(cmd).await {
            Ok(_) => { /* Rscript present + ran */ }
            Err(e) => {
                let msg = format!("{e:?}");
                assert!(
                    !msg.contains("Unsupported language"),
                    "R should not hit the Unsupported-language arm. Got: {msg}"
                );
            }
        }
    }
```

Use whichever compiles. Place inside the existing `#[cfg(test)] mod tests` in `local.rs`.

- [ ] **Step 2: Run the test to verify it FAILS**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_execution --lib sandbox::local::tests
```
Expected: FAIL (R hits the `_ => return Err(SandboxError::Unavailable("Unsupported language: r"))` arm; the fallback test's assert triggers because the message contains "Unsupported language").

- [ ] **Step 3: Add the R match arm**

In `echo-agent/echo-execution/src/sandbox/local.rs`, find `build_code_command` (around line 132). The match looks like:

```rust
        let (interpreter, flag) = match language {
            "python" | "python3" => ("python3", "-c"),
            "node" | "javascript" | "js" => ("node", "-e"),
            "ruby" => ("ruby", "-e"),
            "perl" => ("perl", "-e"),
            "lua" => ("lua", "-e"),
            "php" => ("php", "-r"),
            "bash" | "sh" => ("sh", "-c"),
            _ => {
                return Err(SandboxError::Unavailable(format!(
                    "Unsupported language: {language}"
                )));
            }
        };
```

Add the R arm **before the `_` fallback** (keep the others unchanged):

```rust
        let (interpreter, flag) = match language {
            "python" | "python3" => ("python3", "-c"),
            "node" | "javascript" | "js" => ("node", "-e"),
            "ruby" => ("ruby", "-e"),
            "r" => ("Rscript", "-e"),
            "perl" => ("perl", "-e"),
            "lua" => ("lua", "-e"),
            "php" => ("php", "-r"),
            "bash" | "sh" => ("sh", "-c"),
            _ => {
                return Err(SandboxError::Unavailable(format!(
                    "Unsupported language: {language}"
                )));
            }
        };
```

- [ ] **Step 4: Run the test to verify it PASSES**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_execution --lib sandbox::local::tests
```
Expected: PASS (R no longer hits the Unsupported-language arm).

- [ ] **Step 5: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
git add echo-execution/src/sandbox/local.rs
git -c commit.gpgsign=false commit -m "feat(sandbox): Code 后端原生支持 R (Rscript -e)

Sprint 10b Task 1: local.rs build_code_command 加 \"r\" => (\"Rscript\", \"-e\")
匹配臂。此前 R 走 _ 兜底返回 SandboxError::Unavailable,数据 worker
无法用 CommandKind::Code 跑 R 脚本。R 现作为一等语言(arg-based,与
python/ruby/perl 一致)。Docker 后端修补见下一 commit。"
```

---

## Task 2: Patch `docker.rs` to natively support R in `Code` backend (fix silent mis-run)

**Files:**
- Modify: `echo-agent/echo-execution/src/sandbox/docker.rs:273-283` (`build_inner_command` Code branch)

**Context:** The docker Code branch currently has `_ => ("sh", "-c")` which **silently** runs R code as a shell command (wrong interpreter, no error — worst kind of bug). Per decision D-10b-docker-r-1, we add R explicitly and trust the `rocker/r-base:latest` image exists (already mapped at `echo-execution/src/sandbox/mod.rs:84`); if the image is missing, Docker engine raises `ImageNotFound`/`CommandFailed` which the tool layer surfaces — no tool-layer probing.

- [ ] **Step 1: Add a unit test asserting R maps to `Rscript -e` (not `sh -c`)**

`build_inner_command` is a pure function returning `Vec<String>` — easy to test directly. Find or add `#[cfg(test)] mod tests` at the bottom of `docker.rs` and add:

```rust
    #[test]
    fn build_inner_command_maps_r_to_rscript() {
        // Sprint 10b: before the patch, R fell through to the `_ => ("sh","-c")`
        // arm and was SILENTLY mis-run as shell. After the patch it must map
        // to ("Rscript","-e") like python/node.
        use echo_core::sandbox::{CommandKind, SandboxCommand};
        use std::collections::HashMap;
        use std::time::Duration;
        let cmd = SandboxCommand {
            kind: CommandKind::Code {
                language: "r".to_string(),
                code: "print(1+1)".to_string(),
            },
            working_dir: None,
            env: HashMap::new(),
            timeout: Duration::from_secs(30),
            stdin: None,
        };
        let v = super::DockerSandbox::build_inner_command(&cmd);
        assert_eq!(v[0], "Rscript", "R must map to Rscript, got: {:?}", v);
        assert_eq!(v[1], "-e");
        assert_eq!(v[2], "print(1+1)");
    }
```

NOTE: confirm `build_inner_command` is accessible as `DockerSandbox::build_inner_command` (it's an associated `fn` at `docker.rs:267`, takes `&SandboxCommand`). If it's a private method (`fn` not `pub fn`), either (a) make it `pub(crate)` for the test, or (b) call it via `super::DockerSandbox` if already `pub`. Check the actual visibility first — if private, the simplest fix is to test through the public match shape by extracting the language→(interpreter,flag) into a tiny testable helper. Prefer (a): add `pub` to the `fn build_inner_command` line if it isn't already (it's called from sibling methods in the same impl, so `pub` is harmless).

- [ ] **Step 2: Run the test to verify it FAILS**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_execution --lib sandbox::docker::tests
```
Expected: FAIL (`v[0]` is `"sh"` not `"Rscript"` — the current `_` fallback).

- [ ] **Step 3: Add the R match arm**

In `echo-agent/echo-execution/src/sandbox/docker.rs`, find `build_inner_command` (around line 267). The Code branch looks like:

```rust
            CommandKind::Code { language, code } => {
                let (interpreter, flag) = match language.as_str() {
                    "python" | "python3" => ("python3", "-c"),
                    "node" | "javascript" | "js" => ("node", "-e"),
                    "ruby" => ("ruby", "-e"),
                    "perl" => ("perl", "-e"),
                    "php" => ("php", "-r"),
                    _ => ("sh", "-c"),
                };
                vec![interpreter.to_string(), flag.to_string(), code.clone()]
            }
```

Add the R arm before the `_` fallback:

```rust
            CommandKind::Code { language, code } => {
                let (interpreter, flag) = match language.as_str() {
                    "python" | "python3" => ("python3", "-c"),
                    "node" | "javascript" | "js" => ("node", "-e"),
                    "ruby" => ("ruby", "-e"),
                    "r" => ("Rscript", "-e"),
                    "perl" => ("perl", "-e"),
                    "php" => ("php", "-r"),
                    _ => ("sh", "-c"),
                };
                vec![interpreter.to_string(), flag.to_string(), code.clone()]
            }
```

- [ ] **Step 4: Run the test to verify it PASSES**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_execution --lib sandbox::docker::tests
```
Expected: PASS (`v[0] == "Rscript"`).

- [ ] **Step 5: Run full echo_execution crate tests + fmt**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_execution
cargo fmt --all
cargo fmt --all -- --check
```
Expected: all PASS, fmt clean (exit 0).

- [ ] **Step 6: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
git add echo-execution/src/sandbox/docker.rs
git -c commit.gpgsign=false commit -m "fix(sandbox): docker Code 后端原生支持 R,修复静默 sh -c 误跑

Sprint 10b Task 2: docker.rs build_inner_command Code 分支加 \"r\" =>
(\"Rscript\", \"-e\")。此前 R 走 _ => (\"sh\",\"-c\") 兜底,代码被当作
shell 命令静默误跑(无报错,最坏 bug)。现对齐 local.rs。镜像缺失由
Docker 引擎抛 ImageNotFound,工具层统一捕获(D-10b-docker-r-1)。"
```

---

## Task 3: Create `RunCodeTool` (`echo-tools/src/code.rs`)

**Files:**
- Create: `echo-agent/echo-tools/src/code.rs`

**Reference patterns (already verified in codebase):**
- `ShellTool` (`echo-tools/src/shell.rs:129-164`): holds `Option<Arc<dyn SandboxExecutor>>`, builder `.with_sandbox(...)`, `execute_with_context` reads `ctx.working_dir`.
- `Tool` trait (`echo-core/src/tools/mod.rs:453`).
- `SandboxCommand::code(language, code)` + `.with_working_dir(dir)` (`echo-core/src/sandbox.rs:101,142`).
- `ExecutionResult::success()` + `.combined_output()` (`echo-core/src/sandbox.rs:183`).

- [ ] **Step 1: Write the failing unit test first (TDD)**

Create `echo-agent/echo-tools/src/code.rs` with ONLY the test module + a stub `RunCodeTool` struct (no real impl yet), so the test fails for the right reason (method/field missing):

```rust
//! Inline code execution tool (Sprint 10b).
//!
//! Lets a subagent execute arbitrary Python/R/JS/... code snippets. The code
//! automatically runs inside `ctx.working_dir` (the worker's isolated tmpdir
//! for data/research workers — Sprint 10's `DataWorkspaceFactory` chain).
//!
//! Security model (AGENTS.md "local personal assistant"):
//! - With a configured `SandboxExecutor`: runs via the sandbox (Docker/OS/etc.).
//! - Without a sandbox: `tracing::warn!` + runs bare (local trusted machine —
//!   refusing would break out-of-box UX). This is the opposite of a web
//!   service's zero-trust deny.
//!
//! Modeled on `ShellTool` (`shell.rs`): holds `Option<Arc<dyn SandboxExecutor>>`
//! (the echo-core trait, no echo-execution dependency) and overrides
//! `execute_with_context` to honor `ctx.working_dir`.

use echo_core::error::{Result as ToolResult, ToolError};
use echo_core::sandbox::{ExecutionResult, SandboxCommand, SandboxExecutor};
use echo_core::tools::permission::ToolPermission;
use echo_core::tools::{Tool, ToolParameters, ToolResult as ToolOutput, ToolContext, ToolRiskLevel};
use futures::future::BoxFuture;
use std::sync::Arc;

/// Languages supported by [`RunCodeTool`]. All use arg-based execution
/// (`-c`/`-e` flag) for consistency with the existing `Code` backend
/// (`local.rs`/`docker.rs`). (Switching all languages to stdin-based
/// execution to avoid ARG_MAX is a cross-cutting future follow-up, not
/// in scope for Sprint 10b — see D-10b-stdin-1.)
const SUPPORTED_LANGUAGES: &[&str] = &[
    "python",
    "python3",
    "r",
    "javascript",
    "js",
    "node",
    "ruby",
    "perl",
    "php",
    "bash",
    "sh",
];

/// Validate a language against the supported set (case-insensitive).
/// Returns the normalized lowercase language, or an error.
fn validate_language(language: &str) -> ToolResult<String> {
    let normalized = language.to_lowercase();
    if SUPPORTED_LANGUAGES.iter().any(|&l| l == normalized) {
        Ok(normalized)
    } else {
        // ToolError::InvalidParameter is a struct variant {name, message}
        // (echo-core/src/error.rs:118).
        Err(ToolError::InvalidParameter {
            name: "language".to_string(),
            message: format!(
                "Unsupported language '{language}'. Supported: {:?}",
                SUPPORTED_LANGUAGES
            ),
        })
    }
}

/// Inline code execution tool.
pub struct RunCodeTool {
    sandbox: Option<Arc<dyn SandboxExecutor>>,
    /// Per-call timeout in seconds (default 60, capped at 300 like ShellTool).
    timeout_secs: u64,
}

impl Default for RunCodeTool {
    fn default() -> Self {
        Self {
            sandbox: None,
            timeout_secs: 60,
        }
    }
}

impl RunCodeTool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inject a sandbox executor (Docker / OS-sandbox / local). Without this,
    /// the tool falls back to a bare `tokio::process` run with a warning.
    pub fn with_sandbox(mut self, sandbox: Arc<dyn SandboxExecutor>) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Per-call timeout (default 60s, capped at 300s like ShellTool).
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_language_lowercases_and_accepts_known() {
        // Decision D-10b-case-1: LLM may emit "Python"/"PYTHON"/"R".
        assert_eq!(validate_language("Python").unwrap(), "python");
        assert_eq!(validate_language("PYTHON").unwrap(), "python");
        assert_eq!(validate_language("R").unwrap(), "r");
        assert_eq!(validate_language("JavaScript").unwrap(), "javascript");
    }

    #[test]
    fn validate_language_rejects_unknown() {
        // Circuit-breaker (user review patch #1): unknown language fails at the
        // tool layer, never reaches the sandbox.
        assert!(validate_language("haskell").is_err());
        assert!(validate_language("").is_err());
    }

    #[test]
    fn supported_languages_includes_r() {
        // Sprint 10b headline: R is a first-class language.
        assert!(SUPPORTED_LANGUAGES.contains(&"r"));
        assert!(SUPPORTED_LANGUAGES.contains(&"python"));
    }
}
```

NOTE on `ToolError::InvalidParameter`: it is a **struct variant** `{ name: String, message: String }` (verified at `echo-core/src/error.rs:118`). The code above already uses the correct shape. Do NOT write `InvalidParameter(String)` (won't compile).

- [ ] **Step 2: Register the module in `lib.rs`**

Edit `echo-agent/echo-tools/src/lib.rs`. Find the shell module block (around the top of the file):

```rust
#[cfg(feature = "shell")]
#[cfg_attr(docsrs, doc(cfg(feature = "shell")))]
pub mod shell;
```

Add `code` right after it, under the same `shell` feature gate (code execution is the same risk class as shell — no point enabling one without the other):

```rust
#[cfg(feature = "shell")]
#[cfg_attr(docsrs, doc(cfg(feature = "shell")))]
pub mod shell;

/// Sprint 10b: inline code execution tool (run_code). Gated under `shell`
/// because it's the same execute/risk class.
#[cfg(feature = "shell")]
#[cfg_attr(docsrs, doc(cfg(feature = "shell")))]
pub mod code;
```

- [ ] **Step 3: Run the tests to verify the THREE unit tests PASS**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_tools --features shell --lib code::
```
Expected: PASS (3 tests: `validate_language_lowercases_and_accepts_known`, `validate_language_rejects_unknown`, `supported_languages_includes_r`).

- [ ] **Step 4: Implement the `Tool` trait for `RunCodeTool`**

Append to `echo-agent/echo-tools/src/code.rs`, after the `impl RunCodeTool` block:

```rust
impl Tool for RunCodeTool {
    fn name(&self) -> &str {
        "run_code"
    }

    fn description(&self) -> &str {
        "执行一段代码(Python/R/JavaScript/...)。代码自动在当前任务工作目录(working_dir)中运行 — 无需创建新目录,直接读写当前目录文件即可。返回 stdout/stderr/exit code。"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "enum": ["python", "r", "javascript", "ruby", "perl", "php", "bash"],
                    "description": "代码语言(大小写不敏感)。默认 python。"
                },
                "code": {
                    "type": "string",
                    "description": "要执行的代码片段。"
                },
                "timeout": {
                    "type": "integer",
                    "description": "超时秒数(可选,默认 60,上限 300)。"
                }
            },
            "required": ["language", "code"]
        })
    }

    fn permissions(&self) -> Vec<ToolPermission> {
        vec![ToolPermission::Execute]
    }

    fn risk_level(&self) -> ToolRiskLevel {
        ToolRiskLevel::Dangerous
    }

    fn execute_with_context<'a>(
        &'a self,
        parameters: ToolParameters,
        ctx: &'a ToolContext,
    ) -> BoxFuture<'a, ToolResult<ToolOutput>> {
        Box::pin(async move {
            // 1. Parse + circuit-break on unknown language (user review patch #1).
            let raw_lang = parameters
                .get("language")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::MissingParameter("language".to_string()))?;
            let language = validate_language(raw_lang)?;

            let code = parameters
                .get("code")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::MissingParameter("code".to_string()))?;

            let timeout_secs = parameters
                .get("timeout")
                .and_then(|v| v.as_u64())
                .unwrap_or(self.timeout_secs)
                .min(300);
            let timeout_duration = tokio::time::Duration::from_secs(timeout_secs);

            // 2. Build the sandbox command, binding the worker's working_dir.
            //    `with_timeout` takes a Duration (echo-core/src/sandbox.rs:148),
            //    not seconds.
            let mut sandbox_cmd = SandboxCommand::code(&language, code);
            if let Some(dir) = &ctx.working_dir {
                sandbox_cmd = sandbox_cmd.with_working_dir(dir.clone());
            }
            sandbox_cmd = sandbox_cmd.with_timeout(timeout_duration);

            // 3. Execute via sandbox if configured, else warn + bare fallback.
            if let Some(sandbox) = &self.sandbox {
                match tokio::time::timeout(
                    timeout_duration,
                    sandbox.execute(sandbox_cmd),
                )
                .await
                {
                    Ok(Ok(result)) => Ok(format_execution_result(&result)),
                    Ok(Err(e)) => Ok(ToolOutput::error(format!(
                        "Sandbox execution failed: {e}"
                    ))),
                    Err(_) => Ok(ToolOutput::error(format!(
                        "⏱️ Code execution timed out after {timeout_secs}s"
                    ))),
                }
            } else {
                // Decision D-10b-RCE-1: warn-not-deny. EKO is a local personal
                // assistant; refusing here would break out-of-box UX.
                tracing::warn!(
                    language = %language,
                    "run_code: no SandboxExecutor configured — running unsandboxed. \
                     Ensure you trust the generated code."
                );
                run_bare(&language, code, ctx, timeout_duration).await
            }
        })
    }
}

/// Render an `ExecutionResult` as a `ToolResult` (success or error).
fn format_execution_result(result: &ExecutionResult) -> ToolResult<ToolOutput> {
    if result.success() {
        Ok(ToolOutput::success(result.combined_output()))
    } else {
        Ok(ToolOutput::error(format!(
            "Code execution failed (exit code {}).\n{}",
            result.exit_code,
            result.combined_output()
        )))
    }
}

/// Bare-process fallback when no `SandboxExecutor` is configured.
///
/// Writes nothing to disk; passes code via the interpreter's `-c`/`-e` flag,
/// honoring `ctx.working_dir` as the process's `current_dir`. Mirrors the
/// arg-based convention of the `Code` backend.
async fn run_bare(
    language: &str,
    code: &str,
    ctx: &ToolContext,
    timeout_duration: std::time::Duration,
) -> ToolResult<ToolOutput> {
    let (interpreter, flag) = match language {
        "python" | "python3" => ("python3", "-c"),
        "node" | "javascript" | "js" => ("node", "-e"),
        "r" => ("Rscript", "-e"),
        "ruby" => ("ruby", "-e"),
        "perl" => ("perl", "-e"),
        "php" => ("php", "-r"),
        "bash" | "sh" => ("sh", "-c"),
        other => {
            return Err(ToolError::InvalidParameter {
                name: "language".to_string(),
                message: format!("Unsupported language '{other}'"),
            })
        }
    };

    let mut command = tokio::process::Command::new(interpreter);
    command.arg(flag).arg(code);
    command.kill_on_drop(true);
    if let Some(dir) = &ctx.working_dir {
        command.current_dir(dir);
    }

    let output = tokio::time::timeout(timeout_duration, command.output())
        .await
        .map_err(|_| {
            ToolError::ExecutionFailed {
                tool: "run_code".to_string(),
                message: format!("Code execution timed out after {timeout_duration:?}"),
            }
        })?
        .map_err(|e| ToolError::ExecutionFailed {
            tool: "run_code".to_string(),
            message: format!("Failed to spawn interpreter {interpreter}: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        let combined = if stderr.is_empty() {
            stdout
        } else {
            format!("{stdout}\n{stderr}")
        };
        Ok(ToolOutput::success(combined))
    } else {
        Ok(ToolOutput::error(format!(
            "Code execution failed (exit code {}).\n{}\n{}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        )))
    }
}
```

- [ ] **Step 5: Run cargo check to catch any remaining signature mismatches**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo check -p echo_tools --features shell
```
The plan already uses verified-correct signatures (`ToolError::InvalidParameter { name, message }` struct variant, `with_timeout(Duration)`, `env: HashMap`). If anything still fails to compile, re-check the actual definition in `echo-core/src/{error,sandbox,tools/mod}.rs` and adjust — do NOT guess.

- [ ] **Step 6: Add an integration test for working_dir propagation**

Append to `echo-tools/src/code.rs` `#[cfg(test)] mod tests`:

```rust
    use echo_core::sandbox::{ExecutionResult, IsolationLevel};
    use std::time::Duration;
    use futures::future::BoxFuture;

    /// A stub sandbox that records the working_dir it was asked to use.
    struct RecordingSandbox {
        seen_working_dir: std::sync::Mutex<Option<std::path::PathBuf>>,
    }

    impl SandboxExecutor for RecordingSandbox {
        fn name(&self) -> &str { "recording" }
        fn isolation_level(&self) -> IsolationLevel { IsolationLevel::None }
        fn is_available(&self) -> BoxFuture<'_, bool> { Box::pin(async { true }) }
        fn execute<'a>(&'a self, command: SandboxCommand) -> BoxFuture<'a, ToolResult<ExecutionResult>> {
            let seen = self.seen_working_dir.lock().unwrap().clone();
            let _ = seen; // record for inspection
            *self.seen_working_dir.lock().unwrap() = command.working_dir.clone();
            Box::pin(async move {
                Ok(ExecutionResult {
                    exit_code: 0,
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    duration: Duration::from_millis(1),
                    sandbox_type: "recording".to_string(),
                    timed_out: false,
                })
            })
        }
    }

    #[tokio::test]
    async fn run_code_binds_ctx_working_dir() {
        // Sprint 10b headline: data worker's tmpdir (ctx.working_dir) must be
        // propagated to the SandboxCommand so the code runs in the worker's
        // isolated workspace, not the process cwd.
        use std::collections::HashMap;
        let sandbox = Arc::new(RecordingSandbox {
            seen_working_dir: std::sync::Mutex::new(None),
        });
        let captured: Arc<RecordingSandbox> = sandbox.clone();
        let tool = RunCodeTool::new().with_sandbox(sandbox);

        // ToolParameters = HashMap<String, serde_json::Value> (echo-core:266).
        // Idiomatic construction (matches shell.rs:756, web/search.rs:350):
        let mut params = HashMap::new();
        params.insert("language".to_string(), serde_json::json!("python"));
        params.insert("code".to_string(), serde_json::json!("print(1+1)"));

        let ctx = ToolContext {
            working_dir: Some(std::path::PathBuf::from("/tmp/eko-data-worker-xyz")),
            conversation_id: None,
            run_id: None,
            cancel: None,
            trace_sink: None,
        };
        let _ = tool.execute_with_context(params, &ctx).await.unwrap();

        let seen = captured.seen_working_dir.lock().unwrap().clone();
        assert_eq!(
            seen,
            Some(std::path::PathBuf::from("/tmp/eko-data-worker-xyz")),
            "ctx.working_dir must propagate to the sandbox command"
        );
    }
```

NOTE before running: (a) verify `ToolParameters::from_json` is the correct constructor — check `echo-core/src/tools/mod.rs` for the actual method (could be `ToolParameters::from(serde_json::Value)` via impl, or a `from_value`, or just `ToolParameters(value)`). Adjust to the real API. (b) Verify `IsolationLevel::None` exists in `echo-core/src/sandbox.rs`. (c) If `RecordingSandbox` can't capture because `execute` consumes `command`, clone the `working_dir` into the mutex BEFORE the await point (as shown).

- [ ] **Step 7: Run all code.rs tests**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_tools --features shell --lib code::
```
Expected: all PASS (3 unit + 1 integration).

- [ ] **Step 8: fmt + clippy**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo fmt --all
cargo fmt --all -- --check
cargo clippy -p echo_tools --features shell --all-targets -- -D warnings
```
Expected: fmt clean (exit 0), clippy zero warnings.

- [ ] **Step 9: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
git add echo-tools/src/code.rs echo-tools/src/lib.rs
git -c commit.gpgsign=false commit -m "feat(tools): 新增 run_code 工具(内联 Python/R/JS 代码执行)

Sprint 10b Task 3: echo-tools/src/code.rs 的 RunCodeTool。模型 = ShellTool
(持 Option<Arc<dyn SandboxExecutor>> echo-core trait,无 echo-execution 依赖)
+ execute_with_context 读 ctx.working_dir(Sprint 10 数据 worker tmpdir 链路)。

关键决策:
- RCE 护栏 'warn 不拒'(D-10b-RCE-1):无 sandbox 时 tracing::warn + 走 bare
  tokio::process 回退(本地个人助理模型,拒会破坏开箱即用)。
- 语言白名单大小写不敏感(D-10b-case-1):入口 .to_lowercase(),防 LLM 输出
  'Python'/'PYTHON'。
- R 作为一等语言(arg-based,与所有语言一致)。
- 全语言切 stdin 是 cross-cutting 优化,留 follow-up(D-10b-stdin-1)。

测试:validate_language × 3 + working_dir 传播集成测试。"
```

---

## Task 4: Register `RunCodeTool` in `register_all_tools`

**Files:**
- Modify: `echo-agent/echo-tools/src/registry.rs:197` (`register_all_tools`)

- [ ] **Step 1: Read the current `register_all_tools` body**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
sed -n '197,230p' echo-tools/src/registry.rs
```
Confirm where `ShellTool` is registered (around line 201-202) — `run_code` goes next to it (same shell feature gate).

- [ ] **Step 2: Add a test asserting `register_all_tools` includes `run_code`**

Find `#[cfg(test)] mod tests` in `registry.rs` (or add one) and add:

```rust
    #[test]
    fn register_all_tools_includes_run_code() {
        // Sprint 10b: run_code must be in the writer toolset.
        use crate::code::RunCodeTool;
        struct Collector { names: std::sync::Mutex<Vec<String>> }
        impl echo_core::tools::ToolRegistrar for Collector {
            fn register(&mut self, tool: Box<dyn echo_core::tools::Tool>) {
                self.names.lock().unwrap().push(tool.name().to_string());
            }
        }
        let mut c = Collector { names: std::sync::Mutex::new(vec![]) };
        crate::register_all_tools(&mut c);
        let names = c.names.lock().unwrap().clone();
        assert!(names.contains(&"run_code".to_string()), "run_code missing from register_all_tools: {:?}", names);
    }
```

- [ ] **Step 3: Run the test to verify it FAILS**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_tools --features shell --lib registry::tests
```
Expected: FAIL (`run_code` not yet registered).

- [ ] **Step 4: Add the registration line**

In `echo-agent/echo-tools/src/registry.rs`, inside `register_all_tools` (around line 201, right after `tool_manager.register(Box::new(ShellTool::new()));`):

```rust
        use crate::shell::ShellTool;
        tool_manager.register(Box::new(ShellTool::new()));
        // Sprint 10b: inline code execution (Python/R/JS/...). Same shell
        // feature gate; writer toolset only (readonly subset excludes it).
        tool_manager.register(Box::new(crate::code::RunCodeTool::new()));
```

- [ ] **Step 5: Run the test to verify it PASSES**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
cargo test -p echo_tools --features shell --lib registry::tests
```
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
git add echo-tools/src/registry.rs
git -c commit.gpgsign=false commit -m "feat(tools): register_all_tools 注册 run_code

Sprint 10b Task 4: writer toolset 加 RunCodeTool(与 ShellTool 同 shell feature
gate)。readonly subset 不含 run_code(data worker 单独 add_tool,见应用层 commit)。"
```

---

## Task 5: Wire `RunCodeTool` onto data-worker agents + update prompts

**Files:**
- Modify: `echo-agent-cli/echo-agent-app-core/src/infra.rs` (data-worker registration in `register_default_subagents` loop, ~line 443-515)
- Modify: `echo-agent-cli/echo-agent-app-core/src/subagents/data/data-shaper.md`
- Modify: `echo-agent-cli/echo-agent-app-core/src/subagents/data/analyst.md`

**Context (verified):** data-shaper.md / analyst.md have `workspace: true` but NO `readonly:` field → `readonly` defaults to `false` → they go through `build_writer_worker_agent` (full toolset, **already includes `run_code` after Task 4** via `register_all_tools`). So strictly, `run_code` is already available to them after Task 4. **However**, for explicitness and to inject the app's configured `SandboxExecutor` (so the tool uses the real sandbox, not the bare fallback), we explicitly `add_tool` on data-worker handles. Verify this assumption first in Step 1.

- [ ] **Step 1: Verify data-worker toolset after Task 4**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
grep -n "readonly" echo-agent-app-core/src/subagents/data/data-shaper.md echo-agent-app-core/src/subagents/data/analyst.md
```
Expected: no `readonly` line in either (confirms `readonly: false` default → writer path → full toolset incl. `run_code`). If either DOES have `readonly: true`, the plan needs adjusting (would need explicit `add_tool`). Document the finding.

- [ ] **Step 2: Check whether the app has a shared `SandboxExecutor` to inject**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
grep -rn "SandboxExecutor\|SandboxManager\|sandbox_manager\|Arc<dyn Sandbox" echo-agent-app-core/src/ | grep -v "test\|//\|/\*" | head -10
```
If the app already constructs a `SandboxManager`/`SandboxExecutor` (e.g. for hooks), note its location — we want to inject the SAME instance into `RunCodeTool`. If none exists app-side, the tool runs with `sandbox: None` (warn + bare fallback) — acceptable per D-10b-RCE-1, document it.

- [ ] **Step 3: Decide wiring based on Steps 1-2 findings**

- **If data-workers are writer-path (Step 1 confirms no `readonly`)** AND **app has no shared sandbox to inject (Step 2 finds nothing)**: `run_code` is ALREADY registered on data-workers via `register_all_tools`. No `infra.rs` change needed for registration — skip to Step 6 (prompt updates only). The tool runs bare (warn). Document this in the commit.
- **If app HAS a shared `SandboxExecutor`**: add `worker.add_tool(Box::new(RunCodeTool::new().with_sandbox(sandbox.clone())))` on data-worker handles in the `register_default_subagents` loop (gate on `worker_def.isolate_workspace` to target data workers specifically).
- **If data-workers are actually readonly**: must `add_tool` explicitly (readonly subset excludes run_code).

Write the decision + reasoning into the commit message regardless of which path.

- [ ] **Step 4 (conditional, only if Step 3 says inject): Modify `infra.rs`**

In `echo-agent-cli/echo-agent-app-core/src/infra.rs`, inside `register_default_subagents`, after the worker is built and registered (around line 477-506), add for data workers (those with `isolate_workspace`):

```rust
            // Sprint 10b: data workers (workspace:true) get run_code with the
            // app's configured sandbox (if any) so they can run arbitrary
            // Python/R scripts in their tmpdir workspace.
            if worker_def.isolate_workspace {
                let mut run_code = echo_tools::code::RunCodeTool::new();
                if let Some(ref sandbox) = sandbox_executor {
                    run_code = run_code.with_sandbox(sandbox.clone());
                }
                // Note: add_tool on the handle requires access to the underlying
                // agent; if AgentHandle doesn't expose add_tool, register via
                // the builder instead (build_data_worker_agent path).
                // ... (adjust to actual API)
            }
```

NOTE: the exact mechanism depends on whether `AgentHandle` exposes `add_tool` or whether it must be done at builder time. If `worker_handle` doesn't allow post-construction `add_tool`, move the registration into a new `build_data_worker_agent` function (mirroring `build_writer_worker_agent`) that calls `register_all_tools` AND `add_tool(RunCodeTool)`. Prefer the simplest path that compiles. The goal: a data worker's agent ends up with `run_code` available + (if possible) the app's sandbox injected.

- [ ] **Step 5: Update the data-worker prompts**

Edit `echo-agent-cli/echo-agent-app-core/src/subagents/data/data-shaper.md` — add a "工具" section to the body (after the existing role description):

```markdown
## 可用工具

除了 Polars 数据工具(read_data/filter_data/transform_data/export_data 等),你现在可以用 **`run_code`** 工具运行任意 Python/R 脚本。

**重要:`run_code` 跑的代码会自动在当前任务的临时隔离目录(`working_dir`)中执行。** 无需 `os.makedirs("/tmp/...")`,直接读写当前目录下的文件即可。产出文件命名带 worker id(如 `run_001_clean.parquet`)。
```

Do the equivalent for `echo-agent-cli/echo-agent-app-core/src/subagents/data/analyst.md` (replace "ETL/清洗" wording with "统计/出图/建模").

- [ ] **Step 6: Verify the full app builds + existing subagent tests pass**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
cargo check --workspace
cargo test --workspace
cargo check --no-default-features --features gui --bin echo-agent-tauri
```
Expected: all PASS (existing subagent_loader tests must still pass — prompt body changes don't affect frontmatter parsing).

- [ ] **Step 7: fmt + clippy**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

- [ ] **Step 8: Commit**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
git add echo-agent-app-core/src/infra.rs \
        echo-agent-app-core/src/subagents/data/data-shaper.md \
        echo-agent-app-core/src/subagents/data/analyst.md
git -c commit.gpgsign=false commit -m "feat(app): 数据 worker 接入 run_code + 更新提示词

Sprint 10b Task 5: data-shaper/analyst 经 register_all_tools 自动获得
run_code(writer toolset,因 frontmatter 无 readonly → writer 路径)。
若 app 有共享 SandboxExecutor 则显式注入(用真沙箱,非 bare 回退)。

提示词更新:告知 LLM run_code 可用 + 强调 working_dir 语义(代码自动在
worker tmpdir 跑,无需 os.makedirs,防 LLM 瞎写绝对路径脱节)。"
```

---

## Task 6: Full verification (both repos) + cargo clean + final commit if any

**This task has NO new code — it's the AGENTS.md mandatory pre-commit verification gate.**

- [ ] **Step 1: echo-agent full verification (per-crate, mandatory)**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent
./scripts/verify-all-crates.sh
```
This runs fmt + per-crate test (all 8 crates: echo_core/echo_macros/echo_execution/echo_tools/echo_state/echo_orchestration/echo_integration/echo_agent) + clippy + feature matrix. Expected: ALL PASS, exit 0. If any sub-crate fails, fix before proceeding (AGENTS.md: no skipping).

- [ ] **Step 2: echo-agent-cli full verification**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo clippy --all-targets -- -D warnings
```
Expected: ALL PASS.

- [ ] **Step 3: Frontend (if touched — it wasn't, but verify the build still passes)**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend
npx tsc -b && npm run build
```
Expected: PASS (no frontend changes, but AGENTS.md requires the gate).

- [ ] **Step 4: cargo clean BOTH repos (mandatory, AGENTS.md)**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent && cargo clean
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli && cargo clean
```

- [ ] **Step 5: Update MASTER-PLAN + deep-iteration-plan**

Edit `docs/MASTER-PLAN.md`:
- Update the "最后更新" date + add `Sprint 10b` commit hashes (fill after commits).
- §三 add a new ✅ Sprint 10b entry (one paragraph: what shipped, file:line refs, decisions D-10b-RCE-1/case-1/docker-r-1/stdin-1).
- §五 mark Sprint 10b ✅, leave Sprint 11 ⏳待做 with note "spec 在 docs/superpowers/specs/2026-07-01-sprint-10b-and-11-design.md §三,新窗口执行".

Edit `docs/deep-iteration-plan.md`:
- §七 status table: Sprint 10b → ✅ + commit hash; Sprint 11 → ⏳待做(spec)指向新 spec 文件.

- [ ] **Step 6: Commit the doc updates**

```bash
cd /Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs
# docs/ is part of which repo? Check: it's likely a standalone or part of lp-agent root (not a git repo per AGENTS.md).
# If docs/ is NOT in any git repo, skip git commit; just save the files.
```
NOTE: `docs/` is at the lp-agent root which AGENTS.md says is "not a git repository" (the three sub-repos are echo-agent / echo-agent-cli / echo-website). So docs updates are just file saves, no commit. The two code commits (Tasks 1-4 in echo-agent, Task 5 in echo-agent-cli) are the actual git commits.

- [ ] **Step 7: Final confirmation**

Confirm:
- 2 git commits exist (1 in echo-agent covering Tasks 1-4 as separate commits or squashed per preference; 1 in echo-agent-cli for Task 5).
- All verification gates green.
- `cargo clean` ran on both repos.
- MASTER-PLAN + deep-iteration-plan updated.

**Done. Sprint 10b shipped. Sprint 11 deferred to a fresh context window (read `docs/superpowers/specs/2026-07-01-sprint-10b-and-11-design.md` §三 to resume).**

---

## Self-Review Notes (for the implementer)

**Spec coverage check (spec §二 → tasks):**
- Decision (a) RCE warn-not-deny → Task 3 Step 4 (the `else` branch with `tracing::warn!`).
- Decision (b) R native Code backend → Tasks 1 (local) + 2 (docker).
- Decision (c) arg-based consistency → Task 3 (uses `SandboxCommand::code`, no stdin).
- Decision (d) case-insensitive `validate_language` → Task 3 Step 1 (`validate_language` + `.to_lowercase()`).
- Decision (e) docker R blind-trust image → Task 2 (no tool-layer probing).
- Registration in `register_all_tools` → Task 4.
- Data worker wiring → Task 5.
- Prompt updates (user review patch #2) → Task 5 Step 5.
- Working_dir propagation → Task 3 Step 6 (integration test).
- Verification + cargo clean (AGENTS.md) → Task 6.

**Type consistency (verified during plan self-review):**
- `RunCodeTool::new()` / `.with_sandbox(Arc<dyn SandboxExecutor>)` used consistently in Tasks 3, 4, 5.
- `validate_language(&str) -> ToolResult<String>` signature consistent; uses `ToolError::InvalidParameter { name, message }` struct variant (NOT `InvalidParameter(String)`).
- `SandboxExecutor` trait methods (`name`, `isolation_level`, `is_available`, `execute`) match `echo-core/src/sandbox.rs:18`.
- `SandboxCommand.with_timeout(Duration)` — takes Duration, NOT seconds (verified `sandbox.rs:148`).
- `SandboxCommand.env` is `HashMap<String,String>` (NOT Vec) — verified `sandbox.rs:82`.
- `ToolParameters = HashMap<String, serde_json::Value>` (`echo-core:266`); `.get("x").and_then(|v| v.as_str())` is the idiomatic access (matches shell.rs:380, files/edit.rs).
- `IsolationLevel::None` exists (`sandbox.rs:52`).

**Known unknowns to resolve during implementation (only these remain):**
- Whether app has a shared `SandboxExecutor` to inject into RunCodeTool (Task 5 Step 2 — grep determines this; if none, tool runs bare with warn, which is acceptable).
- `AgentHandle::add_tool` API shape IF Step 2 finds a sandbox to inject (Task 5 Step 4 — may need builder-time registration via a new `build_data_worker_agent` if handle doesn't expose add_tool).
