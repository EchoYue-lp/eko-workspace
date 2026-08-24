use echo_core::{
    error::Result,
    tools::{Tool, ToolParameters, ToolResult, ToolRunner},
};

#[derive(echo_macros::Tool)]
#[tool(name = "unit", description = "Unit tool validation probe")]
struct Unit;

impl ToolRunner<UnitParams> for Unit {
    async fn run(&self, _params: UnitParams) -> Result<ToolResult> {
        Ok(ToolResult::success("ok"))
    }
}

fn main() {
    let mut unexpected = ToolParameters::new();
    unexpected.insert("unexpected".to_string(), serde_json::json!(1));
    let accepted = futures::executor::block_on(Unit.validate_parameters(&unexpected)).is_ok();
    let schema = Unit.parameters();
    assert!(accepted);
    assert_eq!(schema["properties"], serde_json::json!({}));
    println!("unexpected_parameter_accepted={accepted} schema={schema}");
}
