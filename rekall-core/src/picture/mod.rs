//! Pictures kept as files in an `images` folder beside the database and addressed by URL from the
//! markdown. The disk is a port that `rekall-infra` implements.

mod event;
mod model;
mod port;
mod rule;

pub use event::picture_cleanup_stream_event::PictureCleanupStreamEvent;
pub use model::picture_cleanup_state::PictureCleanupState;
pub use model::picture_format::PictureFormat;
pub use model::picture_path::PicturePath;
pub use model::stored_picture::StoredPicture;
pub use port::picture_store::PictureStore;
#[cfg(any(test, feature = "testing"))]
pub use port::in_memory_picture_store::InMemoryPictureStore;
pub use rule::picture_reference::PictureReference;
