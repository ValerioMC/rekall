use rekall_common::{jstr, Id, Result};
use rekall_model::DocumentContextMode;

use super::{ContextRenderer, ContextService};
use crate::{in_read, Ctx};

use super::{ContextSize, ContextSizePart};

pub const CHARACTERS_PER_TOKEN: f64 = 3.5;

#[derive(Clone)]
pub struct ContextSizeService {
    ctx: Ctx,
    context: ContextService,
}

impl ContextSizeService {
    pub fn new(ctx: Ctx, context: ContextService) -> Self {
        Self { ctx, context }
    }

    pub async fn measure(&self, task_id: Id) -> Result<ContextSize> {
        in_read!(&self.ctx, |tx| {
            let renderer = ContextRenderer;
            let records = self.context.for_task_in(&tx, task_id).await?;
            let total = jstr::len(&renderer.render(&records));
            let task = records.last().expect("for_task returns the project and the task");

            let mut parts = Vec::new();
            add_part(&mut parts, "Description", &renderer.render_description(task.description.as_deref()), false);
            add_part(&mut parts, "Steps", &renderer.render_steps(&task.steps, task.wrapup.as_ref()), false);
            add_part(&mut parts, "Wrapup", &renderer.render_wrapup(task.wrapup.as_ref()), false);
            add_part(&mut parts, "Commits", &renderer.render_commits(&task.commits), false);
            for document in &task.documents {
                add_part(
                    &mut parts,
                    &format!("Note: {}", document.title),
                    &renderer.render_documents(std::slice::from_ref(document)),
                    document.context_mode == DocumentContextMode::Reference,
                );
            }
            let measured: usize = parts.iter().map(|p| p.characters).sum();
            if total > measured {
                parts.push(ContextSizePart {
                    label: "Project, blueprint and headings".into(),
                    characters: total - measured,
                    reference: false,
                });
            }
            parts.sort_by(|a, b| b.characters.cmp(&a.characters));
            Ok(ContextSize { characters: total, estimated_tokens: estimate_tokens(total), parts })
        })
    }
}

pub fn estimate_tokens(characters: usize) -> usize {
    (characters as f64 / CHARACTERS_PER_TOKEN).ceil() as usize
}

fn add_part(parts: &mut Vec<ContextSizePart>, label: &str, rendered: &str, reference: bool) {
    if !rendered.is_empty() {
        parts.push(ContextSizePart { label: label.to_string(), characters: jstr::len(rendered), reference });
    }
}
