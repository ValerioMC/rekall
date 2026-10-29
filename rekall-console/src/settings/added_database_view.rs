use super::DatabaseView;

pub struct AddedDatabaseView {
    /// The folder already held a database, which is opened rather than created.
    pub opened_existing: bool,
    pub entry: DatabaseView,
}
