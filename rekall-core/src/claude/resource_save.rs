use rekall_kernel::Instant;

/// A file's whole new text. `expected_modified_at` is the version the editor opened: none creates
/// a file that must not exist yet, a value rewrites one that has not changed since.
#[derive(Clone, Debug, Default)]
pub struct ResourceSave {
    pub path: Option<String>,
    pub body: Option<String>,
    pub expected_modified_at: Option<Instant>,
}
