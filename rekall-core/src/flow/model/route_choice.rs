//! Which route a claimed step leaves by. The rule a flow turns on, kept apart from the writes so
//! it reads and tests on its own.

use rekall_kernel::{RekallError, Result};
use rekall_store::step_link;

/// The route `outcome` names. Without one, the only route is taken and none ends the flow (`None`);
/// several is refused, since only the session knows what happened.
pub fn choose_route<'a>(
    step_title: &str,
    outgoing: &'a [step_link::Model],
    outcome: Option<&str>,
) -> Result<Option<&'a step_link::Model>> {
    match outcome {
        Some(outcome) => route_named(step_title, outgoing, outcome).map(Some),
        None => route_without_outcome(step_title, outgoing),
    }
}

fn route_named<'a>(step_title: &str, outgoing: &'a [step_link::Model], outcome: &str) -> Result<&'a step_link::Model> {
    if outgoing.is_empty() {
        return Err(RekallError::illegal(format!(
            "'{step_title}' has no routes out: it ends the flow. Claim it without an outcome."
        )));
    }
    outgoing.iter().find(|link| link.outcome == outcome).ok_or_else(|| {
        RekallError::illegal(format!(
            "'{step_title}' has no route for `{outcome}`. It routes by {}: pass the one that happened.",
            outcomes(outgoing)
        ))
    })
}

fn route_without_outcome<'a>(step_title: &str, outgoing: &'a [step_link::Model]) -> Result<Option<&'a step_link::Model>> {
    match outgoing {
        [] => Ok(None),
        [only] => Ok(Some(only)),
        _ => Err(RekallError::illegal(format!(
            "'{step_title}' routes by {}. Claim it with the `outcome` that happened.",
            outcomes(outgoing)
        ))),
    }
}

fn outcomes(outgoing: &[step_link::Model]) -> String {
    outgoing.iter().map(|link| format!("`{}`", link.outcome)).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
#[path = "../../../tests/unit/flow/model/route_choice_tests.rs"]
mod tests;
