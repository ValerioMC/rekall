use std::fmt;
use std::str::FromStr;

use rekall_kernel::Id;

pub const GUARD_PORT_ENV: &str = "REKALL_GUARD_PORT";
pub const GUARD_SCOPE_ENV: &str = "REKALL_GUARD_SCOPE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardScope {
    Work(Id),
    WithoutStep,
}

impl fmt::Display for GuardScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work(task_id) => task_id.fmt(f),
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
        value.parse().map(Self::Work).map_err(|_| "guard scope must be a task ID or without-step")
    }
}
