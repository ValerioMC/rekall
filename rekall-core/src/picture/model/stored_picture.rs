use rekall_kernel::Instant;

use crate::picture::PicturePath;

/// A picture file as the disk holds it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredPicture {
    pub path: PicturePath,
    pub size_bytes: u64,
    pub modified_at: Instant,
}
