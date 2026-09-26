use rekall_common::jstr;

use crate::error::{ApiError, ApiResult};

/// The `@Valid` constraints of a request body, checked in declaration order.
#[derive(Default)]
pub struct Validator {
    errors: Vec<(String, String)>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    /// `@NotBlank`.
    pub fn not_blank(mut self, field: &str, value: Option<&str>) -> Self {
        if jstr::is_null_or_blank(value) {
            self.errors.push((field.into(), "must not be blank".into()));
        }
        self
    }

    /// `@NotNull`.
    pub fn not_null<T>(mut self, field: &str, value: &Option<T>) -> Self {
        if value.is_none() {
            self.errors.push((field.into(), "must not be null".into()));
        }
        self
    }

    pub fn finish(self) -> ApiResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::Validation(self.errors))
        }
    }
}
