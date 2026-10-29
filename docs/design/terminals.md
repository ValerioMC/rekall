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
which the Steps pane's **Plan here** asks for, and `/rk … plan flow` for `TerminalMode.PLAN_FLOW`,
which its **Plan a flow** asks for; `/rk … generate` and `/rk … explain` for the Diagrams
screen's **Generate** and **Explain code**, the only modes a terminal on a project alone takes), and pumps its raw bytes to whoever is watching. The reason
it is a PTY and not `claude -p` is token cost: a hand-run `claude` keeps its own cache warm,
compacts its own context and shows its own `/context` and cost read-outs, and a PTY running the
same binary the same way inherits all of that, where a headless stream-json process reassembled
into a transcript could not. It is also the only place an interactive permission prompt actually
renders and can be answered.

The bytes do not go over HTTP. The terminal controller in `rekall-http/src/terminal/` only opens
(`POST /api/tasks/{id}/terminals`), lists and closes, through `TerminalService` in `rekall-console`;
once a terminal exists the pane connects to the socket controller beside it at
`/api/terminal/{id}/io`, Rekall's one WebSocket, which reaches the bytes through
`TerminalPipeService`. Both services hold a port of `rekall-core` (`TerminalHost`, `TerminalPipe`)
that `PtyTerminals` in `rekall-infra` implements over `PtyTerminalManager` and `rekall-app` injects
through `ApiPorts`, so `rekall-http` never names the PTY. The run queue is driven the same way:
`RunQueueControlService` edits the rows through `RunQueueService` and wakes the worker through the
`QueueRunner` port, which `RunQueueRunner` implements. Binary frames are
stdin and stdout, a `{"resize":[cols,rows]}` text frame sets the window size, and a
`{"type":"ended",...}` frame closes it out. Sends go through one bounded
queue per socket, so a pane that stops reading is dropped rather than left to back up memory. `useTerminalSocket` is the client half,
`TerminalPane.vue` wires it to an `xterm.js` instance.

A terminal that follows steps can be renewed (`PtyTerminalManager::renew_session`): it is closed and
another is opened on the same task with the same model, effort and permissions, which is how a
session boundary (4.3 in [workflow.md](workflow.md)) gives the next step a clean context. The pane switches to the new terminal
once the old one's end reaches it (`terminal.store`'s `followNextSession`).

One terminal per task. Opening again on a task that already has one is routed to it; opening on
a different step releases the step it was on and marks the new (see [workflow.md](workflow.md)). A plan open is always on
the task, never a step, and on a task that already has a terminal it types the plan line into it
rather than spawning a second. `PtyTerminalManager`
also carries the checklist marker: `TaskStepService.markRunning` / `releaseRunning` for a step,
`TaskReviewService.sessionRunning` for a stepless task, on open and on every way the terminal
ends.

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
| `rekall-git-credential` | git's credential helper: answers `get` with `username=oauth2` and the token |
| `rekall-gitlab` | `rekall-gitlab --connection <label> <METHOD> <path> [curl options]`: a GitLab API call with the `PRIVATE-TOKEN` header, `:project` standing for the binding's project path, URL-encoded |
| `rekall-jira` | `rekall-jira --connection <label> <METHOD> <path> [curl options]`: a Jira call under `/rest/api/2` with the credential's `Authorization` header (Basic on Cloud, Bearer on Data Center) and JSON both ways, `:project` standing for the binding's project key, through the connection's proxy (`curl --proxy`) when it names one |

None holds a credential. `SessionEnvironment` adds the folder first on `PATH`, `KUBECONFIG` (only
Rekall's clusters, so the session uses the credential you provided), a `GIT_CONFIG_KEY_n`/`VALUE_n`
pair per GitLab host (an empty `credential.<url>.helper` that drops helpers configured elsewhere,
then Rekall's, after any entries the shell already set; bindings of one instance share its token, so
the first answers for the host) and `REKALL_CREDENTIAL_TOKEN`, 244 random bits
for this terminal. The helpers send the token as a header read from stdin, so it never appears in a
process listing, to `GET /api/terminal/{id}/credentials/{label}?for=kubectl|git|gitlab|jira`. That route
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
then Jira, by label within a kind, the order of the project's Connections tab. The counts are `credentialAccess.store`'s, read from one subscription
and kept only for the console's lifetime; the access log is the record.
