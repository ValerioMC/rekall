use rekall_kernel::{jstr, RekallError, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CodexSessionOptions {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub skip_permissions: bool,
}

impl CodexSessionOptions {
    pub fn new(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Result<Self> {
        Ok(Self {
            model: codex_choice(model, "model")?,
            effort: codex_choice(effort, "reasoning effort")?,
            skip_permissions,
        })
    }
}

fn codex_choice(value: Option<&str>, label: &str) -> Result<Option<String>> {
    let Some(value) = value.map(jstr::strip).filter(|value| !value.is_empty() && *value != "default") else {
        return Ok(None);
    };
    if value.len() <= 150
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_.:/".contains(&byte))
    {
        return Ok(Some(value.to_owned()));
    }
    Err(RekallError::illegal(format!("This Codex {label} is invalid.")))
}
