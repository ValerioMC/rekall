//! A company's connections to GitLab, Kubernetes, Jira, Confluence and Grafana, their credentials (written by one service, read back by another), the
//! projects that bind them, the repositories a project binds by discovery, their access log, and
//! the Jira issues a project imports as tasks, the Confluence space it is filled from, and the Jira project or Confluence space a binding's target is checked against. Console-only: a session sees a project's connections
//! through its context and never reaches these services.

mod command;
mod rule;
mod service;
mod support;
mod view;

pub use command::connection_access_page_command::ConnectionAccessPageCommand;
pub use service::connection_access_query_service::ConnectionAccessQueryService;
pub use view::connection_access_view::ConnectionAccessView;
pub use command::confluence_import_command::ConfluenceImportCommand;
pub use service::confluence_space_service::ConfluenceSpaceService;
pub use view::confluence_import_view::{ConfluenceImportView, ConfluenceSectionView};
pub use command::connection_command::ConnectionCommand;
pub use command::connection_credential_command::ConnectionCredentialCommand;
pub use service::connection_credential_reveal_service::ConnectionCredentialRevealService;
pub use service::connection_credential_service::ConnectionCredentialService;
pub use view::connection_company_view::ConnectionCompanyView;
pub use view::connection_detail_view::ConnectionDetailView;
pub use view::connection_project_view::ConnectionProjectView;
pub use view::connection_removed_view::ConnectionRemovedView;
pub use service::connection_service::ConnectionService;
pub use service::connection_target_service::ConnectionTargetService;
pub use service::gitlab_target_service::GitlabTargetService;
pub use view::connection_view::ConnectionView;
pub use service::credential_hand_out_service::CredentialHandOutService;
pub use view::discovered_repository_view::DiscoveredRepositoryView;
pub use view::discovery_outcome::DiscoveryOutcome;
pub use command::jira_import_command::JiraImportCommand;
pub use service::jira_issue_service::JiraIssueService;
pub use service::task_tracker_sync_service::TaskTrackerSyncService;
pub use view::jira_issue_view::JiraIssueView;
pub use view::kube_context_view::KubeContextView;
pub use service::kubeconfig_preview_service::KubeconfigPreviewService;
pub use command::project_connection_command::ProjectConnectionCommand;
pub use service::project_connection_service::ProjectConnectionService;
pub use view::project_connection_view::ProjectConnectionView;
pub use service::repository_discovery_service::RepositoryDiscoveryService;
pub use view::repository_discovery_view::RepositoryDiscoveryView;
pub use view::gitlab_namespace_view::GitlabNamespaceView;
pub use view::target_check_view::TargetCheckView;
pub use view::target_suggestion_view::TargetSuggestionView;
pub use view::revealed_credential_view::RevealedCredentialView;
