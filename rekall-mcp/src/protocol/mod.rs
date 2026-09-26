//! The JSON-RPC envelope, the protocol revisions, and the tool contract.

mod json_rpc;
mod mcp_tool;
mod tool_error;
mod tool_schema;
mod version;

pub use json_rpc::{
    text_at as json_rpc_text, Request, Response, RpcError, HEADER_MISMATCH, INTERNAL_ERROR, INVALID_PARAMS,
    INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR, UNSUPPORTED_PROTOCOL_VERSION, VERSION,
};
pub use mcp_tool::McpTool;
pub use tool_error::ToolError;
pub use tool_schema::ToolSchema;
pub use version::ProtocolVersion;
