//! The one way a session writes a note on one task. It adds a new one, or, asked to replace, rewrites
//! the body of the note that task carries under the same title. It never moves, detaches or deletes
//! one; the console's document service in rekall-api stays the only path for those.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::{document, document_task, DocumentContextMode};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use serde::Serialize;

use crate::{in_write, load, Ctx, DomainEvent};

pub const KIND: &str = "notes";

pub const TITLE_MAX_CHARACTERS: usize = 255;

pub const BODY_MAX_CHARACTERS: usize = 100_000;

/// Emitted when a session writes or rewrites a note on a task, so an open console can pick it up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteStreamEvent {
    pub task_id: Id,
    pub document_id: Id,
}

/// What a write came to: the note, where it landed, how many notes that task now carries, and,
/// on a replace, how many other tasks read the rewritten body too.
#[derive(Clone, Debug)]
pub struct Written {
    pub title: String,
    pub note_anchor: String,
    pub task_anchor: String,
    pub notes_on_task: usize,
    pub replaced: bool,
    pub other_tasks: usize,
}

/// What to do when the task already carries a note with the requested title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnTitleClash {
    Refuse,
    Replace,
}

#[derive(Clone)]
pub struct NoteService {
    ctx: Ctx,
}

impl NoteService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn write(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        title: Option<&str>,
        body: Option<&str>,
        on_clash: OnTitleClash,
    ) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let wanted = validated_title(title)?;
            let text = validated_body(body)?;
            let existing = repo::document::documents_of_task(tx.db(), task.id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let task_anchor = format!("project:{} task:{}", project.label, task.label);
            let now = self.ctx.now();

            if let Some(clash) =
                existing.iter().find(|note| jstr::equals_ignore_case(jstr::strip(&note.title), &wanted))
            {
                if on_clash == OnTitleClash::Refuse {
                    return Err(RekallError::illegal(format!(
                        "This task already has a note titled '{}' ({}). Nothing was written: pass `replace` to \
                         rewrite that note, or pick another title to add a new one.",
                        clash.title,
                        clash.anchor()
                    )));
                }
                // The title stays as it was typed in the console; only the body is the session's.
                let mut rewritten = clash.clone();
                rewritten.body_markdown = text;
                if rewritten != *clash {
                    rewritten.updated_at = now;
                    rewritten.validate(Phase::Update)?;
                    rewritten.clone().into_active_model().reset_all().update(tx.db()).await?;
                }
                let readers = repo::document::links_of_document_as_added(tx.db(), rewritten.id).await?;
                tx.publish(DomainEvent::Note(NoteStreamEvent { task_id: task.id, document_id: rewritten.id }));
                return Ok(Written {
                    note_anchor: rewritten.anchor(),
                    title: rewritten.title,
                    task_anchor,
                    notes_on_task: existing.len(),
                    replaced: true,
                    other_tasks: readers.len().saturating_sub(1),
                });
            }

            let note = document::Model {
                id: Id::random(),
                title: wanted,
                kind: KIND.to_string(),
                body_markdown: text,
                source_path: None,
                context_mode: DocumentContextMode::Full,
                created_at: now,
                updated_at: now,
            };
            note.validate(Phase::Persist)?;
            note.clone().into_active_model().insert(tx.db()).await?;
            document_task::Model { document_id: note.id, task_id: task.id, position: existing.len() as i32 }
                .into_active_model()
                .insert(tx.db())
                .await?;
            tx.publish(DomainEvent::Note(NoteStreamEvent { task_id: task.id, document_id: note.id }));

            Ok(Written {
                note_anchor: note.anchor(),
                title: note.title,
                task_anchor,
                notes_on_task: existing.len() + 1,
                replaced: false,
                other_tasks: 0,
            })
        })
    }
}

fn validated_title(title: Option<&str>) -> Result<String> {
    let Some(title) = title.filter(|t| !jstr::is_blank(t)) else {
        return Err(RekallError::illegal("A note needs a title."));
    };
    let text = jstr::strip(title);
    let length = jstr::len(text);
    if length > TITLE_MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A note's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}."
        )));
    }
    Ok(text.to_string())
}

fn validated_body(body: Option<&str>) -> Result<String> {
    let Some(body) = body.filter(|b| !jstr::is_blank(b)) else {
        return Err(RekallError::illegal("A note needs a body. An empty one is something a person makes in the console."));
    };
    let text = jstr::strip(body);
    let length = jstr::len(text);
    if length > BODY_MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A note is capped at {BODY_MAX_CHARACTERS} characters and this one is {length}. Split it."
        )));
    }
    Ok(text.to_string())
}
