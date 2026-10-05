//! Request reading: a body that cannot be parsed is a 400 with the parser's reason, a path
//! variable that is not a UUID is a 400, and a missing required query parameter is a 400.

mod json_body;
mod optional_json_body;
mod path_params;
mod validator;

pub use json_body::JsonBody;
pub use optional_json_body::OptionalJsonBody;
pub use path_params::{uuid, optional_uuid, required, uuid_path_params};
pub use validator::{present, Validator};
