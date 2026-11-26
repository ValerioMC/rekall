use async_trait::async_trait;
use rekall_kernel::Result;

use crate::guard::GuardProtection;

#[async_trait]
pub trait GuardInstaller: Send + Sync {
    async fn status(&self) -> Result<GuardProtection>;

    async fn install(&self) -> Result<GuardProtection>;

    async fn uninstall(&self) -> Result<GuardProtection>;
}
