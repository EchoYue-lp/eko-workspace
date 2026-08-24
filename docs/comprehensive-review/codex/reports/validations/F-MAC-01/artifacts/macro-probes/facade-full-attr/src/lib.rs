use echo_agent::{error::Result, tool, tools::ToolResult};

#[tool(name = "echo", description = "Return the input")]
async fn echo(value: String) -> Result<ToolResult> {
    Ok(ToolResult::success(value))
}
