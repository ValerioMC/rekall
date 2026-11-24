use std::collections::HashMap;

use rekall_core::{in_read, in_write, Ctx, Tx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;

use crate::workingset::rule::closing_rule::landing_after_closing;
use crate::workingset::view::working_set_view::OpenTaskView;
use crate::workingset::{PlaceView, WorkingSetView};

#[derive(Clone)]
pub struct WorkingSetService {
    ctx: Ctx,
}

impl WorkingSetService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn view(&self) -> Result<WorkingSetView> {
        in_read!(&self.ctx, |tx| view_of(&tx).await)
    }

    pub async fn open(&self, task_id: Id) -> Result<WorkingSetView> {
        in_write!(&self.ctx, |tx| {
            repo::working_set_task::open(tx.db(), task_id).await?;
            view_of(&tx).await
        })
    }

    /// Closes the task and answers where its project lands. A project that remembered the closed
    /// task remembers that landing instead.
    pub async fn close(&self, task_id: Id) -> Result<PlaceView> {
        in_write!(&self.ctx, |tx| {
            let project_id = repo::task::require(tx.db(), task_id).await?.project_id;
            let before = open_ids_of_project(&tx, project_id).await?;
            repo::working_set_task::close(tx.db(), task_id).await?;
            if !before.contains(&task_id) {
                return place_of_project(&tx, project_id).await;
            }
            let landing = PlaceView { project_id, task_id: landing_after_closing(&before, task_id) };
            if remembered_task_of(&tx, project_id).await? == Some(task_id) {
                repo::working_set_place::remember(tx.db(), project_id, landing.task_id).await?;
            }
            Ok::<_, RekallError>(landing)
        })
    }

    /// Remembers where the person was in the project. A task of another project is refused.
    pub async fn remember_place(&self, project_id: Id, task_id: Option<Id>) -> Result<PlaceView> {
        in_write!(&self.ctx, |tx| {
            if let Some(task_id) = task_id {
                require_task_of_project(&tx, task_id, project_id).await?;
            }
            repo::working_set_place::remember(tx.db(), project_id, task_id).await?;
            Ok::<_, RekallError>(PlaceView { project_id, task_id })
        })
    }
}

async fn view_of(tx: &Tx) -> Result<WorkingSetView> {
    let opened = repo::working_set_task::list(tx.db()).await?;
    let ids = opened.iter().map(|open| open.task_id).collect();
    let project_of: HashMap<Id, Id> = repo::task::find_by_ids(tx.db(), ids).await?.into_iter().map(|task| (task.id, task.project_id)).collect();
    let tasks = opened
        .iter()
        .filter_map(|open| project_of.get(&open.task_id).map(|project_id| OpenTaskView { task_id: open.task_id, project_id: *project_id }))
        .collect();
    let places = repo::working_set_place::list(tx.db()).await?.into_iter().map(|place| PlaceView { project_id: place.project_id, task_id: place.task_id }).collect();
    Ok(WorkingSetView { tasks, places })
}

async fn open_ids_of_project(tx: &Tx, project_id: Id) -> Result<Vec<Id>> {
    let view = view_of(tx).await?;
    Ok(view.tasks.into_iter().filter(|open| open.project_id == project_id).map(|open| open.task_id).collect())
}

async fn remembered_task_of(tx: &Tx, project_id: Id) -> Result<Option<Id>> {
    Ok(repo::working_set_place::find_of_project(tx.db(), project_id).await?.and_then(|place| place.task_id))
}

async fn place_of_project(tx: &Tx, project_id: Id) -> Result<PlaceView> {
    Ok(PlaceView { project_id, task_id: remembered_task_of(tx, project_id).await? })
}

async fn require_task_of_project(tx: &Tx, task_id: Id, project_id: Id) -> Result<()> {
    let task = repo::task::require(tx.db(), task_id).await?;
    if task.project_id != project_id {
        return Err(RekallError::illegal("The task belongs to another project"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/workingset/service/working_set_service_tests.rs"]
mod tests;
