use rekall_service::connection::Secret;
use serde::Deserialize;

/// A kubeconfig to list the contexts of, without storing it.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KubeconfigPreviewRequest {
    pub kubeconfig: Option<Secret>,
}
