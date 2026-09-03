---
description: Load a Rekall working context by anchor, e.g. /rk project:vega task:report-builder
argument-hint: "company:|project:|task: <label> ...  [wrapup [\"how to write it\"]]   (labels, not titles)"
allowed-tools: mcp__rekall__rekall_context, mcp__rekall__rekall_wrapup
---

The arguments are:

$ARGUMENTS

If the terms include the bare word `wrapup`, follow **Wrapping up**. Otherwise follow **Loading**.

## Loading

Call `rekall_context` with `anchors` set to exactly the arguments above, unchanged.

- Do not call any other tool first. The anchors are already qualified, there is nothing to look up.
- The entities are `company`, `project` and `task`. If one was wrong, say which and stop.
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

You cannot tick a step, and no tool here can. When you finish one, say which, in the words the step
uses, so I can tick it in the console.

Anchors that name a project and no task have no brief in them. Summarise what is there and wait.

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
  overwriting a wrapup edited by hand is announced because nothing kept a copy of it.

Small enough to read in one go. Short paragraphs or short bullets, not an essay and not an index.

Send it whole. Nothing is merged and no previous version is kept, so keep whatever is still true
from the wrapup you just read and rewrite the rest. Describing only the part you touched would leave
the task claiming to be a fraction of itself.

If the tool answers that the version you replaced had been edited by hand, tell me: those were my
words and nothing kept a copy.
