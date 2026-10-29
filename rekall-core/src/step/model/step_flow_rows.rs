use rekall_kernel::{Id, Result};
use rekall_store::{step_link, step_visit};
use rekall_store::repository as repo;
use sea_orm::ConnectionTrait;

/// The routes and visits a set of step views is built from, read once rather than per step.
#[derive(Clone, Debug, Default)]
pub struct StepFlowRows {
    pub links: Vec<step_link::Model>,
    pub visits: Vec<step_visit::Model>,
}

impl StepFlowRows {
    pub async fn of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Self> {
        Ok(Self {
            links: repo::step_link::list_of_task(db, task_id).await?,
            visits: repo::step_visit::list_of_task(db, task_id).await?,
        })
    }

    pub async fn of_every_task(db: &impl ConnectionTrait) -> Result<Self> {
        Ok(Self { links: repo::step_link::list_all(db).await?, visits: repo::step_visit::list_all(db).await? })
    }

    pub fn links_from(&self, step_id: Id) -> impl Iterator<Item = &step_link::Model> {
        self.links.iter().filter(move |link| link.from_step_id == step_id)
    }

    pub fn visits_of(&self, step_id: Id) -> impl Iterator<Item = &step_visit::Model> {
        self.visits.iter().filter(move |visit| visit.step_id == step_id)
    }
}
