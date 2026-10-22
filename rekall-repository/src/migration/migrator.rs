use sea_orm_migration::prelude::*;

use super::{baseline, m031_project_section, m032_project_icon_image, m033_project_component, m034_company_logo_image, m035_mermaid_diagram, m036_token_usage, m037_skill_placement, m038_skill_assignment, m039_step_flow, m040_flow_start_end, m041_token_usage_day, m042_step_max_visits, m043_diagram_walkthrough, m044_connection, m045_connection_proxy, m046_drop_task_revision};

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
            Box::new(m035_mermaid_diagram::MermaidDiagram),
            Box::new(m036_token_usage::TokenUsage),
            Box::new(m037_skill_placement::SkillPlacement),
            Box::new(m038_skill_assignment::SkillAssignment),
            Box::new(m039_step_flow::StepFlow),
            Box::new(m040_flow_start_end::FlowStartEnd),
            Box::new(m041_token_usage_day::TokenUsageDay),
            Box::new(m042_step_max_visits::StepMaxVisits),
            Box::new(m043_diagram_walkthrough::DiagramWalkthrough),
            Box::new(m044_connection::Connection),
            Box::new(m045_connection_proxy::ConnectionProxy),
            Box::new(m046_drop_task_revision::DropTaskRevision),
        ]
    }
}
