use echo_agent::{
    audit_logger, callback, compressor,
    compression::{CompressionInput, CompressionOutput},
    error::Result,
    guard,
    guard::{GuardDirection, GuardResult},
    handler,
    human_loop::ApprovalDecision,
    permission_policy, tool,
    tools::{ToolResult, permission::{PermissionDecision, ToolPermission}},
};

#[tool(name = "echo", description = "Valid tool attribute")]
async fn echo(value: String) -> Result<ToolResult> {
    Ok(ToolResult::success(value))
}

struct Callback;

#[callback]
impl Callback {
    async fn on_final_answer(&self, _agent: &str, _answer: &str) {}
}

#[guard(name = "valid-guard")]
async fn valid_guard(_content: &str, _direction: GuardDirection) -> Result<GuardResult> {
    Ok(GuardResult::Pass)
}

struct Handler;

#[handler]
impl Handler {
    async fn on_approval(
        &self,
        _tool_name: &str,
        _args: &serde_json::Value,
        _prompt: &str,
    ) -> ApprovalDecision {
        loop {}
    }

    async fn on_input(&self, _prompt: &str) -> String {
        loop {}
    }
}

#[compressor]
async fn valid_compressor(input: CompressionInput) -> Result<CompressionOutput> {
    let _ = input;
    loop {}
}

#[permission_policy]
async fn valid_policy(
    tool_name: &str,
    permissions: &[ToolPermission],
) -> PermissionDecision {
    let _ = (tool_name, permissions);
    loop {}
}

struct Logger;

#[audit_logger]
impl Logger {
    async fn log(&self, _event: echo_agent::audit::AuditEvent) -> Result<()> {
        loop {}
    }

    async fn query(
        &self,
        _filter: echo_agent::audit::AuditFilter,
    ) -> Result<Vec<echo_agent::audit::AuditEvent>> {
        loop {}
    }
}
