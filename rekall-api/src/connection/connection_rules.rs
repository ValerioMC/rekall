//! The rules on what a person types into a connection, shared by the services that write one.

use rekall_common::{jstr, RekallError, Result};
use rekall_model::{ConnectionKind, Slug};

pub(super) fn kind_of(raw: Option<&str>) -> Result<ConnectionKind> {
    let raw = raw.map(jstr::strip).unwrap_or("");
    ConnectionKind::value_of(&raw.to_uppercase()).ok_or_else(|| RekallError::illegal(format!("'{raw}' is not a connection kind. Use GITLAB or KUBERNETES.")))
}

pub(super) fn title_of(raw: Option<&str>) -> Result<String> {
    raw.map(jstr::strip).filter(|title| !title.is_empty()).map(str::to_string).ok_or_else(|| RekallError::illegal("A connection needs a title."))
}

/// The label typed, or the title made into one.
pub(super) fn label_of(label: Option<&str>, title: &str) -> Result<String> {
    Slug::of(Some(label.filter(|label| !jstr::is_blank(label)).unwrap_or(title)))
}

/// A Kubernetes connection reads its server from the kubeconfig, so only GitLab takes a URL typed in.
pub(super) fn base_url_of(kind: ConnectionKind, raw: Option<&str>) -> Result<Option<String>> {
    let url = raw.map(jstr::strip).filter(|url| !url.is_empty()).map(|url| url.trim_end_matches('/').to_string());
    if kind == ConnectionKind::Kubernetes && url.is_some() {
        return Err(RekallError::illegal("A Kubernetes connection takes its server from the kubeconfig, not from a typed URL."));
    }
    Ok(url)
}

/// A blank target means the project uses the connection as a whole.
pub(super) fn target_of(raw: Option<&str>) -> Option<String> {
    raw.map(jstr::strip).filter(|target| !target.is_empty()).map(str::to_string)
}

#[cfg(test)]
#[path = "../../tests/unit/connection/connection_rules_tests.rs"]
mod tests;
