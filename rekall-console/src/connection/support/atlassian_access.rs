//! What a call to a Jira or Confluence site needs of a connection: where the site is and the
//! credential that opens it. The credential is read from the store off the async runtime.

use std::sync::Arc;

use rekall_core::connection::{AtlassianCredential, AtlassianFailure, AtlassianSite, SecretStore};
use rekall_kernel::{RekallError, Result};
use rekall_store::external_connection;

use crate::connection::support::credential_keeping::recall;

pub(in crate::connection) struct AtlassianAccess {
    pub site: AtlassianSite,
    pub credential: AtlassianCredential,
}

/// `product` words the refusals: Jira or Confluence.
pub(in crate::connection) async fn access_of(secrets: Arc<dyn SecretStore>, connection: &external_connection::Model, product: &str) -> Result<AtlassianAccess> {
    if !connection.has_credential() {
        return Err(RekallError::illegal(format!("No credential is stored for this {product} connection. Set one on the Connections page.")));
    }
    let stored = recall(secrets, connection.secret_ref.clone()).await?;
    Ok(AtlassianAccess { site: AtlassianSite::of(connection), credential: AtlassianCredential::from_stored(&stored)? })
}

/// A site's refusal is the stored credential or the site itself, which the person fixes on the Connections page.
pub(in crate::connection) fn refused_by(product: &str, failure: AtlassianFailure) -> RekallError {
    match failure {
        AtlassianFailure::Rejected { status } => RekallError::conflict(format!("{product} rejected the stored credential (HTTP {status}). Replace it on the Connections page.")),
        AtlassianFailure::Failed { reason } => RekallError::conflict(format!("{product} could not be asked: {reason}.")),
    }
}
