use rekall_console::catalog::CompanyCommand;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompanyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl From<CompanyRequest> for CompanyCommand {
    fn from(request: CompanyRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
        }
    }
}
