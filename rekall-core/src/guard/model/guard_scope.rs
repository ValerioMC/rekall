use std::fmt;
use std::str::FromStr;

use rekall_kernel::Id;

pub const GUARD_PORT_ENV: &str = "REKALL_GUARD_PORT";
pub const GUARD_SCOPE_ENV: &str = "REKALL_GUARD_SCOPE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardScope {
    Work(Id),
    ResolveMerge(Id),
    WithoutStep,
}

impl fmt::Display for GuardScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work(task_id) => task_id.fmt(f),
            Self::ResolveMerge(task_id) => write!(f, "merge:{task_id}"),
            Self::WithoutStep => f.write_str("without-step"),
        }
    }
}

impl FromStr for GuardScope {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "without-step" {
            return Ok(Self::WithoutStep);
        }
        let (task_id, scope): (&str, fn(Id) -> Self) = match value.strip_prefix("merge:") {
            Some(task_id) => (task_id, Self::ResolveMerge),
            None => (value, Self::Work),
        };
        task_id.parse().map(scope).map_err(|_| "guard scope must be a work or merge task ID, or without-step")
    }
}
