use crate::picture::UnusedPictureView;

/// Where the cleanup of unused pictures stands, with what each state has to show.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PictureCleanupStatusView {
    #[default]
    Idle,
    Searching { percentage: u8 },
    Ready { pictures: Vec<UnusedPictureView> },
    Deleting { percentage: u8 },
    /// `kept` are listed pictures a text shows by now, or written again since the search.
    Finished { removed: usize, freed_bytes: u64, kept: usize, not_deleted: usize },
    Failed { message: String },
}
