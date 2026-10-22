use rekall_common::Id;
use rekall_service::connection::JiraIssue;
use serde::Serialize;

use super::jira_rules::issue_url;

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

impl JiraIssueResponse {
    pub fn of(issue: &JiraIssue, site: &str, task_id: Option<Id>) -> Self {
        Self {
            key: issue.key.clone(),
            summary: issue.summary.clone(),
            status: issue.status.clone(),
            issue_type: issue.issue_type.clone(),
            url: issue_url(site, &issue.key),
            task_id,
        }
    }
}
