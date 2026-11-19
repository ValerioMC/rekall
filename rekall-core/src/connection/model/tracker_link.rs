/// The issue a task manager holds for a task: the key it is named by there and the page that shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackerLink {
    pub key: String,
    pub url: String,
}
