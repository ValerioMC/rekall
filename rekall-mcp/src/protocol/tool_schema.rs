use serde_json::{json, Map, Value};

/// `ToolSchema`: an object schema of string and boolean properties, in the order they were declared.
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

    pub fn optional_boolean(mut self, name: &str, description: &str) -> Self {
        self.properties.insert(name.into(), json!({ "type": "boolean", "description": description }));
        self
    }

    pub fn build(self) -> Value {
        json!({ "type": "object", "properties": self.properties, "required": self.required })
    }
}
