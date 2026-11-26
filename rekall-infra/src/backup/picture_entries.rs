//! The pictures of a backup zip: each is the entry `images/<picture path>`, beside the database's.

use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

use rekall_core::picture::PicturePath;
use rekall_kernel::{InternalFailure, RekallError, Result};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::picture::picture_files;

const FOLDER: &str = "images/";

/// Picture formats are compressed already, so each is stored as it is.
pub(super) fn add_pictures<W: io::Write + Seek>(zip: &mut ZipWriter<W>, images: &Path) -> io::Result<()> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for picture in picture_files(images)? {
        zip.start_file(format!("{FOLDER}{}", picture.relative()), options).map_err(io::Error::other)?;
        io::copy(&mut File::open(images.join(picture.relative()))?, zip)?;
    }
    Ok(())
}

/// A backup zip held open, so that pruning the file afterwards takes no picture away.
pub(super) struct BackupPictures {
    archive: ZipArchive<File>,
}

impl BackupPictures {
    pub fn open(zip: &Path) -> Result<Self> {
        let file = File::open(zip).map_err(|_| unreadable())?;
        Ok(Self { archive: ZipArchive::new(file).map_err(|_| unreadable())? })
    }

    /// Writes each picture over a file of the same path and leaves every other file. An entry that
    /// is not a picture path is skipped, which keeps the zip from writing outside the folder.
    pub fn restore_into(mut self, images: &Path) -> Result<()> {
        for index in 0..self.archive.len() {
            let mut entry = self.archive.by_index(index).map_err(|_| unreadable())?;
            let Some(picture) = picture_of(entry.name()) else {
                continue;
            };
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).map_err(|_| unreadable())?;
            write(&images.join(picture.relative()), &bytes)?;
        }
        Ok(())
    }
}

fn picture_of(entry: &str) -> Option<PicturePath> {
    PicturePath::parse(entry.strip_prefix(FOLDER)?).ok()
}

fn write(file: &Path, bytes: &[u8]) -> Result<()> {
    let folder = file.parent().expect("a picture path has folders above its file");
    std::fs::create_dir_all(folder)
        .and_then(|()| std::fs::write(file, bytes))
        .map_err(|failed| RekallError::internal(InternalFailure::Io, format!("Could not write {}: {failed}", file.display())))
}

fn unreadable() -> RekallError {
    RekallError::illegal("A picture in that zip cannot be read. The database was not replaced.")
}

#[cfg(test)]
#[path = "../../tests/unit/backup/picture_entries_tests.rs"]
mod tests;
