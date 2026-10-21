//! One request a terminal made for a connection's credential, granted or refused. Append-only, and
//! it never holds the credential: it is what the console shows a person about what a session reached.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const TERMINAL_ID_MAX_CHARACTERS: usize = 64;
pub const REASON_MAX_CHARACTERS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "connection_access")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub connection_id: Id,
    /// The task the terminal was opened on; none for a project terminal or once the task is deleted.
    pub task_id: Option<Id>,
    pub terminal_id: String,
    pub accessed_at: Instant,
    pub granted: bool,
    /// Why a request was refused; a granted one has none.
    pub reason: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::external_connection::Entity",
        from = "Column::ConnectionId",
        to = "crate::external_connection::Column::Id",
        on_delete = "Cascade"
    )]
    Connection,
    #[sea_orm(belongs_to = "crate::task::Entity", from = "Column::TaskId", to = "crate::task::Column::Id", on_delete = "SetNull")]
    Task,
}

impl Related<crate::external_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Connection.def()
    }
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn granted(connection_id: Id, task_id: Option<Id>, terminal_id: String, now: Instant) -> Self {
        Self { id: Id::random(), connection_id, task_id, terminal_id, accessed_at: now, granted: true, reason: None }
    }

    pub fn refused(connection_id: Id, task_id: Option<Id>, terminal_id: String, reason: String, now: Instant) -> Self {
        Self { id: Id::random(), connection_id, task_id, terminal_id, accessed_at: now, granted: false, reason: Some(reason) }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        if self.granted == self.reason.is_some() {
            return Err(RekallError::illegal("A refused access says why, and a granted one carries no reason."));
        }
        Violations::new("ConnectionAccess")
            .not_blank("terminalId", Some(&self.terminal_id))
            .max_length("terminalId", Some(&self.terminal_id), TERMINAL_ID_MAX_CHARACTERS)
            .max_length("reason", self.reason.as_deref(), REASON_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/connection_access_tests.rs"]
mod tests;
