# Rekall

Rekall stores your projects, tasks and markdown notes, and loads one task in full into a Claude Code session on a single command.

Projects, tasks and notes live in a local app. `/rk project:vega task:report-builder` loads that task, its project, its checklist and every attached note. `/rk project:vega task:report-builder wrapup` records what the implementation is now, so the next session starts from the current state instead of reading the code back.

Source-available, not open source. See [License](#license).

## Requirements

| Tool  | Version |
|-------|---------|
| Java  | 25      |
| Maven | 3.9+    |
| Node  | 22+     |
| pnpm  | 9+      |

The database is an H2 file under `./data`. No database server, Docker or cluster.

The backend also exists as a Rust port (Rust 1.85+), one binary serving the same console, API and
MCP endpoint on the same port, with a SQLite database and a Tauri desktop shell. See
[The Rust server](#the-rust-server).

## Run

```bash
make run     # build the frontend, then start on http://localhost:47355
make build   # the above, plus the packaged jar
make ui      # the frontend alone
make reset   # delete the database file (no undo)
make console # H2 shell on the database
```

`make run` and `make build` build the UI first. The frontend compiles to `rekall-ui/dist` (git-ignored) and `rekall-app` copies it into the jar under `static/` at package time. Packaging without a built UI fails with instructions to run `make ui`.

| Service | Address                      |
|---------|------------------------------|
| UI      | `http://localhost:47355`     |
| MCP     | `http://localhost:47355/mcp` |

Port 47355 is fixed because the MCP endpoint is registered with Claude Code by URL, so a clash breaks the registration rather than moving the app. `SERVER_PORT` overrides it for the server, the `.app` launcher and the tests.

### This machine only

Nothing in Rekall authenticates, and its API can open a `claude` terminal in a project folder, so `LocalAccessFilter` answers only this machine. It refuses with a 403 a peer that is not loopback (another machine on the same network), a `Host` header that is not `localhost`, `127.0.0.1` or `[::1]` (a page using DNS rebinding), and an `Origin` header from another site (a page open in the same browser). The terminal's WebSocket accepts only loopback origins too. Claude Code and `curl` send no `Origin` and pass.

| Property | Default | Meaning |
|---|---|---|
| `rekall.security.remote-access` | `false` | `true` accepts other machines and their own origin. It gives up the DNS-rebinding check; only for a network you trust |

## The Rust server

```bash
make server      # target/release/rekall-server, with the console embedded
make run-rust    # start it on http://localhost:47355
make test-rust   # cargo test: every crate, the ported JUnit suites included
make desktop     # the Tauri desktop app (needs the platform WebView SDK)
make import-java-db DIR=~/rekall-data   # import a Java-era H2 database into SQLite
```

Same port, same routes, same JSON, same MCP tools; the database is `rekall.db` (SQLite) in the
folder that held `rekall.mv.db`, and an existing H2 database is imported rather than recreated.
`docs/RUST-PORT.md` describes the layout, how parity with the Java server was checked, and every
place where the two still differ.

## macOS application

A disk image for Apple Silicon is published on every commit to `main`.

| | |
|---|---|
| Download | [Rekall-macos-arm64.dmg](https://github.com/ValerioMC/rekall/releases/download/latest/Rekall-macos-arm64.dmg) |
| Needs | macOS 13 or later, Apple Silicon |
| Install | Drag Rekall onto Applications, then run the `xattr` command below once |

The image tracks the head of `main` and changes under the link without notice. A fixed build is a `v*` tag. It builds on a `macos-14` runner from `.github/workflows/release.yml` and `scripts/macos-bundle.sh`. There is no Intel, Windows or Linux bundle; the jar on the same release runs on any platform with a JVM 25.

The published image is the **jvm** flavour. The GraalVM (native) build reaches 5.9 GB of heap on a 7 GB runner and the watchdog aborts it after about 30 minutes, so it builds only locally.

### Build locally

```bash
make dmg-native   # GraalVM binary in the bundle. Needs GraalVM as JAVA_HOME
make dmg-jvm      # jar plus a bundled Java runtime. Any JDK 25
```

Both need the Xcode Command Line Tools for `swiftc`. Both write `dist/Rekall-<version>-<flavour>-<arch>.dmg` and install it into `/Applications` on the build machine, replacing an existing `Rekall.app` and relaunching a running copy. `REKALL_INSTALL=0 make dmg-jvm` stops at the disk image.

The dock icon is `packaging/macos/AppIcon.png`, a 1024 master on Apple's icon grid. It, the PWA icons and `apple-touch-icon.png` are all rendered from `rekall-ui/public/favicon.svg`, which the console also shows as its logo: edit the SVG, run `make icons` (needs Google Chrome), and commit the PNGs it writes. The bundle build only reads them, so it needs neither Chrome nor the SVG.

| | `dmg-native` | `dmg-jvm` |
|---|---|---|
| Payload | `Contents/Resources/rekall-app` | `Contents/runtime` plus `rekall-app.jar` |
| Disk image | 91 MB | 99 MB |
| First screen | under a second | about three seconds |
| Build | GraalVM, 4 to 8 minutes | any JDK 25, under a minute |

### Running the app

The bundle executable is `packaging/macos/Launcher.swift`. It shows a splash screen while the server boots, then loads the console once port 47355 answers. Quitting sends `SIGTERM` so the H2 file closes cleanly. Server output goes to `~/Library/Logs/Rekall/server.log` (View > Open Server Log).

The app uses the same port, the same `~/.rekall/config.json` and the same MCP endpoint as `make run`. If something is already listening on 47355, the app attaches to it instead of starting a second server. The folder icon in the database field opens the system folder chooser, which a browser tab cannot do; in a browser that field stays a typed input.

The bundle is signed ad hoc, which is enough for the machine that built it. A disk image downloaded through a browser on another machine is quarantined and needs one command before it opens:

```bash
xattr -dr com.apple.quarantine /Applications/Rekall.app
```

## Connect Claude Code

**Settings > Claude Code** in the app registers the MCP server for every folder and installs the `/rk` command in one click. It also repairs a registration that points at the wrong port, carries an older command, or is shadowed in one folder by a registration made without `--scope user`.

From a terminal:

```bash
make mcp-add    # claude mcp add --scope user --transport http rekall http://localhost:47355/mcp
make mcp-check  # verify the endpoint answers
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

`--scope user` registers the server for every directory. Without it, `/rk` works only in the directory `claude mcp add` ran from. Restart any open session, then:

```
/rk project:vega task:report-builder
```

**Open in terminal**, on a task or a project, hands the session to your own terminal app in the project's folder with `/rk` already running. Set the folder in the **Folder** field on the project page. The terminal is iTerm2 when installed, Terminal.app otherwise. A switch in **Settings > Claude Code** adds `--dangerously-skip-permissions`; it is stored on the machine, not in the database. This button works only inside Rekall.app.

## Terminal in Rekall

**Run here**, next to **Open in terminal** on the description and steps panes, or `c` from any task, runs a real terminal inside the app. The backend starts the interactive `claude` in a pseudo-terminal (pty4j) in the project's folder, types `/rk <anchors>` as the first line, and hands it to you; the pane renders it with `xterm.js` and the bytes travel both ways over one WebSocket at `/api/terminal/{id}/io`. Because it is the same binary run the same way as in your own terminal, its prompt caching, context compaction and `/context` read-outs behave identically, and a permission prompt actually renders and can be answered. Works in a plain browser, not only Rekall.app.

The terminal gets the environment a new window of your own terminal would: Rekall asks your login shell (`$SHELL -i -l -c`) for its variables once it starts, so whatever your profile adds to `PATH` (`~/.cargo/bin`, nvm, pyenv, sdkman) and exports is there, along with `SSH_AUTH_SOCK`. The answer is cached and refreshed in the background every time a terminal opens, so a tool installed while Rekall runs reaches the next terminal but one. A shell that prints nothing usable within the timeout leaves the last good answer in place, or Rekall's own environment when there is none. The shell runs with `REKALL_RESOLVING_ENVIRONMENT=1`, so a slow profile can skip work with `[[ -n $REKALL_RESOLVING_ENVIRONMENT ]] && return`.

One terminal per task, the way one terminal window is. A second open on a task that already has one just refocuses it; opening on a different step moves the checklist marker (the old step back to open, the new one to running) without touching the process. **Restart** kills the `claude` process and starts a fresh one on the same task; **Close** ends it. Opening on a step marks that step `RUNNING` while the terminal is on it; on a task with no checklist the review line goes `RUNNING` instead, and both are released when the terminal closes. Nothing is persisted: a restart of Rekall clears every terminal and releases any step it left running, and reopening the pane replays a bounded scrollback so it repaints.

Live work sits in one bar in the bottom right corner of every screen: a segment for running timers (with the newest timer's clock) and a segment for live terminals, each shown only while it has something to count. A segment opens its sheet above the bar, one sheet at a time; pressing the other segment switches, and pressing the same one again, Escape, or a click anywhere else closes it. From the sheet a row jumps to its task (the terminal one also opens the terminal pane), stops the timer or terminal, and, for a terminal, logs the latest commit against the task and step it was opened on. The bar reports the room it takes so nothing else in that corner sits under it, whichever segments are present; the terminal segment steps aside while the terminal pane is on screen, and the bar leaves entirely when nothing is live.

**Settings > Claude Code** picks the model and effort a new terminal starts with, and the **skip permissions** switch:

- **Model** (`Account default`, `Sonnet`, `Fable`, `Opus`, `Haiku`): anything but the default adds `--model <alias>`. Each alias is Claude Code's own name for the latest model of that family, so no version is pinned.
- **Reasoning effort** (`Account default`, `Low`, `Medium`, `High`, `Extra-high`, `Max`): anything but the default adds `--effort <level>`.
- **Skip permissions** adds `--dangerously-skip-permissions`; with it off, an interactive permission prompt in the terminal is yours to answer.

All three are stored on the machine, not in the database. It works in both the jvm and native macOS bundles: the pty4j/JNA GraalVM metadata is committed under `rekall-app/src/main/resources/META-INF/native-image/`.

The top bar carries a usage meter: the current 5-hour session as a ring with its percentage and time to reset, and, on hover, a bar per window including the weekly per-model limits. The figures are the ones Claude Code's own `/usage` shows, read with the OAuth token Claude Code stores (every `Claude Code-credentials` item in the macOS keychain, else `~/.claude/.credentials.json`; the token expiring last wins, and an expired one counts as no token). With no valid token the meter asks you to open a Claude Code terminal, which signs in or refreshes the token; when Anthropic cannot be reached it holds the last figures. Readings are not real time: one when the console opens, one every five minutes while it is on screen, one when the window comes back to the front after a minute away, and one whenever you click the blank meter or **Check again** in its popover (`GET /api/claude/usage?refresh=true`, past the server's cache). If Anthropic answers 429 the meter shows **Wait** with the time left and takes nothing, not even a manual check, until that `Retry-After` has passed: asking again is what extends the block.

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
| `rekall.run-queue.settle-grace-seconds` | `8` | How long a finished session keeps its terminal before the queue closes it |

## Run queue

The run queue works through a list of tasks without you: each one in its own terminal, one after another, until everything on it is claimed. It opens from the dial next to the usage meter in the top bar, or `q`. The dial is also its status: three bars when idle, a clock hand at the start time when scheduled, an orbiting comet with `2/5` while running, and a ring stopped at the ceiling with the time it resumes while holding. A strand of beads shows each task in the run.

- **Order.** Add tasks by typing a title, label or anchor. Waiting tasks move up and down or come off. The running one stays until you stop the queue, and finished ones stay as the record of the run until **Clear finished**.
- **When.** **Start now**, or **At a time** for a schedule. Rekall has to be running then: the queue runs inside it.
- **Usage ceiling.** With it on, no new task or step starts once a usage window reaches the line: the 5-hour session and the weekly total always, plus the weekly Opus or Sonnet window when the queue runs that model. The queue then holds until the latest reset among the windows over the line, plus a minute, and carries on from there. It checks before every task and at every step claim, never mid-step, so a step that is under way finishes. Set the line below 100 to leave room for it. With a ceiling set and no usage reading available, the queue holds and looks again every five minutes.
- **Sessions.** Model, effort and **Skip permission prompts** for the terminals the queue opens. They are stored with the queue, separately from **Settings > Claude Code**. Without skip, an unattended session stops at its first permission prompt.

How each task runs: the queue opens a terminal on it, and the session works the open steps in order, claiming each (or writes the wrapup on a task with no checklist). When nothing is left open, the queue closes the terminal and starts the next task. If the ceiling is reached at a claim, it closes the terminal, puts the task back at the head of the queue, sends any step the session had started back to open, and holds; the next session picks up at the first open step. A task with nothing open is skipped. A task fails, and the queue moves on, when the session can't open (no folder, no CLI, a terminal already open on it) or ends before its work is claimed. **Stop the queue** closes the running session after a confirmation. The queue survives a restart: a task that was running goes back to waiting, and an armed queue picks it up again.

`/api/run-queue` holds the queue: `GET`, `PUT /settings`, `POST /items`, `DELETE /items/{id}`, `PUT /items/{id}/position`, `POST /clear`, `POST /start` (`startAt` null starts now), `POST /stop`. Each answers with the whole queue, and every change is pushed on the console's event stream as a `run-queue` frame.

## Anchor syntax

An anchor is `entity:value`, where the entity is `company`, `project` or `task` and the value is the record's **label** (its `name` for a company). Label rules are under [Model](#model).

| Form | Meaning |
|------|---------|
| `/rk project:vega task:report-builder` | The project and that task, both in full |
| `/rk project:vega` | The project, plus its tasks as a list of anchors |
| `/rk vega report-builder` | Positional. Works while each term matches one record |
| `/rk task:"report builder"` | Quote a value containing spaces |
| `/rk project:vega task:report-builder wrapup` | Write the task's wrapup instead of loading it |
| `/rk project:vega task:report-builder plan` | Propose the task's checklist as drafts, then stop |
| `/rk note:3f2a9c1e` | A note sent by reference, loaded in full (see [Context size](#context-size-and-reference-notes)) |

An anchor brings back the record, everything it references resolved in full with their notes, what references it as anchors, and its own markdown. A note can be attached to several tasks and arrives with each. If a bare term matches more than one record, the candidates come back and nothing loads. `project:` disambiguates a label two projects share.

## MCP tools

| Tool | Access | Effect |
|------|--------|--------|
| `rekall_context` | read | Resolve anchors, return markdown |
| `rekall_wrapup` | write | Replace one task's wrapup |
| `rekall_step` | write | Move one step: `open` to `running` to `claimed` |
| `rekall_record_commit` | write | Log a commit of the project's repo folder against one task or step |
| `rekall_propose_step` | write | Add one step as a draft, for a person to promote |

There is no query, get or schema tool. `rekall_step` refuses `done`; that state is set by hand in the console. `rekall_step` (on `claimed`) and `rekall_wrapup` take an optional `commit_message`, used only on a project that commits on claim (see [Commit on claim](#commit-on-claim)).

## Wrapup

A task has at most one wrapup: what its implementation currently is, not a changelog. It names code by the code's own names (class, file, endpoint, table) and does not transcribe field lists or signatures. It is capped at 20,000 characters, against 100,000 for a note.

A quoted term after `wrapup` is a directive on what to write. It can narrow the subject, dictate the wording, or set the language or length.

```
/rk project:vega task:report-builder wrapup "export module only"
```

The description pane and the steps pane both carry a **Generate the wrapup every session** toggle under the header, so it is set from whichever surface the work is driven from. Turning it on reveals an optional directive field. Set once on the task, the toggle stands in for the quoted term: the context load then tells the session a wrapup is expected every time without being asked, and the words it should follow. Turning the toggle off drops the directive with it.

A wrapup written after a step finishes folds that step's work into the same description. The console counts steps ticked and notes added since the wrapup was last written. You can edit the wrapup in the console; the next `/rk … wrapup` replaces it and the tool reports when it overwrites a hand edit.

### History

**History**, on the wrapup and on the description, lists the earlier versions of that text, newest first, and restores one. `TaskRevisionService` keeps what a write replaces: every session write, every delete and every restore, so a restore can be undone the same way. The console autosaves as you type, so a hand edit over hand-written text keeps one version per ten minutes, the one from before you started, not every keystroke; a hand edit over a session's text is always kept. The newest 30 versions of each text are kept per task. Sessions never read the history. `GET /api/tasks/{id}/revisions?kind=WRAPUP|DESCRIPTION` lists it and `POST /api/tasks/{id}/revisions/{revisionId}/restore` restores one.

## Steps

A step sits on a line: **draft** while you are still wording it, **open** once you promote it and it is ready to work, **running** while a session works it, **claimed** when the session reports it finished, **done** when you accept it. Each step is a title and an optional markdown detail. The pane opens on the first step whose work is not finished.

A new step is created as a draft and sits on a staging shelf below the checklist, off the rail. **Promote** moves it onto the list; an open step not yet started can go back with **To draft**. A draft is not work: it stays out of the checklist a session reads, `rekall_step` will not move it, and the navigator's step count ignores it. A task that holds only drafts is still treated as having no checklist.

A session drives its own checklist over `/rk`:

```
/rk project:vega task:report-builder step:3 start   # step 3 -> running
/rk project:vega task:report-builder step:3 done    # step 3 -> claimed
```

The console holds one `text/event-stream` connection (`GET /api/steps/stream`) carrying five frames: `steps` for a checklist, `task-review` for a stepless task's review line, `wrapup` for a wrapup write or delete, `commit-reference` for a commit logged against a task or step, and `run-queue` for the run queue. A step moved from an in-app terminal, a box ticked in another window, a wrapup written over MCP, or a commit logged by a session all land without a reload.

Claude receives an open or running step with its detail, tagged `(in progress)` or `(claimed, …)`. A finished step arrives as its title alone. A draft step is not sent at all, only counted as `draft="N"` on the `<steps>` tag. A step ticked without a following wrapup is marked `(finished since the wrapup was written)` and handed back with its detail until the next wrapup folds it in.

With steps on a task, the open steps are the work and the description becomes the constraints the steps are built against. Anything the description asks for that no open step covers is not built; `/rk` flags it and asks for the step.

Only you set a step to **done**. `rekall_step` stops at `claimed`. The navigator's progress count is built on `done`.

The mark in front of each navigator row says where an in-progress task's work stands. A hollow amber ring around a small light means nothing is handed in yet, and a green arc on that ring is the share of the checklist already accepted. An orbiting comet means a session or a timer is on the task now; it is the only mark that moves. An amber check in an open ring means work is claimed and waiting for your review. A filled green seal means every piece of work is accepted and the task is ready to move to Done. Tasks in any other status keep a plain dot in their status colour. Hover the mark to see its meaning in words.

### Planning

`/rk project:vega task:report-builder plan` has a session turn the task into a checklist for you to review. It reads the description, the wrapup, the finished steps and the code the task touches, then calls `rekall_propose_step` once per step, in order: a title saying what the step delivers, a detail saying what to build, where, what it must satisfy and how you can tell it is done. Every proposal lands as a draft on the staging shelf, so nothing is work until you promote it, and the session builds nothing. A title the task already has is refused, so a second `plan` adds only what the first one missed; a task holds at most 20 drafts.

The Steps pane starts it in two places. An empty checklist shows a `/rk … plan` chip to copy into any session. While a session runs on the task in the console's terminal, **Plan here** in the pane header types the command into it. Nothing opens a fresh session for it, because a fresh one starts with a plain `/rk` and would start working the task instead of planning it.

A claimed step is reviewed from its detail: **Accept** ticks it to done, **Send back** returns it to open for another pass. The **N awaiting review** count in the pane header jumps to the first one. The step node itself only moves a step forward, so a stray click never walks it back; reopening an accepted step is a separate **Reopen** button that arms before it fires.

## Commits

A task, or one of its steps, keeps a ledger of the commits that built it: the hash, the commit's own subject line as the comment, and the diff it introduced, read from the project's **repo folder** (set on the project's page). The console shows the ledger under the description as a collapsible rail, and under each step's detail; a row opens to its diff, and can be deleted by hand.

**Log commit** on the description, or inside a running or claimed step, logs the tip of the repo. The chevron next to it opens a picker over the last 30 commits, newest first, with their subject and age, so an earlier commit can be logged against the task or step instead; rows already logged there are marked. A hash the log does not reach can be pasted in the same panel, abbreviated or full. A hash that names no commit is refused and nothing is written. Logging the same commit against the same task and step twice is a no-op.

A session does the same over MCP right after `git commit`:

```
rekall_record_commit  anchors="project:vega task:report-builder"                 # the tip
rekall_record_commit  anchors="project:vega task:report-builder" step="3"        # against step 3
rekall_record_commit  anchors="project:vega task:report-builder" commit="a0fd5cc" # an earlier commit
```

A logged commit stays out of `rekall_context` until it is chosen for it. Each row carries an **add to context** toggle (shown on hover; **in context** once on) and the rail fills the node of every chosen commit and counts them. A chosen commit travels with the task's context as a `<commit hash="…" step="…">` element inside `<commits>`: its subject, then its diff (capped at 60,000 characters), oldest logged first. Choosing is console-only; `rekall_record_commit` logs but never chooses.

The REST side is `GET /api/commit-references`, `POST /api/tasks/{id}/commit-references/latest`, `POST /api/tasks/{id}/commit-references` (body `commitHash`, optional `stepId`), `GET /api/tasks/{id}/recent-commits`, `GET /api/commit-references/{id}/diff`, `PATCH /api/commit-references/{id}` (body `inContext`) and `DELETE /api/commit-references/{id}`.

### Commit on claim

A project can commit for the session instead of waiting for it to. The **Commit on claim** switch sits under the **Folder** field on the project page and only arms when that folder is a git repository: the strip under it says which branch is checked out and whom git would commit as there (`user.name` / `user.email` as `git config` resolves them in that folder, so the global identity unless the repo overrides it). Outside a repository the switch stays off and says why; on a repository with no `user.email` it arms but warns that the commit will fail until one is set. The setting is saved on the project (`autoCommit` on `PUT /api/projects/{id}`; the server keeps it off whenever the folder is not a repository) and `GET /api/projects/{id}/repository` is what the strip reads.

With it on, a claim commits: `rekall_step … state="claimed"` stages everything in the folder (`git add -A`), commits it, and logs that commit against the step; on a task with no checklist, the Claude-authored `rekall_wrapup` that claims the task does the same against the task. A task with a checklist never commits on its wrapup.

The message says what the work does, never which files it touched. The session that claims writes it, as `commit_message` on `rekall_step` or `rekall_wrapup`: a Conventional Commits subject under 72 characters, a blank line, and a few sentences of body. Without one, `CommitMessageGenerator` derives it: the subject is the step or task title, typed `fix:` or `refactor:` when the title says so, `docs:` when every file is documentation, `test:` when every file is a test and `feat:` otherwise; the body is the opening of the step's detail (or of the wrapup, for a task with no checklist) as plain prose, headings, emphasis and code dropped, cut at a sentence. Either way the body is wrapped at 72 columns and ends with a `Refs: project:… task:…, step N` line. A clean tree commits nothing and says so; a git refusal (no identity, a hook, a folder that stopped being a repository) is reported in the tool's answer and the claim stands either way. `rekall_context` marks such a project with an `auto-commit` field telling the session not to `git commit` itself.

## Description review

A task with no checklist walks the same line at task scope: **open**, **running** while a terminal is open on it with no step target, **claimed** when a Claude-authored wrapup lands, **accepted** when you accept it in the console. Nothing new is typed for it: running follows the terminal and claimed follows the wrapup write. The description pane shows the running pill and a review bar: on **running** it carries **Accept** and a link to the wrapup, so a session driven by hand still has a console exit; on **claimed** it adds **Send back** (with an optional note the next session sees); accepting offers to also mark the task done. A note left on send back rides in `rekall_context` as a `review` field on the task (`sent back — "…"`), so the next session opened on that anchor reads why, not just that it was. It arrives on the same `GET /api/steps/stream` connection as a `task-review` frame, the wrapup that claims it rides the same connection as a `wrapup` frame so the pane shows the new text with the claim rather than on the next reload, and `PATCH /api/tasks/{id}/review` is the console-only Accept / Send back. Adding a first step retires the task-level line and the checklist takes over.

## Review queue and notifications

**Review** (the checklist icon with a count, in the top bar) counts everything a session claimed and you have not reviewed, across every task: claimed steps, and tasks with no checklist whose wrapup claimed them. It lists them longest-waiting first, and a row opens the step on its steps pane, or the task on its description's review bar. It is built in the console from the same step and review frames the event stream already carries, so it moves as sessions claim and you accept, and it is not there while nothing waits.

When something joins the queue while the console is hidden or another app has the focus, Rekall posts a system notification: through the native bridge in Rekall.app (`packaging/macos/Notifier.swift`, the system asks for permission the first time), through the browser's Notification API elsewhere. What was already waiting when the console loaded never notifies. **Settings > Notifications** turns it off; the choice is stored on the machine, and in a browser that is also where permission is asked, since a browser grants it only from a click.

## Console

One surface, three panes: pick a task on the left, pick its checklist, its wrapup, a note or a session in the middle, write on the right. The field at the top takes the same grammar as `/rk`.

That field filters the navigator by titles and labels as you type. For a phrase of three characters or more that is not an anchor, it also searches the text behind them and lists the hits under the bar: task descriptions, steps (title and detail), wrapups and notes, each with the words around the match. `↑` `↓` choose, `↵` opens the hit on the pane that shows that text (a step opens expanded on its steps pane). `GET /api/search?q=` answers it: at most 8 hits per kind, newest first, the phrase matched whole and case-insensitively, `%` and `_` literal.

The description, steps, wrapup and terminal are pinned above the notes. Each opens in the writing pane; a task missing one shows an empty card. `c` opens the terminal pane, the same way `s`, `w` and `d` open steps, wrapup and description; `q` opens the run queue; `r` toggles read/write on whichever of the description or a note is open. Companies, projects and tasks are created, edited and deleted from one editor, opened on the parent record. Title and label sit together with the anchor assembled live as you type. Deleting states what goes with it.

A task optionally wears one **tag**: a name, a glowing icon and a glow colour, both picked from a fixed set the console already knows how to draw. Tags are configured in their own panel, opened from the star button in the header next to Settings — add, rename, re-colour or delete one there, with a live count of the tasks currently wearing it. A tag is assigned to a task from that task's edit dialog, and shows as a small glowing badge next to the title wherever the task is listed.

Finished tasks are folded into a "filed" drawer, closed on every load. Writing autosaves; a note has no Save button.

A note is put on a task from either side. From the note, the task chips on its pane open a picker that walks company, project and task. From the task, the **Notes** button in the description and steps headers drops a list of every note under itself, the ones on this task first: one click, or `↑` `↓` and `↵`, adds a note or takes it off, without leaving the pane. A note whose only task is this one stays put, since a note needs at least one.

The note cards in the task's column take a note off the task too. The right end of a card says where else the note lives; under the pointer, or once the card has focus, it becomes a `×` that takes the note off this task without asking, and `⌫` on a focused card does the same. The toast that follows offers **Undo** for a few seconds, which puts the note back and reopens it if it was the one in the editor. On a note that is only on this task the same spot is a bin: taking it off its last task is deleting it, so a confirm names that before anything is written.

`b` switches the left column between tasks and notes. Browsing notes, picking one leaves the task in view alone: the middle column becomes the note's placements, the tasks it is on grouped by project. **Put it on a task** at the top opens the same company → project → task picker the note's "+ task" chip opens on the tasks side. Each row carries a `×` at its right end that takes the note off that task, and under the pointer it colours the row it is about to empty and says **Take off**; the row itself does nothing on a click. On the note's last task that control is **Delete note** and asks first. The arrow on a row is the only thing that opens that task on the tasks side. Switching back to tasks brings the task up to date with the note.

A note starts from either side too. Browsing tasks, **New note** (or `n`) writes one on the task in view. Browsing notes, the same button, `n`, or **+ New note** at the top of the list turns the middle column into a composer: a name, and the tasks to put it on, with the task in view ticked to begin with and the live tasks in scope offered underneath. Typing finds any task, `↵` ticks it, `⌘↵` or **Create note** makes the note and opens it in the editor. A note lives on at least one task, so nothing is created until one is ticked; `esc` walks away. **Delete** on the editor header removes the note from every task it is on.

## Report

**Report** shows what went to which client and for how long. The frame is a week or a month, stepped with the arrows either side. One column per day is stacked in each company's colour against a dashed line at eight hours. Below it, a section per company, its projects and its tasks, with the hours and the days each ran on.

Every task row opens on the steps it closed inside the period, oldest first, with the day each was ticked and a count of those still open. A step ticked outside the period is counted but not named. The chips narrow the report to the companies you pick. **Copy as markdown** puts the whole report on the clipboard with every task's anchor and its closed steps.

The screen is built from the sessions the timer recorded. A session counts on the day it started; one still running counts up to now.

## Model

```
Company ──< Project ──< Task >──< Document
                         │       via document_task
                         ├──< TaskStep
                         ├──1 Wrapup
                         └──> Tag (optional)
```

| Entity | Anchored by | Holds |
|--------|-------------|-------|
| `Company` | `name` | description, its projects |
| `Project` | `label`, unique per company | title, status, description, its tasks |
| `Task` | `label`, unique per project | title, status, markdown description, a standing wrapup directive, its notes, steps, wrapup, an optional tag |
| `Document` | none | title, kind, markdown body, the tasks it is on |
| `TaskStep` | through its task | title, optional detail, state, position |
| `Wrapup` | through its task | markdown body, who wrote it last. One per task |
| `Tag` | `name`, unique | icon key, glow-colour key, the tasks currently wearing it |

`label` is what an anchor resolves: lowercase letters, digits, `-`, `_`, `.`, no spaces, normalised on write. `title` is free text and changing it never breaks an anchor. Renaming a label moves the anchor, and the editor says so before saving.

A project belongs to one company, a task to one project. A note belongs to at least one task and often several. Deleting a task unlinks its notes and removes only the ones left on nothing. A wrapup and a step belong to exactly one task and are deleted with it. Adding an entity is a JPA class plus a Liquibase changeset, not a UI action.

## Context size and reference notes

Next to the anchor on a task's description, a chip estimates what `/rk project:… task:…` costs a session: the characters of the markdown it hands over and roughly how many tokens that is (3.5 characters to a token, an estimate for comparing tasks, not a bill). Under the pointer it lists the parts heaviest first: the description, the steps, the wrapup, the commits chosen for the context, each note, and the project around them. `GET /api/tasks/{id}/context-size` measures it with the same `ContextRenderer` that answers `rekall_context`, so the figure is the length of what a session gets.

A note most sessions do not need can go **By reference**, switched on the note's own pane. It then travels as its title, its first line and an anchor such as `note:3f2a9c1e` (`loaded="on request"`), and a session loads the rest with `rekall_context` and that anchor only when the work asks for it. **In full** puts it back. The mode belongs to the note, so it holds on every task the note is on.

## Backups

H2's `BACKUP TO` copies the open database, consistently and without stopping it, into a `backups` folder beside it (for the default location, `./data/backups`). One is taken when Rekall starts and whenever the newest is older than the interval, checked every hour; **Back up now** in **Settings > Backups** takes one on demand, and every restore takes one first. Only the newest are kept.

| Property | Default | Meaning |
|---|---|---|
| `rekall.backup.enabled` | `true` | Scheduled backups. **Back up now** and restores work either way |
| `rekall.backup.interval-hours` | `24` | Age of the newest backup that makes another one due |
| `rekall.backup.keep` | `10` | Backups kept, of every kind together; the oldest go first |

**Restore** on a listed backup, or **Restore from a file…** with a zip from elsewhere, replaces the whole database: the zip has to hold an H2 database file (`*.mv.db`, checked by its header), what is there now is backed up first, and Rekall restarts on the restored file, migrating it forward if it came from an older version. That round trip is also how a database moves to another machine: **Download** a backup here, restore it there. An in-memory database has no backups. The REST side is `GET` and `POST /api/backups`, `GET /api/backups/{name}`, `POST /api/backups/{name}/restore` and `POST /api/backups/restore` (multipart `file`, up to 1 GB).

## Export

```bash
curl -OJ http://localhost:47355/api/export
```

Or the **Export** button in the top bar. The archive is a folder tree, one folder per company, then project, then task, one markdown file per note, plus a `MANIFEST.md` with statuses and anchors. It is for reading, and nothing reads it back: to save and restore, or to move a database, use a [backup](#backups). A note on several tasks appears under each; `MANIFEST.md` lists the copies.

## Develop

```bash
make ui-dev   # Vite dev server on :5173, proxying /api and /mcp to :47355
make test     # backend tests, then frontend lint, types and unit tests
```

`rekall-app/src/main/resources/claude/commands/rk.md` is a symlink to `.claude/commands/rk.md`. Maven copies the content, not the link.

### Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API, `<script setup lang="ts">` |
| Shell | One surface, three panes. No router |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, semantic tokens in `src/assets/main.css` |
| Fonts | Fira Sans and Fira Code, bundled |
| State | Pinia setup stores; `console.store` holds the working set |
| HTTP | `ofetch`, timeout, retry on 5xx only, correlation id per request |
| Validation | Zod on every response |
| Markdown | `md-editor-v3` wrapped by `AppMarkdownEditor`, highlight.js passed in as a local instance |
| Tests | Vitest + Vue Test Utils |

```
src/
├── model/        domain types and branded ids
├── api/          http calls and Zod schemas, no state
├── stores/       Pinia
├── composables/  reusable logic without UI
└── components/
    ├── ui/       atomic and presentational
    └── console/  the three panes, the anchor bar, the editors
```

### Debug (IntelliJ IDEA)

The backend uses Lombok. Enable Settings > Build, Execution, Deployment > Compiler > Annotation Processors > **Enable annotation processing**, or the IDE reports missing getters on code that compiles with Maven.

1. Run > Edit Configurations > Add > Remote JVM Debug, host `localhost`, port `5005`
2. Start with the debug port open:

```bash
mvn -pl rekall-app -am spring-boot:run \
  -Dspring-boot.run.jvmArguments="-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005"
```

## Environment variables

All optional; the defaults run against `./data/rekall`.

| Variable | Default | Description |
|----------|---------|-------------|
| `REKALL_DB_URL` | `jdbc:h2:file:./data/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1` | JDBC url |
| `REKALL_DB_USER` | `rekall` | |
| `REKALL_DB_PASSWORD` | `rekall` | |
| `SERVER_PORT` | `47355` | HTTP port for the UI, the API and MCP |

Notes are stored in plain text in the database file. Credentials kept in them are only as protected as the disk is.

## Modules

```
rekall-common/     ConflictException, NotFoundException: the error vocabulary every layer shares
rekall-model/      Company, Project, Task, Document, hosted-session entities and their state rules
rekall-repository/ Spring Data repositories and the Liquibase changelog for their schema
rekall-service/    Business logic: context assembly, the step and review lines, wrapups, time entries
rekall-api/        REST API for the UI, and the step event stream
rekall-mcp/        MCP server, on rekall-service and never rekall-api: one tool reads, four write
rekall-claude/     the in-app terminal (pty4j over one WebSocket), the Claude Code usage meter and the run queue
rekall-app/        Spring Boot entry point, serves everything
rekall-ui/         Vue 3 + Vite frontend
```

Classes still live under the `dev.rekall.domain.*` packages they had before the split; the module
boundary, not the package name, is what keeps `rekall-repository` off the API's classpath and the
MCP server off the write controllers. That is also why `CatalogService`, `DocumentService` and
`RevisionRestoreService` live in `rekall-api` and not in `rekall-service`: they write the catalog,
and the MCP module must not be able to reach them. Read-only services (`SearchService`,
`ContextSizeService`) and the narrow writes a session is allowed are in `rekall-service`.

## Run tests

```bash
mvn test                                            # everything, against in-memory H2
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

`RekallEndToEndTest` drives the real HTTP API and MCP endpoint against the same Liquibase changelogs the application uses, so a migration that disagrees with an entity fails there rather than at startup. `BackupApiTest` runs on a file database in a temporary folder, since an in-memory one has no backups.

`.github/workflows/check.yml` runs all of it on every pull request and every push to a branch other than `main`: `mvn test`, then eslint, vue-tsc, vitest and a production build of the UI. `release.yml` builds and publishes `main`.

## Design

`docs/DESIGN.md` records the decisions and the reasoning, including the ones that were reversed and why.

`docs/MEMORY.md` is the memory soak of the console: what was measured, how, and why the numbers say there is no leak.

`docs/SPECIFICATION.md` describes what the application is and does, entities and rules only, with no visual direction. `docs/native-image-hibernate.md` records what it took to run Hibernate in the GraalVM native image.

## License

Source-available, not open source. Licensed under the
[PolyForm Internal Use License 1.0.0](LICENSE): you may read the source, run the
software for your own and your company's internal business operations, and change it
for those purposes. You may not distribute it, in original or modified form.

Copyright 2026 Valerio Mario Casale.
