use crate::credentials::ClaudeCredentials;

/// Where the token comes from: the real credentials, or what a test says.
pub trait TokenSource: Send + Sync {
    fn access_token(&self) -> Option<String>;
}

impl TokenSource for ClaudeCredentials {
    fn access_token(&self) -> Option<String> {
        ClaudeCredentials::access_token(self)
    }
}
