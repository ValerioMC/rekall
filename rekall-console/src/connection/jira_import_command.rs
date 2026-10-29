

/// The keys of the Jira issues to import as the project's tasks.
#[derive(Clone, Debug, Default)]
pub struct JiraImportCommand {
    pub keys: Vec<String>,
}
