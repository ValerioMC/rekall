use serde::Serialize;

/// What happened on a task of an epic, as its dashboard lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EpicActivityKind {
    StepStarted,
    StepClaimed,
    StepAccepted,
    TaskClaimed,
    TaskAccepted,
    WrapupWritten,
    CommitLogged,
}
