use eko_framework::{error::Result, tool, tools::ToolResult};

#[tool(name = "renamed", description = "Exercise renamed facade hygiene")]
async fn renamed(value: String) -> Result<ToolResult> {
    Ok(ToolResult::success(value))
}
