use rekall_store::ConnectionKind;
use serde::Deserialize;

/// Which helper is asking, which decides the shape of the answer: an `ExecCredential` for kubectl,
/// git's credential lines, or the header a GitLab, GitHub or Jira API call carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialPurpose {
    Kubectl,
    Git,
    Gitlab,
    Github,
    Jira,
}

impl CredentialPurpose {
    pub fn serves(self, kind: ConnectionKind) -> bool {
        match self {
            Self::Kubectl => kind == ConnectionKind::Kubernetes,
            Self::Git => matches!(kind, ConnectionKind::Gitlab | ConnectionKind::Github),
            Self::Gitlab => kind == ConnectionKind::Gitlab,
            Self::Github => kind == ConnectionKind::Github,
            Self::Jira => kind == ConnectionKind::Jira,
        }
    }
}
