use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PictureOwnerKind {
    Task,
    Blueprint,
    Page,
    Component,
}
