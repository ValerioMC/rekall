//! Where a person puts a project's pages: which context holds each and in what order, and the
//! order of the contexts. One call replaces the whole arrangement, so a drag is all or nothing.

use rekall_core::section::rule::placement_rules;
use rekall_core::section::{ProjectSectionView, SectionContextView, SectionStreamEvent};
use rekall_core::{in_write, Ctx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use crate::section::rule::section_layout_rules::{placements_of, require_whole};
use crate::section::{SectionLayoutCommand, SectionLayoutView};

#[derive(Clone)]
pub struct SectionLayoutService {
    ctx: Ctx,
}

impl SectionLayoutService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn arrange(&self, project_id: Id, command: SectionLayoutCommand) -> Result<SectionLayoutView> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let contexts = repo::section_context::list_of_project(tx.db(), project_id).await?;
            let sections = repo::project_section::list_of_project(tx.db(), project_id).await?;
            let placements = placements_of(&command)?;
            require_whole(&placements, &contexts, &sections)?;
            placement_rules::apply(tx.db(), &contexts, &sections, &placements, self.ctx.now()).await?;
            tx.publish(SectionStreamEvent { project_id });
            let contexts = repo::section_context::list_of_project(tx.db(), project_id).await?;
            let sections = repo::project_section::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(SectionLayoutView {
                contexts: contexts.iter().map(SectionContextView::of).collect(),
                sections: sections.iter().map(ProjectSectionView::of).collect(),
            })
        })
    }
}
