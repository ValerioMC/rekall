use std::path::Path;
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

use super::codex_chat::{CodexChat, CodexInput};
use super::stream_parser::StreamParser;
use super::wire::{interrupt_line, permission_answer_line, user_message_line};
use crate::session::SessionOptions;

const EVENT_BUFFER: usize = 256;
const CODEX_STARTUP: Duration = Duration::from_secs(10);
/// How long the last events may take to be read once the process is gone. A grandchild holding
/// the pipe open must not keep the session from ending.
const READER_DRAIN: Duration = Duration::from_secs(2);

pub struct ChatProcess {
    pid: Option<u32>,
    input: ChatInput,
    exit: watch::Receiver<Option<i32>>,
    kill_now: Arc<Notify>,
    interrupts: AtomicU64,
}

enum ChatInput {
    Claude(mpsc::UnboundedSender<String>),
    Codex(mpsc::UnboundedSender<CodexInput>),
}

/// Starts `command` with `first_message` as its first stdin write. The events it prints arrive on
/// the receiver, which closes once the process has ended and everything it printed was read.
pub fn spawn_chat_process(id: Id, mut command: Command, first_message: &str) -> Result<(ChatProcess, mpsc::Receiver<ChatBody>)> {
    let mut child = command.spawn().map_err(|failed| RekallError::conflict(format!("Could not start claude: {failed}")))?;
    let pipes = take_pipes(&mut child)?;
    let (input, lines) = mpsc::unbounded_channel();
    let (events_sender, events) = mpsc::channel(EVENT_BUFFER);

    let first = user_message_line(first_message);
    input.send(first).map_err(|_| RekallError::conflict("Could not start claude: its input closed."))?;
    tokio::spawn(write_stdin(id, pipes.stdin, lines));
    tokio::spawn(drain_stderr(id, pipes.stderr));
    let reader = tokio::spawn(read_stdout(id, pipes.stdout, events_sender));

    let process = ChatProcess::follow(id, child, ChatInput::Claude(input), reader);
    Ok((process, events))
}

pub(super) async fn spawn_codex_process(
    id: Id,
    command: Command,
    options: &SessionOptions,
    directory: &Path,
    first_message: &str,
) -> Result<(ChatProcess, mpsc::Receiver<ChatBody>)> {
    let (mut child, chat) = tokio::time::timeout(CODEX_STARTUP, CodexChat::open(command, options, directory))
        .await
        .map_err(|_| codex_startup_failure("The app server did not respond in time."))?
        .map_err(codex_startup_failure)?;
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(drain_stderr(id, stderr));
    }
    let (input, commands) = mpsc::unbounded_channel();
    let (sender, events) = mpsc::channel(EVENT_BUFFER);
    let reader = tokio::spawn(chat.run(id, first_message.to_owned(), commands, sender));
    Ok((ChatProcess::follow(id, child, ChatInput::Codex(input), reader), events))
}

fn codex_startup_failure(error: impl std::fmt::Display) -> RekallError {
    RekallError::conflict(format!("Codex Chat could not start. Try updating the Codex CLI. {error}"))
}

impl ChatProcess {
    fn follow(id: Id, child: Child, input: ChatInput, reader: JoinHandle<()>) -> Self {
        let pid = child.id();
        let (exit_sender, exit) = watch::channel(None);
        let kill_now = Arc::new(Notify::new());
        tokio::spawn(wait_for_exit(id, child, reader, kill_now.clone(), exit_sender));
        Self { pid, input, exit, kill_now, interrupts: AtomicU64::new(0) }
    }

    pub fn alive(&self) -> bool {
        self.exit.borrow().is_none()
    }

    /// The exit code, or `None` while it runs. A process ended by a signal has -1.
    pub fn exit_code(&self) -> Option<i32> {
        *self.exit.borrow()
    }

    /// Queues a user message. A process that has ended refuses it.
    pub fn send(&self, text: &str) -> Result<()> {
        match &self.input {
            ChatInput::Claude(input) => self.write(input, user_message_line(text)),
            ChatInput::Codex(input) => self.write(input, CodexInput::Message(text.to_owned())),
        }
    }

    /// Ends the running turn; the process stays up for the next message.
    pub fn interrupt(&self) -> Result<()> {
        match &self.input {
            ChatInput::Claude(input) => {
                let request_id = format!("interrupt-{}", self.interrupts.fetch_add(1, Ordering::SeqCst) + 1);
                self.write(input, interrupt_line(&request_id))
            }
            ChatInput::Codex(input) => self.write(input, CodexInput::Interrupt),
        }
    }

    /// Answers the permission request `request_id`, which the process is waiting on.
    pub fn answer_permission(&self, request_id: &str, allow: bool) -> Result<()> {
        match &self.input {
            ChatInput::Claude(input) => self.write(input, permission_answer_line(request_id, allow)),
            ChatInput::Codex(input) => self.write(input, CodexInput::Approval { request_id: request_id.to_owned(), allow }),
        }
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

    fn write<T>(&self, input: &mpsc::UnboundedSender<T>, value: T) -> Result<()> {
        if !self.alive() {
            return Err(RekallError::conflict("This chat session has closed."));
        }
        input.send(value).map_err(|_| RekallError::conflict("This chat session is no longer accepting input."))
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
        warn!(chat_id = %id, stderr = %line, "Chat provider wrote to stderr");
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
        warn!(chat_id = %id, "Chat output stayed open after the process exited");
    }
    let code = match status {
        Ok(status) => status.code().unwrap_or(-1),
        Err(failed) => {
            warn!(%failed, chat_id = %id, "Could not read how the chat process ended");
            -1
        }
    };
    let _ = exit.send(Some(code));
}

#[cfg(test)]
#[path = "../../tests/unit/chat/chat_process_tests.rs"]
mod tests;
