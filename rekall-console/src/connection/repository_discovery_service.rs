//! Binds every repository under a project's folder to the GitLab connection its `origin` points
//! at, and makes each a component. Only a connection with a stored credential is bound: a token
//! configured once for an instance reaches every repository on it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rekall_kernel::{jstr, Id, InternalFailure, RekallError, Result};
use rekall_store::{external_connection, project_connection, ConnectionKind, Slug};
use rekall_core::commit::GitRepositoryInspector;
use rekall_core::{in_write, Ctx, Tx};

use super::connection_rules::first_free_label;
use super::repository_folders::repositories_under;
use super::repository_origin::RepositoryOrigin;
use super::{ConnectionService, DiscoveredRepositoryView, DiscoveryOutcome, ProjectConnectionService, RepositoryDiscoveryView};
use crate::catalog::ProjectService;
use crate::component::{folder_name, ProjectComponentCatalogService};

#[derive(Clone)]
pub struct RepositoryDiscoveryService {
    ctx: Ctx,
    inspector: GitRepositoryInspector,
    projects: ProjectService,
    connections: ConnectionService,
    bindings: ProjectConnectionService,
    components: ProjectComponentCatalogService,
}

struct FoundRepository {
    folder: String,
    origin: Option<RepositoryOrigin>,
}

/// What one discovery has to remember from repository to repository.
struct DiscoveryRun {
    project_id: Id,
    gitlabs: Vec<external_connection::Model>,
    bindings: Vec<(project_connection::Model, external_connection::Model)>,
    /// The GitLab projects a repository of this run already settled, so a second clone of one
    /// gets no component of its own.
    settled: HashSet<(Id, String)>,
}

impl RepositoryDiscoveryService {
    pub fn new(
        ctx: Ctx,
        inspector: GitRepositoryInspector,
        projects: ProjectService,
        connections: ConnectionService,
        bindings: ProjectConnectionService,
        components: ProjectComponentCatalogService,
    ) -> Self {
        Self { ctx, inspector, projects, connections, bindings, components }
    }

    /// The folder is walked and each origin read before the transaction opens.
    pub async fn discover(&self, project_id: Id) -> Result<RepositoryDiscoveryView> {
        let project = self.projects.get(project_id).await?;
        let root = root_folder(project.repo_folder.as_deref())?;
        let found = self.found_under(&root).await?;
        in_write!(&self.ctx, |tx| {
            let gitlabs = self.connections.list_in(&tx, project.company_id).await?.into_iter().filter(|connection| connection.kind == ConnectionKind::Gitlab).collect();
            let bindings = self.bindings.list_in(&tx, project_id).await?;
            let mut run = DiscoveryRun { project_id, gitlabs, bindings, settled: HashSet::new() };
            let mut repositories = Vec::with_capacity(found.len());
            for repository in &found {
                repositories.push(self.settle_in(&mut tx, &mut run, repository).await?);
            }
            Ok::<_, RekallError>(RepositoryDiscoveryView { folder: root.display().to_string(), repositories })
        })
    }

    async fn found_under(&self, root: &Path) -> Result<Vec<FoundRepository>> {
        let walked = root.to_path_buf();
        let folders = tokio::task::spawn_blocking(move || repositories_under(&walked))
            .await
            .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?;
        let mut found = Vec::with_capacity(folders.len());
        for folder in folders {
            let origin = self.inspector.origin_url(&folder).await.as_deref().and_then(RepositoryOrigin::parse);
            found.push(FoundRepository { folder: folder.display().to_string(), origin });
        }
        Ok(found)
    }

    async fn settle_in(&self, tx: &mut Tx, run: &mut DiscoveryRun, repository: &FoundRepository) -> Result<DiscoveredRepositoryView> {
        let folder = repository.folder.clone();
        let Some(origin) = &repository.origin else {
            return Ok(DiscoveredRepositoryView::unmatched(folder, None, DiscoveryOutcome::NoOrigin, None));
        };
        let Some((connection, target)) = connection_at(&run.gitlabs, origin) else {
            return Ok(DiscoveredRepositoryView::unmatched(folder, Some(origin.display()), DiscoveryOutcome::NoConnection, None));
        };
        if !connection.has_credential() {
            return Ok(DiscoveredRepositoryView::unmatched(folder, Some(origin.display()), DiscoveryOutcome::NoCredential, Some(connection.label)));
        }
        let first_clone = run.settled.insert((connection.id, target.clone()));
        let (outcome, binding) = match bound_to(&run.bindings, &connection, &target) {
            Some(binding) => (DiscoveryOutcome::AlreadyBound, binding),
            None => (DiscoveryOutcome::Bound, self.bind_in(tx, run, &connection, &folder, &target).await?),
        };
        let component = if first_clone { self.components.add_for_folder_in(tx, run.project_id, &folder).await?.map(|component| component.name) } else { None };
        Ok(DiscoveredRepositoryView {
            folder,
            origin: Some(origin.display()),
            outcome,
            connection_label: Some(connection.label),
            binding_id: Some(binding.id),
            label: Some(binding.label),
            target: Some(target),
            component,
        })
    }

    /// A new binding labelled after the repository's folder, or after the connection when the
    /// folder's name leaves no usable label.
    async fn bind_in(&self, tx: &mut Tx, run: &mut DiscoveryRun, connection: &external_connection::Model, folder: &str, target: &str) -> Result<project_connection::Model> {
        let wanted = Slug::of(Some(&folder_name(folder))).unwrap_or_else(|_| connection.label.clone());
        let taken: Vec<String> = run.bindings.iter().map(|(binding, _)| binding.label.clone()).collect();
        let label = first_free_label(&wanted, &taken);
        let (binding, connection) = self.bindings.bind_in(tx, run.project_id, connection.id, Some(&label), Some(target.to_string())).await?;
        run.bindings.push((binding.clone(), connection));
        Ok(binding)
    }
}

fn root_folder(repo_folder: Option<&str>) -> Result<PathBuf> {
    let Some(folder) = repo_folder.filter(|folder| !jstr::is_blank(folder)) else {
        return Err(RekallError::illegal("This project has no folder yet. Set it before looking for its repositories."));
    };
    let root = PathBuf::from(jstr::strip(folder));
    if !root.is_dir() {
        return Err(RekallError::illegal(format!("{} is not a folder on this machine.", root.display())));
    }
    Ok(root)
}

/// The GitLab connection at the origin's host and the project path on it. One with a credential is
/// preferred, so a second, empty connection to the same instance does not hide a usable one.
fn connection_at(gitlabs: &[external_connection::Model], origin: &RepositoryOrigin) -> Option<(external_connection::Model, String)> {
    let candidates: Vec<(&external_connection::Model, String)> = gitlabs
        .iter()
        .filter_map(|connection| connection.base_url.as_deref().and_then(|url| origin.path_on(url)).map(|path| (connection, path)))
        .collect();
    let chosen = candidates.iter().find(|(connection, _)| connection.has_credential()).or(candidates.first())?;
    Some((chosen.0.clone(), chosen.1.clone()))
}

fn bound_to(bindings: &[(project_connection::Model, external_connection::Model)], connection: &external_connection::Model, target: &str) -> Option<project_connection::Model> {
    bindings.iter().find(|(binding, _)| binding.connection_id == connection.id && binding.target.as_deref() == Some(target)).map(|(binding, _)| binding.clone())
}
