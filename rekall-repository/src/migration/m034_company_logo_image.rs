//! `034`: the logo a company wears in the scope picker. A table of its own, like the project icon,
//! so the image bytes are read only when a logo is asked for and never ride along with company queries.

use super::{changeset, exec};

changeset!(CompanyLogoImage, "034-company-logo-image", |manager| {
    exec(manager, &[
        "CREATE TABLE company_logo_image (
            company_id TEXT NOT NULL CONSTRAINT pk_company_logo_image PRIMARY KEY,
            png BLOB NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_company_logo_image_company FOREIGN KEY (company_id) REFERENCES company (id) ON DELETE CASCADE)",
    ])
    .await
});
