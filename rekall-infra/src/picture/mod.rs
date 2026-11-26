//! The pictures kept beside the database: the `PictureStore` port of `rekall_core::picture`.

mod folder_picture_store;
mod picture_files;

pub use folder_picture_store::FolderPictureStore;
pub(crate) use picture_files::picture_files;
