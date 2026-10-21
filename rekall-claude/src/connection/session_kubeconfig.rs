//! The kubeconfig a credential session writes: one context per cluster, named by its label, whose
//! user is Rekall's exec plugin. Written as JSON, which kubectl reads as YAML. No credential in it.

use std::path::Path;

use rekall_service::connection::{KubeAccess, KubeCredential};
use serde::Serialize;

const EXEC_API_VERSION: &str = "client.authentication.k8s.io/v1";

#[derive(Serialize)]
struct Document<'a> {
    #[serde(rename = "apiVersion")]
    api_version: &'static str,
    kind: &'static str,
    #[serde(rename = "current-context", skip_serializing_if = "Option::is_none")]
    current_context: Option<&'a str>,
    clusters: Vec<Named<'a, Cluster<'a>>>,
    contexts: Vec<Named<'a, Context<'a>>>,
    users: Vec<Named<'a, User<'a>>>,
}

#[derive(Serialize)]
struct Named<'a, T> {
    name: &'a str,
    #[serde(flatten)]
    item: T,
}

#[derive(Serialize)]
struct Cluster<'a> {
    cluster: ClusterFields<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ClusterFields<'a> {
    server: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    certificate_authority_data: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tls_server_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_url: Option<&'a str>,
}

#[derive(Serialize)]
struct Context<'a> {
    context: ContextFields<'a>,
}

#[derive(Serialize)]
struct ContextFields<'a> {
    cluster: &'a str,
    user: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<&'a str>,
}

#[derive(Serialize)]
struct User<'a> {
    user: UserFields<'a>,
}

#[derive(Serialize)]
struct UserFields<'a> {
    exec: Exec<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Exec<'a> {
    api_version: &'static str,
    command: String,
    args: Vec<&'a str>,
    interactive_mode: &'static str,
    provide_cluster_info: bool,
}

/// One cluster the session reaches: its label, what the stored kubeconfig says of it, and the
/// namespace the project's binding narrows it to.
pub(super) struct SessionCluster<'a> {
    pub label: &'a str,
    pub access: &'a KubeAccess,
    pub namespace: Option<&'a str>,
}

pub(super) fn session_kubeconfig(clusters: &[SessionCluster<'_>], plugin: &Path) -> String {
    let document = Document {
        api_version: "v1",
        kind: "Config",
        current_context: clusters.first().map(|cluster| cluster.label),
        clusters: clusters
            .iter()
            .map(|cluster| Named {
                name: cluster.label,
                item: Cluster {
                    cluster: ClusterFields {
                        server: &cluster.access.server,
                        certificate_authority_data: cluster.access.certificate_authority.as_deref(),
                        tls_server_name: cluster.access.tls_server_name.as_deref(),
                        proxy_url: cluster.access.proxy_url.as_deref(),
                    },
                },
            })
            .collect(),
        contexts: clusters
            .iter()
            .map(|cluster| Named {
                name: cluster.label,
                item: Context {
                    context: ContextFields { cluster: cluster.label, user: cluster.label, namespace: cluster.namespace.or(cluster.access.namespace.as_deref()) },
                },
            })
            .collect(),
        users: clusters
            .iter()
            .map(|cluster| Named {
                name: cluster.label,
                item: User {
                    user: UserFields {
                        exec: Exec {
                            api_version: EXEC_API_VERSION,
                            command: plugin.display().to_string(),
                            args: vec![cluster.label],
                            interactive_mode: "Never",
                            provide_cluster_info: false,
                        },
                    },
                },
            })
            .collect(),
    };
    serde_json::to_string_pretty(&document).expect("a kubeconfig of plain strings always serialises")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecCredential<'a> {
    api_version: &'static str,
    kind: &'static str,
    status: ExecStatus<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecStatus<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_certificate_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_key_data: Option<String>,
}

/// What the exec plugin prints to kubectl. A certificate and key travel as PEM text, where the
/// stored kubeconfig keeps them as base64; data that is not base64 is an error, never passed on.
pub(super) fn exec_credential(access: &KubeAccess) -> Result<String, String> {
    let status = match &access.credential {
        KubeCredential::Token(token) => ExecStatus { token: Some(token.expose()), client_certificate_data: None, client_key_data: None },
        KubeCredential::ClientCertificate { certificate, key } => ExecStatus {
            token: None,
            client_certificate_data: Some(pem_of(certificate.expose(), "client certificate")?),
            client_key_data: Some(pem_of(key.expose(), "client key")?),
        },
    };
    let credential = ExecCredential { api_version: EXEC_API_VERSION, kind: "ExecCredential", status };
    serde_json::to_string(&credential).map_err(|_| "the credential could not be written".to_string())
}

fn pem_of(data: &str, what: &str) -> Result<String, String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let bytes = STANDARD.decode(data.trim()).map_err(|_| format!("the stored {what} is not base64"))?;
    String::from_utf8(bytes).map_err(|_| format!("the stored {what} is not PEM text"))
}

#[cfg(test)]
#[path = "../../tests/unit/connection/session_kubeconfig_tests.rs"]
mod tests;
