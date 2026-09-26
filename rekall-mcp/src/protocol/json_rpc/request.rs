use serde_json::Value;

use super::as_text;

/// A request as Jackson bound it: every field optional, `id` and `params` any JSON.
#[derive(Clone, Debug, Default)]
pub struct Request {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub params: Option<Value>,
}

impl Request {
    /// Reads the fields the way Jackson bound `JsonRpc.Request`: a JSON `null` is an absent field,
    /// and a scalar where a string belongs is coerced to its text.
    pub fn from_value(value: &Value) -> Self {
        let field = |name: &str| value.get(name).filter(|v| !v.is_null());
        Self {
            jsonrpc: field("jsonrpc").map(as_text),
            id: field("id").cloned(),
            method: field("method").map(as_text),
            params: field("params").cloned(),
        }
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    /// The id to answer with: the request's own, or JSON null.
    pub fn response_id(&self) -> Value {
        self.id.clone().unwrap_or(Value::Null)
    }
}
