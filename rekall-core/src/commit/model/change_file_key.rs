use rekall_kernel::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangeFileKey {
    #[serde(deserialize_with = "Deserialize::deserialize")]
    pub component_id: Option<Id>,
    pub path: String,
}
