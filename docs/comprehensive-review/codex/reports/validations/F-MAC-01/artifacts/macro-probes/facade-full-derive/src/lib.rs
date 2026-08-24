use echo_agent::{
    Tool,
    error::Result,
    tools::ToolResult,
    workspace::core::tools::ToolRunner,
};

#[derive(Tool)]
#[tool(name = "derived", description = "Full facade derive probe")]
struct Derived {
    value: String,
}

impl ToolRunner<DerivedParams> for Derived {
    async fn run(&self, params: DerivedParams) -> Result<ToolResult> {
        Ok(ToolResult::success(params.value))
    }
}
