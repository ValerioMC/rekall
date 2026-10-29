use std::sync::Mutex;

use rekall_kernel::RekallError;

use super::*;

/// A pipe to one terminal, `open`, that records what is written to it and how it is sized.
struct FakePipe {
    open: Id,
    written: Mutex<Vec<u8>>,
    sizes: Mutex<Vec<(i64, i64)>>,
    detached: Mutex<Vec<u64>>,
}

impl TerminalPipe for FakePipe {
    fn attach(&self, _id: Id, _listener: Arc<dyn TerminalListener>) -> Result<(TerminalView, u64)> {
        Err(RekallError::conflict("Not used by these tests."))
    }

    fn detach(&self, _id: Id, token: u64) {
        self.detached.lock().unwrap().push(token);
    }

    fn write(&self, id: Id, data: &[u8]) -> Result<()> {
        if id != self.open {
            return Err(RekallError::conflict("This terminal has closed."));
        }
        self.written.lock().unwrap().extend_from_slice(data);
        Ok(())
    }

    fn resize(&self, _id: Id, columns: i64, rows: i64) {
        self.sizes.lock().unwrap().push((columns, rows));
    }
}

#[test]
fn input_size_and_detaching_reach_the_pipe_and_its_refusal_comes_back() {
    let open = Id::random();
    let pipe = Arc::new(FakePipe { open, written: Mutex::default(), sizes: Mutex::default(), detached: Mutex::default() });
    let service = TerminalPipeService::new(pipe.clone());

    service.write(open, b"ls\n").unwrap();
    service.resize(open, 120, 40);
    service.detach(open, 7);
    let refused = service.write(Id::random(), b"ls\n");

    assert_eq!(pipe.written.lock().unwrap().as_slice(), b"ls\n");
    assert_eq!(pipe.sizes.lock().unwrap().as_slice(), &[(120, 40)]);
    assert_eq!(pipe.detached.lock().unwrap().as_slice(), &[7]);
    assert!(matches!(refused, Err(RekallError::Conflict(_))));
}
