pub struct AddDatabaseCommand {
    pub path: Option<String>,
    /// Blank or absent: the folder's own name.
    pub label: Option<String>,
}
