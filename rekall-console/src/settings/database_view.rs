#[derive(Clone)]
pub struct DatabaseView {
    pub id: String,
    pub label: String,
    pub path: String,
    pub active: bool,
    /// Its folder is there now: an unplugged drive is not.
    pub reachable: bool,
    pub added_at: String,
    pub last_used_at: String,
}
