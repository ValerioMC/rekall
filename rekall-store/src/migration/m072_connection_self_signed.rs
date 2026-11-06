//! `072`: a Grafana connection may accept the self-signed certificate its instance serves. Existing
//! rows keep verifying certificates.

use super::{changeset, exec};

changeset!(ConnectionSelfSigned, "072-connection-self-signed", |manager| {
    exec(manager, &["ALTER TABLE external_connection ADD COLUMN accept_self_signed BOOLEAN NOT NULL DEFAULT 0"]).await
});
