use async_trait::async_trait;
use crate::connection::{JiraCredential, JiraSite, KubeAccess, Secret};

use super::ProbeOutcome;

/// Asks a GitLab instance, a Kubernetes cluster or a Jira site whether a credential is accepted.
#[async_trait]
pub trait ConnectionProbe: Send + Sync {
    async fn gitlab(&self, base_url: &str, token: &Secret) -> ProbeOutcome;

    async fn kubernetes(&self, access: &KubeAccess) -> ProbeOutcome;

    async fn jira(&self, site: &JiraSite, credential: &JiraCredential) -> ProbeOutcome;
}
