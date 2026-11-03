use rekall_core::guard::{GuardContext, GuardService, GuardVerdict};
use rekall_http::guard::GuardContextResponse;
use tracing::warn;

use super::context_source::GuardContextSource;
use super::guard_mode::GuardMode;
use super::guard_outcome::GuardOutcome;
use super::hook_input::HookInput;

/// Lets the action go ahead whenever it cannot decide: an input it cannot read, or a Rekall that
/// does not answer. A guard that fails closed would stop every session while the app is shut.
pub async fn run(mode: GuardMode, input_text: &str, source: &dyn GuardContextSource) -> GuardOutcome {
    let input = match serde_json::from_str::<HookInput>(input_text) {
        Ok(input) => input,
        Err(failure) => {
            warn!(%failure, "The guard could not read the hook's input and let the action through");
            return GuardOutcome::allow();
        }
    };
    if mode == GuardMode::BeforeStop && input.stop_hook_active {
        return GuardOutcome::allow();
    }
    let context = match source.context_of(&input.cwd).await {
        Ok(response) => context_of(response),
        Err(failure) => {
            warn!(%failure, folder = %input.cwd, "The guard could not reach Rekall and let the action through");
            return GuardOutcome::allow();
        }
    };
    match GuardService::decide(&input.event(mode), &context) {
        GuardVerdict::Allow => GuardOutcome::allow(),
        GuardVerdict::Block { reason } => GuardOutcome::block(reason),
    }
}

fn context_of(response: GuardContextResponse) -> GuardContext {
    GuardContext {
        auto_commit: response.auto_commit,
        running_step_number: response.running_step.map(|step| step.number),
        protected_credential_references: response.protected_credential_references,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/guard_command/guard_runner_tests.rs"]
mod tests;
