/// One Jira issue as Rekall reads it, its description already turned from Atlassian's document
/// format into markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
    pub status: Option<String>,
    pub issue_type: Option<String>,
    pub description: Option<String>,
}
