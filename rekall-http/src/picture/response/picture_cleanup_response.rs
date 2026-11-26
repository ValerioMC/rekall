use rekall_console::picture::PictureCleanupStatusView;
use serde::Serialize;

use crate::picture::response::unused_picture_response::UnusedPictureResponse;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE", rename_all_fields = "camelCase")]
pub enum PictureCleanupResponse {
    Idle,
    Searching { percentage: u8 },
    Ready { pictures: Vec<UnusedPictureResponse>, total_bytes: u64 },
    Deleting { percentage: u8 },
    Finished { removed: usize, freed_bytes: u64, kept: usize, not_deleted: usize },
    Failed { message: String },
}

impl From<PictureCleanupStatusView> for PictureCleanupResponse {
    fn from(view: PictureCleanupStatusView) -> Self {
        match view {
            PictureCleanupStatusView::Idle => Self::Idle,
            PictureCleanupStatusView::Searching { percentage } => Self::Searching { percentage },
            PictureCleanupStatusView::Ready { pictures } => Self::Ready {
                total_bytes: pictures.iter().map(|picture| picture.size_bytes).sum(),
                pictures: pictures.into_iter().map(UnusedPictureResponse::from).collect(),
            },
            PictureCleanupStatusView::Deleting { percentage } => Self::Deleting { percentage },
            PictureCleanupStatusView::Finished { removed, freed_bytes, kept, not_deleted } => Self::Finished { removed, freed_bytes, kept, not_deleted },
            PictureCleanupStatusView::Failed { message } => Self::Failed { message },
        }
    }
}
