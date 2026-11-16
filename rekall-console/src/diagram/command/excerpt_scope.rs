/// How much of a file an excerpt carries: the span with a few lines around it, or every line of the
/// file with the span only marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExcerptScope {
    Around,
    WholeFile,
}
