use rekall_common::{InternalFailure, RekallError, Result};

use super::kubeconfig_document::{Cluster, Context, KubeconfigDocument, NamedCluster, NamedContext, NamedUser, User};
use super::{KubeCredential, Kubeconfig, Secret};

/// One kubeconfig context resolved to what reaching its cluster takes: the server, the CA that
/// proves it, the proxy in front of it, the namespace and the credential. Certificate data is base64 of the PEM, as in a kubeconfig.
#[derive(Debug)]
pub struct KubeAccess {
    pub context: String,
    pub server: String,
    pub certificate_authority: Option<String>,
    pub tls_server_name: Option<String>,
    pub proxy_url: Option<String>,
    pub namespace: Option<String>,
    pub credential: KubeCredential,
}

impl KubeAccess {
    /// The document kept in the keychain: this context alone, every file inlined, everything else dropped.
    pub fn to_kubeconfig(&self) -> Result<Secret> {
        let document = KubeconfigDocument {
            api_version: Some("v1".into()),
            kind: Some("Config".into()),
            current_context: Some(self.context.clone()),
            clusters: vec![NamedCluster { name: self.context.clone(), cluster: self.cluster() }],
            contexts: vec![NamedContext {
                name: self.context.clone(),
                context: Context { cluster: self.context.clone(), user: self.context.clone(), namespace: self.namespace.clone() },
            }],
            users: vec![NamedUser { name: self.context.clone(), user: self.user() }],
        };
        serde_norway::to_string(&document)
            .map(Secret::new)
            .map_err(|_| RekallError::internal(InternalFailure::Encoding, "a kubeconfig could not be written"))
    }

    /// Reads back what `to_kubeconfig` stored.
    pub fn from_stored(stored: &Secret) -> Result<Self> {
        let corrupt = |error: RekallError| RekallError::internal(InternalFailure::CorruptData, format!("a stored kubeconfig no longer reads: {}", error.message()));
        let kubeconfig = Kubeconfig::parse(stored).map_err(corrupt)?;
        let context = kubeconfig
            .current_context()
            .ok_or_else(|| RekallError::internal(InternalFailure::CorruptData, "a stored kubeconfig names no current context"))?
            .to_string();
        kubeconfig.select(&context).map_err(corrupt)
    }

    fn cluster(&self) -> Cluster {
        Cluster {
            server: Some(self.server.clone()),
            certificate_authority_data: self.certificate_authority.clone(),
            tls_server_name: self.tls_server_name.clone(),
            proxy_url: self.proxy_url.clone(),
            ..Cluster::default()
        }
    }

    fn user(&self) -> User {
        match &self.credential {
            KubeCredential::Token(token) => User { token: Some(token.expose().to_string()), ..User::default() },
            KubeCredential::ClientCertificate { certificate, key } => User {
                client_certificate_data: Some(certificate.expose().to_string()),
                client_key_data: Some(key.expose().to_string()),
                ..User::default()
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/kube_access_tests.rs"]
mod tests;
