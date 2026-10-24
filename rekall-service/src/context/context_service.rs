//! Resolves anchors and walks the associations: up to the parent records in full with their
//! notes, down to the children as anchors only, and a task's wrapup and steps in full.

use std::sync::LazyLock;

use regex::Regex;
use rekall_common::{jstr, Id, InternalFailure, RekallError, Result};
use rekall_model::document::ANCHOR_ID_LENGTH;
use rekall_model::{company, project, task, TaskStepState};
use rekall_repository::repository as repo;

use super::{ContextCommitView, ContextRecord, ContextSkillScope, ContextSkillView, DocumentView, StepConnectionView};
use crate::component::ProjectComponentView;
use crate::connection::ConnectionView;
use crate::step::{StepFlowRows, TaskStepView};
use crate::wrapup::WrapupView;
use crate::{in_read, load, Ctx, Tx};

pub const ENTITY_NAMES: [&str; 4] = ["company", "project", "task", "note"];

#[derive(Clone)]
pub struct ContextService {
    ctx: Ctx,
}

impl ContextService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn load(&self, entity_name: Option<&str>, value: &str) -> Result<ContextRecord> {
        in_read!(&self.ctx, |tx| self.load_in(&tx, entity_name, value).await)
    }

    pub async fn load_in(&self, tx: &Tx, entity_name: Option<&str>, value: &str) -> Result<ContextRecord> {
        let Some(entity_name) = entity_name else {
            return self.load_anywhere(tx, value).await;
        };
        match entity_name.to_lowercase().as_str() {
            "company" | "companies" | "azienda" | "aziende" | "cliente" => {
                match repo::company::find_by_name(tx.db(), value).await? {
                    Some(company) => self.render_company(tx, &company).await,
                    None => Err(RekallError::unknown_anchor(format!("No company matches '{value}'"))),
                }
            }
            "project" | "projects" | "progetto" | "progetti" => {
                let found = repo::project::find_by_label(tx.db(), value).await?;
                self.render_single_project(tx, found, value).await
            }
            "task" | "tasks" | "issue" | "issues" | "attivita" => {
                let task = load::resolve_task(tx.db(), None, value).await?;
                self.render_task(tx, &task).await
            }
            "note" | "notes" | "nota" | "document" => self.render_single_note(tx, value).await,
            _ => Err(RekallError::unknown_anchor(format!(
                "No entity matches '{entity_name}'. Known entities: {}",
                ENTITY_NAMES.join(", ")
            ))),
        }
    }

    pub async fn load_task(&self, project_label: &str, task_label: &str) -> Result<ContextRecord> {
        in_read!(&self.ctx, |tx| self.load_task_in(&tx, project_label, task_label).await)
    }

    pub async fn load_task_in(&self, tx: &Tx, project_label: &str, task_label: &str) -> Result<ContextRecord> {
        let task = load::resolve_task(tx.db(), Some(project_label), task_label).await?;
        self.render_task(tx, &task).await
    }

    /// The project and the task, as `/rk project:<label> task:<label>` loads them for this task.
    pub async fn for_task(&self, task_id: Id) -> Result<Vec<ContextRecord>> {
        in_read!(&self.ctx, |tx| self.for_task_in(&tx, task_id).await)
    }

    pub async fn for_task_in(&self, tx: &Tx, task_id: Id) -> Result<Vec<ContextRecord>> {
        let task = load::task_or_unknown(tx.db(), task_id).await?;
        let project = load::project_of(tx.db(), &task).await?;
        Ok(vec![self.render_project(tx, &project).await?, self.render_task(tx, &task).await?])
    }

    pub async fn load_project(&self, company_name: &str, project_label: &str) -> Result<ContextRecord> {
        in_read!(&self.ctx, |tx| {
            match repo::project::find_by_anchor(tx.db(), company_name, project_label)
                .await?
            {
                Some(project) => self.render_project(&tx, &project).await,
                None => Err(RekallError::unknown_anchor(format!(
                    "No project '{project_label}' on company '{company_name}'"
                ))),
            }
        })
    }

    async fn load_anywhere(&self, tx: &Tx, value: &str) -> Result<ContextRecord> {
        let mut matches = Vec::new();
        if let Some(company) = repo::company::find_by_name(tx.db(), value).await? {
            matches.push(self.render_company(tx, &company).await?);
        }
        for project in repo::project::find_by_label(tx.db(), value).await? {
            matches.push(self.render_project(tx, &project).await?);
        }
        for task in repo::task::find_by_label(tx.db(), value).await? {
            matches.push(self.render_task(tx, &task).await?);
        }
        if matches.is_empty() {
            return Err(RekallError::unknown_anchor(format!(
                "Nothing matches '{value}'. Known entities: {}",
                ENTITY_NAMES.join(", ")
            )));
        }
        if matches.len() > 1 {
            return Err(RekallError::ambiguous(value, matches.iter().map(|m| m.anchor.clone()).collect()));
        }
        Ok(matches.remove(0))
    }

    async fn render_single_project(&self, tx: &Tx, found: Vec<project::Model>, value: &str) -> Result<ContextRecord> {
        if found.is_empty() {
            return Err(RekallError::unknown_anchor(format!("No project matches '{value}'")));
        }
        if found.len() > 1 {
            let mut candidates = Vec::new();
            for project in &found {
                let company = company_of(tx, project).await?;
                candidates.push(format!("company:{}", company.name));
            }
            return Err(RekallError::ambiguous(value, candidates));
        }
        self.render_project(tx, &found[0]).await
    }

    /// A note loaded by its own anchor: the id's first eight characters or more. Fewer is
    /// refused, since a shorter prefix stops being unique long before it stops looking like one.
    async fn render_single_note(&self, tx: &Tx, value: &str) -> Result<ContextRecord> {
        static HEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f-]+$").expect("a valid pattern"));
        let prefix = jstr::strip(value).to_lowercase();
        if jstr::len(&prefix) < ANCHOR_ID_LENGTH || !HEX.is_match(&prefix) {
            return Err(RekallError::unknown_anchor(format!(
                "'{value}' is not a note anchor. A note is loaded by the anchor its task's context gives, \
                 `note:` and the first {ANCHOR_ID_LENGTH} characters of its id."
            )));
        }
        let found = repo::document::find_by_anchor_prefix(tx.db(), &prefix).await?;
        if found.is_empty() {
            return Err(RekallError::unknown_anchor(format!("No note matches '{value}'")));
        }
        if found.len() > 1 {
            return Err(RekallError::ambiguous(value, found.iter().map(|d| d.anchor()).collect()));
        }
        let note = &found[0];
        let tasks = repo::document::tasks_of_document(tx.db(), note.id).await?;
        Ok(ContextRecord {
            kind: "Note".into(),
            label: note.title.clone(),
            anchor: note.anchor(),
            fields: vec![("kind".into(), note.kind.clone())],
            related: tasks.iter().map(|t| format!("task:{}", t.label)).collect(),
            documents: vec![DocumentView::of(note).in_full()],
            ..Default::default()
        })
    }

    async fn render_company(&self, tx: &Tx, company: &company::Model) -> Result<ContextRecord> {
        let projects = repo::project::list_of_company(tx.db(), company.id).await?;
        Ok(ContextRecord {
            kind: "Company".into(),
            label: company.name.clone(),
            anchor: format!("company:{}", company.name),
            related: projects.iter().map(project::Model::anchor).collect(),
            description: company.description.clone(),
            ..Default::default()
        })
    }

    async fn render_project(&self, tx: &Tx, project: &project::Model) -> Result<ContextRecord> {
        let mut fields = vec![("status".to_string(), project.status.name().to_string())];
        if project.auto_commit {
            fields.push((
                "auto-commit".into(),
                "on. Claiming a step with `rekall_step`, or writing the wrapup of a task with no steps, commits \
                 everything in the project's folder and logs that commit against the step or task. Do not `git commit` \
                 yourself."
                    .into(),
            ));
        }
        let company = company_of(tx, project).await?;
        let tasks = repo::task::list_of_project(tx.db(), project.id).await?;
        let components = repo::project_component::list_of_project(tx.db(), project.id).await?;
        let connections = repo::project_connection::list_of_project(tx.db(), project.id).await?;
        Ok(ContextRecord {
            kind: "Project".into(),
            label: project.title.clone(),
            anchor: project.anchor(),
            fields,
            references: vec![self.referenced_company(tx, &company).await?],
            related: tasks.iter().map(|t| format!("task:{}", t.label)).collect(),
            components: components.iter().map(ProjectComponentView::of).collect(),
            connections: connections.iter().map(|(binding, connection)| ConnectionView::of(binding, connection)).collect(),
            blueprint: project.blueprint_markdown.clone(),
            description: project.description.clone(),
            ..Default::default()
        })
    }

    async fn render_task(&self, tx: &Tx, task: &task::Model) -> Result<ContextRecord> {
        let db = tx.db();
        let flow = StepFlowRows::of_task(db, task.id).await?;
        let steps: Vec<TaskStepView> =
            load::steps_of(db, task.id).await?.iter().map(|step| TaskStepView::of(step, &flow)).collect::<Result<_>>()?;
        let project = load::project_of(db, task).await?;

        let mut fields = vec![("status".to_string(), task.status.name().to_string())];
        if let Some(note) = task.review_note.as_deref().filter(|n| !jstr::is_blank(n)) {
            fields.push(("review".into(), format!("sent back — \"{note}\"")));
        }
        let drafts = steps.iter().filter(|s| s.state.draft()).count();
        let checklist = steps.len() - drafts;
        if checklist > 0 {
            let done = steps.iter().filter(|s| s.state.complete()).count();
            let running = steps.iter().filter(|s| s.state.running()).count();
            let claimed = steps.iter().filter(|s| s.state == TaskStepState::Claimed).count();
            let mut summary = format!("{done} of {checklist} done, {} open", checklist - done);
            if running > 0 {
                summary.push_str(&format!(", {running} running"));
            }
            if claimed > 0 {
                summary.push_str(&format!(", {claimed} awaiting review"));
            }
            fields.push(("steps".into(), summary));
        }
        if drafts > 0 {
            fields.push(("drafts".into(), format!("{drafts} not yet promoted to the checklist")));
        }
        if let Some(summary) = flow_summary(&steps) {
            fields.push(("flow".into(), summary));
        }

        let components = repo::project_component::list_of_project(db, project.id).await?;
        let documents = repo::document::documents_of_task(db, task.id).await?;
        let wrapup = load::wrapup_of(db, task.id).await?.map(|found| WrapupView::of(&found, task, &project));
        Ok(ContextRecord {
            kind: "Task".into(),
            label: task.title.clone(),
            anchor: format!("task:{}", task.label),
            fields,
            references: vec![self.referenced_project(tx, &project).await?],
            related: Vec::new(),
            documents: documents.iter().map(DocumentView::of).collect(),
            skills: self.skills_of(tx, task, &steps).await?,
            step_connections: self.step_connections_of(tx, &project, &steps).await?,
            steps,
            commits: self.commits_of(tx, task).await?,
            wrapup,
            components: Vec::new(),
            component_lookup: Some(components.iter().map(ProjectComponentView::of).collect()),
            connections: Vec::new(),
            blueprint: None,
            description: task.description.clone(),
        })
    }

    /// The task's own skills, then its tag's, then those of the steps still to be worked. A skill
    /// already listed at a wider scope is not listed again.
    async fn skills_of(&self, tx: &Tx, task: &task::Model, steps: &[TaskStepView]) -> Result<Vec<ContextSkillView>> {
        let assigned = repo::task_skill::list_of_task(tx.db(), task.id).await?;
        let mut skills: Vec<ContextSkillView> = assigned
            .iter()
            .filter(|skill| skill.step_id.is_none())
            .map(|skill| ContextSkillView { name: skill.skill_name.clone(), scope: ContextSkillScope::Task })
            .collect();
        if let Some(tag) = self.tag_of(tx, task).await? {
            for skill in repo::tag_skill::list_of_tag(tx.db(), tag.id).await? {
                skills.push(ContextSkillView { name: skill.skill_name, scope: ContextSkillScope::Tag(tag.name.clone()) });
            }
        }
        let mut seen = std::collections::HashSet::new();
        skills.retain(|skill| seen.insert(skill.name.clone()));
        for step in steps.iter().filter(|step| !step.state.draft() && !step.state.complete()) {
            for skill in assigned.iter().filter(|skill| skill.step_id == Some(step.id) && !seen.contains(&skill.skill_name)) {
                skills.push(ContextSkillView { name: skill.skill_name.clone(), scope: ContextSkillScope::Step(step.title.clone()) });
            }
        }
        Ok(skills)
    }

    /// The connection each step reaches, and the project's bindings of it.
    async fn step_connections_of(&self, tx: &Tx, project: &project::Model, steps: &[TaskStepView]) -> Result<Vec<StepConnectionView>> {
        let mut found = Vec::new();
        for step in steps {
            let Some(connection_id) = step.connection_id else {
                continue;
            };
            let Some(connection) = repo::external_connection::find_by_id(tx.db(), connection_id).await? else {
                continue;
            };
            let bindings = repo::project_connection::list_of_project_connection(tx.db(), project.id, connection_id).await?;
            let bound_as = bindings.into_iter().map(|binding| binding.label).collect();
            found.push(StepConnectionView { step_id: step.id, label: connection.label, bound_as });
        }
        Ok(found)
    }

    async fn tag_of(&self, tx: &Tx, task: &task::Model) -> Result<Option<rekall_model::tag::Model>> {
        match task.tag_id {
            Some(tag_id) => repo::tag::find_by_id(tx.db(), tag_id).await,
            None => Ok(None),
        }
    }

    /// A project reached from one of its tasks: in full, but with no list of the tasks around it.
    async fn referenced_project(&self, tx: &Tx, project: &project::Model) -> Result<ContextRecord> {
        let full = Box::pin(self.render_project(tx, project)).await?;
        Ok(ContextRecord { related: Vec::new(), steps: Vec::new(), commits: Vec::new(), wrapup: None, ..full })
    }

    /// A company reached from one of its projects: in full, with no list of its other projects.
    async fn referenced_company(&self, tx: &Tx, company: &company::Model) -> Result<ContextRecord> {
        let full = self.render_company(tx, company).await?;
        Ok(ContextRecord { references: Vec::new(), related: Vec::new(), steps: Vec::new(), commits: Vec::new(), wrapup: None, ..full })
    }

    /// Only the commits the console marked for the context, oldest logged first.
    async fn commits_of(&self, tx: &Tx, task: &task::Model) -> Result<Vec<ContextCommitView>> {
        let references =
            repo::commit_reference::list_in_context_of_task(tx.db(), task.id).await?;
        let mut out = Vec::with_capacity(references.len());
        for reference in &references {
            let step = match reference.step_id {
                Some(step_id) => repo::task_step::find_by_id(tx.db(), step_id).await?,
                None => None,
            };
            out.push(ContextCommitView::of(reference, step.as_ref()));
        }
        Ok(out)
    }
}

async fn company_of(tx: &Tx, project: &project::Model) -> Result<company::Model> {
    repo::company::find_by_id(tx.db(), project.company_id)
        .await?
        .ok_or_else(|| RekallError::internal(InternalFailure::Integrity, format!("Project {} has no company", project.id)))
}

/// Where a flow stands, for the field list: the step to work and the moves so far. `None` on a
/// plain checklist.
fn flow_summary(steps: &[TaskStepView]) -> Option<String> {
    let routed: Vec<&TaskStepView> = steps.iter().filter(|step| !step.state.draft()).collect();
    if routed.iter().all(|step| step.links.is_empty()) {
        return None;
    }
    let moves: usize = routed.iter().map(|step| step.visits.len()).sum();
    let summary = match (routed.iter().find(|step| step.is_current()), routed.first()) {
        (Some(current), _) => format!(
            "the steps are routed; the work stands on step {} \"{}\" after {moves} move{}. Claim it with the `outcome` that happened",
            current.position + 1,
            current.title,
            if moves == 1 { "" } else { "s" }
        ),
        (None, Some(first)) if moves == 0 => {
            format!("the steps are routed and not started: the flow begins at step 1 \"{}\"", first.title)
        }
        _ => format!("the steps are routed; no step is current after {moves} moves"),
    };
    Some(summary)
}
