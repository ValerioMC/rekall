use rekall_core::decision::{DecisionDetailView, DecisionLineView, DecisionOptionView, DecisionView};
use rekall_kernel::{Id, Instant};
use rekall_store::{DecisionSource, DecisionState, Speaker};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionResponse {
    pub id: Id,
    pub key: String,
    pub topic: Option<String>,
    pub question: String,
    pub why: Option<String>,
    pub step_number: Option<i32>,
    pub step_title: Option<String>,
    pub state: DecisionState,
    pub answer: Option<String>,
    pub hold_note: Option<String>,
    pub previous_answer: Option<String>,
    pub changed_at: Option<Instant>,
    pub source: DecisionSource,
    pub options: Vec<DecisionOptionResponse>,
    pub lines: Vec<DecisionLineResponse>,
    pub details: Vec<DecisionDetailResponse>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionOptionResponse {
    pub label: String,
    pub effect: Option<String>,
    pub recommended: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionLineResponse {
    pub speaker: Speaker,
    pub text: String,
    pub spoken_at: Instant,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionDetailResponse {
    pub text: String,
    pub added_at: Instant,
}

impl From<DecisionView> for DecisionResponse {
    fn from(view: DecisionView) -> Self {
        Self {
            id: view.id,
            key: view.key,
            topic: view.topic,
            question: view.question,
            why: view.why,
            step_number: view.step_number,
            step_title: view.step_title,
            state: view.state,
            answer: view.answer,
            hold_note: view.hold_note,
            previous_answer: view.previous_answer,
            changed_at: view.changed_at,
            source: view.source,
            options: view.options.into_iter().map(DecisionOptionResponse::from).collect(),
            lines: view.lines.into_iter().map(DecisionLineResponse::from).collect(),
            details: view.details.into_iter().map(DecisionDetailResponse::from).collect(),
        }
    }
}

impl From<DecisionOptionView> for DecisionOptionResponse {
    fn from(view: DecisionOptionView) -> Self {
        Self { label: view.label, effect: view.effect, recommended: view.recommended }
    }
}

impl From<DecisionLineView> for DecisionLineResponse {
    fn from(view: DecisionLineView) -> Self {
        Self { speaker: view.speaker, text: view.text, spoken_at: view.spoken_at }
    }
}

impl From<DecisionDetailView> for DecisionDetailResponse {
    fn from(view: DecisionDetailView) -> Self {
        Self { text: view.text, added_at: view.added_at }
    }
}
