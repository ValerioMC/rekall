use rekall_common::{RekallError, Result};
use rekall_service::connection::{Kubeconfig, Secret};

use super::credential_keeping::blocking;
use super::KubeContextResponse;

/// Lists the contexts of a kubeconfig the person is importing, so they can choose one. Stores
/// nothing and answers no credential.
#[derive(Clone, Default)]
pub struct KubeconfigPreviewService;

impl KubeconfigPreviewService {
    pub fn new() -> Self {
        Self
    }

    pub async fn contexts(&self, kubeconfig: Option<Secret>) -> Result<Vec<KubeContextResponse>> {
        let kubeconfig = kubeconfig.ok_or_else(|| RekallError::illegal("Paste or choose a kubeconfig first."))?;
        blocking(move || Ok(Kubeconfig::parse(&kubeconfig)?.contexts().into_iter().map(KubeContextResponse::of).collect())).await
    }
}
