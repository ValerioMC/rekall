use tracing::warn;

use crate::guard::model::guard_context::GuardContext;
use crate::guard::model::guard_event::GuardEvent;
use crate::guard::model::guard_verdict::GuardVerdict;
use crate::guard::rule::shell_command::git_subcommands;

const SHELL_TOOL: &str = "Bash";
const SUBCOMMANDS_REKALL_RUNS: [&str; 3] = ["commit", "push", "add"];

/// Decides, from an event and the facts around it, whether Claude Code may go on. A command it
/// cannot read is allowed: a guard that stumbles must not stop the work.
pub struct GuardService;

impl GuardService {
    pub fn decide(event: &GuardEvent, context: &GuardContext) -> GuardVerdict {
        let refusal = match event {
            GuardEvent::BeforeTool { tool_name, command_or_path } => refusal_before_tool(tool_name, command_or_path, context),
            GuardEvent::BeforeStop => refusal_before_stop(context),
        };
        match refusal {
            Some(reason) => GuardVerdict::Block { reason },
            None => GuardVerdict::Allow,
        }
    }
}

fn refusal_before_tool(tool_name: &str, command_or_path: &str, context: &GuardContext) -> Option<String> {
    credential_access_refusal(command_or_path, context).or_else(|| manual_git_refusal(tool_name, command_or_path, context))
}

fn credential_access_refusal(command_or_path: &str, context: &GuardContext) -> Option<String> {
    let touches_credentials = context.protected_credential_references.iter().any(|reference| command_or_path.contains(reference.as_str()));
    touches_credentials.then(|| {
        "This reaches Rekall's credentials, which a session never reads, copies or searches. \
         Use the connection's helper command from the session's environment instead."
            .to_string()
    })
}

fn manual_git_refusal(tool_name: &str, command: &str, context: &GuardContext) -> Option<String> {
    if !context.auto_commit || tool_name != SHELL_TOOL {
        return None;
    }
    let subcommands = match git_subcommands(command) {
        Ok(subcommands) => subcommands,
        Err(failure) => {
            warn!(%failure, command, "The guard could not read a shell command and let it through");
            return None;
        }
    };
    let forbidden = subcommands.iter().find(|subcommand| SUBCOMMANDS_REKALL_RUNS.contains(&subcommand.as_str()))?;
    Some(format!(
        "On this project Rekall commits when you claim a step, so do not run `git {forbidden}`: \
         finish the work with `/rk … step:N done` and Rekall commits it."
    ))
}

fn refusal_before_stop(context: &GuardContext) -> Option<String> {
    let number = context.running_step_number?;
    Some(format!(
        "Step {number} is still running and was never claimed. Claim it with `/rk … step:{number} done`, \
         or tell the user why it cannot be claimed."
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/service/guard_service_tests.rs"]
mod tests;
