use std::collections::HashMap;
use std::path::PathBuf;

/// What a credential session adds to the terminal it runs in: its folder first on `PATH`, the
/// kubeconfig, git's credential helpers, the terminal's token, and the Claude Code settings `claude`
/// starts with. None of it is a credential.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionEnvironment {
    pub path_prefix: Option<PathBuf>,
    pub kubeconfig: Option<PathBuf>,
    /// `git config` entries, as `GIT_CONFIG_KEY_n` / `GIT_CONFIG_VALUE_n` carry them.
    pub git_config: Vec<(String, String)>,
    pub token: Option<String>,
    /// Passed to `claude` as `--settings`: the session's deny rules.
    pub claude_settings: Option<PathBuf>,
}

impl SessionEnvironment {
    /// Adds the session to `environment`, after any `GIT_CONFIG_*` entries the person's shell already set.
    pub fn apply(&self, environment: &mut HashMap<String, String>) {
        if let Some(prefix) = &self.path_prefix {
            let path = environment.get("PATH").cloned().unwrap_or_default();
            let joined = if path.is_empty() { prefix.display().to_string() } else { format!("{}:{path}", prefix.display()) };
            environment.insert("PATH".into(), joined);
        }
        if let Some(kubeconfig) = &self.kubeconfig {
            environment.insert("KUBECONFIG".into(), kubeconfig.display().to_string());
        }
        if !self.git_config.is_empty() {
            let first = environment.get("GIT_CONFIG_COUNT").and_then(|count| count.parse::<usize>().ok()).unwrap_or(0);
            for (offset, (key, value)) in self.git_config.iter().enumerate() {
                environment.insert(format!("GIT_CONFIG_KEY_{}", first + offset), key.clone());
                environment.insert(format!("GIT_CONFIG_VALUE_{}", first + offset), value.clone());
            }
            environment.insert("GIT_CONFIG_COUNT".into(), (first + self.git_config.len()).to_string());
        }
        if let Some(token) = &self.token {
            environment.insert("REKALL_CREDENTIAL_TOKEN".into(), token.clone());
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/session_environment_tests.rs"]
mod tests;
