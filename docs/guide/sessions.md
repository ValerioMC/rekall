# Terminals, the run queue and token usage

## Provider settings

**Settings > Sessions and run queue > Provider** selects **Claude Code** or **Codex** for new
interactive sessions. Rekall remembers each provider’s model, reasoning effort and permission
settings separately. Existing sessions and automatic renewals keep the provider and options they
started with.

When both CLIs are installed, open the arrow beside **Run here** on a task or **Start quick task**
after entering a description. Choose **Run with Claude Code** or **Run with Codex**. The choice
applies to that launch and uses the provider's saved model, reasoning effort and permissions.
Close any open session on the task before switching its provider.

For Codex, install the CLI, sign in, and choose **Connect Codex** to register Rekall’s MCP endpoint
and install the `$rekall` skill. **Model** and **Reasoning effort** use the CLI’s model catalog.
**Bypass approvals and sandbox** starts off and adds
`--dangerously-bypass-approvals-and-sandbox` when enabled.

The console shows separate **Claude** and **Codex** account usage rings when the Codex CLI is detected.
Hover or focus either meter to see its available usage windows and reset times. The warning
percentage in **Settings > Time and usage** applies to both rings. Codex limits require a ChatGPT
login that reports account limits.

Unattended queue runs, Rekall’s installed guard hooks and per-task token attribution use
Claude Code.

## Terminal in Rekall

On macOS and Linux, both providers can use the project’s saved connections through the
[connection helpers](projects.md#project-page). Rekall starts Codex with its own app server
(`--no-daemon`) so the helpers use that terminal’s environment. Codex’s sandbox and approval
settings govern the network calls.

**Run here**, next to **Open in terminal** on the description and steps panes, or `c` from any task, runs a real terminal inside the app. The backend starts the selected CLI in a pseudo-terminal in the project’s folder and opens the task with `/rk` for Claude Code or the `$rekall` skill for Codex; the pane renders it with `xterm.js` and the bytes travel both ways over one WebSocket at `/api/terminal/{id}/io`. It is the same binary run the same way as in your own terminal, so its own screens, `/context` read-outs and permission prompts are the ones you know. The [Chat view](#chat-view) is the alternative. Works in a plain browser, not only the desktop app.

The terminal gets the environment a new window of your own terminal would: Rekall asks your login shell (`$SHELL -i -l -c`) for its variables once it starts, so whatever your profile adds to `PATH` (`~/.cargo/bin`, nvm, pyenv, sdkman) and exports is there, along with `SSH_AUTH_SOCK`. The answer is cached and refreshed in the background every time a terminal opens, so a tool installed while Rekall runs reaches the next terminal but one. A shell that prints nothing usable within the timeout leaves the last good answer in place, or Rekall's own environment when there is none. Only a terminal opened before the first attempt has finished waits for it; after that a terminal never waits, so a profile that takes longer than the timeout costs the `PATH` it adds, not a delay on Run here. The shell runs with `REKALL_RESOLVING_ENVIRONMENT=1`, so a slow profile can skip work with `[[ -n $REKALL_RESOLVING_ENVIRONMENT ]] && return`.

One terminal per task, the way one terminal window is. A second **Run here** on a task that already has one refocuses it and types `/rk <anchors>` into the live session, so it reloads the task with any steps written since; opening on a different step also moves the checklist marker (the old step back to open, the new one to running) without restarting the process. **Restart** kills the `claude` process and starts a fresh one on the same task; **Close** ends it. Opening on a step marks that step `RUNNING` while the terminal is on it; on a task with no checklist the review line goes `RUNNING` instead, and both are released when the terminal closes. A **Discuss** session marks nothing running and releases nothing, unless a later **Run here** puts it on a step. Nothing is persisted: a restart of Rekall clears every terminal and releases any step it left running, and reopening the pane replays a bounded scrollback so it repaints.

**Run here** starts the session without leaving the pane. Pressed on a whole task with nothing left to execute (every checklist step claimed or done with only drafts remaining, or a task with no checklist whose review is claimed and only drafts remain), it starts nothing and shows a "Nothing to run" message; a press on one chosen step always launches. The play mark runs out of the button, the amber step seal stamps in with its check drawn, as a step's does when it is completed, and the button then reads **Working** beside a running step's own seal (the comet orbiting its bezel, the ticks lighting as it passes, the core breathing) for as long as Claude is on that work. On a step, that lasts while the step is `RUNNING`. On a task with no checklist, it lasts while the task's review line is `RUNNING`. On a checklist task launched from its description, it lasts while a step runs, or while the task's terminal is live and a step is still open. A **Discuss** session on no step is not work, so the button stays **Run here**. **Working** takes no press. The button returns to **Run here** once the work is claimed or the session ends.

In the navigator, a task a session was opened on carries a small mark under its status pip: bars that play while the session is open, an amber pulse while it waits for your approval, and the bars at rest once it has ended.

Live work sits in one bar: the foot of the navigator on the console, a capsule in the bottom left corner on every other screen, so the right edge of each pane stays free for its own buttons. It has a segment for running timers (with the newest timer's clock) and a segment for live terminals, each shown only while it has something to count. A segment opens its sheet above the bar (over the navigator list on the console), one sheet at a time; pressing the other segment switches, and pressing the same one again, Escape, or a click anywhere else closes it. From the sheet a row jumps to its task (the terminal one also opens the terminal pane; a session generating a diagram for a project alone opens on the Diagrams screen instead), stops the timer or terminal, and, for a terminal on a task, logs the latest commit against the task and step it was opened on. The terminal segment steps aside while the terminal pane is on screen, and the bar leaves entirely when nothing is live.

**Settings > Sessions and run queue** picks the model and effort a new terminal starts with, and the **skip permissions** switch:

- **Model** (`Account default`, `Sonnet`, `Fable`, `Opus`, `Haiku`): anything but the default adds `--model <alias>`. Each alias is Claude Code's own name for the latest model of that family, so no version is pinned.
- **Reasoning effort** (`Account default`, `Low`, `Medium`, `High`, `Extra-high`, `Max`): anything but the default adds `--effort <level>`.
- **Skip permissions** adds `--dangerously-skip-permissions`; with it off, an interactive permission prompt in the terminal is yours to answer.

All three are stored on the machine, not in the database.

The top bar is a four-cell grid, so the screen tabs sit centred between the history arrows and the centre cell at the same place on every screen (Console, Catalog, Diagrams, Security, Claude, Time). Under a 1280px window they fold into one key showing the screen you are on, which opens every screen as a vertical menu, a group's screens under its name; a tab's alert (Security's count of open critical advisories) rides on that key too, so folding never hides a warning. In the console the centre cell holds the search bar and the live readings (usage, queue, review) sit centred between it and the tools; on the other screens it holds the title and the actions sit at the right. Both keep their centre as the window resizes. The top bar carries a usage meter: the current 5-hour session as a ring with its percentage and time to reset, and, on hover, a bar per window including the weekly per-model limits. The figures are the ones Claude Code's own `/usage` shows, read with the OAuth token Claude Code stores (every `Claude Code-credentials` item in the macOS keychain, else `~/.claude/.credentials.json`; the token expiring last wins, and an expired one counts as no token). With no valid token the meter asks you to open a Claude Code terminal, which signs in or refreshes the token; when Anthropic cannot be reached it holds the last figures. Readings are not real time: one when the console opens, one every five minutes while it is on screen, one when the window comes back to the front after a minute away, and one whenever you click the blank meter or **Check again** in its popover (`GET /api/claude/usage?refresh=true`, past the server's cache). If Anthropic answers 429 the meter shows **Wait** with the time left and takes nothing, not even a manual check, until that `Retry-After` has passed: asking again is what extends the block.

| Property | Default | Meaning |
|---|---|---|
| `rekall.claude.cli-path` | search the login shell's `PATH` and the usual install dirs | Absolute path to `claude`, overriding discovery |
| `rekall.terminal.shell` | `$SHELL`, else `/bin/zsh` | Login shell asked for the terminal's environment |
| `rekall.terminal.shell-environment-timeout-seconds` | `10` | How long that shell gets to print its environment |
| `rekall.claude.usage-url` | `https://api.anthropic.com/api/oauth/usage` | Where the usage meter reads session and weekly limits |
| `rekall.terminal.max-sessions` | `8` | Terminals allowed at once |
| `rekall.terminal.idle-minutes` | `120` | A terminal untouched this long is closed by the sweep |
| `rekall.terminal.sweep-minutes` | `5` | How often the idle sweep runs |
| `rekall.terminal.scrollback-bytes` | `131072` | Bytes of output replayed to a pane that reopens |
| `rekall.run-queue.tick-seconds` | `20` | How often the run queue looks at its clock, schedule and hold |
| `rekall.run-queue.settle-grace-seconds` | `8` | How long a finished session keeps its terminal before Rekall closes it, or hands the task on to a fresh session |

## Chat view

Choose **Chat** under **Settings > Sessions and run queue > Session view** to see the selected provider’s conversation, reasoning and tool calls beside the task. Chat uses Claude Code’s stream JSON or Codex’s app server. The preference applies to agent actions including **Run here**, **Discuss**, **Plan here**, **Plan a flow** and **Refine**. The run queue has its own **Session view**. If the installed Codex CLI cannot open Chat, the launch dialog gives the reason and offers to retry the same action in Terminal.

- **Window.** It docks beside the note pane, over the page when there is not room, or full width when expanded; drag its edge or use the arrow keys (Shift for bigger steps) to resize between 360 and 720 px. The window belongs to the task its session runs on: it disappears when you select another task and returns with the task. Its width and whether it is hidden are remembered on this machine. `C` hides or shows it and `Shift+C` expands it; while it is hidden and the task has a session, a **Chat** tag on the right edge of the console brings it back; `Esc` gives the keyboard back to the console.
- **What the history shows.** The same as the terminal does. A quiet row opens the session with its model, working directory and permission mode. The output of a local command such as `/context` is drawn on its own, labelled with the command. A tool call is one row with what it did after what it was pointed at ("Read 120 lines", "exit code 1"), and opens to its input: a todo list as a checklist, already open, a web call as its address, an MCP call as its server and tool with its arguments as lines. An Agent call stays open while it runs, showing the calls its subagent makes, and folds to a count of them when its result arrives.
- **Approvals.** Claude Code tool approvals and Codex command and file-change approvals appear with **Allow** and **Deny** (`Y` and `N`). The provider’s permission settings determine when it asks.
- **Messages and Stop.** Enter sends and Shift+Enter inserts a line break. Claude Code accepts messages during a turn and offers supported slash commands. Codex processes messages in order, one turn at a time. **Stop** interrupts the current turn and leaves the conversation open for the next message.
- **Steps and sessions.** Opening on a step marks it running, and claiming steps, `new_session` groups and the usage ceiling work as for a terminal. A chat session is replaced by a fresh one when the next step asks for a new session, and the window follows it. Plan and wrapup lines on a live chat go in as a message.
- **When it ends.** If the provider process exits, the window shows its exit code and Rekall releases the step it held. Closing the app ends the live sessions and releases their running markers.
- **What the terminal still does that chat does not.** `/clear` is not offered in the window, and the screens Claude Code draws in its own interface exist only in the terminal. A chat session counts toward the same session cap as terminals.

Each finished turn leaves the share of the context window its last call held, for the task and for the step it ran on. `GET /api/context-fill?taskId=` returns them, and the console's event stream carries a task's fills as a `context-fill` frame after every turn. The console shows the last one as a badge with the percent, on the task's properties bar and on the step the turn ran on: grey below 60 %, yellow from 60 and red from 85, with the tokens over the window in its tooltip. The chat header also shows the live session's fill and its total cost, and each turn's footer its cost.

Tokens are counted from the transcripts either way, and the token chip counts a chat session's work. Whether chat spends more or fewer tokens than a terminal has not been measured; `scripts/session_shape.py` compares them by session type, and a first run on a very small sample, with no cold turn on either side, says nothing about cache expiry (see [claude-and-tokens](../design/claude-and-tokens.md)).

## Guard command

`rekall-server guard before-tool` and `rekall-server guard before-stop` are the programs Claude Code
runs as a `PreToolUse` and a `Stop` hook. Each reads the hook's JSON on standard input, asks the
running Rekall for the context of the session's folder, and exits 0 to let the action go ahead or 2 to
block it with the reason on standard error.

- On an `auto-commit` project it blocks `git commit`, `git push` and `git add` (also after `&&`, in a
  subshell or in `sh -c`), because Rekall commits when a step is claimed.
- On every project it blocks a tool call that names Rekall's credential file, Claude Code's credential
  file or the credential helpers' endpoint.
- On `Stop` it blocks while a step of the folder's project is running and unclaimed, once: the next `Stop`
  goes through so the session can explain why it could not claim.

It lets the action through, and writes a line to `<rekall.home>/guard.log`, when Rekall does not answer
within a second, when the input cannot be read or when a command has an unclosed quote. The port is
`server.port` (`SERVER_PORT`), as for the server. Try it by hand:

```
echo '{"cwd":"/path/to/project","tool_name":"Bash","tool_input":{"command":"git commit -m x"}}' | rekall-server guard before-tool; echo $?
```

**Settings, Protections** installs the two hooks into `~/.claude/settings.json` and removes them
again, keeping every other setting and hook of yours; a settings file that is not valid JSON is shown as
such and never edited. The design is in [write-safety](../design/write-safety.md).

## Run queue

The run queue works through a list of tasks without you: each one in its own terminal, one after another, until everything on it is claimed. It opens from the dial next to the usage meter in the top bar, or `q`. The dial is also its status: three bars when idle, a clock hand at the start time when scheduled, an orbiting comet with `2/5` while running, and a ring stopped at the ceiling with the time it resumes while holding. When the running task has a checklist the comet becomes a ring of one slice per step: claimed steps filled, the current one lighter and pulsing, the rest hairline (a flow's not-needed steps have no slice, and above 12 steps the ring is one arc filled in proportion). When the top bar has room for it (a wide window) the figure adds `step 3/6`, and the tooltip always reads in full, for example "Run queue: running Report, task 2 of 5, step 3 of 6". A strand of beads shows each task in the run.

- **Order.** Add tasks by typing a title, label or anchor. Waiting tasks move up and down or come off. The running one stays until you stop the queue, and finished ones stay as the record of the run until **Clear finished**. A task with a checklist shows `n of m steps` under its title, which opens a plain list of its steps with their state and the current one marked; the running task's list is open, the others start collapsed. A step a flow never needed reads "Not needed" and does not count in `m`. A finished task says why: "Claimed, waiting for your review", or "Flow finished, 3 steps not needed". The status line under the header says which step is being worked.
- **Parallel sessions.** One to four tasks at once; one (the default) works them in turn, in the project's folder, exactly as before. From two on, every task the queue starts first gets a git worktree of its own: a second working folder of the same repository on a `rekall/<task-label>` branch, under `<Rekall home>/worktrees/<project>/<task-label>`, so two sessions never overwrite each other's files. The session, its "Done when" checks, the snapshot at `running` and the commit on claim all work in that folder, and the commit lands on that branch. Nothing merges while the task is under review; when you accept the task its branch is merged into the branch checked out in the project folder (see the workflow guide). Two tasks that depend on each other never run together, and the usage ceiling is checked before each new task starts, against the account's usage as a whole. A worktree git refuses leaves its task waiting with the reason (`Could not make a worktree: …`) while the others go on; start the queue again to retry. **Stop** closes every open session and puts their items back at the head. After a restart, a task whose worktree folder is gone forgets it (its branch and commits stay), and a worktree no task names is removed when it has nothing unsaved and left alone when it has.
- **Dependencies.** A task that depends on another (see Dependencies in the workflow guide) is skipped while that one is not accepted: the queue starts the first waiting task whose dependencies are all accepted, whatever the order they were added in. An item that waits reads `Waiting for task:…`, and says when that task is only claimed and waits for your review. Adding a task without the one it depends on shows a warning on its row; the queue never adds it for you. When every waiting task depends on an unaccepted one, the queue stops (`IDLE`) and each waiting item carries what it waits for as its detail. Accept the task it waits for and start the queue again.
- **When.** **Start now**, or **At a time** for a schedule. Rekall has to be running then: the queue runs inside it.
- **Usage ceiling.** With it on, no new task or step starts once a usage window reaches the line: the 5-hour session and the weekly total always, plus the weekly Opus or Sonnet window when the queue runs that model. The queue then holds until the latest reset among the windows over the line, plus a minute, and carries on from there. It checks before every task and at every step claim, never mid-step, so a step that is under way finishes. Set the line below 100 to leave room for it. With a ceiling set and no usage reading available, the queue holds and looks again every five minutes.
- **Sessions.** Model, effort, **Skip permission prompts** and the kind of session (`TERMINAL`, the default, or `CHAT`) for the sessions the queue opens. They are stored with the queue, separately from **Settings > Sessions and run queue**. Without skip, an unattended terminal stops at its first permission prompt. An unattended chat session stops there too, but it is not failed: its item stays running with the detail "Waiting for you to answer a permission prompt in the chat session.", the idle sweep leaves the session open, and the detail goes once the prompt is answered.

How each task runs: the queue opens a terminal on it, and the session works the open steps in order, claiming each (or writes the wrapup on a task with no checklist). A claim is enough: the queue never waits for a step or a task to be accepted, so a finished task reads "All of it claimed and waiting for review" while the next one already runs. A flow is finished when it reaches its end, even if some of its steps were never needed and stay open: the task reads "Flow finished: the steps it did not need were left unrun". Sending a step back from review, or resetting the flow's moves, makes the task work again. When nothing is left open, the queue closes the terminal and starts the next task. If the ceiling is reached at a claim, it closes the terminal, puts the task back at the head of the queue, sends any step the session had started back to open, and holds; the next session picks up at the first open step. A task with nothing open is skipped. A task fails, and the queue moves on, when the session can't open (no folder, no CLI, a terminal already open on it) or ends before its work is claimed. **Stop the queue** closes the running session after a confirmation. The queue survives a restart: a task that was running goes back to waiting, and an armed queue picks it up again.

**Sessions in a multi-step task.** By default one session works the whole checklist. A step can instead open a fresh Claude session: set `new_session` when a session proposes it (`rekall_propose_step`, `rekall_propose_flow`, `rekall_replan`; `/rk … plan` and `replan` always cut the sessions, picking the grouping they judge cheapest, without being asked), or switch it by hand with **New session from here** on the seam between two steps in the checklist (**Join session above** on a session's header undoes it), with **New session** on a draft, or when adding a draft. The first step always opens the first session, and the steps up to the next one that opens a session share one. The checklist draws a header over each session when there is more than one. A session claims the last step of its group, writes its wrapup and stops; Rekall closes that terminal and opens another on the same task, with the same model, effort and permissions, and the pane switches to it. A queued task stays running across its sessions, and the usage ceiling is checked before the next one opens. The choice is stored per step (`PATCH /api/steps/{id}/session` with `startsSession`) and `rekall_context` shows it as `(opens a new session)` and `sessions="N"` on `<steps>`.

**Run in queue.** `POST /api/run-queue/runs` adds a task and starts the queue in the same step. An idle queue starts at once; a running, holding or scheduled one only gains the task, which runs when its turn comes. A task left waiting by an earlier **Stop the queue** runs ahead of it, because the queue is armed in order. A task already queued or running is refused with 409.

In the console it is the arrow on the right of **Run here**, on a task's description and on its checklist: the arrow opens a menu of names with an icon each, in two groups split by a line. The first runs the work: **Run with Claude Code** and **Run with Codex** when both CLIs are installed, and **Run in queue**, which asks the same questions as the main key (the project folder, something left to run) and then says either "Running now in the queue" or "Queued, N ahead". The second prepares it: [**Discuss**](workflow.md#planning), always there, **Plan here**, **Plan a flow** (only while the task has no steps) and **Replan** (only while it has a draft or an open step). Where there is no room under the arrow, as from a step's keys at the foot of the page, the menu opens above it. The arrow does nothing while the key is working.

`/api/run-queue` holds the queue: `GET`, `PUT /settings` (`ceilingPercent`, `skipPermissions`, `model`, `effort`, `sessionKind`), `POST /items` (adds without starting), `POST /runs` (`taskId`: adds the task and arms the queue), `DELETE /items/{id}`, `PUT /items/{id}/position`, `POST /clear`, `POST /start` (`startAt` null starts now), `POST /stop`. Each answers with the whole queue, and every change is pushed on the console's event stream as a `run-queue` frame.

Both add endpoints take `taskId` and an optional `provider` (`CLAUDE` or `CODEX`). Only Codex requests may include `codexOptions` with the model, effort and permission preference. Existing rows and requests with no provider use Claude Code and the queue's session settings. The queue returns each item's saved choice in responses and events, including after a restart. The queue worker currently launches Claude Code.

## Token usage

Every task and every step shows the tokens Claude Code generated while working on it: on the task's row in the navigator and on each step of its checklist. A project shows the sum of its tasks wherever it is drawn: its heading in the navigator, its card and page in the catalog, and its group in a calendar day. The figure is output tokens; the tooltip adds the input, cache-write and cache-read tokens read to produce them. A task's figure includes its steps'.

Beside the name of a task in its steps, wrapup and terminal panes, and at the right end of the tag and skills line in its description pane, the figure sits with a small sparkline of the last month. A project's heading in the navigator carries the same chip for the sum of its tasks. Click it for the daily trend: the running total of generated tokens, one point per day, over **1 month**, **3 months** or **All**. Move over the chart, or use the arrow keys, to read a day's total and what that day added. The task's title line holds a copy button that puts the task's `/rk project:… task:…` command on the clipboard (the command is in its tooltip). Days are UTC days, as the server cuts them. The tag on a task's description is a badge that opens the list of tags to change it; a task with none shows a dashed **New tag** button, the way skills show **Add a skill**.

The counts come from Claude Code's own session transcripts (`~/.claude/projects/<folder>/<session>.jsonl`, and `<session>/subagents/` for its subagents), read in the background every minute. Only sessions started in a project's folder, or a folder under it, are read, and a transcript is read again only once it has grown. Each model call is charged to what the session was working on when it made it:

- an `/rk` line, or a `rekall_context` call naming one task, puts the session on that task;
- `rekall_step` with `running` puts it on that step, and with `claimed` or `open` back on the task;
- a subagent's calls go to what the session was on at the moment they were made;
- calls before any of these are not Rekall work and are not counted.

The tokens follow the anchors, not the folder: a session in one project's folder working another project's task counts for that task. A task that no longer resolves is not counted; a step that no longer resolves, or is deleted, leaves its tokens on the task. Sessions run in any terminal count, not only those opened from Rekall.

`GET /api/token-usage` returns one tally per task and step, `GET /api/token-usage/daily` one entry per task and day, and the console's event stream carries a task's tallies and days as a `token-usage` frame whenever a read changes them.

| Property | Default | Meaning |
|---|---|---|
| `rekall.claude.transcripts-dir` | `~/.claude/projects` | Where Claude Code keeps its session transcripts |
| `rekall.claude.transcript-scan-seconds` | `60` | How often they are read |
| `rekall.claude.home` | `~/.claude` | Claude Code's own folder, which the Claude screen reads and edits |

