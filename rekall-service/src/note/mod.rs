//! The one way a session writes a note: a new one, on one task. It cannot open a note that is
//! already there, so it cannot edit, move, detach or delete one; the console's document service
//! in rekall-api stays the only path for those, and out of the MCP tools' reach.

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

/// Emitted when a session writes a new note onto a task, so an open console can pick it up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteStreamEvent {
    pub task_id: Id,
    pub document_id: Id,
}

/// What a write came to: the note, where it landed, and how many notes that task now carries.
#[derive(Clone, Debug)]
pub struct Written {
    pub title: String,
    pub note_anchor: String,
    pub task_anchor: String,
    pub notes_on_task: usize,
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
    ) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let wanted = validated_title(title)?;
            let text = validated_body(body)?;
            let existing = repo::document::documents_of_task(tx.db(), task.id).await?;
            if let Some(clash) =
                existing.iter().find(|note| jstr::equals_ignore_case(jstr::strip(&note.title), &wanted))
            {
                return Err(RekallError::illegal(format!(
                    "This task already has a note titled '{}' ({}). Nothing was written: a session only adds notes, \
                     so pick another title, or leave changing that one to a person in the console.",
                    clash.title,
                    clash.anchor()
                )));
            }

            let now = self.ctx.now();
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

            let project = load::project_of(tx.db(), &task).await?;
            Ok(Written {
                note_anchor: note.anchor(),
                title: note.title,
                task_anchor: format!("project:{} task:{}", project.label, task.label),
                notes_on_task: existing.len() + 1,
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
