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
   session finished, so I can tick them: nothing you can call will do it for you. Then stop. On a
   flow, do not ask me to tick anything: say the flow is finished and its claimed steps are waiting
   for review.

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
write one there. Write the text again so it describes the system with that step's work in it. The
result is one description of the whole task, not the previous wrapup with a paragraph stuck on the
end.

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

**Write it fresh.** A wrapup does not grow. It is rewritten from the code every time. Read the current
wrapup as a list of claims to check, not as a text to extend: a sentence stays only if the code
still makes it true and the next session needs it to start work. Open points that are closed,
decisions nobody will revisit and detail about finished, stable pieces all go. The new text still
covers the whole task, not only the part you touched.

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
  overwriting a wrapup edited by hand is announced, because only a person can read the text it
  replaced, from the Versions panel in the console.

**Shape it to be scanned.** The kind of information chooses the form:

| Information | Form |
|---|---|
| What the task delivers and where it stands | One or two sentences, first |
| The pieces of the implementation | A table: the name in the code, what it is for or decides |
| A rule | One bullet: what it decides, on what, the edges, what is refused |
| Results of something checked or tested | A table, one row per check, its outcome and what was seen |
| A flow or a lifecycle | A numbered list |
| What is open or fragile | Bullets under their own heading, last; left out when nothing is |

`###` headings separate those blocks and nothing else. Prose is only for what connects the parts.
It fits on one screen: about 300 words suits most tasks. Past 600 the tool says so, because the
text is carrying history or detail that belongs in a note.

If the tool answers that the version you replaced had been edited by hand, tell me: those were my
words, and only the Versions panel in the console still holds them.
