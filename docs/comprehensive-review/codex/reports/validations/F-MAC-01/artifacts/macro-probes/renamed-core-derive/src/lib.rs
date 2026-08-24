use core_kit::{
    error::Result,
    tools::{ToolResult, ToolRunner},
};

#[derive(echo_macros::Tool)]
#[tool(name = "derived", description = "Exercise renamed core hygiene")]
pub struct Derived {
    pub value: String,
}

impl ToolRunner<DerivedParams> for Derived {
    async fn run(&self, params: DerivedParams) -> Result<ToolResult> {
        Ok(ToolResult::success(params.value))
    }
}
