//! The Jira search the console imports issues through, over Jira's REST API.

mod atlassian_document;
mod http_jira_issues;

pub use http_jira_issues::HttpJiraIssues;

pub(crate) use http_jira_issues::{authorization, jira_client};
