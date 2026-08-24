use echo_core::{
    error::Result,
    tools::{ToolResult, ToolRunner},
};

#[derive(echo_macros::Tool)]
#[tool(name = "generic", description = "Generic derive probe")]
pub struct Generic<T>
where
    T: Clone + Send + Sync + serde::de::DeserializeOwned + schemars::JsonSchema,
{
    pub value: T,
}

impl<T> ToolRunner<GenericParams<T>> for Generic<T>
where
    T: Clone + Send + Sync + serde::de::DeserializeOwned + schemars::JsonSchema,
{
    async fn run(&self, _params: GenericParams<T>) -> Result<ToolResult> {
        Ok(ToolResult::success("ok"))
    }
}
