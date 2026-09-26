/// A fixed-size ring of recent PTY bytes so a late-opening pane repaints. A snapshot may start
/// mid escape-sequence and flicker once on attach.
pub(super) struct Scrollback {
    pub(super) ring: Vec<u8>,
    pub(super) size: usize,
    pub(super) start: usize,
}

impl Scrollback {
    pub(super) fn new(capacity: usize) -> Self {
        Self { ring: vec![0; capacity.max(1)], size: 0, start: 0 }
    }

    pub(super) fn append(&mut self, data: &[u8]) {
        let capacity = self.ring.len();
        for byte in data {
            self.ring[(self.start + self.size) % capacity] = *byte;
            if self.size < capacity {
                self.size += 1;
            } else {
                self.start = (self.start + 1) % capacity;
            }
        }
    }

    pub(super) fn snapshot(&self) -> Vec<u8> {
        let capacity = self.ring.len();
        (0..self.size).map(|at| self.ring[(self.start + at) % capacity]).collect()
    }
}
