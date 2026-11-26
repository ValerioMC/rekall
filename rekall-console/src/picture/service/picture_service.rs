use std::path::Path;
use std::sync::Arc;

use rekall_core::picture::{PicturePath, PictureStore};
use rekall_core::{in_read, Ctx};
use rekall_kernel::{RekallError, Result};

use crate::picture::rule::picture_content::{self, MAX_PICTURE_BYTES};
use crate::picture::rule::picture_naming;
use crate::picture::rule::picture_thumbnail;
use crate::picture::support::owner_folder;
use crate::picture::{ImportedPictureView, PictureFileView, PictureImportCommand};
use crate::shared::blocking;

#[derive(Clone)]
pub struct PictureService {
    ctx: Ctx,
    store: Arc<dyn PictureStore>,
}

impl PictureService {
    pub fn new(ctx: Ctx, store: Arc<dyn PictureStore>) -> Self {
        Self { ctx, store }
    }

    pub async fn import(&self, command: PictureImportCommand) -> Result<ImportedPictureView> {
        let source = Path::new(&command.source_path);
        let bytes = self.store.read_source(source, MAX_PICTURE_BYTES).await?;
        let format = picture_content::format_of(&bytes)?;
        let folder = in_read!(&self.ctx, |tx| owner_folder::of(tx.db(), command.owner).await)?;
        let path = PicturePath::parse(&format!("{folder}/{}", picture_naming::file_name(source, &bytes, format)))?;
        self.store.write(&path, &bytes).await?;
        Ok(ImportedPictureView { url: path.address() })
    }

    /// A preview of a file the person is browsing, under the size and content rules of an import.
    /// Nothing is copied.
    pub async fn thumbnail(&self, source_path: &str) -> Result<PictureFileView> {
        let bytes = self.store.read_source(Path::new(source_path), MAX_PICTURE_BYTES).await?;
        let format = picture_content::format_of(&bytes)?;
        blocking(move || picture_thumbnail::of(bytes, format)).await
    }

    pub async fn read(&self, relative: &str) -> Result<PictureFileView> {
        let path = PicturePath::parse(relative).map_err(|_| RekallError::not_found_msg(format!("There is no picture at '{relative}'.")))?;
        let bytes = self.store.read(&path).await?;
        Ok(PictureFileView { bytes, format: path.format() })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/service/picture_service_tests.rs"]
mod tests;
