use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use rekall_kernel::Id;
use serde::Deserialize;

use super::{CredentialPurpose, HandOut, Refusal};
use crate::ClaudeState;

const TOKEN_HEADER: &str = "x-rekall-credential-token";

#[derive(Deserialize)]
struct CredentialQuery {
    #[serde(rename = "for")]
    purpose: CredentialPurpose,
}

/// Outside the router's UUID check: the label in the path is a slug, not an id.
pub fn routes() -> Router<ClaudeState> {
    Router::new().route("/api/terminal/{id}/credentials/{label}", get(hand_out))
}

/// Answers in plain text, which the helpers print as they are. Only this machine is answered, even
/// when Rekall is served to others: a credential never crosses the network from here.
async fn hand_out(State(state): State<ClaudeState>, Path((id, label)): Path<(String, String)>, Query(query): Query<CredentialQuery>, request: Request) -> Response {
    let from_this_machine = request.extensions().get::<ConnectInfo<SocketAddr>>().is_some_and(|info| info.0.ip().to_canonical().is_loopback());
    if !from_this_machine {
        return refused(Refusal::Forbidden, "Credentials are only handed to this machine.");
    }
    let Ok(terminal_id) = id.parse::<Id>() else {
        return refused(Refusal::Missing, "This is not a terminal Rekall knows.");
    };
    let presented = request.headers().get(TOKEN_HEADER).and_then(|value| value.to_str().ok());
    match state.broker.hand_out(terminal_id, presented, &label, query.purpose).await {
        HandOut::Granted { body, content_type } => {
            let mut response = (StatusCode::OK, body.expose().to_string()).into_response();
            response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
        HandOut::Refused { refusal, message } => refused(refusal, &message),
    }
}

fn refused(refusal: Refusal, message: &str) -> Response {
    let status = match refusal {
        Refusal::Forbidden => StatusCode::FORBIDDEN,
        Refusal::Missing => StatusCode::NOT_FOUND,
        Refusal::WrongPurpose => StatusCode::BAD_REQUEST,
        Refusal::Failed => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, message.to_string()).into_response()
}
