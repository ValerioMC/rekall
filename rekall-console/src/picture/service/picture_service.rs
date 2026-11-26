use std::path::Path;
use std::sync::Arc;

use rekall_core::picture::{PicturePath, PictureStore};
use rekall_core::{in_read, Ctx};
use rekall_kernel::{RekallError, Result};

use crate::picture::rule::picture_content::{self, MAX_PICTURE_BYTES};
use crate::picture::rule::picture_naming;
use crate::picture::rule::picture_print;
use crate::picture::rule::picture_thumbnail;
use crate::picture::support::owner_folder;
use crate::picture::{ImportedPictureView, PictureFileView, PictureImportCommand, PictureOwner, PictureUploadCommand, PrintablePictureView};
use crate::shared::blocking;

/// What a picture that arrived without a name is kept as.
const PASTED_NAME: &str = "pasted-image";

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
        self.keep(command.owner, source, &bytes).await
    }

    pub async fn upload(&self, command: PictureUploadCommand) -> Result<ImportedPictureView> {
        picture_content::within_limit(&command.bytes)?;
        let name = command.name.filter(|name| !name.trim().is_empty()).unwrap_or_else(|| PASTED_NAME.to_string());
        self.keep(command.owner, Path::new(&name), &command.bytes).await
    }

    /// Judges the content, then writes it into the owner's folder under a name taken from `source`.
    async fn keep(&self, owner: PictureOwner, source: &Path, bytes: &[u8]) -> Result<ImportedPictureView> {
        let format = picture_content::format_of(bytes)?;
        let folder = in_read!(&self.ctx, |tx| owner_folder::of(tx.db(), owner).await)?;
        let path = PicturePath::parse(&format!("{folder}/{}", picture_naming::file_name(source, bytes, format)))?;
        self.store.write(&path, bytes).await?;
        Ok(ImportedPictureView { url: path.address() })
    }

    /// A preview of a file the person is browsing, under the size and content rules of an import.
    /// Nothing is copied.
    pub async fn thumbnail(&self, source_path: &str) -> Result<PictureFileView> {
        let bytes = self.store.read_source(Path::new(source_path), MAX_PICTURE_BYTES).await?;
        let format = picture_content::format_of(&bytes)?;
        blocking(move || picture_thumbnail::of(bytes, format)).await
    }

    /// The PNG an export prints for the picture a markdown address names.
    pub async fn printable(&self, address: &str) -> Result<PrintablePictureView> {
        let bytes = self.store.read(&PicturePath::of_address(address)?).await?;
        let format = picture_content::format_of(&bytes)?;
        blocking(move || picture_print::of(bytes, format)).await
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
