use rekall_kernel::{Id, Instant, Result};
use rekall_store::commit_reference;
use rekall_store::{project_component, task_step};
use rekall_store::repository as repo;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceView {
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub step_title: Option<String>,
    pub component_id: Option<Id>,
    pub component_name: Option<String>,
    pub commit_hash: String,
    pub comment: String,
    pub in_context: bool,
    pub created_at: Instant,
}

impl CommitReferenceView {
    pub fn of(reference: &commit_reference::Model, step: Option<&task_step::Model>, component: Option<&project_component::Model>) -> Self {
        Self {
            id: reference.id,
            task_id: reference.task_id,
            step_id: step.map(|s| s.id),
            step_title: step.map(|s| s.title.clone()),
            component_id: component.map(|c| c.id),
            component_name: component.map(|c| c.name.clone()),
            commit_hash: reference.commit_hash.clone(),
            comment: reference.comment.clone(),
            in_context: reference.in_context,
            created_at: reference.created_at,
        }
    }

    pub(in crate::commit) async fn load(db: &impl sea_orm::ConnectionTrait, reference: &commit_reference::Model) -> Result<Self> {
        let step = match reference.step_id {
            Some(step_id) => repo::task_step::find_by_id(db, step_id).await?,
            None => None,
        };
        let component = match reference.component_id {
            Some(component_id) => repo::project_component::find_by_id(db, component_id).await?,
            None => None,
        };
        Ok(Self::of(reference, step.as_ref(), component.as_ref()))
    }
}
