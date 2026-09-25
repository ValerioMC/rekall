use std::collections::HashSet;

use rekall_common::{Id, RekallError};
use rekall_model::document::{Column, Entity, Model};
use rekall_model::{document_task, task};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::contains_ignore_case;

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

/// The links of one task, in `position` order: `Task.documents` with its `@OrderColumn`.
pub async fn links_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<document_task::Model>, RekallError> {
    Ok(document_task::Entity::find()
        .filter(document_task::Column::TaskId.eq(task_id))
        .order_by_asc(document_task::Column::Position)
        .all(db)
        .await?)
}

/// The links of one note in the order they were added: `Document.tasks` as a note just created
/// held it, a `LinkedHashSet` filled in the order the request named the tasks.
pub async fn links_of_document_as_added(db: &impl ConnectionTrait, document_id: Id) -> Result<Vec<document_task::Model>, RekallError> {
    Ok(document_task::Entity::find()
        .filter(document_task::Column::DocumentId.eq(document_id))
        .order_by_asc(sea_orm::sea_query::Expr::cust("rowid"))
        .all(db)
        .await?)
}

/// The links of one note as `Document.getTasks()` iterated them once loaded: Hibernate fills the
/// inverse side of the many-to-many into a plain `HashSet`, whose order is its buckets' (see
/// `rekall_common::jcoll`). Within a bucket it is the order the rows came back in, which H2 read
/// off the `(document_id, task_id)` primary key: by task id.
pub async fn links_of_document(db: &impl ConnectionTrait, document_id: Id) -> Result<Vec<document_task::Model>, RekallError> {
    let mut links = links_of_document_as_added(db, document_id).await?;
    links.sort_by_key(|link| link.task_id);
    Ok(rekall_common::jcoll::hash_set_order(links, |link| link.task_id.as_uuid()))
}

/// `Task.getDocuments()`: a task's notes in their order on it.
pub async fn documents_of_task(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    let links = links_of_task(db, task_id).await?;
    let docs = find_by_ids(db, links.iter().map(|l| l.document_id).collect()).await?;
    Ok(links
        .iter()
        .filter_map(|l| docs.iter().find(|d| d.id == l.document_id).cloned())
        .collect())
}

/// `Document.getTasks()`: the tasks a note is on.
pub async fn tasks_of_document(db: &impl ConnectionTrait, document_id: Id) -> Result<Vec<task::Model>, RekallError> {
    let links = links_of_document(db, document_id).await?;
    let tasks = super::task::find_by_ids(db, links.iter().map(|l| l.task_id).collect()).await?;
    Ok(links
        .iter()
        .filter_map(|l| tasks.iter().find(|t| t.id == l.task_id).cloned())
        .collect())
}

/// `findByTasksIdOrderByTitleAsc`.
pub async fn find_by_tasks_id_order_by_title_asc(db: &impl ConnectionTrait, task_id: Id) -> Result<Vec<Model>, RekallError> {
    let mut docs = documents_of_task(db, task_id).await?;
    docs.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(docs)
}

pub async fn find_all_by_order_by_updated_at_desc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_desc(Column::UpdatedAt).all(db).await?)
}

/// `findByProject`: every note on any task of the project, once, newest first.
pub async fn find_by_project(db: &impl ConnectionTrait, project_id: Id) -> Result<Vec<Model>, RekallError> {
    let tasks: HashSet<Id> = super::task::find_by_project_id_order_by_label_asc(db, project_id)
        .await?
        .into_iter()
        .map(|t| t.id)
        .collect();
    let linked: HashSet<Id> = document_task::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .filter(|l| tasks.contains(&l.task_id))
        .map(|l| l.document_id)
        .collect();
    Ok(find_all_by_order_by_updated_at_desc(db)
        .await?
        .into_iter()
        .filter(|d| linked.contains(&d.id))
        .collect())
}

/// `findOrphans`: notes attached to no task.
pub async fn find_orphans(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    let linked: HashSet<Id> = document_task::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|l| l.document_id)
        .collect();
    Ok(Entity::find().all(db).await?.into_iter().filter(|d| !linked.contains(&d.id)).collect())
}

/// `search`: title or body holds the term, by title. The Java query did not escape the term, so a
/// `%` or `_` typed into it is a wildcard here too (see `like_matches`).
pub async fn search(db: &impl ConnectionTrait, term: &str) -> Result<Vec<Model>, RekallError> {
    let mut found: Vec<Model> = Entity::find()
        .all(db)
        .await?
        .into_iter()
        .filter(|d| like_matches(&d.title, term) || like_matches(&d.body_markdown, term))
        .collect();
    found.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(found)
}

/// `searchText`: title or body holds the (already escaped, so literal) term, newest first.
pub async fn search_text(db: &impl ConnectionTrait, term: &str, limit: usize) -> Result<Vec<Model>, RekallError> {
    Ok(find_all_by_order_by_updated_at_desc(db)
        .await?
        .into_iter()
        .filter(|d| contains_ignore_case(&d.title, term) || contains_ignore_case(&d.body_markdown, term))
        .take(limit)
        .collect())
}

/// `findByIdPrefix`: notes whose id starts with the prefix, how a `note:` anchor resolves.
pub async fn find_by_id_prefix(db: &impl ConnectionTrait, prefix: &str) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::Id.starts_with(prefix))
        .all(db)
        .await?)
}

/// `LOWER(text) LIKE LOWER('%' || term || '%')` with the term unescaped: `%` matches any run of
/// characters and `_` any one, as H2 read them.
pub fn like_matches(text: &str, term: &str) -> bool {
    let text: Vec<char> = text.to_lowercase().chars().collect();
    let mut pattern = vec!['%'];
    pattern.extend(term.to_lowercase().chars());
    pattern.push('%');
    like(&text, &pattern)
}

fn like(text: &[char], pattern: &[char]) -> bool {
    // Iterative wildcard matching with backtracking on the last '%'.
    let (mut t, mut p) = (0usize, 0usize);
    let (mut star, mut mark) = (None::<usize>, 0usize);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '_' || (pattern[p] != '%' && pattern[p] == text[t])) {
            t += 1;
            p += 1;
        } else if p < pattern.len() && pattern[p] == '%' {
            star = Some(p);
            mark = t;
            p += 1;
        } else if let Some(s) = star {
            p = s + 1;
            mark += 1;
            t = mark;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '%' {
        p += 1;
    }
    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::like_matches;

    #[test]
    fn an_unescaped_like_reads_percent_and_underscore_as_wildcards() {
        assert!(like_matches("Cluster Access", "cluster"));
        assert!(like_matches("a1b", "a_b"));
        assert!(like_matches("anything", "a%g"));
        assert!(!like_matches("abc", "abd"));
    }
}
