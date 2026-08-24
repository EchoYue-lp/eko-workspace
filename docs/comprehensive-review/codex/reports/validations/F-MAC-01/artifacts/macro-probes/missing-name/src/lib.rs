use echo_agent::{error::Result, tool, tools::ToolResult};

#[tool(description = "Missing required name")]
async fn missing(value: String) -> Result<ToolResult> {
    Ok(ToolResult::success(value))
}
