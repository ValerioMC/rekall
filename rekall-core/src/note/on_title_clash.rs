/// What to do when the task already carries a note with the requested title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnTitleClash {
    Refuse,
    Replace,
}
