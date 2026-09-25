//! `McpController`: `POST /mcp`. Two protocol eras meet on this one endpoint, and serving a request
//! under the wrong one fails silently on the client, so the era is decided first and every answer
//! is shaped for it.
//!
//! | | Handshake era | Stateless era |
//! |---|---|---|
//! | Opens with | `initialize`, answered with the revision asked for | nothing |
//! | Unknown method | `-32601` on a 200 | `-32601` on a 404 |
//! | Unknown revision | - | `-32022` on a 400, listing what is supported |
//! | Result envelope | a result is a result | every result carries `resultType` |
//! | `tools/list` | the tools | the tools plus `ttlMs` and `cacheScope` |

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use base64::Engine;
use rekall_common::RekallError;
use serde_json::{json, Map, Value};
use tracing::{debug, error, info};

use crate::protocol::{
    json_rpc_text, McpTool, ProtocolVersion, Request, Response as RpcResponse, ToolError, HEADER_MISMATCH, INTERNAL_ERROR,
    INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, UNSUPPORTED_PROTOCOL_VERSION, VERSION,
};
use crate::texts;

const SERVER_NAME: &str = "rekall";
const SERVER_VERSION: &str = "0.1.0";
const CACHE_TTL_MS: i64 = 3_600_000;
const CACHE_SCOPE: &str = "public";
const BASE64_PREFIX: &str = "=?base64?";
const BASE64_SUFFIX: &str = "?=";

/// What a call came to: a status and a JSON-RPC body, or an accepted notification with none.
#[derive(Debug)]
pub enum Answer {
    Body(StatusCode, RpcResponse),
    Accepted,
}

impl Answer {
    fn ok(response: RpcResponse) -> Self {
        Self::Body(StatusCode::OK, response)
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Body(status, _) => *status,
            Self::Accepted => StatusCode::ACCEPTED,
        }
    }

    pub fn body(&self) -> Option<&RpcResponse> {
        match self {
            Self::Body(_, body) => Some(body),
            Self::Accepted => None,
        }
    }
}

impl IntoResponse for Answer {
    fn into_response(self) -> Response {
        match self {
            Self::Accepted => StatusCode::ACCEPTED.into_response(),
            Self::Body(status, body) => {
                let mut response = (status, serde_json::to_string(&body).unwrap_or_default()).into_response();
                response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
                response
            }
        }
    }
}

/// A method's handler failed: `IllegalArgumentException` became `-32602`, anything else `-32603`.
enum Failure {
    Illegal(String),
    Other(RekallError),
}

impl From<RekallError> for Failure {
    fn from(error: RekallError) -> Self {
        match error {
            RekallError::IllegalArgument(message) => Self::Illegal(message),
            other => Self::Other(other),
        }
    }
}

#[derive(Clone)]
pub struct McpController {
    tools: Vec<Arc<dyn McpTool>>,
    by_name: HashMap<&'static str, Arc<dyn McpTool>>,
}

impl McpController {
    pub fn new(tools: Vec<Arc<dyn McpTool>>) -> Self {
        let mut by_name = HashMap::new();
        let mut kept = Vec::new();
        for tool in tools {
            // First registration wins, as `Collectors.toMap(..., (a, b) -> a, LinkedHashMap::new)` did.
            if !by_name.contains_key(tool.name()) {
                by_name.insert(tool.name(), tool.clone());
                kept.push(tool);
            }
        }
        let names: Vec<&str> = kept.iter().map(|t| t.name()).collect();
        let writing: Vec<&str> = kept.iter().filter(|t| t.writes()).map(|t| t.name()).collect();
        info!(
            "MCP server exposing {} tool(s): [{}]. Writes: {}",
            kept.len(),
            names.join(", "),
            if writing.is_empty() { "none".to_string() } else { format!("[{}]", writing.join(", ")) }
        );
        Self { tools: kept, by_name }
    }

    /// The whole dispatch, for a request already read from its body and headers.
    pub async fn handle(
        &self,
        version_header: Option<&str>,
        method_header: Option<&str>,
        name_header: Option<&str>,
        request: &Request,
    ) -> Answer {
        if request.jsonrpc.as_deref() != Some(VERSION) {
            return Answer::ok(RpcResponse::failure(request.response_id(), INVALID_REQUEST, "Expected jsonrpc 2.0"));
        }
        if request.method.as_deref().is_none_or(rekall_common::jstr::is_blank) {
            return Answer::ok(RpcResponse::failure(request.response_id(), INVALID_REQUEST, "Request has no method"));
        }

        let declared = version_header.map(str::to_string).or_else(|| meta_protocol_version(request));
        let version = match &declared {
            None => ProtocolVersion::ASSUMED_WHEN_HEADER_ABSENT,
            Some(declared) => match ProtocolVersion::parse(Some(declared)) {
                Some(version) => version,
                None => return unsupported_version(request, declared),
            },
        };
        if version.is_modern() {
            self.modern(request, version_header, method_header, name_header).await
        } else {
            self.legacy(request).await
        }
    }

    async fn modern(
        &self,
        request: &Request,
        version_header: Option<&str>,
        method_header: Option<&str>,
        name_header: Option<&str>,
    ) -> Answer {
        let Some(version_header) = version_header else {
            return header_mismatch(request, "MCP-Protocol-Version header is required from 2026-07-28 on");
        };
        if let Some(meta_version) = meta_protocol_version(request) {
            if version_header != meta_version {
                return header_mismatch(
                    request,
                    &format!("MCP-Protocol-Version '{version_header}' does not match _meta '{meta_version}'"),
                );
            }
        }
        let method = request.method.clone().unwrap_or_default();
        if request.is_notification() {
            debug!("MCP notification: {method}");
            return Answer::Accepted;
        }
        if method_header != Some(method.as_str()) {
            return header_mismatch(
                request,
                &format!("Mcp-Method '{}' does not match body method '{method}'", method_header.unwrap_or("null")),
            );
        }
        if method == "tools/call" {
            let body_name = json_rpc_text(request.params.as_ref(), "name");
            let header_name = match decode_header_value(name_header) {
                Ok(decoded) => decoded,
                Err(()) => return header_mismatch(request, "Mcp-Name is not valid base64"),
            };
            if header_name.is_none() || header_name != body_name {
                return header_mismatch(
                    request,
                    &format!(
                        "Mcp-Name '{}' does not match body name '{}'",
                        name_header.unwrap_or("null"),
                        body_name.as_deref().unwrap_or("null")
                    ),
                );
            }
        }
        let id = request.response_id();
        let answer: Result<Answer, Failure> = async {
            Ok(match method.as_str() {
                "server/discover" => Answer::ok(RpcResponse::success(id.clone(), complete(cacheable(discover())))),
                "tools/list" => Answer::ok(RpcResponse::success(id.clone(), complete(cacheable(self.tool_list())))),
                "tools/call" => {
                    Answer::ok(RpcResponse::success(id.clone(), complete(self.call_tool(request.params.as_ref()).await?)))
                }
                "ping" => Answer::ok(RpcResponse::success(id.clone(), complete(Map::new()))),
                _ => Answer::Body(
                    StatusCode::NOT_FOUND,
                    RpcResponse::failure(id.clone(), METHOD_NOT_FOUND, format!("Unsupported method: {method}")),
                ),
            })
        }
        .await;
        guarded(request, answer)
    }

    async fn legacy(&self, request: &Request) -> Answer {
        let method = request.method.clone().unwrap_or_default();
        if request.is_notification() {
            debug!("MCP notification: {method}");
            return Answer::Accepted;
        }
        let id = request.response_id();
        let answer: Result<Answer, Failure> = async {
            Ok(Answer::ok(match method.as_str() {
                "initialize" => RpcResponse::success(id.clone(), Value::Object(initialize(request))),
                "server/discover" => RpcResponse::success(id.clone(), complete(cacheable(discover()))),
                "tools/list" => RpcResponse::success(id.clone(), Value::Object(self.tool_list())),
                "tools/call" => RpcResponse::success(id.clone(), Value::Object(self.call_tool(request.params.as_ref()).await?)),
                "ping" => RpcResponse::success(id.clone(), json!({})),
                _ => RpcResponse::failure(id.clone(), METHOD_NOT_FOUND, format!("Unsupported method: {method}")),
            }))
        }
        .await;
        guarded(request, answer)
    }

    fn tool_list(&self) -> Map<String, Value> {
        let descriptors: Vec<Value> = self
            .tools
            .iter()
            .map(|tool| json!({ "name": tool.name(), "description": tool.description(), "inputSchema": tool.input_schema() }))
            .collect();
        let mut result = Map::new();
        result.insert("tools".into(), Value::Array(descriptors));
        result
    }

    async fn call_tool(&self, params: Option<&Value>) -> Result<Map<String, Value>, Failure> {
        let Some(name) = json_rpc_text(params, "name") else {
            return Err(Failure::Illegal("tools/call needs a 'name'".into()));
        };
        let Some(tool) = self.by_name.get(name.as_str()) else {
            let available: Vec<&str> = self.tools.iter().map(|t| t.name()).collect();
            return Err(Failure::Illegal(format!("Unknown tool '{name}'. Available: {}", available.join(", "))));
        };
        let arguments = params.and_then(|p| p.get("arguments"));
        match tool.execute(arguments).await {
            Ok(text) => Ok(content(&text, false)),
            Err(ToolError::Failure(message)) | Err(ToolError::Illegal(message)) => Ok(content(&message, true)),
            Err(ToolError::Other(error)) => Err(Failure::Other(error)),
        }
    }
}

fn initialize(request: &Request) -> Map<String, Value> {
    let asked = json_rpc_text(request.params.as_ref(), "protocolVersion");
    let answer = ProtocolVersion::parse(asked.as_deref())
        .filter(|version| !version.is_modern())
        .unwrap_or_else(ProtocolVersion::latest_legacy);
    let mut result = Map::new();
    result.insert("protocolVersion".into(), json!(answer.wire()));
    result.insert("capabilities".into(), json!({ "tools": {} }));
    result.insert("serverInfo".into(), json!({ "name": SERVER_NAME, "version": SERVER_VERSION }));
    result
}

fn discover() -> Map<String, Value> {
    let mut result = Map::new();
    result.insert("supportedVersions".into(), json!(ProtocolVersion::advertised_versions()));
    result.insert("capabilities".into(), json!({ "tools": {} }));
    result.insert(
        "_meta".into(),
        json!({ "io.modelcontextprotocol/serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION } }),
    );
    result.insert("instructions".into(), json!(texts::INSTRUCTIONS));
    result
}

/// Every stateless-era result says which kind of result it is, or the client throws it away.
fn complete(mut result: Map<String, Value>) -> Value {
    result.insert("resultType".into(), json!("complete"));
    Value::Object(result)
}

/// The cache annotations without which a stateless-era `tools/list` is rejected whole.
fn cacheable(mut result: Map<String, Value>) -> Map<String, Value> {
    result.insert("ttlMs".into(), json!(CACHE_TTL_MS));
    result.insert("cacheScope".into(), json!(CACHE_SCOPE));
    result
}

fn content(text: &str, is_error: bool) -> Map<String, Value> {
    let mut result = Map::new();
    result.insert("content".into(), json!([{ "type": "text", "text": text }]));
    result.insert("isError".into(), json!(is_error));
    result
}

fn guarded(request: &Request, answer: Result<Answer, Failure>) -> Answer {
    match answer {
        Ok(answer) => answer,
        Err(Failure::Illegal(message)) => Answer::ok(RpcResponse::failure(request.response_id(), INVALID_PARAMS, message)),
        Err(Failure::Other(failure)) => {
            let text = java_exception_text(&failure);
            error!("MCP call {} failed: {text}", request.method.as_deref().unwrap_or(""));
            Answer::ok(RpcResponse::failure(request.response_id(), INTERNAL_ERROR, text))
        }
    }
}

/// `e.getClass().getSimpleName() + ": " + e.getMessage()` for the exception each error was.
fn java_exception_text(error: &RekallError) -> String {
    match error {
        RekallError::Internal(text) => text.clone(),
        RekallError::NotFound(m) => format!("NotFoundException: {m}"),
        RekallError::Conflict(m) => format!("ConflictException: {m}"),
        RekallError::UnknownAnchor(m) => format!("UnknownAnchorException: {m}"),
        RekallError::AmbiguousAnchor { message, .. } => format!("AmbiguousAnchorException: {message}"),
        RekallError::Integrity(m) => format!("DataIntegrityViolationException: {m}"),
        RekallError::IllegalArgument(m) => format!("IllegalArgumentException: {m}"),
    }
}

fn unsupported_version(request: &Request, declared: &str) -> Answer {
    let mut data = Map::new();
    data.insert("supported".into(), json!(ProtocolVersion::advertised_versions()));
    data.insert("requested".into(), json!(declared));
    Answer::Body(
        StatusCode::BAD_REQUEST,
        RpcResponse::failure_with(request.response_id(), UNSUPPORTED_PROTOCOL_VERSION, "Unsupported protocol version", data),
    )
}

fn header_mismatch(request: &Request, detail: &str) -> Answer {
    Answer::Body(
        StatusCode::BAD_REQUEST,
        RpcResponse::failure(request.response_id(), HEADER_MISMATCH, format!("Header mismatch: {detail}")),
    )
}

/// `=?base64?...?=` is decoded; anything else is taken as it is.
fn decode_header_value(value: Option<&str>) -> Result<Option<String>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.len() < BASE64_PREFIX.len() + BASE64_SUFFIX.len()
        || !value.starts_with(BASE64_PREFIX)
        || !value.ends_with(BASE64_SUFFIX)
    {
        return Ok(Some(value.to_string()));
    }
    let encoded = &value[BASE64_PREFIX.len()..value.len() - BASE64_SUFFIX.len()];
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_| ())?;
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

fn meta_protocol_version(request: &Request) -> Option<String> {
    let meta = request.params.as_ref()?.get("_meta").filter(|m| !m.is_null())?;
    json_rpc_text(Some(meta), "io.modelcontextprotocol/protocolVersion")
}

// ---------------------------------------------------------------------------------- HTTP

/// `POST /mcp`.
pub fn router(controller: McpController) -> Router {
    Router::new().route("/mcp", post(handle_http)).with_state(Arc::new(controller))
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

async fn handle_http(State(controller): State<Arc<McpController>>, headers: HeaderMap, body: Bytes) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value @ Value::Object(_)) => value,
        Ok(_) => return unreadable("Cannot deserialize value of type `dev.rekall.mcp.protocol.JsonRpc$Request`"),
        Err(e) if body.iter().all(u8::is_ascii_whitespace) => {
            let _ = e;
            return unreadable("Required request body is missing");
        }
        Err(e) => return unreadable(&e.to_string()),
    };
    let request = Request::from_value(&value);
    controller
        .handle(
            header(&headers, "MCP-Protocol-Version"),
            header(&headers, "Mcp-Method"),
            header(&headers, "Mcp-Name"),
            &request,
        )
        .await
        .into_response()
}

/// A body that is not a JSON object is the 400 problem `HttpMessageNotReadableException` gave.
fn unreadable(detail: &str) -> Response {
    let body = json!({
        "detail": detail,
        "instance": "/mcp",
        "status": 400,
        "title": "Bad Request",
    });
    let mut response = (StatusCode::BAD_REQUEST, body.to_string()).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
    response
}
