//! `043`: a diagram says whether one of its sheets plays as a walkthrough, so the gallery can mark
//! it without reading the document. No stored document carries one yet.

use super::{changeset, exec};

changeset!(DiagramWalkthrough, "043-diagram-walkthrough", |manager| {
    exec(manager, &["ALTER TABLE diagram ADD COLUMN walkthrough BOOLEAN NOT NULL DEFAULT 0"]).await
});
