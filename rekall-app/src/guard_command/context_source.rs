use std::time::Duration;

use async_trait::async_trait;
use rekall_http::guard::GuardContextResponse;

const ASK_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, thiserror::Error)]
#[error("Rekall did not answer the guard's question: {0}")]
pub struct ContextUnavailable(pub String);

#[async_trait]
pub trait GuardContextSource: Send + Sync {
    async fn context_of(&self, folder: &str) -> Result<GuardContextResponse, ContextUnavailable>;
}

/// The Rekall listening on this machine, asked once over loopback.
pub struct LocalRekall {
    base_url: String,
}

impl LocalRekall {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait]
impl GuardContextSource for LocalRekall {
    async fn context_of(&self, folder: &str) -> Result<GuardContextResponse, ContextUnavailable> {
        let unavailable = |failure: reqwest::Error| ContextUnavailable(failure.to_string());
        let client = reqwest::Client::builder().no_proxy().timeout(ASK_TIMEOUT).build().map_err(unavailable)?;
        client
            .get(format!("{}/api/guard/context", self.base_url))
            .query(&[("folder", folder)])
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(unavailable)?
            .json()
            .await
            .map_err(unavailable)
    }
}
