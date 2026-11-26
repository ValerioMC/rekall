use rekall_kernel::Id;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TaskChangesScope {
    All,
    Repository {
        #[serde(deserialize_with = "Deserialize::deserialize")]
        component_id: Option<Id>,
    },
    Step {
        step_id: Id,
    },
    Commit {
        reference_id: Id,
    },
    WorkingTree {
        #[serde(deserialize_with = "Deserialize::deserialize")]
        component_id: Option<Id>,
    },
}

impl TaskChangesScope {
    pub(in crate::commit) fn includes_commit(self, repository: Option<Id>, reference: Id, step: Option<Id>) -> bool {
        match self {
            Self::All => true,
            Self::Repository { component_id } => component_id == repository,
            Self::Step { step_id } => Some(step_id) == step,
            Self::Commit { reference_id } => reference_id == reference,
            Self::WorkingTree { .. } => false,
        }
    }

    pub(in crate::commit) fn includes_working_tree(self, repository: Option<Id>) -> bool {
        match self {
            Self::All => true,
            Self::Repository { component_id } | Self::WorkingTree { component_id } => component_id == repository,
            Self::Step { .. } | Self::Commit { .. } => false,
        }
    }
}
