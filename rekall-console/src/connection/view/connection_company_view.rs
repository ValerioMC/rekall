use rekall_kernel::Id;
use rekall_store::company;

/// A company a connection is shared with.
#[derive(Clone, Debug)]
pub struct ConnectionCompanyView {
    pub id: Id,
    pub name: String,
}

impl ConnectionCompanyView {
    pub fn of(company: &company::Model) -> Self {
        Self { id: company.id, name: company.name.clone() }
    }
}
