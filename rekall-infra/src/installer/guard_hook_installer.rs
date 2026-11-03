use std::fs;
use std::path::{Path, PathBuf};

use rekall_core::claude::{GuardInstaller, GuardProtection, GuardProtectionState};
use rekall_kernel::{InternalFailure, RekallError, Result};
use serde_json::{json, Map, Value};
use tempfile::NamedTempFile;
use tracing::info;

const SETTINGS_FILE: &str = ".claude/settings.json";
const BACKUP_SUFFIX: &str = ".rekall-backup";

const BEFORE_TOOL: Hook = Hook { event: "PreToolUse", matcher: Some("Bash|Read|Edit|Write"), subcommand: "before-tool" };
const BEFORE_STOP: Hook = Hook { event: "Stop", matcher: None, subcommand: "before-stop" };
const HOOKS: [Hook; 2] = [BEFORE_TOOL, BEFORE_STOP];

struct Hook {
    event: &'static str,
    matcher: Option<&'static str>,
    subcommand: &'static str,
}

/// Writes the guard hooks into `~/.claude/settings.json`. An entry is Rekall's when its command
/// ends in `guard before-tool` or `guard before-stop`; nothing else in the file is ever changed.
#[derive(Clone)]
pub struct GuardHookInstaller {
    settings_file: PathBuf,
    program: PathBuf,
}

impl GuardHookInstaller {
    /// `program` is the executable Claude Code runs for `guard <event>`.
    pub fn new(user_home: &Path, program: PathBuf) -> Self {
        Self { settings_file: user_home.join(SETTINGS_FILE), program }
    }

    fn protection(&self, state: GuardProtectionState, problem: Option<String>) -> GuardProtection {
        GuardProtection { state, settings_file: self.settings_file.display().to_string(), problem }
    }

    fn command_of(&self, hook: &Hook) -> String {
        format!("{} guard {}", shell_quoted(&self.program.to_string_lossy()), hook.subcommand)
    }

    fn read_settings(&self) -> Result<Option<Map<String, Value>>> {
        let text = match fs::read_to_string(&self.settings_file) {
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
            self.settings_file.display()
        ))
    }

    fn file_failure(&self, action: &str, failure: &dyn std::fmt::Display) -> RekallError {
        RekallError::internal(InternalFailure::Io, format!("Rekall could not {action} {}: {failure}", self.settings_file.display()))
    }

    fn write_settings(&self, settings: Map<String, Value>) -> Result<()> {
        let folder = self.settings_file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(folder).map_err(|failure| self.file_failure("create the folder of", &failure))?;
        self.keep_first_backup()?;
        let text = serde_json::to_string_pretty(&Value::Object(settings)).expect("a JSON object always serialises");
        let mut file = NamedTempFile::new_in(folder).map_err(|failure| self.file_failure("write", &failure))?;
        std::io::Write::write_all(&mut file, format!("{text}\n").as_bytes()).map_err(|failure| self.file_failure("write", &failure))?;
        self.copy_permissions_to(file.path())?;
        file.persist(&self.settings_file).map_err(|failure| self.file_failure("replace", &failure.error))?;
        Ok(())
    }

    fn keep_first_backup(&self) -> Result<()> {
        let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", self.settings_file.display()));
        if !self.settings_file.exists() || backup.exists() {
            return Ok(());
        }
        fs::copy(&self.settings_file, &backup).map(drop).map_err(|failure| self.file_failure("back up", &failure))
    }

    fn copy_permissions_to(&self, target: &Path) -> Result<()> {
        match fs::metadata(&self.settings_file) {
            Ok(original) => fs::set_permissions(target, original.permissions()).map_err(|failure| self.file_failure("write", &failure)),
            Err(_) => Ok(()),
        }
    }

    fn is_installed_in(&self, settings: &Map<String, Value>) -> bool {
        HOOKS.iter().all(|hook| commands_of(settings, hook.event).contains(&self.command_of(hook)))
    }
}

impl GuardInstaller for GuardHookInstaller {
    fn status(&self) -> GuardProtection {
        match self.read_settings() {
            Ok(Some(settings)) if self.is_installed_in(&settings) => self.protection(GuardProtectionState::Installed, None),
            Ok(_) => self.protection(GuardProtectionState::NotInstalled, None),
            Err(RekallError::IllegalArgument(problem)) => self.protection(GuardProtectionState::InvalidSettings, Some(problem)),
            Err(failure) => self.protection(GuardProtectionState::InvalidSettings, Some(failure.to_string())),
        }
    }

    fn install(&self) -> Result<GuardProtection> {
        let mut settings = self.read_settings()?.unwrap_or_default();
        for hook in &HOOKS {
            let groups = groups_of(&mut settings, hook.event, &self.settings_file)?;
            remove_rekall_entries(groups);
            groups.push(self.group_of(hook));
        }
        self.write_settings(settings)?;
        info!(settings = %self.settings_file.display(), "Installed the guard hooks");
        Ok(self.status())
    }

    fn uninstall(&self) -> Result<GuardProtection> {
        let Some(mut settings) = self.read_settings()? else {
            return Ok(self.status());
        };
        for hook in &HOOKS {
            if settings.get("hooks").and_then(|hooks| hooks.get(hook.event)).is_some() {
                remove_rekall_entries(groups_of(&mut settings, hook.event, &self.settings_file)?);
            }
        }
        drop_empty_hooks(&mut settings);
        self.write_settings(settings)?;
        info!(settings = %self.settings_file.display(), "Removed the guard hooks");
        Ok(self.status())
    }
}

impl GuardHookInstaller {
    fn group_of(&self, hook: &Hook) -> Value {
        let handler = json!({ "type": "command", "command": self.command_of(hook) });
        match hook.matcher {
            Some(matcher) => json!({ "matcher": matcher, "hooks": [handler] }),
            None => json!({ "hooks": [handler] }),
        }
    }
}

fn groups_of<'a>(settings: &'a mut Map<String, Value>, event: &str, file: &Path) -> Result<&'a mut Vec<Value>> {
    let not_usable = |what: &str| RekallError::illegal(format!("{} holds {what} where Rekall needs to add its hooks, so it was left alone.", file.display()));
    let hooks = settings.entry("hooks").or_insert_with(|| json!({}));
    let hooks = hooks.as_object_mut().ok_or_else(|| not_usable("a `hooks` entry that is not an object"))?;
    hooks
        .entry(event)
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| not_usable(&format!("a `{event}` entry that is not a list")))
}

fn remove_rekall_entries(groups: &mut Vec<Value>) {
    for group in groups.iter_mut() {
        if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            handlers.retain(|handler| !is_rekall_command(handler));
        }
    }
    groups.retain(|group| group.get("hooks").and_then(Value::as_array).is_none_or(|handlers| !handlers.is_empty()));
}

fn drop_empty_hooks(settings: &mut Map<String, Value>) {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    hooks.retain(|_, groups| groups.as_array().is_none_or(|groups| !groups.is_empty()));
    if hooks.is_empty() {
        settings.remove("hooks");
    }
}

fn commands_of(settings: &Map<String, Value>, event: &str) -> Vec<String> {
    settings
        .get("hooks")
        .and_then(|hooks| hooks.get(event))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .filter_map(|handler| handler.get("command").and_then(Value::as_str).map(String::from))
        .collect()
}

fn is_rekall_command(handler: &Value) -> bool {
    handler
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| HOOKS.iter().any(|hook| command.ends_with(&format!(" guard {}", hook.subcommand))))
}

fn shell_quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[cfg(test)]
#[path = "../../tests/unit/installer/guard_hook_installer_tests.rs"]
mod tests;
