use std::sync::Mutex;

use async_trait::async_trait;

use crate::connection::{TaskTracker, TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};

/// A `TaskTracker` that numbers the issues it is given in memory, for tests of what decides when
/// to write one. `fail_with` makes every call fail until `recover`.
#[derive(Default)]
pub struct ScriptedTaskTracker {
    created: Mutex<Vec<(String, TrackerIssue)>>,
    updated: Mutex<Vec<(String, TrackerIssue)>>,
    failure: Mutex<Option<TrackerFailure>>,
}

impl ScriptedTaskTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_with(&self, failure: TrackerFailure) {
        *self.failure.lock().expect("scripted tracker lock") = Some(failure);
    }

    pub fn recover(&self) {
        *self.failure.lock().expect("scripted tracker lock") = None;
    }

    /// The location each created issue went to, with the issue.
    pub fn created(&self) -> Vec<(String, TrackerIssue)> {
        self.created.lock().expect("scripted tracker lock").clone()
    }

    /// The key each rewritten issue carries, with the issue as it was rewritten.
    pub fn updated(&self) -> Vec<(String, TrackerIssue)> {
        self.updated.lock().expect("scripted tracker lock").clone()
    }

    fn refusal(&self) -> Option<TrackerFailure> {
        self.failure.lock().expect("scripted tracker lock").clone()
    }
}

#[async_trait]
impl TaskTracker for ScriptedTaskTracker {
    async fn create(&self, endpoint: &TrackerEndpoint, issue: &TrackerIssue) -> Result<TrackerLink, TrackerFailure> {
        if let Some(failure) = self.refusal() {
            return Err(failure);
        }
        let mut created = self.created.lock().expect("scripted tracker lock");
        created.push((endpoint.location.clone(), issue.clone()));
        let key = format!("{}-{}", endpoint.location, created.len());
        Ok(TrackerLink { url: format!("{}/issues/{key}", endpoint.site_url), key })
    }

    async fn update(&self, _: &TrackerEndpoint, key: &str, issue: &TrackerIssue) -> Result<(), TrackerFailure> {
        if let Some(failure) = self.refusal() {
            return Err(failure);
        }
        self.updated.lock().expect("scripted tracker lock").push((key.to_string(), issue.clone()));
        Ok(())
    }
}
