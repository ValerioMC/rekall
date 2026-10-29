

/// One crumb of a listing's breadcrumb, from the filesystem root down to the folder itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathSegmentView {
    pub name: String,
    pub path: String,
}
