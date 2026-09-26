#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStatus {
    Ready,
    Unreachable,
    SetupNeeded,
}

impl SetupStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Unreachable => "UNREACHABLE",
            Self::SetupNeeded => "SETUP_NEEDED",
        }
    }
}
