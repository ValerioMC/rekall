//! What each project is built from: the libraries and frameworks its manifests declare, at the
//! version each resolves to. A session reads the manifests and writes the list whole
//! (`DependencyInventoryService::write_in`); the console reads it. It is what a vulnerability scan
//! asks the advisory database about.

mod event;
mod model;
pub mod rule;
mod service;
mod view;


pub use model::declared_dependency::DeclaredDependency;
pub use service::dependency_inventory_service::DependencyInventoryService;
pub use event::inventory_stream_event::InventoryStreamEvent;
pub use view::inventory_summary_view::InventorySummaryView;
pub use model::inventory_written::InventoryWritten;
pub use view::project_dependency_view::ProjectDependencyView;
