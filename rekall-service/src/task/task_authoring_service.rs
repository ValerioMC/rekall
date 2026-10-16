use rekall_common::{jstr, RekallError, Result};
use rekall_model::task::Model;
use rekall_model::Slug;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::{Created, StepDraft, TaskStreamEvent};
use crate::step::{TaskStepService, PROPOSED_DRAFTS_MAX};
use crate::{in_write, load, text_rule, Ctx, Tx};

/// Creates a task for a session. The task starts `TODO` and its steps start as drafts: nothing a
/// session writes here is work until a person promotes it in the console.
#[derive(Clone)]
pub struct TaskAuthoringService {
    ctx: Ctx,
    steps: TaskStepService,
}

impl TaskAuthoringService {
    pub fn new(ctx: Ctx, steps: TaskStepService) -> Self {
        Self { ctx, steps }
    }

    /// The label defaults to the title's slug. Task and drafts land in one transaction, so a
    /// refused step leaves no half-made task behind.
    pub async fn create(
        &self,
        project_label: &str,
        title: Option<&str>,
        label: Option<&str>,
        description: Option<&str>,
        drafts: &[StepDraft],
    ) -> Result<Created> {
        in_write!(&self.ctx, |tx| {
            let project = load::resolve_project(tx.db(), project_label).await?;
            let title = required_title(title)?;
            require_drafts_fit(drafts)?;
            let mut task = Model::new(Slug::of(Some(label.unwrap_or(&title)))?, title, project.id, self.ctx.now());
            task.description = text_rule::present(description).map(str::to_string);
            task.validate()?;
            require_label_free(&tx, &task).await?;
            task.clone().into_active_model().insert(tx.db()).await?;
            tx.publish(TaskStreamEvent { task_id: task.id, project_id: project.id });
            self.add_drafts(&mut tx, &task, drafts).await?;
            Ok(Created { task_anchor: task.anchor(&project.label), title: task.title, drafts: drafts.len() })
        })
    }

    async fn add_drafts(&self, tx: &mut Tx, task: &Model, drafts: &[StepDraft]) -> Result<()> {
        for draft in drafts {
            self.steps.add_in(tx, task.id, Some(&draft.title), draft.detail.as_deref()).await?;
        }
        Ok(())
    }
}

fn required_title(title: Option<&str>) -> Result<String> {
    text_rule::present(title).map(str::to_string).ok_or_else(|| RekallError::illegal("A task needs a title."))
}

/// Same caps as proposing, checked before anything is written: the cap on drafts, and a title
/// twice in one request.
fn require_drafts_fit(drafts: &[StepDraft]) -> Result<()> {
    if drafts.len() > PROPOSED_DRAFTS_MAX {
        return Err(RekallError::illegal(format!(
            "A task holds at most {PROPOSED_DRAFTS_MAX} drafts and {} were sent. Create the task with the first \
             ones, then propose the rest.",
            drafts.len()
        )));
    }
    for (at, draft) in drafts.iter().enumerate() {
        let repeated = drafts[..at]
            .iter()
            .any(|earlier| jstr::equals_ignore_case(jstr::strip(&earlier.title), jstr::strip(&draft.title)));
        if repeated {
            return Err(RekallError::illegal(format!("The step '{}' appears twice. Nothing was created.", draft.title)));
        }
    }
    Ok(())
}

/// A label names one task within its project, or a `task:` anchor would be ambiguous.
async fn require_label_free(tx: &Tx, task: &Model) -> Result<()> {
    if repo::task::label_taken(tx.db(), task.project_id, &task.label, None).await? {
        return Err(RekallError::illegal(format!(
            "Another task on this project already uses the label '{}'. Nothing was created: pass a different `label`.",
            task.label
        )));
    }
    Ok(())
}
