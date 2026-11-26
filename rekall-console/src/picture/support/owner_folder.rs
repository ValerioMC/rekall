//! The folder under `images` an owner's pictures are kept in. It is read when a picture is kept:
//! renaming a record later moves no file, so no address breaks.

use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;
use sea_orm::ConnectionTrait;

use crate::picture::rule::picture_naming::folder_segment;
use crate::picture::PictureOwner;

pub(in crate::picture) async fn of(db: &impl ConnectionTrait, owner: PictureOwner) -> Result<String> {
    match owner {
        PictureOwner::Task(id) => {
            let task = repo::task::require(db, id).await?;
            let project = project_folder(db, task.project_id).await?;
            Ok(format!("{project}/{}", task.code.to_ascii_lowercase()))
        }
        PictureOwner::Blueprint(project_id) => Ok(format!("{}/blueprint", project_folder(db, project_id).await?)),
        PictureOwner::Page(id) => {
            let page = repo::project_section::require(db, id).await?;
            let project = project_folder(db, page.project_id).await?;
            Ok(format!("{project}/pages/{}", folder_segment(&page.title, page.id)))
        }
        PictureOwner::Component(id) => {
            let component = repo::project_component::require(db, id).await?;
            let project = project_folder(db, component.project_id).await?;
            Ok(format!("{project}/components/{}", folder_segment(&component.name, component.id)))
        }
    }
}

async fn project_folder(db: &impl ConnectionTrait, project_id: Id) -> Result<String> {
    let project = repo::project::require(db, project_id).await?;
    let company = repo::company::require(db, project.company_id).await?;
    Ok(format!("{}/{}", folder_segment(&company.name, company.id), project.label))
}
