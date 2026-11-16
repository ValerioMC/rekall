/// One Confluence page as Rekall reads it, its body already turned from Confluence's storage
/// format into markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfluencePage {
    pub id: String,
    pub title: String,
    pub labels: Vec<String>,
    pub markdown: String,
    pub url: String,
}
