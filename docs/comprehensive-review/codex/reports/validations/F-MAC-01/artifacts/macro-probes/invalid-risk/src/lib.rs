use echo_core::{
    error::Result,
    tools::{ToolResult, ToolRunner},
};

#[derive(echo_macros::Tool)]
#[tool(name = "invalid", description = "Invalid risk probe", risk_level = "Critical")]
struct InvalidRisk;

impl ToolRunner<InvalidRiskParams> for InvalidRisk {
    async fn run(&self, _params: InvalidRiskParams) -> Result<ToolResult> {
        Ok(ToolResult::success("ok"))
    }
}
