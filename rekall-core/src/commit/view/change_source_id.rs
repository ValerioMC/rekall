use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum ChangeSourceId {
    Commit { reference_id: Id },
    WorkingTree,
}
