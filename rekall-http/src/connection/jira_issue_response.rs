use rekall_kernel::Id;
use rekall_console::connection::JiraIssueView;
use serde::Serialize;

/// A Jira issue the console can import, and the task it already became, if it did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueResponse {
    pub key: String,
    pub summary: String,
    pub status: Option<String>,
    pub issue_type: Option<String>,
    pub url: String,
    pub task_id: Option<Id>,
}

impl From<JiraIssueView> for JiraIssueResponse {
    fn from(view: JiraIssueView) -> Self {
        Self {
            key: view.key,
            summary: view.summary,
            status: view.status,
            issue_type: view.issue_type,
            url: view.url,
            task_id: view.task_id,
        }
    }
}
