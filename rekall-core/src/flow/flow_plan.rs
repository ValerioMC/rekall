//! A flow a planning session proposes whole: drafts in order, the first where the flow begins, and
//! the routes between them. Its graph rules are checked here, before anything is written, so a
//! refused plan names what to change instead of leaving half a graph on the shelf.

use std::collections::HashSet;

use rekall_kernel::{jstr, RekallError, Result};
use rekall_store::step_link::{self, ROUTES_PER_STEP_MAX};

use super::PlannedStep;
use crate::step::PROPOSED_DRAFTS_MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowPlan {
    pub steps: Vec<PlannedStep>,
}

impl FlowPlan {
    pub fn new(steps: Vec<PlannedStep>) -> Self {
        Self { steps }
    }

    pub fn route_count(&self) -> usize {
        self.steps.iter().map(|step| step.routes.len()).sum()
    }

    /// Refuses a plan that could not run unattended: a step the flow never reaches, or a loop with
    /// no way out, would stall a session that follows the routes.
    pub fn check(&self) -> Result<()> {
        self.check_shape()?;
        self.require_a_route()?;
        self.require_every_step_reached()?;
        self.require_every_step_can_end()
    }

    /// Refuses what no flow could hold: no steps or too many, a title twice, a route that leaves
    /// its step badly. A plan that passes can be stored, even if it cannot run yet.
    pub fn check_shape(&self) -> Result<()> {
        self.require_size()?;
        self.require_distinct_titles()?;
        for (at, step) in self.steps.iter().enumerate() {
            self.require_routes_fit(at, step)?;
        }
        Ok(())
    }

    fn require_size(&self) -> Result<()> {
        if self.steps.is_empty() {
            return Err(RekallError::illegal("A flow needs at least one step."));
        }
        if self.steps.len() > PROPOSED_DRAFTS_MAX {
            return Err(RekallError::illegal(format!(
                "A task holds at most {PROPOSED_DRAFTS_MAX} drafts and this flow has {} steps. Merge steps that \
                 always follow one another.",
                self.steps.len()
            )));
        }
        Ok(())
    }

    fn require_distinct_titles(&self) -> Result<()> {
        for (at, step) in self.steps.iter().enumerate() {
            let repeated = self.steps[..at]
                .iter()
                .any(|earlier| jstr::equals_ignore_case(jstr::strip(&earlier.title), jstr::strip(&step.title)));
            if repeated {
                return Err(RekallError::illegal(format!("The step '{}' appears twice. Nothing was proposed.", step.title)));
            }
        }
        Ok(())
    }

    /// One outcome names one route, a route leads to another step of the plan, and a step
    /// decides between at most `ROUTES_PER_STEP_MAX`.
    fn require_routes_fit(&self, at: usize, step: &PlannedStep) -> Result<()> {
        if step.routes.len() > ROUTES_PER_STEP_MAX {
            return Err(RekallError::illegal(format!(
                "'{}' has {} routes out and a step decides between at most {ROUTES_PER_STEP_MAX}. Split it.",
                step.title,
                step.routes.len()
            )));
        }
        let mut outcomes = HashSet::new();
        for route in &step.routes {
            let outcome = step_link::normalise_outcome(Some(&route.outcome))?;
            if !outcomes.insert(outcome.clone()) {
                return Err(RekallError::illegal(format!(
                    "'{}' routes by `{outcome}` twice. One outcome names one route.",
                    step.title
                )));
            }
            match route.to {
                Some(to) if to == at => {
                    return Err(RekallError::illegal(format!(
                        "'{}' routes to itself by `{outcome}`. A step that repeats is reached again through another step.",
                        step.title
                    )));
                }
                Some(to) if to >= self.steps.len() => {
                    return Err(RekallError::illegal(format!(
                        "'{}' routes by `{outcome}` to step {}, and the flow has {} steps.",
                        step.title,
                        to + 1,
                        self.steps.len()
                    )));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn require_a_route(&self) -> Result<()> {
        if self.route_count() == 0 {
            return Err(RekallError::illegal(
                "No step routes anywhere, so this is a checklist, not a flow. Propose it with `rekall_propose_step`.",
            ));
        }
        Ok(())
    }

    /// The flow begins at the first step, so a step no route leads to from there never runs.
    fn require_every_step_reached(&self) -> Result<()> {
        let mut reached = vec![false; self.steps.len()];
        let mut pending = vec![0];
        while let Some(at) = pending.pop() {
            if std::mem::replace(&mut reached[at], true) {
                continue;
            }
            pending.extend(self.steps[at].routes.iter().filter_map(|route| route.to));
        }
        self.refuse_unless_all(&reached, "The flow begins at the first step and never reaches", "Route to them, or drop them.")
    }

    /// A step ends the flow when it has no routes or one to the end; any other step must lead to
    /// one that does, or the work could loop forever with no session able to stop it.
    fn require_every_step_can_end(&self) -> Result<()> {
        let mut ends: Vec<bool> = self
            .steps
            .iter()
            .map(|step| step.routes.is_empty() || step.routes.iter().any(|route| route.to.is_none()))
            .collect();
        let mut changed = true;
        while changed {
            changed = false;
            for (at, step) in self.steps.iter().enumerate() {
                if !ends[at] && step.routes.iter().any(|route| route.to.is_some_and(|to| ends[to])) {
                    ends[at] = true;
                    changed = true;
                }
            }
        }
        self.refuse_unless_all(
            &ends,
            "The flow can loop forever through",
            "Give one of them a route to `end`, or to a step that ends the flow.",
        )
    }

    fn refuse_unless_all(&self, holds: &[bool], problem: &str, remedy: &str) -> Result<()> {
        let failing: Vec<String> =
            self.steps.iter().zip(holds).filter(|(_, holds)| !**holds).map(|(step, _)| format!("'{}'", step.title)).collect();
        if failing.is_empty() {
            return Ok(());
        }
        Err(RekallError::illegal(format!("{problem} {}. {remedy}", failing.join(", "))))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/flow_plan_tests.rs"]
mod tests;
