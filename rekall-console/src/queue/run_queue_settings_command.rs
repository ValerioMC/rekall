/// The queue's settings: no ceiling when `ceiling_percent` is none, the account's model and effort
/// when those are.
#[derive(Clone, Debug, Default)]
pub struct RunQueueSettingsCommand {
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
}
