//! A project's dependency inventory, read by the console. Only a session writes it, over MCP.

mod project_dependency_controller;
mod routes;

pub use routes::routes;
