mod protocol;
mod timeline;

use std::collections::VecDeque;
use std::io;
use std::path::Path;

use rekall_core::chat::{ChatBody, NoticeKind, TurnOutcome};
use rekall_kernel::{Id, RekallError, Result};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use self::protocol::{
    Approval, ApprovalParams, ApprovalResolved, Delta, ItemNotification, ReplyResult, RequestId, RetryNotice, RpcError,
    ServerMessage, TextInput, ThreadStart, ThreadStarted, TokenUsage, TokenUsageUpdated, Turn, TurnInterrupt, TurnNotification,
    TurnStart, TurnStatus, UsageBreakdown,
};
use crate::codex::app_server::AppServer;
use crate::session::SessionOptions;

pub(super) enum CodexInput {
    Message(String),
    Interrupt,
    Approval { request_id: String, allow: bool },
}

pub(super) struct CodexChat {
    thread_id: String,
    started: ChatBody,
    effort: Option<String>,
    input: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_id: u32,
    next_approval: u64,
    turn: Option<ActiveTurn>,
    queued: VecDeque<String>,
    approvals: VecDeque<Approval>,
    previous_usage: UsageBreakdown,
}

struct ActiveTurn {
    start_request: u32,
    id: Option<String>,
    interrupt_pending: bool,
    usage: Option<TokenUsage>,
}

impl CodexChat {
    pub(super) async fn open(command: Command, options: &SessionOptions, directory: &Path) -> Result<(Child, Self)> {
        let directory = std::path::absolute(directory)
            .map_err(|failure| RekallError::conflict(format!("Could not resolve the Chat folder: {failure}")))?;
        let (mut server, version) = AppServer::start(command).await?;
        let opened: ThreadStarted = server
            .request(
                "thread/start",
                ThreadStart {
                    cwd: &directory,
                    model: options.model.as_deref(),
                    approval_policy: options.skip_permissions.then_some("never"),
                    sandbox: options.skip_permissions.then_some("danger-full-access"),
                },
            )
            .await?;
        let (child, input, output, next_id) = server.into_parts();
        let chat = Self {
            thread_id: opened.thread.id,
            started: ChatBody::SessionStarted {
                model: opened.model,
                cwd: opened.cwd,
                permission_mode: opened.approval_policy.label(),
                version,
            },
            effort: options.effort.clone(),
            input,
            lines: output.lines(),
            next_id,
            next_approval: 0,
            turn: None,
            queued: VecDeque::new(),
            approvals: VecDeque::new(),
            previous_usage: UsageBreakdown::default(),
        };
        Ok((child, chat))
    }

    pub(super) async fn run(
        mut self,
        id: Id,
        first: String,
        mut commands: mpsc::UnboundedReceiver<CodexInput>,
        events: mpsc::Sender<ChatBody>,
    ) {
        if let Err(failure) = self.follow(first, &mut commands, &events).await {
            warn!(chat_id = %id, %failure, "Codex Chat stream closed");
            let _ =
                events.send(ChatBody::notice(NoticeKind::Warning, format!("Codex Chat lost its connection. {failure}"))).await;
        }
    }

    async fn follow(
        &mut self,
        first: String,
        commands: &mut mpsc::UnboundedReceiver<CodexInput>,
        events: &mpsc::Sender<ChatBody>,
    ) -> io::Result<()> {
        emit(events, self.started.clone()).await?;
        self.start_turn(&first).await?;
        loop {
            let bodies = tokio::select! {
                line = self.lines.next_line() => {
                    let Some(line) = line? else { return Ok(()) };
                    self.receive(&line).await?
                }
                command = commands.recv() => {
                    let Some(command) = command else { return Ok(()) };
                    self.accept(command).await?
                }
            };
            for body in bodies {
                emit(events, body).await?;
            }
            self.start_next(events).await?;
        }
    }

    async fn accept(&mut self, command: CodexInput) -> io::Result<Vec<ChatBody>> {
        match command {
            CodexInput::Message(text) => self.queued.push_back(text),
            CodexInput::Interrupt => self.interrupt().await?,
            CodexInput::Approval { request_id, allow } => return self.answer(&request_id, allow).await,
        }
        Ok(Vec::new())
    }

    async fn start_turn(&mut self, text: &str) -> io::Result<()> {
        let params = TurnStart {
            thread_id: &self.thread_id,
            effort: self.effort.as_deref(),
            input: [TextInput::Text { text, text_elements: [] }],
        };
        let start_request = self.request("turn/start", json!(params)).await?;
        self.turn = Some(ActiveTurn { start_request, id: None, interrupt_pending: false, usage: None });
        Ok(())
    }

    async fn start_next(&mut self, events: &mpsc::Sender<ChatBody>) -> io::Result<()> {
        if self.turn.is_none() {
            if let Some(text) = self.queued.pop_front() {
                emit(events, ChatBody::UserMessage { text: text.clone() }).await?;
                self.start_turn(&text).await?;
            }
        }
        Ok(())
    }

    async fn interrupt(&mut self) -> io::Result<()> {
        let Some(turn) = &mut self.turn else {
            return Ok(());
        };
        if let Some(turn_id) = &turn.id {
            let params = json!(TurnInterrupt { thread_id: &self.thread_id, turn_id });
            self.request("turn/interrupt", params).await?;
        } else {
            turn.interrupt_pending = true;
        }
        Ok(())
    }

    async fn mark_started(&mut self, id: String) -> io::Result<()> {
        let Some(turn) = &mut self.turn else {
            return Ok(());
        };
        if turn.id.is_none() {
            turn.id = Some(id);
            if std::mem::take(&mut turn.interrupt_pending) {
                self.interrupt().await?;
            }
        }
        Ok(())
    }

    async fn receive(&mut self, line: &str) -> io::Result<Vec<ChatBody>> {
        match serde_json::from_str(line) {
            Ok(ServerMessage::Event { method, params, id: Some(id) }) => self.server_request(&method, id, params).await,
            Ok(ServerMessage::Event { method, params, id: None }) => self.notification(&method, params).await,
            Ok(ServerMessage::Reply { id, result, error }) => self.reply(id, result, error).await,
            Err(failure) => {
                debug!(%failure, "Ignoring malformed Codex Chat output");
                Ok(Vec::new())
            }
        }
    }

    async fn reply(&mut self, id: u32, result: Option<ReplyResult>, error: Option<RpcError>) -> io::Result<Vec<ChatBody>> {
        let starts_turn = self.turn.as_ref().is_some_and(|turn| turn.start_request == id);
        if let Some(error) = error {
            let warning = ChatBody::notice(NoticeKind::Warning, error.message);
            if starts_turn {
                return Ok(self.finish_turn(TurnOutcome::Failed, 0, Some(warning)));
            }
            return Ok(vec![warning]);
        }
        if starts_turn {
            if let Some(turn) = result.and_then(|result| result.turn) {
                self.mark_started(turn.id).await?;
            }
        }
        Ok(Vec::new())
    }

    async fn notification(&mut self, method: &str, params: Option<Value>) -> io::Result<Vec<ChatBody>> {
        let params = params.unwrap_or_default();
        match method {
            "turn/started" => {
                if let Some(reply) = self.own_turn(params) {
                    self.mark_started(reply.id).await?;
                }
            }
            "turn/completed" => {
                if let Some(turn) = self.own_turn(params) {
                    return Ok(self.complete(turn));
                }
            }
            "thread/tokenUsage/updated" => {
                if let Some(usage) = decode::<TokenUsageUpdated>(params) {
                    if self.matches_turn(&usage.thread_id, &usage.turn_id) {
                        if let Some(turn) = self.turn.as_mut() {
                            turn.usage = Some(usage.token_usage);
                        }
                    }
                }
            }
            "serverRequest/resolved" => {
                if let Some(resolved) = decode::<ApprovalResolved>(params) {
                    if resolved.thread_id == self.thread_id {
                        return Ok(self.resolve_approval(&resolved.request_id));
                    }
                }
            }
            _ => return Ok(self.timeline_event(method, params).into_iter().collect()),
        }
        Ok(Vec::new())
    }

    fn timeline_event(&self, method: &str, params: Value) -> Option<ChatBody> {
        match method {
            "item/agentMessage/delta" | "item/plan/delta" | "item/reasoning/summaryTextDelta" => {
                let delta: Delta = decode(params)?;
                if delta.thread_id != self.thread_id {
                    return None;
                }
                Some(if method == "item/reasoning/summaryTextDelta" {
                    timeline::assistant_thinking(delta.item_id, delta.delta, false)
                } else {
                    timeline::assistant_text(delta.item_id, delta.delta, false)
                })
            }
            "item/started" | "item/completed" => {
                let update: ItemNotification = decode(params)?;
                if update.thread_id != self.thread_id {
                    return None;
                }
                if method == "item/started" {
                    timeline::item_started(update.item)
                } else {
                    timeline::item_completed(update.item)
                }
            }
            "error" => {
                let retry: RetryNotice = decode(params)?;
                (retry.thread_id == self.thread_id && retry.will_retry)
                    .then(|| ChatBody::notice(NoticeKind::ApiRetry, retry.error.message))
            }
            _ => None,
        }
    }

    fn own_turn(&self, params: Value) -> Option<Turn> {
        let notice: TurnNotification = decode(params)?;
        let thread = notice.thread_id.as_deref()?;
        self.matches_turn(thread, &notice.turn.id).then_some(notice.turn)
    }

    fn matches_turn(&self, thread: &str, id: &str) -> bool {
        thread == self.thread_id && self.turn.as_ref().is_some_and(|turn| turn.id.as_deref().is_none_or(|active| active == id))
    }

    fn complete(&mut self, turn: Turn) -> Vec<ChatBody> {
        let outcome = match turn.status {
            TurnStatus::Completed => TurnOutcome::Completed,
            TurnStatus::Interrupted => TurnOutcome::Interrupted,
            TurnStatus::Failed => TurnOutcome::Failed,
            TurnStatus::InProgress => return Vec::new(),
        };
        let warning = turn.error.map(|error| ChatBody::notice(NoticeKind::Warning, error.message));
        self.finish_turn(outcome, turn.duration_ms.unwrap_or_default(), warning)
    }

    fn finish_turn(&mut self, outcome: TurnOutcome, duration_ms: u64, warning: Option<ChatBody>) -> Vec<ChatBody> {
        let usage = self.turn.take().and_then(|turn| turn.usage);
        let (tokens, context) = usage.map_or_else(Default::default, |usage| {
            let tokens = usage.total.since(self.previous_usage);
            self.previous_usage = usage.total;
            (tokens, usage.context())
        });
        self.approvals.clear();
        let finished = ChatBody::TurnFinished { usage: tokens, duration_ms, outcome, cost_usd: None, context };
        warning.into_iter().chain([finished]).collect()
    }

    async fn server_request(&mut self, method: &str, id: RequestId, params: Option<Value>) -> io::Result<Vec<ChatBody>> {
        if matches!(method, "item/commandExecution/requestApproval" | "item/fileChange/requestApproval") {
            if let Some(params) = params.and_then(decode::<ApprovalParams>) {
                if self.matches_turn(&params.thread_id, &params.turn_id) {
                    self.next_approval += 1;
                    let view_id = format!("codex-approval-{}", self.next_approval);
                    let approval = timeline::approval(method, id, view_id, params);
                    let visible = self.approvals.is_empty().then(|| approval.event.clone());
                    self.approvals.push_back(approval);
                    return Ok(visible.into_iter().collect());
                }
            }
        }
        let text = format!("Codex Chat cannot handle {method}.");
        self.write(json!({ "id": id, "error": { "code": -32601, "message": text } })).await?;
        Ok(vec![ChatBody::notice(NoticeKind::Warning, text)])
    }

    async fn answer(&mut self, request_id: &str, allow: bool) -> io::Result<Vec<ChatBody>> {
        let Some(pending) = self.approvals.front().filter(|approval| approval.view_id == request_id) else {
            return Ok(Vec::new());
        };
        let decision = if allow { "accept" } else { "decline" };
        let id = pending.id.clone();
        self.write(json!({ "id": id, "result": { "decision": decision } })).await?;
        Ok(self.advance_approval(allow))
    }

    fn resolve_approval(&mut self, id: &RequestId) -> Vec<ChatBody> {
        if self.approvals.front().is_some_and(|approval| approval.id == *id) {
            return self.advance_approval(false);
        }
        self.approvals.retain(|approval| approval.id != *id);
        Vec::new()
    }

    fn advance_approval(&mut self, allowed: bool) -> Vec<ChatBody> {
        let Some(approval) = self.approvals.pop_front() else {
            return Vec::new();
        };
        let resolved = ChatBody::PermissionResolved { request_id: approval.view_id, allowed };
        [resolved].into_iter().chain(self.approvals.front().map(|next| next.event.clone())).collect()
    }

    async fn request(&mut self, method: &str, params: Value) -> io::Result<u32> {
        self.next_id += 1;
        let id = self.next_id;
        self.write(json!({ "id": id, "method": method, "params": params })).await?;
        Ok(id)
    }

    async fn write(&mut self, message: Value) -> io::Result<()> {
        self.input.write_all(format!("{message}\n").as_bytes()).await
    }
}

fn decode<T: DeserializeOwned>(params: Value) -> Option<T> {
    match serde_json::from_value(params) {
        Ok(value) => Some(value),
        Err(failure) => {
            debug!(%failure, "Ignoring malformed Codex Chat event");
            None
        }
    }
}

async fn emit(events: &mpsc::Sender<ChatBody>, body: ChatBody) -> io::Result<()> {
    events.send(body).await.map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "The Chat event receiver closed."))
}
