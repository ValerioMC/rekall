use rekall_common::{jstr, RekallError, Result};
use rekall_model::task;
use rekall_repository::repository as repo;

use crate::{in_read, load, Ctx, Tx};

use super::{SearchHit, SearchHitKind};

pub const TERM_MIN: usize = 3;
pub const PER_KIND: usize = 8;
/// Characters kept either side of the match in an excerpt.
pub const EXCERPT_RADIUS: usize = 60;

#[derive(Clone)]
pub struct SearchService {
    ctx: Ctx,
}

impl SearchService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn search(&self, raw_term: Option<&str>) -> Result<Vec<SearchHit>> {
        let term = jstr::collapse_whitespace(jstr::strip(raw_term.unwrap_or("")));
        if jstr::len(&term) < TERM_MIN {
            return Ok(Vec::new());
        }
        in_read!(&self.ctx, |tx| {
            // The repositories take the term literally, which is what escaping it for LIKE did.
            let mut hits = Vec::new();
            for task in repo::task::search_descriptions(tx.db(), &term, PER_KIND).await? {
                hits.push(SearchHit {
                    kind: SearchHitKind::Description,
                    task_id: Some(task.id),
                    step_id: None,
                    document_id: None,
                    title: task.title.clone(),
                    r#where: anchor_of(&tx, &task).await?,
                    excerpt: excerpt(task.description.as_deref(), &term),
                });
            }
            for step in repo::task_step::search(tx.db(), &term, PER_KIND).await? {
                let task = load::task_or_unknown(tx.db(), step.task_id).await?;
                let body = if contains(step.body_markdown.as_deref(), &term) {
                    step.body_markdown.clone()
                } else {
                    Some(step.title.clone())
                };
                hits.push(SearchHit {
                    kind: SearchHitKind::Step,
                    task_id: Some(task.id),
                    step_id: Some(step.id),
                    document_id: None,
                    title: step.title.clone(),
                    r#where: anchor_of(&tx, &task).await?,
                    excerpt: excerpt(body.as_deref(), &term),
                });
            }
            for wrapup in repo::wrapup::search(tx.db(), &term, PER_KIND).await? {
                let task = load::task_or_unknown(tx.db(), wrapup.task_id).await?;
                hits.push(SearchHit {
                    kind: SearchHitKind::Wrapup,
                    task_id: Some(task.id),
                    step_id: None,
                    document_id: None,
                    title: task.title.clone(),
                    r#where: anchor_of(&tx, &task).await?,
                    excerpt: excerpt(Some(&wrapup.body_markdown), &term),
                });
            }
            for document in repo::document::search_text(tx.db(), &term, PER_KIND).await? {
                let first = repo::document::tasks_of_document(tx.db(), document.id).await?.into_iter().next();
                let body = if contains(Some(&document.body_markdown), &term) {
                    document.body_markdown.clone()
                } else {
                    document.title.clone()
                };
                hits.push(SearchHit {
                    kind: SearchHitKind::Note,
                    task_id: first.as_ref().map(|t| t.id),
                    step_id: None,
                    document_id: Some(document.id),
                    title: document.title.clone(),
                    r#where: match &first {
                        Some(task) => anchor_of(&tx, task).await?,
                        None => String::new(),
                    },
                    excerpt: excerpt(Some(&body), &term),
                });
            }
            // Stable: within a kind, the newest first, and a title hit before a body hit.
            hits.sort_by_key(|hit| (hit.kind as usize, !contains(Some(&hit.title), &term)));
            Ok::<_, RekallError>(hits)
        })
    }
}

async fn anchor_of(tx: &Tx, task: &task::Model) -> Result<String> {
    let project = load::project_of(tx.db(), task).await?;
    Ok(format!("project:{} task:{}", project.label, task.label))
}

fn contains(text: Option<&str>, term: &str) -> bool {
    text.is_some_and(|t| t.to_lowercase().contains(&term.to_lowercase()))
}

/// `SearchService.escapeLike`: `\`, `%` and `_` escaped with a backslash.
pub fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

/// The match with the radius either side, whitespace collapsed, cut at words.
pub fn excerpt(text: Option<&str>, term: &str) -> String {
    let Some(text) = text else {
        return String::new();
    };
    let flat_owned = jstr::collapse_whitespace(text);
    let flat = jstr::strip(&flat_owned);
    let length = jstr::len(flat);
    let lowered = flat.to_lowercase();
    let Some(at) = jstr::index_of(&lowered, &term.to_lowercase()) else {
        return if length <= EXCERPT_RADIUS * 2 {
            flat.to_string()
        } else {
            format!("{}…", jstr::strip_trailing(jstr::prefix(flat, EXCERPT_RADIUS * 2)))
        };
    };
    let term_length = jstr::len(term);
    let mut start = at.saturating_sub(EXCERPT_RADIUS);
    let mut end = length.min(at + term_length + EXCERPT_RADIUS);
    if start > 0 {
        if let Some(space) = jstr::index_of_from(flat, " ", start) {
            if space < at {
                start = space + 1;
            }
        }
    }
    if end < length {
        if let Some(space) = jstr::last_index_of_from(flat, " ", end) {
            if space > at + term_length {
                end = space;
            }
        }
    }
    format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        jstr::substring(flat, start, end),
        if end < length { "…" } else { "" }
    )
}

#[cfg(test)]
#[path = "../../test/search/search_service_tests.rs"]
mod tests;
