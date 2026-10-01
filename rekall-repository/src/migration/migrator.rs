use sea_orm_migration::prelude::*;

use super::{m001_core, m002_documents_on_many_tasks, m003_company, m004_label_title_description, m005_to_m010, m011_to_m018, m019_to_m026, m027_project_icon, m028_diagram, m029_task_step_passes, m030_note_scope};

/// Every changeset, in the order the Liquibase changelog applied them.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_core::Environment),
            Box::new(m001_core::Project),
            Box::new(m001_core::Task),
            Box::new(m001_core::Document),
            Box::new(m002_documents_on_many_tasks::DocumentTask),
            Box::new(m002_documents_on_many_tasks::CarryTaskNotes),
            Box::new(m002_documents_on_many_tasks::DropOwnerlessNotes),
            Box::new(m002_documents_on_many_tasks::DocumentLosesItsOwners),
            Box::new(m002_documents_on_many_tasks::DropEnvironment),
            Box::new(m003_company::Company),
            Box::new(m003_company::HoldingCompanyForExistingProjects),
            Box::new(m003_company::ProjectBelongsToACompany),
            Box::new(m003_company::ProjectNameUniquePerCompany),
            Box::new(m004_label_title_description::ProjectLabelAndTitle),
            Box::new(m004_label_title_description::TaskLabelAndTitle),
            Box::new(m005_to_m010::Wrapup),
            Box::new(m005_to_m010::TimeEntry),
            Box::new(m005_to_m010::ProjectBlueprint),
            Box::new(m005_to_m010::ProjectRepoFolder),
            Box::new(m005_to_m010::TaskStep),
            Box::new(m005_to_m010::TaskStepState),
            Box::new(m011_to_m018::ClaudeSession),
            Box::new(m011_to_m018::ClaudeSessionModel),
            Box::new(m011_to_m018::ClaudeSessionEffort),
            Box::new(m011_to_m018::TaskReviewState),
            Box::new(m011_to_m018::TaskWrapupDirective),
            Box::new(m011_to_m018::TaskStepDraft),
            Box::new(m011_to_m018::DropClaudeSession),
            Box::new(m011_to_m018::DropTaskWrapupDirective),
            Box::new(m019_to_m026::CommitReference),
            Box::new(m019_to_m026::CommitReferenceDiff),
            Box::new(m019_to_m026::CommitReferenceInContext),
            Box::new(m019_to_m026::ProjectAutoCommit),
            Box::new(m019_to_m026::Tag),
            Box::new(m019_to_m026::TaskRevision),
            Box::new(m019_to_m026::DocumentContextMode),
            Box::new(m019_to_m026::RunQueue),
            Box::new(m027_project_icon::ProjectIcon),
            Box::new(m028_diagram::Diagram),
            Box::new(m029_task_step_passes::TaskStepPasses),
            Box::new(m030_note_scope::NoteScope),
        ]
    }
}
