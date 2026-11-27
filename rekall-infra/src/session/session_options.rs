use std::ffi::OsString;
use std::path::PathBuf;

use rekall_core::claude::{claude_effort_choice, claude_model_choice, CodexSessionOptions, SessionProvider};
use rekall_kernel::Result;
use tracing::warn;

#[derive(Clone, Debug, Default)]
pub struct SessionOptions {
    pub provider: SessionProvider,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub settings: Option<PathBuf>,
}

impl SessionOptions {
    pub fn chosen(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Self {
        Self {
            skip_permissions,
            model: or_account_default(claude_model_choice(model)),
            effort: or_account_default(claude_effort_choice(effort)),
            ..Self::default()
        }
    }

    pub fn codex(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Result<Self> {
        let options = CodexSessionOptions::new(skip_permissions, model, effort)?;
        Ok(Self {
            provider: SessionProvider::Codex,
            skip_permissions: options.skip_permissions,
            model: options.model,
            effort: options.effort,
            ..Self::default()
        })
    }

    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments = Vec::new();
        if self.provider == SessionProvider::Codex {
            arguments.push("--no-daemon".into());
        }
        if self.skip_permissions {
            arguments.push(match self.provider {
                SessionProvider::Claude => "--dangerously-skip-permissions",
                SessionProvider::Codex => "--dangerously-bypass-approvals-and-sandbox",
            }.into());
        }
        if let Some(model) = &self.model {
            arguments.extend(["--model".into(), model.into()]);
        }
        if let Some(effort) = &self.effort {
            match self.provider {
                SessionProvider::Claude => arguments.extend(["--effort".into(), effort.into()]),
                SessionProvider::Codex => arguments.extend(["-c".into(), format!("model_reasoning_effort=\"{effort}\"").into()]),
            }
        }
        if let Some(settings) = &self.settings {
            arguments.extend(["--settings".into(), settings.as_os_str().to_owned()]);
        }
        arguments
    }
}

/// A session still opens over a malformed choice, leaving that one to the account.
fn or_account_default(choice: Result<Option<String>>) -> Option<String> {
    choice.unwrap_or_else(|refusal| {
        warn!(%refusal, "Opening the session on the account default");
        None
    })
}

#[cfg(test)]
#[path = "../../tests/unit/session/session_options_tests.rs"]
mod tests;
