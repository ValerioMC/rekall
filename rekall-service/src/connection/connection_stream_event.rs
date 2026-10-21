use rekall_common::Id;
use serde::Serialize;

/// Emitted when a company's connections, their credentials or a project's bindings change, so an
/// open console reads them again. It never carries a credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStreamEvent {
    pub company_id: Id,
}

impl crate::StreamEvent for ConnectionStreamEvent {
    const FRAME: &'static str = "connection";
}
