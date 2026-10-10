use std::collections::BTreeSet;
use std::path::PathBuf;

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::{token_usage, transcript_scan};
use rekall_repository::repository as repo;
use tracing::debug;

use super::{TokenCount, TokenTallyView, TokenUsageStreamEvent, UnitUsage, WorkUnit};
use crate::{in_read, in_write, load, Ctx, DomainEvent, Tx};

/// The one writer of `token_usage` and `transcript_scan`. A session's rows are replaced whole,
/// so reading a transcript twice never counts it twice.
#[derive(Clone)]
pub struct TokenUsageService {
    ctx: Ctx,
}

impl TokenUsageService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn tallies(&self) -> Result<Vec<TokenTallyView>> {
        in_read!(&self.ctx, |tx| Ok(TokenTallyView::summed(&repo::token_usage::list_all(tx.db()).await?)))
    }

    /// The project folders whose sessions are worth reading: every one set on a project.
    pub async fn watched_folders(&self) -> Result<Vec<PathBuf>> {
        in_read!(&self.ctx, |tx| {
            let projects = repo::project::list_oldest_first(tx.db()).await?;
            let folders: BTreeSet<PathBuf> = projects
                .iter()
                .filter_map(|project| project.repo_folder.as_deref())
                .filter(|folder| !jstr::is_blank(folder))
                .map(|folder| PathBuf::from(jstr::strip(folder)))
                .collect();
            Ok(folders.into_iter().collect())
        })
    }

    /// How many bytes of the session's transcript were read last time, if it has been read.
    pub async fn scanned_bytes(&self, session_id: &str) -> Result<Option<u64>> {
        in_read!(&self.ctx, |tx| {
            let scan = repo::transcript_scan::find(tx.db(), session_id).await?;
            Ok(scan.map(|scan| u64::try_from(scan.scanned_bytes).unwrap_or(0)))
        })
    }

    /// The session's spend, replacing an earlier read's. A unit whose task does not resolve is
    /// dropped; one whose step does not leaves its tokens on the task.
    pub async fn record_session(&self, session_id: &str, scanned_bytes: u64, spent: &[UnitUsage]) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let before = repo::token_usage::list_of_session(tx.db(), session_id).await?;
            let rows = self.resolved_rows(&tx, session_id, spent).await?;
            if !same_spend(&before, &rows) {
                let touched: BTreeSet<Id> = before.iter().chain(&rows).map(|row| row.task_id).collect();
                repo::token_usage::replace_of_session(tx.db(), session_id, rows).await?;
                for task_id in touched {
                    self.publish(&mut tx, task_id).await?;
                }
            }
            let scan = transcript_scan::Model {
                session_id: session_id.to_string(),
                scanned_bytes: i64::try_from(scanned_bytes).unwrap_or(i64::MAX),
                scanned_at: self.ctx.now(),
            };
            repo::transcript_scan::save(tx.db(), scan).await
        })
    }

    async fn resolved_rows(&self, tx: &Tx, session_id: &str, spent: &[UnitUsage]) -> Result<Vec<token_usage::Model>> {
        let mut merged: Vec<((Id, Option<Id>), TokenCount)> = Vec::new();
        for usage in spent.iter().filter(|usage| !usage.tokens.is_empty()) {
            let Some(key) = resolve_unit(tx, &usage.unit).await? else { continue };
            match merged.iter_mut().find(|(known, _)| *known == key) {
                Some((_, tokens)) => *tokens += usage.tokens,
                None => merged.push((key, usage.tokens)),
            }
        }
        let now = self.ctx.now();
        Ok(merged
            .into_iter()
            .map(|((task_id, step_id), tokens)| token_usage::Model {
                id: Id::random(),
                session_id: session_id.to_string(),
                task_id,
                step_id,
                input_tokens: stored(tokens.input),
                output_tokens: stored(tokens.output),
                cache_creation_tokens: stored(tokens.cache_creation),
                cache_read_tokens: stored(tokens.cache_read),
                recorded_at: now,
            })
            .collect())
    }

    async fn publish(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let rows = repo::token_usage::list_of_task(tx.db(), task_id).await?;
        tx.publish(DomainEvent::TokenUsage(TokenUsageStreamEvent { task_id, tallies: TokenTallyView::summed(&rows) }));
        Ok(())
    }
}

/// The task and step ids a unit names, or nothing when its task is gone or was never Rekall's.
async fn resolve_unit(tx: &Tx, unit: &WorkUnit) -> Result<Option<(Id, Option<Id>)>> {
    let task = match load::resolve_task(tx.db(), unit.project_label.as_deref(), &unit.task_label).await {
        Ok(task) => task,
        Err(RekallError::UnknownAnchor(reason) | RekallError::AmbiguousAnchor { message: reason, .. }) => {
            debug!(%reason, task = %unit.task_label, "Tokens on a task that does not resolve are not counted");
            return Ok(None);
        }
        Err(other) => return Err(other),
    };
    let Some(reference) = unit.step.as_deref() else { return Ok(Some((task.id, None))) };
    match load::resolve_step(tx.db(), task.id, Some(reference)).await {
        Ok(step) => Ok(Some((task.id, Some(step.id)))),
        Err(RekallError::IllegalArgument(reason)) => {
            debug!(%reason, task_id = %task.id, step = reference, "Tokens on a step that does not resolve stay on its task");
            Ok(Some((task.id, None)))
        }
        Err(other) => Err(other),
    }
}

/// Whether a fresh read found what the stored rows already say, ignoring their ids and stamps.
fn same_spend(before: &[token_usage::Model], after: &[token_usage::Model]) -> bool {
    fn key(row: &token_usage::Model) -> (Id, Option<Id>, i64, i64, i64, i64) {
        (row.task_id, row.step_id, row.input_tokens, row.output_tokens, row.cache_creation_tokens, row.cache_read_tokens)
    }
    let mut before: Vec<_> = before.iter().map(key).collect();
    let mut after: Vec<_> = after.iter().map(key).collect();
    before.sort();
    after.sort();
    before == after
}

fn stored(count: u64) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}
