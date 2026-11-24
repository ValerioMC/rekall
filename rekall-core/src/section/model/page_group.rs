/// One context a session wants and the titles of the pages it holds, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageGroup {
    pub context: String,
    pub pages: Vec<String>,
}
