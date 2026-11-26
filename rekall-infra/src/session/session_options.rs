use std::ffi::OsString;
use std::path::PathBuf;

use rekall_core::claude::{CodexSessionOptions, SessionProvider, EFFORT_LEVELS, MODEL_ALIASES};
use rekall_kernel::{jstr, Result};

#[derive(Clone, Debug, Default)]
pub struct SessionOptions {
    pub provider: SessionProvider,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub settings: Option<PathBuf>,
    pub(crate) codex_credential_session: bool,
}

impl SessionOptions {
    pub fn chosen(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Self {
        Self {
            skip_permissions,
            model: normalise(model, &MODEL_ALIASES),
            effort: normalise(effort, &EFFORT_LEVELS),
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
        if self.codex_credential_session {
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

fn normalise(value: Option<&str>, allowed: &[&str]) -> Option<String> {
    let trimmed = jstr::strip(value?).to_lowercase();
    allowed.contains(&trimmed.as_str()).then_some(trimmed)
}

#[cfg(test)]
#[path = "../../tests/unit/session/session_options_tests.rs"]
mod tests;
