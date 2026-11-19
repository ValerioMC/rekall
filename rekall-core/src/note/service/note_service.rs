use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{document, task, DocumentContextMode};
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::{in_write, load, text_rule, Ctx, Tx};

use crate::note::{NoteStreamEvent, OnTitleClash, Written};

pub const KIND: &str = "notes";

pub const TITLE_MAX_CHARACTERS: usize = 255;

pub const BODY_MAX_CHARACTERS: usize = 100_000;

/// Writes a session's note on one task, or rewrites the one it carries under the same title.
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
            let target = Target { task: &task, task_anchor: task.anchor(), notes_on_task: existing.len() };

            match existing.iter().find(|note| jstr::equals_ignore_case(jstr::strip(&note.title), &wanted)) {
                Some(clash) if on_clash == OnTitleClash::Refuse => Err(clash_refusal(clash)),
                Some(clash) => self.rewrite(&mut tx, target, clash, text).await,
                None => self.add(&mut tx, target, wanted, text).await,
            }
        })
    }

    /// The title stays as it was typed in the console; only the body is the session's.
    async fn rewrite(&self, tx: &mut Tx, target: Target<'_>, clash: &document::Model, text: String) -> Result<Written> {
        let mut rewritten = clash.clone();
        rewritten.body_markdown = text;
        let rewritten = repo::update_if_changed(tx.db(), clash, rewritten, self.ctx.now()).await?;
        tx.publish(NoteStreamEvent { task_id: target.task.id, document_id: rewritten.id });
        Ok(Written {
            note_anchor: rewritten.anchor(),
            title: rewritten.title,
            task_anchor: target.task_anchor,
            notes_on_task: target.notes_on_task,
            replaced: true,
        })
    }

    async fn add(&self, tx: &mut Tx, target: Target<'_>, title: String, text: String) -> Result<Written> {
        let now = self.ctx.now();
        let note = document::Model {
            id: Id::random(),
            task_id: target.task.id,
            position: target.notes_on_task as i32,
            title,
            kind: KIND.to_string(),
            body_markdown: text,
            source_path: None,
            context_mode: DocumentContextMode::Full,
            created_at: now,
            updated_at: now,
        };
        note.validate()?;
        note.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(NoteStreamEvent { task_id: target.task.id, document_id: note.id });
        Ok(Written {
            note_anchor: note.anchor(),
            title: note.title,
            task_anchor: target.task_anchor,
            notes_on_task: target.notes_on_task + 1,
            replaced: false,
        })
    }
}

/// The task a note is written on, as both write paths report it.
struct Target<'a> {
    task: &'a task::Model,
    task_anchor: String,
    notes_on_task: usize,
}

fn clash_refusal(clash: &document::Model) -> RekallError {
    RekallError::illegal(format!(
        "This task already has a note titled '{}' ({}). Nothing was written: pass `replace` to rewrite that note, or \
         pick another title to add a new one.",
        clash.title,
        clash.anchor()
    ))
}

fn validated_title(title: Option<&str>) -> Result<String> {
    let title = text_rule::present(title).ok_or_else(|| RekallError::illegal("A note needs a title."))?;
    text_rule::within(title, TITLE_MAX_CHARACTERS, |length| {
        format!("A note's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}.")
    })
}

fn validated_body(body: Option<&str>) -> Result<String> {
    let body = text_rule::present(body).ok_or_else(|| {
        RekallError::illegal("A note needs a body. An empty one is something a person makes in the console.")
    })?;
    text_rule::within(body, BODY_MAX_CHARACTERS, |length| {
        format!("A note is capped at {BODY_MAX_CHARACTERS} characters and this one is {length}. Split it.")
    })
}
