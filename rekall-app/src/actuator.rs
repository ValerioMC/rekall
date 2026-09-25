//! The actuator endpoints something outside the console relies on: `/actuator/health`, which the
//! macOS launcher and the console poll to know the server is (back) up, with the liveness and
//! readiness probes `management.endpoint.health.probes.enabled` added, and `/actuator/info`.
//! `/actuator/metrics` was JVM-specific (heap, GC, threads) and is not served.

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{json, Value};

const ACTUATOR_JSON: &str = "application/vnd.spring-boot.actuator.v3+json";

pub fn routes(conn: DatabaseConnection) -> Router {
    Router::new()
        .route("/actuator", get(links))
        .route("/actuator/health", get(health))
        .route("/actuator/health/liveness", get(liveness))
        .route("/actuator/health/readiness", get(readiness))
        .route("/actuator/info", get(info))
        .with_state(conn)
}

fn actuator(status: StatusCode, body: Value) -> Response {
    let mut response = (status, body.to_string()).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(ACTUATOR_JSON));
    response
}

/// `DataSourceHealthIndicator`: the database answers a query, or the application is DOWN (503).
async fn health(State(conn): State<DatabaseConnection>) -> Response {
    let up = conn.execute_unprepared("SELECT 1").await.is_ok();
    let status = if up { "UP" } else { "DOWN" };
    let code = if up { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    actuator(code, json!({ "groups": ["liveness", "readiness"], "status": status }))
}

async fn liveness() -> Response {
    actuator(StatusCode::OK, json!({ "status": "UP" }))
}

async fn readiness() -> Response {
    actuator(StatusCode::OK, json!({ "status": "UP" }))
}

async fn info() -> Response {
    actuator(StatusCode::OK, json!({}))
}

async fn links(request: axum::extract::Request) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
        .to_string();
    let base = format!("http://{host}/actuator");
    let link = |href: String, templated: bool| json!({ "href": href, "templated": templated });
    actuator(
        StatusCode::OK,
        json!({
            "_links": {
                "self": link(base.clone(), false),
                "health": link(format!("{base}/health"), false),
                "health-path": link(format!("{base}/health/{{*path}}"), true),
                "info": link(format!("{base}/info"), false),
            }
        }),
    )
}
