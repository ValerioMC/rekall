//! What a person does with pictures: keep one beside the database, in its owner's folder, read it
//! back, print it in an export and delete the ones no text shows. The disk is reached through the
//! `PictureStore` port of `rekall_core::picture`.

mod command;
mod rule;
mod service;
mod support;
mod view;

pub use command::picture_import_command::PictureImportCommand;
pub use command::picture_owner::PictureOwner;
pub use command::picture_upload_command::PictureUploadCommand;
pub use rule::picture_content::{too_large, MAX_PICTURE_BYTES};
pub use service::picture_cleanup_service::PictureCleanupService;
pub use service::picture_service::PictureService;
pub use view::imported_picture_view::ImportedPictureView;
pub use view::picture_cleanup_status_view::PictureCleanupStatusView;
pub use view::picture_file_view::PictureFileView;
pub use view::printable_picture_view::PrintablePictureView;
pub use view::unused_picture_view::UnusedPictureView;
