/// What a session's reorganization came to: the contexts now on the project with the pages each
/// holds, how many pages changed context, and how many emptied contexts were dropped.
#[derive(Clone, Debug)]
pub struct SectionsReorganized {
    pub project_anchor: String,
    pub contexts: Vec<(String, usize)>,
    pub pages_moved: usize,
    pub contexts_dropped: usize,
}
