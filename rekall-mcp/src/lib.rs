//! The hand-rolled MCP server Claude Code talks to: JSON-RPC over one POST endpoint. A folder per
//! domain holds its tools, `tool` the kit they share. It depends on `rekall-core` and never on
//! `rekall-console` or `rekall-http`, so no console write is reachable from here.

pub mod commit;
pub mod context;
pub mod controller;
pub mod diagram;
pub mod flow;
pub mod inventory;
pub mod note;
pub mod protocol;
pub mod section;
pub mod step;
pub mod task;
mod texts;
pub mod tool;
pub mod tool_registry;
pub mod vulnerability;
pub mod wrapup;

pub use controller::{router, McpController};
