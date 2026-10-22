use rekall_model::ConnectionKind;
use serde::Deserialize;

/// Which helper is asking, which decides the shape of the answer: an `ExecCredential` for kubectl,
/// git's credential lines, or the header a GitLab or Jira API call carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialPurpose {
    Kubectl,
    Git,
    Gitlab,
    Jira,
}

impl CredentialPurpose {
    pub fn serves(self, kind: ConnectionKind) -> bool {
        match self {
            Self::Kubectl => kind == ConnectionKind::Kubernetes,
            Self::Git | Self::Gitlab => kind == ConnectionKind::Gitlab,
            Self::Jira => kind == ConnectionKind::Jira,
        }
    }
}
