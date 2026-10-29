use rekall_console::tag::TagCommand;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TagRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

impl From<TagRequest> for TagCommand {
    fn from(request: TagRequest) -> Self {
        Self {
            name: request.name,
            icon: request.icon,
            color: request.color,
        }
    }
}
