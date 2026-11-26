//! How a picture's folders and file are named: readable slugs, with the record's id or the
//! content's hash where a name alone would not do.

use std::path::Path;

use rekall_core::picture::PictureFormat;
use rekall_kernel::Id;
use rekall_store::Slug;
use sha2::{Digest, Sha256};

const UNNAMED_FILE: &str = "image";
const SHORT_HEX_CHARACTERS: usize = 8;

/// A record's folder: its name as a slug, or the start of its id when the name leaves none.
pub(in crate::picture) fn folder_segment(name: &str, id: Id) -> String {
    Slug::of(Some(name)).unwrap_or_else(|_| id.to_string()[..SHORT_HEX_CHARACTERS].to_string())
}

/// The hash comes from the content: the same picture keeps one name, two never share it.
pub(in crate::picture) fn file_name(source: &Path, bytes: &[u8], format: PictureFormat) -> String {
    let stem = source
        .file_stem()
        .and_then(|stem| Slug::of(Some(&stem.to_string_lossy())).ok())
        .unwrap_or_else(|| UNNAMED_FILE.to_string());
    format!("{stem}-{}.{}", content_hash(bytes), format.extension())
}

fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().take(SHORT_HEX_CHARACTERS / 2).map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_naming_tests.rs"]
mod tests;
