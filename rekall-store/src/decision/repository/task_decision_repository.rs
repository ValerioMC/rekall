use std::collections::HashMap;

use rekall_kernel::{Id, RekallError};
use crate::decision::DecisionRecord;
use crate::task_decision::{Column, Entity, Model};
use crate::{task_decision_detail, task_decision_line, task_decision_option};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};

pub async fn find_by_key(db: &impl ConnectionTrait, task_id: Id, key: &str) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find().filter(Column::TaskId.eq(task_id)).filter(Column::Key.eq(key)).one(db).await?)
}

/// The task's decision under `key`, or `NotFound` naming the key.
pub async fn require(db: &impl ConnectionTrait, task_id: Id, key: &str) -> Result<Model, RekallError> {
    find_by_key(db, task_id, key).await?.ok_or_else(|| RekallError::not_found_msg(format!("No decision with key '{key}' on this task")))
}

/// The task's own decisions first, then each step's in step order; oldest first within one scope.
pub async fn list_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<DecisionRecord>, RekallError> {
    let decisions = Entity::find()
        .filter(Column::TaskId.eq(task_id))
        .order_by_asc(Column::StepNumber)
        .order_by_asc(Column::CreatedAt)
        .order_by_asc(Column::Id)
        .all(db)
        .await?;
    let ids: Vec<Id> = decisions.iter().map(|decision| decision.id).collect();
    let mut options = group_by_decision(options_of(db, &ids).await?, |option| option.decision_id);
    let mut lines = group_by_decision(lines_of(db, &ids).await?, |line| line.decision_id);
    let mut details = group_by_decision(details_of(db, &ids).await?, |detail| detail.decision_id);
    Ok(decisions
        .into_iter()
        .map(|decision| DecisionRecord {
            options: options.remove(&decision.id).unwrap_or_default(),
            lines: lines.remove(&decision.id).unwrap_or_default(),
            details: details.remove(&decision.id).unwrap_or_default(),
            decision,
        })
        .collect())
}

pub async fn insert(db: &impl ConnectionTrait, record: &DecisionRecord) -> Result<(), RekallError> {
    record.decision.validate()?;
    record.decision.clone().into_active_model().insert(db).await?;
    insert_children(db, record).await
}

/// Writes the record's options, lines and details in place of the ones the decision has.
pub async fn replace_children(db: &impl ConnectionTrait, record: &DecisionRecord) -> Result<(), RekallError> {
    let decision_id = record.decision.id;
    task_decision_option::Entity::delete_many().filter(task_decision_option::Column::DecisionId.eq(decision_id)).exec(db).await?;
    task_decision_line::Entity::delete_many().filter(task_decision_line::Column::DecisionId.eq(decision_id)).exec(db).await?;
    task_decision_detail::Entity::delete_many().filter(task_decision_detail::Column::DecisionId.eq(decision_id)).exec(db).await?;
    insert_children(db, record).await
}

/// Deletes the decision; its options, lines and details go with it.
pub async fn delete(db: &impl ConnectionTrait, id: Id) -> Result<(), RekallError> {
    Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

async fn insert_children(db: &impl ConnectionTrait, record: &DecisionRecord) -> Result<(), RekallError> {
    for option in &record.options {
        option.validate()?;
    }
    for line in &record.lines {
        line.validate()?;
    }
    for detail in &record.details {
        detail.validate()?;
    }
    if !record.options.is_empty() {
        task_decision_option::Entity::insert_many(record.options.iter().cloned().map(IntoActiveModel::<task_decision_option::ActiveModel>::into_active_model)).exec(db).await?;
    }
    if !record.lines.is_empty() {
        task_decision_line::Entity::insert_many(record.lines.iter().cloned().map(IntoActiveModel::<task_decision_line::ActiveModel>::into_active_model)).exec(db).await?;
    }
    if !record.details.is_empty() {
        task_decision_detail::Entity::insert_many(record.details.iter().cloned().map(IntoActiveModel::<task_decision_detail::ActiveModel>::into_active_model)).exec(db).await?;
    }
    Ok(())
}

async fn options_of(db: &impl ConnectionTrait, ids: &[Id]) -> Result<Vec<task_decision_option::Model>, RekallError> {
    Ok(task_decision_option::Entity::find()
        .filter(task_decision_option::Column::DecisionId.is_in(ids.iter().copied()))
        .order_by_asc(task_decision_option::Column::Position)
        .all(db)
        .await?)
}

async fn lines_of(db: &impl ConnectionTrait, ids: &[Id]) -> Result<Vec<task_decision_line::Model>, RekallError> {
    Ok(task_decision_line::Entity::find()
        .filter(task_decision_line::Column::DecisionId.is_in(ids.iter().copied()))
        .order_by_asc(task_decision_line::Column::Position)
        .all(db)
        .await?)
}

async fn details_of(db: &impl ConnectionTrait, ids: &[Id]) -> Result<Vec<task_decision_detail::Model>, RekallError> {
    Ok(task_decision_detail::Entity::find()
        .filter(task_decision_detail::Column::DecisionId.is_in(ids.iter().copied()))
        .order_by_asc(task_decision_detail::Column::AddedAt)
        .order_by_asc(task_decision_detail::Column::Id)
        .all(db)
        .await?)
}

fn group_by_decision<T>(rows: Vec<T>, decision_id: impl Fn(&T) -> Id) -> HashMap<Id, Vec<T>> {
    let mut grouped: HashMap<Id, Vec<T>> = HashMap::new();
    for row in rows {
        grouped.entry(decision_id(&row)).or_default().push(row);
    }
    grouped
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/repository/task_decision_repository_tests.rs"]
mod tests;
