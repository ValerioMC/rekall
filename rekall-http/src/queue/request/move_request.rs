use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::queue) struct MoveRequest {
    pub index: Option<i32>,
}
