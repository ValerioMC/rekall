use rekall_console::settings::AddedDatabaseView;
use serde::Serialize;

use super::DatabaseResponse;

#[derive(Serialize)]
pub(super) struct AddResponse {
    pub(super) mode: &'static str,
    pub(super) entry: DatabaseResponse,
}

impl From<AddedDatabaseView> for AddResponse {
    fn from(view: AddedDatabaseView) -> Self {
        let mode = if view.opened_existing { "opened" } else { "created" };
        Self { mode, entry: view.entry.into() }
    }
}
