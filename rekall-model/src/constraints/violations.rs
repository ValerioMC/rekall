use rekall_common::{jstr, RekallError};

use super::Phase;
use crate::slug::Slug;

/// Collects bean-validation and column-length violations and reports them the way Hibernate and H2 did.
pub struct Violations {
    class: &'static str,
    phase: Phase,
    bean: Vec<(String, String, &'static str)>,
    too_long: Option<String>,
}

impl Violations {
    pub fn new(class: &'static str, phase: Phase) -> Self {
        Self { class, phase, bean: Vec::new(), too_long: None }
    }

    /// `@NotBlank`: not null, and not only whitespace.
    pub fn not_blank(&mut self, property: &str, value: Option<&str>) -> &mut Self {
        if jstr::is_null_or_blank(value) {
            self.bean.push((property.into(), "must not be blank".into(), "NotBlank"));
        }
        self
    }

    /// `@Size(max = n)`; null is valid, as it is in bean validation.
    pub fn size(&mut self, property: &str, value: Option<&str>, max: usize) -> &mut Self {
        if value.is_some_and(|v| jstr::len(v) > max) {
            self.bean.push((property.into(), format!("size must be between 0 and {max}"), "Size"));
        }
        self
    }

    /// `@Pattern(regexp = Slug.PATTERN)` with the message the entities gave it.
    pub fn slug(&mut self, property: &str, value: Option<&str>) -> &mut Self {
        if value.is_some_and(|v| !Slug::matches(v)) {
            self.bean.push((
                property.into(),
                "must be a slug: lowercase letters, digits, '-', '_' or '.'".into(),
                "Pattern",
            ));
        }
        self
    }

    /// A `varchar(n)` column H2 would have refused a longer value for.
    pub fn column(&mut self, column: &str, value: Option<&str>, max: usize) -> &mut Self {
        if self.too_long.is_none() {
            if let Some(v) = value {
                let length = jstr::len(v);
                if length > max {
                    let shown = jstr::prefix(v, 20);
                    self.too_long = Some(format!(
                        "Value too long for column \"{} CHARACTER VARYING({max})\": \"'{shown}...' ({length})\"",
                        column.to_uppercase()
                    ));
                }
            }
        }
        self
    }

    pub fn finish(&self) -> Result<(), RekallError> {
        if !self.bean.is_empty() {
            let phase = match self.phase {
                Phase::Persist => "persist",
                Phase::Update => "update",
            };
            let listed: Vec<String> = self
                .bean
                .iter()
                .map(|(property, message, constraint)| {
                    format!(
                        "\tConstraintViolationImpl{{interpolatedMessage='{message}', propertyPath={property}, \
                         rootBeanClass=class dev.rekall.domain.{}, \
                         messageTemplate='{{jakarta.validation.constraints.{constraint}.message}}'}}",
                        self.class
                    )
                })
                .collect();
            return Err(RekallError::internal(
                "ConstraintViolationException",
                format!(
                    "Validation failed for classes [dev.rekall.domain.{}] during {phase} time for groups \
                     [jakarta.validation.groups.Default, ]\nList of constraint violations:[\n{}\n]",
                    self.class,
                    listed.join("\n")
                ),
            ));
        }
        if let Some(cause) = &self.too_long {
            return Err(RekallError::Integrity(cause.clone()));
        }
        Ok(())
    }
}