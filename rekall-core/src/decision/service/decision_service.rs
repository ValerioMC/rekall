use rekall_kernel::{jstr, Id, Instant, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task_decision, task_decision_detail, task_decision_line, task_decision_option, task_step};
use rekall_store::{DecisionRecord, DecisionState};

use crate::decision::command::answer_decision::AnswerDecision;
use crate::decision::command::ask_decision::AskDecision;
use crate::decision::command::correct_decision::CorrectDecision;
use crate::decision::command::offered_option::OfferedOption;
use crate::decision::command::spoken_line::SpokenLine;
use crate::decision::rule::{decision_scope_rule, decision_text_rule};
use crate::decision::{DecisionStreamEvent, DecisionView};
use crate::deviation::step_at;
use crate::{in_read, in_write, load, Ctx, Tx};

const HOLD_NOTE_MAX_CHARACTERS: usize = 500;

/// What every writer of decisions calls: a session through the MCP tool, the console for a person's
/// own edit. A decision on hold keeps its note in the answer column; an answered one keeps the
/// answer it replaced.
#[derive(Clone)]
pub struct DecisionService {
    ctx: Ctx,
}

impl DecisionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn ask(&self, project_label: Option<&str>, task_label: &str, command: AskDecision) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let steps = load::steps_of(tx.db(), task.id).await?;
            let key = decision_scope_rule::key(&command.key)?;
            let step_number = decision_scope_rule::step_scope(&steps, command.step_number)?;
            refuse_taken_key(&tx, task.id, &key).await?;
            let record = new_record(task.id, key, step_number, command, self.ctx.now())?;
            repo::task_decision::insert(tx.db(), &record).await?;
            announce(&mut tx, &record);
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn answer(&self, project_label: Option<&str>, task_label: &str, command: AnswerDecision) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let steps = load::steps_of(tx.db(), task.id).await?;
            let mut record = repo::task_decision::require_record(tx.db(), task.id, &command.key).await?;
            refuse_answering_an_answered(&record)?;
            let answer = decision_text_rule::required(command.answer.as_deref(), "answer", task_decision::ANSWER_MAX_CHARACTERS, "what was decided")?;
            let detail = detail_text(command.detail.as_deref())?;
            decision_scope_rule::within_call(0, command.lines.len())?;
            let now = self.ctx.now();
            let before = record.decision.clone();
            record.decision.state = DecisionState::Answered;
            record.decision.answer = Some(answer);
            record.decision = repo::update_if_changed(tx.db(), &before, record.decision, now).await?;
            append_detail(&mut record, detail, now);
            append_lines(&mut record, &command.lines, now)?;
            repo::task_decision::replace_children(tx.db(), &record).await?;
            announce(&mut tx, &record);
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn correct(&self, project_label: Option<&str>, task_label: &str, command: CorrectDecision) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let steps = load::steps_of(tx.db(), task.id).await?;
            let mut record = repo::task_decision::require_record(tx.db(), task.id, &command.key).await?;
            refuse_correcting_an_unanswered(&record)?;
            let answer = decision_text_rule::optional(command.answer.as_deref(), "answer", task_decision::ANSWER_MAX_CHARACTERS)?;
            let detail = detail_text(command.detail.as_deref())?;
            refuse_empty_correction(&record.decision.key, answer.is_some(), detail.is_some())?;
            let now = self.ctx.now();
            if let Some(answer) = answer {
                let before = record.decision.clone();
                record.decision.change_answer(answer, now);
                record.decision = repo::update_if_changed(tx.db(), &before, record.decision, now).await?;
            }
            append_detail(&mut record, detail, now);
            repo::task_decision::replace_children(tx.db(), &record).await?;
            announce(&mut tx, &record);
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn hold(&self, project_label: Option<&str>, task_label: &str, key: &str, note: Option<&str>) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let steps = load::steps_of(tx.db(), task.id).await?;
            let mut record = repo::task_decision::require_record(tx.db(), task.id, key).await?;
            refuse_holding_an_answered(&record)?;
            let note = decision_text_rule::required(note, "a note", HOLD_NOTE_MAX_CHARACTERS, "what the person is finding out")?;
            let before = record.decision.clone();
            record.decision.state = DecisionState::OnHold;
            record.decision.answer = Some(jstr::collapse_whitespace(&note));
            record.decision = repo::update_if_changed(tx.db(), &before, record.decision, self.ctx.now()).await?;
            announce(&mut tx, &record);
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn remove(&self, project_label: Option<&str>, task_label: &str, key: &str) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let decision = repo::task_decision::require(tx.db(), task.id, key).await?;
            repo::task_decision::delete(tx.db(), decision.id).await?;
            tx.publish(DecisionStreamEvent { task_id: task.id, key: decision.key });
            Ok(())
        })
    }

    pub async fn list(&self, task_id: Id) -> Result<Vec<DecisionView>> {
        in_read!(&self.ctx, |tx| {
            let steps = load::steps_of(tx.db(), task_id).await?;
            let records = repo::task_decision::list_of_task(tx.db(), task_id).await?;
            Ok(records.iter().map(|record| view_of(record, &steps)).collect())
        })
    }
}

fn new_record(task_id: Id, key: String, step_number: Option<i32>, command: AskDecision, now: Instant) -> Result<DecisionRecord> {
    decision_scope_rule::within_call(command.options.len(), command.lines.len())?;
    let question = decision_text_rule::required(command.question.as_deref(), "a question", task_decision::QUESTION_MAX_CHARACTERS, "what is being asked")?;
    let mut decision = task_decision::Model::new(task_id, key, question, command.source, now);
    decision.step_number = step_number;
    decision.topic = decision_text_rule::optional(command.topic.as_deref(), "topic", task_decision::TOPIC_MAX_CHARACTERS)?;
    decision.why = decision_text_rule::optional(command.why.as_deref(), "why", task_decision::WHY_MAX_CHARACTERS)?;
    let mut record = DecisionRecord { options: offered_options(decision.id, &command.options)?, decision, lines: Vec::new(), details: Vec::new() };
    append_lines(&mut record, &command.lines, now)?;
    Ok(record)
}

fn offered_options(decision_id: Id, offered: &[OfferedOption]) -> Result<Vec<task_decision_option::Model>> {
    offered
        .iter()
        .zip(1..)
        .map(|(option, position)| {
            let label = decision_text_rule::required(Some(&option.label), "an option label", task_decision_option::LABEL_MAX_CHARACTERS, "what the option is called")?;
            let mut stored = task_decision_option::Model::new(decision_id, position, label);
            stored.effect = decision_text_rule::optional(option.effect.as_deref(), "option effect", task_decision_option::EFFECT_MAX_CHARACTERS)?;
            stored.recommended = option.recommended;
            Ok(stored)
        })
        .collect()
}

fn append_lines(record: &mut DecisionRecord, spoken: &[SpokenLine], now: Instant) -> Result<()> {
    let first_position = record.lines.iter().map(|line| line.position).max().unwrap_or(0) + 1;
    for (line, position) in spoken.iter().zip(first_position..) {
        let text = decision_text_rule::required(Some(&line.text), "a conversation line", task_decision_line::TEXT_MAX_CHARACTERS, "what was said")?;
        record.lines.push(task_decision_line::Model::new(record.decision.id, position, line.speaker, text, now));
    }
    Ok(())
}

fn detail_text(detail: Option<&str>) -> Result<Option<String>> {
    decision_text_rule::optional(detail, "detail", task_decision_detail::TEXT_MAX_CHARACTERS)
}

fn append_detail(record: &mut DecisionRecord, detail: Option<String>, now: Instant) {
    if let Some(text) = detail {
        record.details.push(task_decision_detail::Model::new(record.decision.id, text, now));
    }
}

async fn refuse_taken_key(tx: &Tx, task_id: Id, key: &str) -> Result<()> {
    if repo::task_decision::find_by_key(tx.db(), task_id, key).await?.is_none() {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "This task already has a decision with key '{key}'. Use correct to change its answer or add a detail, or choose another key. Nothing was written."
    )))
}

fn refuse_answering_an_answered(record: &DecisionRecord) -> Result<()> {
    if record.decision.state != DecisionState::Answered {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "Decision '{}' is already answered. Use correct to change its answer or add a detail. Nothing was written.",
        record.decision.key
    )))
}

fn refuse_correcting_an_unanswered(record: &DecisionRecord) -> Result<()> {
    if record.decision.state == DecisionState::Answered {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "Decision '{}' has no answer yet. Use answer to give it one. Nothing was written.",
        record.decision.key
    )))
}

fn refuse_holding_an_answered(record: &DecisionRecord) -> Result<()> {
    if record.decision.state != DecisionState::Answered {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "Decision '{}' is answered. Hold is for a question still waiting; use correct to change what was decided. Nothing was written.",
        record.decision.key
    )))
}

fn refuse_empty_correction(key: &str, has_answer: bool, has_detail: bool) -> Result<()> {
    if has_answer || has_detail {
        return Ok(());
    }
    Err(RekallError::illegal(format!("Correcting decision '{key}' needs a new answer, a detail, or both. Nothing was written.")))
}

fn announce(tx: &mut Tx, record: &DecisionRecord) {
    tx.publish(DecisionStreamEvent { task_id: record.decision.task_id, key: record.decision.key.clone() });
}

fn view_of(record: &DecisionRecord, steps: &[task_step::Model]) -> DecisionView {
    let step_title = record.decision.step_number.and_then(|number| step_at(steps, number)).map(|step| step.title.clone());
    DecisionView::of(record, step_title)
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/service/decision_service_tests.rs"]
mod tests;
