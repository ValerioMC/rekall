//! The REST API the console calls, one folder per domain: each holds its routes, its request and
//! response shapes and the console services that write the catalog. `rekall-mcp` never sees any
//! of it, which keeps the MCP write surface narrow. The error body, extractors and response
//! helpers every router shares are in `rekall-web`.

mod api_state;
mod router;

pub mod catalog;
pub mod commit;
pub mod component;
pub mod connection;
pub mod context;
pub mod diagram;
pub mod filesystem;
pub mod flow;
pub mod inventory;
pub mod note;
pub mod search;
pub mod section;
pub mod skill;
pub mod step;
pub mod stream;
pub mod tag;
pub mod timeentry;
pub mod token;
pub mod vulnerability;
pub mod wrapup;

pub use api_state::ApiState;
pub use router::{router, routes};
pub use stream::StepEventStream;
