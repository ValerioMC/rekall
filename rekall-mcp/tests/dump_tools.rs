//! Writes the tool list to REKALL_DUMP_TOOLS when that is set, for comparing against the Java
//! server's text outside the test run. A no-op otherwise.

use rekall_service::{Ctx, EventBus, Services};

#[tokio::test]
async fn dump_the_tool_list() {
    let Ok(path) = std::env::var("REKALL_DUMP_TOOLS") else {
        return;
    };
    let database = rekall_repository::open_temporary().await.unwrap();
    let services = Services::new(Ctx::new(database.conn.clone(), EventBus::new()));
    let tools: Vec<serde_json::Value> = rekall_mcp::tool::all(&services)
        .iter()
        .map(|t| serde_json::json!({ "name": t.name(), "description": t.description(), "inputSchema": t.input_schema() }))
        .collect();
    std::fs::write(path, serde_json::to_string_pretty(&tools).unwrap()).unwrap();
}
