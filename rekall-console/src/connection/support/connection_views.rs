//! A connection read together with what the console shows beside it, inside the caller's transaction.

use rekall_kernel::Result;
use rekall_store::external_connection;
use rekall_store::repository as repo;
use rekall_core::Tx;

use crate::connection::ConnectionView;

pub(in crate::connection) async fn view_in(tx: &Tx, connection: &external_connection::Model) -> Result<ConnectionView> {
    let company_ids = repo::connection_company::company_ids_of_connection(tx.db(), connection.id).await?;
    let project_count = repo::project_connection::project_count(tx.db(), connection.id).await?;
    Ok(ConnectionView::of(connection, company_ids, project_count))
}
