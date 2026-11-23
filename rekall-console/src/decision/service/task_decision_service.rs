use std::collections::HashSet;

use rekall_core::context::ContextService;
use rekall_core::decision::rule::decision_scope_rule;
use rekall_core::decision::rule::decision_text_rule;
use rekall_core::decision::{DecisionService, DecisionSource, DecisionState, DecisionStreamEvent, DecisionView};
use rekall_core::deviation::step_at;
use rekall_core::{in_read, in_write, Ctx};
use rekall_kernel::{jstr, Id, Instant, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task_decision, task_decision_detail, task_step, DecisionRecord, Slug};

use crate::decision::{AddDecision, EditDecision, TaskDecisionsView};

const KEY_STEM_MAX_CHARACTERS: usize = 60;

#[derive(Clone)]
pub struct TaskDecisionService {
    ctx: Ctx,
    decisions: DecisionService,
    context: ContextService,
}

impl TaskDecisionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { decisions: DecisionService::new(ctx.clone()), context: ContextService::new(ctx.clone()), ctx }
    }

    pub async fn list_of(&self, task_id: Id) -> Result<TaskDecisionsView> {
        in_read!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            Ok(())
        })?;
        Ok(TaskDecisionsView { decisions: self.decisions.list(task_id).await?, session_block: self.context.decisions_block(task_id).await? })
    }

    pub async fn add(&self, task_id: Id, command: AddDecision) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            let steps = repo::task_step::list_of_task(tx.db(), task_id).await?;
            let step_number = decision_scope_rule::step_scope(&steps, command.step_number)?;
            let question = decision_text_rule::required(command.question.as_deref(), "a question", task_decision::QUESTION_MAX_CHARACTERS, "what was decided about")?;
            let answer = decision_text_rule::required(command.answer.as_deref(), "an answer", task_decision::ANSWER_MAX_CHARACTERS, "what was decided")?;
            let key = free_key(&repo::task_decision::list_of_task(tx.db(), task_id).await?, &question);
            let mut decision = task_decision::Model::new(task_id, key, question, DecisionSource::Person, self.ctx.now());
            decision.step_number = step_number;
            decision.topic = decision_text_rule::optional(command.topic.as_deref(), "topic", task_decision::TOPIC_MAX_CHARACTERS)?;
            decision.state = DecisionState::Answered;
            decision.answer = Some(answer);
            let record = DecisionRecord { decision, options: Vec::new(), lines: Vec::new(), details: Vec::new() };
            repo::task_decision::insert(tx.db(), &record).await?;
            tx.publish(DecisionStreamEvent { task_id, key: record.decision.key.clone() });
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn edit(&self, decision_id: Id, command: EditDecision) -> Result<DecisionView> {
        in_write!(&self.ctx, |tx| {
            let mut record = repo::task_decision::require_record_by_id(tx.db(), decision_id).await?;
            let steps = repo::task_step::list_of_task(tx.db(), record.decision.task_id).await?;
            let answer = decision_text_rule::optional(command.answer.as_deref(), "answer", task_decision::ANSWER_MAX_CHARACTERS)?;
            let note = decision_text_rule::optional(command.note.as_deref(), "note", task_decision_detail::TEXT_MAX_CHARACTERS)?;
            refuse_empty_edit(&record, answer.is_some(), note.is_some())?;
            let now = self.ctx.now();
            let before = record.decision.clone();
            apply_answer(&mut record.decision, answer, now);
            record.decision.source = DecisionSource::Person;
            record.decision = repo::update_if_changed(tx.db(), &before, record.decision, now).await?;
            if let Some(text) = note {
                record.details.push(task_decision_detail::Model::new(record.decision.id, text, now));
            }
            repo::task_decision::replace_children(tx.db(), &record).await?;
            tx.publish(DecisionStreamEvent { task_id: record.decision.task_id, key: record.decision.key.clone() });
            Ok(view_of(&record, &steps))
        })
    }

    pub async fn remove(&self, decision_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let record = repo::task_decision::require_record_by_id(tx.db(), decision_id).await?;
            repo::task_decision::delete(tx.db(), decision_id).await?;
            tx.publish(DecisionStreamEvent { task_id: record.decision.task_id, key: record.decision.key });
            Ok(())
        })
    }
}

fn apply_answer(decision: &mut task_decision::Model, answer: Option<String>, now: Instant) {
    let Some(answer) = answer else {
        return;
    };
    if decision.state != DecisionState::Answered {
        decision.state = DecisionState::Answered;
        decision.answer = Some(answer);
    } else if decision.answer.as_deref() != Some(answer.as_str()) {
        decision.change_answer(answer, now);
    }
}

fn refuse_empty_edit(record: &DecisionRecord, has_answer: bool, has_note: bool) -> Result<()> {
    let key = &record.decision.key;
    if record.decision.state != DecisionState::Answered && !has_answer {
        return Err(RekallError::illegal(format!("Decision '{key}' has no answer yet: give it one. Nothing was changed.")));
    }
    if !has_answer && !has_note {
        return Err(RekallError::illegal(format!("Editing decision '{key}' needs a new answer, a note, or both. Nothing was changed.")));
    }
    Ok(())
}

fn free_key(existing: &[DecisionRecord], question: &str) -> String {
    let taken: HashSet<&str> = existing.iter().map(|record| record.decision.key.as_str()).collect();
    let stem = Slug::of(Some(question)).unwrap_or_else(|_| "decision".to_string());
    let stem = jstr::prefix(&stem, KEY_STEM_MAX_CHARACTERS).trim_end_matches(['-', '_', '.']).to_string();
    jstr::first_free_suffix(&stem, |candidate| taken.contains(candidate))
}

fn view_of(record: &DecisionRecord, steps: &[task_step::Model]) -> DecisionView {
    let step_title = record.decision.step_number.and_then(|number| step_at(steps, number)).map(|step| step.title.clone());
    DecisionView::of(record, step_title)
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/service/task_decision_service_tests.rs"]
mod tests;
