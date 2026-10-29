//! `045`: a Jira connection may name the proxy its calls go through, for a network that only
//! reaches the site that way. Null goes direct.

use super::{changeset, exec};

changeset!(ConnectionProxy, "045-connection-proxy", |manager| {
    exec(manager, &["ALTER TABLE external_connection ADD COLUMN proxy_url VARCHAR(1000)"]).await
});
