//! Putting every context and page of a project where an arrangement says. The console's drag and a
//! session's reorganization both end here, so a page lands the same way whoever moved it.

use rekall_kernel::{Id, Instant, Result};
use rekall_store::repository as repo;
use rekall_store::{project_section, section_context};
use sea_orm::ConnectionTrait;

/// One context of an arrangement, with its pages in order.
#[derive(Clone, Debug)]
pub struct Placement {
    pub context_id: Id,
    pub section_ids: Vec<Id>,
}

/// Contexts take the order of `placements` and each page the context and place it is named in.
/// Every id has to belong to `contexts` and `sections`, which the caller has already checked.
/// Returns how many pages changed context.
pub async fn apply(
    db: &impl ConnectionTrait,
    contexts: &[section_context::Model],
    sections: &[project_section::Model],
    placements: &[Placement],
    now: Instant,
) -> Result<usize> {
    let mut pages_moved = 0;
    for (context_position, placement) in placements.iter().enumerate() {
        place_context(db, contexts, placement, context_position as i32, now).await?;
        pages_moved += place_sections(db, sections, placement, now).await?;
    }
    Ok(pages_moved)
}

async fn place_context(
    db: &impl ConnectionTrait,
    contexts: &[section_context::Model],
    placement: &Placement,
    position: i32,
    now: Instant,
) -> Result<()> {
    let before = known(contexts, |context| context.id, placement.context_id);
    let after = section_context::Model { position, ..before.clone() };
    repo::update_if_changed(db, &before, after, now).await?;
    Ok(())
}

async fn place_sections(db: &impl ConnectionTrait, sections: &[project_section::Model], placement: &Placement, now: Instant) -> Result<usize> {
    let mut moved = 0;
    for (position, section_id) in placement.section_ids.iter().enumerate() {
        let before = known(sections, |section| section.id, *section_id);
        if before.context_id != placement.context_id {
            moved += 1;
        }
        let after = project_section::Model { context_id: placement.context_id, position: position as i32, ..before.clone() };
        repo::update_if_changed(db, &before, after, now).await?;
    }
    Ok(moved)
}

fn known<T: Clone>(records: &[T], id_of: impl Fn(&T) -> Id, id: Id) -> T {
    records.iter().find(|record| id_of(record) == id).cloned().expect("an arrangement was checked to name only the project's own records")
}
