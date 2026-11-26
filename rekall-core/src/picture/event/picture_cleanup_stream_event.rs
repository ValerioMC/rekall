use serde::Serialize;

use crate::picture::PictureCleanupState;

/// Emitted when a cleanup changes state or its percentage moves. `count` is the unused pictures
/// of a ready list and the removed ones of a finished run; the console reads the rest again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PictureCleanupStreamEvent {
    pub state: PictureCleanupState,
    pub percentage: u8,
    pub count: usize,
}

impl crate::StreamEvent for PictureCleanupStreamEvent {
    const FRAME: &'static str = "picture-cleanup";
}
