use rekall_common::{Id, Instant};
use rekall_console::tag::TagView;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
    pub id: Id,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub updated_at: Instant,
}

impl From<TagView> for TagResponse {
    fn from(view: TagView) -> Self {
        Self {
            id: view.id,
            name: view.name,
            icon: view.icon,
            color: view.color,
            updated_at: view.updated_at,
        }
    }
}
