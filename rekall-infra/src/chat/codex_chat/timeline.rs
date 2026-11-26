use rekall_core::chat::{ChatBody, ToolInput};

use super::protocol::{
    Approval, ApprovalParams, ApprovalReply, FileChange, Item, McpToolApproval, McpToolApprovalParams, RequestId,
};

pub(super) fn assistant_text(message_id: String, text: String, complete: bool) -> ChatBody {
    ChatBody::AssistantText { message_id, text, complete, parent_tool_use_id: None }
}

pub(super) fn assistant_thinking(message_id: String, text: String, complete: bool) -> ChatBody {
    ChatBody::AssistantThinking { message_id, text, complete, parent_tool_use_id: None }
}

pub(super) fn item_started(item: Item) -> Option<ChatBody> {
    let (id, name, input) = match item {
        Item::CommandExecution { id, command, .. } => (id, "Bash".to_owned(), ToolInput::Bash { command, description: None }),
        Item::FileChange { id, changes, .. } => (id, "File changes".to_owned(), file_input(changes)),
        Item::McpToolCall { id, server, tool, arguments, .. } => {
            let name = format!("mcp__{server}__{tool}");
            let input = ToolInput::of(&name, &arguments);
            (id, name, input)
        }
        _ => return None,
    };
    Some(ChatBody::ToolUse { tool_use_id: id, name, input, parent_tool_use_id: None })
}

pub(super) fn item_completed(item: Item) -> Option<ChatBody> {
    match item {
        Item::AgentMessage { id, text } | Item::Plan { id, text } => Some(assistant_text(id, text, true)),
        Item::Reasoning { id, summary } if !summary.is_empty() => Some(assistant_thinking(id, summary.join("\n\n"), true)),
        Item::CommandExecution { id, status, aggregated_output, exit_code, .. } => {
            let summary = exit_code.map(|code| format!("Exit code {code}"));
            Some(ChatBody::tool_result(&id, status.failed(), aggregated_output.as_deref().unwrap_or_default(), summary, None))
        }
        Item::FileChange { id, changes, status } => {
            Some(ChatBody::tool_result(&id, status.failed(), &diff(changes), None, None))
        }
        Item::McpToolCall { id, status, result, error, .. } => {
            let output =
                error.map(|error| error.message).or_else(|| result.map(|result| result.to_string())).unwrap_or_default();
            Some(ChatBody::tool_result(&id, status.failed(), &output, None, None))
        }
        _ => None,
    }
}

pub(super) fn approval(method: &str, id: RequestId, request_id: String, params: ApprovalParams) -> Approval {
    let (name, input) = if let Some(network) = params.network_approval_context {
        let address = format!("{}://{}", network.protocol, network.host);
        let text = [Some(address), params.reason].into_iter().flatten().collect::<Vec<_>>().join("\n");
        ("network access", ToolInput::Other { name: "network access".to_owned(), text })
    } else if let Some(command) = params.command {
        ("Bash", ToolInput::Bash { command, description: params.reason })
    } else if method == "item/commandExecution/requestApproval" {
        let text = params.reason.unwrap_or_default();
        ("terminal input", ToolInput::Other { name: "terminal input".to_owned(), text })
    } else {
        let text = [params.grant_root, params.reason].into_iter().flatten().collect::<Vec<_>>().join("\n");
        ("File changes", ToolInput::Other { name: "File changes".to_owned(), text })
    };
    let event = ChatBody::PermissionRequested {
        request_id: request_id.clone(),
        tool_use_id: params.item_id,
        name: name.to_owned(),
        input,
    };
    Approval { id, view_id: request_id, event, reply: ApprovalReply::Decision }
}

pub(super) fn mcp_tool_approval(id: RequestId, request_id: String, params: McpToolApprovalParams) -> Approval {
    let McpToolApproval::McpToolCall { tool_name, tool_params } = params.tool;
    let name = format!("mcp__{}__{tool_name}", params.server_name);
    let input = ToolInput::of(&name, &tool_params);
    let event = ChatBody::PermissionRequested {
        request_id: request_id.clone(),
        tool_use_id: request_id.clone(),
        name,
        input,
    };
    Approval { id, view_id: request_id, event, reply: ApprovalReply::McpTool }
}

fn file_input(changes: Vec<FileChange>) -> ToolInput {
    let text = changes.into_iter().map(|change| change.path).collect::<Vec<_>>().join("\n");
    ToolInput::Other { name: "File changes".to_owned(), text }
}

fn diff(changes: Vec<FileChange>) -> String {
    changes.into_iter().map(|change| format!("{}\n{}", change.path, change.diff)).collect::<Vec<_>>().join("\n")
}
