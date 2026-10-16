use serde::Deserialize;

/// Which text of a task the revision list is for.
#[derive(Deserialize)]
pub struct RevisionKindQuery {
    pub kind: Option<String>,
}
