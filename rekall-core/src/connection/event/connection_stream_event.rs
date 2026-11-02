use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a connection, its credential, the companies it is shared with or a project's
/// bindings change, so an open console reads them again. It names the companies concerned and
/// never carries a credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStreamEvent {
    pub company_ids: Vec<Id>,
}

impl crate::StreamEvent for ConnectionStreamEvent {
    const FRAME: &'static str = "connection";
}
