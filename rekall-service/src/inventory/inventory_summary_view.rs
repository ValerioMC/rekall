use std::collections::BTreeMap;

use rekall_common::{Id, Instant};
use rekall_store::{project_dependency, Ecosystem};
use serde::Serialize;

/// A project's inventory at a glance: how many dependencies, per ecosystem, when it was recorded,
/// and which of the manifests it was read from changed since.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySummaryView {
    pub project_id: Id,
    pub dependency_count: usize,
    pub ecosystems: BTreeMap<&'static str, usize>,
    pub recorded_at: Instant,
    pub changed_manifests: Vec<String>,
}

impl InventorySummaryView {
    /// `rows` are one project's, and there is at least one: a project with none has no inventory.
    pub fn of(project_id: Id, rows: &[project_dependency::Model]) -> Option<Self> {
        let recorded_at = rows.iter().map(|row| row.created_at).max()?;
        let mut ecosystems = BTreeMap::new();
        for row in rows {
            *ecosystems.entry(Ecosystem::name(&row.ecosystem)).or_insert(0) += 1;
        }
        Some(Self { project_id, dependency_count: rows.len(), ecosystems, recorded_at, changed_manifests: Vec::new() })
    }
}
