use std::path::PathBuf;
use std::sync::Arc;

use rekall_service::commit::CommitReferenceService;
use rekall_service::connection::{JiraIssues, SecretStore};
use rekall_service::context::ContextSizeService;
use rekall_service::diagram::DiagramService;
use rekall_service::flow::{FlowService, StepLinkService};
use rekall_service::inventory::DependencyInventoryService;
use rekall_service::search::SearchService;
use rekall_service::section::ProjectSectionService;
use rekall_service::step::TaskStepService;
use rekall_service::timeentry::TimeEntryService;
use rekall_service::token::TokenUsageService;
use rekall_service::vulnerability::{SecurityAdvisoryService, VulnerabilityExposureService, VulnerabilityScanService};
use rekall_service::wrapup::WrapupService;
use rekall_service::Services;

use crate::{catalog, connection, diagram, filesystem, graph, note, section, component, skill, tag, vulnerability, StepEventStream};

/// What the handlers share: each service a controller calls, and nothing it does not.
#[derive(Clone)]
pub struct ApiState {
    pub steps: TaskStepService,
    pub step_links: StepLinkService,
    pub flow: FlowService,
    pub wrapups: WrapupService,
    pub time_entries: TimeEntryService,
    pub token_usage: TokenUsageService,
    pub commit_references: CommitReferenceService,
    pub context_size: ContextSizeService,
    pub search: SearchService,
    pub diagrams: DiagramService,
    pub sections: ProjectSectionService,
    pub inventory: DependencyInventoryService,
    pub advisories: SecurityAdvisoryService,
    pub exposures: VulnerabilityExposureService,
    pub scans: VulnerabilityScanService,
    pub vulnerability_fixes: vulnerability::VulnerabilityFixService,
    pub companies: catalog::CompanyService,
    pub logos: catalog::CompanyLogoService,
    pub projects: catalog::ProjectService,
    pub tasks: catalog::TaskService,
    pub documents: note::DocumentService,
    pub tags: tag::TagService,
    pub skill_assignments: skill::SkillAssignmentService,
    pub diagram_catalog: diagram::DiagramCatalogService,
    pub sources: diagram::SourceExcerptService,
    pub section_catalog: section::ProjectSectionCatalogService,
    pub component_catalog: component::ProjectComponentCatalogService,
    pub graphs: graph::DeploymentGraphService,
    pub graph_attach: graph::GraphAttachService,
    pub connections: connection::ConnectionService,
    pub connection_credentials: connection::ConnectionCredentialService,
    pub kubeconfig_preview: connection::KubeconfigPreviewService,
    pub project_connections: connection::ProjectConnectionService,
    pub repository_discovery: connection::RepositoryDiscoveryService,
    pub connection_access: connection::ConnectionAccessQueryService,
    pub jira_issues: connection::JiraIssueService,
    pub icons: catalog::ProjectIconService,
    pub directories: filesystem::DirectoryListingService,
    pub stream: Arc<StepEventStream>,
}

impl ApiState {
    /// The composition root of the API: `services` is taken apart here, so each controller
    /// reaches only the services it calls. `user_home` is where the path picker starts; `scans`
    /// is built by the caller, which also runs it on a schedule.
    pub fn new(
        services: Services,
        stream: Arc<StepEventStream>,
        user_home: PathBuf,
        secrets: Arc<dyn SecretStore>,
        jira: Arc<dyn JiraIssues>,
        scans: VulnerabilityScanService,
    ) -> Self {
        let ctx = services.ctx.clone();
        let tasks = catalog::TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
        let project_connections = connection::ProjectConnectionService::new(ctx.clone());
        let projects = catalog::ProjectService::new(ctx.clone(), services.repositories);
        let connections = connection::ConnectionService::new(ctx.clone(), secrets.clone());
        let component_catalog = component::ProjectComponentCatalogService::new(ctx.clone());
        let repository_discovery = connection::RepositoryDiscoveryService::new(
            ctx.clone(),
            services.repositories,
            projects.clone(),
            connections.clone(),
            project_connections.clone(),
            component_catalog.clone(),
        );
        let jira_issues = connection::JiraIssueService::new(ctx.clone(), project_connections.clone(), tasks.clone(), secrets.clone(), jira);
        let vulnerability_fixes = vulnerability::VulnerabilityFixService::new(ctx.clone(), services.exposures.clone(), tasks.clone(), services.steps.clone());
        let graphs = graph::DeploymentGraphService::new(ctx.clone());
        let graph_attach = graph::GraphAttachService::new(ctx.clone(), graphs.clone(), services.steps.clone(), services.step_links.clone());
        Self {
            companies: catalog::CompanyService::new(ctx.clone()),
            logos: catalog::CompanyLogoService::new(ctx.clone()),
            projects,
            tasks,
            documents: note::DocumentService::new(ctx.clone()),
            tags: tag::TagService::new(ctx.clone()),
            skill_assignments: skill::SkillAssignmentService::new(ctx.clone()),
            diagram_catalog: diagram::DiagramCatalogService::new(ctx.clone()),
            sources: diagram::SourceExcerptService::new(ctx.clone()),
            section_catalog: section::ProjectSectionCatalogService::new(ctx.clone()),
            component_catalog,
            graphs,
            graph_attach,
            connections,
            connection_credentials: connection::ConnectionCredentialService::new(ctx.clone(), secrets),
            kubeconfig_preview: connection::KubeconfigPreviewService::new(),
            project_connections,
            repository_discovery,
            connection_access: connection::ConnectionAccessQueryService::new(ctx.clone()),
            jira_issues,
            icons: catalog::ProjectIconService::new(ctx),
            directories: filesystem::DirectoryListingService::new(user_home),
            steps: services.steps,
            step_links: services.step_links,
            flow: services.flow,
            wrapups: services.wrapups,
            time_entries: services.time_entries,
            token_usage: services.token_usage,
            commit_references: services.commit_references,
            context_size: services.context_size,
            search: services.search,
            diagrams: services.diagrams,
            sections: services.sections,
            inventory: services.inventory,
            advisories: services.advisories,
            exposures: services.exposures,
            scans,
            vulnerability_fixes,
            stream,
        }
    }
}
