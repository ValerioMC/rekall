//! `McpController`: `POST /mcp`. Two protocol eras meet on this one endpoint, and serving a request
//! under the wrong one fails silently on the client, so the era is decided first and every answer
//! is shaped for it.
//!
//! | | Handshake era | Stateless era |
//! |---|---|---|
//! | Opens with | `initialize`, answered with the revision asked for | nothing |
//! | Unknown method | `-32601` on a 200 | `-32601` on a 404 |
//! | Unknown revision | - | `-32022` on a 400, listing what is supported |
//! | Result envelope | a result is a result | every result carries `resultType` |
//! | `tools/list` | the tools | the tools plus `ttlMs` and `cacheScope` |

mod answer;
mod failure;
mod mcp_controller;
mod mcp_router;

pub use answer::Answer;
pub use mcp_controller::McpController;
pub use mcp_router::router;
