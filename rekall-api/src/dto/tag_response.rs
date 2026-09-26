use rekall_common::{Id, Instant};
use rekall_model::tag;
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

impl TagResponse {
    pub fn of(tag: &tag::Model) -> Self {
        Self { id: tag.id, name: tag.name.clone(), icon: tag.icon.clone(), color: tag.color.clone(), updated_at: tag.updated_at }
    }
}
