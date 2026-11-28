# Terminals, the run queue and token usage

## Provider settings

**Settings > Sessions and run queue** keeps what both providers share apart from what belongs to
one. **Both providers** holds the **Session provider** (**Claude Code** or **Codex**) for new
interactive sessions, the **Session view** and the MCP server address. Below it, a column per
provider holds its connection, model, reasoning effort and permission setting. Existing sessions
and automatic renewals keep the provider and options they started with.

Choose Claude Code or Codex in the top bar. This is the same choice shown in Settings, and the
browser remembers it across screens and reloads. **Run here**, **Start quick task** and the other
interactive launch actions use that provider for new sessions. If a task has a live session,
its **Run here** and planning actions use that session’s provider. The top bar menu marks a
provider whose CLI is missing and offers a link to agent setup.

For Codex, install the CLI, sign in, and choose **Connect Codex** to register Rekall’s MCP endpoint
and install the `$rekall` skill. **Model** and **Reasoning effort** use the CLI’s model catalog, read each time the section opens.
**Bypass approvals and sandbox** starts off and adds
`--dangerously-bypass-approvals-and-sandbox` when enabled.

The console shows separate **Claude** and **Codex** account usage rings when the Codex CLI is detected.
Hover or focus either meter to see its available usage windows and reset times. The warning
percentage in **Settings > Time and usage** applies to both rings. Codex limits require a ChatGPT
login that reports account limits.

Rekall offers guard hooks and task usage attribution for both providers.

## Terminal in Rekall

On macOS and Linux, both providers can use the project’s saved connections through the
[connection helpers](projects.md#project-page). Rekall starts Codex with its own app server
(`--no-daemon`) so the helpers use that terminal’s environment. Codex’s sandbox and approval
settings govern the network calls.

**Run here**, next to **Open in terminal** on the description and steps panes, runs a real terminal inside the app. The backend starts the selected CLI in a pseudo-terminal in the project’s folder and opens the task with `/rk` for Claude Code or the `$rekall` skill for Codex; the pane renders it with `xterm.js` and the bytes travel both ways over one WebSocket at `/api/terminal/{id}/io`. It is the same binary run the same way as in your own terminal, so its own screens, `/context` read-outs and permission prompts are the ones you know. The [Chat view](#chat-view) is the alternative. Works in a plain browser, not only the desktop app.

The terminal gets the environment a new window of your own terminal would: Rekall asks your login shell (`$SHELL -i -l -c`) for its variables once it starts, so whatever your profile adds to `PATH` (`~/.cargo/bin`, nvm, pyenv, sdkman) and exports is there, along with `SSH_AUTH_SOCK`. The answer is cached and refreshed in the background every time a terminal opens, so a tool installed while Rekall runs reaches the next terminal but one. A shell that prints nothing usable within the timeout leaves the last good answer in place, or Rekall's own environment when there is none. Only a terminal opened before the first attempt has finished waits for it; after that a terminal never waits, so a profile that takes longer than the timeout costs the `PATH` it adds, not a delay on Run here. The shell runs with `REKALL_RESOLVING_ENVIRONMENT=1`, so a slow profile can skip work with `[[ -n $REKALL_RESOLVING_ENVIRONMENT ]] && return`.

One terminal per task, the way one terminal window is. A second **Run here** on a task that already has one refocuses it and types `/rk <anchors>` into the live session, so it reloads the task with any steps written since. **Restart** kills the `claude` process and starts a fresh one on the same task; **Close** ends it. A session is opened on the task, never on a step: on a checklist it marks each step `RUNNING` as it starts it, and on a task with no checklist the review line goes `RUNNING` while the terminal is on it. When a session that was given work ends (**Close**, the process exiting, the idle sweep or closing the app), every step of the task still `RUNNING` goes back to `OPEN`, so **Run here** is ready again. A session is given work when it opens to work, or when **Run here** types the work line into it later. The review line and any revision the session took and never answered are released too. A **Discuss** or **Refine** session marks nothing running and releases nothing until it is given work, and then only the running steps. Rekall does not see a session in your own terminal app end, so that one releases nothing; ending a Rekall session of the same task releases the step it was working too. Nothing is persisted: a restart of Rekall clears every terminal and releases any step it left running, and reopening the pane replays a bounded scrollback so it repaints.

**Run here** starts a terminal session without leaving the pane; in [Chat view](#chat-view) it opens the chat full-size. It is on the task page's header and in a project's task preview, never on a single step. Pressed on a task with nothing left to execute (every checklist step claimed or done with only drafts remaining, or a task with no checklist whose review is claimed and only drafts remain), it starts nothing and shows a "Nothing to run" message. The play mark runs out of the button, the amber step seal stamps in with its check drawn, as a step's does when it is completed, and the button then reads **Working** beside a running step's own seal (the comet orbiting its bezel, the ticks lighting as it passes, the core breathing) for as long as Claude is on that work. On a task with no checklist, that lasts while the task's review line is `RUNNING`. On a checklist task, it lasts while a step runs, or while the task's terminal is live and a step is still open. A live **Discuss** session alone is not work, so the button stays **Run here**. **Working** takes no press. The button returns to **Run here** once the work is claimed or the session ends.

In the navigator, a task a session was opened on carries a small mark under its status pip: bars that play while the session is open, an amber pulse while it waits for your approval, and the bars at rest once it has ended.

Live work sits in one bar: the foot of the navigator on the console, a capsule in the bottom left corner on every other screen, so the right edge of each pane stays free for its own buttons. It has a segment for running timers (with the newest timer's clock) and a segment for live terminals, each shown only while it has something to count. A segment opens its sheet above the bar (over the navigator list on the console), one sheet at a time; pressing the other segment switches, and pressing the same one again, Escape, or a click anywhere else closes it. From the sheet a timer row jumps to its task, and a terminal row floats its session above the bar (a session generating a diagram for a project alone opens on the Diagrams screen instead); the full-size terminal is reached from the terminal key on the task’s tab. A row also stops the timer or terminal, and, for a terminal on a task, logs the latest commit against the task. The terminal segment steps aside while the terminal pane is on screen, and the bar leaves entirely when nothing is live.

The **Claude Code** column of **Settings > Sessions and run queue** picks the model and effort a new session opened from the console starts with, and the **skip permissions** switch:

- **Model**: `Account default` and the models the installed Claude Code reports, under its own names. Anything but the default adds `--model <value>` with the value the CLI gave: an alias such as `sonnet` follows Claude Code's releases, a versioned value stays on that version. The line under the field shows the value and, for an alias, the version it starts today.
- **Reasoning effort**: `Account default` and the levels the chosen model declares, out of `Low`, `Medium`, `High`, `Extra-high` and `Max`; anything but the default adds `--effort <level>`. A model the CLI says takes none offers only the default, and one it says nothing about offers all five. Picking a model that does not offer the current level resets it to `Account default`.
- **Skip permissions** adds `--dangerously-skip-permissions`; with it off, an interactive permission prompt in the terminal is yours to answer.

All three are stored on the machine, not in the database.

The models are read from the CLI each time the section opens, apart from the connection status. While no read has succeeded the field offers the family names (`Sonnet`, `Fable`, `Opus`, `Haiku`); after a failed read it keeps the last list and shows **Retry**. A read never changes a saved choice: a model or level the CLI no longer reports stays selected, with a line saying so, and new sessions still receive it.

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

Choose **Chat** under **Settings > Sessions and run queue > Session view** to see the selected provider’s conversation, reasoning and file changes full-size, under the bar of open tasks. Chat uses Claude Code’s stream JSON or Codex’s app server. The preference applies to agent actions including **Run here**, **Discuss**, **Plan here**, **Plan a flow** and **Refine**. The run queue has its own **Session view**. If the installed Codex CLI cannot open Chat, the launch dialog gives the reason and offers to retry the same action in Terminal.

- **Where it shows.** A chat fills the screen under the bar of open tasks. The actions above open it there, as do the chat key on a task's tab, the sessions list, **Watch session**, and Security's session buttons. The rest of the task's tab goes back to its page, with nothing over it. Generating or explaining a diagram, rewriting a description, applying feedback, autoconfiguring, reorganizing and scanning components leave you on the page. The selected task's chat stays followed from any page, so the sessions capsule reads **approve** and the browser tab is marked while it waits for you.
- **What the history shows.** The same as the terminal does. A quiet row opens the session with its model, working directory and permission mode. The output of a local command such as `/context` is drawn on its own, labelled with the command. A change to files is open on its diff, each file of a Codex change set under its path. A command is one line that says what it is for, with its state: its text and output are not shown, and a command with no description, as Codex runs them, is not listed. Any other tool call is one row with what it did after what it was pointed at ("Read 120 lines", "3 files"), and opens to its input: a todo list as a checklist, already open, a web call as its address, an MCP call as its server and tool with its arguments as lines. An Agent call stays open while it runs, showing the calls its subagent makes, and folds to a count of them when its result arrives.
- **Approvals.** Claude Code tool approvals and Codex command, file-change and MCP tool approvals appear with **Allow** and **Deny** (`Y` and `N`). MCP tool approvals show the server, tool and arguments, and a command approval shows the command you are allowing. The provider’s permission settings determine when it asks.
- **Messages and Stop.** Enter sends and Shift+Enter inserts a line break. Claude Code accepts messages during a turn and offers supported slash commands. Codex processes messages in order, one turn at a time. **Stop** interrupts the current turn and leaves the conversation open for the next message.
- **Steps and sessions.** Marking steps running, claiming them, `new_session` groups and the usage ceiling work as for a terminal. A chat session is replaced by a fresh one when the next step asks for a new session, and the console follows it. Plan and wrapup lines on a live chat go in as a message.
- **When it ends.** Running steps are released as for a terminal: on a task the chat was given work on, every step still `RUNNING` goes back to `OPEN`. Only a chat opened to work releases the review line and unanswered revisions. If the provider process exits, the chat shows its exit code. Closing the app ends the live sessions.
- **What the terminal still does that chat does not.** `/clear` is not offered in the chat, and the screens Claude Code draws in its own interface exist only in the terminal. A chat session counts toward the same session cap as terminals.

Each finished turn leaves the share of the context window its last call held, for the task and for the step it ran on. `GET /api/context-fill?taskId=` returns them, and the console's event stream carries a task's fills as a `context-fill` frame after every turn. The console shows the last one as a badge with the percent, on the task's properties bar and on the step the turn ran on: grey below 60 %, yellow from 60 and red from 85, with the tokens over the window in its tooltip. The chat view also shows the live session's fill and its total cost, and each turn's footer its cost.

Tokens are counted from the transcripts either way, and the token chip counts a chat session's work. Whether chat spends more or fewer tokens than a terminal has not been measured; `scripts/session_shape.py` compares them by session type, and a first run on a very small sample, with no cold turn on either side, says nothing about cache expiry (see [claude-and-tokens](../design/claude-and-tokens.md)).

## Worktree protections

Open **Settings → Sessions → Protections** to install, repair, or remove the hooks for each
provider. For Codex, open `/hooks` in the Codex terminal, review and trust both Rekall entries,
then refresh their status in Settings. Start a fresh session after setup.

- On a project where Rekall commits for you, the hooks block direct `git commit`, `git push`, and
  `git add` commands. Claiming a step lets Rekall run checks and commit the work.
- On any project, the hooks block supported tool calls that name protected credential locations.
- When a session stops with its task’s step still running, the hook gives the exact claim command,
  such as `/rk task:VE-12 step:2 done`. The provider can continue once. Preparation launches and
  sessions opened on a project folder without a task receive no step reminder.
- When all the work of a task is claimed and not yet accepted, a stop in its worktree with
  uncommitted files is refused once on a project where Rekall commits for you. The hook names the
  files and the claim, or wrapup, that commits them.

Rekall launches the provider with its existing permissions. The hooks let the action through if
Rekall is unreachable or cannot read the event and log the failure to `<rekall.home>/guard.log`.
See [write safety](../design/write-safety.md) for hook inputs and configuration paths.

## Task pull requests

For a project with pull request flow enabled, Run here and Run in queue work in the task’s
worktree. Each successful claim commits and pushes to the same branch. The first push opens a
pull request. Later pushes update the commits on that request. The task’s Git and review
surfaces show push and merge failures while Rekall retries.

After you accept the task, Rekall merges through the host and closes the task on success.
Conflicting work opens a merge session with the provider recorded for the task. Rekall
keeps a dirty worktree and releases the clean checkout after a successful merge.

## Run queue

The run queue works through a list of tasks until everything on it is claimed. Each task opens in the queue's configured view, Chat or Terminal, using its saved provider, model and effort. It opens from the dial next to the usage meter in the top bar, or `q`. The dial is also its status: three bars when idle, a clock hand at the start time when scheduled, an orbiting comet with `2/5` while running, and a ring stopped at the ceiling with the time it resumes while holding. When the running task has a checklist the comet becomes a ring of one slice per step: claimed steps filled, the current one lighter and pulsing, the rest hairline (a flow's not-needed steps have no slice, and above 12 steps the ring is one arc filled in proportion). When the top bar has room for it (a wide window) the figure adds `step 3/6`, and the tooltip always reads in full, for example "Run queue: running Report, task 2 of 5, step 3 of 6". A strand of beads shows each task in the run.

- **Order.** Find a task by title, label or anchor, then confirm the provider, model and effort. **Add to queue** saves the task and leaves an idle queue available to start separately. Each row shows its saved launch settings as tags. Waiting tasks move up and down or come off. The running one stays until you stop the queue, and finished ones stay as the record of the run until **Clear finished**. A task with a checklist shows `n of m steps` under its title, which opens a plain list of its steps with their state and the current one marked; the running task's list is open, the others start collapsed. A step a flow never needed reads "Not needed" and does not count in `m`. A finished task says why: "Claimed, waiting for your review", or "Flow finished, 3 steps not needed". The status line under the header says which step is being worked.
- **Parallel sessions.** One to four tasks at once; one (the default) works them in turn, in the project's folder, exactly as before. From two on, every task the queue starts first gets a git worktree of its own: a second working folder of the same repository on a `rekall/<task-label>` branch, under `<Rekall home>/worktrees/<project>/<task-label>`, so two sessions never overwrite each other's files. The session, its "Done when" checks, the snapshot at `running` and the commit on claim all work in that folder, and the commit lands on that branch. Nothing merges while the task is under review; when you accept the task its branch is merged into the branch checked out in the project folder (see the workflow guide). Two tasks that depend on each other never run together, and the usage ceiling is checked before each new task starts, against the saved provider's account usage. A worktree git refuses leaves its task waiting with the reason (`Could not make a worktree: …`) while the others go on; start the queue again to retry. **Stop** closes every open session and puts their items back at the head. After a restart, a task whose worktree folder is gone forgets it (its branch and commits stay), and a worktree no task names is removed when it has nothing unsaved and left alone when it has.
- **Dependencies.** A task that depends on another (see Dependencies in the workflow guide) is skipped while that one is not accepted: the queue starts the first waiting task whose dependencies are all accepted, whatever the order they were added in. An item that waits reads `Waiting for task:…`, and says when that task is only claimed and waits for your review. Adding a task without the one it depends on shows a warning on its row; the queue never adds it for you. When every waiting task depends on an unaccepted one, the queue stops (`IDLE`) and each waiting item carries what it waits for as its detail. Accept the task it waits for and start the queue again.
- **When.** **Start now**, or **At a time** for a schedule. Rekall has to be running then: the queue runs inside it.
- **Usage ceiling.** The queue checks the task's provider before launch and after a step is claimed when work remains. For Claude Code it checks the 5-hour and weekly total windows, plus the weekly Opus or Sonnet window when the session uses that model. For Codex it checks the primary and secondary windows the account reports. At the ceiling, the queue waits until one minute after the latest applicable reset. If a window at the ceiling has no reset time, it checks again after fifteen minutes. If account usage is unavailable, it checks again after five minutes.
- **Sessions.** The queue panel groups these by who uses them. Under **Both providers**, the queue's own session view (`TERMINAL` by default, or `CHAT`) and parallel sessions. Under **Claude Code**, permission settings and the model and effort used by older entries without saved choices. New entries use the model and effort you confirm. **Account default** delegates that field to the provider. The **Queue model** and **Queue effort** tags identify older Claude Code entries that inherit the queue settings. Each Codex entry keeps the bypass setting in effect when you confirm. Without skip, an unattended terminal stops at its first permission prompt. An unattended chat session stops there too, but it is not failed: its item stays running with the detail "Waiting for you to answer a permission prompt in the chat session.", the idle sweep leaves the session open, and the detail goes once the prompt is answered.

The queue opens a session on the task, which works the open steps in order, claiming each (or writes the wrapup on a task with no checklist). A claim is enough: the queue never waits for a step or a task to be accepted, so a finished task reads "All of it claimed and waiting for review" while the next one already runs. A flow is finished when it reaches its end, even if some of its steps were never needed and stay open: the task reads "Flow finished: the steps it did not need were left unrun". Sending a step back from review, or resetting the flow's moves, makes the task work again. When nothing is left open, the queue closes the session and starts the next task. If the ceiling is reached at a claim, it closes the session, puts the task back at the head of the queue, sends any step the session had started back to open, and holds; the next session picks up at the first open step. A task with nothing open is skipped. A task fails, and the queue moves on, when the session can't open (no folder, no CLI, a session already open on it) or ends before its work is claimed. **Stop the queue** closes the running session after a confirmation. The queue survives a restart: a task that was running goes back to waiting, and an armed queue picks it up again.

**Sessions in a multi-step task.** By default one session works the whole checklist. A step can open a fresh session with the same provider. Set `new_session` when a session proposes it (`rekall_propose_step`, `rekall_propose_flow`, `rekall_replan`; `/rk … plan` and `replan` always cut the sessions, picking the grouping they judge cheapest, without being asked), or switch it by hand with **New session from here** on the seam between two steps in the checklist (**Join session above** on a session's header undoes it), with **New session** on a draft, or when adding a draft. The first step always opens the first session, and the steps up to the next one that opens a session share one. The checklist draws a header over each session when there is more than one. A session claims the last step of its group, writes its wrapup and stops; Rekall opens another session on the same task with the same provider, view, model, effort and permissions, and the pane switches to it. A queued task stays running across its sessions, and the usage ceiling is checked before the next one opens. The choice is stored per step (`PATCH /api/steps/{id}/session` with `startsSession`) and `rekall_context` shows it as `(opens a new session)` and `sessions="N"` on `<steps>`.

**Run in queue** opens the same launch settings form from a task’s run menu. It starts with the top bar provider and that provider’s defaults. For Claude Code it offers the family names and the model saved in Settings. You can choose a different provider for this entry and review its model and effort before confirming. Confirming adds the task and arms the queue in one request. An idle queue starts at once; a running, holding or scheduled one only gains the task, which runs when its turn comes. A task left waiting by an earlier **Stop the queue** runs ahead of it, because the queue is armed in order. A task already queued or running is refused with 409.

**More ways to run** opens from the arrow beside **Run here** on a task or its preview.
**Discuss**, **Plan here**, **Plan a flow** and **Replan** use the live session’s provider or the
top bar choice. The menu names the provider on its first line. When that CLI is missing the menu says so,
and links to its setup in Settings. **Run in queue** still lets you choose another provider.
Close any live session on the task before confirming its queue entry.
**Plan a flow** is available on tasks with no steps. **Replan** asks for confirmation when draft
or open steps can be replaced. From the **Decisions** tab, **Refine** opens or reuses a session directly and records answers
in that task’s ledger.

`/api/run-queue` holds the queue: `GET`, `PUT /settings` (`ceilingPercent`, `skipPermissions`, `model`, `effort`, `sessionKind`), `POST /items` (adds without starting), `POST /runs` (`taskId`: adds the task and arms the queue), `DELETE /items/{id}`, `PUT /items/{id}/position`, `POST /clear`, `POST /start` (`startAt` null starts now), `POST /stop`. Each answers with the whole queue, and every change is pushed on the console's event stream as a `run-queue` frame.

Both add endpoints take `taskId` and a provider (`CLAUDE` or `CODEX`). A Claude Code launch sends `claudeOptions` with model and effort. A Codex launch sends `codexOptions` with model, effort and the bypass preference. A null model or effort delegates to the provider’s account. Legacy requests with no provider or options use Claude Code and the queue’s model and effort. The queue returns saved options in responses and events, including after a restart. The worker uses them for dispatch and when resuming after a hold, stop or restart.

## Token usage

The task’s token panel and the step’s evidence show generated tokens by provider, with input
and cache counts in the tooltip. The task total includes its steps. Project and calendar totals
sum the tasks. When a provider has reported no attributable usage, the panel says so.

Beside the name of a task in its steps, wrapup and terminal panes, and at the right end of the tag and skills line in its description pane, the figure sits with a small sparkline of the last month. A project's heading in the navigator carries the same chip for the sum of its tasks. Click it for the daily trend: the running total of generated tokens, one point per day, over **1 month**, **3 months** or **All**. Move over the chart, or use the arrow keys, to read a day's total and what that day added. The task's title line holds a copy button that puts the task's `/rk project:… task:…` command on the clipboard (the command is in its tooltip). Days are UTC days, as the server cuts them. The tag on a task's description is a badge that opens the list of tags to change it; a task with none shows a dashed **New tag** button, the way skills show **Add a skill**.

Rekall reads Claude Code transcripts and Codex rollouts about once a minute. It looks in the
providers’ configured homes for sessions in project folders and task worktrees. A session’s
reading replaces its previous reading. A Codex thread uses its cumulative counters, so repeated
records and archived copies do not add usage twice.

The launch prompt or a Rekall context call identifies the task. Step start moves attribution to
that step, and a claim moves it back to the task. Counts go to the named task even if its anchor
belongs to another project. When a step can no longer be resolved, its tokens stay on the task.

`GET /api/token-usage` returns tallies by task, optional step, and provider. The daily endpoint
returns task totals by day. The `token-usage` stream refreshes both views after a scan.

| Property | Default | Meaning |
|---|---|---|
| `rekall.claude.transcripts-dir` | `~/.claude/projects` | Where Claude Code keeps its session transcripts |
| `rekall.claude.transcript-scan-seconds` | `60` | How often they are read |
| `rekall.claude.home` | `~/.claude` | Claude Code's own folder, which the Claude screen reads and edits |

