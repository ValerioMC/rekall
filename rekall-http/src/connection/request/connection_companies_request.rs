use rekall_kernel::Id;
use serde::Deserialize;

/// The companies a connection is shared with, replacing the ones it has.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCompaniesRequest {
    pub company_ids: Vec<Id>,
}
