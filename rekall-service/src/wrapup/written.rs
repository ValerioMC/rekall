use rekall_model::WrapupAuthor;

use super::WrapupView;

/// What a write came to: the wrapup, whether it was the first, and whose words it replaced.
#[derive(Clone, Debug)]
pub struct Written {
    pub wrapup: WrapupView,
    pub created: bool,
    pub replaced: Option<WrapupAuthor>,
}
