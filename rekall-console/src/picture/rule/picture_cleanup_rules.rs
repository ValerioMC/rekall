//! Which pictures a cleanup may offer and delete, and how far a run has got.

use rekall_core::picture::StoredPicture;
use rekall_kernel::Instant;

/// A file written this recently may belong to a text that is not saved yet.
const GRACE_MINUTES: i64 = 24 * 60;

pub(in crate::picture) fn is_settled(picture: &StoredPicture, now: Instant) -> bool {
    !picture.modified_at.is_after(&now.plus_minutes(-GRACE_MINUTES))
}

pub(in crate::picture) fn forget_shown(unused: &mut Vec<StoredPicture>, texts: &[String]) {
    unused.retain(|picture| !is_shown(picture, texts));
}

/// Whether a listed picture, as the disk and the texts hold it now, may still be deleted.
pub(in crate::picture) fn is_still_unused(picture: &StoredPicture, texts: &[String], now: Instant) -> bool {
    is_settled(picture, now) && !is_shown(picture, texts)
}

/// A text shows a picture by containing its address, so punctuation around a link never makes it
/// look unused.
fn is_shown(picture: &StoredPicture, texts: &[String]) -> bool {
    let address = picture.path.address();
    texts.iter().any(|text| text.contains(&address))
}

/// Whole and never past 100, though rows can be added while they are read.
pub(in crate::picture) fn percentage(done: u64, total: u64) -> u8 {
    let whole = done.saturating_mul(100).checked_div(total).unwrap_or(100);
    u8::try_from(whole.min(100)).unwrap_or(100)
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_cleanup_rules_tests.rs"]
mod tests;
