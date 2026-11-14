---
name: Rekall
description: A dark ledger for driving Claude Code sessions: amber for what you do next, cyan for what you copy, status colours only for the state of a record.
colors:
  canvas: "#0d1017"
  surface: "#151922"
  surface-raised: "#1d2330"
  surface-hover: "#262d3c"
  border: "#2a3243"
  border-strong: "#3b4558"
  border-field: "#5e6a82"
  text: "#f2f5f9"
  text-muted: "#a3acba"
  text-subtle: "#8c96a6"
  accent: "#eaa43c"
  accent-strong: "#ffd88e"
  accent-deep: "#a8651a"
  accent-ink: "#2b1804"
  anchor: "#55c9ea"
  anchor-strong: "#a9ecfa"
  session: "#7c83f7"
  session-strong: "#b4b9ff"
  safe: "#3ddc97"
  warn: "#f3c04a"
  danger: "#fb7185"
  filed: "#8fa89d"
  epic: "#a99bff"
typography:
  display:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "26px"
    fontWeight: 500
  heading:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "19px"
    fontWeight: 500
  title:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "15px"
    fontWeight: 600
  body:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "13.5px"
    fontWeight: 400
  small:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "12.5px"
    fontWeight: 400
  caption:
    fontFamily: "Fira Sans, ui-sans-serif, sans-serif"
    fontSize: "11.5px"
    fontWeight: 500
  anchor:
    fontFamily: "Fira Code Variable, ui-monospace, monospace"
    fontSize: "11.5px"
    fontWeight: 400
rounded:
  key: "3px"
  control: "5px"
  card: "8px"
  full: "9999px"
spacing:
  row: "36px"
  control: "36px"
  header: "48px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    rounded: "{rounded.control}"
    height: "36px"
    padding: "0 14px"
  button-secondary:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    height: "36px"
    padding: "0 14px"
  status-pill-review:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.accent-ink}"
    rounded: "{rounded.full}"
    height: "22px"
    padding: "0 8px"
  anchor-chip:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.anchor}"
    rounded: "{rounded.key}"
  field:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    height: "36px"
  header-capsule:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.card}"
    height: "36px"
---

# Design System: Rekall

## Overview

**Creative North Star: "The Night Ledger"**

Rekall is a ledger kept at night beside a terminal. Everything is a row in a table that answers one question, what waits for me, and the screen is dark so it does not compete with the session next to it. Colour is rationed: brushed gold marks where you are and what you do next, a single cool cyan marks the strings you copy, and green, yellow and red mean the state of a record and nothing else. A screen is calm until something needs a person, and then exactly one thing is amber.

The density is that of a working tool: 13.5px body text, 36px rows and controls on a 4px rhythm, a header of 48px. Surfaces climb out of a deep blue-slate well in small steps and are lit from one source, top left, with a bright upper lip and a contact shadow. The product has a small amount of physical character (the gold tube that fills while a session works, the step seal that closes on acceptance, the pulse of the session orb) and all of it is state made visible, never decoration.

**Key Characteristics:**
- Dark only; the world is a blue-slate well with brushed-gold and cyan light.
- Three roles carry all colour: amber (here, next), cyan (anchors), status (record state).
- Tables and tabs over cards and panels; the status pill, the plan tube and the copy-anchor chip are the vocabulary every screen shares.
- Motion shows a state change or a session at work, and Reduced motion keeps every change.

## Colors

A blue-slate ground, warm gold for action, one cool cyan for anchors, and status colours held back for the state of a record.

### Primary
- **Brushed Gold** (#eaa43c): the selected tab's rule, the primary key, focus rings, the review count, the plan tube while a session works. Its lip (#ffd88e) and shade (#a8651a) build the metal gradient; ink on gold is #2b1804.

### Secondary
- **Anchor Cyan** (#55c9ea): `project:vega`, `task:RK-12` and every other string the product exists to hand you; nothing else is cyan. Strong (#a9ecfa) for the current chip.
- **Session Indigo** (#7c83f7): a session at work and the work it handed in: the orb, the running comet, the claimed check.

### Tertiary
- **Safe Green** (#3ddc97), **Warn Yellow** (#f3c04a), **Danger Rose** (#fb7185): only the state of a record or a risk. Danger is also a risk that exists whatever you do (a critical advisory), so it is not amber.
- **Filed Verdigris** (#8fa89d): work that is done and put away, an archive that has not gone dim.
- **Epic Periwinkle** (#a99bff): epics, always with the stacked glyph.

### Neutral
- **Deep Well** (#0d1017): the canvas. **Slate Surface** (#151922), **Raised Slate** (#1d2330) and **Hover Slate** (#262d3c): the climb of surfaces.
- **Hairline** (#2a3243) and **Strong Hairline** (#3b4558): borders; **Field Edge** (#5e6a82) is the edge of anything you type into, 3:1 against its well.
- **Text** (#f2f5f9), **Muted** (#a3acba), **Subtle** (#8c96a6): 17.4:1, 8.3:1 and 6.4:1 on the canvas. Subtle is the floor for small chrome and is never faded further.

### Named Rules
**The One Amber Rule.** On any screen exactly one thing is asking to be done, and it is the amber one. The review count is the only amber segment of the header capsule.
**The Cyan Is Copyable Rule.** Cyan means an anchor you can paste after `/rk`. Do not use it for links, selection or decoration.
**The State Is Not Style Rule.** Green, yellow and red describe a record. They never decorate a chart series, a tab or a button.

## Typography

**Display and Body Font:** Fira Sans (with the system sans as fallback)
**Label and Anchor Font:** Fira Code Variable (with the system mono as fallback)

**Character:** One family designed together, so a page full of identifiers reads as one document: the interface speaks in Fira Sans and the strings you copy speak in Fira Code at the same proportions. Bundled, never fetched.

### Hierarchy
- **Hero** (34px): the one big figure on a screen, the tracked time.
- **Display** (26px, 500): a page's own figure or title where a page has one.
- **Heading** (19px, 500): the title of a screen or a section of Settings.
- **Title** (15px, 600): a card, a dialog, a panel.
- **Body** (13.5px, 400): rows, descriptions and prose; lines held near 65 to 75 characters in reading panes.
- **Small and Caption** (12.5px and 11.5px): meta, table headers, tab labels and counts. **Micro** (11px) is the floor, for kbd hints and counts, never prose.
- **Anchor** (Fira Code, 11.5px to 12.5px): every anchor and code.

### Named Rules
**The Eight Steps Rule.** The scale has eight sizes and none between them; a bracketed pixel size is a bug and `design-tokens.spec.ts` fails on it.
**The Subtle Floor Rule.** Text never goes below Subtle in contrast, and an opacity modifier is never used to quieten it.

## Layout

Screens are a title and a sub-navigation over a table, a list with a detail pane, or a queue beside a decision. Every page sits in one centred column of 1320px with 32px side padding (`page-column`), so a screen keeps its left edge when you move to its sub-navigation neighbour; a list with a detail pane is a bordered card inside that column, not a full-bleed workspace. Work's board and Review use the width they have. The header is one 48px row on a three-cell grid: the menu key and history arrows, the `/rk` search at the exact centre (240px to 620px), and the status capsule with tags and settings. Rows and controls share a 36px height on a 4px rhythm; the Work ledger offers Comfortable and Compact. Under 1280px the header sheds its words before it moves anything, and under 1100px the ⌘K hint goes. There is no mobile layout.

## Elevation & Depth

Hybrid: tonal layering for structure, light for state. Surfaces step up through slate and are separated by hairlines, not shadows. Light comes from the top left everywhere, so raised things carry a bright upper lip and a contact shadow, and sunk things (fields, tubes) an inner shade.

### Shadow Vocabulary
- **Raised** (`inset 0 1px 0 rgb(255 255 255 / 0.05), 0 1px 2px rgb(0 0 0 / 0.35), 0 6px 12px -8px rgb(0 0 0 / 0.45)`): a card or a panel standing on the canvas.
- **Key** (`inset 0 1px 0 rgb(255 255 255 / 0.07), 0 1px 1px rgb(0 0 0 / 0.35)`): a pressable key; pressed is `inset 0 1px 2px rgb(0 0 0 / 0.35)`.
- **Well** (`inset 0 2px 4px rgb(0 0 0 / 0.5), 0 1px 0 rgb(255 255 255 / 0.035)`): a field, a tube trough.
- **Lift** (`0 10px 22px -8px rgb(0 0 0 / 0.65)` with the lit lip): a popover or dock.
- **Modal** (`0 40px 80px -24px rgb(0 0 0 / 0.9)`): dialogs and sheets.
- **Glow** (`0 0 28px -6px` in soft gold): the one halo, for a session at work.

### Named Rules
**The One Light Rule.** One light source, top left, on every surface. A shadow that falls the other way is wrong.

## Shapes

Three corners and a pill: 3px for a keycap or chip, 5px for a control or a row, 8px for a card or a dialog, and a full radius for status pills, the plan tube and the dock. Borders are one pixel hairlines; a thick coloured edge on a rounded card is not used (the keycap's thicker bottom edge is a key, not an accent border).

## Components

### Buttons
- **Shape:** 5px corners, 36px high (28px small), Fira Sans 13.5px.
- **Primary:** the gold key (accent metal, ink text, semibold); one per screen.
- **Secondary:** slate key on Raised Slate; **Quiet** and **Ghost** have no fill until hover; **Danger** turns rose and **Danger-quiet** stays ghost until hovered, for a delete repeated down a list.
- **Hover / Focus:** colour changes at 140ms; a 2px amber outline with a 2px offset on keyboard focus (`focus-ring`).

### Status Pill
A 22px full-radius pill with one word, the state of a record: neutral, progress (soft gold), review (solid gold), done (green), filed (verdigris), running (indigo with a pulsing orb) or danger.

### Anchor Chip
Fira Code in cyan on the surface well with a hairline, 3px corners. A copy button beside it copies `/rk` followed by the anchor; the current anchor is the bright one.

### Sub-navigation
Tabs under a title with a count each, a gold rule that scales in under the current one, and an amber pill for a count that waits on the reader. It is the same component for Work, the task page, Projects, Time, Library and the project page, as links.

### Fields
Recessed: a 1px Field Edge border over a dark well with an inner shade. Focus warms the hairline and opens a soft halo; it never stacks a second ring. Invalid turns the edge rose.

### Header Capsule
A 36px surface capsule with a hairline, holding usage, the run queue and the review count as segments; review is the only amber one.

### Plan Tube and Step Seal
The plan tube is a glossy gold fill in a sunk trough, green when complete, with a sheen while a session works. The step seal is a ring that stays open on a claimed step and closes on hover, ripples and pops once on acceptance, and shows a comet and ticks while running. Only a person marks a step done.

### Dock
One capsule at the bottom left (`1 in progress | 1 session`) that opens a panel above itself; one panel at a time. A terminal session picked from its sessions sheet opens in a floating terminal panel above the capsule instead of the full terminal pane; Escape and outside presses go to the terminal, not to the panel.

## Do's and Don'ts

### Do:
- **Do** keep exactly one amber thing asking for action on a screen, and let the review count be the only amber segment of the header.
- **Do** show an anchor as a cyan Fira Code chip with a copy key, wherever a task, project or note is named.
- **Do** show a project's company beside its name wherever the name could be ambiguous.
- **Do** read sizes, radii, durations and colours from the tokens in `main.css`; the 11px floor and the three corners are the whole scale.
- **Do** make a state change visible without motion: the seal, the pill and the tube must still end in their new state under Reduced motion.

### Don't:
- **Don't** introduce a palette, a typeface or a new token family; a redesign changes structure and hierarchy, never the world.
- **Don't** use amber, cyan or the status colours for decoration, series colours or selection that is not "here".
- **Don't** add a light theme or a mobile layout; both are out of scope.
- **Don't** put a bracketed font size, a bracketed radius or a numeric `duration-*` in a component; the tokens spec fails on them.
- **Don't** let Claude's work read as accepted: only the console marks a step done, and the UI must make the claimed state legible as waiting.
