//! A row whose `updated_at` moves only when one of its values does, and that is validated before
//! every update. `rekall_repository::repository::update_if_changed` is the one write that uses it.

use rekall_common::{Instant, RekallError};

pub trait Stamped {
    fn stamp(&mut self, now: Instant);

    fn validate_update(&self) -> Result<(), RekallError>;
}

/// Implements `Stamped` for entity modules whose `Model` has `updated_at` and `validate()`.
macro_rules! stamped {
    ($($entity:ident),+ $(,)?) => {$(
        impl Stamped for crate::$entity::Model {
            fn stamp(&mut self, now: Instant) {
                self.updated_at = now;
            }

            fn validate_update(&self) -> Result<(), RekallError> {
                self.validate()
            }
        }
    )+};
}

stamped!(company, project, task, task_step, document, wrapup, tag, project_section, project_component, step_link, external_connection, project_connection, security_advisory, vulnerability_exposure, scan_scope, deployment_graph);
