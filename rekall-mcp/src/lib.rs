//! The MCP server Claude Code talks to, hand-rolled as the Java one was: MCP over HTTP is a few
//! JSON-RPC methods on one POST endpoint, and a library in between would be one more place for the
//! wire shapes to drift from the ones the registered clients already speak.
//!
//! It depends on `rekall-service` and never on `rekall-api`: the catalog services, the controllers
//! and the document writes are not reachable from here.

pub mod controller;
pub mod protocol;
mod texts;
pub mod tool;

pub use controller::{router, McpController};
