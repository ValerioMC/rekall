use std::sync::Mutex;

use async_trait::async_trait;

use crate::connection::{Mergeability, PullRequest, PullRequestHost, PullRequestMerge, TrackerEndpoint, TrackerFailure};

/// A `PullRequestHost` that numbers the pull requests it is given in memory, for tests of what
/// decides when to open or merge one. `merge_answers_with` sets the answer of every merge until it
/// is set again; `fail_with` makes every call fail until `recover`.
pub struct ScriptedPullRequestHost {
    opened: Mutex<Vec<OpenedPullRequest>>,
    merged: Mutex<Vec<u64>>,
    closed_issues: Mutex<Vec<String>>,
    merge_answer: Mutex<PullRequestMerge>,
    failure: Mutex<Option<TrackerFailure>>,
}

/// A pull request as it was opened: the branches and text it was opened with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedPullRequest {
    pub pull_request: PullRequest,
    pub head: String,
    pub base: String,
    pub title: String,
    pub body: String,
}

impl Default for ScriptedPullRequestHost {
    fn default() -> Self {
        Self {
            opened: Mutex::new(Vec::new()),
            merged: Mutex::new(Vec::new()),
            closed_issues: Mutex::new(Vec::new()),
            merge_answer: Mutex::new(PullRequestMerge::Merged),
            failure: Mutex::new(None),
        }
    }
}

impl ScriptedPullRequestHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_answers_with(&self, answer: PullRequestMerge) {
        *self.merge_answer.lock().expect("scripted host lock") = answer;
    }

    pub fn fail_with(&self, failure: TrackerFailure) {
        *self.failure.lock().expect("scripted host lock") = Some(failure);
    }

    pub fn recover(&self) {
        *self.failure.lock().expect("scripted host lock") = None;
    }

    pub fn opened(&self) -> Vec<OpenedPullRequest> {
        self.opened.lock().expect("scripted host lock").clone()
    }

    /// The numbers of the pull requests a merge was asked for and answered `Merged`.
    pub fn merged(&self) -> Vec<u64> {
        self.merged.lock().expect("scripted host lock").clone()
    }

    pub fn closed_issues(&self) -> Vec<String> {
        self.closed_issues.lock().expect("scripted host lock").clone()
    }

    fn refusal(&self) -> Result<(), TrackerFailure> {
        match self.failure.lock().expect("scripted host lock").clone() {
            Some(failure) => Err(failure),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl PullRequestHost for ScriptedPullRequestHost {
    async fn open(&self, endpoint: &TrackerEndpoint, head: &str, base: &str, title: &str, body: &str) -> Result<PullRequest, TrackerFailure> {
        self.refusal()?;
        if let Some(existing) = self.find(endpoint, head).await? {
            return Ok(existing);
        }
        let mut opened = self.opened.lock().expect("scripted host lock");
        let number = opened.len() as u64 + 1;
        let pull_request = PullRequest { number, url: format!("{}/pull/{number}", endpoint.site_url), mergeability: Mergeability::Mergeable };
        opened.push(OpenedPullRequest { pull_request: pull_request.clone(), head: head.to_string(), base: base.to_string(), title: title.to_string(), body: body.to_string() });
        Ok(pull_request)
    }

    async fn find(&self, _: &TrackerEndpoint, head: &str) -> Result<Option<PullRequest>, TrackerFailure> {
        self.refusal()?;
        let opened = self.opened.lock().expect("scripted host lock");
        Ok(opened.iter().find(|entry| entry.head == head).map(|entry| entry.pull_request.clone()))
    }

    async fn merge(&self, _: &TrackerEndpoint, pull_request: &PullRequest) -> Result<PullRequestMerge, TrackerFailure> {
        self.refusal()?;
        let answer = self.merge_answer.lock().expect("scripted host lock").clone();
        if answer == PullRequestMerge::Merged {
            self.merged.lock().expect("scripted host lock").push(pull_request.number);
        }
        Ok(answer)
    }

    async fn close_issue(&self, _: &TrackerEndpoint, issue_key: &str) -> Result<(), TrackerFailure> {
        self.refusal()?;
        self.closed_issues.lock().expect("scripted host lock").push(issue_key.to_string());
        Ok(())
    }
}
