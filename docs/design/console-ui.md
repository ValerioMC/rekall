# Console design

## The report

The weekly and monthly report is a regrouping of the sessions the timer writes, and it is built
in the browser rather than behind an endpoint of its own. Everything it needs is already in the
window: the sessions, the tasks and the companies are loaded whole at startup, so a report is a
question about data that has already arrived rather than a new one to ask the server. The
grouping is a pure function over them, `common/report/time-report.ts`, which is what makes the
arithmetic testable without a database.

A session counts on the local day it started on, which is the rule the calendar already uses, and
one still open counts up to now. Both are the readings a person checking a total against their
own memory of the week would make.

The company filter holds ids, and an empty selection means every company rather than none. The
chips are built from the unfiltered period, so narrowing to one client never removes the way back
to the others.

## The navigator's filing drawer

A finished task does not leave the navigator, and after a while that is most of the list: a
project with thirty closed tasks buries the three still open. So `DONE` tasks are folded into a
drawer, one per project in the company-wide view and one below the status groups when a single
project is in scope. The drawer is shut on every load and reopens only for the session: the
point is that finished work starts out of the way, not that the window remembers you looked at
it once. This is the opposite of the project collapse next to it, which persists in
`localStorage`, because that one is a layout preference and this one is a default.

The split is `common/catalog/partition-tasks.ts`, a pure function, so the rule that a `DONE`
task is filed lives in one tested place rather than in the template. `console.store` is
untouched: `visibleTasks` stays the one honest list, the tasks/notes counter still counts all
of it, and `J`/`K` still walks it. A filed task that gets selected, by search or by walking,
opens the drawer it sits in, because a selection you cannot see is one you cannot tell you made.

## The navigator row

A row is two lines with air around them rather than one crushed line. The status pip is a
coloured core inside a soft halo of the same hue, so a state carries its own weight instead of
being a hard dot shoved against the title; while a timer runs the pip becomes a sweeping ping,
because "worked right now" should read before the colour does. Progress, a wrapup mark and the
note count sit in a steady right-hand column and do not move: a number that vanishes the moment
you reach for it is a number you stop trusting, so edit rides in as its own chip on hover
instead of displacing them.

A project heading is also two lines: the title and task count on top, the company name and the
project's token total below, so none of the three is fighting the others for the same 12 pixels. The
navigator is given the width for this rather than made to crush it.

## The selected surface

A selected row sits on the raised surface, warmed from its left by its tint (the project's
identity hue in the navigators, the accent elsewhere), held by a tinted hairline, and marked by a
short solid capsule in the left gutter, clear of the top and bottom edges. Its relief is
`--shadow-key`, the same as a key, a tag chip and a project header in the navigator, so
selection reads by colour and outline, never by standing taller than its neighbours. A hovered
row only takes the raised surface's colour. The status pip of a selected row gains a thin
accent ring. `selected-row` and `selected-card` in `main.css` are the same idea at two scales.

A tag badge (`tag-chip`) is the tag picker's quiet key: a neutral face warmed a few percent by the
tag's colour, a tinted hairline, and the colour itself carried by the glowing icon.

## Labels

The kicker above a title, "Description", "Time", "Tracked this week", a form-section label in a
dialog, a company name in the scope popover, was tracked-out capitals everywhere. It is now
ordinary case in the shared `eyebrow` utility, a hair of tracking and nothing more, so it names
its section without shouting over the content beneath it. In the three writing panes it is
preceded by a 3px accent tick, the same mark on all of them, which is how you tell at a glance
which of the four surfaces is on screen. Raw status enums that used to be printed as-is
(`PAUSED`) now go through their label map (`Paused`) wherever they are shown.

The filed tone is a muted verdigris, `--color-filed`. It is deliberately neither the green of a
live `DONE` status nor the grey of disabled chrome: a shelf of finished tasks should read as an
archive, not as a list that has gone dim.

## Tokens, not values

Every size, corner, grey and tempo a component uses is a token in `main.css`, because values
picked one component at a time drift: the console had reached twenty-two font sizes, half of them
half a pixel apart.

| Token | Steps | Rule |
|---|---|---|
| Type, `text-micro` … `text-hero` | 10.5, 11.5, 12.5, 13.5, 15, 19, 26, 34 px | Micro is the floor and never prose. No line heights: text inherits its leading |
| Corners, `rounded-key` / `-control` / `-card` | 3, 5, 8 px | A chip or kbd, a button, field or row, a card, dialog or header capsule. Pills are `full` |
| Relief, `--shadow-key` / `--shadow-raised` | lip + contact; + a short cast | A key gets a faint lit lip and a contact shadow, nothing cast; a raised card adds a short soft one. Depth is a hint, never an extrusion |
| Tempo, `--motion-press` … `--motion-slow` | 90, 140, 220, 320 ms | Press, hover, a pane moving, a seal. Choreographed keyframes keep their own |
| `--color-text-subtle` | `#8c96a6` | 4.5:1 up to `surface-hover`, so small hints stay readable. Never faded further with an opacity modifier |
| `--color-border-field` | `#5e6a82` | The edge of something you type into, 3:1 against the field's well; hairline separators keep `border` |

`tests/unit/design-tokens.spec.ts` fails on a bracketed font size or radius, a faded subtle grey,
a pixel `font-size` in a style block or a numeric `duration-*`. `TerminalPane.vue` is exempt: it
frames a real terminal and is left as it is.

Two components carry what every screen repeats. `AppDialog` is the one modal shell (backdrop,
`role`, `aria-modal`, Escape, focus trap and the modal gate); `common/a11y/dialog-stack.ts` makes
only the dialog on top answer the keyboard, so a confirm over a panel closes alone, and a dialog
that is not dismissible lets Escape through to the control inside it. `AppButton` adds `quiet`,
an `icon` size and `pressed`, which lights a toggle as `key-lit` and sets `aria-pressed`, so
toolbar keys stop being hand-built from the keycap utilities. A control drawn smaller than 24px
takes `hit-target`, which grows the pointer target, not the glyph.

The two fixed panes are tokens too, `--spacing-nav` and `--spacing-notelist`. Below a 1280px
window both narrow to 272px, so at the desktop shell's 960px minimum the note still keeps a
column of about 420px.

The header is one 48px row (`--spacing-header`, shared by the pane headers so their rules line
up), a four-cell grid: the 24px mark with the history arrows, the screen tabs centred in a flexible cell, a centre cell of at most 620px and a flexible right cell. A screen teleports exactly two children into the last two, so the tabs sit in the same place on every screen; the console puts the `/rk` field in the centre and the readings and tools on the right, the others a title and actions. A
screen tab is text with an amber rule under the screen you are on (`nav-tab`, `nav-tab-current`), and a
tab whose screen counts something that needs the reader wears it (`NavAlertCount`, from `useNavAlerts`).
Under a 1280px window the tabs fold into one key (`AppNavMenu`) naming the screen on display, which opens
every screen as a vertical menu, a group's screens under its name; the sum of the tabs' alerts rides on
the key, so folding never hides a warning.
On the console the `/rk` field is also the location bar: at rest it shows the open note's or
task's anchors, or the scope's, as chips, and focused it is the search. Everything right of
the tabs is the same piece, a 36px `header-capsule` with the field's corner: usage, the run queue
and the review count share one (`HeaderStatus`, segments drawn with `status-segment`), where
review is the only amber segment because it is the only one asking for action; the save dot,
Tags and Settings (`header-key`) and New note share the other. The save state is
a dot whose word is for screen readers and the tooltip. The bar gives way under 1280px: the
anchors keep only the last one, the review segment drops its words and New note becomes a `+`;
under 1100px the `⌘K` hint goes.

Browsing notes, the middle column (`NotePlacementsPane`) is one card per project the note reaches
(`NotePlacementCard`), lit with the project's hue, each task in it with its status, anchor, step
progress and two labelled keys (`NotePlacedTaskRow`): **Open task**, and **Detach**, which acts
at once and offers Undo in its toast because the note stays in its scope. A note on no task gets
an empty state with **Put it on a task**.

The calendar (`CalendarPage`, `CalendarDayCell`) heads each day with its total and a load bar:
its length is the day against an eight-hour day, its segments the projects largest first
(`common/calendar/day-shares.ts`). An eighth column totals each week; today wears an amber ring.

The console with nothing open is a desk (`ConsoleHome`, its parts under `components/console/home/`):
a headline that counts the live sessions and what waits for review (`situationOf`), the week's
tracked time as seven bars (`trackedLastDays`), a review card and a running-sessions card, the
four notes touched last as tiles, and the keys. It lays out on a container query, so the same
markup is one column in the 420px pane at the window's floor and two to four columns when wide.

