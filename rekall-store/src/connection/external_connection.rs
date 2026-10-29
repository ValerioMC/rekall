//! A system outside Rekall that a company's projects reach: a GitLab instance, a Kubernetes
//! cluster or a Jira site. The row holds only what the console shows; the credential lives in the OS keychain
//! under `secret_ref`, so no query, export or context can carry it.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::ConnectionKind;

pub const LABEL_MAX_CHARACTERS: usize = 120;
pub const TITLE_MAX_CHARACTERS: usize = 200;
pub const BASE_URL_MAX_CHARACTERS: usize = 1_000;
pub const PROXY_URL_MAX_CHARACTERS: usize = 1_000;
pub const KUBE_CONTEXT_MAX_CHARACTERS: usize = 253;
pub const SECRET_REF_MAX_CHARACTERS: usize = 80;

const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "external_connection")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub company_id: Id,
    pub label: String,
    pub title: String,
    pub kind: ConnectionKind,
    /// The GitLab instance or Jira site, or the Kubernetes server read from the imported kubeconfig.
    pub base_url: Option<String>,
    /// The kubeconfig context chosen on import.
    pub kube_context: Option<String>,
    /// Jira only: the proxy every call to the site goes through; none goes direct.
    pub proxy_url: Option<String>,
    /// The keychain account holding the credential, generated with the row.
    pub secret_ref: String,
    /// When a credential was stored; none means the keychain holds nothing for this row.
    pub secret_set_at: Option<Instant>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::company::Entity", from = "Column::CompanyId", to = "crate::company::Column::Id", on_delete = "Cascade")]
    Company,
}

impl Related<crate::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(company_id: Id, label: String, title: String, kind: ConnectionKind, base_url: Option<String>, now: Instant) -> Self {
        let id = Id::random();
        Self {
            id,
            company_id,
            label,
            title,
            kind,
            base_url,
            kube_context: None,
            proxy_url: None,
            secret_ref: format!("connection-{id}"),
            secret_set_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn has_credential(&self) -> bool {
        self.secret_set_at.is_some()
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        self.check_kind_fields()?;
        self.check_base_url()?;
        self.check_proxy_url()?;
        Violations::new("Connection")
            .not_blank("label", Some(&self.label))
            .max_length("label", Some(&self.label), LABEL_MAX_CHARACTERS)
            .slug("label", Some(&self.label))
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("baseUrl", self.base_url.as_deref(), BASE_URL_MAX_CHARACTERS)
            .max_length("kubeContext", self.kube_context.as_deref(), KUBE_CONTEXT_MAX_CHARACTERS)
            .max_length("proxyUrl", self.proxy_url.as_deref(), PROXY_URL_MAX_CHARACTERS)
            .not_blank("secretRef", Some(&self.secret_ref))
            .max_length("secretRef", Some(&self.secret_ref), SECRET_REF_MAX_CHARACTERS)
            .finish()
    }

    /// GitLab and Jira are reached at a typed URL; a context only means something to a kubeconfig.
    fn check_kind_fields(&self) -> Result<(), RekallError> {
        match self.kind {
            ConnectionKind::Gitlab if self.base_url.is_none() => {
                Err(RekallError::illegal("A GitLab connection needs the instance URL, such as https://gitlab.example.com."))
            }
            ConnectionKind::Jira if self.base_url.is_none() => {
                Err(RekallError::illegal("A Jira connection needs the site URL, such as https://example.atlassian.net."))
            }
            ConnectionKind::Gitlab | ConnectionKind::Jira if self.kube_context.is_some() => {
                Err(RekallError::illegal("Only a Kubernetes connection has a kubeconfig context."))
            }
            ConnectionKind::Gitlab | ConnectionKind::Kubernetes if self.proxy_url.is_some() => {
                Err(RekallError::illegal("Only a Jira connection takes a proxy. A cluster is reached through its kubeconfig's proxy-url."))
            }
            _ => Ok(()),
        }
    }

    /// A credential travels to this URL, so plain HTTP is accepted only for this machine.
    fn check_base_url(&self) -> Result<(), RekallError> {
        match self.base_url.as_deref() {
            Some(url) if !is_allowed_base_url(url) => Err(RekallError::illegal(format!(
                "'{url}' has to be an https:// address, or http:// on this machine only."
            ))),
            _ => Ok(()),
        }
    }

    /// The address is stored and shown as typed, so a user or password in it is refused rather than echoed.
    fn check_proxy_url(&self) -> Result<(), RekallError> {
        match self.proxy_url.as_deref() {
            Some(url) if !is_usable_proxy_url(url) => Err(RekallError::illegal(
                "A proxy has to be an http:// or https:// address, such as http://proxy.example.com:8080, with no user or password in it.",
            )),
            _ => Ok(()),
        }
    }
}

fn is_usable_proxy_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://").or_else(|| url.strip_prefix("https://")) else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    !authority.contains('@') && !host_of(rest).is_empty()
}

fn is_allowed_base_url(url: &str) -> bool {
    if url.strip_prefix("https://").is_some_and(|rest| !host_of(rest).is_empty()) {
        return true;
    }
    url.strip_prefix("http://").is_some_and(|rest| LOOPBACK_HOSTS.contains(&host_of(rest)))
}

/// The host of what follows the scheme, without port or path; a bracketed IPv6 keeps its brackets.
fn host_of(rest: &str) -> &str {
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.starts_with('[') {
        return authority.find(']').map_or(authority, |end| &authority[..=end]);
    }
    authority.split(':').next().unwrap_or("")
}

#[cfg(test)]
#[path = "../../tests/unit/connection/external_connection_tests.rs"]
mod tests;
