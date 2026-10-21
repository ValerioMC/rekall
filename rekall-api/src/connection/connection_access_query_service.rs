use std::collections::HashMap;

use rekall_common::{Id, RekallError, Result};
use rekall_repository::repository as repo;
use rekall_service::{in_read, Ctx};

use super::{ConnectionAccessQuery, ConnectionAccessResponse};

const DEFAULT_PAGE: u64 = 50;
const LARGEST_PAGE: u64 = 200;

/// Reads a connection's access log a page at a time, newest first, naming the task of each request.
#[derive(Clone)]
pub struct ConnectionAccessQueryService {
    ctx: Ctx,
}

impl ConnectionAccessQueryService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn page(&self, connection_id: Id, query: ConnectionAccessQuery) -> Result<Vec<ConnectionAccessResponse>> {
        let limit = query.limit.unwrap_or(DEFAULT_PAGE).clamp(1, LARGEST_PAGE);
        in_read!(&self.ctx, |tx| {
            repo::external_connection::require(tx.db(), connection_id).await?;
            let accesses = repo::connection_access::list_of_connection(tx.db(), connection_id, query.offset.unwrap_or(0), limit).await?;
            let task_ids: Vec<Id> = accesses.iter().filter_map(|access| access.task_id).collect();
            let tasks: HashMap<Id, _> = repo::task::find_by_ids(tx.db(), task_ids).await?.into_iter().map(|task| (task.id, task)).collect();
            Ok::<_, RekallError>(accesses.iter().map(|access| ConnectionAccessResponse::of(access, access.task_id.and_then(|id| tasks.get(&id)))).collect())
        })
    }
}
