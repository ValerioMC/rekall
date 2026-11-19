use rekall_kernel::Id;
use serde::Deserialize;

/// `componentId` picks a component's repository; omitted reads the project's own folder.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecentCommitsQuery {
    pub component_id: Option<Id>,
}
