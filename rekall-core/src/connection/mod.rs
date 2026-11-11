//! A company's connections as Rekall's services see them: the view a context names them by, the
//! credential type that never prints, the store that keeps it outside the database, the kubeconfig
//! a Kubernetes credential arrives as, and the Jira credential and issues a Jira connection reads, and the task manager a project writes its tasks to.

mod event;
mod model;
mod port;
mod service;
mod view;

pub use service::connection_access_service::ConnectionAccessService;
pub use model::connection_check::ConnectionCheck;
pub use service::connection_check_service::ConnectionCheckService;
pub use port::connection_probe::ConnectionProbe;
pub use event::connection_stream_event::ConnectionStreamEvent;
pub use view::connection_view::ConnectionView;
pub use port::credential_desk::CredentialDesk;
pub use event::credential_access_stream_event::CredentialAccessStreamEvent;
pub use model::credential_purpose::CredentialPurpose;
pub use model::github_api_root::github_api_root;
pub use model::hand_out::{HandOut, Refusal};
pub use port::in_memory_secret_store::InMemorySecretStore;
pub use model::grafana_credential::GrafanaCredential;
pub use model::grafana_instance::GrafanaInstance;
pub use model::jira_credential::JiraCredential;
pub use model::jira_deployment::JiraDeployment;
pub use model::jira_failure::JiraFailure;
pub use model::jira_issue::JiraIssue;
pub use port::jira_issues::JiraIssues;
pub use model::jira_site::JiraSite;
pub use model::kube_access::KubeAccess;
pub use model::kube_context_summary::KubeContextSummary;
pub use model::kube_credential::KubeCredential;
pub use model::kube_user_kind::KubeUserKind;
pub use model::kubeconfig::Kubeconfig;
pub use model::probe_outcome::ProbeOutcome;
pub use port::migrating_secret_store::MigratingSecretStore;
pub use port::remembering_secret_store::RememberingSecretStore;
pub use model::secret::Secret;
pub use port::secret_store::SecretStore;
pub use port::scripted_task_tracker::ScriptedTaskTracker;
pub use port::task_tracker::TaskTracker;
pub use service::task_tracker_service::TaskTrackerService;
pub use model::tracker_endpoint::TrackerEndpoint;
pub use model::tracker_failure::TrackerFailure;
pub use model::tracker_issue::TrackerIssue;
pub use model::tracker_link::TrackerLink;
