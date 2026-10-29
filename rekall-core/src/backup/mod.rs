//! Rotating copies of the open database in a `backups` folder beside it, and putting one back.
//! Both touch the disk, so each is a port that `rekall-infra` implements.

mod model;
mod port;

pub use model::backup_file::BackupFile;
pub use model::backup_reason::BackupReason;
pub use model::backup_status::BackupStatus;
pub use port::database_backups::DatabaseBackups;
pub use port::database_restore::{DatabaseRestore, UploadChunks};
