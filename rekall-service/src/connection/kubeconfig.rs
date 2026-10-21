use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rekall_common::{RekallError, Result};

use super::kubeconfig_document::{Cluster, Context, KubeconfigDocument, User};
use super::{KubeAccess, KubeContextSummary, KubeCredential, KubeUserKind, Secret};

/// A kubeconfig as the person pasted or imported it, with every context it holds. Choosing one
/// resolves it to a `KubeAccess`; files it names are read then, so nothing later depends on paths.
pub struct Kubeconfig {
    document: KubeconfigDocument,
}

impl Kubeconfig {
    /// A parse error names only where the text broke: serde's own message can quote a value.
    pub fn parse(text: &Secret) -> Result<Self> {
        if text.is_blank() {
            return Err(RekallError::illegal("The kubeconfig is empty."));
        }
        let document = serde_norway::from_str::<KubeconfigDocument>(text.expose()).map_err(|error| {
            let at = error.location().map(|place| format!(" at line {}, column {}", place.line(), place.column())).unwrap_or_default();
            RekallError::illegal(format!("The kubeconfig is not a valid kubeconfig{at}."))
        })?;
        Ok(Self { document })
    }

    pub fn current_context(&self) -> Option<&str> {
        self.document.current_context.as_deref()
    }

    /// Every context, each with why it cannot be imported when it cannot.
    pub fn contexts(&self) -> Vec<KubeContextSummary> {
        self.document
            .contexts
            .iter()
            .map(|named| KubeContextSummary {
                name: named.name.clone(),
                server: self.cluster(&named.context).and_then(|cluster| cluster.server.clone()),
                namespace: named.context.namespace.clone(),
                user_kind: self.user(&named.context).map_or(KubeUserKind::Nothing, user_kind),
                refusal: self.select(&named.name).err().map(|error| error.message().to_string()),
            })
            .collect()
    }

    pub fn select(&self, name: &str) -> Result<KubeAccess> {
        let context = self.context(name)?;
        let refused = |why: &str| RekallError::illegal(format!("Context '{name}' cannot be imported: {why}."));
        let cluster = self.cluster(context).ok_or_else(|| refused(&format!("it names a cluster '{}' the file does not have", context.cluster)))?;
        let user = self.user(context).ok_or_else(|| refused(&format!("it names a user '{}' the file does not have", context.user)))?;
        let server = verified_server(cluster).map_err(|why| refused(&why))?;
        let credential = credential_of(user).map_err(|why| refused(&why))?;
        let certificate_authority = certificate_authority_of(cluster).map_err(|why| refused(&why))?;
        let proxy_url = proxy_of(cluster).map_err(|why| refused(&why))?;
        Ok(KubeAccess {
            context: name.to_string(),
            server,
            certificate_authority,
            tls_server_name: cluster.tls_server_name.clone(),
            proxy_url,
            namespace: context.namespace.clone(),
            credential,
        })
    }

    fn context(&self, name: &str) -> Result<&Context> {
        let found = self.document.contexts.iter().find(|named| named.name == name).map(|named| &named.context);
        found.ok_or_else(|| {
            let names: Vec<&str> = self.document.contexts.iter().map(|named| named.name.as_str()).collect();
            RekallError::illegal(format!("The kubeconfig has no context '{name}'. It has: {}.", if names.is_empty() { "none".into() } else { names.join(", ") }))
        })
    }

    fn cluster(&self, context: &Context) -> Option<&Cluster> {
        self.document.clusters.iter().find(|named| named.name == context.cluster).map(|named| &named.cluster)
    }

    fn user(&self, context: &Context) -> Option<&User> {
        self.document.users.iter().find(|named| named.name == context.user).map(|named| &named.user)
    }
}

/// A credential is sent to the server, so it has to be one the client can verify over TLS.
fn verified_server(cluster: &Cluster) -> std::result::Result<String, String> {
    let server = cluster.server.as_deref().map(str::trim).filter(|server| !server.is_empty()).ok_or("its cluster has no server")?;
    if !server.starts_with("https://") {
        return Err(format!("its server '{server}' is not an https:// address"));
    }
    if cluster.insecure_skip_tls_verify == Some(true) {
        return Err("its cluster skips TLS verification, so the server could not be trusted with the credential".into());
    }
    Ok(server.to_string())
}

/// The check and the session's kubectl both go through it, so it has to be a scheme both speak, and
/// it is written into the session folder, where no credential may sit.
fn proxy_of(cluster: &Cluster) -> std::result::Result<Option<String>, String> {
    let Some(proxy) = cluster.proxy_url.as_deref().map(str::trim).filter(|proxy| !proxy.is_empty()) else {
        return Ok(None);
    };
    let Some(authority) = proxy.strip_prefix("http://").or_else(|| proxy.strip_prefix("https://")) else {
        return Err(format!("its proxy-url '{proxy}' is not an http:// or https:// address"));
    };
    if authority.split('/').next().is_some_and(|host| host.contains('@')) {
        return Err("its proxy-url carries a username or password, which would be written where the session can read it".into());
    }
    Ok(Some(proxy.to_string()))
}

fn user_kind(user: &User) -> KubeUserKind {
    if user.exec.is_some() {
        KubeUserKind::Exec
    } else if user.auth_provider.is_some() {
        KubeUserKind::AuthProvider
    } else if user.token.is_some() || user.token_file.is_some() {
        KubeUserKind::Token
    } else if user.client_certificate.is_some() || user.client_certificate_data.is_some() {
        KubeUserKind::ClientCertificate
    } else if user.username.is_some() || user.password.is_some() {
        KubeUserKind::BasicAuth
    } else {
        KubeUserKind::Nothing
    }
}

fn credential_of(user: &User) -> std::result::Result<KubeCredential, String> {
    let kind = user_kind(user);
    if let Some(refusal) = kind.refusal() {
        return Err(refusal.to_string());
    }
    if kind == KubeUserKind::Token {
        return token_of(user).map(KubeCredential::Token);
    }
    let certificate = inline_data(user.client_certificate_data.as_deref(), user.client_certificate.as_deref(), "its client certificate")?;
    let key = inline_data(user.client_key_data.as_deref(), user.client_key.as_deref(), "its client key")?
        .ok_or("its user has a client certificate but no client key")?;
    let certificate = certificate.ok_or("its user has no client certificate")?;
    Ok(KubeCredential::ClientCertificate { certificate: Secret::new(certificate), key: Secret::new(key) })
}

fn token_of(user: &User) -> std::result::Result<Secret, String> {
    let token = match (&user.token, &user.token_file) {
        (Some(token), _) => token.trim().to_string(),
        (None, Some(path)) => String::from_utf8(read_referenced(path, "its token file")?).map_err(|_| "its token file is not text".to_string())?.trim().to_string(),
        (None, None) => String::new(),
    };
    if token.is_empty() {
        return Err("its user's token is empty".into());
    }
    Ok(Secret::new(token))
}

fn certificate_authority_of(cluster: &Cluster) -> std::result::Result<Option<String>, String> {
    inline_data(cluster.certificate_authority_data.as_deref(), cluster.certificate_authority.as_deref(), "its certificate authority")
}

/// The `*-data` value as written, or the named file's bytes encoded the way a `*-data` value is.
fn inline_data(data: Option<&str>, file: Option<&str>, what: &str) -> std::result::Result<Option<String>, String> {
    if let Some(data) = data.map(str::trim).filter(|data| !data.is_empty()) {
        return Ok(Some(data.to_string()));
    }
    file.map(|path| read_referenced(path, what).map(|bytes| STANDARD.encode(bytes))).transpose()
}

/// Only an absolute path means something here: the console sends the text, not the file it came from.
fn read_referenced(path: &str, what: &str) -> std::result::Result<Vec<u8>, String> {
    if !Path::new(path).is_absolute() {
        return Err(format!("{what} names '{path}' relative to a kubeconfig file Rekall cannot see; inline it as data or use an absolute path"));
    }
    std::fs::read(path).map_err(|error| format!("{what} '{path}' cannot be read ({})", error.kind()))
}

#[cfg(test)]
#[path = "../../tests/unit/connection/kubeconfig_tests.rs"]
mod tests;
