use rekall_core::guard::GuardScope;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContextCommand {
    pub folder: String,
    pub scope: Option<GuardScope>,
}
