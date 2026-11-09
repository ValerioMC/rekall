# Terminals and credential sessions

## Opening a session from a button

**Open in terminal**, on a task, hands the session to your own terminal app in that
project's folder with `/rk` already running. It answers the last thing the anchor chips could not: an anchor still has
to be pasted somewhere, and that somewhere has to be the right directory, because Claude Code
takes the folder it was launched from and keeps it for the session. So the folder is a column on
the project, `repo_folder`, and it travels down onto every task response beside the project label
those rows already carry.

It is a bridge in the desktop app (`rekall-app/desktop/src/bridges/claude_code_launcher.rs`), next to the
folder chooser, and not an endpoint. A `POST /api/launch` on 47355 would be reachable by any page
open in any browser on the machine, which makes a button that starts a terminal into a way to
start one without a click. Through the WebView, only this application can call it.

The page never names a command either. It sends a folder, an anchor and one flag; the folder has
to exist, the anchor has to be `entity:value` characters and nothing a shell reads as a command,
and the line is assembled on the native side. A note rendered in that window is markdown someone
else may have written, and it is one XSS away from being the caller.

The terminal is launched by opening a short script with it rather than by scripting the terminal
itself. AppleEvents would put a "Rekall wants to control Terminal" prompt in front of a button
whose whole point is that it is one click, and would fail silently for anyone who declines. The
script removes itself, `cd`s and `exec`s, so nothing is left behind and closing the window ends
the session and nothing else.

`--dangerously-skip-permissions` is a switch in Settings, off until it is turned on, and it is
kept in the browser's storage rather than in the database: "run without asking" is a property of
this terminal on this machine, and a database opened somewhere else has no business carrying that
answer along with it.

## The terminal pane

The in-app session is a real terminal, `C` in the console. `PtyTerminalManager` in
`rekall-infra` starts the interactive `claude` TUI in a pseudo-terminal (`portable-pty`) in the task's
folder, with `/rk` as the first line (`/rk … plan` for a terminal opened in `TerminalMode.PLAN`,
which **Plan here** in the arrow of **Run here** asks for, and `/rk … plan flow` for `TerminalMode.PLAN_FLOW`,
which its **Plan a flow** asks for, and `/rk … replan` for `TerminalMode.REPLAN`, which its **Replan**
asks for after a confirmation that can carry what the plan missed, quoted after `replan`, and `/rk … revise` for `TerminalMode.REVISE`, which the description's **Run revision** asks for to have the session work one pass over the waiting revisions of the task's thread, the ones aimed at the plan included (the button starts the session first and the pass after, so a terminal that cannot open leaves the revisions waiting; the plain `replan` line is typed from the steps pane); `/rk … generate` and `/rk … explain` for the Diagrams
screen's **Generate** and **Explain code**, the only modes a terminal on a project alone takes), and pumps its raw bytes to whoever is watching. The terminal is the
unmodified interactive `claude`: it shows its own `/context` and cost read-outs, renders its own
permission prompts and answers them, and is the same program a hand-run session is. The chat view
(below) runs the same binary headless over stream-json. Whether the two spend different amounts of
tokens has not been measured: `scripts/session_shape.py` groups transcripts by entrypoint (`cli` for a
terminal, `sdk-cli` for chat) so the two can be compared on the same days, and no difference is
claimed either way (see [claude-and-tokens.md](claude-and-tokens.md)).

The bytes do not go over HTTP. The terminal controller in `rekall-http/src/terminal/` only opens
(`POST /api/tasks/{id}/terminals`), lists and closes, through `TerminalService` in `rekall-console`;
once a terminal exists the pane connects to the socket controller beside it at
`/api/terminal/{id}/io`, Rekall's one WebSocket, which reaches the bytes through
`TerminalPipeService`. Both services hold a port of `rekall-core` (`TerminalHost`, `TerminalPipe`)
that `PtyTerminals` in `rekall-infra` implements over `PtyTerminalManager` and `rekall-app` injects
through `ApiPorts`, so `rekall-http` never names the PTY. `TerminalHost` also announces every end
(`subscribe_ended`, a `TerminalEnded`), answers whether a task has a live session (`has_live_for`) and which
one works its steps (`work_session_for`), and renews one (`renew`), so `RunQueueWorker` and
`SessionRelay` hold it and not the manager. A `TerminalOpening` and a `TerminalView` carry a
`SessionKind` (`TERMINAL` or `CHAT`, `TERMINAL` when a request names none); `PtyTerminals` always reports `TERMINAL`. A `TerminalView` also names the `TerminalMode` the session was opened in (`WORK`, `GENERATE`, `INVENTORY`, `AUTOCONFIGURE`, `MERGE`...), which is how the console titles a session opened on a project alone. The run queue is driven the same way:
With `parallel_sessions` above one, `RunQueueWorker` keeps a list of `Active` sessions, one `generation` each, and every command (`Review`, `SessionEnded`, `Settle`, `Renew`, `Pause`) names the session it is about; the worktree of each is made by `TaskWorktreeService` before its terminal opens, and `Stop` closes them all. `RunQueueControlService` edits the rows through `RunQueueService` and wakes the worker through the
`QueueRunner` port, which `RunQueueRunner` implements. `run_in_queue` adds a task and arms an idle
queue in one transaction (`RunQueueService::add_and_arm`), then wakes the worker; `finish` returns
the queue to `IDLE` only when no item is still waiting, so a task added meanwhile is not stranded. Binary frames are
stdin and stdout, a `{"resize":[cols,rows]}` text frame sets the window size, and a
`{"type":"ended",...}` frame closes it out. Sends go through one bounded
queue per socket, so a pane that stops reading is dropped rather than left to back up memory. `useTerminalSocket` is the client half,
`TerminalPane.vue` wires it to an `xterm.js` instance.

A chat session is opened, listed and closed through these same routes: the request's `kind` picks the
host. `SessionHosts` (`rekall-infra/src/session/`) is the one `TerminalHost` that `rekall-app` injects
for the queue, the relay and `TerminalService`: it sends an opening to the host of its kind, every other call to the
host that holds the session, merges the lists oldest first and the two end buses into one, and holds the cap
(`max_sessions`) across both kinds, except for an opening that reuses a session already on its target. Once a chat session
exists, `rekall-http/src/chat/` serves it through `ChatService` in `rekall-console`, which refuses the id of a terminal:
`GET /api/chat/{id}/events` streams its events as Server-Sent Events whose `id` is the event's `seq` (an
`EventSource` that reconnects sends `Last-Event-ID` and gets only what came after, `?after=` asks the same on the first
connection; a keep-alive comment every 15 s; a reader a megabyte behind is dropped; only a loopback origin or none is
accepted), and `POST /api/chat/{id}/messages` (`{text}`, trimmed, at most 20,000 characters), `/interrupt` and
`/permissions/{requestId}` (`{allow}`) write to it. The permission route is outside the layer that reads every path variable as a
UUID, because the id is the one `claude` gave the prompt. An unknown or closed session is a 404.

What a chat session is made of. `ChatProcess` (`rekall-infra/src/chat/`) runs `claude -p --verbose
--input-format stream-json --output-format stream-json --include-partial-messages` with only the
shell environment Rekall resolved, and `StreamParser` turns each stdout line into `ChatEvent`s of
`rekall-core`: a `seq` and a `ChatBody` (`UserMessage`, `AssistantText` and `AssistantThinking` that
grow until `complete`, `ToolUse` with a typed `ToolInput`, `ToolResult`, `PermissionRequested` and
`PermissionResolved`, `TurnFinished` with the turn's usage, `Notice` and `Ended`). `EventJournal`
numbers and keeps them, which is what lets a reader that reconnects resume from `Last-Event-ID`.
`ChatSessionManager` owns the processes, the journal and the idle sweep; it talks to the rest of
the application through two ports, `ChatPipe` (what a client does to a session: attach a listener, send a
message, interrupt, answer a permission) and `ChatListener` (what the session reports to it: an event, the end).

Two rules come from how the headless process behaves, found by running it
(`rekall-infra/tests/fixtures/chat/` holds the captures). A permission is asked over the same pipes
(`--permission-prompt-tool stdio`, used when skip-permissions is off): `claude` writes a
`control_request` and waits, and Rekall answers allow or deny with the matching `request_id`, with no
time limit. Stop is the control-protocol interrupt and never SIGINT, because SIGINT ends the
process while the interrupt ends only the turn and the same process answers the next message. A
message sent while a turn runs is queued by `claude`, not refused.

The run queue opens the kind of session its settings name (`run_queue.session_kind`, `TERMINAL` unless set; migration 058).
A renewal keeps the kind of the session it replaces, so a task carried across the sessions its steps ask for stays in chat
sessions. A chat session stopped on a permission prompt shows `awaiting_permission` on its `TerminalView`; the worker
then says so on the running item instead of failing it, and `ChatSessionManager::sweep_idle` does not close it.

A terminal that follows steps can be renewed (`TerminalHost::renew`, which `PtyTerminals` runs as `PtyTerminalManager::renew_session`): it is closed and
another is opened on the same task with the same model, effort and permissions, which is how a
session boundary (4.3 in [workflow.md](workflow.md)) gives the next step a clean context. The pane switches to the new terminal
once the old one's end reaches it (`terminal.store`'s `followNextSession`).

One terminal per task. Opening again on a task that already has one is routed to it; opening on
a different step releases the step it was on and marks the new (see [workflow.md](workflow.md)). A plan open is always on
the task, never a step, and on a task that already has a terminal it types the plan line into it
rather than spawning a second. The checklist marker is `WorkMarker` (`rekall-infra/src/session/`):
`TaskStepService.markRunning` / `releaseRunning` for a step, `TaskReviewService.sessionRunning` for a
stepless task, on open and on every way the terminal ends. `EndedBus` there carries the ends and
`spawn_idle_sweep` runs the idle sweep, both written for any kind of session manager.

Nothing is persisted. There is no row behind a terminal, so `TerminalLaunchService` (in
`rekall-core`, against the entities) only resolves a task id to its folder and anchors, and a
`PtyTerminalManager` restart starts empty and releases any step it left `RUNNING` on the way
down. A bounded in-memory scrollback ring is replayed to a pane that reopens, so it repaints;
that snapshot can begin mid escape-sequence and flicker once, which is the accepted cost of not
storing a terminal. A cap on how many run at once (`rekall.terminal.max-sessions`, eight), an
idle sweep, and a shutdown hook that kills the rest inside the 5s shutdown budget keep strays
from piling up.

The PTY's environment is the user's login shell's, not the app's: launched from Finder, the app
inherits launchd's bare `PATH`, which lacks everything a shell profile adds (`~/.cargo/bin` and the like). `LoginShellEnvironment`
runs `$SHELL -i -l -c` with stdin on `/dev/null`, has it print `env -0` between two random markers
(so a profile's greeting cannot corrupt it), and caches the result; each read starts a background
refresh. Failure, no markers or the timeout keep the last good copy, falling back to the process
environment. `ClaudeCli` adds `HOME` and the usual install directories to it, strips
`CLAUDECODE`/`CLAUDE_CODE_ENTRYPOINT` so a Rekall started from a Claude session can still open one,
and searches that `PATH` for `claude`. Windows skips the shell and uses the process environment.

**Credential sessions.** A terminal opened on a project that binds a connection with a stored
credential gets a credential session from `CredentialBroker` (`rekall-infra/src/connection/`), so
git and kubectl can use the credential without it entering the session. `TerminalLaunchService`
hands over the project's bindings (`TerminalConnection`: the binding's label, kind, target, instance URL,
keychain account; never the credential), so a connection bound twice is reached by either label. The broker writes a folder under `rekall.home/terminal-credentials/`
(0700, files 0600, scripts 0700; emptied at every start), holding:

| File | What it does |
|---|---|
| `kubeconfig` | One context per cluster, named by its label: the server, CA, TLS name and proxy read from the stored kubeconfig, the binding's namespace, and a user that is the exec plugin (`interactiveMode: Never`) |
| `rekall-kube-credential` | kubectl's exec plugin: prints the `ExecCredential` (a token, or a client certificate and key as PEM) |
| `rekall-git-credential` | git's credential helper for GitLab and GitHub hosts: answers `get` with `username=oauth2` and the token |
| `rekall-gitlab` | `rekall-gitlab --connection <label> <METHOD> <path> [curl options]`: a GitLab API call with the `PRIVATE-TOKEN` header, `:project` standing for the binding's project path, URL-encoded |
| `rekall-github` | `rekall-github --connection <label> <METHOD> <path> [curl options]`: a GitHub REST call with a Bearer `Authorization` header and `X-GitHub-Api-Version`, under `https://api.github.com` for github.com or `<server>/api/v3` for an Enterprise Server (`github_api_root`), `:project` standing for the binding's `owner/repo` as two path segments |
| `rekall-jira` | `rekall-jira --connection <label> <METHOD> <path> [curl options]`: a Jira call under `/rest/api/2` with the credential's `Authorization` header (Basic on Cloud, Bearer on Data Center) and JSON both ways, `:project` standing for the binding's project key, through the connection's proxy (`curl --proxy`) when it names one |
| `rekall-grafana` | `rekall-grafana --connection <label> <METHOD> <path> [curl options]`: a Grafana call under `/api` with the service account token as a Bearer `Authorization` header and JSON both ways, through the connection's proxy when it names one and with `--insecure` when it accepts a self-signed certificate, `:project` standing for the binding's folder UID |

None holds a credential. `SessionEnvironment` adds the folder first on `PATH`, `KUBECONFIG` (only
Rekall's clusters, so the session uses the credential you provided), a `GIT_CONFIG_KEY_n`/`VALUE_n`
pair per GitLab or GitHub host (an empty `credential.<url>.helper` that drops helpers configured elsewhere,
then Rekall's, after any entries the shell already set; bindings of one instance share its token, so
the first answers for the host) and `REKALL_CREDENTIAL_TOKEN`, 244 random bits
for this terminal. The helpers send the token as a header read from stdin, so it never appears in a
process listing, to `GET /api/terminal/{id}/credentials/{label}?for=kubectl|git|gitlab|github|jira|grafana`. That route
sits outside the router's UUID check, answers only a loopback peer even with
`rekall.security.remote-access` on, and answers in plain text the helpers print as they are: 403
for a token that is not the terminal's (compared in constant time) or a peer that is not this
machine, 404 for an ended terminal, an unbound label or a missing credential, 400 for a shape the
connection's kind does not have. Every request about a bound connection is a line in
`connection_access` through `ConnectionAccessService` (granted, or refused with why); a credential is
handed out only once its line is written. A request from an ended terminal or for an unbound label
names no connection to log against and is only logged by `tracing`. The session ends with the
terminal on every path (`close`, exit, the idle sweep, shutdown): the token stops working and the
folder goes. A cluster whose stored kubeconfig no longer reads is left out of the session and
logged. The helpers are shell scripts using `curl`, so a Windows terminal gets no session, and the
external **Open in terminal** gets none either.

`TerminalView` carries the project's connections (`TerminalConnectionView`: id, label, kind, whether
a credential is stored), and every access line is published as a `credential-access` frame (terminal,
connection, granted) in the same transaction as the line, so the pane never shows an access the log
lacks. `TerminalConnectionChips` draws, on the terminal pane's header and on each row of the sessions
panel, a chip per connection the session's helpers asked for: green with a count once they were handed
the credential (pulsing once per hand-out), amber when a request was refused, each linking to the
company's Connections screen. The connections never asked for fold into one `N unused` chip that opens
to list them (idle, or amber when no credential is stored); a connection leaves it at its first request.
Both lists follow `sortedByKind` (`features/connection/lib/connection-order.ts`): clusters, then GitLab,
GitHub, Jira, then Grafana, by label within a kind, the order of the project's Connections tab. The counts are `credentialAccess.store`'s, read from one subscription
and kept only for the console's lifetime; the access log is the record.
