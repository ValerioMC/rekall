use std::collections::HashMap;
use std::sync::Arc;

use crate::login_shell::LoginShellEnvironment;

/// Where a login shell's environment comes from: the real one, or what a test says.
#[async_trait::async_trait]
pub trait EnvironmentSource: Send + Sync {
    async fn current(&self) -> HashMap<String, String>;
}

#[async_trait::async_trait]
impl EnvironmentSource for Arc<LoginShellEnvironment> {
    async fn current(&self) -> HashMap<String, String> {
        LoginShellEnvironment::current(self).await
    }
}
