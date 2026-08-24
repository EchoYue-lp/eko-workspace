use echo_agent::{error::Result, tool, tools::ToolResult};

#[tool(name = "hidden", description = "Must remain cfg-disabled")]
#[cfg(any())]
async fn hidden(value: String) -> Result<ToolResult> {
    Ok(ToolResult::success(value))
}

fn main() {
    let _unexpectedly_present = HiddenTool;
    println!("cfg_disabled_generated_type_present=true");
}
