//! `DatabaseLocationEnvironmentPostProcessor`: which database this start opens, decided before
//! anything else comes up, from the registry in `rekall.home`.
//!
//! - A registry whose active folder is there: that folder's database.
//! - No registry but a `./data` folder holding a database (the layout before the registry): it is
//!   adopted as the first entry, labelled "Local", and opened.
//! - A registry whose active folder is gone (an unplugged drive): an in-memory database, so the
//!   console can come up and offer to pick another.
//! - Nothing at all: an in-memory database and the setup screen.
//!
//! `REKALL_DB_URL` (or `spring.datasource.url`) set by hand overrides the choice, as it did.

mod location_resolver;
mod resolved;
mod setup_status;

pub use location_resolver::{resolve, data_file_in};
pub use resolved::Resolved;
pub use setup_status::SetupStatus;
