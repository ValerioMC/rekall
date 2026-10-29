//! What each project is built from: the libraries and frameworks its manifests declare, at the
//! version each resolves to. A session reads the manifests and writes the list whole
//! (`DependencyInventoryService::write_in`); the console reads it. It is what a vulnerability scan
//! asks the advisory database about.

mod declared_dependency;
mod dependency_inventory_service;
mod inventory_stream_event;
mod inventory_summary_view;
mod inventory_written;
mod manifest_freshness;
mod project_dependency_view;

pub mod rules;

pub use declared_dependency::DeclaredDependency;
pub use dependency_inventory_service::DependencyInventoryService;
pub use inventory_stream_event::InventoryStreamEvent;
pub use inventory_summary_view::InventorySummaryView;
pub use inventory_written::InventoryWritten;
pub use project_dependency_view::ProjectDependencyView;
