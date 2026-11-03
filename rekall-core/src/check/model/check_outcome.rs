/// How a command ended: with an exit code (`-1` when a signal ended it), or still running at its deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckOutcome {
    Exited(i32),
    TimedOut,
}

impl CheckOutcome {
    pub fn passed(self) -> bool {
        self == Self::Exited(0)
    }

    pub fn exit_code(self) -> Option<i32> {
        match self {
            Self::Exited(code) => Some(code),
            Self::TimedOut => None,
        }
    }
}
