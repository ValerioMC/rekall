//! A project filled from the Confluence space its binding names: the space's home page becomes the
//! blueprint, its description the project's, and its other pages the sections of their concepts.
//! Confluence is asked before any transaction opens.

use std::sync::Arc;

use rekall_core::connection::{ConfluenceSpaces, SecretStore};
use rekall_core::{in_write, Ctx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::ConnectionKind;

use crate::catalog::ProjectService;
use crate::connection::rule::confluence_grouping_rules::project_content;
use crate::connection::rule::confluence_rules::{space_key_of, PAGE_LIMIT};
use crate::connection::support::atlassian_access::{access_of, refused_by};
use crate::connection::view::confluence_import_view::{ConfluenceImportView, ConfluenceSectionView};
use crate::connection::{ConfluenceImportCommand, ProjectConnectionService};
use crate::section::ProjectSectionCatalogService;

#[derive(Clone)]
pub struct ConfluenceSpaceService {
    ctx: Ctx,
    bindings: ProjectConnectionService,
    projects: ProjectService,
    sections: ProjectSectionCatalogService,
    secrets: Arc<dyn SecretStore>,
    confluence: Arc<dyn ConfluenceSpaces>,
}

impl ConfluenceSpaceService {
    pub fn new(
        ctx: Ctx,
        bindings: ProjectConnectionService,
        projects: ProjectService,
        sections: ProjectSectionCatalogService,
        secrets: Arc<dyn SecretStore>,
        confluence: Arc<dyn ConfluenceSpaces>,
    ) -> Self {
        Self { ctx, bindings, projects, sections, secrets, confluence }
    }

    pub async fn import(&self, project_id: Id, binding_id: Id, command: ConfluenceImportCommand) -> Result<ConfluenceImportView> {
        let (binding, connection) = self.bindings.binding(project_id, binding_id).await?;
        if connection.kind != ConnectionKind::Confluence {
            return Err(RekallError::illegal("Only a Confluence connection has a space to fill a project from."));
        }
        let space_key = space_key_of(binding.target.as_deref())?;
        let access = access_of(self.secrets.clone(), &connection, "Confluence").await?;
        let space = self
            .confluence
            .space(&access.site, &access.credential, &space_key)
            .await
            .map_err(|failure| refused_by("Confluence", failure))?
            .ok_or_else(|| RekallError::illegal(format!("Confluence has no space '{space_key}'.")))?;
        let pages = self.confluence.pages(&access.site, &access.credential, &space_key, PAGE_LIMIT).await.map_err(|failure| refused_by("Confluence", failure))?;
        let content = project_content(&space, &pages);
        in_write!(&self.ctx, |tx| {
            let texts = self.projects.fill_texts_in(&tx, project_id, content.description.clone(), content.blueprint.clone(), command.replace).await?;
            let mut sections = Vec::with_capacity(content.sections.len());
            for draft in &content.sections {
                let outcome = self.sections.import_in(&mut tx, project_id, &draft.title, draft.body.clone(), command.replace).await?;
                sections.push(ConfluenceSectionView { title: draft.title.clone(), pages: draft.pages, outcome });
            }
            Ok::<_, RekallError>(ConfluenceImportView {
                space_key: space.key.clone(),
                space_name: space.name.clone(),
                pages_read: pages.len(),
                description_written: texts.description,
                blueprint_written: texts.blueprint,
                sections,
            })
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/confluence_space_service_tests.rs"]
mod tests;
