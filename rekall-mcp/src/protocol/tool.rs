use async_trait::async_trait;
use rekall_common::RekallError;
use serde_json::{json, Map, Value};

/// Why a tool call did not produce its answer.
#[derive(Debug)]
pub enum ToolError {
    /// `ToolFailure`: the session asked for something this tool will not do; it is told why.
    Failure(String),
    /// `IllegalArgumentException`: an argument broke a rule; it is told why, too.
    Illegal(String),
    /// Anything else: a JSON-RPC error.
    Other(RekallError),
}

impl From<RekallError> for ToolError {
    fn from(error: RekallError) -> Self {
        match error {
            RekallError::IllegalArgument(message) => Self::Illegal(message),
            other => Self::Other(other),
        }
    }
}

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

/// `ToolSchema`: an object schema of string properties, in the order they were declared.
#[derive(Default)]
pub struct ToolSchema {
    properties: Map<String, Value>,
    required: Vec<String>,
}

impl ToolSchema {
    pub fn object() -> Self {
        Self::default()
    }

    pub fn required_string(mut self, name: &str, description: &str) -> Self {
        self.properties.insert(name.into(), json!({ "type": "string", "description": description }));
        self.required.push(name.into());
        self
    }

    pub fn optional_string(mut self, name: &str, description: &str) -> Self {
        self.properties.insert(name.into(), json!({ "type": "string", "description": description }));
        self
    }

    pub fn build(self) -> Value {
        json!({ "type": "object", "properties": self.properties, "required": self.required })
    }
}
