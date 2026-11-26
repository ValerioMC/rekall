//! What a person does with pictures: keep one beside the database, in its owner's folder, and read
//! it back. The disk is reached through the `PictureStore` port of `rekall_core::picture`.

mod command;
mod rule;
mod service;
mod support;
mod view;

pub use command::picture_import_command::PictureImportCommand;
pub use command::picture_owner::PictureOwner;
pub use service::picture_service::PictureService;
pub use view::imported_picture_view::ImportedPictureView;
pub use view::picture_file_view::PictureFileView;
