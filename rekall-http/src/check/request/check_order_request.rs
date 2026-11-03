use rekall_console::check::CheckOrderCommand;
use serde::Deserialize;

/// The checks of one scope, by id, in the order they are to run.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckOrderRequest {
    pub check_ids: Option<Vec<String>>,
}

impl From<CheckOrderRequest> for CheckOrderCommand {
    fn from(request: CheckOrderRequest) -> Self {
        Self {
            check_ids: request.check_ids,
        }
    }
}
