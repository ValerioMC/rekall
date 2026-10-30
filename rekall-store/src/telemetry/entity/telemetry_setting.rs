//! The Langfuse instance Claude Code's traces go to, and whether they are sent. One row, absent
//! until a person first saves it. The instance's secret key is not here: it sits in the credential store.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const HOST_MAX_CHARACTERS: usize = 300;
pub const PUBLIC_KEY_MAX_CHARACTERS: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "telemetry_setting")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub enabled: bool,
    pub host: String,
    pub public_key: String,
    /// Whether prompt and command text travel with the traces; without it they are redacted.
    pub log_content: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TelemetrySetting")
            .max_length("host", Some(&self.host), HOST_MAX_CHARACTERS)
            .max_length("publicKey", Some(&self.public_key), PUBLIC_KEY_MAX_CHARACTERS)
            .finish()
    }
}
