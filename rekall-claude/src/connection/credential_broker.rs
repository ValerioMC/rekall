use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rekall_common::{cleanup, Id, InternalFailure, RekallError, Result};
use rekall_model::ConnectionKind;
use rekall_service::claude::{TerminalConnection, TerminalLaunch};
use rekall_service::connection::{ConnectionAccessService, JiraCredential, KubeAccess, Secret, SecretStore};
use tracing::{info, warn};

use super::credential_session::CredentialSession;
use super::guard_rails::settings_document;
use super::helper_scripts::{api_wrapper, git_helper, kube_plugin, GITLAB_API, GIT_HELPER, JIRA_API, KUBE_PLUGIN};
use super::session_kubeconfig::{exec_credential, session_kubeconfig, SessionCluster};
use super::{CredentialPurpose, HandOut, Refusal, SessionEnvironment};

const KUBECONFIG_FILE: &str = "kubeconfig";
const SETTINGS_FILE: &str = "claude-settings.json";

/// Opens a credential session for every terminal whose project binds a connection with a stored
/// credential, answers its helpers, and closes it with the terminal. The only reader of credentials
/// outside the console's own services.
pub struct CredentialBroker {
    secrets: Arc<dyn SecretStore>,
    access: ConnectionAccessService,
    /// Where the helpers ask, `http://127.0.0.1:<port>`.
    endpoint: String,
    root: PathBuf,
    sessions: Mutex<HashMap<Id, Arc<CredentialSession>>>,
}

impl CredentialBroker {
    /// `root` holds one folder per live session; anything left in it by a Rekall that did not stop cleanly is removed.
    pub fn new(secrets: Arc<dyn SecretStore>, access: ConnectionAccessService, endpoint: String, root: PathBuf) -> Self {
        cleanup::remove_dir_all(&root);
        Self { secrets, access, endpoint, root, sessions: Mutex::new(HashMap::new()) }
    }

    /// What to add to the terminal's environment, or nothing when no bound connection has a
    /// credential. The helpers are shell scripts, so a Windows terminal gets none.
    pub async fn open(&self, terminal_id: Id, launch: &TerminalLaunch) -> Result<Option<SessionEnvironment>> {
        let usable: Vec<TerminalConnection> = launch.connections.iter().filter(|connection| connection.credential_set).cloned().collect();
        if usable.is_empty() || !cfg!(unix) {
            return Ok(None);
        }
        let folder = self.root.join(terminal_id.to_string());
        create_private_folder(&folder)?;
        // Claude Code matches its deny rules against the real path, so a symlinked root is resolved.
        let folder = folder.canonicalize().unwrap_or(folder);
        let environment = match self.write_helpers(terminal_id, &folder, &usable).await {
            Ok(environment) => environment,
            Err(failure) => {
                cleanup::remove_dir_all(&folder);
                return Err(failure);
            }
        };
        let token = environment.token.clone().expect("every session environment carries its token");
        let session = CredentialSession { task_id: launch.task_id, token: Secret::new(token), folder, connections: usable };
        self.sessions().insert(terminal_id, Arc::new(session));
        info!(%terminal_id, "Opened a credential session");
        Ok(Some(environment))
    }

    /// Ends the session: its token stops working and its folder is removed. Idempotent.
    pub fn close(&self, terminal_id: Id) {
        if let Some(session) = self.sessions().remove(&terminal_id) {
            cleanup::remove_dir_all(&session.folder);
            info!(%terminal_id, "Closed a credential session");
        }
    }

    pub fn has_session(&self, terminal_id: Id) -> bool {
        self.sessions().contains_key(&terminal_id)
    }

    /// The credential of `label` in the shape `purpose` reads, for a live terminal presenting its own
    /// token. Every request about a connection the terminal binds is logged, granted or refused;
    /// a credential is handed out only once its line is written.
    pub async fn hand_out(&self, terminal_id: Id, presented: Option<&str>, label: &str, purpose: CredentialPurpose) -> HandOut {
        let Some(session) = self.sessions().get(&terminal_id).cloned() else {
            warn!(%terminal_id, label, "A credential was asked for by a terminal with no credential session");
            return HandOut::refused(Refusal::Missing, "This terminal has no credentials: it has ended, or its project binds no connection with one.");
        };
        let Some(connection) = session.connection(label).cloned() else {
            warn!(%terminal_id, label, "A credential was asked for a connection the terminal's project does not bind");
            return HandOut::refused(Refusal::Missing, format!("This terminal's project uses no connection labelled '{label}'."));
        };
        if !session.accepts(presented) {
            return self.refuse(&session, &connection, terminal_id, Refusal::Forbidden, "the token is not this terminal's").await;
        }
        if !purpose.serves(connection.kind) {
            return self.refuse(&session, &connection, terminal_id, Refusal::WrongPurpose, "the helper asked for a shape this kind of connection does not have").await;
        }
        let body = match self.answer(&connection, purpose).await {
            Ok(body) => body,
            Err((refusal, why)) => return self.refuse(&session, &connection, terminal_id, refusal, &why).await,
        };
        if let Err(failure) = self.access.record_granted(connection.connection_id, session.task_id, terminal_id).await {
            warn!(%failure, %terminal_id, label, "A credential was not handed out because its access could not be logged");
            return HandOut::refused(Refusal::Failed, "Rekall could not log this access, so it handed nothing out.");
        }
        let content_type = if purpose == CredentialPurpose::Kubectl { "application/json" } else { "text/plain; charset=utf-8" };
        HandOut::Granted { body, content_type }
    }

    async fn refuse(&self, session: &CredentialSession, connection: &TerminalConnection, terminal_id: Id, refusal: Refusal, why: &str) -> HandOut {
        if let Err(failure) = self.access.record_refused(connection.connection_id, session.task_id, terminal_id, why).await {
            warn!(%failure, %terminal_id, label = connection.label, "A refused credential request could not be logged");
        }
        HandOut::refused(refusal, format!("Rekall refused the credential of '{}': {why}.", connection.label))
    }

    /// The stored credential written the way `purpose` reads it.
    async fn answer(&self, connection: &TerminalConnection, purpose: CredentialPurpose) -> std::result::Result<Secret, (Refusal, String)> {
        let stored = match self.read(&connection.secret_ref).await {
            Ok(stored) => stored,
            Err(RekallError::NotFound(_)) => return Err((Refusal::Missing, "no credential is stored for it; set one on the company's Connections page".into())),
            Err(failure) => return Err((Refusal::Failed, format!("the OS credential store failed ({})", failure.message()))),
        };
        match purpose {
            CredentialPurpose::Kubectl => {
                let access = KubeAccess::from_stored(&stored).map_err(|failure| (Refusal::Failed, failure.message().to_string()))?;
                exec_credential(&access).map(Secret::new).map_err(|why| (Refusal::Failed, why))
            }
            CredentialPurpose::Git => Ok(Secret::new(format!("username=oauth2\npassword={}\n", stored.expose()))),
            CredentialPurpose::Gitlab => Ok(Secret::new(format!("PRIVATE-TOKEN: {}\n", stored.expose()))),
            CredentialPurpose::Jira => {
                let credential = JiraCredential::from_stored(&stored).map_err(|failure| (Refusal::Failed, failure.message().to_string()))?;
                Ok(Secret::new(format!("Authorization: {}\n", credential.authorization().expose())))
            }
        }
    }

    /// Writes the helpers into `folder` and answers the environment that points at them.
    async fn write_helpers(&self, terminal_id: Id, folder: &Path, connections: &[TerminalConnection]) -> Result<SessionEnvironment> {
        let endpoint = format!("{}/api/terminal/{terminal_id}/credentials", self.endpoint);
        let settings = folder.join(SETTINGS_FILE);
        write_private(&settings, &settings_document(folder, &endpoint))?;
        let mut environment = SessionEnvironment {
            path_prefix: Some(folder.to_path_buf()),
            token: Some(new_token()),
            claude_settings: Some(settings),
            ..SessionEnvironment::default()
        };
        let clusters = self.clusters_of(connections).await;
        if !clusters.is_empty() {
            let plugin = folder.join(KUBE_PLUGIN);
            write_script(&plugin, &kube_plugin(&endpoint))?;
            let sessions: Vec<SessionCluster<'_>> =
                clusters.iter().map(|(connection, access)| SessionCluster { label: &connection.label, access, namespace: connection.target.as_deref() }).collect();
            let kubeconfig = folder.join(KUBECONFIG_FILE);
            write_private(&kubeconfig, &session_kubeconfig(&sessions, &plugin))?;
            environment.kubeconfig = Some(kubeconfig);
        }
        let gitlabs: Vec<&TerminalConnection> = connections.iter().filter(|connection| connection.kind == ConnectionKind::Gitlab).collect();
        if !gitlabs.is_empty() {
            let helper = folder.join(GIT_HELPER);
            write_script(&helper, &git_helper(&endpoint))?;
            write_script(&folder.join(GITLAB_API.name), &api_wrapper(&GITLAB_API, &endpoint, &gitlabs))?;
            environment.git_config = git_config(&gitlabs, &helper);
        }
        let jiras: Vec<&TerminalConnection> = connections.iter().filter(|connection| connection.kind == ConnectionKind::Jira).collect();
        if !jiras.is_empty() {
            write_script(&folder.join(JIRA_API.name), &api_wrapper(&JIRA_API, &endpoint, &jiras))?;
        }
        Ok(environment)
    }

    /// Each cluster with what its stored kubeconfig says of it. One that no longer reads is left
    /// out and logged: its `kubectl --context` then fails plainly instead of the whole terminal.
    async fn clusters_of<'a>(&self, connections: &'a [TerminalConnection]) -> Vec<(&'a TerminalConnection, KubeAccess)> {
        let mut clusters = Vec::new();
        for connection in connections.iter().filter(|connection| connection.kind == ConnectionKind::Kubernetes) {
            match self.read(&connection.secret_ref).await.and_then(|stored| KubeAccess::from_stored(&stored)) {
                Ok(access) => clusters.push((connection, access)),
                Err(failure) => warn!(%failure, label = connection.label, "A cluster's stored kubeconfig could not be read, so the terminal cannot reach it"),
            }
        }
        clusters
    }

    /// The keychain may block on a prompt, so it is read off the async runtime.
    async fn read(&self, account: &str) -> Result<Secret> {
        let secrets = self.secrets.clone();
        let account = account.to_string();
        tokio::task::spawn_blocking(move || secrets.read(&account)).await.map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?
    }

    fn sessions(&self) -> MutexGuard<'_, HashMap<Id, Arc<CredentialSession>>> {
        self.sessions.lock().expect("never poisoned")
    }
}

/// For each GitLab host: an empty helper that drops any configured elsewhere, then Rekall's. Several
/// bindings of one instance share its token, so the first one answers for the host.
fn git_config(gitlabs: &[&TerminalConnection], helper: &Path) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for connection in gitlabs {
        let Some(instance) = connection.instance_url.as_deref() else { continue };
        let key = format!("credential.{}.helper", instance.trim_end_matches('/'));
        if entries.iter().any(|(known, _)| *known == key) {
            continue;
        }
        entries.push((key.clone(), String::new()));
        entries.push((key, format!("!'{}' {}", helper.display(), connection.label)));
    }
    entries
}

/// 244 random bits: two version-4 UUIDs without their dashes.
fn new_token() -> String {
    format!("{}{}", Id::random(), Id::random()).replace('-', "")
}

fn create_private_folder(folder: &Path) -> Result<()> {
    std::fs::create_dir_all(folder).map_err(|failure| RekallError::internal(InternalFailure::Io, format!("could not create {}: {failure}", folder.display())))?;
    set_mode(folder, 0o700)
}

fn write_script(path: &Path, text: &str) -> Result<()> {
    write_private(path, text)?;
    set_mode(path, 0o700)
}

fn write_private(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text).map_err(|failure| RekallError::internal(InternalFailure::Io, format!("could not write {}: {failure}", path.display())))?;
    set_mode(path, 0o600)
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|failure| RekallError::internal(InternalFailure::Io, format!("could not restrict {}: {failure}", path.display())))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/connection/credential_broker_tests.rs"]
mod tests;
