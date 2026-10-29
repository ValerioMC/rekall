use std::ops::AddAssign;

/// The four counts the API reports for a model call. Input read from the cache is counted apart
/// from fresh input, since it costs a tenth as much.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
}

impl TokenCount {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl AddAssign for TokenCount {
    fn add_assign(&mut self, other: Self) {
        self.input += other.input;
        self.output += other.output;
        self.cache_creation += other.cache_creation;
        self.cache_read += other.cache_read;
    }
}
