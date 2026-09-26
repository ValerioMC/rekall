//! The two kinds of limit an entity carried, kept apart because they failed differently.
//!
//! - Bean validation (`@NotBlank`, `@Size`, `@Pattern` on the entity) ran when Hibernate
//!   persisted or updated the row and threw `ConstraintViolationException`, which no handler
//!   mapped, so it reached the console as a 500 naming the violation.
//! - A column's declared length (`varchar(100000)` and the like, with no `@Size` beside it) was
//!   enforced by H2 itself, as a `DataIntegrityViolationException`, which the console showed as a
//!   409. SQLite enforces no length, so the same check is made here before the write.
//!
//! Both measure length in UTF-16 code units, as `String.length()` and H2 did.

mod phase;
mod violations;

pub use phase::Phase;
pub use violations::Violations;
