/// `reason` is written for the model: it says what to do instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardVerdict {
    Allow,
    Block { reason: String },
}
