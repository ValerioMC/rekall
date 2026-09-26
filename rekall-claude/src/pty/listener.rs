/// A sink for one terminal's output and the moment it ends. Implemented by the socket handler.
pub trait Listener: Send + Sync {
    fn output(&self, data: &[u8]);
    fn ended(&self, exit_code: i32, detail: &str);
}
