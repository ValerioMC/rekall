use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rekall_kernel::{jstr, Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::task_pull_request::FAILURE_MAX_CHARACTERS;
use rekall_store::{task, task_pull_request, TaskKind};
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
use tracing::{info, warn};

use crate::claude::{Remaining, TaskWork, TaskWorkService};
use crate::commit::GitRepositoryInspector;
use crate::connection::{PullRequest, PullRequestHost, PullRequestMerge, TaskTrackerService, TrackerEndpoint, TrackerFailure, TrackerPictureService, Mergeability};
use crate::worktree::rule::request_noun::request_noun;
use crate::worktree::PullRequestChanged;
use crate::worktree::{pull_request_body, PullRequestLanding, PushOutcome, StepCommit, Worktree, WorktreeHost, WorktreeLayout};
use crate::{in_read, in_write, load, Ctx};

/// What a claimed task of a project with the pull request flow on needs sent to the remote.
struct Claim {
    task: task::Model,
    project_folder: String,
    main_branch: Option<String>,
    body: String,
    issue_key: Option<String>,
    held_number: Option<i64>,
    version: i64,
}

/// What the project of an accepted task holds for its pull request.
struct Landing {
    project_id: Id,
    project_folder: Option<String>,
    main_branch: Option<String>,
    branch: String,
    worktree_folder: Option<String>,
    pull_request: Option<task_pull_request::Model>,
    issue_key: Option<String>,
}

/// Pushes the branch of a task whose work is claimed and opens its pull request, once. The number
/// and page, or why it failed, are kept on the task; the next claim tries again.
#[derive(Clone)]
pub struct TaskPullRequestService {
    ctx: Ctx,
    tracker: TaskTrackerService,
    pictures: TrackerPictureService,
    worktrees: Arc<dyn WorktreeHost>,
    hosts: Arc<dyn PullRequestHost>,
    layout: WorktreeLayout,
    repositories: GitRepositoryInspector,
}

impl TaskPullRequestService {
    pub fn new(
        ctx: Ctx,
        tracker: TaskTrackerService,
        pictures: TrackerPictureService,
        worktrees: Arc<dyn WorktreeHost>,
        hosts: Arc<dyn PullRequestHost>,
        layout: WorktreeLayout,
    ) -> Self {
        Self { ctx, tracker, pictures, worktrees, hosts, layout, repositories: GitRepositoryInspector }
    }

    /// A task outside the flow, or one whose work is not all claimed, is left alone. A task that
    /// already has its pull request only has its new commits pushed.
    pub async fn open_if_claimed(&self, task_id: Id) -> Result<()> {
        let Some(claim) = self.read(task_id).await? else { return Ok(()) };
        match self.deliver(&claim).await {
            Ok(Some(pull_request)) => {
                info!(%task_id, number = pull_request.number, "A task's pull request is open");
                self.keep_delivery(task_id, claim.version, Some(pull_request)).await
            }
            Ok(None) => self.keep_delivery(task_id, claim.version, None).await,
            Err(failure) => {
                warn!(%failure, %task_id, "A task's pull request could not be opened");
                self.keep_failure(task_id, Some(failure.message().to_string())).await
            }
        }
    }

    pub async fn retry_candidates(&self) -> Result<Vec<Id>> {
        in_read!(&self.ctx, |tx| {
            let mut candidates = Vec::new();
            for task in repo::task::list_with_worktree(tx.db()).await? {
                let project = load::project_of(tx.db(), &task).await?;
                if !project.pull_request_flow {
                    continue;
                }
                let held = repo::task_pull_request::find_of_task(tx.db(), task.id).await?;
                let work = TaskWorkService::read_in(&tx, task.id).await?;
                if held.as_ref().is_some_and(|request| request.awaiting_push())
                    || (held.as_ref().is_none_or(|request| request.number.is_none()) && has_claim(&work))
                    || (work.remaining == Remaining::Accepted && task.status != rekall_store::TaskStatus::Done)
                {
                    candidates.push(task.id);
                }
            }
            Ok(candidates)
        })
    }

    pub async fn record_failure(&self, task_id: Id, reason: String) -> Result<()> {
        self.keep_failure(task_id, Some(reason)).await
    }

    /// Merges the accepted task's pull request with a merge commit, closes its issue and deletes its branch on the remote. A merge the
    /// host turns down is kept on the task as the reason. A network error or a 5xx is an `Err`: the
    /// pull request is untouched and the next move of the task tries again.
    pub async fn merge_accepted(&self, task_id: Id) -> Result<PullRequestLanding> {
        let Some(landing) = self.read_landing(task_id).await? else { return Ok(PullRequestLanding::NotInFlow) };
        let Some(held) = landing.pull_request.clone() else { return Ok(PullRequestLanding::NothingToMerge) };
        if held.merged_at.is_some() {
            return Ok(PullRequestLanding::AlreadyMerged);
        }
        if held.awaiting_push() {
            return Ok(PullRequestLanding::Refused(held.failure.unwrap_or_else(|| "The latest claim is waiting to be pushed.".into())));
        }
        let Some(number) = held.number.and_then(|number| u64::try_from(number).ok()) else {
            let reason = held.failure.unwrap_or_else(|| "No pull request was opened for this task.".to_string());
            return Ok(PullRequestLanding::Refused(reason));
        };
        if let (Some(folder), Some(project_folder)) = (&landing.worktree_folder, &landing.project_folder) {
            let worktree = Worktree { folder: folder.into(), project_folder: project_folder.into(), branch: landing.branch.clone() };
            if worktree.folder.is_dir() && self.worktrees.status(&worktree).await?.has_unsaved_changes() {
                return self.refuse_merge(task_id, "The task worktree has unsaved changes. The branch stays open.".into()).await;
            }
        }
        let pull_request = PullRequest { number, url: held.url.unwrap_or_default(), mergeability: Mergeability::Unknown };
        let endpoint = match self.tracker.endpoint_of_project(landing.project_id).await {
            Ok(Some(endpoint)) => endpoint,
            Ok(None) => return self.refuse_merge(task_id, "The project has no task manager to merge the pull request through.".to_string()).await,
            Err(failure) => return self.refuse_merge(task_id, failure.message().to_string()).await,
        };
        self.merge(task_id, &endpoint, &pull_request, &landing).await
    }

    async fn merge(&self, task_id: Id, endpoint: &TrackerEndpoint, pull_request: &PullRequest, landing: &Landing) -> Result<PullRequestLanding> {
        match self.hosts.merge(endpoint, pull_request).await {
            Ok(PullRequestMerge::Merged) => {
                info!(%task_id, number = pull_request.number, "A task's pull request is merged");
                self.keep_merged(task_id).await?;
                self.close_issue(task_id, endpoint, landing.issue_key.as_deref()).await;
                self.delete_remote_branch(task_id, endpoint, &landing.branch).await;
                Ok(PullRequestLanding::Merged)
            }
            Ok(PullRequestMerge::Conflicting) => {
                let base = self.base_of(landing.project_folder.as_deref().unwrap_or_default(), landing.main_branch.as_deref()).await?;
                self.keep_failure(task_id, Some(format!("The {} conflicts with {base}.", request_noun(endpoint.kind)))).await?;
                Ok(PullRequestLanding::Conflicting { base })
            }
            Ok(PullRequestMerge::Refused(reason)) => self.refuse_merge(task_id, format!("The {} was not merged: {reason}", request_noun(endpoint.kind))).await,
            Err(failure) => {
                let message = failure_message(&failure, &format!("merge the {}", request_noun(endpoint.kind)));
                warn!(failure = %message, %task_id, "A task's pull request could not be merged");
                self.keep_failure(task_id, Some(message.clone())).await?;
                Err(RekallError::conflict(message))
            }
        }
    }

    async fn refuse_merge(&self, task_id: Id, reason: String) -> Result<PullRequestLanding> {
        self.keep_failure(task_id, Some(reason.clone())).await?;
        Ok(PullRequestLanding::Refused(reason))
    }

    async fn close_issue(&self, task_id: Id, endpoint: &TrackerEndpoint, issue_key: Option<&str>) {
        let Some(issue_key) = issue_key else { return };
        if let Err(failure) = self.hosts.close_issue(endpoint, issue_key).await {
            let message = format!("The {} is merged. {}", request_noun(endpoint.kind), failure_message(&failure, "close its issue"));
            warn!(failure = %message, %task_id, issue = issue_key, "A merged task's issue could not be closed");
            if let Err(failure) = self.keep_failure(task_id, Some(message)).await {
                warn!(%failure, %task_id, "Could not keep why a merged task's issue stayed open");
            }
        }
    }

    async fn delete_remote_branch(&self, task_id: Id, endpoint: &TrackerEndpoint, branch: &str) {
        if let Err(failure) = self.hosts.delete_branch(endpoint, branch).await {
            let message = format!("The {} is merged. {}", request_noun(endpoint.kind), failure_message(&failure, "delete its branch"));
            warn!(failure = %message, %task_id, branch, "A merged task's branch could not be deleted on the remote");
            if let Err(failure) = self.keep_failure(task_id, Some(message)).await {
                warn!(%failure, %task_id, "Could not keep why a merged task's branch stayed on the remote");
            }
        }
    }

    async fn read_landing(&self, task_id: Id) -> Result<Option<Landing>> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            if !project.pull_request_flow {
                return Ok(None);
            }
            let pull_request = match TaskWorkService::read_in(&tx, task_id).await?.remaining {
                Remaining::Accepted => repo::task_pull_request::find_of_task(tx.db(), task_id).await?,
                _ => None,
            };
            let issue_key = repo::task_tracker_link::find_of_task(tx.db(), task_id).await?.and_then(|link| link.issue_key);
            let branch = self.layout.branch_for(&task.label)?;
            Ok(Some(Landing { project_id: project.id, project_folder: project.repo_folder, main_branch: project.main_branch, branch, worktree_folder: task.worktree_folder, pull_request, issue_key }))
        })
    }

    async fn read(&self, task_id: Id) -> Result<Option<Claim>> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            if task.kind != TaskKind::Task || task.quick {
                return Ok(None);
            }
            let project = load::project_of(tx.db(), &task).await?;
            let Some(project_folder) = project.repo_folder.clone().filter(|folder| !jstr::is_blank(folder)) else { return Ok(None) };
            if !project.pull_request_flow {
                return Ok(None);
            }
            let held = repo::task_pull_request::find_of_task(tx.db(), task_id).await?;
            let work = TaskWorkService::read_in(&tx, task_id).await?;
            if !awaits_push(&work, held.as_ref()) {
                return Ok(None);
            }
            let held_number = held.as_ref().and_then(|held| held.number);
            let version = held.map_or(0, |held| held.delivery_version);
            let issue_key = repo::task_tracker_link::find_of_task(tx.db(), task_id).await?.and_then(|link| link.issue_key);
            let commits = step_commits_of(tx.db(), task_id).await?;
            let body = pull_request_body(task.description.as_deref(), issue_key.as_deref(), &commits);
            Ok(Some(Claim { task, project_folder, main_branch: project.main_branch, body, issue_key, held_number, version }))
        })
    }

    async fn deliver(&self, claim: &Claim) -> Result<Option<PullRequest>> {
        let branch = self.layout.branch_for(&claim.task.label)?;
        self.push(claim, &branch).await?;
        if claim.held_number.is_some() {
            return Ok(None);
        }
        let endpoint = self.tracker.endpoint_of_project(claim.task.project_id).await?
            .ok_or_else(|| RekallError::conflict("The project needs a task manager to open a pull request."))?;
        let base = self.base_of(&claim.project_folder, claim.main_branch.as_deref()).await?;
        let body = self.pictures.send(&endpoint, claim.task.id, claim.issue_key.as_deref(), &claim.body).await?;
        let opened = self.hosts.open(&endpoint, &branch, &base, &claim.task.title, &body).await;
        opened.map(Some).map_err(|failure| RekallError::conflict(failure_message(&failure, &format!("open the {}", request_noun(endpoint.kind)))))
    }

    async fn push(&self, claim: &Claim, branch: &str) -> Result<()> {
        match self.worktrees.push_branch(Path::new(&claim.project_folder), branch).await? {
            PushOutcome::Pushed | PushOutcome::UpToDate => Ok(()),
            PushOutcome::Refused(reason) => Err(RekallError::conflict(format!("The push of {branch} was refused: {reason}"))),
        }
    }

    async fn base_of(&self, project_folder: &str, main_branch: Option<&str>) -> Result<String> {
        if let Some(branch) = main_branch.map(jstr::strip).filter(|branch| !branch.is_empty()) {
            return Ok(branch.to_string());
        }
        let status = self.repositories.inspect(Some(project_folder)).await;
        status.branch.ok_or_else(|| RekallError::illegal("Name this project's main branch: its folder is not on a branch to open pull requests against."))
    }

    async fn keep_delivery(&self, task_id: Id, version: i64, pull_request: Option<PullRequest>) -> Result<()> {
        let number = pull_request.as_ref().map(|request| i64::try_from(request.number))
            .transpose().map_err(|failure| RekallError::internal(InternalFailure::Integrity, failure))?;
        self.keep(task_id, |mut held| {
            if let Some(request) = pull_request {
                held.number = number;
                held.url = Some(request.url);
            }
            held.published_version = held.published_version.max(version);
            held.failure = None;
            held
        }).await
    }

    async fn keep_merged(&self, task_id: Id) -> Result<()> {
        let now = self.ctx.now();
        self.keep(task_id, |held| task_pull_request::Model { merged_at: Some(now), failure: None, ..held }).await
    }

    async fn keep_failure(&self, task_id: Id, failure: Option<String>) -> Result<()> {
        let failure = failure.map(|message| message.chars().take(FAILURE_MAX_CHARACTERS).collect());
        self.keep(task_id, |held| task_pull_request::Model { failure, ..held }).await
    }

    /// Saves what the claim found, creating the row the first time, and moves the task's own stamp
    /// when the row changed, since the console orders what it receives by it.
    async fn keep(&self, task_id: Id, change: impl FnOnce(task_pull_request::Model) -> task_pull_request::Model) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let now = self.ctx.now();
            let existing = repo::task_pull_request::find_of_task(tx.db(), task_id).await?;
            let held = existing.clone().unwrap_or_else(|| task_pull_request::Model::empty(task_id, now));
            let changed = change(held.clone());
            if existing.as_ref() == Some(&changed) || (existing.is_none() && changed == held) {
                return Ok(());
            }
            repo::task_pull_request::save(tx.db(), task_pull_request::Model { updated_at: now, ..changed }).await?;
            let mut stamped = task.into_active_model();
            stamped.updated_at = Set(now);
            stamped.update(tx.db()).await?;
            tx.publish(PullRequestChanged { task_id });
            Ok(())
        })
    }
}

fn awaits_push(work: &TaskWork, held: Option<&task_pull_request::Model>) -> bool {
    if !has_claim(work) {
        return false;
    }
    match held {
        None => true,
        Some(request) if request.merged_at.is_some() => false,
        Some(request) => request.awaiting_push() || request.number.is_none()
            || (request.delivery_version == 0 && work.remaining == Remaining::Claimed),
    }
}

fn has_claim(work: &TaskWork) -> bool {
    work.settled_steps > 0 || work.remaining != Remaining::Open
}

async fn step_commits_of(db: &impl sea_orm::ConnectionTrait, task_id: Id) -> Result<Vec<StepCommit>> {
    let titles: HashMap<Id, String> = load::steps_of(db, task_id).await?.into_iter().map(|step| (step.id, step.title)).collect();
    let mut logged = repo::commit_reference::list_of_task(db, task_id).await?;
    logged.sort_by_key(|commit| commit.created_at);
    Ok(logged
        .into_iter()
        .map(|commit| StepCommit { step_title: commit.step_id.and_then(|step_id| titles.get(&step_id).cloned()), hash: commit.commit_hash, subject: commit.comment })
        .collect())
}

fn failure_message(failure: &TrackerFailure, action: &str) -> String {
    match failure {
        TrackerFailure::Rejected { status } => format!("The task manager rejected the stored credential (HTTP {status}). Replace it on the Connections page."),
        TrackerFailure::Failed { reason } => format!("The host could not {action}: {reason}."),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/task_pull_request_service_tests.rs"]
mod tests;
