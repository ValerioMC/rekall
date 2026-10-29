use rekall_console::settings::AddedDatabaseView;
use serde::Serialize;

use crate::settings::DatabaseResponse;

#[derive(Serialize)]
pub(in crate::settings) struct AddResponse {
    pub(in crate::settings) mode: &'static str,
    pub(in crate::settings) entry: DatabaseResponse,
}

impl From<AddedDatabaseView> for AddResponse {
    fn from(view: AddedDatabaseView) -> Self {
        let mode = if view.opened_existing { "opened" } else { "created" };
        Self { mode, entry: view.entry.into() }
    }
}
