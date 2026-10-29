use std::collections::BTreeMap;

use rekall_common::{Id, Result};
use rekall_store::{project, project_dependency};
use rekall_store::repository as repo;

use crate::{in_read, load, Ctx, Tx};

use super::{manifest_freshness, rules, DeclaredDependency, InventoryStreamEvent, InventorySummaryView, InventoryWritten, ProjectDependencyView};

/// The dependencies each project declares: listed for the console and a scan, written whole by a session.
#[derive(Clone)]
pub struct DependencyInventoryService {
    ctx: Ctx,
}

impl DependencyInventoryService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<ProjectDependencyView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::project_dependency::list_of_project(tx.db(), project_id).await?;
            Ok(rows.iter().map(ProjectDependencyView::of).collect())
        })
    }

    /// Every project's inventory at a glance, with the manifests changed since it was recorded; a
    /// project without one is not listed.
    pub async fn summaries(&self) -> Result<Vec<InventorySummaryView>> {
        Ok(self.inventoried_projects().await?.iter().filter_map(|(project, rows)| summary_of(project, rows)).collect())
    }

    /// Every project that holds an inventory, with its dependencies, by project label.
    pub async fn inventoried_projects(&self) -> Result<Vec<(project::Model, Vec<project_dependency::Model>)>> {
        in_read!(&self.ctx, |tx| {
            let mut grouped: BTreeMap<Id, Vec<project_dependency::Model>> = BTreeMap::new();
            for row in repo::project_dependency::list_all(tx.db()).await? {
                grouped.entry(row.project_id).or_default().push(row);
            }
            let mut projects = repo::project::find_by_ids(tx.db(), grouped.keys().copied().collect()).await?;
            projects.sort_by(|left, right| left.label.cmp(&right.label));
            Ok(projects.into_iter().filter_map(|project| grouped.remove(&project.id).map(|rows| (project, rows))).collect())
        })
    }

    /// Replaces the inventory of the project `project:<label>` names with what a session declared.
    pub async fn write_in(&self, tx: &mut Tx, project_label: &str, declared: &[DeclaredDependency]) -> Result<InventoryWritten> {
        let project = load::resolve_project(tx.db(), project_label).await?;
        let rows = rules::dependencies(project.id, declared, self.ctx.now())?;
        let written = InventoryWritten::of(&project, &rows, declared.len());
        repo::project_dependency::replace_of_project(tx.db(), project.id, rows).await?;
        tx.publish(InventoryStreamEvent { project_id: project.id });
        Ok(written)
    }
}

fn summary_of(project: &project::Model, rows: &[project_dependency::Model]) -> Option<InventorySummaryView> {
    let summary = InventorySummaryView::of(project.id, rows)?;
    let changed_manifests = manifest_freshness::changed_since(project.repo_folder.as_deref(), rows, summary.recorded_at);
    Some(InventorySummaryView { changed_manifests, ..summary })
}
