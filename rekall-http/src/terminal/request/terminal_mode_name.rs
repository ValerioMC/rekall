use serde::Deserialize;

/// The mode's name as the console sends it; a generate request and its kinds travel beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TerminalModeName {
    Work,
    Plan,
    #[serde(rename = "PLAN_FLOW")]
    PlanFlow,
    Replan,
    Revise,
    Generate,
    Explain,
    Inventory,
    Assess,
    #[serde(rename = "ASSESS_ALL")]
    AssessAll,
    Autoconfigure,
    #[serde(rename = "REWRITE_FIX")]
    RewriteFix,
    #[serde(rename = "REWRITE_RESTRUCTURE")]
    RewriteRestructure,
}
