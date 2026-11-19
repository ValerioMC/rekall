/// A person's edit of a step: only what is `Some` changes.
#[derive(Clone, Copy, Debug, Default)]
pub struct StepEdit<'a> {
    pub title: Option<&'a str>,
    pub body_markdown: Option<&'a str>,
    pub done: Option<bool>,
    pub draft: Option<bool>,
}
