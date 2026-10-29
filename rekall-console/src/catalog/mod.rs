//! Companies, projects and tasks: the console services that write them, their logos and icons,
//! and `Snapshot`, which loads what their views are built from in a handful of queries.

mod company_command;
mod company_logo_service;
mod company_service;
mod company_view;
mod icon_raster;
mod project_command;
mod project_icon_service;
mod project_repository_view;
mod project_service;
mod project_view;
mod snapshot;
mod task_command;
mod task_graph;
mod task_ref_view;
mod task_service;
mod task_view;

pub use company_command::CompanyCommand;
pub use company_logo_service::CompanyLogoService;
pub use company_service::CompanyService;
pub use company_view::CompanyView;
pub use icon_raster::{MASTER_SIDE, MAX_UPLOAD_BYTES};
pub use project_command::ProjectCommand;
pub use project_icon_service::ProjectIconService;
pub use project_repository_view::ProjectRepositoryView;
pub use project_service::ProjectService;
pub use project_view::ProjectView;
pub use snapshot::Snapshot;
pub use task_command::TaskCommand;
pub use task_graph::TaskGraph;
pub use task_ref_view::TaskRefView;
pub use task_service::TaskService;
pub use task_view::TaskView;
