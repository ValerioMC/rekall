//! Companies, projects and tasks: the console services that write them, their logos and icons,
//! and `Snapshot`, which loads what their views are built from in a handful of queries.

mod command;
mod service;
mod support;
mod view;

pub use command::company_command::CompanyCommand;
pub use service::company_logo_service::CompanyLogoService;
pub use service::company_service::CompanyService;
pub use view::company_view::CompanyView;
pub use support::icon_raster::{MASTER_SIDE, MAX_UPLOAD_BYTES};
pub use command::project_command::ProjectCommand;
pub use service::project_icon_service::ProjectIconService;
pub use view::project_repository_view::ProjectRepositoryView;
pub use service::project_service::ProjectService;
pub use view::project_view::ProjectView;
pub use support::snapshot::Snapshot;
pub use command::task_command::TaskCommand;
pub use support::task_graph::TaskGraph;
pub use view::task_ref_view::TaskRefView;
pub use service::task_service::TaskService;
pub use view::task_view::TaskView;
