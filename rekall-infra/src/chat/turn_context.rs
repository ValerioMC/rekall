//! The context fill a finished turn left: what its last API call held over the window of the model
//! that ran it.

use rekall_core::chat::ContextFill;

use super::wire::{WireModelUsage, WireResult};

/// The tokens of the last entry of `usage.iterations`: `result.usage` as a whole sums every call of
/// the turn and would overstate it. `None` when `claude` reported no iteration or no window.
pub(super) fn context_of(result: &WireResult) -> Option<ContextFill> {
    let last_call = result.usage.as_ref()?.iterations.last()?;
    let window_tokens = busiest_model(result)?.context_window.filter(|window| *window > 0)?;
    let used_tokens = last_call.input_tokens + last_call.cache_read_input_tokens + last_call.cache_creation_input_tokens;
    Some(ContextFill { used_tokens, window_tokens })
}

fn busiest_model(result: &WireResult) -> Option<&WireModelUsage> {
    result.model_usage.values().max_by_key(|model| model.output_tokens)
}
