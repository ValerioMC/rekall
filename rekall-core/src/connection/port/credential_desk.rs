use async_trait::async_trait;
use rekall_kernel::Id;

use crate::connection::{CredentialPurpose, HandOut};

/// Where a terminal's helper asks for a credential. The answer is the credential in the shape
/// the helper reads, still a `Secret`, or why not; every request is logged.
#[async_trait]
pub trait CredentialDesk: Send + Sync {
    async fn hand_out(&self, terminal_id: Id, presented: Option<&str>, label: &str, purpose: CredentialPurpose) -> HandOut;
}
