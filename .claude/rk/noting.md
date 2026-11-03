## Noting

`/rk project:vega task:report-builder note "the export formats we settled on"` means: write a note
on that task and attach it there, or rewrite the one it already carries on that subject.

The text in double quotes after `note` says what the note has to hold, in my words. It is not an
anchor and not the note's text: it is what to put in it. Without one, the note holds what this
session just produced that I would want again.

1. Drop the `note` term and the quoted text, and call `rekall_context` with the anchors that are
   left, unless that exact task is already loaded in this session.
2. Write the note from what the quoted text asks for: the output of a step, a piece of the
   description worked out, something the session found. Written to be read on its own by someone
   who was not here, in markdown.
3. Call `rekall_note` with those anchors, a short `title` named like a file (`export-formats.md`)
   and the whole note as `body`. If the task already carries a note this one supersedes (the same
   subject, an older version of the same results), use that note's title and pass `replace: true`,
   so the task keeps one current note instead of two.
4. Say in one line which note you wrote or rewrote and its `note:` anchor. Then stop.

`replace` overwrites the body outright: the old text is not kept anywhere, and a note shared with
other tasks changes for them too, so send the whole note, not the part that changed. Without it, a
title the task's notes already carry is refused. A note is not the wrapup, and writing one does not
claim the task or a step.
