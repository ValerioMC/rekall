//! The REST API the console calls, one folder per domain: each holds its controllers, its request
//! and response shapes and its routes, over the services of `rekall-console` and `rekall-core`.
//! `rekall-mcp` never sees any of it, which keeps the MCP write surface narrow. The error body,
//! extractors and response helpers every controller uses are in `shared`.

mod api_ports;
mod api_state;
mod router;

pub mod shared;

pub mod backup;
pub mod catalog;
pub mod claude;
pub mod commit;
pub mod component;
pub mod connection;
pub mod context;
pub mod diagram;
pub mod filesystem;
pub mod flow;
pub mod graph;
pub mod inventory;
pub mod note;
pub mod queue;
pub mod search;
pub mod section;
pub mod settings;
pub mod skill;
pub mod step;
pub mod stream;
pub mod tag;
pub mod terminal;
pub mod timeentry;
pub mod token;
pub mod version;
pub mod vulnerability;
pub mod wrapup;

pub use api_ports::ApiPorts;
pub use api_state::ApiState;
pub use router::{router, routes};
pub use stream::StepEventStream;
