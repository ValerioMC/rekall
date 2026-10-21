use async_trait::async_trait;
use rekall_service::connection::{KubeAccess, Secret};

use super::ProbeOutcome;

/// Asks a GitLab instance or a Kubernetes cluster whether a credential is accepted.
#[async_trait]
pub trait ConnectionProbe: Send + Sync {
    async fn gitlab(&self, base_url: &str, token: &Secret) -> ProbeOutcome;

    async fn kubernetes(&self, access: &KubeAccess) -> ProbeOutcome;
}
