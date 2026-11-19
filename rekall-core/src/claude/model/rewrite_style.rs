/// How a rewrite treats a description: `Fix` corrects the wording and keeps the structure,
/// `Restructure` lays the same content out under the standard headings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteStyle {
    Fix,
    Restructure,
}

impl RewriteStyle {
    pub fn term(self) -> &'static str {
        match self {
            RewriteStyle::Fix => "fix",
            RewriteStyle::Restructure => "restructure",
        }
    }
}
