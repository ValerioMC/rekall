//! The HTTP side of a company's connections, their credentials, the projects that bind them and
//! their access log, served by `rekall_console::connection`; and, still here, the repositories a
//! project binds by discovery and the Jira issues it imports as tasks.

mod connection_access_query;
mod connection_access_response;
mod connection_controller;
mod connection_credential_controller;
mod connection_credential_request;
mod connection_detail_response;
mod connection_project_response;
mod connection_removed_response;
mod connection_request;
mod connection_response;
mod discovered_repository_response;
mod discovery_outcome;
mod jira_import_request;
mod jira_issue_controller;
mod jira_issue_response;
mod jira_issue_service;
mod jira_rules;
mod kube_context_response;
mod kubeconfig_preview_request;
mod project_connection_controller;
mod project_connection_request;
mod project_connection_response;
mod repository_discovery_response;
mod repository_discovery_service;
mod repository_folders;
mod repository_origin;
mod routes;

pub use connection_access_query::ConnectionAccessQuery;
pub use connection_access_response::ConnectionAccessResponse;
pub use connection_credential_request::ConnectionCredentialRequest;
pub use connection_detail_response::ConnectionDetailResponse;
pub use connection_project_response::ConnectionProjectResponse;
pub use connection_removed_response::ConnectionRemovedResponse;
pub use connection_request::ConnectionRequest;
pub use connection_response::ConnectionResponse;
pub use discovered_repository_response::DiscoveredRepositoryResponse;
pub use discovery_outcome::DiscoveryOutcome;
pub use jira_import_request::JiraImportRequest;
pub use jira_issue_response::JiraIssueResponse;
pub use jira_issue_service::JiraIssueService;
pub use kube_context_response::KubeContextResponse;
pub use kubeconfig_preview_request::KubeconfigPreviewRequest;
pub use project_connection_request::ProjectConnectionRequest;
pub use project_connection_response::ProjectConnectionResponse;
pub use repository_discovery_response::RepositoryDiscoveryResponse;
pub use repository_discovery_service::RepositoryDiscoveryService;
pub(crate) use routes::routes;
