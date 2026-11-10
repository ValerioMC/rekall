use async_trait::async_trait;
use crate::connection::{GrafanaCredential, GrafanaInstance, JiraCredential, JiraSite, KubeAccess, Secret};

use crate::connection::ProbeOutcome;

/// Asks a GitLab instance, a GitHub server, a Kubernetes cluster, a Jira site or a Grafana instance whether a credential is accepted.
#[async_trait]
pub trait ConnectionProbe: Send + Sync {
    async fn gitlab(&self, base_url: &str, token: &Secret) -> ProbeOutcome;

    async fn github(&self, server_url: &str, token: &Secret) -> ProbeOutcome;

    async fn kubernetes(&self, access: &KubeAccess) -> ProbeOutcome;

    async fn jira(&self, site: &JiraSite, credential: &JiraCredential) -> ProbeOutcome;

    async fn grafana(&self, instance: &GrafanaInstance, credential: &GrafanaCredential) -> ProbeOutcome;
}
