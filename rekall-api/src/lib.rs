//! The REST API the console calls: one router per Java controller, the request and response
//! shapes of `ApiDtos`, the console-only services that write the catalog (`CatalogService`,
//! `DocumentService`, `TagService`, `ExportService`, `RevisionRestoreService`), the folder
//! listing behind the path picker, and the one Server-Sent Events feed. `rekall-mcp` never sees any of it, which is what keeps the MCP write
//! surface narrow.

mod api_state;
pub mod controller;
pub mod dto;
pub mod error;
pub mod extract;
pub mod service;
pub mod stream;

pub use api_state::ApiState;
pub use controller::router;
pub use error::{ApiError, ApiResult};
pub use stream::StepEventStream;
