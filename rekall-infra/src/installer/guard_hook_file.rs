use std::fs;
use std::path::{Path, PathBuf};

use rekall_core::claude::SessionProvider;
use rekall_core::guard::{GuardProtection, GuardProtectionState};
use rekall_kernel::{InternalFailure, RekallError, Result};
use serde_json::{json, Map, Value};
use tempfile::NamedTempFile;
use tracing::info;

const BACKUP_SUFFIX: &str = ".rekall-backup";

const BEFORE_TOOL: Hook = Hook { event: "PreToolUse", matcher: Some("Bash|Read|Edit|Write"), subcommand: "before-tool" };
const BEFORE_STOP: Hook = Hook { event: "Stop", matcher: None, subcommand: "before-stop" };
const HOOKS: [Hook; 2] = [BEFORE_TOOL, BEFORE_STOP];

struct Hook {
    event: &'static str,
    matcher: Option<&'static str>,
    subcommand: &'static str,
}

#[derive(Clone)]
pub(super) struct GuardHookFile {
    pub(super) path: PathBuf,
    program: PathBuf,
    provider: SessionProvider,
}

impl GuardHookFile {
    pub(super) fn new(path: PathBuf, program: PathBuf, provider: SessionProvider) -> Self {
        Self { path, program, provider }
    }

    pub(super) fn protection(&self, state: GuardProtectionState, configured: bool, problem: Option<String>) -> GuardProtection {
        GuardProtection { state, settings_file: self.path.display().to_string(), configured, problem }
    }

    fn invocation(&self, hook: &Hook) -> String {
        match self.provider {
            SessionProvider::Claude => format!(" guard {}", hook.subcommand),
            SessionProvider::Codex => format!(" guard codex {}", hook.subcommand),
        }
    }

    fn command_of(&self, hook: &Hook) -> String {
        format!("{}{}", shell_quoted(&self.program.to_string_lossy()), self.invocation(hook))
    }

    pub(super) fn commands(&self) -> [String; 2] {
        HOOKS.map(|hook| self.command_of(&hook))
    }

    fn read_settings(&self) -> Result<Option<Map<String, Value>>> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(failure) => return Err(self.file_failure("read", &failure)),
        };
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(settings)) => Ok(Some(settings)),
            Ok(_) => Err(self.invalid("its top level is not an object")),
            Err(failure) => Err(self.invalid(&failure.to_string())),
        }
    }

    fn invalid(&self, why: &str) -> RekallError {
        RekallError::illegal(format!(
            "{} is not usable ({why}), so Rekall left it alone. Fix the file by hand, then try again.",
            self.path.display()
        ))
    }

    fn file_failure(&self, action: &str, failure: &dyn std::fmt::Display) -> RekallError {
        RekallError::internal(InternalFailure::Io, format!("Rekall could not {action} {}: {failure}", self.path.display()))
    }

    fn write_settings(&self, settings: Map<String, Value>) -> Result<()> {
        let folder = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(folder).map_err(|failure| self.file_failure("create the folder of", &failure))?;
        self.keep_first_backup()?;
        let text = serde_json::to_string_pretty(&Value::Object(settings)).expect("a JSON object always serialises");
        let mut file = NamedTempFile::new_in(folder).map_err(|failure| self.file_failure("write", &failure))?;
        std::io::Write::write_all(&mut file, format!("{text}\n").as_bytes()).map_err(|failure| self.file_failure("write", &failure))?;
        self.copy_permissions_to(file.path())?;
        file.persist(&self.path).map_err(|failure| self.file_failure("replace", &failure.error))?;
        Ok(())
    }

    fn keep_first_backup(&self) -> Result<()> {
        let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", self.path.display()));
        if !self.path.exists() || backup.exists() {
            return Ok(());
        }
        fs::copy(&self.path, &backup).map(drop).map_err(|failure| self.file_failure("back up", &failure))
    }

    fn copy_permissions_to(&self, target: &Path) -> Result<()> {
        match fs::metadata(&self.path) {
            Ok(original) => fs::set_permissions(target, original.permissions()).map_err(|failure| self.file_failure("write", &failure)),
            Err(_) => Ok(()),
        }
    }

    pub(super) fn status(&self) -> GuardProtection {
        match self.read_settings().and_then(|settings| self.status_in(&settings.unwrap_or_default())) {
            Ok(protection) => protection,
            Err(RekallError::IllegalArgument(problem)) => self.protection(GuardProtectionState::InvalidSettings, false, Some(problem)),
            Err(failure) => self.protection(GuardProtectionState::InvalidSettings, false, Some(failure.to_string())),
        }
    }

    fn status_in(&self, settings: &Map<String, Value>) -> Result<GuardProtection> {
        let mut installed = true;
        let mut configured = false;
        for hook in &HOOKS {
            let groups = event_groups(settings, hook.event, &self.path)?;
            let owned: Vec<_> = groups.iter().flat_map(handlers).filter(|handler| self.owns(handler)).collect();
            configured |= !owned.is_empty();
            installed &= owned.len() == 1 && groups.contains(&self.group_of(hook));
        }
        Ok(match (installed, configured) {
            (true, _) => self.protection(GuardProtectionState::Installed, true, None),
            (false, true) => self.protection(GuardProtectionState::NeedsRepair, true, Some("Repair updates Rekall’s hook entries.".into())),
            (false, false) => self.protection(GuardProtectionState::NotInstalled, false, None),
        })
    }

    pub(super) fn install(&self) -> Result<GuardProtection> {
        let mut settings = self.read_settings()?.unwrap_or_default();
        let protection = self.status_in(&settings)?;
        if protection.state == GuardProtectionState::Installed {
            return Ok(protection);
        }
        for hook in &HOOKS {
            let groups = groups_of(&mut settings, hook.event, &self.path)?;
            self.replace_entries(groups, Some(self.group_of(hook)));
        }
        self.write_settings(settings)?;
        info!(settings = %self.path.display(), "Installed the guard hooks");
        Ok(self.status())
    }

    pub(super) fn uninstall(&self) -> Result<GuardProtection> {
        let Some(mut settings) = self.read_settings()? else {
            return Ok(self.status());
        };
        let protection = self.status_in(&settings)?;
        if !protection.configured {
            return Ok(protection);
        }
        for hook in &HOOKS {
            if settings.get("hooks").and_then(|hooks| hooks.get(hook.event)).is_none() {
                continue;
            }
            let groups = groups_of(&mut settings, hook.event, &self.path)?;
            self.replace_entries(groups, None);
            drop_event_if_empty(&mut settings, hook.event);
        }
        self.write_settings(settings)?;
        info!(settings = %self.path.display(), "Removed the guard hooks");
        Ok(self.status())
    }

    fn group_of(&self, hook: &Hook) -> Value {
        let command = self.command_of(hook);
        match (self.provider, hook.matcher) {
            (SessionProvider::Codex, Some(_)) => json!({ "matcher": "*", "hooks": [{ "type": "command", "command": command, "timeout": 3 }] }),
            (SessionProvider::Codex, None) => json!({ "hooks": [{ "type": "command", "command": command, "timeout": 3 }] }),
            (SessionProvider::Claude, Some(matcher)) => json!({ "matcher": matcher, "hooks": [{ "type": "command", "command": command }] }),
            (SessionProvider::Claude, None) => json!({ "hooks": [{ "type": "command", "command": command }] }),
        }
    }

    fn owns(&self, handler: &Value) -> bool {
        let Some(command) = handler.get("command").and_then(Value::as_str) else { return false };
        HOOKS.iter().any(|hook| {
            if command == self.command_of(hook) {
                return true;
            }
            let Some(program) = command.strip_suffix(&self.invocation(hook)) else { return false };
            let Some(program) = hook_program(program) else { return false };
            matches!(Path::new(&program).file_name().and_then(|name| name.to_str()),
                Some("rekall-server" | "rekall-desktop" | "rekall-server.exe" | "rekall-desktop.exe"))
        })
    }

    fn replace_entries(&self, groups: &mut Vec<Value>, mut replacement: Option<Value>) {
        let mut index = 0;
        while index < groups.len() {
            let Some(entries) = groups[index].get_mut("hooks").and_then(Value::as_array_mut) else {
                index += 1;
                continue;
            };
            let count = entries.len();
            entries.retain(|handler| !self.owns(handler));
            if count != entries.len() && entries.is_empty() {
                if let Some(group) = replacement.take() {
                    groups[index] = group;
                } else if self.provider == SessionProvider::Codex && index + 1 < groups.len() {
                    index += 1;
                    continue;
                } else {
                    groups.remove(index);
                    continue;
                }
            }
            index += 1;
        }
        groups.extend(replacement);
    }
}

fn hook_program(command: &str) -> Option<String> {
    if let Some(quoted) = command.strip_prefix('\'').and_then(|quoted| quoted.strip_suffix('\'')) {
        let program = quoted.replace("'\\''", "'");
        return (shell_quoted(&program) == command).then_some(program);
    }
    (!command.contains(char::is_whitespace)).then(|| command.to_string())
}

fn handlers(group: &Value) -> &[Value] {
    group.get("hooks").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default()
}

fn event_groups<'a>(settings: &'a Map<String, Value>, event: &str, file: &Path) -> Result<&'a [Value]> {
    let Some(hooks) = settings.get("hooks") else { return Ok(&[]) };
    let hooks = hooks.as_object().ok_or_else(|| invalid_hooks(file, "a `hooks` entry that is not an object"))?;
    match hooks.get(event) {
        Some(groups) => groups.as_array().map(Vec::as_slice)
            .ok_or_else(|| invalid_hooks(file, &format!("a `{event}` entry that is not a list"))),
        None => Ok(&[]),
    }
}

fn groups_of<'a>(settings: &'a mut Map<String, Value>, event: &str, file: &Path) -> Result<&'a mut Vec<Value>> {
    let hooks = settings.entry("hooks").or_insert_with(|| json!({}));
    let hooks = hooks.as_object_mut().ok_or_else(|| invalid_hooks(file, "a `hooks` entry that is not an object"))?;
    hooks.entry(event).or_insert_with(|| json!([])).as_array_mut()
        .ok_or_else(|| invalid_hooks(file, &format!("a `{event}` entry that is not a list")))
}

fn invalid_hooks(file: &Path, what: &str) -> RekallError {
    RekallError::illegal(format!("{} holds {what} where Rekall needs to add its hooks, so it was left alone.", file.display()))
}

fn drop_event_if_empty(settings: &mut Map<String, Value>, event: &str) {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else { return };
    if hooks.get(event).and_then(Value::as_array).is_some_and(Vec::is_empty) {
        hooks.remove(event);
    }
    if hooks.is_empty() {
        settings.remove("hooks");
    }
}

fn shell_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[cfg(test)]
#[path = "../../tests/unit/installer/guard_hook_file_tests.rs"]
mod tests;
