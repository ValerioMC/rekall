use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct MoveRequest {
    pub index: Option<i32>,
}
