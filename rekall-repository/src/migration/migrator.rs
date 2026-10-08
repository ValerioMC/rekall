use sea_orm_migration::prelude::*;

use super::{baseline, m031_project_section, m032_project_icon_image, m033_project_component, m034_company_logo_image};

/// Every migration, in the order it is applied.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(baseline::Baseline),
            Box::new(m031_project_section::ProjectSection),
            Box::new(m032_project_icon_image::ProjectIconImage),
            Box::new(m033_project_component::ProjectComponent),
            Box::new(m034_company_logo_image::CompanyLogoImage),
        ]
    }
}
