use serde::Serialize;
use serde_json::{Map, Value};

pub const VERSION: &str = "2.0";

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
pub const HEADER_MISMATCH: i32 = -32020;
pub const UNSUPPORTED_PROTOCOL_VERSION: i32 = -32022;

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

/// Jackson's `asString()`: a string as itself, a number or boolean as its text, anything else empty.
pub fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// `node.hasNonNull(field) ? node.get(field).asString() : null`.
pub fn text_at(node: Option<&Value>, field: &str) -> Option<String> {
    node.and_then(|n| n.get(field)).filter(|v| !v.is_null()).map(as_text)
}

#[derive(Clone, Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Map<String, Value>>,
}

/// `@JsonInclude(NON_NULL)`: a success carries `result`, a failure `error`; `id` is always there.
#[derive(Clone, Debug, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn success(id: Value, result: Value) -> Self {
        Self { jsonrpc: VERSION, id, result: Some(result), error: None }
    }

    pub fn failure(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self { jsonrpc: VERSION, id, result: None, error: Some(RpcError { code, message: message.into(), data: None }) }
    }

    pub fn failure_with(id: Value, code: i32, message: impl Into<String>, data: Map<String, Value>) -> Self {
        Self { jsonrpc: VERSION, id, result: None, error: Some(RpcError { code, message: message.into(), data: Some(data) }) }
    }
}
