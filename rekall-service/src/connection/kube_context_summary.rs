use super::KubeUserKind;

/// One context of an imported kubeconfig, as the console offers it for choosing. It names the
/// server and how the user logs in, never the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KubeContextSummary {
    pub name: String,
    pub server: Option<String>,
    pub namespace: Option<String>,
    pub user_kind: KubeUserKind,
    /// Why this context cannot be imported; none when it can.
    pub refusal: Option<String>,
}
