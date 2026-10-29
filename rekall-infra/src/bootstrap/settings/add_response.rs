use serde::Serialize;

use super::DatabaseView;

#[derive(Serialize)]
pub(super) struct AddResponse {
    pub(super) mode: &'static str,
    pub(super) entry: DatabaseView,
}
