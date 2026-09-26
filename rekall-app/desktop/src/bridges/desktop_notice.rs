use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DesktopNotice {
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) body: String,
}
