use std::path::Path;

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::repository as repo;

use crate::commit::{CommitTouch, GitLogReader};
use crate::{in_read, Ctx, Tx};

use crate::wrapup::model::named_paths::MAX_NAMED_PATHS;
use crate::wrapup::{Freshness, NamedPaths};

/// Asks git whether the files a wrapup names changed after the project's `HEAD` the wrapup was
/// written at. Commits logged against the task itself are not counted: its wrapup is written
/// before the claim that commits the work it describes.
#[derive(Clone)]
pub struct WrapupFreshnessService {
    ctx: Ctx,
    log: GitLogReader,
}

impl WrapupFreshnessService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, log: GitLogReader }
    }

    pub async fn check(&self, task_id: Id) -> Result<Freshness> {
        in_read!(&self.ctx, |tx| self.check_in(&tx, task_id).await)
    }

    pub async fn check_in(&self, tx: &Tx, task_id: Id) -> Result<Freshness> {
        let Some(wrapup) = repo::wrapup::find_of_task(tx.db(), task_id).await? else {
            return Ok(Freshness::Unknown);
        };
        let versions = repo::wrapup_version::list_of_task_newest_first(tx.db(), task_id).await?;
        let Some(written_at) = versions.first().and_then(|version| version.commit_hash.clone()) else {
            return Ok(Freshness::Unknown);
        };
        let task = crate::load::task_or_unknown(tx.db(), task_id).await?;
        let project = crate::load::project_of(tx.db(), &task).await?;
        let Some(folder) = project.repo_folder.filter(|folder| !jstr::is_blank(folder)) else {
            return Ok(Freshness::Unknown);
        };
        let paths = NamedPaths::in_text(&wrapup.body_markdown);
        if paths.is_empty() || paths.len() > MAX_NAMED_PATHS {
            return Ok(Freshness::Unknown);
        }
        let own: Vec<String> =
            repo::commit_reference::list_of_task(tx.db(), task_id).await?.into_iter().map(|row| row.commit_hash).collect();
        match self.log.touching_since(Path::new(jstr::strip(&folder)), &written_at, &paths).await {
            Ok(touches) => Ok(verdict(touches, &own)),
            Err(failure) => Ok(unknown_because(failure, task_id)),
        }
    }
}

fn verdict(touches: Vec<CommitTouch>, own_commits: &[String]) -> Freshness {
    let foreign: Vec<CommitTouch> = touches.into_iter().filter(|touch| !own_commits.contains(&touch.hash)).collect();
    if foreign.is_empty() {
        return Freshness::Fresh;
    }
    let mut changed_paths: Vec<String> = foreign.iter().flat_map(|touch| touch.files.clone()).collect();
    changed_paths.sort();
    changed_paths.dedup();
    Freshness::PossiblyStale { changed_paths, commits: foreign.into_iter().map(|touch| touch.hash).collect() }
}

fn unknown_because(failure: RekallError, task_id: Id) -> Freshness {
    tracing::debug!(%failure, %task_id, "the freshness of a wrapup could not be read");
    Freshness::Unknown
}
