//! A chat session's process: the headless `claude` with a task writing its stdin, a task reading
//! its stdout into events, a drain for its stderr, and the one that waits for it to exit.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rekall_core::chat::ChatBody;
use rekall_kernel::{Id, RekallError, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, watch, Notify};
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use super::stream_parser::StreamParser;
use super::wire::{interrupt_line, permission_answer_line, user_message_line};

const EVENT_BUFFER: usize = 256;
/// How long the last events may take to be read once the process is gone. A grandchild holding
/// the pipe open must not keep the session from ending.
const READER_DRAIN: Duration = Duration::from_secs(2);

/// The handle on a running headless `claude`. Dropping it closes the process's stdin, which ends it.
pub struct ChatProcess {
    pid: Option<u32>,
    input: mpsc::UnboundedSender<String>,
    exit: watch::Receiver<Option<i32>>,
    kill_now: Arc<Notify>,
    interrupts: AtomicU64,
}

/// Starts `command` with `first_message` as its first stdin write. The events it prints arrive on
/// the receiver, which closes once the process has ended and everything it printed was read.
pub fn spawn_chat_process(id: Id, mut command: Command, first_message: &str) -> Result<(ChatProcess, mpsc::Receiver<ChatBody>)> {
    let mut child = command.spawn().map_err(|failed| RekallError::conflict(format!("Could not start claude: {failed}")))?;
    let pipes = take_pipes(&mut child)?;
    let pid = child.id();
    let (input, lines) = mpsc::unbounded_channel();
    let (events_sender, events) = mpsc::channel(EVENT_BUFFER);
    let (exit_sender, exit) = watch::channel(None);
    let kill_now = Arc::new(Notify::new());

    let first = user_message_line(first_message);
    input.send(first).map_err(|_| RekallError::conflict("Could not start claude: its input closed."))?;
    tokio::spawn(write_stdin(id, pipes.stdin, lines));
    tokio::spawn(drain_stderr(id, pipes.stderr));
    let reader = tokio::spawn(read_stdout(id, pipes.stdout, events_sender));
    tokio::spawn(wait_for_exit(id, child, reader, kill_now.clone(), exit_sender));

    let process = ChatProcess { pid, input, exit, kill_now, interrupts: AtomicU64::new(0) };
    Ok((process, events))
}

impl ChatProcess {
    pub fn alive(&self) -> bool {
        self.exit.borrow().is_none()
    }

    /// The exit code, or `None` while it runs. A process ended by a signal has -1.
    pub fn exit_code(&self) -> Option<i32> {
        *self.exit.borrow()
    }

    /// Queues a user message. A process that has ended refuses it.
    pub fn send(&self, text: &str) -> Result<()> {
        self.write_line(user_message_line(text))
    }

    /// Ends the running turn; the process stays up for the next message.
    pub fn interrupt(&self) -> Result<()> {
        let request_id = format!("interrupt-{}", self.interrupts.fetch_add(1, Ordering::SeqCst) + 1);
        self.write_line(interrupt_line(&request_id))
    }

    /// Answers the permission request `request_id`, which the process is waiting on.
    pub fn answer_permission(&self, request_id: &str, allow: bool) -> Result<()> {
        self.write_line(permission_answer_line(request_id, allow))
    }

    /// Asks the process to end, with SIGTERM where there is one.
    pub fn terminate(&self) {
        if !self.alive() {
            return;
        }
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            return;
        }
        self.kill_now.notify_one();
    }

    /// Ends the process outright.
    pub fn kill(&self) {
        self.kill_now.notify_one();
    }

    pub async fn wait_for(&self, timeout: Duration) -> bool {
        let mut exit = self.exit.clone();
        let exited = tokio::time::timeout(timeout, exit.wait_for(Option::is_some)).await.is_ok();
        exited
    }

    fn write_line(&self, line: String) -> Result<()> {
        if !self.alive() {
            return Err(RekallError::conflict("This chat session has closed."));
        }
        self.input.send(line).map_err(|_| RekallError::conflict("This chat session is no longer accepting input."))
    }
}

struct Pipes {
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

fn take_pipes(child: &mut Child) -> Result<Pipes> {
    match (child.stdin.take(), child.stdout.take(), child.stderr.take()) {
        (Some(stdin), Some(stdout), Some(stderr)) => Ok(Pipes { stdin, stdout, stderr }),
        _ => Err(RekallError::conflict("Could not start claude: its pipes were not opened.")),
    }
}

async fn write_stdin(id: Id, mut stdin: ChildStdin, mut lines: mpsc::UnboundedReceiver<String>) {
    while let Some(line) = lines.recv().await {
        let written = async {
            stdin.write_all(line.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await
        };
        if let Err(failed) = written.await {
            debug!(%failed, chat_id = %id, "Chat input closed");
            break;
        }
    }
}

async fn read_stdout(id: Id, stdout: ChildStdout, events: mpsc::Sender<ChatBody>) {
    let mut parser = StreamParser::new();
    let mut lines = BufReader::new(stdout).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                for body in parser.parse_line(&line) {
                    if events.send(body).await.is_err() {
                        return;
                    }
                }
            }
            Ok(None) => return,
            Err(failed) => {
                warn!(%failed, chat_id = %id, "Could not read claude's output");
                return;
            }
        }
    }
}

async fn drain_stderr(id: Id, stderr: ChildStderr) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        warn!(chat_id = %id, stderr = %line, "claude wrote to stderr");
    }
}

async fn wait_for_exit(id: Id, mut child: Child, reader: JoinHandle<()>, kill_now: Arc<Notify>, exit: watch::Sender<Option<i32>>) {
    let status = tokio::select! {
        status = child.wait() => status,
        _ = kill_now.notified() => match child.kill().await {
            Ok(()) => child.wait().await,
            Err(failed) => {
                debug!(%failed, chat_id = %id, "Kill found the chat process already gone");
                child.wait().await
            }
        },
    };
    if tokio::time::timeout(READER_DRAIN, reader).await.is_err() {
        warn!(chat_id = %id, "claude's output stayed open after it exited");
    }
    let code = match status {
        Ok(status) => status.code().unwrap_or(-1),
        Err(failed) => {
            warn!(%failed, chat_id = %id, "Could not read how claude ended");
            -1
        }
    };
    let _ = exit.send(Some(code));
}

#[cfg(test)]
#[path = "../../tests/unit/chat/chat_process_tests.rs"]
mod tests;
