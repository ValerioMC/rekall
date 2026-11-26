use std::io;
use std::path::Path;

use rekall_core::picture::PicturePath;

/// Every file under the images folder that a `PicturePath` names: anything else there is not
/// Rekall's and is left alone. A folder that is not there holds none.
pub(crate) fn picture_files(images: &Path) -> io::Result<Vec<PicturePath>> {
    let mut found = Vec::new();
    if images.try_exists()? {
        collect(images, images, &mut found)?;
    }
    Ok(found)
}

/// Links are not followed, so nothing outside the folder is ever listed.
fn collect(images: &Path, folder: &Path, found: &mut Vec<PicturePath>) -> io::Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(images, &entry.path(), found)?;
        } else if kind.is_file() {
            found.extend(picture_path(images, &entry.path()));
        }
    }
    Ok(())
}

fn picture_path(images: &Path, file: &Path) -> Option<PicturePath> {
    let segments: Option<Vec<&str>> = file.strip_prefix(images).ok()?.iter().map(|segment| segment.to_str()).collect();
    PicturePath::parse(&segments?.join("/")).ok()
}

#[cfg(test)]
#[path = "../../tests/unit/picture/picture_files_tests.rs"]
mod tests;
