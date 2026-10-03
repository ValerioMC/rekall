# Rekall

Rekall stores your projects, tasks and markdown notes, and loads one task in full into a Claude Code session on a single command.

Projects, tasks and notes live in a local app. `/rk project:vega task:report-builder` loads that task, its project, its checklist and every attached note. `/rk project:vega task:report-builder wrapup` records what the implementation is now, so the next session starts from the current state instead of reading the code back.

Source-available, not open source. See [License](#license).

## Requirements

| Tool  | Version |
|-------|---------|
| Rust  | 1.85+   |
| Node  | 22+     |
| pnpm  | 9+      |

The backend is one Rust binary, `rekall-server`, serving the console, the REST API and the MCP endpoint on one port. The database is a SQLite file (`rekall.db`) in a folder you choose on first run. No database server, Docker or cluster.

## Run

```bash
make run     # build the frontend, then start on http://localhost:47355
make build   # the above, as target/release/rekall-server, without starting it
make start   # start the binary `make build` left, no rebuild
make ui      # the frontend alone
```

`make run` and `make build` build the UI first. The frontend compiles to `rekall-ui/dist` (git-ignored) and `rekall-app` embeds it into the binary at compile time, so `rekall-server` is the only file that has to travel. `rekall.ui.dist=<folder>` serves the console from a folder instead.

| Service | Address                      |
|---------|------------------------------|
| UI      | `http://localhost:47355`     |
| MCP     | `http://localhost:47355/mcp` |

Port 47355 is fixed because the MCP endpoint is registered with Claude Code by URL, so a clash breaks the registration rather than moving the app. `SERVER_PORT` overrides it for the server, the desktop app and the tests.

### This machine only

Nothing in Rekall authenticates, and its API can open a `claude` terminal in a project folder, so the local-access guard (`rekall-app/src/security.rs`) answers only this machine. It refuses with a 403 a peer that is not loopback (another machine on the same network), a `Host` header that is not `localhost`, `127.0.0.1` or `[::1]` (a page using DNS rebinding), and an `Origin` header from another site (a page open in the same browser). The terminal's WebSocket accepts only loopback origins too. Claude Code and `curl` send no `Origin` and pass.

| Property | Default | Meaning |
|---|---|---|
| `rekall.security.remote-access` | `false` | `true` accepts other machines and their own origin. It gives up the DNS-rebinding check; only for a network you trust |

## macOS application

A disk image for Apple Silicon is published with every `v*` tag pushed on a commit of `main`.

| | |
|---|---|
| Download | [Rekall-macos-arm64.dmg](https://github.com/ValerioMC/rekall/releases/latest/download/Rekall-macos-arm64.dmg) |
| Needs | macOS 13 or later, Apple Silicon |
| Install | Drag Rekall onto Applications, then run the `xattr` command below once |

The link always resolves to the newest tagged release. A push to `main` builds nothing; a release is cut with `git tag v0.1.1 && git push origin v0.1.1`, and a tag on a commit that is not on `main` fails before anything is built. It builds on a `macos-14` runner from `.github/workflows/release.yml`. The same release carries `rekall-server` for Linux and Windows, with the console embedded.

### Version and updates

A release build carries the tag it was built from (`v0.1.1` shows as `v0.1.1`; the workflow compiles it in as `REKALL_VERSION`); a local build shows the crate's version. `GET /api/version` asks GitHub for the latest release and reports whether its tag is newer than the running version, with the file for the platform the build runs on (the disk image on macOS) or the release page when the release has none; `?refresh=true` skips the remembered answer. The answer is remembered for six hours (five minutes after a failure) and a restart forgets it. Only `MAJOR.MINOR.PATCH` tags are compared.

The header shows no version. The desktop app checks once when it starts and, when a newer release exists, opens an update dialog (**Later** / **Install**); nothing is shown when GitHub cannot be reached. **Settings → Version** shows the running version, checks again on request and carries the same **Install** button, in a browser tab too (there it links the download).

| Property | Default | Meaning |
|---|---|---|
| `rekall.update-check.enabled` | `true` | `false` never contacts GitHub; Settings says checks are off |
| `rekall.update-check.url` | the repository's `releases/latest` API | The endpoint answering with the latest release |

In the macOS app **Install** puts the new version in place itself (`rekall-app/src/version/update_installer.rs`, called by the `installUpdate()` bridge in `rekall-app/desktop/src/update_install.rs`):

1. It downloads the release's `.dmg` from inside the app, and only from `github.com/ValerioMC/rekall/releases/download/`.
2. It mounts the image with `hdiutil`, copies `Rekall.app` with `ditto` beside the running one as `.Rekall.app.new`, and swaps the two. The old bundle is kept as `.Rekall.app.old` until the new one is in place, and is put back if the swap fails.
3. It restarts into the new version, stopping the server cleanly first and reopening the app through the system so the new window comes to the front, not behind other apps. When a Claude session is live it asks first, in the console's own dialog, as quitting does (the system dialog only stands in when the console cannot show one).

The file never passes through a browser, so it gets no `com.apple.quarantine` and the new version opens without `xattr`. When the install fails (the app runs from the mounted disk image, `/Applications` is not writable, the release has no disk image), the dialog says why and its button opens the download in the browser. A browser tab, and the desktop app on other platforms, keep the plain download link. The first install from a browser download still needs the `xattr` command below once: only a Developer ID signature with notarization removes that.

A real install of the published release into a temporary folder (network, macOS only): `cargo test -p rekall-app live_ -- --ignored`.

### Build locally

```bash
cargo install tauri-cli --version "^2" --locked   # once

make dmg      # Rekall.app and its disk image, under target/release/bundle
make app      # the above, then mount the disk image and run Rekall from it
make install  # the above, then copy Rekall.app into /Applications
```

The app is a Tauri v2 window (`rekall-app/desktop`) over the same server `rekall-server` runs, started in the same process. `make app` installs nothing: it quits a running Rekall, ejects a Rekall disk image left mounted by an earlier build, mounts the new one and opens `Rekall.app` from the mounted volume (`scripts/macos-app.sh`), so what you try is exactly what another machine gets and a copy in `/Applications`, if there is one, is left alone. Eject the volume in Finder when you are done. To install it, drag it onto Applications from that window as anyone else would, or run `make install`, which does the same drag-and-drop by quitting any running Rekall, mounting the disk image, replacing `/Applications/Rekall.app` and ejecting the image (`scripts/macos-install.sh`).

`make desktop` builds the app's binary alone, on any platform with the WebView SDK (WebKitGTK on Linux); it is not a default workspace member, so `cargo build` and `cargo test` work on a machine without one.

The dock icon is `rekall-app/desktop/icons/icon@2x.png`, a 1024 master on Apple's icon grid (named `@2x` so Tauri's bundler reads its pixel size as retina density). It, the PWA icons, `apple-touch-icon.png` and `rekall-ui/public/favicon.svg` (a 256 copy wrapped in an SVG, which the console also shows as its logo) are all rendered from `scripts/icon-master.png`, a 1024 plate with transparent rounded corners: replace the master, run `make icons` (needs Google Chrome), and commit what it writes. The bundle build only reads them, so it needs neither Chrome nor the SVG.

### Running the app

The window fades in as a centered splash card (`rekall-app/desktop/splash/`, artwork `splash_screen.png`) sized at 40% of the screen's width, between 640 and 960 points, in the artwork's proportions, while the server boots; once port 47355 answers and the splash has been up for at least 1.5 s, the window fades out, grows to fill the screen unseen, waits for the console to finish loading (up to 5 s) and fades back in on it. Fades need macOS window opacity; elsewhere the window is simply shown and resized. Quitting stops the server and closes the database cleanly. When a Claude session is still live (a terminal started with **Run here** or by the run queue), `⌘Q`, the console's close button and the window's own close first ask whether to quit, since the session ends with the server; **Keep working** returns to the app. `rekall-app/desktop/src/exit_guard.rs` does the asking, in the console's own dialog (`LeaveDialog.vue`, answered through the `answerLeave()` bridge) and in a system dialog only when the console is not there to show it. Nothing is asked when no session is live, when the app attached to a server it did not start (the sessions outlive the window), or on `kill` and logout; Dock > Quit bypasses the question, because macOS ends the process there without telling the app. Server output goes to `~/Library/Logs/Rekall/server.log` (View > Open Server Log).

The app uses the same port, the same `~/.rekall/config.json` and the same MCP endpoint as `make run`. If a Rekall server already answers on 47355, the app attaches to it instead of starting a second one, and leaves it running on quit. The folder icon in the database field opens the system folder chooser, which a browser tab cannot do; in a browser that field stays a typed input.

The bundle is signed ad hoc, which is enough for the machine that built it. A disk image downloaded through a browser on another machine is quarantined and needs one command before it opens the first time (updates installed from the app do not, see *Version and updates*):

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

**Open in terminal**, on a task or a project, hands the session to your own terminal app in the project's folder with `/rk` already running. Set the folder in the **Folder** field on the project page. The terminal is iTerm2 when installed, Terminal.app otherwise. A switch in **Settings > Claude Code** adds `--dangerously-skip-permissions`; it is stored on the machine, not in the database. This button works only inside the desktop app.

## Terminal in Rekall

**Run here**, next to **Open in terminal** on the description and steps panes, or `c` from any task, runs a real terminal inside the app. The backend starts the interactive `claude` in a pseudo-terminal (`portable-pty`) in the project's folder, types `/rk <anchors>` as the first line, and hands it to you; the pane renders it with `xterm.js` and the bytes travel both ways over one WebSocket at `/api/terminal/{id}/io`. Because it is the same binary run the same way as in your own terminal, its prompt caching, context compaction and `/context` read-outs behave identically, and a permission prompt actually renders and can be answered. Works in a plain browser, not only the desktop app.

The terminal gets the environment a new window of your own terminal would: Rekall asks your login shell (`$SHELL -i -l -c`) for its variables once it starts, so whatever your profile adds to `PATH` (`~/.cargo/bin`, nvm, pyenv, sdkman) and exports is there, along with `SSH_AUTH_SOCK`. The answer is cached and refreshed in the background every time a terminal opens, so a tool installed while Rekall runs reaches the next terminal but one. A shell that prints nothing usable within the timeout leaves the last good answer in place, or Rekall's own environment when there is none. The shell runs with `REKALL_RESOLVING_ENVIRONMENT=1`, so a slow profile can skip work with `[[ -n $REKALL_RESOLVING_ENVIRONMENT ]] && return`.

One terminal per task, the way one terminal window is. A second **Run here** on a task that already has one refocuses it and types `/rk <anchors>` into the live session, so it reloads the task with any steps written since; opening on a different step also moves the checklist marker (the old step back to open, the new one to running) without restarting the process. **Restart** kills the `claude` process and starts a fresh one on the same task; **Close** ends it. Opening on a step marks that step `RUNNING` while the terminal is on it; on a task with no checklist the review line goes `RUNNING` instead, and both are released when the terminal closes. Nothing is persisted: a restart of Rekall clears every terminal and releases any step it left running, and reopening the pane replays a bounded scrollback so it repaints.

**Run here** starts the session without leaving the pane. Pressed on a whole task with nothing left to execute (every checklist step claimed or done with only drafts remaining, or a task with no checklist whose review is claimed and only drafts remain), it starts nothing and shows a "Nothing to run" message; a press on one chosen step always launches. The play mark runs out of the button, a ring closes on a check, and the button then reads **Working**, with a comet orbiting, for as long as Claude is on that work. On a step, that lasts while the step is `RUNNING`. On a task with no checklist, it lasts while the task's review line is `RUNNING`. On a checklist task launched from its description, it lasts while a step runs, or while the task's terminal is live and a step is still open. **Working** takes no press. The button returns to **Run here** once the work is claimed or the session ends.

Live work sits in one bar in the bottom right corner of every screen: a segment for running timers (with the newest timer's clock) and a segment for live terminals, each shown only while it has something to count. A segment opens its sheet above the bar, one sheet at a time; pressing the other segment switches, and pressing the same one again, Escape, or a click anywhere else closes it. From the sheet a row jumps to its task (the terminal one also opens the terminal pane), stops the timer or terminal, and, for a terminal, logs the latest commit against the task and step it was opened on. The bar reports the room it takes so nothing else in that corner sits under it, whichever segments are present; the terminal segment steps aside while the terminal pane is on screen, and the bar leaves entirely when nothing is live.

**Settings > Claude Code** picks the model and effort a new terminal starts with, and the **skip permissions** switch:

- **Model** (`Account default`, `Sonnet`, `Fable`, `Opus`, `Haiku`): anything but the default adds `--model <alias>`. Each alias is Claude Code's own name for the latest model of that family, so no version is pinned.
- **Reasoning effort** (`Account default`, `Low`, `Medium`, `High`, `Extra-high`, `Max`): anything but the default adds `--effort <level>`.
- **Skip permissions** adds `--dangerously-skip-permissions`; with it off, an interactive permission prompt in the terminal is yours to answer.

All three are stored on the machine, not in the database.

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
| `/rk project:vega task:report-builder generate "how checkout charges"` | Read the code and store a diagram answering the request, then stop (see [Diagrams](#diagrams)) |
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
| `rekall_note` | write | Write a note on one task, or rewrite one it carries by title with `replace` |
| `rekall_diagram` | write | Store a Semantic Graph generated for one task, or replace one by id with `diagram` |

There is no query, get or schema tool. `rekall_note` adds a note, or with `replace: true` rewrites the body of the note the task already carries under that title (ignoring case); without `replace` that title is refused, and nothing it does detaches or deletes a note. `rekall_step` refuses `done`; that state is set by hand in the console. `rekall_step` (on `claimed`) and `rekall_wrapup` take an optional `commit_message`, used only on a project that commits on claim (see [Commit on claim](#commit-on-claim)).

## Wrapup

A task has at most one wrapup: what its implementation currently is, not a changelog. A session writes it fresh every time from the code, keeping only what is still true and needed to start work, since the history keeps every earlier version. The kind of information picks the form: an opening sentence on what the task delivers, a table for the pieces of the implementation, a bullet per rule, a table for results of checks, a numbered list for a flow, open points last. It names code by the code's own names (class, file, endpoint, table) and does not transcribe field lists or signatures. It aims at about 300 words; past 600 the tool's answer asks for a shorter rewrite, though the write still lands. It is capped at 20,000 characters, against 100,000 for a note.

A quoted term after `wrapup` is a directive on what to write. It can narrow the subject, dictate the wording, or set the language or length.

```
/rk project:vega task:report-builder wrapup "export module only"
```

The description pane and the steps pane both carry a **Generate the wrapup every session** toggle under the header, so it is set from whichever surface the work is driven from. Turning it on reveals an optional directive field. Set once on the task, the toggle stands in for the quoted term: the context load then tells the session a wrapup is expected every time without being asked, and the words it should follow. Turning the toggle off drops the directive with it.

A wrapup written after a step finishes folds that step's work into the same description. The console counts steps ticked and notes added since the wrapup was last written. You can edit the wrapup in the console; the next `/rk … wrapup` replaces it and the tool reports when it overwrites a hand edit.

### History

**History**, on the wrapup and on the description, lists the earlier versions of that text, newest first, and restores one. The revision service keeps what a write replaces: every session write, every delete and every restore, so a restore can be undone the same way. The console autosaves as you type, so a hand edit over hand-written text keeps one version per ten minutes, the one from before you started, not every keystroke; a hand edit over a session's text is always kept. The newest 30 versions of each text are kept per task. Sessions never read the history. `GET /api/tasks/{id}/revisions?kind=WRAPUP|DESCRIPTION` lists it and `POST /api/tasks/{id}/revisions/{revisionId}/restore` restores one.

### Notes written by a session

A session can keep what it produced as a note on the task, rather than in the wrapup. Ask for it with a quoted term after `note`, saying what the note has to hold:

```
/rk project:vega task:report-builder note "the export formats we settled on"
```

The session writes the note and calls `rekall_note`, which creates it on that task, in full, with kind `notes`. It then travels with the task's context and shows up in the console without a reload. A step or a description that asks for its output to be kept as a note, or a task whose whole point is to write one, gets the same without the `note` term. A title the task's notes already carry is refused, unless the session passes `replace`: then that note's body is rewritten in place, its title and placements kept, so a runbook or a table of results stays one current note instead of piling up versions. The old body is not kept anywhere, and a note on several tasks changes on all of them; the tool's answer says how many. Sharing, detaching or deleting a note stays in the console. Writing a note does not claim the task or a step.

## Steps

A step sits on a line: **draft** while you are still wording it, **open** once you promote it and it is ready to work, **running** while a session works it, **claimed** when the session reports it finished, **done** when you accept it. Each step is a title and an optional markdown detail. The pane opens on the first step whose work is not finished.

A new step is created as a draft and sits on a staging shelf below the checklist, off the rail. **Promote** moves it onto the list; an open step not yet started can go back with **To draft**. A draft is not work: it stays out of the checklist a session reads, `rekall_step` will not move it, and the navigator's step count ignores it. A task that holds only drafts is still treated as having no checklist.

A session drives its own checklist over `/rk`:

```
/rk project:vega task:report-builder step:3 start   # step 3 -> running
/rk project:vega task:report-builder step:3 done    # step 3 -> claimed
```

The console holds one `text/event-stream` connection (`GET /api/steps/stream`) carrying six frames: `steps` for a checklist, `task-review` for a stepless task's review line, `wrapup` for a wrapup write or delete, `commit-reference` for a commit logged against a task or step, `run-queue` for the run queue, and `note` for a note a session wrote or rewrote. A step moved from an in-app terminal, a box ticked in another window, a wrapup or a note written over MCP, or a commit logged by a session all land without a reload.

Claude receives an open or running step with its detail, tagged `(in progress)` or `(claimed, …)`. A finished step arrives as its title alone. A draft step is not sent at all, only counted as `draft="N"` on the `<steps>` tag. A step ticked without a following wrapup is marked `(finished since the wrapup was written)` and handed back with its detail until the next wrapup folds it in.

With steps on a task, the open steps are the work and the description becomes the constraints the steps are built against. Anything the description asks for that no open step covers is not built; `/rk` flags it and asks for the step.

Only you set a step to **done**. `rekall_step` stops at `claimed`. The navigator's progress count is built on `done`.

The mark in front of each navigator row says where an in-progress task's work stands. A hollow amber ring around a small light means nothing is handed in yet, and a green arc on that ring is the share of the checklist already accepted. An orbiting comet means a session or a timer is on the task now; it is the only mark that moves. When the timer runs, no session is at work and a claim is waiting for you, the comet keeps orbiting but an amber check replaces the light at its centre: something is running, and it needs your review to go on. A session still working a step keeps the plain comet, whatever it already claimed. An amber check in an open ring means work is claimed and waiting for your review. A filled green seal means every piece of work is accepted and the task is ready to move to Done. Tasks in any other status keep a plain dot in their status colour. Hover the mark to see its meaning in words. The selected row scrolls to the middle of the list when it is out of sight, and a gold comet circles its edge, brightest at the head and fading along its tail, so it is easy to find; with reduced motion set the comet is hidden.

### Planning

`/rk project:vega task:report-builder plan` has a session turn the task into a checklist for you to review. It reads the description, the wrapup, the finished steps and the code the task touches, then calls `rekall_propose_step` once per step, in order: a title saying what the step delivers, a detail saying what to build, where, what it must satisfy and how you can tell it is done. Every proposal lands as a draft on the staging shelf, so nothing is work until you promote it, and the session builds nothing. A title the task already has is refused, so a second `plan` adds only what the first one missed; a task holds at most 20 drafts.

The Steps pane starts it from **Plan here** in its header, with or without a session already running. While one runs on the task in the console's terminal, the button types the command into it. Without one, it opens a terminal in the project's folder that starts on `/rk … plan` instead of the plain `/rk`, so the session plans the task rather than working it; the project needs its folder set, as for **Run here**. Either way the drafts land on the shelf for you to reword, promote and then run. An empty checklist also shows the `/rk … plan` chip, to copy into a session outside the console.

A claimed step is reviewed from its detail: **Accept** ticks it to done, **Send back** returns it to open for another pass. Sending back seals the detail that pass worked from above the editor (`POST /api/steps/{id}/send-back`), and the detail starts empty for the feedback. `/rk` hands the next session each earlier pass in a `<previous-pass>` and the feedback in a `<feedback>`, so it builds on what was done instead of reading the feedback as a new brief. The **N awaiting review** count in the pane header jumps to the first one. The step node itself only moves a step forward, so a stray click never walks it back; reopening an accepted step is a separate **Reopen** button that arms before it fires.

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

The message says what the work does, never which files it touched. The session that claims writes it, as `commit_message` on `rekall_step` or `rekall_wrapup`: a Conventional Commits subject under 72 characters, a blank line, and a few sentences of body. Without one, Rekall derives it: the subject is the step or task title, typed `fix:` or `refactor:` when the title says so, `docs:` when every file is documentation, `test:` when every file is a test and `feat:` otherwise; the body is the opening of the step's detail (or of the wrapup, for a task with no checklist) as plain prose, headings, emphasis and code dropped, cut at a sentence. Either way the body is wrapped at 72 columns and ends with a `Refs: project:… task:…, step N` line. A clean tree commits nothing and says so; a git refusal (no identity, a hook, a folder that stopped being a repository) is reported in the tool's answer and the claim stands either way. `rekall_context` marks such a project with an `auto-commit` field telling the session not to `git commit` itself.

## Description review

A task with no checklist walks the same line at task scope: **open**, **running** while a terminal is open on it with no step target, **claimed** when a Claude-authored wrapup lands, **accepted** when you accept it in the console. Nothing new is typed for it: running follows the terminal and claimed follows the wrapup write. The description pane shows the running pill and a review bar: on **running** it carries **Accept** and a link to the wrapup, so a session driven by hand still has a console exit; on **claimed** it adds **Send back** (with an optional note the next session sees); accepting offers to also mark the task done. A note left on send back rides in `rekall_context` as a `review` field on the task (`sent back — "…"`), so the next session opened on that anchor reads why, not just that it was. It arrives on the same `GET /api/steps/stream` connection as a `task-review` frame, the wrapup that claims it rides the same connection as a `wrapup` frame so the pane shows the new text with the claim rather than on the next reload, and `PATCH /api/tasks/{id}/review` is the console-only Accept / Send back. Adding a first step retires the task-level line and the checklist takes over.

## Review queue and notifications

**Review** (the checklist icon with a count, in the top bar) counts everything a session claimed and you have not reviewed, across every task: claimed steps, and tasks with no checklist whose wrapup claimed them. It lists them longest-waiting first, and a row opens the step on its steps pane, or the task on its description's review bar. It is built in the console from the same step and review frames the event stream already carries, so it moves as sessions claim and you accept, and it is not there while nothing waits.

When something joins the queue while the console is hidden or another app has the focus, Rekall posts a system notification: through the native bridge in the desktop app (`rekall-app/desktop/src/bridges/notifier.rs`, the system asks for permission the first time), through the browser's Notification API elsewhere. What was already waiting when the console loaded never notifies. **Settings > Notifications** turns it off; the choice is stored on the machine, and in a browser that is also where permission is asked, since a browser grants it only from a click.

## Console

One surface, three panes: pick a task on the left, pick its checklist, its wrapup, a note or a session in the middle, write on the right. The field at the top takes the same grammar as `/rk`.

That field filters the navigator by titles and labels as you type. For a phrase of three characters or more that is not an anchor, it also searches the text behind them and lists the hits under the bar: task descriptions, steps (title and detail), wrapups and notes, each with the words around the match. `↑` `↓` choose, `↵` opens the hit on the pane that shows that text (a step opens expanded on its steps pane). `GET /api/search?q=` answers it: at most 8 hits per kind, newest first, the phrase matched whole and case-insensitively, `%` and `_` literal.

The description, steps, wrapup and terminal are pinned above the notes. Each opens in the writing pane; a task missing one shows an empty card. `c` opens the terminal pane, the same way `s`, `w` and `d` open steps, wrapup and description; `q` opens the run queue; `r` toggles read/write on whichever of the description or a note is open. Companies, projects and tasks are created, edited and deleted from one editor, opened on the parent record. Title and label sit together with the anchor assembled live as you type. A task's project picker shows the chosen project's status (Active, Paused, Done) at the right of the field, and each project in the list carries it too. Deleting states what goes with it.

A task optionally wears one **tag**: a name, a glowing icon and a glow colour, both picked from a fixed set the console already knows how to draw. Tags are configured in their own panel, opened from the star button in the header next to Settings — add, rename, re-colour or delete one there, with a live count of the tasks currently wearing it. A tag is assigned to a task from that task's edit dialog, and shows as a small glowing badge next to the title wherever the task is listed.

Finished tasks are folded into a "filed" drawer, and **Backlog** tasks into a backlog drawer of their own, both closed on every load. Creating a task in the backlog does not start its timer. A new task is created without a description and opens on its description pane, on a template of Goal, Requirements, Out of scope and Constraints whose hints are HTML comments that do not render. Writing autosaves; a note has no Save button.

The timer follows the work. Writing on a task whose timer is paused starts it: its description, its wrapup, a step's title or detail, a new step, a note, or an edit from the task's dialog. A note on several tasks starts the timer of the task in view, or of its only task. Ticking, promoting or reordering steps does not count as writing. Accepting a claim stops it, whether that is a claimed step ticked done or a claimed stepless task accepted. Ticking an open step by hand leaves it running. Opening a note, or a save that changes nothing, is not writing. Only the console starts a timer this way; a session's writes over MCP never do.

A timer stops by itself when its task has seen no write for 30 minutes: the task, its steps, wrapup or notes. It stops at the time of that last write, so the idle stretch is not counted. A task with a step a session is running is never idle, and every open timer stops when the application shuts down. The check runs every minute.

Every markdown editor puts a path in the text without a trip to Finder. `⌘⇧P`, or the folder button in the toolbar next to link, opens a file browser under the caret, in the project's folder (for a note, the folder of the task it was opened from), or wherever it was left the last time in that editor. `↑` `↓` move, `→` or `Tab` opens a folder, `⌫` or `←` on an empty filter goes up, and the breadcrumb and the project and home buttons jump. The filter also takes a path: `src/comp` lists `src` and keeps what matches `comp`, `~/` goes home, `/` goes to the root. `↵` inserts the highlighted entry, `⌘↵` the folder in view, and the footer shows exactly what will land: the absolute path, a folder ending in `/`, wrapped in backticks so `_` and `*` survive the preview, bare when the caret is already inside code. Dotfiles stay out until `⌘⇧.`, or until the filter starts with a dot. The first `esc` clears the filter and the second closes. `GET /api/filesystem/directory?base=&path=` lists one folder by name only, never its contents: `~`, absolute and relative paths are resolved on the server, a folder macOS keeps private comes back empty and marked unreadable, and a folder bigger than 2,000 entries is cut off after sorting.

A note lives in a **scope**: one project, one company, or **Global**. The scope owns it: the Notes list groups notes by it, a note can only be put on tasks inside it, and a note taken off its last task stays there. Sessions still read a note only through the tasks it is on. **Lives in** on the note's toolbar moves it; a scope that would leave out one of its tasks is disabled. A note written by a session with `rekall_note` lives in that task's project.

A note is put on a task from either side. From the note, the task chips on its pane open a picker that walks company, project and task, limited to the note's scope. From the task, the **Notes** button in the description and steps headers drops a list of the notes whose scope admits the task, the ones on this task first: one click, or `↑` `↓` and `↵`, adds a note or takes it off, without leaving the pane. A tick never moves a row while the list is open.

The note cards in the task's column take a note off the task too. The right end of a card says where else the note lives; under the pointer, or once the card has focus, it becomes a `×` that takes the note off this task without asking, and `⌫` on a focused card does the same. The toast that follows offers **Undo** for a few seconds, which puts the note back and reopens it if it was the one in the editor. **+ New note** under the last card writes a note in the task's project, already on the task. **Add existing note** just below it opens a picker of the notes this task's scope admits and does not carry yet, newest first, each with an excerpt and where it lives. Typing filters, `↑↓` moves, a click or `↵` puts the note on the task and opens it, `esc` closes. The button is disabled when nothing is left to add.

`b` switches the left column between tasks and notes. Browsing notes, picking one leaves the task in view alone: the middle column becomes the note's placements, the tasks it is on grouped by project. **Put it on a task** at the top opens the same company → project → task picker the note's "+ task" chip opens on the tasks side. Each row carries a `×` at its right end that takes the note off that task, and under the pointer it colours the row it is about to empty and says **Take off**; the row itself does nothing on a click. The arrow on a row is the only thing that opens that task on the tasks side. Switching back to tasks brings the task up to date with the note.

A note starts from either side too. Browsing tasks, **New note** (or `n`) writes one on the task in view. Browsing notes, the same button, `n`, or **+ New note** at the top of the list turns the middle column into a composer: a name, where it lives (starting in the project of the task in view), and optionally the tasks of that scope to put it on. Typing finds any task in the scope, `↵` ticks it, `⌘↵` or **Create note** makes the note and opens it in the editor; `esc` walks away. **Delete** on the editor header removes the note from every task it is on.

## Report

**Report** shows what went to which client and for how long. The frame is a week or a month, stepped with the arrows either side. One column per day is stacked in each company's colour against a dashed line at eight hours. Below it, a section per company, its projects and its tasks, with the hours and the days each ran on.

Every task row opens on the steps it closed inside the period, oldest first, with the day each was ticked and a count of those still open. A step ticked outside the period is counted but not named. The chips narrow the report to the companies you pick. **Copy as markdown** puts the whole report on the clipboard with every task's anchor and its closed steps.

The screen is built from the sessions the timer recorded. A session counts on the day it started; one still running counts up to now.

## Diagrams

**Diagrams** draws what the code does rather than how it is laid out. A diagram is a Semantic Graph: concepts, actions, decisions, states, events, data, external systems and code, joined by typed relations (`leads_to`, `conditionally_leads_to`, `calls`, `reads`, `writes`, `contains`, …). An element can stand for something no function names on its own, such as one of the steps a 200-line function performs, and each one points at the lines that implement it. A session writes the graph and the console draws it. The drawing is decided by code, never by the session. The format and its rules are in [docs/SEMANTIC-GRAPH.md](docs/SEMANTIC-GRAPH.md).

The screen has three columns. On the left is the library, grouped by project. In the middle is the canvas, laid out by dagre. On the right is the inspector.

- **Canvas.** The shape of a node is its kind and its colour repeats it. The ring on its corner is the trace mark: the outer ring is the provenance (solid when observed in the code, dashed when inferred, dotted when it comes from docs, doubled when a person stated it), the arc is the confidence, and a filled core means the element points at code. Flow edges are solid, conditions are dashed and carry their condition, data flow is dotted, and dependencies are faint. Selecting a node lights everything one relation away and dims the rest.
- **Lens.** **Concept** hides the code nodes, so the conceptual pieces of a function stand on their own. **Code** draws each function as a frame around the pieces it implements. A frame folds into one node that takes over its relations, and unfolds again.
- **Direction.** A diagram opens left to right or top to bottom, whichever fits the canvas at the larger scale. The button next to the lens overrides it.
- **CONCEPT → CODE.** The inspector lists a node's sources and opens each one in place, showing the span with a few lines around it, read from the project folder.
- **CODE → CONCEPT.** With nothing selected, the inspector lists the files the diagram points at. Picking one lights the elements it implements. `GET /api/projects/{id}/diagram-trace?file=&line=` answers the same question across every diagram of a project.
- **Keys.** `f` fit, `+` `-` `0` zoom, `/` find, `l` lens, `d` direction, `esc` clear. The wheel pans; pinch or `⌘`-wheel zooms.

The library on the left lists tasks, grouped by project, not diagrams. A task carries a tag for its diagrams (**Generating** with a moving ring while a session draws one, **Diagram** with a count once one exists, nothing otherwise). Picking a task opens its latest diagram, picking a generating one goes to its terminal, and picking one with no diagram asks for one: the dialog only asks what to show. **All tasks / With a diagram** narrows the list, and diagrams tied to no task (imports) sit under *Not tied to a task*.

**Generate** opens a terminal on the task you pick with `/rk project:<p> task:<t> generate "<request>"` as its first line (`mode: GENERATE` on `POST /api/tasks/{id}/terminals`). The task is marked **Generating** until a diagram for that task arrives on the event stream (`diagram` frames); a generation survives a reload within the tab and stops counting once its terminal is known to have ended. The session finds and reads the code on its own, from the project's folder, and never asks where it is or what to read. It describes the behaviour the request is about, splitting monolithic functions into the pieces they perform, and calls `rekall_diagram`. That tool carries the whole of [docs/SEMANTIC-GRAPH.md](docs/SEMANTIC-GRAPH.md) in its description. It stores the diagram on the task's project with the task as its origin, but only after the graph passes every rule and every source span is held against the project folder: the file has to exist and the lines have to be inside it. A refusal lists each broken rule with its path, and nothing is stored until the graph passes. `diagram` replaces an existing diagram in place. **Import** stores a graph written elsewhere, checked against the same rules but not against the folder.

| Route | Effect |
|---|---|
| `GET /api/diagrams` | Every diagram, without its graph, newest first |
| `GET /api/diagrams/{id}` | One diagram with its graph |
| `POST /api/diagrams` | Create one: `projectId`, `taskId` (optional), `title`, `question`, `graph` |
| `PUT /api/diagrams/{id}` | Replace its title, question and graph. It stays on its project |
| `DELETE /api/diagrams/{id}` | Delete it. Console only |
| `GET /api/diagrams/{id}/source?file=&start=&end=` | The lines of a span in the project folder. A path that leaves the folder is refused |
| `GET /api/projects/{id}/diagram-trace?file=&line=` | Every element, per diagram, covering that line, narrowest span first |

## Model

```
Company ──< Project ──< Task >──< Document
               │         │       via document_task
               │         ├──< TaskStep
               │         ├──1 Wrapup
               │         └──> Tag (optional)
               └──< Diagram ──> Task (the one it was generated from, optional)
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
| `Diagram` | id | title, the question it answers, the Semantic Graph as JSON, node and edge counts. Goes with its project; loses its task when the task is deleted |

`label` is what an anchor resolves: lowercase letters, digits, `-`, `_`, `.`, no spaces, normalised on write. `title` is free text and changing it never breaks an anchor. Renaming a label moves the anchor, and the editor says so before saving.

A project belongs to one company, a task to one project. A note belongs to a scope (a project, a company, or none for global) and sits on any number of that scope's tasks. Deleting a task unlinks its notes; deleting a project or company deletes the notes it owns. A wrapup and a step belong to exactly one task and are deleted with it. Adding an entity is a SeaORM model in `rekall-model` plus a migration in `rekall-repository`, not a UI action.

## Context size and reference notes

Next to the anchor on a task's description, a chip estimates what `/rk project:… task:…` costs a session: the characters of the markdown it hands over and roughly how many tokens that is (3.5 characters to a token, an estimate for comparing tasks, not a bill). Under the pointer it lists the parts heaviest first: the description, the steps, the wrapup, the commits chosen for the context, each note, and the project around them. `GET /api/tasks/{id}/context-size` measures it with the same context renderer that answers `rekall_context`, so the figure is the length of what a session gets.

A note most sessions do not need can go **By reference**, switched on the note's own pane. It then travels as its title, its first line and an anchor such as `note:3f2a9c1e` (`loaded="on request"`), and a session loads the rest with `rekall_context` and that anchor only when the work asks for it. **In full** puts it back. The mode belongs to the note, so it holds on every task the note is on.

## Backups

SQLite's `VACUUM INTO` copies the open database, consistently and without stopping it, into a zip in a `backups` folder beside it. One is taken when Rekall starts and whenever the newest is older than the interval, checked every hour; **Back up now** in **Settings > Backups** takes one on demand, and every restore takes one first. Only the newest are kept.

| Property | Default | Meaning |
|---|---|---|
| `rekall.backup.enabled` | `true` | Scheduled backups. **Back up now** and restores work either way |
| `rekall.backup.interval-hours` | `24` | Age of the newest backup that makes another one due |
| `rekall.backup.keep` | `10` | Backups kept, of every kind together; the oldest go first |

**Restore** on a listed backup, or **Restore from a file…** with a zip from elsewhere, replaces the whole database: the zip has to hold a Rekall SQLite database (`rekall.db`, checked by its header), what is there now is backed up first, and Rekall restarts on the restored file, migrating it forward if it came from an older version. That round trip is also how a database moves to another machine: **Download** a backup here, restore it there. An in-memory database has no backups. The REST side is `GET` and `POST /api/backups`, `GET /api/backups/{name}`, `POST /api/backups/{name}/restore` and `POST /api/backups/restore` (multipart `file`, up to 1 GB).

## Export

```bash
curl -OJ http://localhost:47355/api/export
```

Or the **Export** button in the top bar. The archive is a folder tree, one folder per company, then project, then task, one markdown file per note, plus a `MANIFEST.md` with statuses and anchors. It is for reading, and nothing reads it back: to save and restore, or to move a database, use a [backup](#backups). A note on several tasks appears under each; `MANIFEST.md` lists the copies.

## Develop

```bash
make ui-dev   # Vite dev server on :5173, proxying /api and /mcp to :47355
make test     # every crate's tests, then frontend lint, types and unit tests
make lint     # clippy, as CI runs it
```

The `/rk` command the app installs is `.claude/commands/rk.md`, compiled into the binary.

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

### Debugging

`RUST_LOG=rekall=debug make run` turns on debug logging. The server is a plain binary, so any Rust debugger attaches to it (`rust-lldb target/debug/rekall-server`, or the CodeLLDB extension in VS Code).

## Configuration

Properties are passed as `--name=value` arguments or as environment variables in relaxed form (`rekall.terminal.max-sessions` is `REKALL_TERMINAL_MAXSESSIONS`). All optional.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_PORT` | `47355` | HTTP port for the UI, the API and MCP |
| `REKALL_HOME` | `~/.rekall` | Where `config.json` records the database folder |
| `REKALL_DB_URL` | the folder in `config.json` | `sqlite:<path>` to a database file, overriding the setup wizard's choice |

Notes are stored in plain text in the database file. Credentials kept in them are only as protected as the disk is.

### A database from the earlier Java version

The first versions of Rekall ran on the JVM with an H2 file, `rekall.mv.db`. A folder that still holds only that file is imported into `rekall.db` beside it the first time it is opened, and the H2 file is never changed. By hand:

```bash
make import-h2 DIR=~/rekall-data     # or: rekall-server --migrate-from-h2 ~/rekall-data
```

Reading an H2 file needs H2's own jar, so the import needs a Java runtime and `h2-2.x.jar` (`--h2-jar`, `REKALL_H2_JAR`, or `~/.m2`). Run it on the machine and in the time zone the old version ran in: H2 kept timestamps as local time. Nothing else in Rekall needs Java.

## Modules

A Cargo workspace, one crate per layer, each depending only on the ones above it:

```
rekall-common/     RekallError, Id, Instant and the string helpers every layer shares
rekall-model/      SeaORM entities and their state rules
rekall-diagram/    the Semantic Graph: its types, JSON format, validation and CODE → CONCEPT index. No database, no UI
rekall-repository/ queries, one function per lookup, and the SQLite migrations
rekall-service/    business logic: context assembly, the step and review lines, wrapups, notes, time entries
rekall-api/        Axum REST API for the UI, the folder listing and the step event stream
rekall-mcp/        MCP server, on rekall-service and never rekall-api: one tool reads, five write
rekall-claude/     the in-app terminal (portable-pty over one WebSocket), the Claude Code usage meter and the run queue
rekall-app/        rekall-server: settings, backups, the local-access guard, serves everything
rekall-app/desktop the Tauri desktop app over the same server
rekall-ui/         Vue 3 + Vite frontend
```

Inside a crate the layout follows the Java one: a folder per concept or layer (`controller/`, `service/`, `dto/`, `note/`, `step/`), one public type per file named after it (`NoteService` in `note_service.rs`), and a `mod.rs` that only declares the files and re-exports their types. The rules and their exceptions are in [docs/DESIGN.md](docs/DESIGN.md#layout-inside-a-crate).

The crate boundary is what keeps the MCP server off the write paths: `CatalogService`, `DocumentService` and `RevisionRestoreService` live in `rekall-api` and not in `rekall-service` because they write the catalog, and `rekall-mcp` cannot reach them. Read-only services (search, context size) and the narrow writes a session is allowed are in `rekall-service`.

## Run tests

```bash
cargo test                                            # every crate but the desktop app
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

Each crate keeps its tests under `tests/`: `tests/unit/` mirrors `src/` (`rekall-service/src/search/search_service.rs` is tested by `rekall-service/tests/unit/search/search_service_tests.rs`, attached with `#[cfg(test)] #[path = "…"] mod tests;` so it can test private functions) and `tests/integration/` holds the integration suites. A new integration suite has to be declared as a `[[test]]` in the crate's `Cargo.toml`, or it never runs. The end-to-end suites in `rekall-app/tests/integration` start the whole application on a real port with a file database and drive the HTTP API, the MCP endpoint and the event stream; a stub stands in for the `claude` TUI in the terminal and run-queue tests.

`.github/workflows/check.yml` runs all of it on every pull request and every push to a branch other than `main`: clippy and `cargo test`, eslint, vue-tsc, vitest, a production build of the UI, and a build of the desktop app. `release.yml` runs only on a `v*` tag on `main`, and builds and publishes that release.

## Design

`docs/DESIGN.md` records the decisions and the reasoning, including the ones that were reversed and why.

`docs/MEMORY.md` is the memory soak of the console: what was measured, how, and why the numbers say there is no leak.

`docs/SPECIFICATION.md` describes what the application is and does, entities and rules only, with no visual direction.

## License

Source-available, not open source. Licensed under the
[PolyForm Internal Use License 1.0.0](LICENSE): you may read the source, run the
software for your own and your company's internal business operations, and change it
for those purposes. You may not distribute it, in original or modified form.

Copyright 2026 Valerio Mario Casale.
