use rekall_core::claude::SessionProvider;
use rekall_core::guard::{GuardContext, GuardRunningStep, GuardScope, GuardService};
use rekall_http::guard::GuardContextResponse;
use tracing::warn;

use super::context_source::GuardContextSource;
use super::guard_mode::GuardMode;
use super::guard_outcome::GuardOutcome;
use super::hook_input::HookInput;

pub async fn run(
    provider: SessionProvider,
    mode: GuardMode,
    input_text: &str,
    scope: Option<GuardScope>,
    source: &dyn GuardContextSource,
) -> GuardOutcome {
    let input = match serde_json::from_str::<HookInput>(input_text) {
        Ok(input) => input,
        Err(failure) => {
            warn!(%failure, "The guard could not read the hook's input and let the action through");
            return GuardOutcome::allow();
        }
    };
    let Some(event) = input.event(provider, mode) else {
        warn!(%provider, ?mode, "The guard received an incomplete or mismatched event and let it through");
        return GuardOutcome::allow();
    };
    if mode == GuardMode::BeforeStop && input.stop_hook_active {
        return GuardOutcome::allow();
    }
    let context = match source.context_of(&input.cwd, scope).await {
        Ok(response) => context_of(response),
        Err(failure) => {
            warn!(%failure, folder = %input.cwd, "The guard could not reach Rekall and let the action through");
            return GuardOutcome::allow();
        }
    };
    GuardOutcome::of(provider, mode, GuardService::decide(&event, &context))
}

fn context_of(response: GuardContextResponse) -> GuardContext {
    GuardContext {
        auto_commit: response.auto_commit,
        running_step: response.running_step.map(|step| GuardRunningStep { number: step.number, task_anchor: step.task_anchor }),
        protected_credential_references: response.protected_credential_references,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/guard_command/guard_runner_tests.rs"]
mod tests;
