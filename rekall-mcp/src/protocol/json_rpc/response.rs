use serde::Serialize;
use serde_json::{Map, Value};

use super::{RpcError, VERSION};

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
