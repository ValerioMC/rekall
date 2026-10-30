/// The step a session deleted and how many steps its task holds now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Removed {
    pub title: String,
    pub remaining: usize,
}
