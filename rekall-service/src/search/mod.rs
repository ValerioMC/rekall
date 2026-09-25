//! Finds a phrase in the text the console's filter does not reach: task descriptions, steps (title
//! and detail) and wrapups, plus notes so they turn up while browsing tasks. The phrase is matched
//! whole and case-insensitively, as typed; `%` and `_` in it are literal. Under three characters
//! there is no search. Each kind returns at most eight hits, newest first, and within those a hit
//! on a title comes before a hit in the body.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::task;
use rekall_repository::repository as repo;
use serde::Serialize;

use crate::{in_read, load, Ctx, Tx};

pub const TERM_MIN: usize = 3;
pub const PER_KIND: usize = 8;
/// Characters kept either side of the match in an excerpt.
pub const EXCERPT_RADIUS: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SearchHitKind {
    Description,
    Step,
    Wrapup,
    Note,
}

/// One place a search term was found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: SearchHitKind,
    /// The task to open: the one the text is on, or for a note the first task it is on.
    pub task_id: Option<Id>,
    pub step_id: Option<Id>,
    pub document_id: Option<Id>,
    pub title: String,
    /// The anchor of the task the text is on, for a note the one it opens on.
    pub r#where: String,
    pub excerpt: String,
}

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
mod tests {
    use super::*;

    #[test]
    fn the_like_wildcards_in_a_term_are_matched_literally() {
        assert_eq!(escape_like("100%_done\\"), "100\\%\\_done\\\\");
    }

    #[test]
    fn an_excerpt_surrounds_the_match_on_one_line_cut_at_a_word() {
        let text = format!("{}the\nsettlement   batch runs nightly {}", "word ".repeat(30), "tail ".repeat(30));
        let excerpt = excerpt(Some(&text), "Settlement batch");
        assert!(excerpt.starts_with("…word"), "{excerpt}");
        assert!(excerpt.ends_with('…'));
        assert!(excerpt.contains("the settlement batch runs nightly"));
        assert!(!excerpt.contains('\n'));
        assert!(!excerpt.contains("  "));
        assert!(jstr::len(&excerpt) <= 2 * EXCERPT_RADIUS + "settlement batch".len() + 2);
    }

    #[test]
    fn a_short_text_is_returned_whole_with_no_ellipsis() {
        assert_eq!(excerpt(Some("Wire the export"), "export"), "Wire the export");
    }
}
