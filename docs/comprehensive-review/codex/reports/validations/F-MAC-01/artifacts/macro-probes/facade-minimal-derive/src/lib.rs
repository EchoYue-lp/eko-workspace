use echo_agent::{
    Tool,
    error::Result,
    tools::{ToolResult, ToolRunner},
};

#[derive(Tool)]
#[tool(name = "derived", description = "Facade-only derive probe")]
struct Derived {
    value: String,
}

impl ToolRunner<DerivedParams> for Derived {
    async fn run(&self, params: DerivedParams) -> Result<ToolResult> {
        Ok(ToolResult::success(params.value))
    }
}
