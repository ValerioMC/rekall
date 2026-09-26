use async_trait::async_trait;
use serde_json::Value;

use super::ToolError;

#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn input_schema(&self) -> Value;

    /// Declared rather than inferred, so the startup log names the write surface out loud.
    fn writes(&self) -> bool {
        false
    }

    async fn execute(&self, arguments: Option<&Value>) -> Result<String, ToolError>;
}
