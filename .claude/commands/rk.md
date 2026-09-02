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
the implementation currently is, and anything in its notes that changes how the work should be done.

Then get on with it. The description is the brief: it is where the work is written down, and it was
written so that it would not have to be said again. Take it as the instruction it is, say in one line
what you are about to do, and start. A question in it is a question to answer, not one to hand back.

Ask only when the context does not settle it:

- there is no description, or what it holds is background rather than an instruction
- it asks for two things that cannot both be true, or names something that is not there
- doing it would delete or overwrite something nothing keeps a copy of

Then ask exactly what is missing, in one question, and stop. Nothing already written down is worth a
question: if it is in the description, the wrapup or a note, it has been answered once already.

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
   out that the previous wrapup had, say what. Then stop.

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
