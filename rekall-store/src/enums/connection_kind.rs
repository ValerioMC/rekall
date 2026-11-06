use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(ConnectionKind { Gitlab = "GITLAB", Github = "GITHUB", Kubernetes = "KUBERNETES", Jira = "JIRA", Grafana = "GRAFANA" });

impl ConnectionKind {
    /// GitLab, GitHub, Jira and Grafana are reached at the URL a person types; a cluster's server comes from its kubeconfig.
    pub fn reached_at_typed_url(self) -> bool {
        matches!(self, Self::Gitlab | Self::Github | Self::Jira | Self::Grafana)
    }
}
