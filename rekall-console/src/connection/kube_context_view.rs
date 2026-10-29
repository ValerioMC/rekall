use rekall_service::connection::{KubeContextSummary, KubeUserKind};

/// One context of a kubeconfig being imported, for the person to choose from.
#[derive(Clone, Debug)]
pub struct KubeContextView {
    pub name: String,
    pub server: Option<String>,
    pub namespace: Option<String>,
    pub user_kind: KubeUserKind,
    pub supported: bool,
    pub refusal: Option<String>,
}

impl KubeContextView {
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
