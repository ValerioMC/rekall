use rekall_kernel::{RekallError, Result};
use rekall_core::connection::{Kubeconfig, Secret};

use crate::connection::support::credential_keeping::blocking;
use crate::connection::KubeContextView;

/// Lists the contexts of a kubeconfig the person is importing, so they can choose one. Stores
/// nothing and answers no credential.
#[derive(Clone, Default)]
pub struct KubeconfigPreviewService;

impl KubeconfigPreviewService {
    pub fn new() -> Self {
        Self
    }

    pub async fn contexts(&self, kubeconfig: Option<Secret>) -> Result<Vec<KubeContextView>> {
        let kubeconfig = kubeconfig.ok_or_else(|| RekallError::illegal("Paste or choose a kubeconfig first."))?;
        blocking(move || Ok(Kubeconfig::parse(&kubeconfig)?.contexts().into_iter().map(KubeContextView::of).collect())).await
    }
}
