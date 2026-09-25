# Memory behaviour of the console

What the Rekall frontend does to memory over time, measured on 2026-09-16 against the build at
that date, and how to measure it again. The short version: **there is no memory leak**. Every
surface of the console was driven for hundreds of cycles and the JS heap, the DOM node count and
the listener count all plateau within the first minute and stay there. The process that Activity
Monitor shows under Rekall as `http://127.0.0.1:47355` is the WKWebView content process, and it
settles around 0.5 GB under continuous heavy use, with no trend that would carry it to the 1.5 GB
once seen (that reading predates the CodeMirror fix described below).

## What was wrong before

`md-editor-v3` creates a CodeMirror `EditorView` on mount and never destroys it, and the console
unmounts the editor on every pane swap. Its `window` and `document` listeners kept each detached
editor alive: ~5 MB per description toggle in WebKit, linear, which is the growth that reached
1.5 GB. `AppMarkdownEditor.vue` now destroys the view in `onBeforeUnmount`, and
`tests/unit/AppMarkdownEditor.lifecycle.spec.ts` guards it. Everything below is measured with that
fix in place.

## How it was measured

- One isolated instance on port 47399, a throwaway database (the soak ran against the JVM build
  of the server that `rekall-server` replaced; the console it measures is the same code), 4 companies / 12 projects / 120
  tasks / 120 notes from `scripts/seed-demo-data.py`, every project pointed at a scratch git
  repository, and `rekall.claude.cli-path` pointed at a stub shell script that echoes coloured
  output, so terminal sessions run a real PTY without Claude.
- Playwright drives Chrome (for the numbers: CDP `HeapProfiler.collectGarbage` twice, then
  `Performance.getMetrics` for `Nodes`, `JSEventListeners`, `JSHeapUsedSize`) and Playwright's
  WebKit (for the resident size the app really runs in; WebKit exposes no forced GC, so only RSS
  is read there). Renderer RSS comes from `ps`, following the launched browser's process tree.
- Backend: RSS from `ps`, heap from `/actuator/metrics/jvm.memory.used`.
- Three rules, each learned the hard way: **one `page.goto()`** at the start and every screen
  change through the nav switcher, since a reload resets the heap and hides everything; **no
  `waitFor*` or `page.$$` inside the measured loop**, since each leaves a Playwright handle that
  pins DOM and looks exactly like a leak; sync on fixed sleeps or the REST API instead. In the
  tables, `attached elements` is `document.getElementsByTagName('*').length`, so growth in
  `DOM nodes` that tracks it is visible content, not garbage.

## Each surface on its own (Chrome, forced GC before every reading)

### Navigator: selecting tasks (`j`/`k`), 130 new, 130 back, 130 again, then 40 clamped

| point | DOM nodes | listeners | JS heap (MB) | attached elements |
|---|---:|---:|---:|---:|
| after first select | 3139 | 418 | 8.94 | 2026 |
| 70 new tasks | 3657 | 464 | 10.84 | 2343 |
| 130 new tasks | 3859 | 478 | 11.08 | 2462 |
| 130 back (revisit) | 3851 | 478 | 11.23 | 2459 |
| 130 forward again (revisit) | 3860 | 478 | 11.40 | 2462 |
| 40 presses past the end | 3859 | 478 | 11.40 | 2462 |

The growth during the first walk is attached content: walking into a DONE task opens its project's
filing drawer (`revealedProjectIds` in `NavigatorPane.vue`) and the drawer stays open, so more rows
are rendered. The node/element ratio is constant, revisits are flat, and presses past the end of
the list change nothing. Bounded by the number of projects, not by use. This closes the
"selecting a never-seen task leaks" hypothesis from the earlier investigation, which was the
Playwright handle artefact above plus this drawer behaviour.

### Panes: `d d s s w w c c`, 60 cycles (480 editor mounts)

| point | DOM nodes | listeners | JS heap (MB) | attached elements | renderer RSS (MB) |
|---|---:|---:|---:|---:|---:|
| baseline | 3157 | 418 | 9.49 | 2031 | 237.9 |
| 10 cycles | 3155 | 418 | 11.45 | 2031 | 249.5 |
| 30 cycles | 3155 | 418 | 11.71 | 2031 | 256.9 |
| 60 cycles | 3155 | 418 | 12.03 | 2031 | 266.2 |

Zero nodes and zero listeners per mount. The heap's +2.5 MB is JIT warm-up and flattens.

### Notes browse mode (`b`, walk notes, `b`), 40 cycles

| point | DOM nodes | listeners | JS heap (MB) | attached elements |
|---|---:|---:|---:|---:|
| baseline | 3155 | 418 | 12.03 | 2031 |
| 20 cycles | 3299 | 430 | 12.63 | 2118 |
| 40 cycles | 3387 | 438 | 12.77 | 2173 |

Same shape as the task walk: the note walk selects new tasks, drawers open, content grows, ratio
constant.

### Status keys `3 2 4 1` (task moves between navigator groups), 40 cycles

Flat: 3387 nodes, 438 listeners, heap 12.77 → 12.90 MB.

### Dialogs (settings, notes picker, time log, scope picker, filing drawer), 40 cycles

Flat from the first reading: 1215 nodes, 279 listeners, heap 11.6 → 12.1 MB (the drop from the
baseline is the scope narrowing to one project, which the phase leaves in place).

### Timer start/stop with the shell dock opened and closed, 30 cycles

Flat: 1222 nodes, 279 listeners, heap 12.1 → 12.25 MB.

### Routes (Console → Projects → Companies → Calendar → Report → Console), 40 round trips

| point | DOM nodes | listeners | JS heap (MB) | attached elements | renderer RSS (MB) |
|---|---:|---:|---:|---:|---:|
| baseline | 3157 | 418 | 9.51 | 2031 | 241.6 |
| 10 | 3165 | 431 | 12.67 | 2040 | 272.3 |
| 40 | 3165 | 431 | 13.00 | 2040 | 284.5 |

The one-time +8 nodes / +13 listeners is the SPA's first client-side navigation. Then flat.

### Catalog (project cards → detail, companies), 30 cycles

Flat: 3165 nodes, 431 listeners, heap 13.0 → 13.7 MB.

### Report (week/month, prev/next/current, steps toggle, company chip), 30 cycles

Flat at 6261 nodes / 163 listeners while the report is on screen, heap 13.3 → 13.5 MB.

### Calendar (prev/next/today, day detail dialog), 30 cycles

Flat: 1083 nodes, 78 listeners, heap 12.5 → 13.0 MB.

### Description editing (write mode, type a line, autosave, read mode), 20 edits

Flat DOM (3159 nodes, 431 listeners); heap 12.5 → 13.6 MB, which is the description itself
growing by one line per edit plus JIT.

### Steps (add a draft step, hide-done toggle, pane on/off), 20 steps

Flat: 3160 nodes, 431 listeners, heap 13.7 → 13.8 MB.

### Terminal sessions (open a PTY, type, unmount the pane with the socket live, remount, type, close), 30 sessions

| point | DOM nodes | listeners | JS heap (MB) | attached elements | live PTYs |
|---|---:|---:|---:|---:|---:|
| baseline | 3157 | 418 | 9.49 | 2033 | 0 |
| 15 sessions | 3260 | 440 | 13.63 | 2090 | 0 |
| 30 sessions | 3330 | 446 | 13.94 | 2133 | 0 |
| settle (description pane) | 3034 | 296 | 13.66 | 1943 | 0 |

60 xterm mounts, 30 WebSockets opened and closed, every child process reaped (`ps` shows no
stub left, `/api/terminals` reports 0 live), no ERROR or WARN in the server log. The attached
growth is again the task walk between sessions.

## Everything at once, for a long time

One page load, then every iteration does: task walk and notes mode, all four panes, two status
changes, three dialogs, timer start/stop with the dock, a full terminal session with typed input,
a project detail, the companies page, the calendar with a day dialog, the report with period
shifts and the steps toggle, and back to the console.

### Chrome, 80 iterations in 16 minutes

| point | min | DOM nodes | listeners | JS heap (MB) | attached | renderer RSS (MB) | backend RSS (MB) | backend heap (MB) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| start | 0.0 | 3157 | 418 | 9.48 | 2033 | 241.5 | 546.3 | 139.2 |
| it8 | 1.6 | 2886 | 282 | 14.62 | 1860 | 310.4 | 554.8 | 123.9 |
| it15 | 3.0 | 2908 | 281 | 15.40 | 1864 | 316.6 | 553.6 | 77.1 |
| it23 | 4.5 | 2957 | 285 | 14.92 | 1915 | 299.0 | 525.8 | 133.1 |
| it30 | 5.9 | 2967 | 290 | 15.46 | 1911 | 285.4 | 489.5 | 92.4 |
| it38 | 7.5 | 3059 | 297 | 15.59 | 1965 | 293.4 | 523.8 | 164.4 |
| it45 | 8.9 | 3056 | 297 | 15.66 | 1966 | 298.3 | 515.1 | 115.8 |
| it53 | 10.4 | 3092 | 298 | 15.71 | 1979 | 304.6 | 495.7 | 82.5 |
| it60 | 11.8 | 3084 | 297 | 15.80 | 1973 | 308.7 | 495.8 | 138.5 |
| it68 | 13.4 | 3050 | 297 | 15.86 | 1964 | 311.0 | 523.1 | 98.6 |
| it75 | 14.8 | 3098 | 299 | 15.88 | 1985 | 312.7 | 496.8 | 154.6 |
| settle | 16.2 | 3089 | 297 | 15.89 | 1975 | 301.9 | 499.1 | 113.5 |

JS heap: +6 MB in the first three minutes (JIT, caches), then +0.5 MB over the next thirteen.
DOM and listeners flat. Renderer RSS moves between 285 and 320 MB with no direction. Zero page
errors. Backend RSS 490–557 MB with no trend, heap a normal G1 sawtooth between 65 and 165 MB.

### WebKit (the engine `Rekall.app` runs in)

Same driver, 63 iterations in 20 minutes, machine kept awake with `caffeinate`, then three
minutes idle. No forced GC is available, so this is the resident size of the content process,
the number Activity Monitor shows.

| point | min | attached | WebContent RSS (MB) | backend RSS (MB) | backend heap (MB) |
|---|---:|---:|---:|---:|---:|
| start | 0.0 | 2181 | 222.0 | 533.4 | 134.9 |
| it5 | 1.6 | 2019 | 405.4 | 441.6 | 89.9 |
| it10 | 3.2 | 2019 | 479.7 | 442.2 | 121.9 |
| it15 | 4.8 | 2014 | 485.5 | 500.0 | 59.8 |
| it21 | 6.7 | 2014 | 494.5 | 506.8 | 91.8 |
| it26 | 8.3 | 2019 | 497.5 | 491.5 | 123.8 |
| it31 | 9.9 | 2019 | 464.7 | 512.0 | 68.0 |
| it36 | 11.5 | 2019 | 498.9 | 427.8 | 100.0 |
| it41 | 13.1 | 2005 | 497.7 | 443.1 | 124.0 |
| it46 | 14.6 | 2024 | 501.5 | 474.4 | 148.0 |
| it52 | 16.6 | 2004 | 457.5 | 449.5 | 92.3 |
| it57 | 18.2 | 2004 | 450.8 | 456.1 | 108.3 |
| it63 (settle) | 20.0 | 2024 | 448.9 | 446.9 | 148.3 |
| idle 3 min | 23.2 | 2024 | 438.3 | 409.8 | 148.3 |

WebKit reaches its working size in about three minutes (JSC heap sizing, decoded fonts, style
and layout caches), then moves between 450 and 500 MB for the rest of the run with no direction:
it is lower at the end than at minute five. Left idle it comes down further. An earlier
17-minute run of the same driver on the previous build ended at 535 MB after 50 iterations with
the same shape. Two further runs were cut by the machine sleeping mid-run and are not reported,
except for one reading they share: three minutes of idle after the soak changes RSS by 0 MB, so
nothing is being released or accumulated in the background.

For scale: the 1.5 GB reading came from the CodeMirror leak at ~5 MB per description toggle.
At the numbers above, a full day of continuous use stays where a few minutes of use puts it.

## Optimisations

- `TerminalPane.vue` is loaded on demand (`defineAsyncComponent` in `App.vue`): xterm and its
  stylesheet are no longer part of the console's initial bundle and are only parsed once a
  terminal is opened. Sessions that never open one never pay for it.
- Nothing else was worth changing. The JS heap is 10–16 MB; the rest of the process is the
  browser engine's own baseline (fonts, style, layout, compositor), which no app code shrinks.
  `highlight.js` is already the `lib/common` subset.

## What to watch

- The navigator keeps every filing drawer you have walked into open for the session. That is
  content, bounded by the number of projects, and is by design; it is the only "growth" the
  numbers show.
- The renderer's RSS is not the JS heap. It rises and falls with what is on screen and with the
  engine's allocator; a reading that has grown since launch is not a leak unless the heap after
  forced GC, the node count and the listener count grow with it.
