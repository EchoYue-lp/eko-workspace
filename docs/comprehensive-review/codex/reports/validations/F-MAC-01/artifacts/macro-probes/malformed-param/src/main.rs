use echo_core::{
    error::Result,
    tools::{Tool, ToolResult, ToolRunner},
};

#[derive(echo_macros::Tool)]
#[tool(name = "malformed", description = "Malformed helper metadata probe")]
struct Malformed {
    #[tool_param(nonsense)]
    value: String,
}

impl ToolRunner<MalformedParams> for Malformed {
    async fn run(&self, params: MalformedParams) -> Result<ToolResult> {
        Ok(ToolResult::success(params.value))
    }
}

fn main() {
    let schema = Malformed {
        value: String::new(),
    }
    .parameters();
    let property = &schema["properties"]["value"];
    assert!(property.get("description").is_none());
    println!("malformed_metadata_was_silently_ignored={property}");
}
