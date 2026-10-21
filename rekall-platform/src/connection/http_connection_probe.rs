use std::time::Duration;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use rekall_service::connection::{KubeAccess, KubeCredential, Secret};
use serde::Deserialize;
use serde_json::json;

use super::{ConnectionProbe, ProbeOutcome};

const TIMEOUT: Duration = Duration::from_secs(8);
const SELF_SUBJECT_REVIEW: &str = "/apis/authentication.k8s.io/v1/selfsubjectreviews";

#[derive(Deserialize)]
struct GitlabUser {
    username: Option<String>,
}

#[derive(Deserialize)]
struct KubeVersion {
    #[serde(rename = "gitVersion")]
    git_version: Option<String>,
}

#[derive(Deserialize)]
struct SelfSubjectReview {
    status: Option<ReviewStatus>,
}

#[derive(Deserialize)]
struct ReviewStatus {
    #[serde(rename = "userInfo")]
    user_info: Option<UserInfo>,
}

#[derive(Deserialize)]
struct UserInfo {
    username: Option<String>,
}

/// The probe over HTTPS with `reqwest`. A cluster is trusted through the CA its kubeconfig names,
/// and a client certificate is presented as the TLS identity.
pub struct HttpConnectionProbe {
    timeout: Duration,
}

impl HttpConnectionProbe {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }
}

impl Default for HttpConnectionProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConnectionProbe for HttpConnectionProbe {
    async fn gitlab(&self, base_url: &str, token: &Secret) -> ProbeOutcome {
        let client = match Client::builder().timeout(self.timeout).build() {
            Ok(client) => client,
            Err(failure) => return unreachable(&failure),
        };
        let mut header = match HeaderValue::from_str(token.expose()) {
            Ok(header) => header,
            Err(_) => return ProbeOutcome::Unreachable { reason: "the stored token holds characters a header cannot carry".into() },
        };
        header.set_sensitive(true);
        let answer = client.get(format!("{base_url}/api/v4/user")).header("PRIVATE-TOKEN", header).send().await;
        match accepted(answer).await {
            Ok(response) => ProbeOutcome::Accepted { identity: response.json::<GitlabUser>().await.ok().and_then(|user| user.username), version: None },
            Err(outcome) => outcome,
        }
    }

    async fn kubernetes(&self, access: &KubeAccess) -> ProbeOutcome {
        let client = match kube_client(access, self.timeout) {
            Ok(client) => client,
            Err(reason) => return ProbeOutcome::Unreachable { reason },
        };
        let version = match accepted(authorised(client.get(format!("{}/version", access.server)), access).send().await).await {
            Ok(response) => response.json::<KubeVersion>().await.ok().and_then(|version| version.git_version),
            Err(outcome) => return outcome,
        };
        match identity(&client, access).await {
            Ok(identity) => ProbeOutcome::Accepted { identity, version },
            Err(outcome) => outcome,
        }
    }
}

/// Who the credential signs in as; a cluster too old to say still proves it accepts the credential through `/api`.
async fn identity(client: &Client, access: &KubeAccess) -> Result<Option<String>, ProbeOutcome> {
    let review = json!({ "apiVersion": "authentication.k8s.io/v1", "kind": "SelfSubjectReview" });
    let answer = authorised(client.post(format!("{}{SELF_SUBJECT_REVIEW}", access.server)), access)
        .header(CONTENT_TYPE, "application/json")
        .json(&review)
        .send()
        .await;
    match answer {
        Ok(response) if response.status() == StatusCode::NOT_FOUND => {
            accepted(authorised(client.get(format!("{}/api", access.server)), access).send().await).await.map(|_| None)
        }
        other => {
            let response = accepted(other).await?;
            let review = response.json::<SelfSubjectReview>().await.ok();
            Ok(review.and_then(|review| review.status).and_then(|status| status.user_info).and_then(|user| user.username))
        }
    }
}

fn kube_client(access: &KubeAccess, timeout: Duration) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(timeout);
    if let Some(authority) = &access.certificate_authority {
        let pem = decoded(authority, "certificate authority")?;
        let certificate = reqwest::Certificate::from_pem(&pem).map_err(|_| "the stored certificate authority is not a PEM certificate".to_string())?;
        builder = builder.add_root_certificate(certificate);
    }
    if let KubeCredential::ClientCertificate { certificate, key } = &access.credential {
        let mut pem = decoded(certificate.expose(), "client certificate")?;
        pem.extend(decoded(key.expose(), "client key")?);
        let identity = reqwest::Identity::from_pem(&pem).map_err(|_| "the stored client certificate and key are not usable together".to_string())?;
        builder = builder.identity(identity);
    }
    builder.build().map_err(|failure| failure.to_string())
}

fn authorised(request: RequestBuilder, access: &KubeAccess) -> RequestBuilder {
    match &access.credential {
        KubeCredential::Token(token) => request.bearer_auth(token.expose()),
        KubeCredential::ClientCertificate { .. } => request,
    }
}

fn decoded(data: &str, what: &str) -> Result<Vec<u8>, String> {
    STANDARD.decode(data.trim()).map_err(|_| format!("the stored {what} is not base64"))
}

/// The response when the server accepted the request, or the outcome that explains why it did not.
async fn accepted(answer: reqwest::Result<Response>) -> Result<Response, ProbeOutcome> {
    let response = answer.map_err(|failure| unreachable(&failure))?;
    match response.status() {
        status if status.is_success() => Ok(response),
        status @ (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => Err(ProbeOutcome::Rejected { status: status.as_u16() }),
        status => Err(ProbeOutcome::Unreachable { reason: format!("the server answered HTTP {}", status.as_u16()) }),
    }
}

fn unreachable(failure: &reqwest::Error) -> ProbeOutcome {
    ProbeOutcome::Unreachable { reason: failure.to_string() }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/http_connection_probe_tests.rs"]
mod tests;
