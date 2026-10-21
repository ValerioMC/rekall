use rekall_service::connection::{KubeContextSummary, KubeUserKind};
use serde::Serialize;

/// One context of a kubeconfig being imported, for the person to choose from.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KubeContextResponse {
    pub name: String,
    pub server: Option<String>,
    pub namespace: Option<String>,
    pub user_kind: KubeUserKind,
    pub supported: bool,
    pub refusal: Option<String>,
}

impl KubeContextResponse {
    pub fn of(summary: KubeContextSummary) -> Self {
        Self {
            supported: summary.refusal.is_none(),
            name: summary.name,
            server: summary.server,
            namespace: summary.namespace,
            user_kind: summary.user_kind,
            refusal: summary.refusal,
        }
    }
}
