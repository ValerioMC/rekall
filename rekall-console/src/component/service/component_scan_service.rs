//! Makes a component of every git repository under a project's folder, described by what the
//! repository says it does. A repository whose folder already has a component is left as it is.

use std::path::{Path, PathBuf};

use rekall_kernel::{Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_core::{in_read, in_write, Ctx};

use crate::component::support::component_description::describe;
use crate::component::support::repository_folders::{project_root, repositories_under};
use crate::component::{ComponentScanView, ProjectComponentCatalogService};

#[derive(Clone)]
pub struct ComponentScanService {
    ctx: Ctx,
    components: ProjectComponentCatalogService,
}

struct FoundRepository {
    folder: String,
    description: String,
}

impl ComponentScanService {
    pub fn new(ctx: Ctx, components: ProjectComponentCatalogService) -> Self {
        Self { ctx, components }
    }

    /// The folder is walked and each repository read before the transaction opens.
    pub async fn scan(&self, project_id: Id) -> Result<ComponentScanView> {
        let root = self.root_of(project_id).await?;
        let found = found_under(&root).await?;
        in_write!(&self.ctx, |tx| {
            let mut added = Vec::new();
            for repository in &found {
                added.extend(self.components.add_for_folder_in(&tx, project_id, &repository.folder, &repository.description).await?);
            }
            let already_known = found.len() - added.len();
            Ok::<_, RekallError>(ComponentScanView { folder: root.display().to_string(), added, already_known })
        })
    }

    async fn root_of(&self, project_id: Id) -> Result<PathBuf> {
        let project = in_read!(&self.ctx, |tx| repo::project::require(tx.db(), project_id).await)?;
        project_root(project.repo_folder.as_deref())
    }
}

async fn found_under(root: &Path) -> Result<Vec<FoundRepository>> {
    let walked = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        repositories_under(&walked)
            .into_iter()
            .map(|folder| FoundRepository { description: describe(&folder), folder: folder.display().to_string() })
            .collect()
    })
    .await
    .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))
}
