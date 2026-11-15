//! The Claude skills a task, a step or a tag carries. Each write replaces the owner's whole list,
//! so the console sends what it shows. A name is checked for its shape only: whether the skill
//! exists under `~/.claude` is the picker's business, and a vanished one just cannot be invoked.

use std::collections::HashSet;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{skill_arguments, skill_name, tag_skill, task_skill};
use rekall_store::repository as repo;
use rekall_core::{in_read, in_write, Ctx};

use crate::skill::{SkillAssignmentsView, SkillCommand, TagSkillView, TaskSkillView};

pub const SKILLS_PER_OWNER_MAX: usize = 20;

#[derive(Clone)]
pub struct SkillAssignmentService {
    ctx: Ctx,
}

impl SkillAssignmentService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self) -> Result<SkillAssignmentsView> {
        in_read!(&self.ctx, |tx| {
            Ok::<_, RekallError>(SkillAssignmentsView {
                task_skills: repo::task_skill::list_all(tx.db()).await?.iter().map(TaskSkillView::of).collect(),
                tag_skills: repo::tag_skill::list_all(tx.db()).await?.iter().map(TagSkillView::of).collect(),
            })
        })
    }

    pub async fn set_for_task(&self, task_id: Id, skills: Vec<SkillCommand>) -> Result<Vec<TaskSkillView>> {
        in_write!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            self.replace_task_skills(&tx, task_id, None, skills).await
        })
    }

    pub async fn set_for_step(&self, step_id: Id, skills: Vec<SkillCommand>) -> Result<Vec<TaskSkillView>> {
        in_write!(&self.ctx, |tx| {
            let step = repo::task_step::require(tx.db(), step_id).await?;
            self.replace_task_skills(&tx, step.task_id, Some(step_id), skills).await
        })
    }

    pub async fn set_for_tag(&self, tag_id: Id, skills: Vec<SkillCommand>) -> Result<Vec<TagSkillView>> {
        in_write!(&self.ctx, |tx| {
            repo::tag::require(tx.db(), tag_id).await?;
            let now = self.ctx.now();
            let rows: Vec<tag_skill::Model> = distinct_skills(skills)?
                .into_iter()
                .enumerate()
                .map(|(position, skill)| tag_skill::Model {
                    id: Id::random(),
                    tag_id,
                    skill_name: skill.name,
                    arguments: skill.arguments,
                    position: position as i32,
                    created_at: now,
                })
                .collect();
            repo::tag_skill::replace(tx.db(), tag_id, rows.clone()).await?;
            Ok::<_, RekallError>(rows.iter().map(TagSkillView::of).collect())
        })
    }

    async fn replace_task_skills(
        &self,
        tx: &rekall_core::Tx,
        task_id: Id,
        step_id: Option<Id>,
        skills: Vec<SkillCommand>,
    ) -> Result<Vec<TaskSkillView>> {
        let now = self.ctx.now();
        let rows: Vec<task_skill::Model> = distinct_skills(skills)?
            .into_iter()
            .enumerate()
            .map(|(position, skill)| task_skill::Model {
                id: Id::random(),
                task_id,
                step_id,
                skill_name: skill.name,
                arguments: skill.arguments,
                position: position as i32,
                created_at: now,
            })
            .collect();
        repo::task_skill::replace(tx.db(), task_id, step_id, rows.clone()).await?;
        Ok(rows.iter().map(TaskSkillView::of).collect())
    }
}

/// The skills in the order given, each name once with its first arguments, every name valid and
/// every argument text trimmed and within its cap.
fn distinct_skills(skills: Vec<SkillCommand>) -> Result<Vec<SkillCommand>> {
    let mut seen = HashSet::new();
    let mut distinct = Vec::with_capacity(skills.len());
    for skill in skills {
        let name = skill.name.trim().to_string();
        let arguments = skill_arguments::normalize(skill.arguments.as_deref());
        skill_name::validate(&name)?;
        skill_arguments::validate(arguments.as_deref())?;
        if seen.insert(name.clone()) {
            distinct.push(SkillCommand { name, arguments });
        }
    }
    if distinct.len() > SKILLS_PER_OWNER_MAX {
        return Err(RekallError::illegal(format!("At most {SKILLS_PER_OWNER_MAX} skills can be assigned to one task, step or tag.")));
    }
    Ok(distinct)
}

#[cfg(test)]
#[path = "../../../tests/unit/skill/service/skill_assignment_service_tests.rs"]
mod tests;
