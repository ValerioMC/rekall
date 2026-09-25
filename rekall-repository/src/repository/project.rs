use rekall_common::{Id, RekallError};
use rekall_model::company;
use rekall_model::project::{Column, Entity, Model};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use super::{at_most_one, equals_ignore_case};

pub async fn find_by_id(db: &impl ConnectionTrait, id: Id) -> Result<Option<Model>, RekallError> {
    Ok(Entity::find_by_id(id).one(db).await?)
}

pub async fn find_by_ids(db: &impl ConnectionTrait, ids: Vec<Id>) -> Result<Vec<Model>, RekallError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(Entity::find().filter(Column::Id.is_in(ids)).all(db).await?)
}

/// Every project, in creation order: what the ignore-case lookups filter.
pub async fn find_all(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find().order_by_asc(Column::CreatedAt).order_by_asc(Column::Id).all(db).await?)
}

pub async fn find_by_label_ignore_case(db: &impl ConnectionTrait, label: &str) -> Result<Vec<Model>, RekallError> {
    Ok(find_all(db).await?.into_iter().filter(|p| equals_ignore_case(&p.label, label)).collect())
}

pub async fn find_by_company_name_ignore_case_and_label_ignore_case(
    db: &impl ConnectionTrait,
    company_name: &str,
    label: &str,
) -> Result<Option<Model>, RekallError> {
    let companies = company::Entity::find().all(db).await?;
    let wanted: Vec<Id> = companies
        .into_iter()
        .filter(|c| equals_ignore_case(&c.name, company_name))
        .map(|c| c.id)
        .collect();
    let found = find_by_label_ignore_case(db, label)
        .await?
        .into_iter()
        .filter(|p| wanted.contains(&p.company_id))
        .collect();
    at_most_one(found)
}

pub async fn find_by_company_id_order_by_label_asc(db: &impl ConnectionTrait, company_id: Id) -> Result<Vec<Model>, RekallError> {
    Ok(Entity::find()
        .filter(Column::CompanyId.eq(company_id))
        .order_by_asc(Column::Label)
        .all(db)
        .await?)
}

/// `findAllByOrderByCompanyNameAscLabelAsc`.
pub async fn find_all_by_order_by_company_name_asc_label_asc(db: &impl ConnectionTrait) -> Result<Vec<Model>, RekallError> {
    let companies = company::Entity::find().all(db).await?;
    let mut projects = Entity::find().all(db).await?;
    let name_of = |id: &Id| companies.iter().find(|c| c.id == *id).map(|c| c.name.clone()).unwrap_or_default();
    projects.sort_by(|a, b| (name_of(&a.company_id), &a.label).cmp(&(name_of(&b.company_id), &b.label)));
    Ok(projects)
}
