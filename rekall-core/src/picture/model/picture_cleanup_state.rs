use serde::Serialize;

/// Where the cleanup of unused pictures stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PictureCleanupState {
    Idle,
    Searching,
    Ready,
    Deleting,
    Finished,
    Failed,
}
