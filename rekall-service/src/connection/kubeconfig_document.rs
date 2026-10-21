//! The parts of a kubeconfig file Rekall reads and writes. Everything else in a file (preferences,
//! extensions) is ignored on reading and absent from the single-context document Rekall stores.

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub(super) struct KubeconfigDocument {
    #[serde(rename = "apiVersion", default, skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(rename = "current-context", default, skip_serializing_if = "Option::is_none")]
    pub current_context: Option<String>,
    #[serde(default)]
    pub clusters: Vec<NamedCluster>,
    #[serde(default)]
    pub contexts: Vec<NamedContext>,
    #[serde(default)]
    pub users: Vec<NamedUser>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct NamedCluster {
    pub name: String,
    pub cluster: Cluster,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Cluster {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_authority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_authority_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insecure_skip_tls_verify: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_server_name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct NamedContext {
    pub name: String,
    pub context: Context,
}

#[derive(Deserialize, Serialize)]
pub(super) struct Context {
    pub cluster: String,
    pub user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct NamedUser {
    pub name: String,
    #[serde(default)]
    pub user: User,
}

/// `exec` and `auth-provider` are only ever looked for, never kept: a user that has them is refused.
#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct User {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "tokenFile", default, skip_serializing_if = "Option::is_none")]
    pub token_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_certificate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_certificate_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing)]
    pub exec: Option<IgnoredAny>,
    #[serde(default, skip_serializing)]
    pub auth_provider: Option<IgnoredAny>,
}
