use echo_agent::{error::Result, tool, tools::ToolResult};

#[tool(name = "generic_fn", description = "Generic attribute probe")]
async fn generic_fn<T>(value: T) -> Result<ToolResult>
where
    T: ToString,
{
    Ok(ToolResult::success(value.to_string()))
}
