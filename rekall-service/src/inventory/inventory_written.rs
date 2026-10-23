use std::collections::BTreeMap;

use rekall_common::Id;
use rekall_model::{project, project_dependency, Ecosystem};

/// What a session's inventory came to: the project, how many rows it holds per ecosystem, and how
/// many entries were folded away as repeats of another.
#[derive(Clone, Debug)]
pub struct InventoryWritten {
    pub project_id: Id,
    pub project_anchor: String,
    pub stored: usize,
    pub repeats: usize,
    pub by_ecosystem: BTreeMap<&'static str, usize>,
}

impl InventoryWritten {
    pub fn of(project: &project::Model, rows: &[project_dependency::Model], declared: usize) -> Self {
        let mut by_ecosystem = BTreeMap::new();
        for row in rows {
            *by_ecosystem.entry(Ecosystem::advisory_name(row.ecosystem)).or_insert(0) += 1;
        }
        Self { project_id: project.id, project_anchor: project.anchor(), stored: rows.len(), repeats: declared - rows.len(), by_ecosystem }
    }
}
