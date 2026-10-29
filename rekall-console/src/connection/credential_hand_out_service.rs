use std::sync::Arc;

use rekall_core::connection::{CredentialDesk, CredentialPurpose, HandOut};
use rekall_kernel::Id;

/// Passes a terminal helper's request to the desk and its answer back. Nothing here reads,
/// keeps or logs the credential: the desk decides and logs.
#[derive(Clone)]
pub struct CredentialHandOutService {
    desk: Arc<dyn CredentialDesk>,
}

impl CredentialHandOutService {
    pub fn new(desk: Arc<dyn CredentialDesk>) -> Self {
        Self { desk }
    }

    pub async fn hand_out(&self, terminal_id: Id, presented: Option<&str>, label: &str, purpose: CredentialPurpose) -> HandOut {
        self.desk.hand_out(terminal_id, presented, label, purpose).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/credential_hand_out_service_tests.rs"]
mod tests;
