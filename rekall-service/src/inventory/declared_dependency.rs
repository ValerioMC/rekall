use serde::Deserialize;

/// One dependency as a session writes it, before the rules turn it into a stored row.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct DeclaredDependency {
    pub ecosystem: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub manifest: Option<String>,
}
