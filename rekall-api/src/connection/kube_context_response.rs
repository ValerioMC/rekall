use rekall_console::connection::KubeContextView;
use rekall_service::connection::KubeUserKind;
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

impl From<KubeContextView> for KubeContextResponse {
    fn from(view: KubeContextView) -> Self {
        Self { name: view.name, server: view.server, namespace: view.namespace, user_kind: view.user_kind, supported: view.supported, refusal: view.refusal }
    }
}
