use rekall_core::claude::SessionProvider;
use rekall_core::guard::GuardEvent;
use serde::Deserialize;
use serde_json::Value;

use super::guard_mode::GuardMode;

#[derive(Debug, Deserialize)]
pub(super) struct HookInput {
    pub(super) cwd: String,
    hook_event_name: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<Value>,
    #[serde(default)]
    pub(super) stop_hook_active: bool,
}

impl HookInput {
    pub(super) fn event(&self, provider: SessionProvider, mode: GuardMode) -> Option<GuardEvent> {
        let expected = match mode {
            GuardMode::BeforeTool => "PreToolUse",
            GuardMode::BeforeStop => "Stop",
        };
        let name = self.hook_event_name.as_deref();
        if self.cwd.trim().is_empty() || name.is_some_and(|name| name != expected)
            || (provider == SessionProvider::Codex && name.is_none())
        {
            return None;
        }
        match mode {
            GuardMode::BeforeStop => Some(GuardEvent::BeforeStop),
            GuardMode::BeforeTool => self.tool_event(provider),
        }
    }

    fn tool_event(&self, provider: SessionProvider) -> Option<GuardEvent> {
        let tool = self.tool_name.as_deref()?;
        let input = self.tool_input.as_ref()?;
        let is_shell = match provider {
            SessionProvider::Claude => tool == "Bash",
            SessionProvider::Codex => matches!(tool, "Bash" | "shell" | "shell_command" | "exec_command"),
        };
        let shell_command = input.get("command").and_then(Value::as_str).filter(|_| is_shell).map(String::from);
        let input_texts = match provider {
            SessionProvider::Claude => ["command", "file_path", "path"].iter()
                .find_map(|field| input.get(*field).and_then(Value::as_str))
                .map(String::from).into_iter().collect(),
            SessionProvider::Codex if is_shell => shell_command.iter().cloned().collect(),
            SessionProvider::Codex if tool == "apply_patch" => patch_paths(input),
            SessionProvider::Codex => {
                let mut texts = Vec::new();
                collect_text(input, &mut texts);
                texts
            }
        };
        Some(GuardEvent::BeforeTool { shell_command, input_texts })
    }
}

fn patch_paths(input: &Value) -> Vec<String> {
    const PATH_HEADERS: [&str; 4] = ["*** Add File: ", "*** Update File: ", "*** Delete File: ", "*** Move to: "];
    let Some(patch) = input.get("command").and_then(Value::as_str) else { return Vec::new() };
    patch.lines().filter_map(|line| PATH_HEADERS.iter()
        .find_map(|prefix| line.strip_prefix(*prefix)).map(String::from)).collect()
}

fn collect_text(input: &Value, texts: &mut Vec<String>) {
    match input {
        Value::String(text) => texts.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| collect_text(item, texts)),
        Value::Object(fields) => fields.values().for_each(|value| collect_text(value, texts)),
        _ => {}
    }
}
