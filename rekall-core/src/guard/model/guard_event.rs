#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardEvent {
    BeforeTool { shell_command: Option<String>, input_texts: Vec<String> },
    BeforeStop,
}
