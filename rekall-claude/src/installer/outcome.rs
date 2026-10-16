#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub exit_code: i32,
    pub output: String,
}

impl Outcome {
    pub fn succeeded(&self) -> bool {
        self.exit_code == 0
    }
}
