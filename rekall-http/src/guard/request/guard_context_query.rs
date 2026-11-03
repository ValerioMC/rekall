use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::guard) struct GuardContextQuery {
    pub(in crate::guard) folder: Option<String>,
}
