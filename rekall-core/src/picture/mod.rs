//! Pictures kept as files in an `images` folder beside the database and addressed by URL from the
//! markdown. The disk is a port that `rekall-infra` implements.

mod model;
mod port;

pub use model::picture_format::PictureFormat;
pub use model::picture_path::PicturePath;
pub use port::picture_store::PictureStore;
