//! The protocol edge of the MCP server, ported from `McpProtocolTest`. Two eras meet on one
//! endpoint and serving under the wrong one fails silently, so every case asserts the era a
//! request was answered in.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use base64::Engine;
use rekall_mcp::controller::Answer;
use rekall_mcp::protocol::{McpTool, Request, ToolError, HEADER_MISMATCH, INVALID_REQUEST, METHOD_NOT_FOUND, UNSUPPORTED_PROTOCOL_VERSION};
use rekall_mcp::McpController;
use serde_json::{json, Value};

const MODERN: &str = "2026-07-28";

/// Stands in for `ContextTool`: this test is about the envelope, not what it carries.
struct StubTool;

#[async_trait]
impl McpTool for StubTool {
    fn name(&self) -> &'static str {
        "rekall_context"
    }

    fn description(&self) -> &'static str {
        "Load a working context by anchor."
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object" })
    }

    async fn execute(&self, _: Option<&Value>) -> Result<String, ToolError> {
        Ok("# Context".into())
    }
}

fn controller() -> McpController {
    McpController::new(vec![Arc::new(StubTool)])
}

fn request(id: Option<i64>, method: &str, params: Option<Value>) -> Request {
    let mut body = json!({ "jsonrpc": "2.0", "method": method });
    if let Some(id) = id {
        body["id"] = json!(id);
    }
    if let Some(params) = params {
        body["params"] = params;
    }
    Request::from_value(&body)
}

async fn call(version: Option<&str>, method: Option<&str>, name: Option<&str>, request: Request) -> Answer {
    controller().handle(version, method, name, &request).await
}

fn result(answer: &Answer) -> Value {
    let body = answer.body().expect("a body");
    assert!(body.error.is_none(), "{:?}", body.error);
    body.result.clone().unwrap()
}

fn error_code(answer: &Answer) -> i32 {
    answer.body().expect("a body").error.as_ref().expect("an error").code
}

fn tool_call() -> Request {
    request(Some(1), "tools/call", Some(json!({ "name": "rekall_context", "arguments": { "anchors": "project:vega" } })))
}

async fn initialize(requested: Option<&str>) -> Answer {
    let params = requested.map(|r| json!({ "protocolVersion": r }));
    call(None, None, None, request(Some(1), "initialize", params)).await
}

// ------------------------------------------------------------------ the handshake era

#[tokio::test]
async fn initialize_is_answered_with_the_revision_the_client_asked_for() {
    assert_eq!(result(&initialize(Some("2025-06-18")).await)["protocolVersion"], "2025-06-18");
}

#[tokio::test]
async fn a_revision_this_server_does_not_speak_is_answered_with_the_newest_handshake_one() {
    assert_eq!(result(&initialize(Some(MODERN)).await)["protocolVersion"], "2025-11-25");
    assert_eq!(result(&initialize(None).await)["protocolVersion"], "2025-11-25");
}

#[tokio::test]
async fn the_revision_this_endpoint_used_to_pin_is_still_served() {
    assert_eq!(result(&initialize(Some("2024-11-05")).await)["protocolVersion"], "2024-11-05");
}

#[tokio::test]
async fn a_request_with_no_headers_at_all_is_read_as_a_handshake_era_request() {
    let answer = call(None, None, None, request(Some(1), "tools/list", None)).await;
    assert_eq!(answer.status(), StatusCode::OK);
    assert!(result(&answer).get("tools").is_some());
}

#[tokio::test]
async fn an_unknown_method_is_a_json_rpc_error_on_a_200_as_this_era_expects() {
    let answer = call(None, None, None, request(Some(1), "resources/list", None)).await;
    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(error_code(&answer), METHOD_NOT_FOUND);
}

// ------------------------------------------------------------------ the stateless era

#[tokio::test]
async fn tools_list_is_served_when_the_headers_agree_with_the_body() {
    let answer = call(Some(MODERN), Some("tools/list"), None, request(Some(1), "tools/list", None)).await;
    assert_eq!(answer.status(), StatusCode::OK);
    assert!(result(&answer).get("tools").is_some());
}

#[tokio::test]
async fn tools_call_is_served_when_mcp_name_matches_the_tool_in_the_body() {
    let answer = call(Some(MODERN), Some("tools/call"), Some("rekall_context"), tool_call()).await;
    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(result(&answer)["isError"], false);
}

#[tokio::test]
async fn a_base64_wrapped_mcp_name_is_decoded_before_it_is_compared() {
    let wrapped = format!("=?base64?{}?=", base64::engine::general_purpose::STANDARD.encode("rekall_context"));
    let answer = call(Some(MODERN), Some("tools/call"), Some(&wrapped), tool_call()).await;
    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(result(&answer)["isError"], false);
}

#[tokio::test]
async fn the_era_is_refused_when_the_version_arrives_only_in_the_body() {
    let params = json!({ "_meta": { "io.modelcontextprotocol/protocolVersion": MODERN } });
    let answer = call(None, Some("tools/list"), None, request(Some(1), "tools/list", Some(params))).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn a_header_that_disagrees_with_meta_is_refused_rather_than_picked_between() {
    let params = json!({ "_meta": { "io.modelcontextprotocol/protocolVersion": "2025-06-18" } });
    let answer = call(Some(MODERN), Some("tools/list"), None, request(Some(1), "tools/list", Some(params))).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn mcp_method_has_to_match_the_method_in_the_body() {
    let answer = call(Some(MODERN), Some("tools/list"), None, request(Some(1), "tools/call", None)).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn a_missing_mcp_method_is_refused_not_assumed_from_the_body() {
    let answer = call(Some(MODERN), None, None, request(Some(1), "tools/list", None)).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn tools_call_without_mcp_name_is_refused() {
    let answer = call(Some(MODERN), Some("tools/call"), None, tool_call()).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn an_mcp_name_naming_a_different_tool_than_the_body_is_refused() {
    let answer = call(Some(MODERN), Some("tools/call"), Some("something_else"), tool_call()).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&answer), HEADER_MISMATCH);
}

#[tokio::test]
async fn tools_list_carries_the_cache_annotations_without_which_it_is_rejected_whole() {
    let result = result(&call(Some(MODERN), Some("tools/list"), None, request(Some(1), "tools/list", None)).await);
    assert!(result.get("tools").is_some());
    assert!(result["ttlMs"].as_i64().unwrap() >= 0);
    assert_eq!(result["cacheScope"], "public");
}

#[tokio::test]
async fn every_result_says_which_kind_of_result_it_is() {
    for (method, name, request) in [
        ("tools/list", None, request(Some(1), "tools/list", None)),
        ("tools/call", Some("rekall_context"), tool_call()),
        ("ping", None, request(Some(1), "ping", None)),
        ("server/discover", None, request(Some(1), "server/discover", None)),
    ] {
        let answer = call(Some(MODERN), Some(method), name, request).await;
        assert_eq!(result(&answer)["resultType"], "complete", "{method}");
    }
}

#[tokio::test]
async fn initialize_is_an_unknown_method_here_answered_on_a_404() {
    let answer = call(Some(MODERN), Some("initialize"), None, request(Some(1), "initialize", None)).await;
    assert_eq!(answer.status(), StatusCode::NOT_FOUND);
    assert_eq!(error_code(&answer), METHOD_NOT_FOUND);
}

#[tokio::test]
async fn an_unknown_method_is_a_404_so_a_probe_can_tell_this_from_a_wrong_address() {
    let answer = call(Some(MODERN), Some("resources/list"), None, request(Some(1), "resources/list", None)).await;
    assert_eq!(answer.status(), StatusCode::NOT_FOUND);
    assert_eq!(error_code(&answer), METHOD_NOT_FOUND);
}

// ------------------------------------------------------------------ across both eras

#[tokio::test]
async fn a_revision_this_server_does_not_speak_comes_back_with_the_ones_it_does() {
    let answer = call(Some("1900-01-01"), Some("tools/list"), None, request(Some(1), "tools/list", None)).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);
    let error = answer.body().unwrap().error.clone().unwrap();
    assert_eq!(error.code, UNSUPPORTED_PROTOCOL_VERSION);
    let data = error.data.unwrap();
    assert_eq!(data["requested"], "1900-01-01");
    let supported: Vec<&str> = data["supported"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(supported.contains(&MODERN) && supported.contains(&"2025-11-25"));
    assert!(!supported.contains(&"2024-11-05"), "never offered");
}

#[tokio::test]
async fn server_discover_answers_in_both_eras() {
    for answer in [
        call(Some(MODERN), Some("server/discover"), None, request(Some(1), "server/discover", None)).await,
        call(None, None, None, request(Some(1), "server/discover", None)).await,
    ] {
        assert_eq!(answer.status(), StatusCode::OK);
        let result = result(&answer);
        assert_eq!(result["resultType"], "complete");
        assert!(result.get("ttlMs").is_some());
        assert!(result["supportedVersions"].as_array().unwrap().contains(&json!(MODERN)));
        assert!(result["_meta"].get("io.modelcontextprotocol/serverInfo").is_some());
    }
}

#[tokio::test]
async fn a_notification_is_accepted_with_no_body_in_either_era() {
    for answer in [
        // No Mcp-Method on the first: the header is required on a request, not on a notification.
        call(Some(MODERN), None, None, request(None, "notifications/initialized", None)).await,
        call(None, None, None, request(None, "notifications/initialized", None)).await,
    ] {
        assert_eq!(answer.status(), StatusCode::ACCEPTED);
        assert!(answer.body().is_none());
    }
}

#[tokio::test]
async fn a_body_that_is_not_json_rpc_2_is_refused_before_any_of_this_runs() {
    let request = Request::from_value(&json!({ "jsonrpc": "1.0", "id": 1, "method": "tools/list" }));
    assert_eq!(error_code(&controller().handle(None, None, None, &request).await), INVALID_REQUEST);
}
