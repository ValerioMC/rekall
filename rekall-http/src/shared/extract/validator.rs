use rekall_kernel::jstr;

use crate::shared::error::{ApiError, ApiResult};

/// The constraints of a request body, checked in the order they are declared.
#[derive(Default)]
pub struct Validator {
    errors: Vec<(String, String)>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Absent, empty or whitespace only is refused.
    pub fn not_blank(mut self, field: &str, value: Option<&str>) -> Self {
        if jstr::is_null_or_blank(value) {
            self.errors.push((field.into(), "must not be blank".into()));
        }
        self
    }

    /// Absent is refused.
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

/// A required body field: the value itself, or the same 400 `Validator::not_null` reports.
pub fn present<T>(field: &str, value: Option<T>) -> ApiResult<T> {
    value.ok_or_else(|| ApiError::Validation(vec![(field.into(), "must not be null".into())]))
}
