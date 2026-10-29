use serde::Serialize;

/// What a kubeconfig user authenticates with. Only a token and a client certificate are the
/// person's own credential; the rest run a command or send a password Rekall will not broker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KubeUserKind {
    Token,
    ClientCertificate,
    Exec,
    AuthProvider,
    BasicAuth,
    Nothing,
}

impl KubeUserKind {
    /// Why a user of this kind is refused, or none when Rekall can hand its credential out.
    pub fn refusal(self) -> Option<&'static str> {
        match self {
            Self::Token | Self::ClientCertificate => None,
            Self::Exec => Some("its user runs a command to log in (exec), so the credential is not one you provided"),
            Self::AuthProvider => Some("its user logs in through an auth provider, so the credential is not one you provided"),
            Self::BasicAuth => Some("its user logs in with a username and password, which Rekall does not hand out"),
            Self::Nothing => Some("its user carries no token and no client certificate"),
        }
    }
}
