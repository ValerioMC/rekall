---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  [wrapup [\"how to write it\"]] | [note \"what to keep\"] | [plan] | [step:N start|done]   (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_wrapup, mcp__rekall__rekall_step, mcp__rekall__rekall_propose_step, mcp__rekall__rekall_record_commit, mcp__rekall__rekall_note
---

The arguments are:

$ARGUMENTS

If the terms include the bare word `wrapup`, follow **Wrapping up**. If they include the bare word
`note`, follow **Noting**. If they include the bare word `plan`, follow **Planning**. If they include
a `step:` term alongside `start` or `done`, follow **Stepping**. Otherwise follow **Loading**.

## Loading

Call `rekall_context` with `anchors` set to exactly the arguments above, unchanged.

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `company`, `project` and `task`. If one was wrong, say which and stop.
- A note that arrives `loaded="on request"` is a title, a line and a `note:` anchor. Load it with
  `rekall_context` and that anchor only if the work needs it.
- The value is the record's **label**, never its title: `project:vega`, not `project:"Vega Platform"`.
  A label is lowercase and has no spaces. If I gave you a title, say so and stop rather than guessing
  the label from it.
- If a term matched more than one record, show me the candidates and stop. Do not choose.

On success, summarise in no more than ten lines: what the task is, its status, what its wrapup says
the implementation currently is, which steps are still open, and anything in its notes that changes
how the work should be done.

Then get on with it. What you build comes from one of two places, and whether the task has a
checklist is what decides which.

**With open steps, the steps are the work and the description is not.** Read the description for
what the task is for, what the work has to satisfy, what is out of scope and what shape the code has
to take, and then build what the open steps say, in the way the description requires. It is context
you hold while implementing a step, not a list of things to implement.

Nothing that only the description asks for gets built in a session with a checklist, however plainly
it is phrased. A description is written once and does not shrink as the work is done, so it goes on
naming things that are finished, and treating it as a list of instructions is how the same thing is
built twice. If something in it still needs doing and no open step covers it, say so in one line and
ask me for the step. Do not add it to what you are building.

**With no steps at all, the description is the brief and the instruction both.** It is where the
work is written down, and it was written so that it would not have to be said again: take it as the
instruction it is, say in one line what you are about to do, and start. A question in it is a
question to answer, not one to hand back.

Ask only when the context does not settle it:

- there is no description and no step, or what they hold is background rather than an instruction
- an open step and the description ask for two things that cannot both be true, or one of them names
  something that is not there
- the description asks for work no open step covers and you think it has to happen now
- doing it would delete or overwrite something nothing keeps a copy of

Then ask exactly what is missing, in one question, and stop. Nothing already written down is worth a
question: if it is in the description, a step, the wrapup or a note, it has been answered once
already.

### The checklist

If the task carries steps, that is the plan and it decides what you do next.

- The open ones are the work, in the order they are listed, and each arrives with the detail of what
  it has to do. Start on the first one that is open.
- The done ones arrive as a title and nothing else. They are finished: do not build them again, and
  do not go looking for the detail that is not there.
- Where the checklist and the wrapup disagree, the checklist is the one a person ticked. A step that
  is ticked is done however the description still phrases it, and that is not a contradiction worth
  asking me about.

You cannot tick a step **done**, and no tool here can: that is mine to do in the console once I have
looked at the work. What you can do, while you work a checklist, is move a step to **running** and
then **claimed** with `/rk ... step:N start` and `/rk ... step:N done` (see **Stepping**). When you
claim one, say which, in the words the step uses.

Anchors that name a project and no task have no brief in them. Summarise what is there and wait.

### What the work produces

When a step or the description asks for something to be kept as a note (an analysis, a list, a
draft, a runbook, "put the result in a note"), or the task exists to write one, that output goes
to `rekall_note` on this task, not into the wrapup. The wrapup can say the note exists; it does not
carry it. See **Noting**.

## Stepping

`/rk project:vega task:report-builder step:3 start` and `/rk project:vega task:report-builder step:3
done` are how a session drives its own checklist as it works. `step:` takes the step's number as
`rekall_context` lists them, or its exact title.

- `start` marks that step **running**: call `rekall_step` with `state` `running`.
- `done` marks it **claimed**: call `rekall_step` with `state` `claimed`. It does not tick the box.
  Only I do that, in the console, once I have looked at the work.

The loop, once you are working a checklist:

1. `/rk project:x task:y step:N start`, before you touch the code for step N.
2. Do the work the step's detail describes.
3. `/rk project:x task:y wrapup`, to fold what that step built into the task's wrapup, by the rules
   in **Wrapping up**. One state, rewritten whole, never a section per step.
4. `/rk project:x task:y step:N done`, to claim it.
5. If a later step is still open, `/rk project:x task:y step:M start` and go again. When none is,
   stop and tell me the checklist is claimed and waiting for my review.

`rekall_step` refuses `done` as a state, and refuses a step I have already accepted. If it comes
back with either, say so and stop.

If the project's context carries an `auto-commit` field, the claim is what commits: `step:N done`
(and, on a task with no checklist, the wrapup) stages and commits the folder and logs the commit
itself. Do not `git commit` on such a project; read the answer for what it committed, or why not.

On such a project, pass `commit_message` with the claim (with `rekall_wrapup` on a task with no
checklist), because it is the commit's message:

- The first line is a Conventional Commits subject under 72 characters, `feat: …` or `fix: …`, saying
  what the work delivers, not the step's number.
- Then a blank line and two to five sentences of plain prose: what the change does, why, and anything
  a reader of `git log` should know. No list of files, no "this session", no anchors: Rekall adds a
  `Refs:` line with the task and step itself.

Without it, Rekall derives the message from the step's title and detail, which is the fallback and
reads like one.

## Noting

`/rk project:vega task:report-builder note "the export formats we settled on"` means: write a new
note on that task and attach it there.

The text in double quotes after `note` says what the note has to hold, in my words. It is not an
anchor and not the note's text: it is what to put in it. Without one, the note holds what this
session just produced that I would want again.

1. Drop the `note` term and the quoted text, and call `rekall_context` with the anchors that are
   left, unless that exact task is already loaded in this session.
2. Write the note from what the quoted text asks for: the output of a step, a piece of the
   description worked out, something the session found. Written to be read on its own by someone
   who was not here, in markdown.
3. Call `rekall_note` with those anchors, a short `title` named like a file (`export-formats.md`)
   and the whole note as `body`.
4. Say in one line which note you wrote and its `note:` anchor. Then stop.

`rekall_note` only adds. A title the task's notes already carry is refused rather than
overwritten: pick another title, and tell me if it was the old note that needed changing, because
only the console can do that. A note is not the wrapup, and writing one does not claim the task or
a step.

## Planning

`/rk project:vega task:report-builder plan` means: turn the task into a checklist I can review.

1. Drop the `plan` term and call `rekall_context` with the anchors that are left.
2. Read what it gives you: the description says what the work is for and what it has to satisfy,
   the wrapup what already exists, the done steps what is finished. Then read the code the task
   touches, enough to know where each piece will go.
3. Call `rekall_propose_step` once per step, in the order the work should be done. A title says
   what the step delivers; the detail says what to build, where, what it must satisfy and how I can
   tell it is done, enough for a session that never saw this one to do it alone. Propose only what
   is still missing, and nothing the description rules out.
4. Stop. Say in a few lines how many drafts you proposed and anything you left out on purpose.
   Build nothing: the drafts are not work until I promote them in the console.

A title the task already has is refused, so a second `plan` adds only what the first one missed.

## Wrapping up

`/rk project:vega task:report-builder wrapup` means: record what that task's implementation looks
like **now**, replacing what was there.

Anything in double quotes after `wrapup` is a **directive**: what I want the wrapup to say, in my
words, not an anchor. `/rk project:vega task:report-builder wrapup "solo il modulo di export"`.

1. Drop the `wrapup` term and the directive, and call `rekall_context` with the anchors that are
   left, so you are working from the current wrapup and notes rather than from memory of this
   session. Skip this only if you already loaded that exact task in this session and nothing has
   changed since.
2. Call `rekall_wrapup` with those same anchors and the complete new text as `body`. The anchors
   string never carries the directive.
3. Say in one line what you wrote and that it replaced what was there. If a directive left something
   out that the previous wrapup had, say what. If the task has a checklist, name the open steps this
   session finished, so I can tick them: nothing you can call will do it for you. Then stop.

### The directive

Without one, the wrapup is yours to write: the implementation as this session and the code leave it,
by the rule the tool states.

With one, it decides the content and you do not go past it. It can narrow the subject, dictate the
words, set the language or the length, or say what to leave out. Take it literally.

- "write only what I am telling you" means the body carries what I dictated and nothing you inferred
  from the code. What I did not mention is gone, and step 3 is where you tell me so.
- A directive that names a subject, `"solo il modulo di export"`, narrows what you write about. What
  the rest of the wrapup already says stays true and stays in, unless I said to drop it.
- It never changes the shape. Still one state and not a changelog, still the whole text in one call,
  still short enough to read on a screen. If what I dictate is phrased as a change I made, record
  what that change leaves the system as, not the making of it.

The anchors have to name exactly one task. A `project:` anchor on its own names forty and is refused.

### Steps that closed since the last one

A wrapup written after a step is finished has to account for it, and that is the whole reason to
write one there. Read the current wrapup, keep every sentence still true, and rewrite the rest so
the text describes the system with that step's work in it. The result is one description of the
whole task, not the previous wrapup with a paragraph stuck on the end.

You do not have to work out which ones those are. The context marks every finished step the wrapup
predates with `(finished since the wrapup was written)` and hands back its detail, which it does for
no other closed step. Those are exactly what the current text is missing, and it is often more than
the one step this session closed: a step ticked in an earlier session that never got a wrapup is
still marked, and it goes in too. Do not stop at the piece you just built.

- Yes: the piece the step built is now part of what the wrapup says the task is, named where it
  lives, next to what was already there.
- No: a section per step, a heading with the step's title on it, "then I did", anything that lets a
  reader reconstruct the order the pieces arrived in. That is a changelog, and it is what this
  replaces.

If the step's work made something the wrapup already said untrue, the old sentence goes. Keeping
both is how a wrapup starts contradicting itself.

**Write the state, not the session.** The wrapup describes the system as it stands, for a reader who
was not here and does not care what it looked like before.

- Yes: what exists, what it does, how the parts fit, which decisions are settled, what is still open,
  where the sharp edges are.
- No: "added", "changed", "now also", "previously", "fixed", "refactored", "before/after", anything
  dated, anything phrased as a step you took, anything that reads as a changelog entry.

If a sentence only makes sense to someone who watched the change happen, it does not belong in the
wrapup. That is what the notes are for.

**Name the code, do not transcribe it.** It is read next to the repository, so it has to say where
things are: the class, the file, the endpoint, the table, the component, by the name they have.
A line or two on each piece, what it is there for and what it decides.

- Yes: `WrapupService` is the whole write path and the only thing that touches the wrapup row.
- No: the fields of an object, the columns of a table, method signatures, parameter lists, a
  directory tree. That is in the code, it is longer than the wrapup, and it is wrong a week later.
- Where there is a rule, the rule is the point. What it decides, on what, what happens at the edges,
  what is refused and why. A class name says there is a service; only the wrapup says that
  overwriting a wrapup edited by hand is announced because no session reads the history it went to.

Small enough to read in one go. Short paragraphs or short bullets, not an essay and not an index.

Send it whole. Nothing is merged, and the version it replaces goes to a history only I read, so keep
whatever is still true from the wrapup you just read and rewrite the rest. Describing only the part
you touched would leave the task claiming to be a fraction of itself.

If the tool answers that the version you replaced had been edited by hand, tell me: those were my
words, and only the wrapup's History in the console still has them.
