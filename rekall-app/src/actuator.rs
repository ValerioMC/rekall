//! `/actuator/health`: what the macOS launcher and the console poll to know the server is (back)
//! up. The path is kept because both callers, and installed launchers, ask for it by name.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{json, Value};

pub fn routes(conn: DatabaseConnection) -> Router {
    Router::new().route("/actuator/health", get(health)).with_state(conn)
}

/// UP while the database answers a query; DOWN, as a 503, when it does not.
async fn health(State(conn): State<DatabaseConnection>) -> (StatusCode, Json<Value>) {
    match conn.execute_unprepared("SELECT 1").await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "UP" }))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "DOWN" }))),
    }
}
