use rekall_console::diagram::SourceLineView;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLineResponse {
    pub number: u32,
    pub text: String,
}

impl From<SourceLineView> for SourceLineResponse {
    fn from(view: SourceLineView) -> Self {
        Self {
            number: view.number,
            text: view.text,
        }
    }
}
