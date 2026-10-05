use sea_orm_migration::prelude::*;

use super::{baseline, m031_project_section, m032_project_icon_image};

/// Every migration, in the order it is applied.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(baseline::Baseline),
            Box::new(m031_project_section::ProjectSection),
            Box::new(m032_project_icon_image::ProjectIconImage),
        ]
    }
}
