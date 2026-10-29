//! The routes of a company's connections, their credentials, the projects that bind them, the
//! repositories a project binds by discovery, their access log and the Jira issues a project
//! imports as tasks, served by `rekall_console::connection`.

mod controller;
mod request;
mod response;
mod routes;

pub use request::connection_access_query::ConnectionAccessQuery;
pub use response::connection_access_response::ConnectionAccessResponse;
pub use request::connection_credential_request::ConnectionCredentialRequest;
pub use response::connection_detail_response::ConnectionDetailResponse;
pub use response::connection_project_response::ConnectionProjectResponse;
pub use response::connection_removed_response::ConnectionRemovedResponse;
pub use request::connection_request::ConnectionRequest;
pub use response::connection_response::ConnectionResponse;
pub use response::discovered_repository_response::DiscoveredRepositoryResponse;
pub use response::discovery_outcome_response::DiscoveryOutcomeResponse;
pub use request::jira_import_request::JiraImportRequest;
pub use response::jira_issue_response::JiraIssueResponse;
pub use response::kube_context_response::KubeContextResponse;
pub use request::kubeconfig_preview_request::KubeconfigPreviewRequest;
pub use request::project_connection_request::ProjectConnectionRequest;
pub use response::project_connection_response::ProjectConnectionResponse;
pub use response::repository_discovery_response::RepositoryDiscoveryResponse;
pub(crate) use controller::credential_controller::routes as credential_routes;
pub(crate) use routes::routes;
