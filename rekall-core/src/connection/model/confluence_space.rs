/// A Confluence space as Rekall reads it: the text that describes it and the page it opens on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfluenceSpace {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub home_page_id: Option<String>,
}
