/// One answer offered for a question, with what choosing it causes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedOption {
    pub label: String,
    pub effect: Option<String>,
    pub recommended: bool,
}
